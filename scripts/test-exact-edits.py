"""Isolated Windows FFI/SQLite/HTTP newline-aware exact-edit diagnostic.

Build the Flutter bundle; run save/restore separately against one fresh absolute
--directory under output/. Node validates synthetic modules. No live API is used.
"""
import argparse
import ctypes
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import sqlite3
import threading
import time

parser = argparse.ArgumentParser()
parser.add_argument('stage', choices=['save', 'restore'])
parser.add_argument('--directory', type=Path, required=True)
args = parser.parse_args()
root = Path(__file__).resolve().parents[1]
directory = args.directory
if not directory.is_absolute() or not directory.resolve().is_relative_to(root / 'output'):
    raise SystemExit('Use a fresh absolute isolated directory under output/.')
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
    value = envelope(command, **fields)
    assert value['ok'], value.get('error')
    return value.get('result')


old = 'function sum(numbers) {\n  return numbers.reduce((a,b) => a-b, 0);\n}'
new = old.replace('a-b', 'a+b')
before = '\ufeff// 世界\r\n' + old.replace('\n', '\r\n') + '\r\nmodule.exports=sum;'
after = before.replace('a-b', 'a+b')
mixed_before = '// LF header\n' + old.replace('\n', '\r\n') + '\r\nmodule.exports=sum;'
checks = """const assert=require('node:assert/strict');
const sum=require('./module.cjs');
assert.equal(sum([]),0);
assert.equal(sum([2,3]),5);
assert.equal(sum([-2,3]),1);
console.log('3 exact-edit checks passed');
"""
requests, approvals = [], []
mode = 'repair'


class Fixture(BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def do_POST(self):
        payload = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        requests.append(payload)
        spec = next(t['function'] for t in payload['tools'] if t['function']['name'] == 'edit_text_file')
        assert 'uniform newline style' in spec['description']
        count = sum(m['role'] == 'tool' for m in payload['messages'])
        calls = []

        def propose(name, arguments):
            calls.append({'index': len(calls), 'id': f'edit-{len(requests)}-{len(calls)}',
                          'type': 'function', 'function': {'name': name, 'arguments': json.dumps(arguments)}})

        if mode == 'repair':
            if count == 0:
                propose('read_text_file', {'path': 'module.cjs'})
                propose('read_text_file', {'path': 'check.cjs'})
            elif count == 2:
                propose('edit_text_file', {'path': 'module.cjs', 'old_text': old, 'new_text': new})
            elif count == 3:
                propose('run_command', {'program': 'node', 'args': ['check.cjs']})
        elif mode == 'mixed':
            if count == 0:
                propose('edit_text_file', {'path': 'mixed.cjs', 'old_text': old, 'new_text': new})
            elif count == 1:
                assert 'mixed or lone-CR' in payload['messages'][-1]['content']
                assert 'single-line match' in payload['messages'][-1]['content']
                propose('edit_text_file', {'path': 'mixed.cjs', 'old_text': 'a-b', 'new_text': 'a+b'})
            elif count == 2:
                propose('run_command', {'program': 'node', 'args': ['mixed-check.cjs']})
        elif count == 0:
            propose('edit_text_file', {'path': 'module.cjs', 'old_text': new,
                                      'new_text': new.replace('a+b', 'a*b')})
        frames = [
            {'choices': [{'index': 0, 'delta': {'tool_calls': calls} if calls else {'content': 'Actual tool results recorded.'},
                          'finish_reason': None}]},
            {'choices': [{'index': 0, 'delta': {}, 'finish_reason': 'tool_calls' if calls else 'stop'}]},
        ]
        self.send_response(200)
        self.send_header('Content-Type', 'text/event-stream')
        self.end_headers()
        self.wfile.write((''.join('data: ' + json.dumps(f) + '\n\n' for f in frames) + 'data: [DONE]\n\n').encode())


def messages(session):
    return call('messagesPage', session=session)['items']


def run(number, session):
    call('start', id=number, session=session, input=mode)
    deadline = time.monotonic() + 20
    while time.monotonic() < deadline:
        for event in call('poll', id=number):
            if event['type'] == 'toolApproval':
                request = event['request']
                approvals.append(request)
                assert request['name'] in ['read_text_file', 'edit_text_file', 'run_command']
                if request['name'] == 'run_command':
                    assert request['command']['invocation'] in [
                        {'program': 'node', 'args': ['check.cjs']},
                        {'program': 'node', 'args': ['mixed-check.cjs']},
                    ]
                if request['name'] == 'edit_text_file':
                    expected = mixed_before if mode == 'mixed' else after if mode != 'repair' else before
                    file = directory / 'project' / request['target']
                    assert file.read_bytes() == expected.encode()
                    assert '\r\n' in request['diff']
                    assert len(request['diff'].encode()) <= 16 * 1024
                    if mode == 'stop':
                        call('cancel', id=number)
                        continue
                    if mode == 'stale':
                        file.write_bytes(after.replace('\r\n', '\n').encode())
                call('approveTool', id=number, callId=request['callId'], allow=mode != 'deny')
            if event['type'] == 'done':
                return event
        time.sleep(.01)
    raise AssertionError('Exact-edit fixture timed out')


call('bootstrap')
state_file = directory / 'state.json'
server = ThreadingHTTPServer(('127.0.0.1', 0), Fixture)
threading.Thread(target=server.serve_forever, daemon=True).start()
try:
    if args.stage == 'save':
        policy = call('memories')['automaticPolicy']
        call('setAutomaticMemory', enabled=False, revision=policy['revision'])
        call('configure', preferences={'baseUrl': f'http://127.0.0.1:{server.server_port}/v1', 'model': 'fixture'},
             apiKey='', rememberConnection=False)
        project = directory / 'project'
        project.mkdir()
        (project / 'module.cjs').write_bytes(before.encode())
        (project / 'check.cjs').write_bytes(checks.encode())
        (project / 'mixed.cjs').write_bytes(mixed_before.encode())
        (project / 'mixed-check.cjs').write_bytes(checks.replace('./module.cjs', './mixed.cjs').encode())
        session = call('createSession', kind='project', path=str(project))['session']['id']
        assert not run(1, session).get('error')
        last = messages(session)[-1]
        assert 'paused' not in last['metadata']
        records = last['metadata']['agent']['tools']
        assert [r['status'] for r in records] == ['read', 'read', 'edited', 'completed']
        assert json.loads(records[-1]['content'])['exitCode'] == 0
        assert (project / 'module.cjs').read_bytes() == after.encode()
        changes = call('changesPage', session=session)['items']
        assert len(changes) == 1
        change_id = changes[0]['id']
        mode = 'stop'
        saved = messages(session)
        assert run(2, session).get('error')
        assert messages(session) == saved and (project / 'module.cjs').read_bytes() == after.encode()
        mode = 'deny'
        assert not run(3, session).get('error')
        assert messages(session)[-1]['metadata']['agent']['tools'][0]['status'] == 'denied'
        assert (project / 'module.cjs').read_bytes() == after.encode()
        mode = 'stale'
        assert not run(4, session).get('error')
        record = messages(session)[-1]['metadata']['agent']['tools'][0]
        assert record['status'] == 'error' and 'File changed since preview' in record['content']
        assert (project / 'module.cjs').read_bytes() == after.replace('\r\n', '\n').encode()
        assert len(call('changesPage', session=session)['items']) == 1
        (project / 'module.cjs').write_bytes(after.encode())  # Restore synthetic external change only.
        mode = 'mixed'
        assert not run(5, session).get('error')
        records = messages(session)[-1]['metadata']['agent']['tools']
        assert [r['status'] for r in records] == ['blocked', 'edited', 'completed']
        assert json.loads(records[-1]['content'])['exitCode'] == 0
        assert (project / 'mixed.cjs').read_bytes() == mixed_before.replace('a-b', 'a+b').encode()
        with sqlite3.connect(directory / 'data/dolores.db') as db:
            snapshot = db.execute('SELECT before_text,after_text FROM file_changes WHERE id=?', (change_id,)).fetchone()
            assert snapshot == (before, after)
        state_file.write_text(json.dumps({'session': session, 'change': change_id, 'messages': messages(session)}), encoding='utf-8')
    else:
        state = json.loads(state_file.read_text(encoding='utf-8'))
        session = state['session']
        assert messages(session) == state['messages']
        assert (directory / 'project/module.cjs').read_bytes() == after.encode()
        preview = call('previewRevert', session=session, changeId=state['change'])
        assert '\r\n' in preview['diff']
        call('cancelRevert', token=preview['token'])
        assert (directory / 'project/module.cjs').read_bytes() == after.encode()
        preview = call('previewRevert', session=session, changeId=state['change'])
        assert call('applyRevert', session=session, token=preview['token'])['applied']
        assert (directory / 'project/module.cjs').read_bytes() == before.encode()
        assert not envelope('applyRevert', session=session, token=preview['token'])['ok']
        assert len(call('changesPage', session=session)['items']) == 3
        assert messages(session) == state['messages']
finally:
    call('shutdown')
    server.shutdown()
print(json.dumps({'ok': True, 'stage': args.stage, 'fixtureRequests': len(requests),
                  'approvalRequests': len(approvals), 'liveRequests': 0}))
