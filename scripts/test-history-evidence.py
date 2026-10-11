"""Saved command evidence across processes, with keyless or explicit live follow-up.

The seed provider executes only a reviewed public print command. Follow-up must
use its saved receipt without repeating the command. No desktop input is used.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import sqlite3
import subprocess
import sys
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

from desktop_test_support import NativeHost, ROOT


def digest(db):
    names = [r[0] for r in db.execute("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")]
    return {name: hashlib.sha256(repr(sorted(db.execute('SELECT * FROM ' + name).fetchall(), key=repr)).encode()).hexdigest() for name in names}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--directory', type=Path, required=True)
    parser.add_argument('--source', type=Path)
    parser.add_argument('--model', help='Explicit enabled model for an optional live follow-up.')
    parser.add_argument('--phase', choices=['seed', 'verify'])
    args = parser.parse_args()
    directory = args.directory.resolve()
    if not directory.is_relative_to(ROOT / 'output'):
        parser.error('Choose a disposable directory under output.')
    if bool(args.source) != bool(args.model):
        parser.error('Live checks need both --source and --model.')
    if not args.phase:
        directory.mkdir(parents=True, exist_ok=False)
        for phase in ['seed', 'verify']:
            command = [sys.executable, __file__, '--directory', str(directory), '--phase', phase]
            if args.source:
                command += ['--source', str(args.source.resolve()), '--model', args.model]
            subprocess.run(command, check=True, timeout=150)
        return

    cases = [
        {'marker': 'PUBLIC-HISTORY-CHECK-42', 'exit': 0},
        {'marker': 'PUBLIC-HISTORY-FAIL-42', 'exit': 7},
    ]
    requests = []
    current = {}

    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *_):
            pass

        def do_POST(self):
            payload = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
            requests.append(payload)
            if args.phase == 'seed' and not any(m['role'] == 'tool' for m in payload['messages']):
                delta = {'tool_calls': [{'index': 0, 'id': 'public-print', 'type': 'function', 'function': {
                    'name': 'run_command', 'arguments': json.dumps(current['invocation'])}}]}
                finish = 'tool_calls'
            elif args.phase == 'verify':
                text = '\n'.join(m.get('content', '') for m in payload['messages'] if m['role'] == 'assistant')
                assert current['marker'] in text and 'Historical' in text
                delta = {'content': f"It printed {current['marker']} and exited with code {current['exit']}."}
                finish = 'stop'
            else:
                # Keep the marker and exit code out of saved prose: the next
                # request can discover them only from the actual tool receipt.
                delta, finish = {'content': 'Completed the public check.'}, 'stop'
            self.send_response(200)
            self.send_header('Content-Type', 'text/event-stream')
            self.end_headers()
            for item in [{'choices': [{'delta': delta, 'finish_reason': None}]},
                         {'choices': [{'delta': {}, 'finish_reason': finish}]}]:
                self.wfile.write(('data: ' + json.dumps(item) + '\n\n').encode())
            self.wfile.write(b'data: [DONE]\n\n')

    server = None
    original = args.source.resolve() if args.phase == 'verify' and args.source else None
    before = None
    if original:
        with sqlite3.connect(original.as_uri() + '?mode=ro', uri=True) as source:
            before = digest(source)
            with sqlite3.connect(directory / 'data/dolores.db') as target:
                for table in ['preferences', 'model_choices', 'model_contexts', 'request_settings', 'model_request_settings', 'model_images']:
                    rows = source.execute('SELECT * FROM ' + table).fetchall()
                    target.execute('DELETE FROM ' + table)
                    if rows:
                        target.executemany('INSERT INTO ' + table + ' VALUES(' + ','.join('?' for _ in rows[0]) + ')', rows)
        spec = importlib.util.spec_from_file_location('configured_connection', ROOT / 'scripts/test-workspace-editor.py')
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        base, key = module.connection(original.parent)
        with sqlite3.connect(directory / 'data/dolores.db') as db:
            models = json.loads(db.execute('SELECT models FROM model_choices').fetchone()[0])
        assert args.model in models, 'Choose an enabled model.'
    else:
        server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
        threading.Thread(target=server.serve_forever, daemon=True).start()
        base, key, models = f'http://127.0.0.1:{server.server_port}/v1', '', ['fixture']

    host = NativeHost(directory)
    results = []
    try:
        host.call('bootstrap')
        model = args.model if original else 'fixture'
        host.call('configure', preferences={'baseUrl': base, 'model': model}, apiKey=key,
                  rememberConnection=False, enabledModels=models)
        del key
        host.call('setModelRequestSettings', preferences={'baseUrl': base, 'model': model},
                  settings={'maxOutputTokens': 1024, 'timeoutSeconds': 20, 'reasoning': 'providerDefault'})
        policy = host.call('memories')['automaticPolicy']
        if policy['enabled']:
            host.call('setAutomaticMemory', enabled=False, revision=policy['revision'])
        project = directory / 'project'
        if args.phase == 'seed':
            project.mkdir()
        else:
            cases = json.loads((directory / 'seed.json').read_text())
        for identity, case in enumerate(cases, 1):
            current.clear()
            current.update(case)
            if args.phase == 'seed':
                session = host.call('createSession', kind='project', path=str(project))['session']['id']
                case['session'] = session
                invocation = {'program': 'python', 'args': ['-c', f"print('{case['marker']}'); raise SystemExit({case['exit']})"],
                              'timeout_seconds': 10, 'capture_bytes': 4096}
                current['invocation'] = invocation
                prompt = 'Run the public check once. Keep the final acknowledgement short.'
            else:
                session = case['session']
                prompt = ('Using the saved tool evidence, state the exact stdout marker and exit code of the previous command. '
                          'This asks about the earlier result, not a fresh check. Do not use tools or run any command. Reply in one sentence.')
                preview = host.call('context', session=session, input=prompt, tools=True)
                assert case['marker'] in json.dumps(preview['messages'])
                assert 'Historical' in json.dumps(preview['messages'])
            approved = []

            def respond(event):
                if event['type'] not in ['toolApproval', 'userQuestion']:
                    return
                if args.phase != 'seed' or event['type'] != 'toolApproval':
                    host.call('cancel', id=identity)
                    raise AssertionError('Historical inspection must not ask to repeat an action.')
                request = event['request']
                command = request.get('command', {}).get('invocation')
                assert request['name'] == 'run_command'
                assert command == {'program': invocation['program'], 'args': invocation['args']}
                assert not approved, 'Exactly one reviewed public command is allowed.'
                approved.append(command)
                host.call('approveTool', id=identity, callId=request['callId'], allow=True)

            host.call('start', id=identity, session=session, input=prompt)
            done, _ = host.finish(identity, callback=respond, seconds=55)
            assert not done.get('error'), done.get('error')
            messages = host.call('messagesPage', session=session)['items']
            last = messages[-1]
            metadata = last.get('metadata') or {}
            agent = metadata.get('agent') or {}
            if args.phase == 'seed':
                record = agent['tools'][0]
                receipt = json.loads(record['content'])
                assert len(approved) == 1 and len(agent['tools']) == 1
                assert receipt['exitCode'] == case['exit'] and receipt['stdout'].strip() == case['marker']
                assert case['marker'] not in last['content'], 'Marker must come from the receipt, not saved prose.'
            else:
                answer = last['content']
                passed = (case['marker'] in answer
                          and bool(re.search(rf"(?<!\d){case['exit']}(?!\d)", answer))
                          and not agent.get('tools'))
                results.append({'exitCode': case['exit'], 'pass': passed, 'answer': answer[:1000],
                                'usage': agent.get('usageByCall', []), 'modelCalls': agent.get('modelCalls')})
        if args.phase == 'seed':
            (directory / 'seed.json').write_text(json.dumps(cases, indent=2))
            print('Public command receipts saved; follow-up runs in a separate process.')
        else:
            report = {'model': model, 'live': bool(original), 'cases': results,
                      'fixtureHttpRequests': len(requests), 'nativeInput': False}
            if original:
                with sqlite3.connect(original.as_uri() + '?mode=ro', uri=True) as db:
                    report['originalUnchanged'] = digest(db) == before
                assert report['originalUnchanged'], 'Original profile changed.'
            (directory / 'report.json').write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding='utf-8')
            print(json.dumps(report, ensure_ascii=True))
            assert all(c['pass'] for c in results), 'Historical evidence follow-up failed; inspect report.'
    finally:
        host.close()
        if server:
            server.shutdown()
            server.server_close()


if __name__ == '__main__':
    main()
