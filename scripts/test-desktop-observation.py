"""Normal-release observation/transport/restart qualification against a disposable window.

Uses a local provider fixture. No provider keys, user history or private screen sharing.
Use a fresh absolute --directory under output/; run save then reopen separately.
"""
import argparse
import base64
import ctypes
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import subprocess
import sys
import shutil
import threading
import time


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('stage',choices=['save','reopen','stall'])
    parser.add_argument('--directory',type=Path,required=True)
    args=parser.parse_args()
    root=Path(__file__).resolve().parents[1]
    directory=args.directory.resolve()
    if not args.directory.is_absolute() or not directory.is_relative_to(root/'output'):
        raise SystemExit('Choose a disposable absolute directory under output/.')
    if args.stage!='reopen':directory.mkdir(exist_ok=False,parents=True)
    os.environ['DOLORES_DATA_DIR']=str(directory/'data')
    os.environ['DOLORES_GLOBAL_SKILLS_DIR']=str(directory/'skills')
    bundle=root/'apps/dolores_flutter/build/windows/x64/runner/Release'
    loader=os.add_dll_directory(str(bundle))
    host=bundle
    if args.stage=='stall':
        host=directory/'host';host.mkdir()
        shutil.copy2(bundle/'dolores_flutter_bridge.dll',host/'dolores_flutter_bridge.dll')
        shutil.copy2(root/'target/debug/examples/stalled.exe',host/'dolores-desktop-helper.exe')
    native=ctypes.CDLL(str(host/'dolores_flutter_bridge.dll'))
    native.dolores_call.argtypes=[ctypes.c_void_p,ctypes.c_size_t];native.dolores_call.restype=ctypes.c_void_p
    native.dolores_free.argtypes=[ctypes.c_void_p]
    def envelope(command,**fields):
        data=json.dumps({'command':command,**fields}).encode();buffer=ctypes.create_string_buffer(data)
        pointer=native.dolores_call(buffer,len(data))
        try:return json.loads(ctypes.string_at(pointer))
        finally:native.dolores_free(pointer)
    def call(command,**fields):
        result=envelope(command,**fields);assert result['ok'],result.get('error');return result.get('result')
    def finish(identity):
        events=[];deadline=time.monotonic()+12
        while time.monotonic()<deadline:
            for event in call('poll',id=identity):
                events.append(event)
                if event['type']=='done':return event,events
            time.sleep(.02)
        call('cancel',id=identity);raise AssertionError('Bounded observation deadline exceeded')
    server=None;fixture=None
    try:
        call('bootstrap')
        expected_file=directory/'expected.json'
        if args.stage=='stall':
            workspace=directory/'workspace';workspace.mkdir()
            session=call('createSession',kind='project',path=str(workspace))['session']['id']
            kernel=ctypes.WinDLL('kernel32',use_last_error=True)
            kernel.OpenProcess.argtypes=[ctypes.c_uint32,ctypes.c_int,ctypes.c_uint32];kernel.OpenProcess.restype=ctypes.c_void_p
            kernel.WaitForSingleObject.argtypes=[ctypes.c_void_p,ctypes.c_uint32]
            kernel.CloseHandle.argtypes=[ctypes.c_void_p]
            def exited():
                pid=int((host/'stall.pid').read_text());handle=kernel.OpenProcess(0x100000,False,pid)
                if not handle:return ctypes.get_last_error()==87
                try:return kernel.WaitForSingleObject(handle,0)==0
                finally:kernel.CloseHandle(handle)
            start=time.monotonic();call('desktopObserve',id=1,session=session)
            done,_=finish(1);deadline_seconds=time.monotonic()-start
            assert 'five seconds' in done.get('error','') and 4<=deadline_seconds<8
            assert exited(),'Deadline left an owned helper alive'
            call('desktopObserve',id=2,session=session);time.sleep(.15)
            start=time.monotonic();call('cancel',id=2);done,_=finish(2);stop_seconds=time.monotonic()-start
            assert 'stopped' in done.get('error','') and stop_seconds<2 and exited()
            assert call('desktopState',session=session)['captures']==[]
            print(json.dumps({'deadlineKilledAndReaped':True,'stopKilledAndReaped':True,'localEvidenceUnchanged':True,
                'deadlineSeconds':round(deadline_seconds,2),'stopSeconds':round(stop_seconds,2)}));return
        if args.stage=='reopen':
            expected=json.loads(expected_file.read_text());session=expected['session'];capture=expected['capture']
            assert call('desktopPreview',session=session,capture=capture['id'])['capture']==capture
            assert call('messagesPage',session=session)['items']==expected['messages']
            assert call('bootstrap')['preferences']==expected['preferences']
            assert any(c['id']==capture['id'] for c in call('desktopState',session=session)['captures'])
            print(json.dumps({'restartEvidence':True,'textAndImageHistoryReadable':True,'selectionPreserved':True}))
            return
        requests=[];mode='normal'
        class Provider(BaseHTTPRequestHandler):
            def log_message(self,*_):pass
            def do_POST(self):
                payload=json.loads(self.rfile.read(int(self.headers['Content-Length'])));requests.append(payload)
                images=[p for m in payload['messages'] if isinstance(m.get('content'),list) for p in m['content'] if p.get('type')=='image_url']
                delta={'content':'Unverified guess'} if mode=='skip' else {'content':'The visible button is labelled Save note.'} if images else {
                    'tool_calls':[{'index':0,'id':'snapshot-one','type':'function','function':{'name':'inspect_desktop_capture','arguments':'{}'}}]}
                reason='length' if images and mode=='length' else 'stop' if images or mode=='skip' else 'tool_calls'
                self.send_response(200);self.send_header('Content-Type','text/event-stream');self.end_headers()
                frames=[{'choices':[{'index':0,'delta':delta,'finish_reason':None}]},
                    {'choices':[{'index':0,'delta':{},'finish_reason':reason}],'usage':{'prompt_tokens':50,'completion_tokens':10}}]
                for frame in frames:self.wfile.write(('data: '+json.dumps(frame)+'\n\n').encode())
                self.wfile.write(b'data: [DONE]\n\n')
        server=ThreadingHTTPServer(('127.0.0.1',0),Provider);threading.Thread(target=server.serve_forever,daemon=True).start()
        policy=call('memories')['automaticPolicy'];call('setAutomaticMemory',enabled=False,revision=policy['revision'])
        call('configure',preferences={'baseUrl':f'http://127.0.0.1:{server.server_port}/v1','model':'ordinary-fixture'},
            apiKey='',rememberConnection=False,enabledModels=['ordinary-fixture','image-fixture'])
        workspace=directory/'workspace';workspace.mkdir()
        session=call('createSession',kind='project',path=str(workspace))['session']['id']
        fixture=subprocess.Popen([sys.executable,str(root/'scripts/desktop-fixture.py')],creationflags=0x08000000)
        time.sleep(.6)
        call('desktopObserve',id=1,session=session)
        done,_=finish(1);assert not done.get('error'),done
        targets=[w for w in done['observation']['windows'] if w['title']=='Dolores observation fixture']
        assert len(targets)==1,'Close any earlier disposable fixture and try again.'
        target=targets[0]
        call('desktopObserve',id=2,session=session,target=target)
        done,_=finish(2);assert not done.get('error'),done
        capture=done['observation'];preview=call('desktopPreview',session=session,capture=capture['id'])
        assert base64.b64decode(preview['data']).startswith(b'\xff\xd8\xff')
        assert not requests
        call('saveDraft',session=session,text='Retained task')
        rejected=envelope('start',id=3,session=session,input='Inspect the screenshot.',desktopCapture=capture['id'],observationModel='image-fixture')
        assert not rejected['ok'] and 'image-capable' in rejected['error']
        assert call('savedDraft',session=session)=='Retained task' and not requests
        call('setImageModels',models=['image-fixture'])
        call('start',id=4,session=session,input='Identify the labelled button in the selected screenshot.',desktopCapture=capture['id'],observationModel='image-fixture')
        done,events=finish(4);assert not done.get('error'),done
        assert len(requests)==2
        assert not any(e['type']=='toolApproval' for e in events)
        assert all(t['function']['name']=='inspect_desktop_capture' for t in requests[0]['tools'])
        wire=requests[1]['messages'];tool_index=next(i for i,m in enumerate(wire) if m['role']=='tool')
        assert isinstance(wire[tool_index]['content'],str) and wire[tool_index]['tool_call_id']=='snapshot-one'
        assert wire[tool_index+1]['role']=='user' and 'untrusted' in wire[tool_index+1]['content'][0]['text']
        messages=call('messagesPage',session=session)['items'];record=messages[-1]['metadata']['agent']['tools'][0]
        assert record['parts']==[capture['reference']]
        assert call('runs',session=session)[0]['model']=='image-fixture'
        assert call('bootstrap')['preferences']['model']=='ordinary-fixture'
        mode='skip'
        call('start',id=7,session=session,input='Inspect the screenshot again.',desktopCapture=capture['id'],observationModel='image-fixture')
        skipped,_=finish(7)
        assert 'without reading' in skipped.get('error','')
        assert call('savedDraft',session=session)=='Inspect the screenshot again.'
        assert call('messagesPage',session=session)['items']==messages
        failed_run=call('runs',session=session)[0]['id'];before=len(requests)
        recovery=envelope('checkpointDraft',session=session,runId=failed_run)
        assert not recovery['ok'] and 'explicit sharing' in recovery['error'] and len(requests)==before
        mode='length'
        call('start',id=8,session=session,input='Inspect with a deliberately limited reply.',desktopCapture=capture['id'],observationModel='image-fixture')
        limited,_=finish(8);assert not limited.get('error'),limited
        messages=call('messagesPage',session=session)['items']
        assert messages[-1]['metadata'].get('paused') and messages[-1]['content']
        before=len(requests)
        recovery=envelope('start',id=9,session=session,input='Continue working on the previous task.',continuation=messages[-1]['id'])
        assert not recovery['ok'] and 'explicit sharing' in recovery['error'] and len(requests)==before
        assert call('messagesPage',session=session)['items']==messages
        mode='normal'
        image_path=directory/'data/desktop-captures'/f"{capture['id']}.jpg"
        original=image_path.read_bytes();image_path.unlink()
        rejected=envelope('start',id=5,session=session,input='Inspect again.',desktopCapture=capture['id'],observationModel='image-fixture')
        assert not rejected['ok'] and 'fresh capture' in rejected['error'] and len(requests)==before
        image_path.write_bytes(original+b'x'*(512*1024))
        assert not envelope('desktopPreview',session=session,capture=capture['id'])['ok']
        image_path.write_bytes(original)
        fixture.terminate();fixture.wait(5);fixture=None
        call('desktopObserve',id=6,session=session,target=target)
        closed,_=finish(6);assert closed.get('error') and 'window' in closed['error'].lower()
        expected_file.write_text(json.dumps({'session':session,'capture':capture,'messages':messages,'preferences':call('bootstrap')['preferences']}))
        print(json.dumps({'normalNativeCapture':True,'localNoProviderRequests':True,'explicitToolImageProjection':True,
            'unsupportedVisionRetainsDraft':True,'missingOversizedAndClosedRefused':True,'ordinaryModelPreserved':True,'unobservedGuessRefused':True,
            'outputLimitedProgressRetained':True,'ordinaryRecoveryCannotEscalateTools':True,
            'providerRequests':len(requests),'imageBytes':len(original),'dimensions':[capture['observation']['width'],capture['observation']['height']]}))
    finally:
        if fixture is not None:fixture.terminate();fixture.wait(5)
        call('shutdown')
        if server:server.shutdown();server.server_close()
        loader.close()


if __name__=='__main__':main()
