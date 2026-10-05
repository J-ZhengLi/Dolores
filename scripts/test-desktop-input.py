"""Normal-release desktop input qualification with a local provider and owned form.

Run basic then reopen with the same fresh absolute output/ directory. No provider
credentials/private screens; the test drives Dolores's actual broker via its C ABI.
"""
import argparse
import ctypes
from http.server import BaseHTTPRequestHandler,ThreadingHTTPServer
import json
import os
from pathlib import Path
import subprocess
import sys
import threading
import time

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('stage',choices=['basic','reopen']);parser.add_argument('--directory',type=Path,required=True);args=parser.parse_args()
    root=Path(__file__).resolve().parents[1];directory=args.directory.resolve()
    if not args.directory.is_absolute() or not directory.is_relative_to(root/'output'):raise SystemExit('Use a disposable absolute output/ directory.')
    if args.stage=='basic':directory.mkdir(parents=True,exist_ok=False)
    os.environ['DOLORES_DATA_DIR']=str(directory/'data');os.environ['DOLORES_GLOBAL_SKILLS_DIR']=str(directory/'skills')
    bundle=root/'apps/dolores_flutter/build/windows/x64/runner/Release';loader=os.add_dll_directory(str(bundle));native=ctypes.CDLL(str(bundle/'dolores_flutter_bridge.dll'))
    native.dolores_call.argtypes=[ctypes.c_void_p,ctypes.c_size_t];native.dolores_call.restype=ctypes.c_void_p;native.dolores_free.argtypes=[ctypes.c_void_p]
    def envelope(command,**fields):
        data=json.dumps({'command':command,**fields}).encode();buffer=ctypes.create_string_buffer(data);pointer=native.dolores_call(buffer,len(data))
        try:return json.loads(ctypes.string_at(pointer))
        finally:native.dolores_free(pointer)
    def call(command,**fields):
        value=envelope(command,**fields);assert value['ok'],value.get('error');return value.get('result')
    def poll(identity,callback=None):
        events=[];deadline=time.monotonic()+15
        while time.monotonic()<deadline:
            for event in call('poll',id=identity):
                events.append(event)
                if callback:callback(event)
                if event['type']=='done':return event,events
            time.sleep(.03)
        call('cancel',id=identity);raise AssertionError('Bounded fixture deadline exceeded')
    fixture=None;server=None
    try:
        call('bootstrap')
        if args.stage=='reopen':
            expected=json.loads((directory/'expected.json').read_text());session=expected['session']
            assert call('desktopState',session=session)['access'] is None
            assert call('desktopPreview',session=session,capture=expected['capture'])['capture']['session']==session
            assert call('runCheckpoint',session=session,runId=expected['run'])['goal']
            print(json.dumps({'restartRequiresNewConsent':True,'evidenceRetained':True}));return
        phase='basic';requests=[]
        class Provider(BaseHTTPRequestHandler):
            def log_message(self,*_):pass
            def do_POST(self):
                body=json.loads(self.rfile.read(int(self.headers['Content-Length'])));requests.append(body)
                tools=[m for m in body['messages'] if m['role']=='tool'];latest=None
                for message in tools:
                    try:latest=json.loads(message['content']).get('capture',latest)
                    except (TypeError,ValueError):pass
                if not tools:action={'operation':'observe'}
                elif len(tools)==1:
                    if phase=='basic':action={'operation':'type','capture':latest['id'],'text':'Dolores draft'}
                    else:
                        state=json.loads((directory/'form-state.json').read_text());o=latest['observation'];g=o['geometry'];p=state['button']
                        action={'operation':'click','capture':latest['id'],'x':round((p['x']-g['left'])*o['width']/o['originalWidth']),'y':round((p['y']-g['top'])*o['height']/o['originalHeight'])}
                elif len(tools)==2:action={'operation':'observe'}
                else:action=None
                delta={'tool_calls':[{'index':0,'id':'desktop-'+str(len(tools)),'type':'function','function':{'name':'desktop_control','arguments':json.dumps(action)}}]} if action else {'content':'Fresh screen inspected. Local draft remains.'}
                reason='tool_calls' if action else 'stop'
                self.send_response(200);self.send_header('Content-Type','text/event-stream');self.end_headers()
                for frame in [{'choices':[{'index':0,'delta':delta,'finish_reason':None}]},{'choices':[{'index':0,'delta':{},'finish_reason':reason}],'usage':{'prompt_tokens':60,'completion_tokens':20}}]:self.wfile.write(('data: '+json.dumps(frame)+'\n\n').encode())
                self.wfile.write(b'data: [DONE]\n\n')
        server=ThreadingHTTPServer(('127.0.0.1',0),Provider);threading.Thread(target=server.serve_forever,daemon=True).start()
        call('configure',preferences={'baseUrl':f'http://127.0.0.1:{server.server_port}/v1','model':'fixture-vision'},enabledModels=['fixture-vision'],apiKey='',rememberConnection=False)
        call('setImageModels',models=['fixture-vision']);workspace=directory/'workspace';workspace.mkdir();session=call('createSession',kind='project',path=str(workspace))['session']['id']
        policy=call('memories')['automaticPolicy'];call('setAutomaticMemory',enabled=False,revision=policy['revision'])
        call('setTaskPermissions',session=session,revision=0,policy={'mode':'fullAccess','grants':[],'expiresAt':None})
        call('saveScopedSettings',session=session,scope='thread',revision=1,patch={'permissions':{'mode':'fullAccess','grants':[],'expiresAt':None},'generation':{'maxOutputTokens':512,'timeoutSeconds':30},'task':{'modelCalls':4,'toolCalls':4,'segments':1,'elapsedSeconds':60}})
        fixture=subprocess.Popen([sys.executable,str(root/'scripts/desktop-form-fixture.py'),'--directory',str(directory)],creationflags=0x08000000);time.sleep(1)
        call('desktopObserve',id=1,session=session);done,_=poll(1);target=next(w for w in done['observation']['windows'] if w['title']=='Dolores local form fixture')
        def capture(identity):
            call('desktopObserve',id=identity,session=session,target=target);done,_=poll(identity);assert not done.get('error'),done;return done['observation']
        def start(identity,captured):
            access=call('desktopGrant',session=session,capture=captured['id'],automatic=True,consent=True)
            call('start',id=identity,session=session,input='Use the granted local form, preserve existing progress and verify fresh pixels.',desktopCapture=captured['id'],desktopGrant=access['token'],observationModel='fixture-vision')
        captured=capture(2);start(3,captured);done,events=poll(3);assert not done.get('error'),done
        assert not any(e['type']=='toolApproval' for e in events)
        state=json.loads((directory/'form-state.json').read_text());assert state['text']=='Dolores draft' and state['saveClicks']==0,state
        assert sum(1 for e in events if e['type']=='toolResult' and e['record'].get('parts'))==2
        assert any(isinstance(m.get('content'),list) and any(p.get('type')=='image_url' for p in m['content']) for body in requests for m in body['messages'])
        # Native client-coordinate mapping and reviewed click, independently checked.
        phase='save';captured=capture(100);start(101,captured)
        def allowed(event):
            if event['type']=='toolApproval':call('approveTool',id=101,callId=event['request']['callId'],allow=True)
        done,events=poll(101,allowed);assert not done.get('error'),done
        assert json.loads((directory/'form-state.json').read_text())['saved']==['Dolores draft']
        # A click still needs review under Full access. Movement invalidates it.
        phase='move';captured=capture(4);start(5,captured);reviewed=[]
        def moved(event):
            if event['type']=='toolApproval':
                reviewed.append(event);(directory/'form-command.json').write_text(json.dumps({'operation':'move'}));time.sleep(.2)
                call('approveTool',id=5,callId=event['request']['callId'],allow=True)
        done,events=poll(5,moved);assert reviewed,'Full access bypassed desktop input review'
        assert any(e['type']=='toolResult' and e['record']['status']=='error' and 'moved' in e['record']['content'].lower() for e in events),events
        assert json.loads((directory/'form-state.json').read_text())['saveClicks']==1
        # Revoke while queued: no approved input can be committed afterwards.
        phase='revoke';captured=capture(6);start(7,captured)
        def revoked(event):
            if event['type']=='toolApproval':call('desktopRevoke',session=session)
        done,events=poll(7,revoked);assert done.get('error') and json.loads((directory/'form-state.json').read_text())['saveClicks']==1
        assert call('desktopState',session=session)['access'] is None
        run=call('runs',session=session)[0]['id'];(directory/'expected.json').write_text(json.dumps({'session':session,'capture':captured['id'],'run':run}))
        print(json.dumps({'nativeFormDraftVerified':True,'reviewedClientClickVerified':True,'freshImageTransport':True,'fullAccessCannotApproveClicks':True,'movedWindowRefused':True,'queuedRevocationNoDispatch':True,'providerCalls':len(requests)}))
    finally:
        if fixture:fixture.terminate();fixture.wait(5)
        if server:server.shutdown();server.server_close()
        call('shutdown');loader.close()
if __name__=='__main__':main()
