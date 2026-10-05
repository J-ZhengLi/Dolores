"""Normal release FFI qualification with disposable state and a local SSE fixture.

Run save, then reopen in separate processes with the same --directory under output/.
No configured keys, private chats, browser workers or external model calls.
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


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('stage', choices=['save', 'reopen'])
    parser.add_argument('--directory', type=Path, required=True)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    directory = args.directory.resolve()
    if not args.directory.is_absolute() or not directory.is_relative_to(root / 'output'):
        raise SystemExit('Use an absolute disposable directory under output/.')
    if args.stage == 'save':
        directory.mkdir(parents=True, exist_ok=False)
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
        result = envelope(command, **fields)
        assert result['ok'], result.get('error')
        return result.get('result')

    def state(session):
        return call('modState', session=session, category=1)['state']

    def inject(folder, value):
        with sqlite3.connect(directory / 'data/dolores.db') as db:
            db.execute('UPDATE project_mods SET data=? WHERE root=?',
                       (json.dumps(value), str(folder)))

    def finish(identity):
        events = []
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            for event in call('poll', id=identity):
                events.append(event)
                if event['type'] == 'done':
                    return event, events
            time.sleep(.01)
        raise AssertionError('Bounded fixture deadline exceeded')

    manifest = {'id': 'recovery-hints', 'api': 1, 'stateSchema': 1,
                'hook': 'recovery_hint', 'capabilities': [], 'dependencies': [],
                'title': 'Recovery guidance', 'description': 'Synthetic bounded hint card.'}
    repaired = '(module (func (export "recovery_hint") (param i32) (result i32) local.get 0))'
    server = None
    call('bootstrap')
    try:
        receipt = directory / 'expected.json'
        if args.stage == 'reopen':
            expected = json.loads(receipt.read_text())
            s = state(expected['session'])
            assert s['pending'] is None and s['active'] is None
            assert 'Interrupted activation' in s['recoveryReceipt']
            assert s['events'][:len(expected['events'])] == expected['events']
            assert s['versions'][0]['status'] == 'quarantined'
            assert state(expected['draftSession'])['draft'] == '(module'
            assert 'stopped' in state(expected['draftSession'])['draftNotice']
            assert call('messagesPage', session=expected['session'])['items'] == expected['messages']
            assert call('bootstrap')['preferences'] == expected['preferences']
            print(json.dumps({'restartNoReplay': True, 'partialDraftAndHistoryPreserved': True,
                              'pendingReconciled': True, 'quarantinePreserved': True}))
            return

        requests = []
        arrived = threading.Event()
        mode = 'limited'

        class Provider(BaseHTTPRequestHandler):
            def log_message(self, *_):
                pass

            def do_POST(self):
                payload = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
                requests.append(payload)
                self.send_response(200)
                self.send_header('Content-Type', 'text/event-stream')
                self.end_headers()
                try:
                    text = '(module' if mode == 'stop' or payload.get('max_tokens') == 1024 else 'Synthetic retained task progress.'
                    self.wfile.write(('data: ' + json.dumps({'choices': [{'delta': {'content': text}, 'finish_reason': None}]}) + '\n\n').encode())
                    self.wfile.flush()
                    arrived.set()
                    if mode == 'stop':
                        time.sleep(1)
                    reason = 'stop' if mode == 'ordinary' else 'length'
                    self.wfile.write(('data: ' + json.dumps({'choices': [{'delta': {}, 'finish_reason': reason}]}) + '\n\ndata: [DONE]\n\n').encode())
                except (BrokenPipeError, ConnectionResetError, ConnectionAbortedError):
                    pass

        server = ThreadingHTTPServer(('127.0.0.1', 0), Provider)
        threading.Thread(target=server.serve_forever, daemon=True).start()
        policy = call('memories')['automaticPolicy']
        call('setAutomaticMemory', enabled=False, revision=policy['revision'])
        call('configure', preferences={'baseUrl': f'http://127.0.0.1:{server.server_port}/v1', 'model': 'fixture'},
             apiKey='', rememberConnection=False, enabledModels=['fixture'])
        folder = directory / 'workspace'
        folder.mkdir()
        (folder / 'proof.txt').write_text('PRIVATE_SENTINEL_NOT_FOR_MOD_DRAFT')
        session = call('createSession', kind='project', path=str(folder))['session']['id']
        call('saveScopedSettings', session=session, scope='thread', revision=0,
             patch={'task': {'modelCalls': 4, 'toolCalls': 4, 'segments': 3, 'elapsedSeconds': 30}})
        # Local qualification cannot grant new authority or load unsupported code.
        for patch in [{'capabilities': ['filesystem']}, {'stateSchema': 2}, {'title': 'x' * 81}]:
            assert not envelope('testMod', session=session, revision=0, manifest={**manifest, **patch}, source=repaired)['ok']
        assert not envelope('testMod', session=session, revision=0, manifest=manifest,
                            source='(module (import "host" "file" (func)))')['ok']
        view = call('testMod', session=session, revision=0, manifest=manifest, source=repaired)
        candidate = view['state']['versions'][0]
        assert candidate['results'] == [True] * 6 and candidate['status'] == 'review'
        view = call('activateMod', session=session, revision=view['state']['revision'], identity=candidate['identity'])
        assert view['card']['action'] == 'continue'
        call('start', id=1, session=session, input='Return bounded synthetic progress, no tools.')
        assert not envelope('restoreMod', session=session, revision=view['state']['revision'])['ok']
        done, events = finish(1)
        assert not done.get('error') and any(e['type'] == 'modHint' and e['card']['action'] == 'continue' for e in events)
        message = call('messagesPage', session=session)['items'][-1]
        assert message['metadata']['paused']['reason'] == 'outputLimit'
        run = call('runs', session=session)[0]
        assert any(e['kind'] == 'modHint' for e in call('runEvents', session=session, runId=run['id']))
        # Corrupted active bytes fail health and quarantine, including a full audit.
        s = state(session)
        s['versions'][0]['source'] += ' ;; source drift'
        s['events'] = ['Synthetic retained audit'] * 32
        inject(folder, s)
        recovered = call('modState', session=session, category=1)
        assert recovered['state']['active'] is None
        assert recovered['state']['versions'][0]['status'] == 'quarantined'
        assert recovered['state']['events'] == s['events']
        assert 'source or manifest changed' in recovered['card']['notice']
        mode = 'ordinary'
        call('start', id=2, session=session, input='Ordinary chat after a mod health failure.')
        assert not finish(2)[0].get('error')

        draft_folder = directory / 'draft-workspace'
        draft_folder.mkdir()
        draft_session = call('createSession', kind='project', path=str(draft_folder))['session']['id']
        mode = 'limited'
        call('generateMod', id=3, session=draft_session, revision=0)
        done, _ = finish(3)
        assert done['source'] == '(module' and 'output tokens' in done['error']
        assert state(draft_session)['active'] is None and not state(draft_session)['versions']
        mode = 'stop'
        arrived.clear()
        before = len(requests)
        call('generateMod', id=4, session=draft_session, revision=state(draft_session)['revision'])
        assert arrived.wait(3)
        time.sleep(.1)  # Ensure the synthetic first delta has reached the receiver.
        assert not envelope('setModPolicy', session=draft_session, revision=0, automatic=True)['ok']
        began = time.monotonic()
        call('cancel', id=4)
        done, _ = finish(4)
        assert time.monotonic() - began < 2
        assert done['source'] == '(module' and 'stopped' in done['error']
        assert len(requests) == before + 1
        for request in requests[-2:]:
            assert request['max_tokens'] == 1024
            assert 'PRIVATE_SENTINEL' not in json.dumps(request)
        s = state(session)
        s['pending'] = candidate['identity']  # Interrupted pointer transaction fixture.
        inject(folder, s)
        receipt.write_text(json.dumps({'session': session, 'draftSession': draft_session,
                                      'events': s['events'], 'messages': call('messagesPage', session=session)['items'],
                                      'preferences': call('bootstrap')['preferences']}))
        print(json.dumps({'normalReleaseHost': True, 'fixedRepairAndActivation': True,
                          'pausedPinnedHintDurable': True, 'activeRunConflict': True,
                          'capabilityAndSourceDriftRefused': True, 'fullHistoryRollback': True,
                          'ordinaryChatAfterFailure': True, 'outputLimitRetained': True,
                          'stopPartialAndNoRetry': True, 'providerRequests': len(requests)}))
    finally:
        call('shutdown')
        if server:
            server.shutdown()
            server.server_close()
        loader.close()


if __name__ == '__main__':
    main()
