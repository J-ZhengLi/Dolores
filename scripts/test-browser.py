"""Normal Windows release: browser authority, evidence, failure and owned cleanup.

Only synthetic local pages/provider fixtures are used; no keys or user history.
"""
import ctypes
import json
import os
from pathlib import Path
import subprocess
import tempfile
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from desktop_resource_probe import children, memory


def main():
    root = Path(__file__).resolve().parents[1]
    bundle = root / 'apps/dolores_flutter/build/windows/x64/runner/Release'
    with tempfile.TemporaryDirectory(ignore_cleanup_errors=True) as temporary:
        directory = Path(temporary)
        os.environ['DOLORES_DATA_DIR'] = str(directory / 'data')
        os.environ['DOLORES_GLOBAL_SKILLS_DIR'] = str(directory / 'skills')
        os.environ['DOLORES_BROWSER_ADAPTER_DIR'] = os.environ.get('DOLORES_TEST_BROWSER_ADAPTER_DIR', str(root / 'adapters/browser'))
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
            result = envelope(command, **fields)
            assert result['ok'], result.get('error')
            return result.get('result')

        mode = 'basic'
        requests = []
        navigation_started = threading.Event()
        active_cost = None

        def cost():
            owned = children(os.getpid(), names=None)
            values = [memory(pid) or {} for pid in owned]
            return {'workers': len(owned), 'workingBytes': sum(v.get('workingBytes', 0) for v in values),
                    'privateBytes': sum(v.get('privateBytes', 0) for v in values),
                    'limit': 'Point-in-time sample of this test host descendants; short-lived processes can be missed.'}

        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *args):
                pass

            def do_GET(self):
                if self.path == '/slow':
                    navigation_started.set()
                    time.sleep(10)
                self.send_response(200)
                self.send_header('Content-Type', 'text/html')
                self.end_headers()
                try:
                    self.wfile.write(b'<title>Dolores fixture</title><label>Name <input></label><button onclick="document.querySelector(\'p\').textContent=document.querySelector(\'input\').value">Preview</button><p>Waiting</p>')
                except (BrokenPipeError, ConnectionResetError, ConnectionAbortedError):
                    pass

            def do_POST(self):
                payload = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
                requests.append(payload)
                records = []
                for m in payload['messages']:
                    if m['role'] == 'tool':
                        try: records.append(json.loads(m['content']))
                        except json.JSONDecodeError:
                            records.append({'literal': m['content']})
                browser = [r for r in records if 'outcome' in r]
                n = len(browser)
                latest = browser[-1] if browser else {}
                args = None
                name = 'browser'
                parent = not any('Child goal:\n' in str(m.get('content', '')) for m in payload['messages'])
                if not parent:
                    assert not any(t['function']['name'] == 'browser' for t in payload['tools'])
                    if not records:
                        name, args = 'read_text_file', {'path': 'proof.txt'}
                elif mode == 'combined' and not records:
                    name, args = 'delegate_tasks', {'tasks': [{'goal': 'Read proof.txt and report its literal evidence.', 'scope': 'proof.txt', 'readOnly': True}]}
                elif not browser:
                    args = {'operation': 'open', 'url': f'http://127.0.0.1:{server.server_port}/' + ('slow' if mode == 'stop' else '')}
                elif mode in ['basic', 'limit', 'combined']:
                    if n == 1:
                        args = {'operation': 'fill', 'ref': 'e1', 'state': latest['state'], 'text': 'Synthetic value'}
                    elif n == 2:
                        args = {'operation': 'click', 'ref': 'e2', 'state': browser[0]['state']}
                    elif n == 3:
                        assert latest['outcome'] == 'stale' and latest['actionDispatched'] is False
                        args = {'operation': 'click', 'ref': 'e2', 'state': latest['state']}
                    elif n == 4:
                        args = {'operation': 'screenshot', 'state': latest['state']}
                    elif n == 5 and len(records) == (6 if mode == 'combined' else 5):
                        name, args = 'read_text_file', {'path': 'proof.txt'}
                delta = {'tool_calls': [{'index': 0, 'id': f'call_{len(records)}', 'type': 'function', 'function': {'name': name, 'arguments': json.dumps(args)}}]} if args else {'content': 'Fixture finished. Inspect browser and file receipts.'}
                self.send_response(200)
                self.send_header('Content-Type', 'text/event-stream')
                self.end_headers()
                try:
                    for item in [{'choices': [{'delta': delta, 'finish_reason': None}]}, {'choices': [{'delta': {}, 'finish_reason': 'tool_calls' if args else 'stop'}]}]:
                        self.wfile.write(f'data: {json.dumps(item)}\n\n'.encode())
                    self.wfile.write(b'data: [DONE]\n\n')
                except (BrokenPipeError, ConnectionResetError, ConnectionAbortedError):
                    pass

        server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
        threading.Thread(target=server.serve_forever, daemon=True).start()

        def create(name, models=12, tools=12):
            folder = directory / name
            folder.mkdir()
            (folder / 'proof.txt').write_text('FILES_STILL_USABLE')
            session = call('createSession', kind='project', path=str(folder))['session']['id']
            call('saveScopedSettings', session=session, scope='thread', revision=0,
                 patch={'task': {'modelCalls': models, 'toolCalls': tools, 'segments': 3, 'elapsedSeconds': 60}})
            policy = call('taskPermissions', session=session)
            call('setTaskPermissions', session=session, revision=policy['revision'], policy={'mode': 'fullAccess', 'grants': [], 'expiresAt': None})
            return session

        def finish(identity, stop=False):
            nonlocal active_cost
            approvals = []
            end = time.monotonic() + 40
            cancelled = False
            while time.monotonic() < end:
                for event in call('poll', id=identity):
                    if event['type'] == 'toolApproval':
                        request = event['request']
                        approvals.append(request)
                        if active_cost is None and request['name'] == 'browser':
                            active_cost = cost()
                        call('approveTool', id=identity, callId=request['callId'], allow=True)
                    if event['type'] == 'done':
                        return event, approvals
                if stop and navigation_started.is_set() and not cancelled:
                    call('cancel', id=identity)
                    cancelled = True
                time.sleep(.02)
            raise AssertionError('Browser run did not finish in its fixture deadline')

        try:
            call('bootstrap')
            assert call('browserSettings')['available'] is True
            policy = call('memories')['automaticPolicy']
            call('setAutomaticMemory', enabled=False, revision=policy['revision'])
            call('configure', preferences={'baseUrl': f'http://127.0.0.1:{server.server_port}/v1', 'model': 'fixture'}, apiKey='', rememberConnection=False)
            s = create('basic')
            call('start', id=1, session=s, input='Use the synthetic browser and inspect a local file.')
            done, approvals = finish(1)
            assert 'error' not in done, done
            assert cost()['workers'] == 0, 'Owned browser remains after task completion'
            # Only fill/click require fresh review under Full access, including stale clicks.
            assert [json.loads(a['query'])['operation'] for a in approvals] == ['fill', 'click', 'click']
            message = call('messagesPage', session=s)['items'][-1]
            records = message['metadata']['agent']['tools']
            assert len(records) == 6 and all(r['status'] == 'completed' for r in records[:5]) and records[-1]['status'] == 'read', records
            receipt = json.loads(records[4]['content'])
            preview = call('browserCapture', capture=receipt['capture'])
            assert preview['data'] and 'FILES_STILL_USABLE' in records[-1]['content']
            assert not envelope('browserCapture', capture='../../secret')['ok']
            # Saved capture survives owned browser cleanup and supports actionable missing-file errors.
            capture = directory / 'data/browser-captures' / (receipt['capture'] + '.jpg')
            capture.unlink()
            assert 'unavailable' in envelope('browserCapture', capture=receipt['capture'])['error']
            mode = 'limit'
            s = create('limit', models=2, tools=2)
            call('start', id=2, session=s, input='Deliberately exhaust the model step budget after opening a page.')
            done, approvals = finish(2)
            assert 'error' not in done and not approvals
            assert call('messagesPage', session=s)['items'][-1]['metadata']['paused']['reason'] == 'stepLimit'
            mode = 'stop'
            s = create('stop')
            began = time.monotonic()
            call('start', id=3, session=s, input='Stop during slow synthetic navigation.')
            done, approvals = finish(3, stop=True)
            assert 'error' in done and time.monotonic() - began < 8, done
            assert not approvals
            # Browser failure/Stop leaves file tools and the same working session usable.
            mode = 'basic'
            call('start', id=4, session=s, input='Start a fresh browser and inspect the local file.')
            done, _ = finish(4)
            assert 'error' not in done
            mode = 'combined'
            s = create('combined')
            call('start', id=5, session=s, input='Delegate a scoped read, then use the synthetic browser and verify the file.')
            done, _ = finish(5)
            assert 'error' not in done
            saved = call('messagesPage', session=s)['items'][-1]['metadata']['agent']['tools']
            assert saved[0]['name'] == 'delegate_tasks' and json.loads(saved[0]['content'])['children'][0]['status'] == 'reported'
            assert cost()['workers'] == 0
            print(json.dumps({'normalReleaseHost': True, 'fullAccessFreshInputReviews': 3,
                              'staleNoReplay': True, 'capturePreviewAndMissingRecovery': True,
                              'stepLimitRecovery': True, 'stopDuringNavigation': True,
                              'fileToolsRemainUsable': True, 'parentChildBrowserFlow': True,
                              'activeBrowserSample': active_cost, 'idleBrowserWorkers': 0}))
        finally:
            call('shutdown')
            server.shutdown()
            loader.close()


if __name__ == '__main__':
    main()
