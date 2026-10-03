"""Isolated native FFI/HTTP proof of revisioned local task feedback and recovery."""
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


def messages(session):
    return call('messagesPage',session=session)['items']

def draft(reply, revision=0, outcome='needsWork', note='Synthetic incorrect response'):
    return {'messageId':reply['id'],'expectedContent':reply['content'],'expectedMetadata':reply.get('metadata'),
            'revision':revision,'outcome':outcome,'note':note}

requests=[]
class Fixture(BaseHTTPRequestHandler):
    def log_message(self,*_): pass
    def do_POST(self):
        payload=json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        requests.append(payload)
        assert 'Synthetic incorrect response' not in json.dumps(payload)
        frames=[{'choices':[{'delta':{'content':'5'},'finish_reason':None}]},
                {'choices':[{'delta':{},'finish_reason':'stop'}]}]
        self.send_response(200);self.send_header('Content-Type','text/event-stream');self.end_headers()
        self.wfile.write((''.join('data: '+json.dumps(f)+'\n\n' for f in frames)+'data: [DONE]\n\n').encode())

def run(number,session):
    call('start',id=number,session=session,input='What is 2+2? Reply with one number.')
    deadline=time.monotonic()+15
    while time.monotonic()<deadline:
        for event in call('poll',id=number):
            if event['type']=='done':
                assert not event.get('error'),event
                return
        time.sleep(.01)
    raise AssertionError('Fixture timed out')

state_file=fixture/'state.json'
call('bootstrap')
if args.stage=='save':
    server=ThreadingHTTPServer(('127.0.0.1',0),Fixture)
    threading.Thread(target=server.serve_forever,daemon=True).start()
    try:
        policy=call('memories')['automaticPolicy']
        call('setAutomaticMemory',enabled=False,revision=policy['revision'])
        call('configure',preferences={'baseUrl':f'http://127.0.0.1:{server.server_port}/v1','model':'fixture'},apiKey='',rememberConnection=False)
        session=call('createSession',kind='side')['session']['id']
        run(1,session)
        original=messages(session)
        reply=original[-1]
        assert reply['content']=='5'
        proposed=draft(reply)
        first=call('saveTaskFeedback',session=session,draft=proposed)
        assert first['revision']==1 and first['outcome']=='needsWork'
        assert not envelope('saveTaskFeedback',session=session,draft=proposed)['ok']
        bad={**proposed,'revision':1,'expectedContent':'stale'}
        assert not envelope('saveTaskFeedback',session=session,draft=bad)['ok']
        assert not envelope('saveTaskFeedback',session='wrong',draft={**proposed,'revision':1})['ok']
        # Inject a transactional write failure, then explicitly recover without losing input.
        with sqlite3.connect(fixture/'data/dolores.db') as db:
            db.execute("CREATE TRIGGER fail_feedback BEFORE UPDATE ON task_feedback BEGIN SELECT RAISE(ABORT,'fixture'); END;")
        updated={**proposed,'revision':1,'outcome':'worked','note':'Reviewed result'}
        assert not envelope('saveTaskFeedback',session=session,draft=updated)['ok']
        assert messages(session)[-1]['feedback']==first
        with sqlite3.connect(fixture/'data/dolores.db') as db: db.execute('DROP TRIGGER fail_feedback')
        second=call('saveTaskFeedback',session=session,draft=updated)
        assert second['revision']==2
        run(2,session)
        assert messages(session)[1]['feedback']==second
        assert len(requests)==2
        for fmt in ['json','markdown']:
            dest=fixture / ('conversation.json' if fmt == 'json' else 'conversation.md')
            call('export',session=session,path=str(dest),format=fmt)
            assert 'Reviewed result' in dest.read_text(encoding='utf-8')
        state_file.write_text(json.dumps({'session':session,'messages':messages(session),'original':original}),encoding='utf-8')
    finally: server.shutdown()
else:
    state=json.loads(state_file.read_text(encoding='utf-8'))
    session=state['session']
    assert messages(session)==state['messages']
    reply=messages(session)[1]
    clear=call('saveTaskFeedback',session=session,draft=draft(reply,2,None,''))
    assert clear['revision']==3 and clear['outcome'] is None
    assert not envelope('saveTaskFeedback',session=session,draft=draft(reply,2))['ok']
    assert messages(session)[1]['content']==state['original'][1]['content']
call('shutdown')
print(json.dumps({'ok':True,'stage':args.stage,'fixtureRequests':len(requests),'liveRequests':0}))
