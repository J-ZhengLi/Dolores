"""Isolated real-FFI/SQLite/HTTP paused-task recovery diagnostic.

Build the Flutter bundle first, then run save and restore in separate processes
with the same fresh absolute --directory under output/. No live API is used.
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
loader = os.add_dll_directory(str(bundle)) if os.name == 'nt' else None
native = ctypes.CDLL(str(bundle / ('dolores_flutter_bridge.dll' if os.name == 'nt' else 'lib/libdolores_flutter_bridge.so')))
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
    result = envelope(command, **fields)
    assert result['ok'], result.get('error')
    return result.get('result')
requests = []
mode = 'normal'

class ModelFixture(BaseHTTPRequestHandler):

    def log_message(self, *_):
        pass

    def do_POST(self):
        payload = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        requests.append(payload)
        last = payload['messages'][-1]
        input = last['content']
        calls = []
        finish = 'stop'
        text = 'Complete synthetic result.'
        if mode == 'cancel':
            text = 'Unsaved continuation'
            finish = 'stop'
        elif last['role'] == 'user' and input == 'output':
            text = 'Saved partial 世界'
            finish = 'length'
        elif last['role'] == 'user' and '"originalTask":"output"' in input:
            assert 'Saved partial 世界' in input or 'Second partial 世界' in input
            assert 'fresh tools and approvals' in input
            if mode != 'complete':
                text = 'Second partial 世界'
                finish = 'length'
        elif last['role'] == 'user' and '"originalTask":"steps"' in input:
            assert all((f'part-{i}.txt' in input for i in range(3)))
            assert 'completed' in input
            calls = [{'index': 0, 'id': 'verify', 'type': 'function', 'function': {'name': 'read_text_file', 'arguments': json.dumps({'path': 'part-2.txt'})}}]
            finish = 'tool_calls'
            text = 'Verify completed progress.'
        elif input == 'steps' or (last['role'] == 'tool' and next((m['content'] for m in reversed(payload['messages']) if m['role'] == 'user')) == 'steps'):
            step = sum((m['role'] == 'tool' for m in payload['messages']))
            calls = [{'index': 0, 'id': f'create-{step}', 'type': 'function', 'function': {'name': 'create_text_file', 'arguments': json.dumps({'path': f'part-{step}.txt', 'content': f'complete segment {step} 世界'})}}]
            finish = 'tool_calls'
            text = f'Saved step {step}.'
        self.send_response(200)
        self.send_header('Content-Type', 'text/event-stream')
        self.end_headers()
        try:
            self.wfile.write(('data: ' + json.dumps({'choices': [{'delta': {'content': text, **({'tool_calls': calls} if calls else {})}, 'finish_reason': None}]}) + '\n\n').encode())
            self.wfile.flush()
            if mode == 'cancel':
                time.sleep(0.6)
            chunks = [{'choices': [{'delta': {}, 'finish_reason': finish}]}, {'choices': [], 'usage': {'prompt_tokens': 100, 'completion_tokens': 64}}]
            self.wfile.write((''.join(('data: ' + json.dumps(c) + '\n\n' for c in chunks)) + 'data: [DONE]\n\n').encode())
        except (BrokenPipeError, ConnectionResetError):
            pass

def done(number, stop=False):
    deadline = time.monotonic() + 15
    while time.monotonic() < deadline:
        for event in call('poll', id=number):
            if event['type'] == 'toolApproval':
                request = event['request']
                assert request['name'] in ['create_text_file', 'read_text_file']
                call('approveTool', id=number, callId=request['callId'], allow=True)
            if stop and event['type'] in ['delta', 'modelText']:
                call('cancel', id=number)
            if event['type'] == 'done':
                return event
        time.sleep(0.01)
    raise AssertionError('Continuation diagnostic timed out')

def messages(session):
    return call('messagesPage', session=session)['items']

def start(number, session, input, continuation=None):
    call('start', id=number, session=session, input=input, **{'continuation': continuation} if continuation is not None else {})
call('bootstrap')
server = ThreadingHTTPServer(('127.0.0.1', 0), ModelFixture)
threading.Thread(target=server.serve_forever, daemon=True).start()
try:
    policy = call('memories')['automaticPolicy']
    call('setAutomaticMemory', enabled=False, revision=policy['revision'])
    call('configure', preferences={'baseUrl': f'http://127.0.0.1:{server.server_port}/v1', 'model': 'fixture'}, apiKey='', rememberConnection=False)
    state_file = directory / 'state.json'
    if args.stage == 'save':
        session = call('createSession', kind='side')['session']['id']
        start(1, session, 'output')
        assert 'error' not in done(1)
        saved = messages(session)[-1]
        assert saved['content'] == 'Saved partial 世界'
        assert saved['metadata']['paused']['reason'] == 'outputLimit'
        first = saved['id']
        assert len(requests) == 1
        call('setRequestSettings', settings={'maxOutputTokens': 128, 'timeoutSeconds': 30})
        start(2, session, 'Continue working on the previous task.', first)
        assert 'error' not in done(2)
        saved = messages(session)[-1]
        assert saved['content'] == 'Second partial 世界'
        assert saved['metadata']['paused']['task'] == 'output'
        assert len(requests) == 2 and requests[1]['max_tokens'] == 128
        latest = saved['id']
        assert not envelope('start', id=3, session=session, input='Continue working on the previous task.', continuation=first)['ok']
        assert not envelope('start', id=3, session=session, input='changed input', continuation=latest)['ok']
        other = call('createSession', kind='side')['session']['id']
        assert not envelope('start', id=3, session=other, input='Continue working on the previous task.', continuation=latest)['ok']
        mode = 'cancel'
        start(4, session, 'Continue working on the previous task.', latest)
        assert 'error' in done(4, True)
        assert messages(session)[-1] == saved
        mode = 'normal'
        folder = directory / 'project'
        folder.mkdir()
        work = call('createSession', kind='project', path=str(folder))['session']['id']
        start(5, work, 'steps')
        assert 'error' not in done(5)
        paused = messages(work)[-1]
        assert paused['metadata']['paused']['reason'] == 'stepLimit'
        assert len(paused['metadata']['agent']['tools']) == 3
        assert all(((folder / f'part-{i}.txt').exists() for i in range(3))) and (not (folder / 'part-3.txt').exists())
        before = [(folder / f'part-{i}.txt').read_bytes() for i in range(3)]
        start(6, work, 'Continue working on the previous task.', paused['id'])
        assert 'error' not in done(6)
        complete = messages(work)[-1]
        assert 'paused' not in complete['metadata']
        assert complete['metadata']['agent']['tools'][0]['name'] == 'read_text_file'
        assert before == [(folder / f'part-{i}.txt').read_bytes() for i in range(3)] and (not (folder / 'part-3.txt').exists())
        state_file.write_text(json.dumps({'session': session, 'latest': latest, 'saved': saved, 'work': work}))
    else:
        state = json.loads(state_file.read_text())
        session = state['session']
        assert messages(session)[-1] == state['saved']
        mode = 'complete'
        start(7, session, 'Continue working on the previous task.', state['latest'])
        assert 'error' not in done(7)
        assert 'paused' not in messages(session)[-1]['metadata']
        export = directory / 'paused.json'
        call('export', session=session, path=str(export), format='json')
        assert 'outputLimit' in export.read_text(encoding='utf-8')
finally:
    call('shutdown')
    server.shutdown()
print(json.dumps({'ok': True, 'stage': args.stage, 'fixtureRequests': len(requests), 'liveRequests': 0}))
