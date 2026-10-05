"""Normal-host recovery qualification; a test-only wrapper loses real input receipts.

Build the `uncertain` example first. Only a disposable synthetic form is targeted.
Two worker processes exercise a real cold restart without replaying prior input.
"""
import argparse
from http.server import BaseHTTPRequestHandler,ThreadingHTTPServer
import json
from pathlib import Path
import shutil
import subprocess
import sys
import threading
import time
from desktop_test_support import ROOT,BUNDLE,NativeHost,fixture_state

def worker(directory,role):
    host=directory/'host';client=NativeHost(directory,host);call=client.call
    requests=[];mode='effect';server=None
    class Provider(BaseHTTPRequestHandler):
        def log_message(self,*_):pass
        def do_POST(self):
            body=json.loads(self.rfile.read(int(self.headers['Content-Length'])));requests.append(body)
            tools=[m for m in body['messages'] if m['role']=='tool'];latest=None
            for m in tools:
                try:latest=json.loads(m['content']).get('capture',latest)
                except (TypeError,ValueError):pass
            if not tools or mode=='unchanged':action={'operation':'observe'}
            elif len(tools)==1 and mode=='effect':action={'operation':'type','text':'Kept progress','capture':latest['id']}
            else:action=None
            delta={'tool_calls':[{'index':0,'id':'recovery-'+str(len(tools)),'type':'function','function':{'name':'desktop_control','arguments':json.dumps(action)}}]} if action else {'content':'Fresh pixels confirm the existing local draft. No input was repeated.'}
            self.send_response(200);self.send_header('Content-Type','text/event-stream');self.end_headers()
            for frame in [{'choices':[{'index':0,'delta':delta,'finish_reason':None}]},{'choices':[{'index':0,'delta':{},'finish_reason':'tool_calls' if action else 'stop'}]}]:self.wfile.write(('data: '+json.dumps(frame)+'\n\n').encode())
            self.wfile.write(b'data: [DONE]\n\n')
    def image(identity):
        (directory/'form-command.json').write_text(json.dumps({'operation':'restore','sequence':identity}));time.sleep(.15)
        call('desktopObserve',id=identity,session=session,target=target);done,_=client.finish(identity);assert not done.get('error'),done;return done['observation']
    def start(identity,capture,resume=None,inspected=False):
        access=call('desktopGrant',session=session,capture=capture['id'],automatic=True,consent=True)
        value=client.envelope('start',id=identity,session=session,input=goal,desktopCapture=capture['id'],desktopGrant=access['token'],observationModel='fixture-vision',resumeRun=resume,desktopReconciled=inspected)
        return value
    def assert_input_count():
        deadline=time.monotonic()+1
        while time.monotonic()<deadline:
            value=fixture_state(directory)
            if value['text']=='Kept progress' and value['saveClicks']==0:return
            time.sleep(.02)
        raise AssertionError(value)
    try:
        call('bootstrap');server=ThreadingHTTPServer(('127.0.0.1',0),Provider);threading.Thread(target=server.serve_forever,daemon=True).start()
        call('configure',preferences={'baseUrl':f'http://127.0.0.1:{server.server_port}/v1','model':'fixture-vision'},enabledModels=['fixture-vision'],apiKey='',rememberConnection=False);call('setImageModels',models=['fixture-vision'])
        if role=='prepare':
            policy=call('memories')['automaticPolicy'];call('setAutomaticMemory',enabled=False,revision=policy['revision'])
            workspace=directory/'workspace';workspace.mkdir();session=call('createSession',kind='project',path=str(workspace))['session']['id']
            call('saveScopedSettings',session=session,scope='thread',revision=0,patch={'generation':{'maxOutputTokens':512,'timeoutSeconds':30},'task':{'modelCalls':5,'toolCalls':5,'segments':3,'elapsedSeconds':60}})
            call('desktopObserve',id=1,session=session);done,_=client.finish(1);target=next(w for w in done['observation']['windows'] if w['title']=='Dolores local form fixture')
            captured=image(2);goal='Keep the local draft and verify its visible text. Never repeat uncertain input.'
            (host/'fault-mode.txt').write_text('crash');assert start(3,captured)['ok'];done,events=client.finish(3)
            assert not done.get('error'),done
            metadata=call('messagesPage',session=session)['items'][-1]['metadata'];assert metadata['paused']['reason']=='desktopReview'
            assert len(requests)==2,'A failed action triggered another provider call'
            assert_input_count();run=call('runs',session=session)[0];view=call('runCheckpoint',session=session,runId=run['id']);assert len(view['uncertainEffects'])==1,view
            assert call('desktopState',session=session)['recovery']['goal']==goal
            (directory/'expected.json').write_text(json.dumps({'session':session,'target':target,'capture':captured['id'],'run':run['id'],'goal':goal}))
            print(json.dumps({'effectBeforeReceiptPaused':True,'noLaterProviderCall':True,'uncertainCheckpoint':True,'usableWorkRetained':True}));return
        expected=json.loads((directory/'expected.json').read_text());session=expected['session'];target=expected['target'];goal=expected['goal'];run=expected['run']
        assert call('desktopState',session=session)['access'] is None;mode='resume';(host/'fault-mode.txt').write_text('none')
        stale=call('desktopPreview',session=session,capture=expected['capture'])['capture']
        denied=start(4,stale,resume=run,inspected=True);assert not denied['ok'] and 'new local capture' in denied['error'],denied;assert len(requests)==0
        captured=image(5);denied=start(6,captured,resume=run,inspected=False);assert not denied['ok'] and len(requests)==0
        assert start(7,captured,resume=run,inspected=True)['ok'];done,_=client.finish(7);assert not done.get('error'),done
        assert_input_count();latest=call('runs',session=session)[0];assert latest['parentRun']==run and latest['segments']==2 and latest['state']=='completed'
        # A stalled/locked observation is bounded, with a saved goal and no retries.
        mode='unchanged';captured=image(8);before=len(requests);assert start(9,captured)['ok'];done,_=client.finish(9);assert not done.get('error'),done
        metadata=call('messagesPage',session=session)['items'][-1]['metadata'];assert metadata['paused']['reason']=='desktopReview';assert len(requests)-before==3
        captured=image(10);(host/'fault-mode.txt').write_text('locked');before=len(requests);assert start(11,captured)['ok'];done,_=client.finish(11);assert not done.get('error'),done
        assert len(requests)-before==1;assert call('messagesPage',session=session)['items'][-1]['metadata']['paused']['reason']=='desktopReview';assert_input_count()
        # Lose a receipt after actual native input. Stop and the fixed broker
        # deadline must both reap the owned helper and retain the draft.
        import ctypes
        for stop,identity in [(True,12),(False,14)]:
            (directory/'form-command.json').write_text(json.dumps({'operation':'clear','sequence':identity}))
            time.sleep(.2);mode='effect';(host/'fault-mode.txt').write_text('none');captured=image(identity)
            marker=host/'effect-applied.json';marker.unlink(missing_ok=True);(host/'fault-mode.txt').write_text('stall');before=len(requests)
            assert start(identity+1,captured)['ok'];cancelled=False;stopped_at=None
            def stop_after_effect():
                nonlocal cancelled,stopped_at
                if stop and marker.exists() and not cancelled:
                    stopped_at=time.monotonic();call('cancel',id=identity+1);cancelled=True
            began=time.monotonic();done,_=client.finish(identity+1,idle=stop_after_effect,seconds=12);elapsed=time.monotonic()-began
            assert marker.exists() and len(requests)-before==2
            assert_input_count();pid=json.loads(marker.read_text())['pid'];handle=ctypes.windll.kernel32.OpenProcess(0x100000,False,pid)
            if handle:
                assert ctypes.windll.kernel32.WaitForSingleObject(handle,0)==0,'Owned helper still running';ctypes.windll.kernel32.CloseHandle(handle)
            if stop:assert cancelled and time.monotonic()-stopped_at<2 and done.get('error','').startswith('Response stopped.'),done
            else:
                assert elapsed<9 and not done.get('error'),done
                assert call('messagesPage',session=session)['items'][-1]['metadata']['paused']['reason']=='desktopReview'
            assert len(call('runCheckpoint',session=session,runId=call('runs',session=session)[0]['id'])['uncertainEffects'])==1
        print(json.dumps({'coldRestartNeedsConsentAndFreshImage':True,'explicitReconcileNoDuplicateInput':True,'originalGoalAndLineagePreserved':True,'unchangedScreenPausedAfterThree':True,'lockedFixturePausedWithoutRetry':True,'stopAndDeadlineReapHelperAfterRealInput':True}))
    finally:
        if server:server.shutdown();server.server_close()
        client.close()

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--directory',type=Path,required=True);parser.add_argument('--worker',choices=['prepare','reopen']);args=parser.parse_args();directory=args.directory.resolve()
    if not args.directory.is_absolute() or not directory.is_relative_to(ROOT/'output'):raise SystemExit('Use a fresh absolute disposable output/ directory.')
    if args.worker:return worker(directory,args.worker)
    directory.mkdir(parents=True,exist_ok=False);host=directory/'host';host.mkdir()
    shutil.copy2(BUNDLE/'dolores_flutter_bridge.dll',host/'dolores_flutter_bridge.dll');shutil.copy2(BUNDLE/'dolores-desktop-helper.exe',host/'native-desktop-helper.exe');shutil.copy2(ROOT/'target/debug/examples/uncertain.exe',host/'dolores-desktop-helper.exe')
    fixture=subprocess.Popen([sys.executable,str(ROOT/'scripts/desktop-form-fixture.py'),'--directory',str(directory)],creationflags=0x08000000)
    try:
        time.sleep(1)
        for role in ['prepare','reopen']:
            result=subprocess.run([sys.executable,__file__,'--directory',str(directory),'--worker',role],timeout=50,creationflags=0x08000000,capture_output=True,text=True)
            print(result.stdout);print(result.stderr[-5000:]);assert result.returncode==0,role
    finally:fixture.terminate();fixture.wait(5)
if __name__=='__main__':main()
