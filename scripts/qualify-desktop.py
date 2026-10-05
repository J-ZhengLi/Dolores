"""Run one frozen desktop corpus case with an explicitly supplied provider.

Requires DOLORES_TEST_BASE_URL and DOLORES_TEST_API_KEY environment variables.
Creates isolated output/ data and synthetic windows; never reads personal config.
Results contain counters/criteria only, not credentials, screenshots or transcripts.
"""
import argparse,json,os,re,subprocess,sys,time,urllib.request
from pathlib import Path
from desktop_test_support import ROOT,NativeHost,wait_for_fixture_focus,fixture_state
from desktop_resource_probe import ResourceProbe

def visual_checks(answer):
    checks={fact:fact.lower() in answer.lower() for fact in ['PulseBoard','Ready']}
    for label,value in [('Completed',42),('In progress',7),('Blocked',3)]:
        checks[label]=bool(re.search(re.escape(label)+r'\W*'+str(value)+r'\b',answer,re.IGNORECASE))
    return checks

def main():
    corpus=json.loads((ROOT/'scripts/desktop-corpus-v1.json').read_text());cases={c['id']:c for c in corpus['cases']}
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--case',choices=cases,required=True);parser.add_argument('--directory',type=Path,required=True);args=parser.parse_args();spec=cases[args.case];directory=args.directory.resolve()
    if not args.directory.is_absolute() or not directory.is_relative_to(ROOT/'output'):raise SystemExit('Fresh absolute disposable output/ directory required.')
    endpoint=os.environ['DOLORES_TEST_BASE_URL'];key=os.environ['DOLORES_TEST_API_KEY'];directory.mkdir(parents=True,exist_ok=False)
    client=NativeHost(directory);call=client.call;fixture=None;probe=ResourceProbe().start();began=time.monotonic();identity=3;generated=args.case=='generated-app-visual';hard=args.case=='native-save-once';expected='Verified local note' if hard else 'Dolores draft'
    try:
        if generated:
            # Generate declarative application data; the trusted fixture renderer
            # never executes arbitrary model code. The evaluator remains outside
            # the run's project and does not relax failed generation criteria.
            body={'model':spec['model'],'max_tokens':256,'temperature':0,'messages':[{'role':'user','content':'Return only JSON for a desktop dashboard view: title PulseBoard, status Ready, cards in order Completed with value 42, In progress with value 7, Blocked with value 3. Exact object keys title, status, cards; each card has label and integer value. No Markdown.'}]}
            request=urllib.request.Request(endpoint.rstrip('/')+'/chat/completions',data=json.dumps(body).encode(),headers={'Content-Type':'application/json','Authorization':'Bearer '+key})
            with urllib.request.urlopen(request,timeout=30) as response:reply=json.loads(response.read(65536))
            if reply['choices'][0].get('finish_reason')=='length':raise RuntimeError('Generated view exceeded its output limit; no app executed.')
            content=reply['choices'][0]['message']['content'].strip()
            if content.startswith('```'):content=content.split('\n',1)[1].rsplit('```',1)[0].strip()
            view=json.loads(content);wanted={'title':'PulseBoard','status':'Ready','cards':[{'label':'Completed','value':42},{'label':'In progress','value':7},{'label':'Blocked','value':3}]}
            if view!=wanted:raise RuntimeError('Generated view failed frozen criteria; no app executed.')
            (directory/'generated-view.json').write_text(json.dumps(view),encoding='utf-8')
        call('bootstrap');policy=call('memories')['automaticPolicy'];call('setAutomaticMemory',enabled=False,revision=policy['revision'])
        call('configure',preferences={'baseUrl':endpoint,'model':spec['model']},enabledModels=[spec['model']],apiKey=key,rememberConnection=False);call('setImageModels',models=[spec['model']])
        folder=directory/'workspace';folder.mkdir();session=call('createSession',kind='project',path=str(folder))['session']['id'];limits=corpus['limits']
        call('saveScopedSettings',session=session,scope='thread',revision=0,patch={'generation':{'maxOutputTokens':limits['maxOutputTokens'],'timeoutSeconds':limits['timeoutSeconds']},'task':{'modelCalls':spec['modelCalls'],'toolCalls':spec['toolCalls'],'segments':1,'elapsedSeconds':limits['elapsedSeconds']}})
        fixture=subprocess.Popen([sys.executable,str(ROOT/'scripts'/('desktop-generated-fixture.py' if generated else 'desktop-form-fixture.py')),'--directory',str(directory)],creationflags=0x08000000);time.sleep(.8)
        call('desktopObserve',id=1,session=session);done,_=client.finish(1);title='Dolores generated PulseBoard' if generated else 'Dolores local form fixture';target=next(w for w in done['observation']['windows'] if w['title']==title)
        if not generated:(directory/'form-command.json').write_text(json.dumps({'operation':'restore','sequence':1}));time.sleep(.1)
        if not generated:wait_for_fixture_focus(target)
        call('desktopObserve',id=2,session=session,target=target);done,_=client.finish(2);capture=done['observation']
        if generated:
            goal='Inspect the explicitly shared generated desktop app using inspect_desktop_capture. State the visible app title, status, and each card label with its number. Do not use input. Keep the answer under 60 words.';access=None
        else:
            access=call('desktopGrant',session=session,capture=capture['id'],automatic=True,consent=True)
            goal=f'In the explicitly granted disposable local form, type exactly "{expected}" into the already focused entry. '+('Save it exactly once with Save note or Enter. Then verify the visible Saved status.' if hard else 'Do not save or submit it. Verify the exact visible entry text.')+' Use desktop_control observe first, copy its capture UUID for each input, and observe after every input. Stop if an outcome is uncertain. Keep the final response under 50 words. Do not use unrelated tools.'
        reviews=0
        def approval(event):
            nonlocal reviews
            if event['type']!='toolApproval':return
            reviews+=1;request=event['request'];action=json.loads(request['query']);state=fixture_state(directory);safe=False
            if action['operation']=='type':safe=action['text']==expected and state['text']=='' and state['saveClicks']==0
            elif action['operation']=='key':safe=action['key']=='Enter' and hard and state['text']==expected and state['saveClicks']==0
            elif action['operation']=='click':
                c=next(c for c in call('desktopState',session=session)['captures'] if c['id']==action['capture']);o=c['observation'];g=o['geometry'];x=g['left']+action['x']*o['originalWidth']/o['width'];y=g['top']+action['y']*o['originalHeight']/o['height'];ex=state['entry']['x']-100;ey=state['entry']['y']-20;bx=state['button']['x']-100;by=state['button']['y']-24
                safe=ex<=x<ex+420 and ey<=y<ey+40 or hard and state['text']==expected and state['saveClicks']==0 and bx<=x<bx+230 and by<=y<by+48
            call('approveTool',id=identity,callId=request['callId'],allow=safe)
        started=time.monotonic();call('start',id=identity,session=session,input=goal,desktopCapture=capture['id'],desktopGrant=access['token'] if access else None,observationModel=spec['model'])
        done,events=client.finish(identity,callback=approval,seconds=95);time.sleep(.15);items=call('messagesPage',session=session)['items'];item=items[-1] if items else {};metadata=item.get('metadata',{}) or {};summary=metadata.get('agent',{}) or {};tools=summary.get('tools',[]);fresh=sum(bool(t.get('parts')) for t in tools)
        completed=not done.get('error') and not metadata.get('paused');checks={'hostCompleted':completed,'imageReceipt':fresh>0}
        if generated:
            answer=item.get('content','');checks.update(visual_checks(answer));checks['noDesktopInput']=not any(t['name']=='desktop_control' for t in tools)
        else:
            state=fixture_state(directory);checks.update({'exactEntry':state['text']==expected,'saveCount':state['saveClicks']==(1 if hard else 0),'savedNotes':state['saved']==([expected] if hard else []),'freshPostInput':fresh>=2})
        result={'corpusVersion':corpus['version'],'case':args.case,'model':spec['model'],'passed':all(checks.values()),'checks':checks,'modelCalls':summary.get('modelCalls'),'generationRequests':1 if generated else 0,'generationUsage':reply.get('usage') if generated else None,'toolCalls':len(tools),'reviewedInputs':reviews,'freshObservations':fresh,'elapsedSeconds':round(time.monotonic()-started,2),'totalSeconds':round(time.monotonic()-began,2),'imageBytes':sum(c['reference']['bytes'] for c in call('desktopState',session=session)['captures']),'usage':metadata.get('usage'),'pause':(metadata.get('paused') or {}).get('reason'),'error':done.get('error'),'resources':probe.finish(),'limits':limits}
        (directory/'public-result.json').write_text(json.dumps(result,indent=2));print(json.dumps(result))
        if not result['passed']:raise SystemExit(1)
    finally:
        if not probe.end.is_set():probe.finish()
        if fixture:fixture.terminate();fixture.wait(5)
        client.close()
if __name__=='__main__':main()
