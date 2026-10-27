"""Credential-free normal-bridge token efficiency qualification.

Uses an isolated profile/project, an owned loopback provider and public fixtures.
The provider's usage counters are synthetic. Request sizes are measured, not
claims about live model quality, exact tokenization or provider billing.
"""
import argparse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import threading

from desktop_test_support import NativeHost, ROOT


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', required=True, type=Path)
    args = parser.parse_args()
    directory = args.directory.resolve()
    if not args.directory.is_absolute() or not directory.is_relative_to(ROOT / 'output'):
        raise SystemExit('A fresh absolute disposable output/ directory is required')
    directory.mkdir(parents=True, exist_ok=False)
    project = directory / 'project'
    project.mkdir()
    # Distinct large results prevent a duplicate-only test from hiding growth.
    for index in range(35):
        (project / f'source-{index}.txt').write_text(
            f'PUBLIC-FIXTURE-{index}\n' + ('fixture text 世界 ' * 700), encoding='utf-8')
    requests = []
    failures = []

    class Provider(BaseHTTPRequestHandler):
        def log_message(self, *_):
            pass

        def do_POST(self):
            payload = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
            index = len(requests)
            requests.append(payload)
            names = [t['function']['name'] for t in payload.get('tools', [])]
            try:
                assert len(payload['messages'][0]['content'].encode()) <= 3996
                if index == 0:
                    assert 'discover_tools' in names and 'inspect_harness' not in names
                if index == 1:
                    assert 'inspect_harness' in names and 'schedule_task' in names
                if index >= 3:
                    assert 'inspect_harness' not in names and 'schedule_task' not in names
                    assert payload['messages'][0] == requests[0]['messages'][0], 'Unloading must restore the stable system prefix'
                if index == 10:
                    tools = [m for m in payload['messages'] if m['role'] == 'tool']
                    assert any('historicalPreview' in m['content'] for m in tools)
                    assert len(tools[-1]['content'].encode()) > 10000
            except AssertionError as error:
                failures.append(f'Request {index}: {error}')
            if index == 0:
                name, arguments = 'discover_tools', {'names': ['inspect_harness', 'schedule_task']}
            elif index == 1:
                name, arguments = 'inspect_harness', {}
            elif index == 2:
                name, arguments = 'discover_tools', {'unload': ['inspect_harness', 'schedule_task']}
            elif index < 38:
                name, arguments = 'read_text_file', {'path': f'source-{index - 3}.txt'}
            else:
                name, arguments = None, None
            delta = {'tool_calls': [{'index': 0, 'id': f'fixture-{index}', 'type': 'function',
                                     'function': {'name': name, 'arguments': json.dumps(arguments)}}]} if name else {'content': 'Public fixture complete.'}
            chunks = [
                {'choices': [{'index': 0, 'delta': delta, 'finish_reason': None}]},
                {'choices': [{'index': 0, 'delta': {}, 'finish_reason': 'tool_calls' if name else 'stop'}]},
                {'choices': [], 'usage': {'prompt_tokens': 100 + index, 'completion_tokens': 10,
                                          'total_tokens': 110 + index,
                                          'prompt_tokens_details': {'cached_tokens': 20}}},
            ]
            self.send_response(200)
            self.send_header('Content-Type', 'text/event-stream')
            self.end_headers()
            self.wfile.write((''.join('data: ' + json.dumps(chunk) + '\n\n' for chunk in chunks)
                             + 'data: [DONE]\n\n').encode())

    server = ThreadingHTTPServer(('127.0.0.1', 0), Provider)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    client = NativeHost(directory)
    try:
        call = client.call
        call('bootstrap')
        policy = call('memories')['automaticPolicy']
        call('setAutomaticMemory', enabled=False, revision=policy['revision'])
        call('configure', preferences={'baseUrl': f'http://127.0.0.1:{server.server_port}/v1', 'model': 'fixture'},
             enabledModels=['fixture'], apiKey='', rememberConnection=False)
        session = call('createSession', kind='project', path=str(project))['session']['id']
        call('setTaskPermissions', session=session, revision=0,
             policy={'mode': 'fullAccess', 'grants': [], 'expiresAt': None})
        task = 'Inspect the synthetic project fixtures. Keep exact task constraints 世界. Do not change files.'
        preview = call('context', session=session, input=task, tools=True)
        call('start', id=1, session=session, input=task)
        done, _ = client.finish(1, seconds=90)
        assert not done.get('error'), done.get('error')
        assert not failures, failures
        assert len(requests) == 39, len(requests)
        path = directory / 'conversation.json'
        call('export', session=session, path=str(path), format='json')
        exported = json.loads(path.read_text(encoding='utf-8'))
        metadata = exported['messages'][-1]['metadata']
        assert not metadata.get('paused'), metadata.get('paused')
        assert metadata['agent'].get('archive'), 'Fixture must exercise evidence segmentation'
        assert exported['usageSummary']['modelAttempts'] == 39
        assert exported['usageSummary']['missingUsageCalls'] == 0
        assert exported['usageSummary']['knownUsage']['totalTokens'] == sum(110 + n for n in range(39))
        assert exported['usageSummary']['knownUsage']['cachedInputTokens'] == 39 * 20
        assert any(event['kind'] == 'agentEvidence' for run in exported['runs'] for event in run['events'])
        receipts = [event['data'] for run in exported['runs'] for event in run['events'] if event['kind'] == 'toolResult']
        inspection = next(r for r in receipts if r['name'] == 'inspect_harness')
        assert inspection['returned'], inspection
        assert sum(r['name'] == 'read_text_file' for r in receipts) == 35
        # All full read receipts are recoverable without relying on prompt previews.
        assert 'PUBLIC-FIXTURE-0' in path.read_text(encoding='utf-8')
        first = requests[0]
        schema_tokens = lambda body: (len(json.dumps(body['tools'], ensure_ascii=False, separators=(',', ':')).encode()) + 3) // 4
        projected = [sum(len(m['content'].encode()) for m in body['messages'] if m['role'] == 'tool') for body in requests]
        original_content = {r['callId']: r['content'] for r in receipts}
        unprojected = [sum(len(original_content.get(m.get('tool_call_id'), m['content']).encode())
                           for m in body['messages'] if m['role'] == 'tool') for body in requests]
        system = first['messages'][0]['content']
        result = {
            'passed': True, 'requests': len(requests), 'liveModel': False,
            'initialSystemTokensEstimated': (len(system.encode()) + 3) // 4,
            'initialToolSchemaTokensEstimated': schema_tokens(first),
            'initialToolCount': len(first['tools']),
            'loadedToolCount': len(requests[1]['tools']),
            'unloadedToolCount': len(requests[3]['tools']),
            'loadedToolSchemaTokensEstimated': schema_tokens(requests[1]),
            'unloadedToolSchemaTokensEstimated': schema_tokens(requests[3]),
            'previewTokens': preview.get('tokens'),
            'savedInitialTokens': metadata['context'].get('tokens'),
            'maxToolEvidenceRequestBytes': max(projected),
            'cumulativeProjectedToolBytes': sum(projected),
            'sameTraceUnprojectedToolBytes': sum(unprojected),
            'sameTraceToolByteReductionPercent': round(100 * (1 - sum(projected) / sum(unprojected)), 2),
            'fullFixtureReadBytes': sum(len((project / f'source-{n}.txt').read_bytes()) for n in range(35)),
            'usageSummary': exported['usageSummary'],
        }
        assert result['initialSystemTokensEstimated'] < 1000, result
        assert result['initialToolSchemaTokensEstimated'] < 4000, result
        assert result['loadedToolSchemaTokensEstimated'] > result['unloadedToolSchemaTokensEstimated'], result
        assert max(projected) < result['fullFixtureReadBytes'] // 2, result
        assert preview['tokens'] == metadata['context']['tokens'], json.dumps({
            'problem': 'Preview and saved initial context differ', 'preview': preview['tokens'],
            'saved': metadata['context']['tokens'], 'previewTools': [s['name'] for s in preview['tools']],
            'sentTools': [s['function']['name'] for s in first['tools']]})
        (directory / 'result.json').write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding='utf-8')
        print(json.dumps(result, ensure_ascii=True))
    except Exception as error:
        # Preserve actionable, bounded fixture diagnostics without request bodies.
        result = {'passed': False, 'requests': len(requests), 'liveModel': False,
                  'error': str(error)[:2000], 'providerChecks': failures,
                  'requestSizes': [{'messagesBytes': len(json.dumps(r['messages'], ensure_ascii=False).encode()),
                                    'toolCount': len(r.get('tools', []))} for r in requests]}
        (directory / 'result.json').write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding='utf-8')
        raise
    finally:
        client.close()
        server.shutdown()
        server.server_close()


if __name__ == '__main__':
    main()
