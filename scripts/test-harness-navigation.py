"""Normal native host: pinned source navigation and private-data-free failed-stream evidence."""
import ctypes
import json
import os
from pathlib import Path
import tempfile
import subprocess
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


def main(directory):
    root = Path(__file__).resolve().parents[1]
    os.environ['DOLORES_DATA_DIR'] = str(directory / 'data')
    os.environ['DOLORES_GLOBAL_SKILLS_DIR'] = str(directory / 'skills')
    release = root / 'apps/dolores_flutter/build/windows/x64/runner/Release'
    loader = os.add_dll_directory(str(release)) if os.name == 'nt' else None
    library = release / ('dolores_flutter_bridge.dll' if os.name == 'nt' else 'libdolores_flutter_bridge.so')
    native = ctypes.CDLL(str(library))
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

    requests = []
    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *args):
            pass
        def do_POST(self):
            self.rfile.read(int(self.headers['Content-Length']))
            requests.append(1)
            self.send_response(200)
            self.send_header('Content-Type', 'text/event-stream')
            self.end_headers()
            if len(requests) <= 2:
                delta = {'tool_calls': [{'index': 0, 'id': 'navigation', 'type': 'function',
                         'function': {'name': 'inspect_harness', 'arguments': json.dumps(navigation)}}]} if len(requests) == 1 else {'content': 'Source inspected.'}
                frame = {'choices': [{'delta': delta, 'finish_reason': 'tool_calls' if len(requests) == 1 else 'stop'}]}
                self.wfile.write(('data: ' + json.dumps(frame) + '\n\ndata: [DONE]\n\n').encode())
                return
            # Complete one valid delta, then disconnect with an incomplete call.
            frame = {'choices': [{'delta': {'content': 'usable', 'reasoning_content': 'PRIVATE_REASONING_SENTINEL',
                     'tool_calls': [{'index': 0, 'id': 'unfinished', 'type': 'function',
                                     'function': {'name': 'read_text_file', 'arguments': '{"path":'}}]}, 'finish_reason': None}]}
            self.wfile.write(('data: ' + json.dumps(frame) + '\n\n').encode())
    server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    try:
        call('bootstrap')
        project = directory / 'unrelated-project'
        project.mkdir()
        (project / 'keep.txt').write_text('preserved')
        session = call('createSession', kind='project', path=str(project))['session']['id']
        inventory = call('harnessInventory', session=session)
        bundle = inventory['sourceBundle']
        navigation = {'source': 'core', 'startLine': 150, 'lineCount': 180,
                      'bundleId': bundle['bundleId'], 'sourceId': inventory['sources'][0]['sourceId']}
        read = call('harnessNavigation', query={'source': 'core', 'startLine': 150, 'lineCount': 180, 'bundleId': bundle['bundleId']})
        assert read['startLine'] == 150 and read['nextLine'] > 270
        search = call('harnessNavigation', query={'action': 'search', 'query': 'ModelProvider', 'symbol': True, 'source': 'core'})
        assert any(item['line'] > 120 for item in search['matches'])
        assert not envelope('harnessNavigation', query={'source': 'core', 'sourceId': 'stale'})['ok']
        assert not envelope('harnessNavigation', query={'source': '../keep.txt'})['ok']
        assert (project / 'keep.txt').read_text() == 'preserved'
        policy = call('memories')['automaticPolicy']
        call('setAutomaticMemory', enabled=False, revision=policy['revision'])
        call('configure', preferences={'baseUrl': f'http://127.0.0.1:{server.server_port}/v1', 'model': 'fixture'}, apiKey='', rememberConnection=False)
        call('start', id=1, session=session, input='inspect the fixture')
        done = None
        receipts = []
        approvals = 0
        deadline = time.monotonic() + 20
        while time.monotonic() < deadline and done is None:
            for event in call('poll', id=1):
                if event['type'] == 'toolApproval':
                    request = event['request']
                    assert request['name'] == 'inspect_harness'
                    assert len(request['query']) > 256
                    approvals += 1
                    call('approveTool', id=1, callId=request['callId'], allow=True)
                if event['type'] == 'toolResult':
                    receipts.append(event['record'])
                if event['type'] == 'done':
                    done = event
            time.sleep(.01)
        assert done and not done.get('error') and approvals == 1, (done, approvals, receipts)
        assert len(receipts) == 1 and receipts[0]['status'] == 'completed'
        assert json.loads(receipts[0]['content'])['nextLine'] > 270
        call('start', id=2, session=session, input='disconnect the fixture')
        done = None
        approvals = 0
        deadline = time.monotonic() + 20
        while time.monotonic() < deadline and done is None:
            for event in call('poll', id=2):
                approvals += int(event['type'] == 'toolApproval')
                if event['type'] == 'done':
                    done = event
            time.sleep(.01)
        assert done and done.get('error') and approvals == 0
        failure = call('harnessInventory', session=session)['recentFailures'][0]
        assert failure['sourceMatch'] == 'matches'
        measured = failure['latestMeasurements']
        assert measured['contentBytes'] == 6 and measured['wireBytes'] > 0
        assert measured['incompleteCall'] is True and measured['finished'] is False
        assert 'PRIVATE_' not in json.dumps(failure)
        assert (project / 'keep.txt').read_text() == 'preserved'
        print(json.dumps({'ok': True, 'fixtureRequests': len(requests), 'liveRequests': 0,
                          'sourceReadBeyond120': True, 'staleSourceRefused': True,
                          'reviewedNavigationWithBothIdentities': True,
                          'incompleteCallNotDispatched': True, 'failedCountersSaved': True,
                          'bundle': {key: bundle[key] for key in ('files', 'compressedBytes', 'sourceBytes')}}))
    finally:
        call('shutdown')
        server.shutdown()
        if loader:
            loader.close()


if __name__ == '__main__':
    if len(sys.argv)>1:
        main(Path(sys.argv[1]))
    else:
        # The native singleton retains SQLite until process exit. Cleanup belongs
        # to a parent after the child exits, rather than hiding a locked-file error.
        with tempfile.TemporaryDirectory() as temporary:
            subprocess.run([sys.executable,__file__,temporary],check=True)
