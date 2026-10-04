"""Normal native host: exact grants, full access, revocation and stale revisions."""
import ctypes
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

def worker(directory):
    root=Path(__file__).resolve().parents[1]
    os.environ['DOLORES_DATA_DIR']=str(Path(directory)/'data')
    os.environ['DOLORES_GLOBAL_SKILLS_DIR']=str(Path(directory)/'skills')
    bundle=root/'apps/dolores_flutter/build/windows/x64/runner/Release'
    loader=os.add_dll_directory(str(bundle))
    native=ctypes.CDLL(str(bundle/'dolores_flutter_bridge.dll'))
    native.dolores_call.argtypes=[ctypes.c_void_p,ctypes.c_size_t];native.dolores_call.restype=ctypes.c_void_p
    native.dolores_free.argtypes=[ctypes.c_void_p]
    def envelope(command,**fields):
        data=json.dumps({'command':command,**fields}).encode();buf=ctypes.create_string_buffer(data)
        pointer=native.dolores_call(buf,len(data))
        try:return json.loads(ctypes.string_at(pointer))
        finally:native.dolores_free(pointer)
    def call(command,**fields):
        result=envelope(command,**fields);assert result['ok'],result.get('error');return result.get('result')
    requests=[];mode='loop'
    class Handler(BaseHTTPRequestHandler):
        def log_message(self,*args):pass
        def do_POST(self):
            payload=json.loads(self.rfile.read(int(self.headers['Content-Length'])));requests.append(payload)
            if mode=='expired':time.sleep(1.2)
            delta={'tool_calls':[{'index':0,'id':f'call-{len(requests)}','type':'function','function':{'name':'read_text_file','arguments':'{"path":"greeting.txt"}'}}]}
            self.send_response(200);self.send_header('Content-Type','text/event-stream');self.end_headers()
            for item in [{'choices':[{'delta':delta,'finish_reason':None}]},{'choices':[{'delta':{},'finish_reason':'length' if mode=='output' else 'tool_calls'}]}]:
                self.wfile.write(f'data: {json.dumps(item)}\n\n'.encode())
            self.wfile.write(b'data: [DONE]\n\n')
    server=ThreadingHTTPServer(('127.0.0.1',0),Handler);threading.Thread(target=server.serve_forever,daemon=True).start()
    def finish(identity,stop=False):
        deadline=time.monotonic()+15;approvals=0
        while time.monotonic()<deadline:
            for event in call('poll',id=identity):
                if event['type']=='toolApproval':
                    if stop:
                        current=call('taskPermissions',session=session)
                        call('setTaskPermissions',session=session,revision=current['revision'],policy={'mode':'review','grants':[],'expiresAt':None})
                        assert not envelope('approveTool',id=identity,callId=event['request']['callId'],allow=True)['ok']
                    else:call('approveTool',id=identity,callId=event['request']['callId'],allow=True);approvals+=1
                if event['type']=='done':return event,approvals
            time.sleep(.01)
        raise AssertionError('Host did not stop within the fixture deadline')
    try:
        call('bootstrap');policy=call('memories')['automaticPolicy'];call('setAutomaticMemory',enabled=False,revision=policy['revision'])
        call('configure',preferences={'baseUrl':f'http://127.0.0.1:{server.server_port}/v1','model':'fixture'},apiKey='',rememberConnection=False)
        folder=Path(directory)/'project';folder.mkdir();(folder/'greeting.txt').write_text('Hello.\n',newline='')
        session=call('createSession',kind='project',path=str(folder))['session']['id']
        other=call('createSession',kind='project',path=str(folder))['session']['id']
        side=call('createSession',kind='side')['session']['id']
        full={'mode':'fullAccess','grants':[],'expiresAt':None}
        assert not envelope('setTaskPermissions',session=side,revision=0,policy=full)['ok']
        call('start',id=1,session=session,input='review each read');done,count=finish(1)
        assert 'error' not in done and count==3
        grant={'tool':'read_text_file','pathPrefix':'.','command':None,'mcpConnection':None,'mcpRevision':None}
        auto={'mode':'auto','grants':[grant],'expiresAt':None}
        call('setTaskPermissions',session=session,revision=0,policy=auto)
        assert call('taskPermissions',session=other)['policy']['mode']=='review'
        call('start',id=2,session=session,input='granted reads');done,count=finish(2)
        assert 'error' not in done and count==0
        run=call('runs',session=session)[0];events=call('runEvents',session=session,runId=run['id'])
        assert sum(e['kind']=='approvalAutomatic' for e in events)==3
        assert run['effectiveSettings']['permissions']==auto
        call('setTaskPermissions',session=session,revision=1,policy=full)
        call('start',id=3,session=session,input='full access retains budget');done,count=finish(3)
        assert 'error' not in done and count==0
        assert len(call('messagesPage',session=session)['items'][-1]['metadata']['agent']['tools'])==3
        bad={'mode':'auto','grants':[{**grant,'pathPrefix':'other'}],'expiresAt':None}
        call('setTaskPermissions',session=session,revision=2,policy=bad)
        call('start',id=4,session=session,input='revoke at pending uncovered read');done,count=finish(4,stop=True)
        assert 'error' in done and count==0
        run=call('runs',session=session)[0]
        assert not any(e['kind']=='toolIntent' for e in call('runEvents',session=session,runId=run['id']))
        assert not envelope('setTaskPermissions',session=session,revision=2,policy=full)['ok']
        assert call('taskPermissions',session=session)['policy']['mode']=='review'
        mode='expired'
        current=call('taskPermissions',session=session)
        call('setTaskPermissions',session=session,revision=current['revision'],policy={**full,'expiresAt':int(time.time())+1})
        call('start',id=5,session=session,input='grant expires before decision');done,count=finish(5)
        assert 'expired' in done.get('error','').lower() and count==0
        run=call('runs',session=session)[0]
        assert not any(e['kind']=='toolIntent' for e in call('runEvents',session=session,runId=run['id']))
        view=call('scopedSettings',session=session);record=view['scopes'][-1]['record']
        assert not envelope('saveScopedSettings',session=session,scope='thread',revision=record['revision'],patch={'task':None,'generation':None,'interaction':None})['ok']
        call('saveScopedSettings',session=session,scope='thread',revision=record['revision'],patch=record['patch'])
        assert call('taskPermissions',session=session)['policy']['mode']=='fullAccess'
        assert (folder/'greeting.txt').read_bytes()==b'Hello.\n'
        print(json.dumps({'passed':True,'reviewPrompts':3,'autoPrompts':0,'fullPrompts':0,'fullBudgetPreserved':True,'revocationNoDispatch':True,'staleSaveInert':True,'otherChatUnchanged':True}))
    finally:call('shutdown');server.shutdown();loader.close()
if __name__=='__main__':
    if len(sys.argv)>1:worker(sys.argv[1])
    else:
        with tempfile.TemporaryDirectory() as directory:
            subprocess.run([sys.executable,__file__,directory],check=True)
