"""Normal bridge question round trips with a loopback fixture or explicit live model.

Live checks copy connection configuration only; credentials stay in memory and
private history stays in the source profile. No desktop input is dispatched.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
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
    parser.add_argument('--initialize', action='store_true')
    parser.add_argument('--source', type=Path)
    parser.add_argument('--model', help='Explicit enabled model for optional live checks.')
    parser.add_argument('--reasoning', default='providerDefault', help='Explicit optional live reasoning override.')
    args = parser.parse_args()
    directory = args.directory.resolve()
    if not directory.is_relative_to(ROOT / 'output'):
        parser.error('Choose a disposable directory under output.')
    if args.initialize:
        host = NativeHost(directory)
        host.call('bootstrap')
        host.close()
        return
    if bool(args.source) != bool(args.model):
        parser.error('Live checks need both --source and --model.')
    directory.mkdir(parents=True, exist_ok=False)
    subprocess.run([sys.executable, __file__, '--directory', str(directory), '--initialize'], check=True, capture_output=True)
    original = args.source.resolve() if args.source else None
    before = None
    if original:
        with sqlite3.connect(original.as_uri() + '?mode=ro', uri=True) as source:
            before = digest(source)
            with sqlite3.connect(directory / 'data/dolores.db') as target:
                for table in ('preferences', 'model_choices', 'model_contexts', 'request_settings', 'model_request_settings', 'model_images'):
                    rows = source.execute('SELECT * FROM ' + table).fetchall()
                    target.execute('DELETE FROM ' + table)
                    if rows:
                        target.executemany('INSERT INTO ' + table + ' VALUES(' + ','.join('?' for _ in rows[0]) + ')', rows)
    requests = []

    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *args):
            pass

        def do_POST(self):
            payload = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
            requests.append(payload)
            tool_message = next((m for m in reversed(payload['messages']) if m['role'] == 'tool'), None)
            if tool_message:
                assert json.loads(tool_message['content'])['answers'][0]['answer'] == 'PUBLIC-ANSWER-42'
                delta = {'content': 'Selected PUBLIC-ANSWER-42.'}
                finish = 'stop'
            else:
                assert any(t['function']['name'] == 'request_user_input' for t in payload['tools'])
                arguments = {'questions': [{'id': 'date', 'question': 'Which date should I use?', 'options': [{'label': 'Today'}, {'label': 'Previous entry'}]}]}
                delta = {'tool_calls': [{'index': 0, 'id': 'ask-date', 'type': 'function', 'function': {'name': 'request_user_input', 'arguments': json.dumps(arguments)}}]}
                finish = 'tool_calls'
            self.send_response(200)
            self.send_header('Content-Type', 'text/event-stream')
            self.end_headers()
            for item in [{'choices': [{'delta': delta, 'finish_reason': None}]}, {'choices': [{'delta': {}, 'finish_reason': finish}]}]:
                self.wfile.write(('data: ' + json.dumps(item) + '\n\n').encode())
            self.wfile.write(b'data: [DONE]\n\n')

    server = None
    if original:
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
        base, key, models = f'http://127.0.0.1:{server.server_port}/v1', 'synthetic', ['fixture']
    host = NativeHost(directory)
    cases = []
    try:
        host.call('bootstrap')
        model = args.model or 'fixture'
        host.call('configure', preferences={'baseUrl': base, 'model': model}, apiKey=key, rememberConnection=False, enabledModels=models)
        del key
        host.call('setModelRequestSettings', preferences={'baseUrl': base, 'model': model}, settings={'maxOutputTokens': 1024, 'timeoutSeconds': 20, 'reasoning': args.reasoning})
        policy = host.call('memories')['automaticPolicy']
        host.call('setAutomaticMemory', enabled=False, revision=policy['revision'])
        for identity, prompt in enumerate([
            'Help me prepare a public daily report. First ask me which date to use with clickable choices. Do not write the report until I answer. After I answer, acknowledge the exact custom answer in one short sentence. No other tools are needed.',
            '帮我准备公开日报。先让我通过可点击的选项选择日期，收到答案后再继续。不要猜日期。收到我自定义的回答后，用一句话确认完整的回答内容，不需要其他工具。',
        ], 1):
            expected = ('2026-10-11' if identity == 1 else '2026年10月11日') if original else 'PUBLIC-ANSWER-42'
            session = host.call('createSession', kind='side')['session']['id']
            host.call('start', id=identity, session=session, input=prompt)
            asked = []

            def respond(event):
                if event['type'] == 'toolApproval':
                    host.call('cancel', id=identity)
                    raise AssertionError('Question must not grant other tool access.')
                if event['type'] != 'userQuestion':
                    return
                request = event['request']
                asked.append(request)
                assert not host.envelope('answerQuestion', id=identity, callId='stale', answers=['bad'])['ok']
                assert not host.envelope('approveTool', id=identity, callId=request['callId'], allow=True)['ok']
                answers = [expected] * len(request['questions'])
                host.call('answerQuestion', id=identity, callId=request['callId'], answers=answers)
                assert not host.envelope('answerQuestion', id=identity, callId=request['callId'], answers=answers)['ok']

            done, _ = host.finish(identity, callback=respond, seconds=55)
            messages = host.call('messagesPage', session=session)['items']
            last = messages[-1] if messages else {}
            metadata = last.get('metadata') or {}
            records = (metadata.get('agent') or {}).get('tools', [])
            receipt_matches = any(r['name'] == 'request_user_input' and r['status'] == 'completed' and json.loads(r['content'])['answers'][0]['answer'] == expected for r in records)
            # Acknowledgements may add spaces to a date. The tool receipt must
            # still preserve the submitted answer byte-for-byte.
            acknowledged = expected in ''.join(last.get('content', '').split())
            passed = bool(asked) and not done.get('error') and acknowledged and receipt_matches
            cases.append({'language': 'English' if identity == 1 else 'Chinese', 'pass': passed, 'questions': len(asked), 'error': done.get('error'), 'receiptMatches': receipt_matches, 'acknowledged': acknowledged, 'answer': last.get('content', '')[:1000], 'usage': (metadata.get('agent') or {}).get('usageByCall', [])})
        report = {'model': model, 'live': bool(original), 'cases': cases, 'fixtureHttpRequests': len(requests)}
        if original:
            with sqlite3.connect(original.as_uri() + '?mode=ro', uri=True) as db:
                report['originalUnchanged'] = digest(db) == before
            assert report['originalUnchanged'], 'Source profile changed.'
        (directory / 'report.json').write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding='utf-8')
        print(json.dumps(report, ensure_ascii=True))
        assert all(c['pass'] for c in cases), 'Question qualification failed; inspect report.'
    finally:
        host.close()
        if server:
            server.shutdown()
            server.server_close()


if __name__ == '__main__':
    main()
