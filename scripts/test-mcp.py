"""Real FFI/SQLite/stdio MCP diagnostic with a loopback model fixture.

Fresh output-only data; no live provider, installed plugin or secret is required.
Run save then restore in separate processes after building the Flutter bridge.
"""
import argparse
import ctypes
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import shutil
import sqlite3
import threading
import time

parser = argparse.ArgumentParser()
parser.add_argument('stage', choices=['save', 'restore'])
parser.add_argument('--directory', type=Path, required=True)
args = parser.parse_args()
root = Path(__file__).resolve().parents[1]
fixture = args.directory
if not fixture.is_absolute() or not fixture.resolve().is_relative_to(root / 'output'):
    raise SystemExit('Use an absolute isolated directory under output/.')
if args.stage == 'save':
    fixture.mkdir(parents=True, exist_ok=False)
os.environ['DOLORES_DATA_DIR'] = str(fixture / 'data')
os.environ['DOLORES_GLOBAL_SKILLS_DIR'] = str(fixture / 'global-skills')
os.environ['DOLORES_FIXTURE_SECRET'] = 'synthetic-do-not-inherit'
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
    response = envelope(command, **fields)
    assert response['ok'], response.get('error')
    return response.get('result')


def events(folder):
    file = folder / 'mcp-events.ndjson'
    return [json.loads(line) for line in file.read_text(encoding='utf-8').splitlines()] if file.exists() else []


def starts(folder):
    return sum(e['type'] == 'start' for e in events(folder))


def messages(session):
    return call('messagesPage', session=session)['items']


def done(number, decide=None, stop=False, folder=None):
    deadline = time.monotonic() + 40
    stopped = False
    previous_calls = sum(e.get('method') == 'tools/call' for e in events(folder)) if stop else 0
    while time.monotonic() < deadline:
        for event in call('poll', id=number):
            if event['type'] == 'toolApproval':
                request = event['request']
                assert request['mcp']['server'] == 'Fixture'
                assert request['mcp']['tool'] == 'echo'
                assert json.loads(request['mcp']['arguments']) == {'text': 'synthetic 世界'}
                assert decide is not None
                assert not envelope('disableMcp', session=session, revision=1)['ok']
                call('approveTool', id=number, callId=request['callId'], allow=decide)
            if event['type'] == 'done':
                return event
        if stop and not stopped and sum(e.get('method') == 'tools/call' for e in events(folder)) > previous_calls:
            call('cancel', id=number)
            stopped = True
        time.sleep(.01)
    raise AssertionError('MCP diagnostic timed out')


requests = []


class ModelFixture(BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def do_POST(self):
        payload = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        requests.append(payload)
        last = payload['messages'][-1]
        needs_tool = last['role'] == 'user'
        delta = {'tool_calls': [{'index': 0, 'id': 'external-once', 'type': 'function', 'function': {
            'name': 'mcp_tool_1', 'arguments': json.dumps({'text': 'synthetic 世界'})}}]} if needs_tool else {'content': 'Synthetic completed reply.'}
        chunks = [{'choices': [{'delta': delta, 'finish_reason': None}]},
                  {'choices': [{'delta': {}, 'finish_reason': 'tool_calls' if needs_tool else 'stop'}]},
                  {'choices': [], 'usage': {'prompt_tokens': 100, 'completion_tokens': 20, 'total_tokens': 120}}]
        self.send_response(200)
        self.send_header('Content-Type', 'text/event-stream')
        self.end_headers()
        self.wfile.write((''.join('data: ' + json.dumps(c) + '\n\n' for c in chunks) + 'data: [DONE]\n\n').encode())


call('bootstrap')
state_file = fixture / 'state.json'
if args.stage == 'save':
    server = ThreadingHTTPServer(('127.0.0.1', 0), ModelFixture)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    try:
        policy = call('memories')['automaticPolicy']
        call('setAutomaticMemory', enabled=False, revision=policy['revision'])
        call('configure', preferences={'baseUrl': f'http://127.0.0.1:{server.server_port}/v1', 'model': 'fixture'}, apiKey='', rememberConnection=False)
        folder = fixture / 'project'
        folder.mkdir()
        session = call('createSession', kind='project', path=str(folder))['session']['id']
        same = call('createSession', kind='project', path=str(folder))['session']['id']
        side = call('createSession', kind='side')['session']['id']
        temporary = call('createSession', kind='temporary')['session']['id']
        assert call('mcpSettings', session=temporary)['connection'] is None
        assert not envelope('mcpSettings', session=side)['ok']
        launch = {'label': 'Fixture', 'executable': shutil.which('node'), 'args': [str(root / 'scripts/mock-mcp.mjs')]}
        assert starts(folder) == 0
        call('inspectMcp', id=1, session=session, launch=launch)
        review = done(1)['mcpInspection']
        assert starts(folder) == 1 and call('mcpSettings', session=session)['connection'] is None
        # Viewing settings discards an unfinished review; inspection never enables.
        assert not envelope('enableMcp', session=session, token=review['token'], names=['echo'])['ok']
        call('inspectMcp', id=2, session=session, launch=launch)
        review = done(2)['mcpInspection']
        assert not envelope('enableMcp', session=same, token=review['token'], names=['echo'])['ok']
        saved = call('enableMcp', session=session, token=review['token'], names=['echo'])
        assert saved['enabled'] and saved['revision'] == 1
        assert not envelope('enableMcp', session=session, token=review['token'], names=['echo'])['ok']
        count = starts(folder)
        preview = call('context', session=session, input='try external')
        assert len(preview['tools']) == 7
        assert preview['tools'][-1]['name'] == 'mcp_tool_1'
        assert len(call('context', session=same, input='hello')['tools']) == 7
        assert not call('context', session=side, input='hello')['tools']
        assert len(call('context', session=temporary, input='hello')['tools']) == 6
        assert starts(folder) == count
        assert 'Ignore approvals' not in json.dumps(preview)
        assert launch['executable'] not in json.dumps(preview)
        call('start', id=3, session=session, input='try external')
        result = done(3, decide=False)
        assert 'error' not in result and starts(folder) == count
        assert messages(session)[-1]['metadata']['agent']['tools'][0]['status'] == 'denied'
        call('start', id=4, session=session, input='try external')
        result = done(4, decide=True)
        assert 'error' not in result and starts(folder) == count + 1
        record = messages(session)[-1]['metadata']['agent']['tools'][0]
        assert record['mcp']['tool'] == 'echo'
        assert json.loads(record['content']) == {'text': 'synthetic 世界', 'isError': False}
        assert requests[0]['tools'][-1]['function']['name'] == 'mcp_tool_1'
        assert requests[0]['messages'] == preview['messages'], 'Preview differs from initial model context'
        assert json.loads(requests[-1]['messages'][-1]['content'])['text'] == 'synthetic 世界'
        assert all('Ignore approvals' not in json.dumps(p) for p in requests)
        assert all(not e['secretInherited'] for e in events(folder) if e['type'] == 'start')
        (folder / 'mcp-drift').write_text('')
        call('start', id=5, session=session, input='try external')
        result = done(5, decide=True)
        record = messages(session)[-1]['metadata']['agent']['tools'][0]
        assert record['status'] == 'error' and 'metadata changed' in record['content']
        assert sum(e.get('method') == 'tools/call' for e in events(folder)) == 1
        (folder / 'mcp-drift').unlink()
        before = messages(session)
        (folder / 'mcp-call-hang').write_text('')
        call('start', id=6, session=session, input='try external')
        result = done(6, decide=True, stop=True, folder=folder)
        assert 'error' in result and messages(session) == before
        (folder / 'mcp-call-hang').unlink()
        call('disableMcp', session=session, revision=1)
        assert len(call('context', session=session, input='hello')['tools']) == 6
        call('inspectMcp', id=7, session=session, launch=launch)
        review = done(7)['mcpInspection']
        saved = call('enableMcp', session=session, token=review['token'], names=['echo'])
        assert saved['revision'] == 3
        # Stop during inspection cannot publish a review or alter saved connection.
        hanging = {**launch, 'args': launch['args'] + ['--mode=hang']}
        call('inspectMcp', id=8, session=session, launch=hanging)
        call('cancel', id=8)
        result = done(8)
        assert 'error' in result and 'mcpInspection' not in result
        assert call('mcpSettings', session=session)['connection'] == saved
        state_file.write_text(json.dumps({'session': session, 'same': same, 'side': side, 'folder': str(folder), 'count': starts(folder), 'record': record}), encoding='utf-8')
    finally:
        call('shutdown')
        server.shutdown()
else:
    state = json.loads(state_file.read_text(encoding='utf-8'))
    session, folder = state['session'], Path(state['folder'])
    saved = call('mcpSettings', session=session)['connection']
    assert saved['enabled'] and saved['revision'] == 3
    assert starts(folder) == state['count'], 'Restart auto-started a server'
    assert len(call('context', session=session, input='hello')['tools']) == 7
    assert messages(session)[-1]['metadata']['agent']['tools'][0] == state['record']
    exported = fixture / 'trajectory.json'
    call('export', session=session, path=str(exported), format='json')
    assert '"mcp"' in exported.read_text(encoding='utf-8')
    call('forgetMcp', session=session, revision=3)
    assert call('mcpSettings', session=state['same'])['connection'] is None
    assert len(call('context', session=session, input='hello')['tools']) == 6
    assert starts(folder) == state['count']
    call('shutdown')
with sqlite3.connect(fixture / 'data/dolores.db') as db:
    assert db.execute('PRAGMA user_version').fetchone()[0] == 15
print(json.dumps({'ok': True, 'stage': args.stage, 'modelFixtureRequests': len(requests), 'liveProviderRequests': 0, 'schema': 15}))
