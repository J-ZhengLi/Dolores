"""Normal Windows C ABI lifecycle qualification with public loopback fixtures."""
import argparse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import subprocess
import sys
import threading
import zipfile
from desktop_test_support import NativeHost, ROOT

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--directory', type=Path, required=True)
parser.add_argument('--restore', action='store_true')
parser.add_argument('--save', action='store_true', help=argparse.SUPPRESS)
args = parser.parse_args()
directory = args.directory.resolve()
assert directory.is_relative_to(ROOT / 'output') and directory != ROOT / 'output'
if not args.save and not args.restore:
    for stage_arg in ('--save', '--restore'):
        subprocess.run([sys.executable, __file__, '--directory', str(directory), stage_arg], check=True)
    receipt = json.loads((directory / 'receipt.json').read_text(encoding='utf-8'))
    receipt['restartRetainedEvidence'] = True
    (directory / 'receipt.json').write_text(json.dumps(receipt, indent=2), encoding='utf-8')
    print(json.dumps(receipt))
    sys.exit(0)
if args.restore:
    saved = json.loads((directory / 'state.json').read_text(encoding='utf-8'))
    host = NativeHost(directory)
    try:
        assert not host.call('bootstrap')['configured']
        events = host.call('runEvents', session=saved['session'], runId=saved['run'])
        assert any(e['kind'] == 'subagentContext' for e in events)
        assert all(r['state'] not in ('prepared', 'running', 'waitingForApproval') for r in host.call('runs', session=saved['session']))
        assert (directory / 'project/input.txt').read_text() == 'PUBLIC_INPUT'
        assert not host.call('scheduledTasks')['items']
        print(json.dumps({'restartRetainedEvidence': True, 'automaticReplays': 0}))
    finally:
        host.close()
    sys.exit(0)

directory.mkdir(parents=True, exist_ok=False)
project = directory / 'project'
project.mkdir()
(project / 'input.txt').write_text('PUBLIC_INPUT', encoding='utf-8')
(project / 'hook.cjs').write_text("const fs=require('fs');const e=JSON.parse(process.argv.at(-1));fs.appendFileSync('hook-events.jsonl',JSON.stringify(e)+'\\n');if(e.event==='afterTool')console.log('PUBLIC_MALFORMED_OBSERVER');else console.log(JSON.stringify({allow:!fs.existsSync('block-hook'),message:'PUBLIC_GATE'}));", encoding='utf-8')
requests, remote_calls, deletes = [], [], []
mode, stage, serial = 'combined', 0, 0
package_id, alias = '', ''
arrived = threading.Event()
release = threading.Event()

class Remote(BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def send(self, body, session=False):
        data = json.dumps(body).encode()
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Content-Length', str(len(data)))
        if session:
            self.send_header('Mcp-Session-Id', 'public-lifecycle-session')
        self.end_headers()
        self.wfile.write(data)

    def do_POST(self):
        body = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        method = body['method']
        if method == 'server/discover':
            self.send({'jsonrpc': '2.0', 'id': body['id'], 'error': {'code': -32601, 'message': 'legacy fixture'}})
            return
        if method.startswith('notifications/'):
            self.send_response(202)
            self.send_header('Content-Length', '0')
            self.end_headers()
            return
        if method == 'initialize':
            result = {'protocolVersion': '2025-11-25', 'capabilities': {'tools': {}}, 'serverInfo': {'name': 'Public lifecycle', 'version': '1'}}
        elif method == 'tools/list':
            result = {'tools': [{'name': 'echo', 'description': 'Public lifecycle echo', 'inputSchema': {'type': 'object'}}]}
        elif method == 'tools/call':
            remote_calls.append(body)
            if mode == 'remote-stop':
                arrived.set()
                release.wait(8)
            result = {'content': [{'type': 'text', 'text': 'PUBLIC_REMOTE_RESULT'}]}
        else:
            raise AssertionError(method)
        try:
            self.send({'jsonrpc': '2.0', 'id': body['id'], 'result': result}, method == 'initialize')
        except (BrokenPipeError, ConnectionResetError, ConnectionAbortedError):
            pass

    def do_DELETE(self):
        deletes.append(self.headers.get('Mcp-Session-Id'))
        release.set()
        self.send_response(204)
        self.send_header('Content-Length', '0')
        self.end_headers()

def resource():
    return {'skill': 'flow', 'scope': 'project', 'packageId': package_id, 'action': 'read', 'path': 'references/check.md'}

class Provider(BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def do_POST(self):
        global stage, serial
        body = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        requests.append(body)
        messages = body['messages']
        child = 'You are a scoped child' in messages[0]['content']
        tools = [t['function']['name'] for t in body.get('tools', [])]
        assert 'PUBLIC_GATE' not in json.dumps(body) and 'PUBLIC_MALFORMED_OBSERVER' not in json.dumps(body)
        calls = []
        if child:
            assert all(t in ('read_text_file', 'list_folder', 'search_text', 'edit_text_file', 'create_text_file', 'read_skill_resource') for t in tools)
            if not any(m['role'] == 'tool' for m in messages):
                calls = [('read_skill_resource', resource())]
            else:
                assert 'PUBLIC_RESOURCE_V1' in messages[-1]['content']
        else:
            n = stage
            stage += 1
            if mode == 'combined':
                if n == 0:
                    calls = [('read_skill_resource', resource())]
                elif n == 1:
                    calls = [(alias, {})]
                elif n == 2:
                    calls = [('delegate_tasks', {'tasks': [{'goal': 'Read the frozen flow package reference and report the exact public evidence.', 'scope': '.', 'readOnly': True}]})]
                elif n == 3:
                    calls = [('read_text_file', {'path': 'input.txt'})]
                else:
                    assert 'PUBLIC_INPUT' in messages[-1]['content']
            elif mode == 'remote-stop':
                calls = [(alias, {})]
            elif mode in ('queue', 'scheduled'):
                if n == 0:
                    calls = [('read_skill_resource', resource())]
                else:
                    assert ('PUBLIC_RESOURCE_V1' if mode == 'scheduled' else 'PUBLIC_RESOURCE_V2') in messages[-1]['content']
            elif mode == 'steer':
                if n == 0:
                    assert 'PUBLIC_SKILL_FLOW' not in messages[0]['content']
                    calls = [('read_text_file', {'path': 'input.txt'})]
                elif n == 1:
                    assert 'PUBLIC_SKILL_FLOW' in messages[0]['content']
                    calls = [('read_skill_resource', resource())]
                else:
                    assert 'PUBLIC_RESOURCE_V2' in messages[-1]['content']
            elif mode == 'schedule-create':
                if n == 0:
                    calls = [('schedule_task', {'rule': {'kind': 'daily', 'time': '09:00', 'date': None, 'weekdays': [], 'zone': 'UTC'}, 'skill': 'flow', 'model': None})]
        serial += 1
        delta = {'tool_calls': [{'index': i, 'id': f'public-{serial}-{i}', 'type': 'function', 'function': {'name': name, 'arguments': json.dumps(arguments)}} for i, (name, arguments) in enumerate(calls)]} if calls else {'content': 'PUBLIC_FINISHED: retained evidence; no unrun tests claimed.'}
        self.send_response(200)
        self.send_header('Content-Type', 'text/event-stream')
        self.end_headers()
        for delta, reason in ((delta, None), ({}, 'tool_calls' if calls else 'stop')):
            self.wfile.write(('data: ' + json.dumps({'choices': [{'delta': delta, 'finish_reason': reason}]}) + '\n\n').encode())
        self.wfile.write(b'data: [DONE]\n\n')

remote = ThreadingHTTPServer(('127.0.0.1', 0), Remote)
provider = ThreadingHTTPServer(('127.0.0.1', 0), Provider)
for server in (remote, provider):
    threading.Thread(target=server.serve_forever, daemon=True).start()
host = NativeHost(directory)
identity = 0
host_open = True
def package(version):
    path = directory / f'flow-{version}.zip'
    with zipfile.ZipFile(path, 'w', zipfile.ZIP_DEFLATED) as archive:
        archive.writestr('SKILL.md', '---\nname: flow\ndescription: Frozen workflow resources\n---\nPUBLIC_SKILL_FLOW Read references/check.md via the frozen package reader. Do not execute scripts or infer permission.\n')
        archive.writestr('references/check.md', f'PUBLIC_RESOURCE_V{version}')
    return path

def packages(action, **fields):
    return host.call('skillPackages', session=session, scope='project', request={'action': action, **fields})

def install(version):
    review = packages('inspect', name='flow', path=str(package(version)))
    packages('install', token=review['token'])
    activated = packages('activate', token=review['token'])
    return review['package']['package']['id'], activated

def finish(callback=None, idle=None):
    def decide(event):
        if callback:
            callback(event)
        if event['type'] == 'toolApproval':
            host.call('approveTool', id=identity, callId=event['request']['callId'], allow=True)
    done, events = host.finish(identity, callback=decide, idle=idle, seconds=25)
    return done, events

try:
    host.call('bootstrap')
    host.call('configure', preferences={'baseUrl': f'http://127.0.0.1:{provider.server_port}/v1', 'model': 'fixture'}, apiKey='', rememberConnection=False)
    host.call('setAutomaticMemory', enabled=False, revision=host.call('memories')['automaticPolicy']['revision'])
    session = host.call('createSession', kind='project', path=str(project))['session']['id']
    package_id, skill = install(1)
    old_id = package_id
    identity += 1
    host.call('inspectRemoteMcp', id=identity, session=session, remote={'label': 'Public lifecycle', 'endpoint': f'http://127.0.0.1:{remote.server_port}/mcp', 'authentication': 'none'})
    done, _ = host.finish(identity)
    inspection = done['mcpInspection']
    host.call('enableMcp', session=session, token=inspection['token'], names=['echo'])
    connection = host.call('mcpSettings', session=session)['connections'][0]
    alias = 'mcp_tool_' + connection['id'].replace('-', '') + '_1'
    hooks = [{'name': 'gate' if event == 'beforeTool' else 'observer', 'event': event, 'mode': 'blocking' if event == 'beforeTool' else 'observe', 'program': 'node', 'script': 'hook.cjs', 'args': [], 'timeoutSeconds': 1} for event in ('beforeTool', 'afterTool')]
    review = host.call('lifecycleHooks', session=session, request={'action': 'review', 'revision': 0, 'hooks': hooks})
    host.call('lifecycleHooks', session=session, request={'action': 'enable', 'token': review['token']})
    goal = host.call('editGoal', session=session, edit={'action': 'set', 'revision': None, 'objective': 'Use flow, echo through the reviewed connector, delegate a reference check and independently read input.txt.', 'criteria': ['Retained public file and child evidence'], 'constraints': 'Never execute package scripts.'})
    identity += 1
    host.call('startGoal', session=session, revision=goal['revision'], id=identity)
    done, _ = finish()
    assert not done.get('error'), done
    run = host.call('runs', session=session)[0]
    evidence = host.call('runEvents', session=session, runId=run['id'])
    assert any(e['kind'] == 'hookResult' and e['data'].get('childId') for e in evidence)
    assert any(e['kind'] == 'hookResult' and e['data']['status'] == 'error' for e in evidence)
    assert any(e['kind'] == 'mcpCleanup' and e['data']['confirmed'] for e in evidence)
    goal = host.call('goal', session=session)
    assert goal['used']['modelCalls'] == 7 and goal['used']['toolCalls'] == 5, goal['used']
    host.call('editGoal', session=session, edit={'action': 'clear', 'revision': goal['revision']})
    combined_run = run['id']

    mode, stage = 'schedule-create', 0
    identity += 1
    host.call('start', id=identity, session=session, input='Every day at 09:00 UTC use skill flow to read its public reference.')
    done, _ = finish()
    assert not done.get('error'), done
    scheduled = host.call('scheduledTasks')['items'][0]['task']
    assert host.call('scheduledTasks')['items'][0]['receipt']['lifecycleHooks'] == 2
    mode, stage = 'scheduled', 0
    claimed = host.call('scheduledManage', task=scheduled['id'], revision=scheduled['revision'], action='runNow')['claimed'][0]
    identity += 1
    start = host.call('scheduledStart', occurrence=claimed['id'], id=identity)
    done, events = finish()
    assert not done.get('error'), done
    destination = next(e['session'] for e in events if e.get('session'))
    host.call('scheduledTick')
    saved = host.call('runs', session=destination)[0]
    assert any(e['kind'] == 'hookResult' for e in host.call('runEvents', session=destination, runId=saved['id']))
    host.call('lifecycleHooks', session=session, request={'action': 'disable', 'revision': 1})
    current = host.call('scheduledTasks')['items'][0]['task']
    next_claim = host.call('scheduledManage', task=current['id'], revision=current['revision'], action='runNow')['claimed'][0]
    before_requests = len(requests)
    identity += 1
    refused = host.envelope('scheduledStart', occurrence=next_claim['id'], id=identity)
    assert not refused['ok'] and 'hooks changed' in refused['error'] and len(requests) == before_requests, refused
    package_id, skill = install(2)
    review = packages('inspectInstalled', id=old_id)
    skill = packages('forgetVersion', token=review['token'], version=1)['active']
    retained = packages('remove', token=review['token'])
    assert retained['filesRetained']
    # The retained payload location is the reviewed immutable snapshot, not a guessed path.
    old_root = Path(review['package']['package']['root'])
    assert old_root.exists()
    current = host.call('scheduledTasks')['items'][0]['task']
    host.call('scheduledManage', task=current['id'], revision=current['revision'], action='delete')
    review = packages('inspectRetained', entry=old_id)
    assert packages('remove', token=review['token'])['filesRemoved'] and not old_root.exists()

    selection = {'scope': 'project', 'name': 'flow', 'revision': skill['revision'], 'version': 2, 'packageId': package_id}
    mode, stage = 'steer', 0
    steered = False
    def steer(event):
        global steered
        if not steered and event['type'] == 'toolApproval':
            host.call('saveComposerSkills', session=session, selections=[selection])
            host.call('editTaskInputs', session=session, edit={'action': 'enqueue', 'id': 'steering', 'text': 'Use the exact selected workflow for this next check.', 'mode': 'steer'})
            steered = True
    identity += 1
    host.call('start', id=identity, session=session, input='PUBLIC_STEER: read input.txt, then follow the next explicit instruction.')
    done, _ = finish(steer)
    assert not done.get('error'), done
    assert host.call('taskInputs', session=session)['entries'][0]['state'] == 'applied'
    mode, stage = 'queue', 0
    host.call('editTaskInputs', session=session, edit={'action': 'enqueue', 'id': 'queued', 'text': 'Read the selected public package resource.', 'mode': 'queue'})
    identity += 1
    host.call('startTaskInput', session=session, id=identity)
    done, _ = finish()
    assert not done.get('error'), done
    assert all(e['state'] == 'applied' for e in host.call('taskInputs', session=session)['entries'])

    mode, stage = 'remote-stop', 0
    release.clear()
    before_calls, before_deletes = len(remote_calls), len(deletes)
    stopped = False
    def stop():
        global stopped
        if arrived.is_set() and not stopped:
            host.call('cancel', id=identity)
            stopped = True
    identity += 1
    host.call('start', id=identity, session=session, input='PUBLIC_STOP_REMOTE: call the reviewed echo once.')
    done, _ = finish(idle=stop)
    assert stopped and done.get('error')
    assert len(remote_calls) == before_calls + 1 and len(deletes) == before_deletes + 1
    stopped_run = host.call('runs', session=session)[0]
    assert any(e['kind'] == 'mcpCleanup' and e['data']['confirmed'] for e in host.call('runEvents', session=session, runId=stopped_run['id']))
    assert (project / 'input.txt').read_text() == 'PUBLIC_INPUT'
    (directory / 'state.json').write_text(json.dumps({'session': session, 'run': combined_run}), encoding='utf-8')
    host.close()
    host_open = False
    receipt = {'passed': True, 'goalCountsIncludeChildren': True, 'childHooksAndFrozenResources': True, 'scheduledPinnedPackageSurvivedActivationForget': True, 'scheduledHooksFrozen': True, 'steeringAndQueueResourcesBound': True, 'stopJoinedRemoteCleanup': True, 'toolReplays': 0, 'fixtureRequests': len(requests), 'liveRequests': 0}
    (directory / 'receipt.json').write_text(json.dumps(receipt, indent=2), encoding='utf-8')
    print(json.dumps(receipt))
finally:
    if host_open:
        host.close()
    release.set()
    for server in (remote, provider):
        server.shutdown()
