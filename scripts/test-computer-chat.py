"""Exercise ordinary-chat computer use through the normal Windows bridge.

Uses a disposable local form and deterministic loopback provider. No provider
credentials or personal profile are read. Public receipts exclude window titles
and request bodies. Native input is restricted to this fixture's text entry.
"""
import argparse
import json
import runpy
import subprocess
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

from desktop_test_support import ROOT, NativeHost, fixture_state, wait_for_fixture_focus


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', required=True, type=Path)
    parser.add_argument('--kind', choices=['side', 'temporary', 'project'], default='side')
    parser.add_argument('--same-process', action='store_true', help='Host the disposable window in the bridge process to exercise self input')
    args = parser.parse_args()
    directory = args.directory.resolve()
    if not args.directory.is_absolute() or not directory.is_relative_to(ROOT / 'output'):
        raise SystemExit('Fresh absolute output/ directory required')
    directory.mkdir(parents=True, exist_ok=False)
    state = {'mode': 'observe', 'requests': 0, 'images': 0, 'errors': []}

    class Provider(BaseHTTPRequestHandler):
        def log_message(self, *_):
            pass

        def do_POST(self):
            try:
                body = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
                state['requests'] += 1
                names = [t['function']['name'] for t in body.get('tools', [])]
                assert 'computer' in names, f'Ordinary {args.kind} chat did not advertise computer; tools={names}'
                if args.kind == 'side':
                    assert not set(names) & {'read_text_file', 'write_text_file', 'run_command', 'command_session', 'delegate_tasks'}, 'Computer access must not enable folder tools'
                assert 'No Settings handoff is required' in body['messages'][0]['content']
                # Only receipts from this turn drive the scripted provider.
                messages = body['messages']
                last_user = max(i for i, m in enumerate(messages) if m['role'] == 'user' and isinstance(m['content'], str) and m['content'].startswith('Fixture computer task:'))
                current = messages[last_user:]
                receipts = [json.loads(m['content']) for m in current if m['role'] == 'tool']
                images = [p for m in current if isinstance(m['content'], list) for p in m['content'] if p.get('type') == 'image_url']
                state['images'] = max(state['images'], len(images))
                action = None
                if not receipts:
                    action = {'operation': 'list'}
                elif len(receipts) == 1:
                    window = next(w for w in receipts[0]['windows'] if w['title'] == 'Dolores local form fixture')
                    assert window['isDolores'] == args.same_process
                    action = {'operation': 'observe', 'window': window['window']}
                elif len(receipts) == 2 and state['mode'] != 'observe':
                    assert images, 'Capture did not reach the model as image evidence'
                    if state['mode'] == 'click':
                        observation = receipts[1]['capture']['observation']
                        entry = fixture_state(directory)['entry']
                        action = {'operation': 'click', 'capture': receipts[1]['capture']['id'], 'x': int((entry['x'] - observation['geometry']['left']) * observation['width'] / observation['originalWidth']), 'y': int((entry['y'] - observation['geometry']['top']) * observation['height'] / observation['originalHeight'])}
                        state['click'] = action
                    else:
                        action = {'operation': 'type', 'capture': receipts[1]['capture']['id'], 'text': 'Verified ordinary chat'}
                elif len(receipts) == 3 and state['mode'] in ['input', 'click']:
                    action = {'operation': 'observe', 'window': receipts[1]['window']}
                else:
                    assert images, 'Final answer has no image evidence'
                delta = {'tool_calls': [{'index': 0, 'id': f'computer-{len(receipts)}', 'type': 'function', 'function': {'name': 'computer', 'arguments': json.dumps(action)}}]} if action else {'content': 'Inspected the local form using fresh screenshot evidence.'}
                finish = 'tool_calls' if action else 'stop'
            except Exception as error:
                state['errors'].append(str(error))
                self.send_error(500)
                return
            self.send_response(200)
            self.send_header('Content-Type', 'text/event-stream')
            self.end_headers()
            for chunk in [{'choices': [{'index': 0, 'delta': delta, 'finish_reason': None}]}, {'choices': [{'index': 0, 'delta': {}, 'finish_reason': finish}]}]:
                self.wfile.write(('data: ' + json.dumps(chunk) + '\n\n').encode())
            self.wfile.write(b'data: [DONE]\n\n')

    server = ThreadingHTTPServer(('127.0.0.1', 0), Provider)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    if args.same_process:
        fixture = None  # The entry point hosts Tk on the main thread.
    else:
        fixture = subprocess.Popen([sys.executable, str(ROOT / 'scripts/desktop-form-fixture.py'), '--directory', str(directory)], creationflags=subprocess.CREATE_NO_WINDOW)
    client = None
    try:
        deadline = time.monotonic() + 10
        while not (directory / 'form-state.json').is_file():
            assert args.same_process or fixture.poll() is None, 'Owned fixture exited before publishing its state'
            assert time.monotonic() < deadline, 'Owned fixture startup exceeded ten seconds'
            time.sleep(.05)
        client = NativeHost(directory)
        call = client.call
        call('bootstrap')
        policy = call('memories')['automaticPolicy']
        call('setAutomaticMemory', enabled=False, revision=policy['revision'])
        call('configure', preferences={'baseUrl': f'http://127.0.0.1:{server.server_port}/v1', 'model': 'fixture-vision'}, enabledModels=['fixture-vision'], apiKey='', rememberConnection=False)
        call('setImageModels', models=['fixture-vision'])
        folder = directory / 'workspace'
        folder.mkdir()
        session = call('createSession', kind=args.kind, **({'path': str(folder)} if args.kind == 'project' else {}))['session']['id']
        call('saveScopedSettings', session=session, scope='thread', revision=0, patch={'generation': {'maxOutputTokens': 256, 'timeoutSeconds': 15}, 'task': {'modelCalls': 8, 'toolCalls': 8, 'segments': 1, 'elapsedSeconds': 60}})
        fixture_state(directory)
        results = []
        for index, mode in enumerate(['observe', 'denied', 'input'] + (['click'] if args.same_process else []), 1):
            state.update(mode=mode, requests=0, images=0)
            approvals = []
            def approve(event):
                if event['type'] != 'toolApproval':
                    return
                request = event['request']
                approvals.append(request['name'])
                action = json.loads(request['query'])
                safe = request['name'] == 'computer' and action['operation'] == 'type' and action['text'] == 'Verified ordinary chat' and fixture_state(directory)['text'] == ''
                if mode == 'click':
                    safe = request['name'] == 'computer' and action == state['click']
                if safe and mode in ['input', 'click']:
                    capture = next(c for c in call('desktopState', session=session)['captures'] if c['id'] == action['capture'])
                    wait_for_fixture_focus(capture['observation']['target'], seconds=20)
                call('approveTool', id=index, callId=request['callId'], allow=safe and mode in ['input', 'click'])
            call('start', id=index, session=session, input=f'Fixture computer task: {mode}. Inspect only the Dolores local form fixture. For input type Verified ordinary chat once, do not save, then observe again.')
            done, events = client.finish(index, callback=approve, seconds=65)
            assert not state['errors'], state['errors']
            assert not done.get('error'), done.get('error')
            latest = call('messagesPage', session=session)['items'][-1]
            metadata = latest.get('metadata') or {}
            tools = metadata.get('agent', {}).get('tools', [])
            form = fixture_state(directory)
            assert form['saveClicks'] == 0
            assert state['images'] >= 1
            assert all(t['name'] == 'computer' for t in tools)
            if mode == 'denied':
                assert (metadata.get('paused') or {}).get('reason') == 'desktopReview'
                assert state['requests'] == 3 and form['text'] == ''
            else:
                assert not metadata.get('paused'), metadata.get('paused')
                assert form['text'] == ('Verified ordinary chat' if mode in ['input', 'click'] else '')
                assert len([t for t in tools if t.get('parts')]) == (2 if mode in ['input', 'click'] else 1)
            assert approvals == ([] if mode == 'observe' else ['computer'])
            assert call('desktopState', session=session)['access'] is None, 'No persisted desktop grant should be required'
            results.append({'case': mode, 'modelCalls': state['requests'], 'imageMessages': state['images'], 'inputReviews': len(approvals), 'passed': True})
        result = {'passed': True, 'normalBridge': True, 'realModel': False, 'kind': args.kind, 'sameProcess': args.same_process, 'cases': results}
        (directory / 'public-result.json').write_text(json.dumps(result, indent=2))
        print(json.dumps(result))
    finally:
        if client:
            client.close()
        if args.same_process:
            (directory / 'form-command.json').write_text(json.dumps({'operation': 'close'}))
        else:
            fixture.terminate()
            fixture.wait(5)
        server.shutdown()
        server.server_close()


if __name__ == '__main__':
    if '--same-process' not in sys.argv:
        main()
    else:
        # Tk must live and be destroyed on the main thread. The bridge/client
        # runs on a worker; both still have exactly the same native process ID.
        import traceback
        directory = Path(sys.argv[sys.argv.index('--directory') + 1]).resolve()
        outcome = []
        def work():
            try:
                main()
            except BaseException:
                traceback.print_exc()
                outcome.append(1)
            finally:
                if directory.is_dir():
                    (directory / 'form-command.json').write_text(json.dumps({'operation': 'close'}))
        worker = threading.Thread(target=work, daemon=True)
        worker.start()
        deadline = time.monotonic() + 10
        while not directory.is_dir() and worker.is_alive() and time.monotonic() < deadline:
            time.sleep(.01)
        if worker.is_alive() and directory.is_dir():
            sys.argv = ['desktop-form-fixture.py', '--directory', str(directory)]
            runpy.run_path(str(ROOT / 'scripts/desktop-form-fixture.py'), run_name='__main__')
        worker.join(70)
        assert not worker.is_alive(), 'Same-process fixture worker did not finish'
        raise SystemExit(max(outcome, default=0))
