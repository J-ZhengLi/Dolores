"""Opt-in isolated normal-app computer-use check with a configured real model.

Only the saved connection is read. The real key stays in memory behind a bounded
loopback relay; the disposable app remembers a public local placeholder instead.
No source history or request/response bodies are copied into public receipts.
"""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import sys
import threading
import time
import urllib.error
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

import desktop
from desktop_test_support import NativeHost


def seed_profile(directory, model, port, navigation_hint):
    host = NativeHost(directory)
    try:
        call = host.call
        call('bootstrap')
        policy = call('memories')['automaticPolicy']
        call('setAutomaticMemory', enabled=False, revision=policy['revision'])
        call('configure', preferences={'baseUrl': f'http://127.0.0.1:{port}/v1', 'model': model}, apiKey='public-local-relay', rememberConnection=True, enabledModels=[model])
        call('setImageModels', models=[model])
        call('saveAppearance', theme='dark')
        session = call('createSession', kind='side')['session']['id']
        call('saveScopedSettings', session=session, scope='thread', revision=0, patch={'generation': {'maxOutputTokens': 4096, 'timeoutSeconds': 90}, 'task': {'modelCalls': 20, 'toolCalls': 20, 'segments': 1, 'elapsedSeconds': 600}})
        prompt = "Use computer use to switch this Dolores window from dark to light mode, then close Settings, reveal and inspect the bottom navigation dock. Use fresh screenshots and automatic input. Only operate Dolores; do not change model/access settings. Stop when verified and briefly describe what you see."
        if navigation_hint:
            prompt += " Settings is in the bottom dock, revealed by clicking its thin bottom-center handle. Use returned screenshot pixel dimensions. Then choose Settings, General, Light."
        call('saveDraft', session=session, text=prompt)
    finally:
        host.close()
    print(session)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', required=True, type=Path)
    parser.add_argument('--directory', required=True, type=Path)
    parser.add_argument('--model', required=True)
    parser.add_argument('--navigation-hint', action='store_true')
    args = parser.parse_args()
    desktop.require_normal_build()
    directory = args.directory.resolve()
    assert not directory.exists() and directory.is_relative_to(desktop.ROOT / 'output')
    source = args.source.resolve()
    spec = importlib.util.spec_from_file_location('live_connection', desktop.ROOT / 'scripts/test-workspace-editor.py')
    support = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(support)
    before = support.digest(source.parent)
    with sqlite3.connect(source.as_uri() + '?mode=ro', uri=True) as db:
        assert args.model in json.loads(db.execute('SELECT models FROM model_choices').fetchone()[0])
    endpoint, key = support.connection(source.parent)
    directory.mkdir(parents=True)
    counts = {'requests': 0, 'statuses': []}
    lock = threading.Lock()

    class Relay(BaseHTTPRequestHandler):
        def log_message(self, *_):
            pass

        def do_POST(self):
            with lock:
                counts['requests'] += 1
                allowed = counts['requests'] <= 24
            if not allowed or self.path != '/v1/chat/completions':
                self.send_error(429)
                return
            size = int(self.headers.get('Content-Length', '0'))
            if size > 12 * 1024 * 1024:
                self.send_error(413)
                return
            body = json.loads(self.rfile.read(size))
            body['model'] = args.model
            body['max_tokens'] = min(body.get('max_tokens', 4096), 4096)
            request = urllib.request.Request(endpoint.rstrip('/') + '/chat/completions', data=json.dumps(body).encode(), headers={'Content-Type': 'application/json', 'Authorization': 'Bearer ' + key})
            try:
                response = urllib.request.urlopen(request, timeout=90)
            except urllib.error.HTTPError as error:
                response = error
            except Exception:
                self.send_error(502)
                counts['statuses'].append(502)
                return
            with response:
                counts['statuses'].append(response.status)
                self.send_response(response.status)
                self.send_header('Content-Type', response.headers.get('Content-Type', 'application/json'))
                self.end_headers()
                try:
                    while chunk := response.read1(16384):
                        self.wfile.write(chunk)
                        self.wfile.flush()
                except (BrokenPipeError, ConnectionResetError):
                    pass

    server = ThreadingHTTPServer(('127.0.0.1', 0), Relay)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    seeded = subprocess.run([sys.executable, __file__, "--seed", str(directory), args.model, str(server.server_port), str(int(args.navigation_hint))], capture_output=True, text=True, check=True)
    session = seeded.stdout.strip()
    subprocess.run([sys.executable, str(desktop.ROOT / 'scripts/desktop.py'), 'launch', '--data-directory', str(directory / 'data'), '--pid-file', str(directory / 'preview.json')], check=True, env={**os.environ, 'DOLORES_GLOBAL_SKILLS_DIR': str(directory / 'skills')})
    print(json.dumps({'ready': True, 'model': args.model, 'directory': str(directory), 'session': session}), flush=True)
    try:
        deadline = time.monotonic() + 4096
        while not (directory / 'finish').exists() and time.monotonic() < deadline:
            time.sleep(.25)
    finally:
        server.shutdown()
        server.server_close()
        with sqlite3.connect((directory / 'data/dolores.db').as_uri() + '?mode=ro', uri=True) as db:
            theme = db.execute('SELECT theme FROM appearance WHERE id=1').fetchone()[0]
            events = [json.loads(row[0]) for row in db.execute('SELECT data FROM run_events ORDER BY sequence')]
        pauses = [event['data'].get('pauseReason') for event in events if event['kind'] == 'finished']
        result = {**counts, 'model': args.model, 'originalProfileUnchanged': support.digest(source.parent) == before, 'realModel': True, 'navigationHint': args.navigation_hint, 'finalTheme': theme, 'lightModeReached': theme == 'light', 'pauseReasons': pauses}
        (directory / 'relay-result.json').write_text(json.dumps(result, indent=2))
        print(json.dumps(result), flush=True)


if __name__ == '__main__':
    if len(sys.argv) > 1 and sys.argv[1] == "--seed":
        seed_profile(Path(sys.argv[2]), sys.argv[3], int(sys.argv[4]), sys.argv[5] == "1")
    else:
        main()
