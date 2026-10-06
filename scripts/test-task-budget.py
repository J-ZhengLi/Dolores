"""Normal native host: effective operation caps, output limits and cancellation."""
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
            calls=4 if mode=='batch' else 1
            path='missing.txt' if mode=='failed' else 'greeting.txt'
            delta={'tool_calls':[{'index':index,'id':f'call-{len(requests)}-{index}','type':'function','function':{'name':'read_text_file','arguments':json.dumps({'path':path})}} for index in range(calls)]}
            self.send_response(200);self.send_header('Content-Type','text/event-stream');self.end_headers()
            for item in [{'choices':[{'delta':delta,'finish_reason':None}]},{'choices':[{'delta':{},'finish_reason':'length' if mode=='output' else 'tool_calls'}]}]:
                self.wfile.write(f'data: {json.dumps(item)}\n\n'.encode())
            self.wfile.write(b'data: [DONE]\n\n')
    server=ThreadingHTTPServer(('127.0.0.1',0),Handler);threading.Thread(target=server.serve_forever,daemon=True).start()
    def finish(identity,stop=False):
        deadline=time.monotonic()+30;approvals=0
        while time.monotonic()<deadline:
            for event in call('poll',id=identity):
                if event['type']=='toolApproval':
                    if stop:call('cancel',id=identity)
                    else:call('approveTool',id=identity,callId=event['request']['callId'],allow=True);approvals+=1
                if event['type']=='done':return event,approvals
            time.sleep(.01)
        raise AssertionError('Host did not stop within the fixture deadline')
    try:
        call('bootstrap');policy=call('memories')['automaticPolicy'];call('setAutomaticMemory',enabled=False,revision=policy['revision'])
        call('configure',preferences={'baseUrl':f'http://127.0.0.1:{server.server_port}/v1','model':'fixture'},apiKey='',rememberConnection=False)
        folder=Path(directory)/'project';folder.mkdir();(folder/'greeting.txt').write_text('Hello.\n',newline='')
        session=call('createSession',kind='project',path=str(folder))['session']['id']
        budget={'modelCalls':8,'toolCalls':5,'segments':1,'elapsedSeconds':60}
        view=call('saveScopedSettings',session=session,scope='thread',revision=0,patch={'task':budget})
        assert view['effective']['task']==budget
        preview=call('context',session=session,input='read until limit')
        assert 'model call 1/8' in preview['messages'][0]['content']
        call('start',id=1,session=session,input='read until limit');done,count=finish(1)
        assert 'error' not in done and count==5
        messages=call('messagesPage',session=session)['items'];paused=messages[-1]
        assert paused['metadata']['paused']['segments']==1 and paused['metadata']['paused']['reason']=='stepLimit'
        assert len(paused['metadata']['agent']['tools'])==5 and len(requests)==6
        before=len(requests)
        refused=envelope('start',id=2,session=session,input='Continue working on the previous task.',continuation=paused['id'])
        assert not refused['ok'] and 'segments' in refused['error'] and len(requests)==before
        assert not envelope('saveScopedSettings',session=session,scope='thread',revision=1,patch={'task':{**budget,'toolCalls':257}})['ok']
        assert call('scopedSettings',session=session)['effective']['task']==budget
        mode='output';call('start',id=3,session=session,input='truncate a pending call');done,count=finish(3)
        assert 'error' not in done and count==0
        assert call('messagesPage',session=session)['items'][-1]['metadata']['paused']['reason']=='outputLimit'
        mode='loop';call('start',id=4,session=session,input='stop near decision');done,count=finish(4,stop=True)
        assert 'error' in done and count==0
        run=call('runs',session=session)[0]
        assert not any(e['kind'] in ['toolIntent','effectIntent'] for e in call('runEvents',session=session,runId=run['id']))
        assert (folder/'greeting.txt').read_bytes()==b'Hello.\n'
        view=call('saveScopedSettings',session=session,scope='thread',revision=1,patch={'task':None})
        assert view['effective']['task']['modelCalls'] is None and view['effective']['task']['toolCalls'] is None
        mode='batch';before=len(requests)
        call('start',id=5,session=session,input='read in batches until a resource checkpoint');done,count=finish(5)
        assert 'error' not in done and count==128
        paused=call('messagesPage',session=session)['items'][-1]
        assert paused['metadata']['paused']['reason']=='checkpoint'
        assert len(paused['metadata']['agent']['tools'])==128 and len(requests)-before==33
        run=call('runs',session=session)[0]
        events=call('runEvents',session=session,runId=run['id'])
        assert len(events)>256 and events[-1]['kind']=='finished'
        mode='failed';before=len(requests)
        call('start',id=6,session=session,input='read a missing file repeatedly');done,count=finish(6)
        assert 'error' not in done and count==0
        paused=call('messagesPage',session=session)['items'][-1]
        assert paused['metadata']['paused']['reason']=='noProgress'
        assert len(paused['metadata']['agent']['tools'])==2 and len(requests)-before==3
        assert (folder/'greeting.txt').read_bytes()==b'Hello.\n'
        print(json.dumps({'passed':True,'modelRequests':len(requests),'toolCap':5,'continuationRefusedBeforeRequest':True,'outputLimitNoDispatch':True,'stopNoEffectIntent':True,'automaticCheckpointTools':128,'checkpointJournalEvents':len(events),'repeatedFailureReceipts':2}))
    finally:call('shutdown');server.shutdown();loader.close()
if __name__=='__main__':
    if len(sys.argv)>1:worker(sys.argv[1])
    else:
        with tempfile.TemporaryDirectory() as directory:
            subprocess.run([sys.executable,__file__,directory],check=True)
