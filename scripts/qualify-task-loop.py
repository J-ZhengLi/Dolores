"""Opt-in live task-loop check using synthetic files and a launch-only connection.

Set DOLORES_TEST_BASE_URL and DOLORES_TEST_API_KEY in the environment. No keys,
endpoint, provider bodies or conversation transcripts enter the report.
"""
import argparse
import ctypes
import json
import os
import re
from pathlib import Path
import time
import uuid

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--model', required=True, help='Enabled model ID for this optional live check.')
    parser.add_argument('--mode', choices=['reads', 'command'], default='reads')
    args = parser.parse_args()
    base = os.environ.get('DOLORES_TEST_BASE_URL')
    if not base:
        raise SystemExit('Set DOLORES_TEST_BASE_URL for this opt-in live check.')
    directory = ROOT / 'output/task-loop-live' / str(uuid.uuid4())
    project = directory / 'project'
    project.mkdir(parents=True)
    os.environ['DOLORES_DATA_DIR'] = str(directory / 'data')
    release = ROOT / 'apps/dolores_flutter/build/windows/x64/runner/Release'
    loader = os.add_dll_directory(str(release)) if os.name == 'nt' else None
    native = ctypes.CDLL(str(release / 'dolores_flutter_bridge.dll'))
    native.dolores_call.argtypes = [ctypes.c_void_p, ctypes.c_size_t]
    native.dolores_call.restype = ctypes.c_void_p
    native.dolores_free.argtypes = [ctypes.c_void_p]

    def call(command, **fields):
        payload = json.dumps({'command': command, **fields}).encode()
        buffer = ctypes.create_string_buffer(payload)
        pointer = native.dolores_call(buffer, len(payload))
        try:
            result = json.loads(ctypes.string_at(pointer))
        finally:
            native.dolores_free(pointer)
        if not result['ok']:
            raise RuntimeError(result.get('error', 'Bridge failed.'))
        return result.get('result')

    report = {'mode': args.mode, 'model': args.model, 'ok': False}
    running = False
    try:
        call('configure', preferences={'baseUrl': base, 'model': args.model},
             apiKey=os.environ.get('DOLORES_TEST_API_KEY', ''), rememberConnection=False)
        call('setRequestSettings', settings={'maxOutputTokens': 1024 if args.mode == 'reads' else None,
                                          'timeoutSeconds': 90})
        policy = call('memories')['automaticPolicy']
        call('setAutomaticMemory', enabled=False, revision=policy['revision'])
        expected = {f'file-{n}.txt': f'PUBLIC-MARKER-{n}' for n in range(1, 7)}
        for filename, marker in expected.items():
            (project / filename).write_text(marker, encoding='utf-8')
        session = call('createSession', kind='project', path=str(project))['session']['id']
        script = '/*' + 'synthetic padding ' * 380 + '*/console.log("PUBLIC-CHECK-PASS")'
        prompt = ('Read file-1.txt through file-6.txt in numeric order using read_text_file, '
                  'one read per model response. Read all six before answering; do not use '
                  'other tools. Then report their six PUBLIC-MARKER values in one line.') if args.mode == 'reads' else (
                  'Use run_command exactly once with node and args -e plus this example JavaScript: ' + json.dumps(script) +
                  '. Aim for 380 repetitions in the comment; its count may vary, '
                  'but keep the script between 4097 and 8192 ASCII bytes. Preserve the executable console.log statement exactly. '
                  'No other tools or executable code. After its receipt, report the exit code and printed marker in one sentence.')
        tools, approvals, steps, thinking_events = [], 0, 0, 0
        started = time.monotonic()
        call('start', id=1, session=session, input=prompt)
        running = True
        done = None
        while time.monotonic() - started < 240 and done is None:
            for event in call('poll', id=1):
                if event['type'] == 'modelStep':
                    steps += 1
                    assert steps <= 12, 'Live check exceeded its test driver allowance.'
                if event['type'] == 'modelThinking':
                    thinking_events += 1
                if event['type'] == 'toolApproval':
                    request = event['request']
                    invocation = request.get('command', {}).get('invocation', {})
                    command_args = invocation.get('args', [])
                    allowed = (request['name'] == 'read_text_file' and request['target'] in expected) if args.mode == 'reads' else (
                        request['name'] == 'run_command' and invocation.get('program') == 'node'
                        and len(command_args) == 2 and command_args[0] == '-e'
                        and 4096 < len(command_args[1]) <= 8192
                        and re.fullmatch(r'/\*[A-Za-z \t\r\n]+\*/console\.log\("PUBLIC-CHECK-PASS"\)', command_args[1]) is not None)
                    call('approveTool', id=1, callId=request['callId'], allow=allowed)
                    approvals += int(allowed)
                if event['type'] == 'toolResult':
                    tools.append(event['record'])
                if event['type'] == 'done':
                    done = event
            time.sleep(.02)
        assert done is not None, 'Live check reached its test driver deadline.'
        running = False
        report.update(modelCalls=steps, toolCalls=len(tools), approvals=approvals,
                      statuses=[t['status'] for t in tools], thinkingEvents=thinking_events,
                      elapsedSeconds=round(time.monotonic() - started, 2))
        assert not done.get('error'), done.get('error')
        saved = call('messagesPage', session=session)['items'][-1]
        assert saved.get('metadata', {}).get('paused') is None, 'Task paused before completion.'
        if args.mode == 'reads':
            assert [t['target'] for t in tools] == list(expected), 'The six reads were not completed in order.'
            assert all(t['status'] == 'read' for t in tools)
            assert all(marker in saved['content'] for marker in expected.values())
            assert approvals == 6
        else:
            assert len(tools) == 1 and tools[0]['status'] == 'completed'
            outcome = json.loads(tools[0]['content'])
            assert outcome['exitCode'] == 0 and outcome['stdout'].strip() == 'PUBLIC-CHECK-PASS'
            assert not outcome['truncated'] and approvals == 1
            report['commandArgumentBytes'] = len(tools[0]['command']['args'][1].encode())
        report.update(ok=True, modelCalls=steps, toolCalls=len(tools), approvals=approvals,
                      thinkingEvents=thinking_events, elapsedSeconds=round(time.monotonic() - started, 2))
    except Exception as error:
        # Do not retain raw provider/exception data in test artifacts.
        report['failure'] = type(error).__name__
        raise
    finally:
        if running:
            call('cancel', id=1)
        call('shutdown')
        (directory / 'report.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
        print(json.dumps(report), flush=True)
        if loader is not None:
            loader.close()


if __name__ == '__main__':
    main()
