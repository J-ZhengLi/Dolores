"""Isolated normal-bridge sustained-task and supervised-command soak.

The local scripted provider creates verifiable files, forces a context rejection,
then supervises a real child for the requested wall-clock duration. This tests
the harness, not live-model reasoning or arbitrary feature quality. Use --seconds
7200 for a two-hour command lifetime; short runs do not establish that gate.
"""
import argparse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import sys
import threading
import time

from desktop_test_support import NativeHost, ROOT


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', required=True, type=Path)
    parser.add_argument('--seconds', type=int, default=10)
    parser.add_argument('--steps', type=int, default=160)
    args = parser.parse_args()
    directory = args.directory.resolve()
    if not args.directory.is_absolute() or not directory.is_relative_to(ROOT / 'output'):
        raise SystemExit('Fresh absolute disposable output/ directory required')
    if not 1 <= args.seconds <= 21_540 or not 80 <= args.steps <= 2000:
        raise SystemExit('Use 1–21540 seconds and 80–2000 fixture steps')
    directory.mkdir(parents=True, exist_ok=False)
    project = directory / 'project'
    project.mkdir()
    (project / 'AGENTS.md').write_text('Keep KEEP_EARLY_CONSTRAINT. Load applicable nested instructions before editing.',encoding='utf-8')
    (project / 'src/deep').mkdir(parents=True)
    (project / 'src/AGENTS.md').write_text('Nested source rule: preserve NESTED_SCOPE_RULE in new source files.',encoding='utf-8')
    child = project / 'soak-child.py'
    child.write_text(
        "import json,pathlib,sys,time\n"
        "seconds=int(sys.argv[1]); began=time.monotonic(); ticks=0\n"
        "print('PUBLIC-VERBOSE '+('x'*524288),flush=True)\n"
        "while time.monotonic()-began<seconds:\n"
        " print('PUBLIC-ALIVE',ticks,flush=True);ticks+=1;time.sleep(max(0,min(5,seconds-(time.monotonic()-began))))\n"
        "pathlib.Path('soak-child-result.json').write_text(json.dumps({'ticks':ticks,'elapsedSeconds':time.monotonic()-began,'constraint':'KEEP_EARLY_CONSTRAINT'}))\n"
        "print('PUBLIC-SOAK-COMPLETE',flush=True)\n", encoding='utf-8')
    evidence = 'PUBLIC-EVIDENCE 世界\n' * 650
    (project / 'evidence.txt').write_text(evidence, encoding='utf-8')
    payload_for = lambda index: f'KEEP_EARLY_CONSTRAINT\nfixture feature part {index}\n' + 'public implementation line\n' * (1400 if index == 0 else 150)
    state = {'requests': 0, 'summaries': 0, 'actions': 0, 'created': 0, 'overflowInjected': False,
             'historyRequested': False, 'historyVerified': False,
             'retryStage': 0,
             'guidanceRequested':False,'nestedFileRequested':False,
             'session': None, 'offset': 0, 'commandFinished': False, 'readWaits': 0}
    seen = set()
    failures = []
    metrics = []
    began = time.monotonic()

    class Provider(BaseHTTPRequestHandler):
        def log_message(self, *_):
            pass

        def do_POST(self):
            body = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
            index = state['requests']
            state['requests'] += 1
            if not body.get('tools'):
                state['summaries'] += 1
                self.respond({'content': '## Decisions\nPreserve KEEP_EARLY_CONSTRAINT and the original objective.\n## Progress\nCompleted file receipts are saved.\n## Remaining\nInspect the running command and verify all expected artifacts.'}, 'stop')
                return
            if not any('KEEP_EARLY_CONSTRAINT' in m.get('content', '') for m in body['messages']):
                failures.append('Original constraint missing from request')
            metrics.append({'tools': len(body['tools']),
                            'messageBytes': len(json.dumps(body['messages'], ensure_ascii=False).encode())})
            for message in body['messages']:
                if message['role'] != 'tool' or message.get('tool_call_id') in seen:
                    continue
                seen.add(message.get('tool_call_id'))
                try:
                    result = json.loads(message['content'])
                except (ValueError, TypeError):
                    continue
                if isinstance(result, dict) and isinstance(result.get('session'), dict):
                    receipt = result['session']
                    state['session'] = receipt['id']
                    state['offset'] = result.get('nextOffset', state['offset'])
                    if receipt['state'] not in ['starting', 'running', 'stopping']:
                        state['commandFinished'] = True
                        if receipt.get('exitCode') != 0 or not receipt.get('cleanupComplete'):
                            failures.append('Command failed or cleanup remained uncertain')
                        if not receipt.get('outputRetained') or not receipt.get('localLog'):
                            failures.append('Verbose command output was not durably spooled')
                        state['outputLog'] = receipt.get('localLog')
                if isinstance(result, dict) and result.get('callId') == 'public-soak-1' and 'content' in result:
                    state['historyVerified'] = 'PUBLIC-EVIDENCE' in result['content'] and result['totalBytes'] > 8192
            # Trigger after substantial completed work while below heuristic capacity.
            if state['actions'] >= 70 and not state['overflowInjected']:
                state['overflowInjected'] = True
                response = json.dumps({'error': {'code': 'context_length_exceeded',
                                                  'message': 'Public synthetic context rejection'}}).encode()
                self.send_response(400)
                self.send_header('Content-Type', 'application/json')
                self.send_header('Content-Length', str(len(response)))
                self.end_headers()
                self.wfile.write(response)
                return
            # Model-specific failure shape: a truncated active stream, then an
            # unsupported/stalled complete response. The last retry must stream.
            if state['actions'] >= 80 and state['retryStage'] < 3:
                stage = state['retryStage']
                state['retryStage'] += 1
                if stage == 0:
                    payload = 'data: ' + json.dumps({'choices':[{'index':0,'delta':{'reasoning_content':'Public incomplete fixture reasoning'},'finish_reason':None}]}) + '\n\n'
                    self.send_response(200)
                    self.send_header('Content-Type','text/event-stream')
                    self.end_headers()
                    self.wfile.write(payload.encode())
                    return
                if stage == 1:
                    if body.get('stream') is not False:
                        failures.append('Expected one complete-response recovery attempt')
                    time.sleep(2)
                    return
                if body.get('stream') is not True:
                    failures.append('Stalled complete-response mode was repeated')
            if state['actions'] < args.steps:
                n = state['actions']
                if n % 4 == 0:
                    part = state['created']
                    name, arguments = 'create_text_file', {'path': f'part-{part}.txt', 'content': payload_for(part)}
                    state['created'] += 1
                else:
                    name, arguments = 'read_text_file', {'path': 'evidence.txt'}
                state['actions'] += 1
            elif not state['guidanceRequested']:
                state['guidanceRequested']=True
                name,arguments='read_scoped_instructions',{'path':'src/deep/new.txt'}
            elif not state['nestedFileRequested']:
                state['nestedFileRequested']=True
                if not any('NESTED_SCOPE_RULE' in m.get('content','') for m in body['messages'] if m['role']=='system'):
                    failures.append('Applicable nested guidance was not retained as scoped system context')
                name,arguments='create_text_file',{'path':'src/deep/new.txt','content':'KEEP_EARLY_CONSTRAINT NESTED_SCOPE_RULE'}
            elif not state['historyRequested']:
                state['historyRequested'] = True
                name, arguments = 'read_task_history', {'callId': 'public-soak-1'}
            elif state['session'] is None:
                name, arguments = 'command_session', {'operation': 'start', 'program': sys.executable,
                    'args': ['soak-child.py', str(args.seconds)], 'timeout_seconds': args.seconds + 60,
                    'capture_bytes': 8192}
            elif not state['commandFinished']:
                name, arguments = 'command_session', {'operation': 'read', 'session': state['session'],
                    'offset': state['offset'], 'wait_ms': 30000}
                state['readWaits'] += 1
            else:
                name, arguments = None, None
            if name:
                self.respond({'tool_calls': [{'index': 0, 'id': f'public-soak-{index}', 'type': 'function',
                    'function': {'name': name, 'arguments': json.dumps(arguments)}}]}, 'tool_calls')
            else:
                self.respond({'content': 'Public fixture artifacts and supervised command complete. KEEP_EARLY_CONSTRAINT preserved.'}, 'stop')

        def respond(self, delta, finish):
            chunks = [{'choices': [{'index': 0, 'delta': delta, 'finish_reason': None}]},
                      {'choices': [{'index': 0, 'delta': {}, 'finish_reason': finish}]},
                      {'choices': [], 'usage': {'prompt_tokens': 100, 'completion_tokens': 20, 'total_tokens': 120}}]
            self.send_response(200)
            self.send_header('Content-Type', 'text/event-stream')
            self.end_headers()
            self.wfile.write((''.join('data: ' + json.dumps(c) + '\n\n' for c in chunks) + 'data: [DONE]\n\n').encode())

    server = ThreadingHTTPServer(('127.0.0.1', 0), Provider)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    client = NativeHost(directory)
    done = None
    try:
        call = client.call
        call('bootstrap')
        policy = call('memories')['automaticPolicy']
        call('setAutomaticMemory', enabled=False, revision=policy['revision'])
        call('configure', preferences={'baseUrl': f'http://127.0.0.1:{server.server_port}/v1', 'model': 'fixture'},
             enabledModels=['fixture'], modelContexts={'fixture': 500000}, apiKey='', rememberConnection=False)
        call('setRequestSettings', settings={'maxOutputTokens':None,'timeoutSeconds':1})
        session = call('createSession', kind='project', path=str(project))['session']['id']
        guidance = call('reviewInstructions', session=session)
        call('enableInstructions',session=session,token=guidance['token'])
        call('setTaskPermissions', session=session, revision=0,
             policy={'mode': 'fullAccess', 'grants': [], 'expiresAt': None})
        call('start', id=1, session=session,
             input='Implement the public fixture feature, verify its files and supervise its build. KEEP_EARLY_CONSTRAINT. Do not modify unrelated files.')
        deadline = time.monotonic() + args.seconds + max(300, args.steps)
        next_report = time.monotonic() + 30
        maintenance = 0
        draft_saved = False
        complete_waits = 0
        while time.monotonic() < deadline and done is None:
            for event in call('poll', id=1):
                if event['type'] == 'modelActivity' and event['phase'] == 'waitingComplete':
                    complete_waits += 1
                if event['type'] == 'modelStep' and not draft_saved:
                    call('saveDraft', session=session, text='Preserve my unrelated draft')
                    draft_saved = True
                if event['type'] == 'taskMaintenance':
                    maintenance += 1
                if event['type'] == 'done':
                    done = event
            if time.monotonic() >= next_report:
                progress = {'elapsedSeconds': round(time.monotonic() - began), **state,
                            'providerChecks': failures, 'complete': False}
                (directory / 'progress.json').write_text(json.dumps(progress, indent=2), encoding='utf-8')
                print(json.dumps(progress), flush=True)
                next_report = time.monotonic() + 30
            time.sleep(.1)
        assert done is not None, 'Bounded soak deadline exceeded'
        assert not done.get('error'), done.get('error')
        assert not failures, failures
        assert state['overflowInjected'] and state['summaries'] > 0, state
        assert state['actions'] == args.steps and state['commandFinished'], state
        assert state['historyVerified'], 'Agent must recover an original early receipt after compaction'
        assert (project / 'src/deep/new.txt').read_text() == 'KEEP_EARLY_CONSTRAINT NESTED_SCOPE_RULE'
        assert state['retryStage'] == 3 and complete_waits > 0, 'Retry mode and terminal delivery must be observable without Stop'
        for index in range(state['created']):
            assert (project / f'part-{index}.txt').read_text(encoding='utf-8') == payload_for(index)
        child_result = json.loads((project / 'soak-child-result.json').read_text())
        assert child_result['elapsedSeconds'] >= args.seconds
        assert child_result['constraint'] == 'KEEP_EARLY_CONSTRAINT'
        output_log = (project / state['outputLog']).read_bytes()
        assert Path(state['outputLog']).is_absolute(), 'Command log receipt needs its saved absolute path'
        assert not list(project.glob('dolores-command-*')), 'Harness logs polluted the project'
        assert any(path.samefile(Path(state['outputLog'])) for path in (directory / 'data/command-logs').rglob('*.log')), 'Command output was not saved in the owning profile'
        assert len(output_log) > 524288 and b'PUBLIC-SOAK-COMPLETE' in output_log
        assert child.read_text(encoding='utf-8').startswith('import json,pathlib,sys,time')
        assert call('savedDraft', session=session) == 'Preserve my unrelated draft'
        export_path = directory / 'conversation.json'
        call('export', session=session, path=str(export_path), format='json')
        exported = json.loads(export_path.read_text(encoding='utf-8'))
        metadata = exported['messages'][-1]['metadata']
        assert not metadata.get('paused'), metadata.get('paused')
        assert metadata['agent'].get('archive'), 'Sustained fixture must archive evidence'
        readable = call('runProgress', session=session, runId=exported['runs'][0]['snapshot']['id'], ordinal=1)
        assert readable['summary']['tools'], 'Archived UI progress must have decoded original tool records'
        assert len(exported['runs']) == 1, 'One Start must remain one run'
        attempts = [e['data'] for e in exported['runs'][0]['events'] if e['kind'] == 'modelAttemptFinished']
        assert any(a['buffered'] and a['outcome'] == 'timeout' for a in attempts), 'Buffered failure duration and mode must remain inspectable'
        assert exported['usageSummary']['missingUsageCalls'] >= 1, 'Rejected attempt must remain unknown'
        result = {'passed': True, 'liveModel': False, 'wallClockSeconds': round(time.monotonic() - began, 2),
                  'commandSeconds': child_result['elapsedSeconds'], 'twoHourGate': child_result['elapsedSeconds'] >= 7200,
                  'fixtureSteps': args.steps, 'verifiedFiles': state['created'], **state,
                  'maintenanceEvents': maintenance, 'maxRequestMessageBytes': max(m['messageBytes'] for m in metrics),
                  'completeResponseWaitEvents': complete_waits,
                  'usageSummary': exported['usageSummary']}
        (directory / 'result.json').write_text(json.dumps(result, indent=2), encoding='utf-8')
        print(json.dumps(result), flush=True)
    except Exception as error:
        (directory / 'result.json').write_text(json.dumps({'passed': False, 'error': str(error)[:2000],
            'elapsedSeconds': time.monotonic() - began, **state, 'providerChecks': failures}, indent=2), encoding='utf-8')
        raise
    finally:
        if done is None:
            client.envelope('cancel', id=1)
        client.close()
        server.shutdown()
        server.server_close()


if __name__ == '__main__':
    main()
