"""Isolated FFI/SQLite/HTTP failed-check repair diagnostic with actual Node.

Build the Windows Flutter bundle first; run save and restore in separate
processes with the same fresh absolute --directory under output/.
"""
import argparse
import ctypes
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import threading
import time

parser = argparse.ArgumentParser()
parser.add_argument('stage', choices=['save', 'restore'])
parser.add_argument('--directory', type=Path, required=True)
args = parser.parse_args()
root = Path(__file__).resolve().parents[1]
directory = args.directory
if not directory.is_absolute() or not directory.resolve().is_relative_to(root / 'output'):
    raise SystemExit('Use an absolute isolated directory under output/.')
if args.stage == 'save':
    directory.mkdir(parents=True, exist_ok=False)
os.environ['DOLORES_DATA_DIR'] = str(directory / 'data')
os.environ['DOLORES_GLOBAL_SKILLS_DIR'] = str(directory / 'global-skills')
bundle = root / 'apps/dolores_flutter/build/windows/x64/runner/Release'
loader = os.add_dll_directory(str(bundle))
native = ctypes.CDLL(str(bundle / 'dolores_flutter_bridge.dll'))
native.dolores_call.argtypes = [ctypes.c_void_p, ctypes.c_size_t]
native.dolores_call.restype = ctypes.c_void_p
native.dolores_free.argtypes = [ctypes.c_void_p]


def envelope(command, **fields):
    payload = json.dumps({'command': command, **fields}).encode()
    buffer = ctypes.create_string_buffer(payload)
    pointer = native.dolores_call(buffer, len(payload))
    try:
        return json.loads(ctypes.string_at(pointer))
    finally:
        native.dolores_free(pointer)


def call(command, **fields):
    response = envelope(command, **fields)
    assert response['ok'], response.get('error')
    return response.get('result')


broken = 'module.exports = numbers => numbers.reduce((a, b) => a - b, 0);\n'
fixed = broken.replace('a - b', 'a + b')
checks = """const assert = require('node:assert/strict');
const sum = require('./sum.cjs');
assert.equal(sum([]), 0);
assert.equal(sum([2, 3]), 5);
assert.equal(sum([-2, 3]), 1);
console.log('3 checks passed');
"""
invocation = {'program': 'node', 'args': ['check.cjs']}
requests = []
mode = 'build'


class Fixture(BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def do_POST(self):
        payload = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        requests.append(payload)
        count = sum(m['role'] == 'tool' for m in payload['messages'])
        system = payload['messages'][0]['content']
        assert f'model call {count + 1 if mode != "build" else (1 if count == 0 else count)}/4' in system
        assert 'tool operations remain' in system and 'reserve a tool operation' in system
        calls = []

        def propose(name, arguments):
            calls.append({'id': f'call-{len(requests)}-{len(calls)}', 'type': 'function',
                          'function': {'name': name, 'arguments': json.dumps(arguments)}})

        if mode == 'build':
            if count == 0:
                propose('create_text_file', {'path': 'sum.cjs', 'content': broken})
                propose('create_text_file', {'path': 'check.cjs', 'content': checks})
            elif count == 2:
                propose('run_command', invocation)
            elif count == 3:
                assert 'AssertionError' in payload['messages'][-1]['content']
        elif mode == 'unrelated':
            if count == 0:
                propose('run_command', {'program': 'node', 'args': ['--version']})
        elif mode in ['repair', 'cancel', 'stale']:
            prompt = next(m['content'] for m in reversed(payload['messages']) if m['role'] == 'user')
            assert 'repair the implementation' in prompt and 'AssertionError' in prompt
            if count == 0:
                propose('read_text_file', {'path': 'sum.cjs'})
            elif count == 1:
                propose('edit_text_file', {'path': 'sum.cjs', 'old_text': 'a - b', 'new_text': 'a + b'})
            elif count == 2 and mode == 'repair':
                propose('run_command', invocation)
        elif mode == 'incomplete' and count == 0:
            propose('run_command', {'program': 'node', 'args': ['-e', "process.stdout.write('x'.repeat(10000))"]})
        content = 'Everything passed.'  # Deliberately false model claim must not erase failures.
        delta = {'tool_calls': [{'index': i, **c} for i, c in enumerate(calls)]} if calls else {'content': content}
        frames = [
            {'choices': [{'index': 0, 'delta': delta, 'finish_reason': None}]},
            {'choices': [{'index': 0, 'delta': {}, 'finish_reason': 'tool_calls' if calls else 'stop'}]},
            {'choices': [], 'usage': {'prompt_tokens': 100, 'completion_tokens': 50}},
        ]
        self.send_response(200)
        self.send_header('Content-Type', 'text/event-stream')
        self.end_headers()
        self.wfile.write((''.join('data: ' + json.dumps(f) + '\n\n' for f in frames) + 'data: [DONE]\n\n').encode())


approvals = []


def messages(session):
    return call('messagesPage', session=session)['items']


def run(number, session, continuation=None):
    call('start', id=number, session=session,
         input='Build and check sum' if continuation is None else 'Continue working on the previous task.',
         **({'continuation': continuation} if continuation is not None else {}))
    deadline = time.monotonic() + 25
    while time.monotonic() < deadline:
        for event in call('poll', id=number):
            if event['type'] == 'toolApproval':
                request = event['request']
                approvals.append(request)
                assert request['name'] in ['read_text_file', 'create_text_file', 'edit_text_file', 'run_command']
                if request['name'] == 'run_command':
                    assert request['command']['invocation'] in [
                        invocation, {'program': 'node', 'args': ['--version']},
                        {'program': 'node', 'args': ['-e', "process.stdout.write('x'.repeat(10000))"]},
                    ]
                if mode == 'cancel' and request['name'] == 'edit_text_file':
                    call('cancel', id=number)
                else:
                    if mode == 'stale' and request['name'] == 'edit_text_file':
                        (directory / 'project/sum.cjs').write_text(broken + '// external edit\n', encoding='utf-8')
                    call('approveTool', id=number, callId=request['callId'], allow=True)
            if event['type'] == 'done':
                return event
        time.sleep(.01)
    raise AssertionError('Bounded repair fixture timed out')


call('bootstrap')
policy = call('memories')['automaticPolicy']
call('setAutomaticMemory', enabled=False, revision=policy['revision'])
server = ThreadingHTTPServer(('127.0.0.1', 0), Fixture)
threading.Thread(target=server.serve_forever, daemon=True).start()
call('configure', preferences={'baseUrl': f'http://127.0.0.1:{server.server_port}/v1', 'model': 'fixture'},
     apiKey='', rememberConnection=False)
state_file = directory / 'state.json'
try:
    if args.stage == 'save':
        project = directory / 'project'
        project.mkdir()
        session = call('createSession', kind='project', path=str(project))['session']['id']
        # This fixture exercises a bounded repair, independent of Automatic defaults.
        call('saveScopedSettings', session=session, scope='thread', revision=0,
             patch={'task': {'modelCalls': 4, 'toolCalls': 4, 'segments': 4, 'elapsedSeconds': None}})
        assert not run(1, session).get('error')
        last = messages(session)[-1]
        assert last['content'] == 'Everything passed.'
        assert last['metadata']['paused']['reason'] == 'commandReview'
        record = last['metadata']['agent']['tools'][-1]
        assert record['status'] == 'failed' and json.loads(record['content'])['exitCode'] != 0
        assert 'AssertionError' in record['content']
        assert len(approvals) == 3 and len(call('changesPage', session=session)['items']) == 2
        state_file.write_text(json.dumps({'session': session, 'messages': messages(session)}), encoding='utf-8')
    else:
        state = json.loads(state_file.read_text(encoding='utf-8'))
        session = state['session']
        assert messages(session) == state['messages']
        source = messages(session)[-1]['id']
        prior = len(requests)
        wrong = envelope('start', id=2, session=session, input='Continue working on the previous task.', continuation=source - 1)
        assert not wrong['ok'] and len(requests) == prior
        mode = 'cancel'
        assert run(3, session, source).get('error')
        assert messages(session) == state['messages']
        assert (directory / 'project/sum.cjs').read_text(encoding='utf-8') == broken
        mode = 'stale'
        assert not run(4, session, source).get('error')
        last = messages(session)[-1]
        assert last['metadata']['paused']['reason'] == 'commandReview'
        assert last['metadata']['agent']['tools'][-1]['status'] == 'error'
        assert 'File changed since preview' in last['metadata']['agent']['tools'][-1]['content']
        assert len(call('changesPage', session=session)['items']) == 2
        mode = 'unrelated'
        assert not run(5, session, last['id']).get('error')
        last = messages(session)[-1]
        assert last['metadata']['paused']['reason'] == 'commandReview'
        assert last['metadata']['agent']['tools'][-1]['status'] == 'completed'
        mode = 'repair'
        assert not run(6, session, last['id']).get('error')
        last = messages(session)[-1]
        assert 'paused' not in last['metadata']
        assert last['metadata']['agent']['modelCalls'] == 4
        records = last['metadata']['agent']['tools']
        assert [r['status'] for r in records] == ['read', 'edited', 'completed']
        assert json.loads(records[-1]['content'])['exitCode'] == 0
        assert '3 checks passed' in records[-1]['content']
        assert (directory / 'project/check.cjs').read_text(encoding='utf-8') == checks
        assert (directory / 'project/sum.cjs').read_text(encoding='utf-8') == fixed + '// external edit\n'
        assert len(call('changesPage', session=session)['items']) == 3
        mode = 'incomplete'
        assert not run(7, session).get('error')
        last = messages(session)[-1]
        assert last['metadata']['paused']['reason'] == 'commandReview'
        record = last['metadata']['agent']['tools'][-1]
        assert record['status'] == 'incomplete'
        assert json.loads(record['content'])['truncated'] is True
finally:
    call('shutdown')
    server.shutdown()
print(json.dumps({'ok': True, 'stage': args.stage, 'fixtureRequests': len(requests),
                  'approvals': len(approvals), 'liveRequests': 0}))
