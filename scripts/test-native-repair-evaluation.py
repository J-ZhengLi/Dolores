"""Normal native host: separate exact execution review and withheld non-improvement."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from desktop_test_support import NativeHost, ROOT

REPRODUCTION = '#[test] fn ordinary_budget_remains_bounded() { assert_eq!(dolores_core::TaskBudget::default().model_limit(), 64); }'


def main(directory):
    host = NativeHost(directory)
    call = host.call
    project = directory / 'unrelated-project'
    project.mkdir()
    (project / 'keep.txt').write_text('preserved')
    session = call('createSession', kind='project', path=str(project))['session']['id']
    other = call('createSession', kind='project', path=str(project))['session']['id']
    inventory = call('harnessInventory', session=session)
    source = next(s for s in inventory['sources'] if s['id'] == 'failure_watchdog')
    mode = 'prepare'
    requests = []
    approvals = []

    class Provider(BaseHTTPRequestHandler):
        def log_message(self, *args):
            pass

        def do_POST(self):
            request = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
            requests.append(1)
            if any(m['role'] == 'tool' for m in request['messages']):
                delta = {'content': 'Fixture evidence retained; no installation or task replay.'}
                finish = 'stop'
            else:
                name = 'harness_repair'
                args = {'action': 'prepare', 'source': 'failure_watchdog',
                        'sourceId': source['sourceId'], 'bundleId': inventory['sourceBundle']['bundleId']}
                if mode == 'patch':
                    state = call('harnessRepairs', session=session, repairId=repair_id)
                    args = {'action': 'propose', 'repairId': repair_id, 'revision': state['revision'],
                            'source': 'failure_watchdog', 'sourceId': state['files'][0]['candidateId'],
                            'oldText': 'pub(crate) struct FailureWatchdog',
                            'newText': '// Reviewed native evaluation fixture\npub(crate) struct FailureWatchdog'}
                elif mode in ('decline', 'stale', 'evaluate'):
                    name = 'test_harness_repair'
                    args = {'repairId': repair_id, 'revision': 1 if mode == 'stale' else 2,
                            'package': 'dolores-core', 'reproduction': REPRODUCTION}
                delta = {'tool_calls': [{'index': 0, 'id': f'{mode}-call', 'type': 'function',
                                        'function': {'name': name, 'arguments': json.dumps(args)}}]}
                finish = 'tool_calls'
            self.send_response(200)
            self.send_header('Content-Type', 'text/event-stream')
            self.end_headers()
            self.wfile.write(('data: ' + json.dumps({'choices': [{'delta': delta, 'finish_reason': finish}]}) + '\n\ndata: [DONE]\n\n').encode())

    server = ThreadingHTTPServer(('127.0.0.1', 0), Provider)
    threading.Thread(target=server.serve_forever, daemon=True).start()

    def review(event):
        if event['type'] != 'toolApproval':
            return
        request = event['request']
        approvals.append(request)
        if request['name'] == 'test_harness_repair':
            query = json.loads(request['query'])
            assert query['revision'] == 2 and query['repairId'] == repair_id
            assert set(query['commands']) == {'baselineReproduction', 'candidateReproduction', 'candidateRegressions'}
            assert '+'+REPRODUCTION in request['diff']
            assert '+// Reviewed native evaluation fixture' in request['diff']
            for command in query['commands'].values():
                assert command['invocation']['program'] == 'cargo'
                assert '--offline' in command['invocation']['args'] and '--locked' in command['invocation']['args']
                assert command['timeoutSeconds'] == 300 and command['captureBytes'] == 262144
            # Only the exact trusted test above is authorized by this fixture.
            call('approveTool', id=identity, callId=request['callId'], allow=mode == 'evaluate')
        else:
            assert request['name'] == 'harness_repair'
            call('approveTool', id=identity, callId=request['callId'], allow=True)

    try:
        policy = call('memories')['automaticPolicy']
        call('setAutomaticMemory', enabled=False, revision=policy['revision'])
        call('configure', preferences={'baseUrl': f'http://127.0.0.1:{server.server_port}/v1', 'model': 'fixture'}, apiKey='', rememberConnection=False)
        identity = 1
        call('start', id=identity, session=session, input='prepare a reviewed native fixture')
        done, _ = host.finish(identity, review)
        assert not done.get('error')
        repair_id = call('harnessRepairs', session=session)['repairIds'][0]
        mode = 'patch'
        identity = 2
        call('start', id=identity, session=session, input='retain the exact comment proposal')
        done, _ = host.finish(identity, review)
        assert not done.get('error')
        saved = call('harnessRepairs', session=session, repairId=repair_id, source='failure_watchdog')
        mode = 'decline'
        identity = 3
        call('start', id=identity, session=session, input='decline native execution')
        host.finish(identity, review)
        assert call('harnessRepairs', session=session, repairId=repair_id)['evaluations'] == []
        mode = 'stale'
        identity = 4
        count = len(approvals)
        call('start', id=identity, session=session, input='stale native execution fixture')
        _, events = host.finish(identity, review)
        assert len(approvals) == count
        blocked = next(e['record'] for e in events if e['type'] == 'toolResult')
        assert blocked['status'] == 'blocked' and 'current revision' in blocked['content']
        mode = 'evaluate'
        identity = 5
        call('start', id=identity, session=session, input='run the exact separately reviewed bounded test')
        done, _ = host.finish(identity, review, seconds=340)
        assert not done.get('error')
        final = call('harnessRepairs', session=session, repairId=repair_id, source='failure_watchdog')
        assert final['source'] == saved['source'] and final['revision'] == saved['revision']
        result = final['evaluations'][0]
        assert result['status'] == 'withheld' and result['baselinePassed'] == 1 and result['baselineFailed'] == 0, result
        assert result['candidatePassed'] is None and result['current'] is True
        assert not host.envelope('harnessRepairs', session=other, repairId=repair_id)['ok']
        assert (project / 'keep.txt').read_text() == 'preserved'
        print(json.dumps({'ok': True, 'fixtureRequests': len(requests), 'liveRequests': 0,
                          'reviewedProposalSteps': 2, 'nativeReviews': 2,
                          'declineAndStaleExecuteNothing': True, 'baselineTestsExecuted': 1,
                          'candidateTestsExecuted': 0, 'nonImprovementWithheld': True,
                          'originalProjectPreserved': True, 'installationExecuted': False}))
    finally:
        host.close()
        server.shutdown()


if __name__ == '__main__':
    if len(sys.argv) > 1:
        main(Path(sys.argv[1]))
    else:
        with tempfile.TemporaryDirectory(prefix='native-evaluation-', dir=ROOT / 'output') as directory:
            subprocess.run([sys.executable, __file__, directory], check=True)
