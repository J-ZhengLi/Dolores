"""Normal Windows bridge + loopback HTTP fixture; no model or user data is used."""
import ctypes
import contextlib
import json
import os
from pathlib import Path
import tempfile
import subprocess
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


def main():
    root = Path(__file__).resolve().parent.parent
    bundle = root / 'apps/dolores_flutter/build/windows/x64/runner/Release'
    requests = []

    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *args):
            pass

        def do_POST(self):
            size = int(self.headers['Content-Length'])
            assert 0 < size <= 128 * 1024
            payload = json.loads(self.rfile.read(size))
            requests.append(payload)
            self.send_response(200)
            self.send_header('Content-Type', 'text/event-stream')
            self.end_headers()
            for item in [
                {'choices': [{'delta': {'content': 'Fixture reply.'}, 'finish_reason': None}]},
                {'choices': [{'delta': {}, 'finish_reason': 'stop'}]},
            ]:
                self.wfile.write(f'data: {json.dumps(item)}\n\n'.encode())
            self.wfile.write(b'data: [DONE]\n\n')

    with contextlib.nullcontext(sys.argv[1]) as temporary:
        os.environ['DOLORES_DATA_DIR'] = str(Path(temporary) / 'data')
        os.environ['DOLORES_GLOBAL_SKILLS_DIR'] = str(Path(temporary) / 'skills')
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

        server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
        worker = threading.Thread(target=server.serve_forever, daemon=True)
        worker.start()
        try:
            state = call('bootstrap')
            assert not envelope('start', id=1, input='No connection')['ok']
            assert call('bootstrap')['sessions'] == state['sessions']
            policy = call('memories')['automaticPolicy']
            call('setAutomaticMemory', enabled=False, revision=policy['revision'])
            endpoint = f'http://127.0.0.1:{server.server_port}/v1'
            preferences = {'baseUrl': endpoint, 'model': 'fixture'}
            call('configure', preferences=preferences, enabledModels=['fixture'], apiKey='synthetic', rememberConnection=False)
            call('setRequestSettings', settings={'maxOutputTokens': 512, 'timeoutSeconds': 30})
            call('setModelRequestSettings', preferences=preferences, settings={'maxOutputTokens': 1024, 'timeoutSeconds': 40})
            folder = Path(temporary) / 'project'
            folder.mkdir()
            session = call('createSession', kind='project', path=str(folder))['session']['id']
            generation = lambda count: {'generation': {'maxOutputTokens': count, 'timeoutSeconds': 30}, 'interaction': None}
            call('saveScopedSettings', session=session, scope='project', revision=0, patch=generation(256))
            call('saveScopedSettings', session=session, scope='thread', revision=0, patch=generation(128))

            def turn(identity, count, origin, window=131072):
                preview = call('context', session=session, input='Short fixture request', tools=True)
                assert preview['effectiveSettings']['request']['maxOutputTokens'] == count
                call('start', id=identity, session=session, input='Short fixture request')
                deadline = time.monotonic() + 10
                done = None
                while time.monotonic() < deadline and done is None:
                    for event in call('poll', id=identity):
                        assert event['type'] != 'toolApproval'
                        if event['type'] == 'done':
                            done = event
                    time.sleep(.01)
                assert done is not None and not done.get('error'), done
                assert requests[-1]['max_tokens'] == count
                assert 'Dolores interaction policy:' in requests[-1]['messages'][0]['content']
                run = call('runs', session=session)[0]
                assert run['settings']['maxOutputTokens'] == count
                assert run['effectiveSettings']['requestOrigin'] == origin
                assert run['effectiveSettings']['contextWindowTokens'] == window
                return run

            frozen = turn(2, 128, 'thread')
            stale = envelope('saveScopedSettings', session=session, scope='thread', revision=0, patch=generation(768))
            assert not stale['ok'] and 'Refresh' in stale['error']
            assert len(requests) == 1
            call('saveScopedSettings', session=session, scope='thread', revision=1, patch={})
            turn(3, 256, 'project')
            call('saveScopedSettings', session=session, scope='project', revision=1, patch={})
            turn(4, 1024, 'Model profile')
            runs = call('runs', session=session)
            assert next(run for run in runs if run['id'] == frozen['id']) == frozen
            assert len(requests) == 3
            call('saveScopedSettings', session=session, scope='thread', revision=2, patch=generation(16384))
            call('configure', preferences=preferences, enabledModels=['fixture'], modelContexts={'fixture': 8192}, apiKey='synthetic', rememberConnection=False)
            view = call('scopedSettings', session=session)
            assert view['validationError'] and view['effective']['contextWindowTokens'] == 8192
            assert not envelope('start', id=5, session=session, input='Invalid reservation')['ok']
            assert len(requests) == 3 and len(call('runs', session=session)) == 3
            call('saveScopedSettings', session=session, scope='thread', revision=3, patch={})
            turn(6, 1024, 'Model profile', 8192)
            print(json.dumps({'passed': True, 'wireRequests': 4, 'precedence': ['thread', 'project', 'Model profile'], 'staleSaveInert': True, 'frozenRunUnchanged': True, 'disconnectedStartInert': True, 'changedWindowRecovery': True}))
        finally:
            call('shutdown')
            server.shutdown()
            server.server_close()
            worker.join(timeout=2)
            loader.close()


if __name__ == '__main__':
    if len(sys.argv) == 1:
        # The child exits before cleanup, releasing the host's database lock.
        with tempfile.TemporaryDirectory(prefix='dolores-scoped-fixture-') as directory:
            subprocess.run([sys.executable, str(Path(__file__).resolve()), directory], check=True)
    else:
        main()
