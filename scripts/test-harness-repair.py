"""Normal native host: reviewed repair snapshots, visible diff, stale/Stop recovery."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from desktop_test_support import NativeHost, ROOT


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
    bundle = inventory['sourceBundle']['bundleId']
    requests = []
    mode = 'prepare'

    class Provider(BaseHTTPRequestHandler):
        def log_message(self, *args):
            pass
        def do_POST(self):
            request = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
            requests.append(1)
            tool_results = [m for m in request['messages'] if m['role'] == 'tool']
            if tool_results:
                delta = {'content': 'Native proposal retained; no tests or installation executed.'}
                finish = 'stop'
            else:
                args = {'action': 'prepare', 'source': 'failure_watchdog', 'sourceId': source['sourceId'], 'bundleId': bundle}
                if mode != 'prepare' and mode != 'stop':
                    state = call('harnessRepairs', session=session, repairId=repair_id)
                    args = {'action': 'propose', 'repairId': repair_id, 'revision': 1 if mode == 'stale' else state['revision'],
                            'source': 'failure_watchdog', 'sourceId': state['files'][0]['candidateId'],
                            'oldText': 'pub(crate) struct FailureWatchdog',
                            'newText': '// Native proposal fixture\npub(crate) struct FailureWatchdog'}
                delta = {'tool_calls': [{'index': 0, 'id': f'{mode}-call', 'type': 'function',
                         'function': {'name': 'harness_repair', 'arguments': json.dumps(args)}}]}
                finish = 'tool_calls'
            self.send_response(200)
            self.send_header('Content-Type', 'text/event-stream')
            self.end_headers()
            self.wfile.write(('data: ' + json.dumps({'choices': [{'delta': delta, 'finish_reason': finish}]}) + '\n\ndata: [DONE]\n\n').encode())

    server = ThreadingHTTPServer(('127.0.0.1', 0), Provider)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    approvals = []
    def review(event):
        if event['type'] == 'toolApproval':
            request = event['request']
            assert request['name'] == 'harness_repair'
            if mode == 'patch':
                assert '+// Native proposal fixture' in request['diff']
            approvals.append(request)
            call('approveTool', id=identity, callId=request['callId'], allow=True)
    try:
        policy = call('memories')['automaticPolicy']
        call('setAutomaticMemory', enabled=False, revision=policy['revision'])
        call('configure', preferences={'baseUrl': f'http://127.0.0.1:{server.server_port}/v1', 'model': 'fixture'}, apiKey='', rememberConnection=False)
        identity = 1
        call('start', id=identity, session=session, input='prepare a native repair fixture')
        done, events = host.finish(identity, review)
        assert not done.get('error')
        repair_id = call('harnessRepairs', session=session)['repairIds'][0]
        mode = 'patch'
        identity = 2
        call('start', id=identity, session=session, input='save the exact native proposal')
        done, events = host.finish(identity, review)
        assert not done.get('error') and len(approvals) == 2
        saved = call('harnessRepairs', session=session, repairId=repair_id, source='failure_watchdog')
        assert saved['revision'] == 2 and saved['status'] == 'proposed'
        assert saved['artifactIntegrity'] == 'matches'
        assert '+// Native proposal fixture' in saved['diff']
        assert not host.envelope('harnessRepairs', session=other, repairId=repair_id)['ok']
        mode = 'stale'
        identity = 3
        call('start', id=identity, session=session, input='stale proposal fixture')
        done, events = host.finish(identity, review)
        blocked = next(e['record'] for e in events if e['type'] == 'toolResult')
        assert blocked['status'] == 'blocked' and 'fresh proposal' in blocked['content']
        assert len(approvals) == 2
        mode = 'stop'
        identity = 4
        call('start', id=identity, session=session, input='stop before saving another repair')
        def stop(event):
            if event['type'] == 'toolApproval':
                call('cancel', id=identity)
        host.finish(identity, stop)
        assert call('harnessRepairs', session=session)['repairIds'] == [repair_id]
        assert call('harnessRepairs', session=session, repairId=repair_id, source='failure_watchdog') == saved
        assert (project / 'keep.txt').read_text() == 'preserved'
        print(json.dumps({'ok': True, 'fixtureRequests': len(requests), 'liveRequests': 0,
                          'reviewedSteps': len(approvals), 'visibleDiffRetained': True,
                          'staleAndCrossChatRefused': True, 'stopPreservedWork': True,
                          'originalProjectUnchanged': True, 'executedCandidateTests': 0}))
    finally:
        host.close()
        server.shutdown()


if __name__ == '__main__':
    if len(sys.argv) > 1:
        main(Path(sys.argv[1]))
    else:
        with tempfile.TemporaryDirectory(prefix='harness-repair-', dir=ROOT / 'output') as directory:
            subprocess.run([sys.executable, __file__, directory], check=True)
