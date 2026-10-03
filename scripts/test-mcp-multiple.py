"""Isolated real-FFI/SQLite/stdio multi-server pressure diagnostic.

Build the Flutter bridge first, then run save and restore in separate processes
with the same fresh absolute --directory under output/. No live API is used.
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


def finish(number, approve=False):
    deadline = time.monotonic() + 45
    while time.monotonic() < deadline:
        for event in call('poll', id=number):
            if event['type'] == 'toolApproval':
                assert approve
                request = event['request']
                assert request['mcp']['connectionId'] in ids
                assert request['mcp']['tool'] == 'echo'
                call('approveTool', id=number, callId=request['callId'], allow=True)
            if event['type'] == 'done':
                return event
        time.sleep(.01)
    call('cancel', id=number)
    raise AssertionError('Multi-server diagnostic timed out')


def starts():
    file = folder / 'mcp-events.ndjson'
    return sum(json.loads(line)['type'] == 'start' for line in file.read_text().splitlines()) if file.exists() else 0


def inspect(number, identity, launch):
    call('inspectMcp', id=number, session=session, connectionId=identity, launch=launch)
    result = finish(number)
    assert 'error' not in result, result.get('error')
    return result['mcpInspection']['token']


def enabled(token, names):
    return call('enableMcp', session=session, token=token, names=names)


requests = []
aliases = []
ids = []


class ModelFixture(BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def do_POST(self):
        payload = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        requests.append(payload)
        tools = payload['messages'][-1]['role'] == 'user'
        delta = {'tool_calls': [{'index': index, 'id': f'external-{index}', 'type': 'function', 'function': {
            'name': alias, 'arguments': json.dumps({'text': f'synthetic server {index}'})}}
            for index, alias in enumerate(aliases)]} if tools else {'content': 'Both results handled.'}
        chunks = [{'choices': [{'delta': delta, 'finish_reason': None}]},
                  {'choices': [{'delta': {}, 'finish_reason': 'tool_calls' if tools else 'stop'}]},
                  {'choices': [], 'usage': {'prompt_tokens': 100, 'completion_tokens': 20, 'total_tokens': 120}}]
        self.send_response(200)
        self.send_header('Content-Type', 'text/event-stream')
        self.end_headers()
        self.wfile.write((''.join('data: ' + json.dumps(c) + '\n\n' for c in chunks) + 'data: [DONE]\n\n').encode())


call('bootstrap')
state_file = directory / 'state.json'
try:
    if args.stage == 'save':
        server = ThreadingHTTPServer(('127.0.0.1', 0), ModelFixture)
        threading.Thread(target=server.serve_forever, daemon=True).start()
        try:
            policy = call('memories')['automaticPolicy']
            call('setAutomaticMemory', enabled=False, revision=policy['revision'])
            call('configure', preferences={'baseUrl': f'http://127.0.0.1:{server.server_port}/v1', 'model': 'fixture'}, apiKey='', rememberConnection=False)
            folder = directory / 'project'
            folder.mkdir()
            session = call('createSession', kind='project', path=str(folder))['session']['id']
            fixture_args = [str(root / 'scripts/mock-mcp.mjs')]
            alpha_launch = {'label': 'Alpha', 'executable': shutil.which('node'), 'args': fixture_args + ['--mode=crash-on-call']}
            beta_launch = {'label': 'Beta', 'executable': shutil.which('node'), 'args': fixture_args}
            alpha = enabled(inspect(1, '', alpha_launch), ['echo'])
            token = inspect(2, '', beta_launch)
            refused = envelope('enableMcp', session=session, token=token, names=['echo', 'second'])
            assert not refused['ok'] and 'Disable' in refused['error'], refused
            # Free a sibling slot without consuming the pending review.
            call('disableMcp', session=session, connectionId=alpha['id'], revision=alpha['revision'])
            beta = enabled(token, ['echo', 'second'])
            assert len(beta['tools']) == 2
            beta = enabled(inspect(3, beta['id'], beta_launch), ['echo'])
            alpha = enabled(inspect(4, alpha['id'], alpha_launch), ['echo'])
            ids[:] = [alpha['id'], beta['id']]
            context = call('context', session=session, input='try both')
            aliases[:] = [next(t['name'] for t in context['tools'] if identity.replace('-', '') in t['name']) for identity in ids]
            assert len(context['tools']) == 8 and aliases[0] != aliases[1]
            before = starts()
            call('start', id=5, session=session, input='try both')
            assert 'error' not in finish(5, approve=True)
            assert starts() == before + 2
            records = call('messagesPage', session=session)['items'][-1]['metadata']['agent']['tools']
            assert [r['mcp']['connectionId'] for r in records] == ids
            assert all(r['status'] == 'completed' for r in records)
            assert {t['function']['name'] for t in requests[0]['tools']} >= set(aliases)
            # One reviewed server crashes only at invocation; the other still runs.
            (folder / 'mcp-call-crash').write_text('')
            call('start', id=6, session=session, input='try both with one unavailable')
            assert 'error' not in finish(6, approve=True)
            records = call('messagesPage', session=session)['items'][-1]['metadata']['agent']['tools']
            assert records[0]['status'] == 'error' and records[1]['status'] == 'completed'
            assert json.loads(records[1]['content'])['text'] == 'synthetic server 1'
            (folder / 'mcp-call-crash').unlink()
            call('forgetMcp', session=session, connectionId=alpha['id'], revision=alpha['revision'])
            settings = call('mcpSettings', session=session)
            assert settings['connections'] == [beta] and settings['connection'] is None
            assert call('context', session=session, input='hello')['tools'][-1]['name'] == aliases[1]
            state_file.write_text(json.dumps({'session': session, 'folder': str(folder), 'beta': beta, 'alias': aliases[1], 'starts': starts()}))
        finally:
            server.shutdown()
    else:
        state = json.loads(state_file.read_text())
        session, folder = state['session'], Path(state['folder'])
        beta = state['beta']
        assert call('mcpSettings', session=session)['connections'] == [beta]
        assert call('context', session=session, input='hello')['tools'][-1]['name'] == state['alias']
        assert starts() == state['starts'], 'Restart or preparation launched a server'
        exported = directory / 'trajectory.json'
        call('export', session=session, path=str(exported), format='json')
        assert all(name in exported.read_text() for name in ['Alpha', 'Beta', 'connectionId'])
        call('forgetMcp', session=session, connectionId=beta['id'], revision=beta['revision'])
        assert call('mcpSettings', session=session)['connections'] == []
        assert starts() == state['starts']
finally:
    call('shutdown')
with sqlite3.connect(directory / 'data/dolores.db') as db:
    assert db.execute('PRAGMA user_version').fetchone()[0] == 16
print(json.dumps({'ok': True, 'stage': args.stage, 'modelFixtureRequests': len(requests), 'liveProviderRequests': 0, 'schema': 16}))
