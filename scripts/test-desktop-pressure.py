"""Two bounded desktop budget failures through the normal C ABI release.

Targets only the disposable public form; no real provider or private data.
"""
import argparse,json,subprocess,sys,threading,time
from pathlib import Path
from http.server import BaseHTTPRequestHandler,ThreadingHTTPServer
from desktop_test_support import ROOT,NativeHost,wait_for_fixture_focus,fixture_state
from desktop_resource_probe import ResourceProbe

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--directory',type=Path,required=True);parser.add_argument('--worker',action='store_true');args=parser.parse_args();directory=args.directory.resolve()
    if not args.directory.is_absolute() or not directory.is_relative_to(ROOT/'output'):raise SystemExit('Fresh absolute output/ directory required.')
    if not args.worker:
        directory.mkdir(parents=True,exist_ok=False)
        fixture=subprocess.Popen([sys.executable,str(ROOT/'scripts/desktop-form-fixture.py'),'--directory',str(directory)],creationflags=0x08000000)
        try:
            time.sleep(.7)
            result=subprocess.run([sys.executable,__file__,'--directory',str(directory),'--worker'],capture_output=True,text=True,creationflags=0x08000000,timeout=35)
            print(result.stdout);print(result.stderr[-4000:]);assert result.returncode==0
        finally:fixture.terminate();fixture.wait(5)
        return
    client=NativeHost(directory);call=client.call;mode='tool';requests=[]
    class Provider(BaseHTTPRequestHandler):
        def log_message(self,*_):pass
        def do_POST(self):
            body=json.loads(self.rfile.read(int(self.headers['Content-Length'])));requests.append(body);tools=[m for m in body['messages'] if m['role']=='tool']
            if mode=='output':delta={'content':'The form is visible; this partial response is retained.'};reason='length'
            else:
                action={'operation':'observe'} if len(tools)!=1 else {'operation':'type','text':'Budget retained','capture':json.loads(tools[-1]['content'])['capture']['id']}
                delta={'tool_calls':[{'index':0,'id':str(len(tools)),'type':'function','function':{'name':'desktop_control','arguments':json.dumps(action)}}]};reason='tool_calls'
            self.send_response(200);self.send_header('Content-Type','text/event-stream');self.end_headers()
            for frame in [{'choices':[{'index':0,'delta':delta,'finish_reason':None}]},{'choices':[{'index':0,'delta':{},'finish_reason':reason}]}]:self.wfile.write(('data: '+json.dumps(frame)+'\n\n').encode())
            self.wfile.write(b'data: [DONE]\n\n')
    server=ThreadingHTTPServer(('127.0.0.1',0),Provider);threading.Thread(target=server.serve_forever,daemon=True).start()
    probe=ResourceProbe().start()
    try:
        call('bootstrap');policy=call('memories')['automaticPolicy'];call('setAutomaticMemory',enabled=False,revision=policy['revision'])
        call('configure',preferences={'baseUrl':f'http://127.0.0.1:{server.server_port}/v1','model':'fixture'},enabledModels=['fixture'],apiKey='',rememberConnection=False);call('setImageModels',models=['fixture'])
        workspace=directory/'workspace';workspace.mkdir();session=call('createSession',kind='project',path=str(workspace))['session']['id']
        call('saveScopedSettings',session=session,scope='thread',revision=0,patch={'generation':{'maxOutputTokens':128,'timeoutSeconds':15},'task':{'modelCalls':4,'toolCalls':2,'segments':2,'elapsedSeconds':30}})
        time.sleep(.7);call('desktopObserve',id=1,session=session);done,_=client.finish(1);target=next(w for w in done['observation']['windows'] if w['title']=='Dolores local form fixture')
        def start(identity):
            (directory/'form-command.json').write_text(json.dumps({'operation':'restore','sequence':identity}));time.sleep(.4)
            wait_for_fixture_focus(target)
            call('desktopObserve',id=identity,session=session,target=target);done,_=client.finish(identity);capture=done['observation'];access=call('desktopGrant',session=session,capture=capture['id'],consent=True,automatic=True)
            call('start',id=identity+1,session=session,input='Preserve the local note and report remaining verification honestly.',desktopCapture=capture['id'],desktopGrant=access['token'],observationModel='fixture');done,_=client.finish(identity+1);assert not done.get('error'),done
            return call('messagesPage',session=session)['items'][-1]
        item=start(2);assert item['metadata']['paused']['reason']=='stepLimit',item['metadata']['paused'];assert len(requests)==3
        time.sleep(.15);state=fixture_state(directory);assert state['text']=='Budget retained' and state['saveClicks']==0,state
        checkpoint=call('runCheckpoint',session=session,runId=call('runs',session=session)[0]['id']);assert len(checkpoint['uncertainEffects'])==1
        mode='output';before=len(requests);item=start(4);assert item['metadata']['paused']['reason']=='outputLimit';assert 'partial response is retained' in item['content'];assert len(requests)-before==1
        assert json.loads((directory/'form-state.json').read_text())['text']=='Budget retained'
        result={'toolLimitRetainsInputAndUncertainty':True,'outputLimitRetainsPartialReplyWithoutInput':True,'noDefaultBudgetIncreased':True,'resources':probe.finish()};(directory/'public-result.json').write_text(json.dumps(result,indent=2));print(json.dumps(result))
    finally:
        if not probe.end.is_set():probe.finish()
        server.shutdown();server.server_close();client.close()
if __name__=='__main__':main()
