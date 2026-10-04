"""Normal release host: scoped children, shared limits and owned cancellation.

Uses isolated synthetic files and a deterministic local model fixture. No keys.
"""
import ctypes
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


def worker(directory, restart=False):
    root = Path(__file__).resolve().parents[1]
    directory = Path(directory)
    os.environ['DOLORES_DATA_DIR'] = str(directory / 'data')
    os.environ['DOLORES_GLOBAL_SKILLS_DIR'] = str(directory / 'skills')
    bundle = root / 'apps/dolores_flutter/build/windows/x64/runner/Release'
    loader = os.add_dll_directory(str(bundle))
    native = ctypes.CDLL(str(bundle / 'dolores_flutter_bridge.dll'))
    native.dolores_call.argtypes = [ctypes.c_void_p, ctypes.c_size_t]
    native.dolores_call.restype = ctypes.c_void_p
    native.dolores_free.argtypes = [ctypes.c_void_p]

    def envelope(command, **fields):
        data = json.dumps({'command': command, **fields}).encode()
        buffer = ctypes.create_string_buffer(data)
        pointer = native.dolores_call(buffer, len(data))
        try:
            return json.loads(ctypes.string_at(pointer))
        finally:
            native.dolores_free(pointer)

    def call(command, **fields):
        value = envelope(command, **fields)
        assert value['ok'], value.get('error')
        return value.get('result')

    if restart:
        try:
            state = call('bootstrap')
            assert state['configured'] is False
            found = False
            for s in state['sessions']:
                workspace = call('workspace', session=s['id'])
                for run in call('runs', session=s['id']):
                    assert run['state'] not in ['prepared', 'running', 'waitingForApproval']
                if workspace.get('root') and Path(workspace['root']).name == 'normal':
                    found = True
                    message = call('messagesPage', session=s['id'])['items'][-1]
                    report = json.loads(message['metadata']['agent']['tools'][0]['content'])
                    assert len(report['children']) == 2 and all(c['status'] == 'reported' for c in report['children'])
                    assert (directory / 'normal' / 'alpha.txt').read_text() == 'ALPHA'
                    assert len(call('changesPage', session=s['id'])['items']) == 2
            assert found
            print(json.dumps({'restartPassed': True, 'childReportsAndChangesRetained': True,
                              'noConnectionOrAutomaticReplay': True}))
        finally:
            call('shutdown')
            loader.close()
        return

    mode = 'normal'
    requests = []
    active = peak = 0
    lock = threading.Lock()
    gates = threading.Event()
    children_started = threading.Event()

    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *args):
            pass

        def do_POST(self):
            nonlocal active, peak
            payload = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
            parent = any(t['function']['name'] == 'delegate_tasks' for t in payload['tools'])
            with lock:
                requests.append(payload)
                if not parent:
                    active += 1
                    peak = max(peak, active)
                    if active == 2:
                        children_started.set()
            try:
                if not parent:
                    assert not any(t['function']['name'] in ['run_command', 'inspect_harness', 'delegate_tasks']
                                   or t['function']['name'].startswith('mcp_tool_') for t in payload['tools'])
                    if mode == 'hang':
                        gates.wait(10)
                    else:
                        time.sleep(.06)
                messages = payload['messages']
                if parent and not any(m['role'] == 'tool' for m in messages):
                    tasks = [{'goal': 'Write ' + name + ' with ' + value, 'scope': name, 'readOnly': False}
                             for name, value in [('alpha.txt', 'ALPHA'), ('beta.txt', 'BETA')]]
                    if mode == 'overlap':
                        tasks[1]['scope'] = 'ALPHA.txt'
                    calls = [('batch', 'delegate_tasks', {'tasks': tasks})]
                elif parent and mode == 'normal' and sum(m['role'] == 'tool' for m in messages) == 1:
                    calls = [(name, 'read_text_file', {'path': name}) for name in ['alpha.txt', 'beta.txt']]
                elif not parent and not any(m['role'] == 'tool' for m in messages):
                    alpha = 'Child goal:\nWrite alpha.txt' in messages[1]['content']
                    name, value = ('alpha.txt', 'ALPHA') if alpha else ('beta.txt', 'BETA')
                    if mode == 'escape':
                        name = 'outside.txt'
                    calls = [('create', 'create_text_file', {'path': name, 'content': value})]
                else:
                    calls = []
                delta = {'tool_calls': [{'index': i, 'id': ident, 'type': 'function',
                         'function': {'name': name, 'arguments': json.dumps(args)}}
                         for i, (ident, name, args) in enumerate(calls)]} if calls else {'content': 'Bounded fixture report; inspect receipts.'}
                self.send_response(200)
                self.send_header('Content-Type', 'text/event-stream')
                self.end_headers()
                for item in [{'choices': [{'delta': delta, 'finish_reason': None}]},
                             {'choices': [{'delta': {}, 'finish_reason': 'tool_calls' if calls else 'stop'}]}]:
                    self.wfile.write(f'data: {json.dumps(item)}\n\n'.encode())
                self.wfile.write(b'data: [DONE]\n\n')
            except (BrokenPipeError, ConnectionResetError, ConnectionAbortedError):
                pass
            finally:
                if not parent:
                    with lock:
                        active -= 1

    server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
    threading.Thread(target=server.serve_forever, daemon=True).start()

    def session(name, models=12, tools=12):
        folder = directory / name
        folder.mkdir()
        s = call('createSession', kind='project', path=str(folder))['session']['id']
        call('saveScopedSettings', session=s, scope='thread', revision=0,
             patch={'task': {'modelCalls': models, 'toolCalls': tools, 'segments': 3, 'elapsedSeconds': 60}})
        return s, folder

    def finish(identity, session_id, action=None):
        deadline = time.monotonic() + 20
        approvals = []
        all_events = []
        stopped = False
        while time.monotonic() < deadline:
            events = call('poll', id=identity)
            all_events.extend(events)
            for event in events:
                if event['type'] == 'toolApproval':
                    request = event['request']
                    approvals.append(request)
                    if action in ['stopApproval', 'revokeApproval'] and request['callId'].startswith('child.'):
                        if action == 'stopApproval':
                            call('cancel', id=identity)
                        else:
                            policy = call('taskPermissions', session=session_id)
                            call('setTaskPermissions', session=session_id, revision=policy['revision'],
                                 policy={'mode': 'review', 'grants': [], 'expiresAt': None})
                        stopped = True
                        assert not envelope('approveTool', id=identity, callId=request['callId'], allow=True)['ok']
                    elif not stopped:
                        call('approveTool', id=identity, callId=request['callId'], allow=True)
                if event['type'] == 'done':
                    return event, approvals, all_events
            if action == 'stopModels' and children_started.is_set() and not stopped:
                call('cancel', id=identity)
                stopped = True
            time.sleep(.01)
        raise AssertionError('Parent/children did not stop within the fixture deadline')

    def evidence(s):
        run = call('runs', session=s)[0]
        return run, call('runEvents', session=s, runId=run['id'])

    try:
        call('bootstrap')
        policy = call('memories')['automaticPolicy']
        call('setAutomaticMemory', enabled=False, revision=policy['revision'])
        call('configure', preferences={'baseUrl': f'http://127.0.0.1:{server.server_port}/v1', 'model': 'fixture'},
             apiKey='', rememberConnection=False)
        s, folder = session('normal')
        call('start', id=1, session=s, input='Delegate independent files, then read to verify.')
        done, approvals, events = finish(1, s)
        assert 'error' not in done and len(approvals) == 5
        assert (folder / 'alpha.txt').read_text() == 'ALPHA' and (folder / 'beta.txt').read_text() == 'BETA'
        assert peak == 2
        message = call('messagesPage', session=s)['items'][-1]
        assert 'paused' not in message['metadata']
        assert message['metadata']['agent']['tools'][0]['status'] == 'completed'
        report = json.loads(message['metadata']['agent']['tools'][0]['content'])
        assert all(c['status'] == 'reported' and c['verified'] is False for c in report['children'])
        assert report['sharedUsage'] == {'modelCalls': 5, 'toolCalls': 3}
        run, log = evidence(s)
        assert len([e for e in log if e['kind'] == 'subagentUsage']) == 2
        assert len(call('changesPage', session=s)['items']) == 2
        side = call('createSession', kind='side')['session']['id']
        assert not any(t['name'] == 'delegate_tasks' for t in call('context', session=side, input='inspect')['tools'])
        mode = 'limit'
        s, folder = session('limit', models=4, tools=4)
        before = len(requests)
        call('start', id=2, session=s, input='Delegate files under a deliberately small total budget.')
        done, _, _ = finish(2, s)
        assert 'error' not in done and len(requests) - before <= 4
        assert call('messagesPage', session=s)['items'][-1]['metadata']['paused']['reason'] == 'stepLimit'
        assert list(folder.glob('*.txt'))
        run, log = evidence(s)
        assert any(e['kind'] == 'subagent' and e['data']['status'] == 'paused' for e in log)
        for identity, mode, action in [(3, 'limit', 'stopApproval'), (4, 'limit', 'revokeApproval'), (5, 'hang', 'stopModels')]:
            children_started.clear()
            gates.clear()
            s, folder = session(str(identity))
            call('start', id=identity, session=s, input='Delegate files, stopping both children before writes.')
            done, _, _ = finish(identity, s, action)
            assert 'error' in done and not list(folder.glob('*.txt')), (action, done, [p.name for p in folder.glob('*.txt')])
            run, log = evidence(s)
            assert sum(e['kind'] == 'subagent' and e['data']['status'] == 'interrupted' for e in log) == 2
            assert not any(e['kind'] == 'toolIntent' and e['data']['callId'].startswith('child.') for e in log)
            gates.set()
        mode = 'escape'
        s, folder = session('escape')
        current = call('taskPermissions', session=s)
        call('setTaskPermissions', session=s, revision=current['revision'], policy={'mode': 'fullAccess', 'grants': [], 'expiresAt': None})
        call('start', id=6, session=s, input='Attempt child scope escape under full access.')
        done, approvals, _ = finish(6, s)
        assert 'error' not in done and not approvals and not list(folder.glob('*.txt'))
        run, log = evidence(s)
        assert not any(e['kind'] == 'toolIntent' and e['data']['callId'].startswith('child.') for e in log)
        mode = 'overlap'
        s, folder = session('overlap')
        call('start', id=7, session=s, input='Conflicting ownership must not start children.')
        done, approvals, events = finish(7, s)
        assert 'error' not in done and not approvals and not any(e['type'] == 'subagent' for e in events)
        assert not list(folder.glob('*.txt'))
        print(json.dumps({'passed': True, 'concurrentChildren': 2, 'basicApprovals': 5,
                          'sharedLimitPauses': True, 'stopAndRevocationNoWrites': True,
                          'fullAccessScopeEscapeBlocked': True, 'conflictingOwnershipBlocked': True}))
    finally:
        gates.set()
        call('shutdown')
        server.shutdown()
        loader.close()


if __name__ == '__main__':
    if len(sys.argv) > 1:
        worker(sys.argv[1], restart='--restart' in sys.argv)
    else:
        with tempfile.TemporaryDirectory() as directory:
            subprocess.run([sys.executable, __file__, directory], check=True)
            subprocess.run([sys.executable, __file__, directory, '--restart'], check=True)
