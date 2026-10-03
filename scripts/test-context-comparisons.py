"""Isolated native comparison/restart proof; loopback fixture, no credentials."""
import argparse
import ctypes
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import sqlite3
import threading
import time

parser = argparse.ArgumentParser()
parser.add_argument('stage', choices=['save', 'restore'])
parser.add_argument('--directory', type=Path, required=True)
args = parser.parse_args()
root = Path(__file__).resolve().parents[1]
fixture = args.directory
if not fixture.is_absolute() or not fixture.resolve().is_relative_to(root / 'output'):
    raise SystemExit('Use an absolute isolated directory under output/.')
if args.stage == 'save': fixture.mkdir(parents=True, exist_ok=False)
os.environ['DOLORES_DATA_DIR'] = str(fixture / 'data')
os.environ['DOLORES_GLOBAL_SKILLS_DIR'] = str(fixture / 'global-skills')
bundle = root / 'apps/dolores_flutter/build/windows/x64/runner/Release'
loader = os.add_dll_directory(str(bundle)) if os.name == 'nt' else None
native = ctypes.CDLL(str(bundle / ('dolores_flutter_bridge.dll' if os.name == 'nt' else 'lib/libdolores_flutter_bridge.so')))
native.dolores_call.argtypes = [ctypes.c_void_p, ctypes.c_size_t]
native.dolores_call.restype = ctypes.c_void_p
native.dolores_free.argtypes = [ctypes.c_void_p]

def envelope(command, **fields):
    payload = json.dumps({'command': command, **fields}).encode()
    buffer = ctypes.create_string_buffer(payload)
    pointer = native.dolores_call(buffer, len(payload))
    try: return json.loads(ctypes.string_at(pointer))
    finally: native.dolores_free(pointer)

def call(command, **fields):
    result = envelope(command, **fields)
    assert result['ok'], result.get('error')
    return result.get('result')

requests = []
mode = 'normal'
candidate_started = threading.Event()
class Fixture(BaseHTTPRequestHandler):
    def log_message(self, *_): pass
    def do_POST(self):
        payload = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        requests.append(payload)
        assert len(payload['messages']) == 2
        assert 'tools' not in payload and 'tool_choice' not in payload
        assert 'UNRELATED' not in json.dumps(payload)
        assert payload['max_tokens'] == 512
        candidate = 'Include PASS' in payload['messages'][0]['content']
        text = 'PASS' if candidate or mode == 'tie' else 'ordinary'
        if candidate:
            candidate_started.set()
            if mode == 'slow': time.sleep(2)
            if mode == 'write-failure':
                with sqlite3.connect(fixture / 'data/dolores.db') as db:
                    db.execute("CREATE TRIGGER fail_comparison BEFORE UPDATE ON context_comparisons BEGIN SELECT RAISE(ABORT,'fixture'); END;")
        frames = [{'choices': [{'delta': {'content': text}, 'finish_reason': None}]},
                  {'choices': [{'delta': {}, 'finish_reason': 'length' if candidate and mode == 'limit' else 'stop'}]}]
        try:
            self.send_response(200); self.send_header('Content-Type', 'text/event-stream'); self.end_headers()
            self.wfile.write((''.join('data: '+json.dumps(f)+'\n\n' for f in frames)+'data: [DONE]\n\n').encode())
        except (BrokenPipeError, ConnectionResetError, ConnectionAbortedError): pass

def run(number, session, draft, timeout=5, stop=False):
    candidate_started.clear()
    call('startComparison', id=number, session=session, draft=draft,
         settings={'maxOutputTokens': 512, 'timeoutSeconds': timeout})
    deadline = time.monotonic() + 15
    while time.monotonic() < deadline:
        if stop and candidate_started.is_set(): call('cancel', id=number)
        for event in call('poll', id=number):
            if event['type'] == 'done': return event
        time.sleep(.01)
    call('cancel', id=number)
    raise AssertionError('Fixture timed out')

state_file = fixture / 'state.json'
call('bootstrap')
if args.stage == 'save':
    server = ThreadingHTTPServer(('127.0.0.1', 0), Fixture)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    try:
        call('configure', preferences={'baseUrl': f'http://127.0.0.1:{server.server_port}/v1', 'model': 'fixture'}, apiKey='', rememberConnection=False)
        session = call('createSession', kind='side')['session']['id']
        call('saveMemory', scope='all', title='Unrelated', text='UNRELATED', enabled=True)
        memory = call('saveMemory', scope='all', title='Comparison', text='Include PASS', enabled=True)
        source = next(v for v in call('comparisonSources', session=session)['items'] if v['text']=='Include PASS')
        draft = {'title': 'Synthetic literal comparison', 'baseline': {'label': 'Before', 'text': '', 'source': None},
                 'candidate': source, 'trials': [{'prompt': 'Respond briefly', 'required': ['PASS'], 'forbidden': ['FAIL']}]}
        reports = []
        event = run(1, session, draft)
        assert event['persisted'] and not event.get('error')
        reports.append(event['comparison'])
        assert reports[-1]['summary']['improved']
        for i, payload in enumerate(requests):
            key = 'candidateMessages' if i % 2 else 'baselineMessages'
            expected = reports[-1][key][0]
            assert [(v['role'], v['content']) for v in payload['messages']] == [(v['role'], v['content']) for v in expected]
        # Saved source stays frozen after correction; stale selection refuses before a request.
        fields = {k: memory[k] for k in ('id', 'revision', 'scope', 'title', 'text', 'enabled')}
        call('saveMemory', **{**fields, 'text': 'Include PASS corrected'})
        count = len(requests)
        assert not envelope('startComparison', id=2, session=session, draft=draft, settings={'maxOutputTokens':512,'timeoutSeconds':5})['ok']
        assert len(requests) == count
        draft['candidate'] = {'label': 'Manual frozen copy', 'text': 'Include PASS', 'source': None}
        mode = 'tie'
        reports.append(run(3, session, draft)['comparison'])
        assert not reports[-1]['summary']['improved'] and reports[-1]['summary']['baselinePassed'] == 1
        mode = 'limit'
        reports.append(run(4, session, draft)['comparison'])
        assert reports[-1]['results'][1]['output']=='PASS' and reports[-1]['results'][1]['outcome']=='outputLimit'
        assert reports[-1]['summary']['candidatePassed']==0 and not reports[-1]['summary']['improved']
        mode = 'slow'
        reports.append(run(5, session, draft, timeout=1)['comparison'])
        assert reports[-1]['results'][0]['outcome']=='completed' and 'timed out' in reports[-1]['results'][1]['detail']
        reports.append(run(6, session, draft, stop=True)['comparison'])
        assert reports[-1]['status']=='stopped' and reports[-1]['results'][0]['output']=='ordinary'
        mode = 'write-failure'
        volatile = run(7, session, draft)
        assert not volatile['persisted'] and 'Copy' in volatile['error'] and len(volatile['comparison']['results'])==2
        saved = call('comparison', session=session, comparisonId=volatile['comparison']['id'])
        assert saved['status']=='running' and len(saved['results'])==1
        reports.append(saved)
        with sqlite3.connect(fixture/'data/dolores.db') as db: db.execute('DROP TRIGGER fail_comparison')
        mode = 'normal'
        reports.append(run(8, session, draft)['comparison'])
        assert reports[-1]['summary']['improved'] and len(requests)==14
        assert call('messagesPage', session=session)['items']==[]
        for fmt, suffix in [('json','json'), ('markdown','md')]:
            dest = fixture / ('conversation.'+suffix)
            call('export', session=session, path=str(dest), format=fmt)
            assert 'Synthetic literal comparison' in dest.read_text(encoding='utf-8')
        state_file.write_text(json.dumps({'session':session,'reports':reports}),encoding='utf-8')
    finally: server.shutdown()
else:
    state = json.loads(state_file.read_text(encoding='utf-8'))
    for report in state['reports']:
        assert call('comparison', session=state['session'], comparisonId=report['id']) == report
    assert len(call('comparisonsPage', session=state['session'])['items'])==7
    assert call('messagesPage', session=state['session'])['items']==[]
call('shutdown')
print(json.dumps({'ok':True,'stage':args.stage,'fixtureRequests':len(requests),'liveRequests':0}))
