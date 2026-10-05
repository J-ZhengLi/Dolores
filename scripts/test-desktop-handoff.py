"""Bounded conversation-to-window qualification with isolated synthetic data.

Without --model uses an owned deterministic endpoint. With --model requires
DOLORES_TEST_BASE_URL / DOLORES_TEST_API_KEY. No personal window is shared.
"""
import argparse, json, os, subprocess, sys, threading, time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from desktop_test_support import ROOT, NativeHost, fixture_state, wait_for_fixture_focus

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', required=True, type=Path)
    parser.add_argument('--model')
    parser.add_argument('--input', action='store_true')
    args = parser.parse_args()
    directory = args.directory.resolve()
    if not args.directory.is_absolute() or not directory.is_relative_to(ROOT/'output'):
        raise SystemExit('Fresh absolute disposable output/ directory required')
    directory.mkdir(parents=True, exist_ok=False)
    seen = []
    class Provider(BaseHTTPRequestHandler):
        def log_message(self, *_): pass
        def do_POST(self):
            body = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
            seen.append(body)
            names = [t['function']['name'] for t in body.get('tools', [])]
            if 'request_desktop_access' in names:
                name, arguments = 'request_desktop_access', {'purpose':'Inspect the local form'}
            elif 'inspect_desktop_capture' in names and not any(m['role']=='tool' for m in body['messages']):
                name, arguments = 'inspect_desktop_capture', {}
            else: name = None
            delta = {'tool_calls':[{'index':0,'id':'fixture-call','type':'function','function':{'name':name,'arguments':json.dumps(arguments)}}]} if name else {'content':'The shared local form has an entry and Save note button.'}
            chunks = [{'choices':[{'index':0,'delta':delta,'finish_reason':None}]}, {'choices':[{'index':0,'delta':{},'finish_reason':'tool_calls' if name else 'stop'}]}]
            self.send_response(200); self.send_header('Content-Type','text/event-stream'); self.end_headers()
            for chunk in chunks: self.wfile.write(('data: '+json.dumps(chunk)+'\n\n').encode())
            self.wfile.write(b'data: [DONE]\n\n')
    server = None
    if args.model:
        endpoint, key, model = os.environ['DOLORES_TEST_BASE_URL'], os.environ['DOLORES_TEST_API_KEY'], args.model
    else:
        server = ThreadingHTTPServer(('127.0.0.1',0),Provider)
        threading.Thread(target=server.serve_forever,daemon=True).start()
        endpoint, key, model = f'http://127.0.0.1:{server.server_port}/v1', '', 'fixture'
    fixture = subprocess.Popen([sys.executable,str(ROOT/'scripts/desktop-form-fixture.py'),'--directory',str(directory)], creationflags=subprocess.CREATE_NO_WINDOW)
    client = NativeHost(directory)
    began = time.monotonic()
    try:
        call = client.call
        call('bootstrap')
        policy = call('memories')['automaticPolicy']
        call('setAutomaticMemory',enabled=False,revision=policy['revision'])
        call('configure',preferences={'baseUrl':endpoint,'model':model},enabledModels=[model],apiKey=key,rememberConnection=False)
        call('setImageModels',models=[model])
        folder = directory/'workspace'; folder.mkdir()
        session = call('createSession',kind='project',path=str(folder))['session']['id']
        call('saveScopedSettings',session=session,scope='thread',revision=0,patch={'generation':{'maxOutputTokens':1024,'timeoutSeconds':30},'task':{'modelCalls':12 if args.input else 6,'toolCalls':12 if args.input else 6,'segments':4,'elapsedSeconds':90}})
        goal = 'Use request_desktop_access to ask me to choose the local form window. Then '+('type exactly Verified local note into its text entry once, do not save or submit, and verify the visible text with a fresh observation.' if args.input else 'inspect its visible controls and describe the entry and button. Do not use commands or change anything.')
        call('start',id=1,session=session,input=goal)
        first, events = client.finish(1,seconds=35)
        items = call('messagesPage',session=session)['items']
        paused = (items[-1].get('metadata',{}).get('paused') or {}).get('reason') if items else None
        if first.get('error') or paused != 'desktopAccess':
            result = {'passed':False,'stage':'requestWindow','model':model,'pause':paused,'error':first.get('error'),'noWindowShared':True}
            (directory/'public-result.json').write_text(json.dumps(result,indent=2)); print(json.dumps(result)); raise SystemExit(1)
        source = call('runs',session=session)[0]
        first_usage = next(e['data'] for e in call('runEvents',session=session,runId=source['id']) if e['kind']=='desktopHandoff')
        # A pending unrelated attachment must survive without reaching the model.
        call('saveDraft',session=session,text='Unrelated draft preserved')
        attachment = folder/'pending.txt'; attachment.write_text('UNRELATED-PENDING-ATTACHMENT')
        call('attachFile',session=session,path=str(attachment))
        call('desktopObserve',id=2,session=session); listing,_ = client.finish(2)
        assert not listing.get('error'), listing.get('error')
        windows = listing['observation']['windows']
        target = next(w for w in windows if w['title']=='Dolores local form fixture')
        if args.input: wait_for_fixture_focus(target)
        call('desktopObserve',id=3,session=session,target=target); captured,_=client.finish(3)
        capture=captured['observation']
        grant=call('desktopGrant',session=session,capture=capture['id'],automatic=False,consent=True) if args.input else None
        def approval(event):
            if event['type']!='toolApproval': return
            request=event['request']; safe=False
            if request['name']=='desktop_control':
                action=json.loads(request['query']); state=fixture_state(directory)
                if action['operation']=='type': safe=action.get('text')=='Verified local note' and state['text']==''
                elif action['operation']=='click':
                    current=next(c for c in call('desktopState',session=session)['captures'] if c['id']==action['capture'])['observation']; geometry=current['geometry']
                    x=geometry['left']+action['x']*current['originalWidth']/current['width']; y=geometry['top']+action['y']*current['originalHeight']/current['height']
                    entry=state['entry']; safe=entry['x']-100 <= x < entry['x']+320 and entry['y']-20 <= y < entry['y']+20
                elif action['operation']=='key': safe=action.get('key') in ['Tab','Home','End','Left','Right']
            call('approveTool',id=4,callId=request['callId'],allow=safe)
        call('start',id=4,session=session,input=goal,desktopHandoff=True,resumeRun=source['id'],desktopCapture=capture['id'],desktopGrant=grant['token'] if grant else None,observationModel=model)
        done,events=client.finish(4,callback=approval,seconds=95)
        latest=call('runs',session=session)[0]
        items=call('messagesPage',session=session)['items']; metadata=(items[-1].get('metadata') or {}); tools=metadata.get('agent',{}).get('tools',[])
        active_budget=next((e['taskBudget'] for e in events if e['type']=='started'),{})
        state=fixture_state(directory)
        checks={'hostCompleted':not done.get('error') and not metadata.get('paused'), 'lineage':latest['parentRun']==source['id'], 'remainingModels':active_budget.get('modelCalls')== (12 if args.input else 6)-first_usage['modelCalls'], 'remainingTools':active_budget.get('toolCalls')== (12 if args.input else 6)-first_usage['toolCalls'], 'draftRetained':call('savedDraft',session=session)=='Unrelated draft preserved', 'attachmentRetained':len(call('draftAttachments',session=session))==1, 'freshImage':any(t.get('parts') for t in tools), 'noSubmit':state['saveClicks']==0}
        if args.input: checks.update({'exactEntry':state['text']=='Verified local note','freshPostInput':sum(bool(t.get('parts')) for t in tools)>=2})
        if not args.model: checks['unrelatedNotShared']=all('UNRELATED-PENDING-ATTACHMENT' not in json.dumps(body) for body in seen)
        result={'passed':all(checks.values()),'model':model,'checks':checks,'pause':(metadata.get('paused') or {}).get('reason'),'error':done.get('error'),'elapsedSeconds':round(time.monotonic()-began,2),'modelCalls':metadata.get('agent',{}).get('modelCalls'),'toolCalls':len(tools)}
        (directory/'public-result.json').write_text(json.dumps(result,indent=2)); print(json.dumps(result))
        if not result['passed']: raise SystemExit(1)
    finally:
        client.close(); fixture.terminate(); fixture.wait(5)
        if server: server.shutdown(); server.server_close()
if __name__=='__main__': main()
