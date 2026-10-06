"""Inject one stream failure, then ask a configured model to inspect its evidence.

Fresh disposable output profile only. Credentials stay in environment variables;
results contain coverage booleans, never model reasoning or private transcripts.
"""
import argparse
import json
import os
from pathlib import Path
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import threading

from desktop_test_support import NativeHost, ROOT


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', type=Path, required=True)
    parser.add_argument('--model', required=True)
    args = parser.parse_args()
    directory = args.directory.resolve()
    if not args.directory.is_absolute() or not directory.is_relative_to(ROOT / 'output') or directory.exists():
        parser.error('Use a fresh absolute ignored output directory.')
    (directory / 'workspace').mkdir(parents=True)
    class Fixture(BaseHTTPRequestHandler):
        def log_message(self, *_): pass
        def do_POST(self):
            self.rfile.read(int(self.headers['Content-Length']))
            body = ('data: ' + json.dumps({'choices': [], 'padding': 'x' * (256 * 1024 + 1)}) + '\n\ndata: [DONE]\n\n').encode()
            self.send_response(200)
            self.send_header('Content-Type', 'text/event-stream')
            self.send_header('Content-Length', str(len(body)))
            self.end_headers(); self.wfile.write(body)
    server = ThreadingHTTPServer(('127.0.0.1', 0), Fixture)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    host = NativeHost(directory)
    result = {'model': args.model, 'passed': False, 'live': True}
    try:
        host.call('configure', preferences={'baseUrl': f'http://127.0.0.1:{server.server_port}/v1', 'model': 'fixture'}, apiKey='', rememberConnection=False, enabledModels=['fixture'])
        session = host.call('createSession', kind='project', path=str(directory / 'workspace'))['session']['id']
        host.call('start', id=1, session=session, input='Synthetic stream failure. Do not execute tools.')
        failure, _ = host.finish(1)
        result['fixtureFailure'] = str(failure.get('error', '')).startswith('Provider sent an oversized stream event')
        inventory = host.call('harnessInventory', session=session)
        result['failureVisible'] = len(inventory['recentFailures']) == 1
        host.call('configure', preferences={'baseUrl': os.environ['DOLORES_TEST_BASE_URL'], 'model': args.model}, apiKey=os.environ['DOLORES_TEST_API_KEY'], rememberConnection=False, enabledModels=[args.model])
        settings = {'maxOutputTokens': 4096, 'timeoutSeconds': 120}
        if args.model.startswith('deepseek'): settings['reasoning'] = 'deepseekThinkingOff'
        host.call('setRequestSettings', settings=settings)
        policy = host.call('memories')['automaticPolicy']
        host.call('setAutomaticMemory', enabled=False, revision=policy['revision'])
        host.call('saveScopedSettings', session=session, scope='thread', revision=0, patch={'task': {'modelCalls': 4, 'toolCalls': 4, 'segments': 1, 'elapsedSeconds': 150}})
        reads = []
        def approval(event):
            if event['type'] != 'toolApproval': return
            request = event['request']
            allowed = request['name'] == 'inspect_harness'
            if allowed:
                query = json.loads(request.get('query') or '{}')
                reads.append(query.get('source') or 'inventory')
            host.call('approveTool', id=2, callId=request['callId'], allow=allowed)
        host.call('start', id=2, session=session, input='Diagnose the last harness failure. Use inspect_harness inventory first and then provider_stream source (the streaming loop is near line 260). Explain the actual bound, whether an incomplete tool call executed, and a useful recovery. Do not replay the failed task, execute commands or modify files.')
        done, _ = host.finish(2, callback=approval, seconds=165)
        messages = host.call('messagesPage', session=session)['items']
        answer = messages[-1]['content'].lower() if messages else ''
        result.update({'completed': not bool(done.get('error')), 'readInventory': 'inventory' in reads,
            'readStreamSource': 'provider_stream' in reads, 'identifiedBound': '256' in answer or '262144' in answer,
            'mentionsIncompleteCall': 'incomplete' in answer or 'partial' in answer,
            'approvedTools': reads})
        result['passed'] = all(result.get(key, False) for key in ['fixtureFailure', 'failureVisible', 'completed', 'readInventory', 'readStreamSource', 'identifiedBound', 'mentionsIncompleteCall'])
    finally:
        host.close(); server.shutdown(); server.server_close()
    (directory / 'public-result.json').write_text(json.dumps(result, indent=2), encoding='utf-8')
    print(json.dumps(result))
    if not result['passed']: raise SystemExit(1)


if __name__ == '__main__': main()
