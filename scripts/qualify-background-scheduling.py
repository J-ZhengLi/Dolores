"""Normal packaged tray owner and hidden-clock execution in a public profile.

The due timestamp is a fixture. Native lifecycle messages exercise the production
Close/Open/Quit path; they do not claim physical tray-menu clicks.
"""
import argparse,ctypes,hashlib,importlib.util,json,os,sqlite3,subprocess,sys,threading,time
from http.server import BaseHTTPRequestHandler,ThreadingHTTPServer
from pathlib import Path
from desktop_test_support import NativeHost,ROOT,BUNDLE
import desktop
from desktop_resource_probe import cpu_seconds,memory,children

p=argparse.ArgumentParser();p.add_argument('--directory',type=Path,required=True);p.add_argument('--initialize',action='store_true');p.add_argument('--configure-live',action='store_true');p.add_argument('--forget-live',action='store_true');p.add_argument('--port',type=int);p.add_argument('--resources',action='store_true');p.add_argument('--live-source',type=Path);p.add_argument('--live-model',help='Explicit enabled model ID for the optional live report.');p.add_argument('--live-reasoning',default='providerDefault',choices=['providerDefault','deepseekThinkingOff','openaiLow','openaiMedium','openaiHigh']);p.add_argument('--mode',choices=['normal','offline','approval'],default='normal');a=p.parse_args();directory=a.directory.resolve();assert directory.is_relative_to(ROOT/'output')
if a.live_source and not a.live_model:p.error('--live-model is required with --live-source')
mode=a.mode;requests=[]
class Fixture(BaseHTTPRequestHandler):
    def log_message(self,*_):pass
    def do_POST(self):
        data=json.loads(self.rfile.read(int(self.headers['Content-Length'])));requests.append(data)
        scheduled='This is an occurrence of the user' in data['messages'][0]['content']
        if scheduled and mode=='offline':self.send_response(503);self.end_headers();return
        first=data['messages'][-1]['role']=='user';calls=[]
        if not scheduled and first:
            fields={'rule':{'kind':'weekdays','time':'21:00','date':None,'weekdays':[0,1,2,3,4],'zone':'Asia/Shanghai'},'skill':'daily-report','model':None}
            calls=[{'index':0,'id':'create','type':'function','function':{'name':'schedule_task','arguments':json.dumps(fields)}}]
        elif scheduled and mode=='approval' and first:
            calls=[{'index':0,'id':'read','type':'function','function':{'name':'read_text_file','arguments':json.dumps({'path':'public-note.txt'})}}]
        delta={'tool_calls':calls} if calls else {'content':'CLOSED-UI-REPORT\nDone: public demonstration.\nNext: review tomorrow.' if scheduled else 'Task saved.'}
        events=[{'choices':[{'delta':delta,'finish_reason':None}]},{'choices':[{'delta':{},'finish_reason':'tool_calls' if calls else 'stop'}]},{'choices':[],'usage':{'prompt_tokens':120,'completion_tokens':30,'total_tokens':150}}]
        self.send_response(200);self.send_header('Content-Type','text/event-stream');self.end_headers();self.wfile.write((''.join('data: '+json.dumps(e)+'\n\n' for e in events)+'data: [DONE]\n\n').encode())
def read():
    with sqlite3.connect((directory/'data/dolores.db').as_uri()+'?mode=ro',uri=True) as db:
        rows=[json.loads(r[0]) for r in db.execute('SELECT data FROM scheduled_occurrences')]
        return rows
def wait(predicate,seconds=30):
    deadline=time.monotonic()+seconds
    while time.monotonic()<deadline:
        v=predicate()
        if v:return v
        time.sleep(.1)
    raise AssertionError('Bounded background qualification condition was not reached')

class OwnedApp:
    def __init__(self,pid):
        self.pid=pid
        kernel=ctypes.windll.kernel32;kernel.OpenProcess.restype=ctypes.c_void_p
        self.handle=kernel.OpenProcess(0x1000|0x100000,False,pid)
        assert self.handle
    def poll(self):
        code=ctypes.c_uint();fn=ctypes.windll.kernel32.GetExitCodeProcess;fn.argtypes=[ctypes.c_void_p,ctypes.POINTER(ctypes.c_uint)]
        assert fn(self.handle,ctypes.byref(code));return None if code.value==259 else code.value
    def wait(self,timeout):
        wait(lambda:self.poll() is not None,timeout);return self.poll()

def launch():
    record=directory/'owner.json'
    result=subprocess.run([sys.executable,str(ROOT/'scripts/desktop.py'),'launch','--data-directory',str(directory/'data'),'--pid-file',str(record),'--replace-owned'],env=env,capture_output=True,text=True,encoding='utf-8')
    assert result.returncode==0,result.stderr
    return OwnedApp(json.loads(record.read_text())['pid'])

def idle_sample(pid,seconds=60):
    count=len(requests);before=cpu_seconds(pid);start=time.monotonic();values=[];extra=set()
    while time.monotonic()-start<seconds:
        values.append(memory(pid));extra.update(children(pid,names=None));time.sleep(.5)
    elapsed=time.monotonic()-start
    assert len(requests)==count,'Idle generated an unexpected model request'
    return {'seconds':elapsed,'cpuPercentOneCore':100*(cpu_seconds(pid)-before)/elapsed,'privateBytesPeak':max(v['privateBytes'] for v in values),'workingBytesPeak':max(v['workingBytes'] for v in values),'additionalProcesses':len(extra),'modelRequests':len(requests)-count}
def digest(path):
    with sqlite3.connect(path.resolve().as_uri()+'?mode=ro',uri=True) as db:
        return {n:hashlib.sha256(repr(sorted(db.execute('SELECT * FROM '+n).fetchall(),key=repr)).encode()).hexdigest() for (n,) in db.execute("SELECT name FROM sqlite_master WHERE type='table'")}
if a.configure_live or a.forget_live:
    h=NativeHost(directory);h.call('bootstrap')
    if a.forget_live:h.call('forgetConnection')
    else:
        secret=json.load(sys.stdin)
        with sqlite3.connect(directory/'data/dolores.db') as db:
            base,model=db.execute('SELECT base_url,model FROM preferences').fetchone()
            models=json.loads(db.execute('SELECT models FROM model_choices').fetchone()[0])
        assert a.live_model in models,'Choose an enabled live model'
        h.call('configure',preferences={'baseUrl':base,'model':model},apiKey=secret['key'],rememberConnection=True,enabledModels=models)
        del secret
    h.close();sys.exit(0)
if a.initialize:
    h=NativeHost(directory);h.call('bootstrap');h.call('configure',preferences={'baseUrl':f'http://127.0.0.1:{a.port}/v1','model':'fixture'},apiKey='',rememberConnection=True,enabledModels=['fixture'])
    policy=h.call('memories')['automaticPolicy'];h.call('setAutomaticMemory',enabled=False,revision=policy['revision'])
    project=directory/'project';skill=project/'.agents/skills/daily-report/SKILL.md';skill.parent.mkdir(parents=True);skill.write_text('---\nname: daily-report\ndescription: Write a short public report\n---\nNo tools needed. Report CLOSED-UI-REPORT and two concise lines.\n',encoding='utf-8');(project/'public-note.txt').write_text('Public approved-only note.',encoding='utf-8')
    source=h.call('createSession',kind='project',path=str(project))['session']['id'];review=h.call('reviewSkill',session=source,name='daily-report');h.call('activateSkill',session=source,token=review['token'])
    prompt='Every weekday at 9pm, write my daily report using skill daily-report'
    if a.live_source:prompt+='; return CLOSED-UI-REPORT and two short lines as your chat reply. The whole report belongs in this conversation.'
    h.call('start',id=1,session=source,input=prompt);done,_=h.finish(1,seconds=20);assert not done.get('error'),done
    h.call('saveDraft',session=source,text='RETAINED-HOME-DRAFT');h.call('setBackgroundPolicy',enabled=True,revision=1)
    with sqlite3.connect(directory/'data/dolores.db') as db:
        task=json.loads(db.execute('SELECT data FROM scheduled_tasks').fetchone()[0]);task['nextDue']=int(time.time())+35;db.execute('UPDATE scheduled_tasks SET data=?',(json.dumps(task),))
    (directory/'source.json').write_text(json.dumps({'source':source,'task':task['id'],'due':task['nextDue']}));h.close();sys.exit(0)

directory.mkdir(parents=True,exist_ok=False);server=ThreadingHTTPServer(('127.0.0.1',0),Fixture);threading.Thread(target=server.serve_forever,daemon=True).start()
app=None;receipt={};original_before=digest(a.live_source) if a.live_source else None
try:
    initializer=[sys.executable,__file__,'--directory',str(directory),'--initialize','--port',str(server.server_port),'--mode',mode]
    if a.live_source:initializer+=['--live-source',str(a.live_source.resolve()),'--live-model',a.live_model,'--live-reasoning',a.live_reasoning]
    r=subprocess.run(initializer,capture_output=True,text=True,encoding='utf-8');assert not r.returncode,r.stderr
    if a.live_source:
        assert mode=='normal' and not a.resources
        # Public fixture task plus real pinned execution; no original transcript or
        # vault material is copied, and no original configuration is changed.
        with sqlite3.connect(a.live_source.resolve().as_uri()+'?mode=ro',uri=True) as source, sqlite3.connect(directory/'data/dolores.db') as target:
            for table in ('preferences','model_choices','model_contexts','request_settings','model_request_settings','model_images'):
                rows=source.execute('SELECT * FROM '+table).fetchall();target.execute('DELETE FROM '+table)
                if rows:target.executemany('INSERT INTO '+table+' VALUES('+','.join('?' for _ in rows[0])+')',rows)
            base=target.execute('SELECT base_url FROM preferences').fetchone()[0]
            task=json.loads(target.execute('SELECT data FROM scheduled_tasks').fetchone()[0])
            task['preferences']={'baseUrl':base,'model':a.live_model}
            task['effective']['request'].update(maxOutputTokens=1024,timeoutSeconds=30,reasoning=a.live_reasoning)
            target.execute('UPDATE scheduled_tasks SET data=?',(json.dumps(task),))
        spec=importlib.util.spec_from_file_location('configured',ROOT/'scripts/test-workspace-editor.py');module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
        _,key=module.connection(a.live_source.resolve().parent)
        configured=subprocess.run([sys.executable,__file__,'--directory',str(directory),'--configure-live','--live-model',a.live_model],input=json.dumps({'key':key}),capture_output=True,text=True,encoding='utf-8');del key
        assert configured.returncode==0,'Isolated remembered connection setup failed'
        receipt['liveModel']=a.live_model;receipt['liveLimit']='One public report; 1024 output tokens, 30-second provider timeout; fixture-created task and due time'
    env=dict(os.environ,DOLORES_DATA_DIR=str(directory/'data'),DOLORES_GLOBAL_SKILLS_DIR=str(directory/'skills'))
    desktop.require_normal_build();app=launch()
    handle=desktop.window_visible(app.pid);wait(lambda:desktop.profile_window(directory/'data'));time.sleep(4)
    user=ctypes.WinDLL('user32',use_last_error=True);user.PostMessageW.argtypes=[ctypes.c_void_p,ctypes.c_uint,ctypes.c_size_t,ctypes.c_ssize_t];user.IsWindowVisible.argtypes=[ctypes.c_void_p]
    user.PostMessageW(handle,0x0010,0,0);wait(lambda:not user.IsWindowVisible(handle));assert app.poll() is None
    receipt['closedUi']=True;receipt['ownerPid']=app.pid
    expected={'normal':'succeeded','offline':'failed','approval':'waitingForApproval'}[mode]
    terminal={expected,'failed','paused','waitingForApproval'}
    occurrence=wait(lambda:next((o for o in read() if o['state'] in terminal),None),70)
    receipt['observedState']=occurrence['state'];assert occurrence['state']==expected,receipt
    receipt['state']=occurrence['state'];receipt['occurrences']=len(read());assert len(read())==1
    second=subprocess.Popen([str(desktop.executable())],cwd=BUNDLE,env=env,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL);assert second.wait(timeout=8)==0
    wait(lambda:user.IsWindowVisible(handle));assert desktop.profile_window(directory/'data')==(app.pid,handle);receipt['secondLaunchReusedOwner']=True
    source=json.loads((directory/'source.json').read_text())['source']
    with sqlite3.connect((directory/'data/dolores.db').as_uri()+'?mode=ro',uri=True) as db:
        # Draft table names come from the normal store, not a speculative schema.
        receipt['savedResult']=db.execute("SELECT content FROM messages WHERE session_id=? AND role='assistant' ORDER BY id DESC LIMIT 1",(occurrence.get('session'),)).fetchone()
        tables=[r[0] for r in db.execute("SELECT name FROM sqlite_master WHERE type='table'")]
        receipt['draftPreserved']=any('RETAINED-HOME-DRAFT' in str(r) for table in tables if 'draft' in table for r in db.execute('SELECT * FROM '+table))
    assert receipt['draftPreserved']
    if a.resources:
        receipt['visibleIdle']=idle_sample(app.pid)
        user.PostMessageW(handle,0x0010,0,0);wait(lambda:not user.IsWindowVisible(handle))
        receipt['trayIdle']=idle_sample(app.pid)
        added_cpu=receipt['trayIdle']['cpuPercentOneCore']-receipt['visibleIdle']['cpuPercentOneCore']
        added_memory=receipt['trayIdle']['privateBytesPeak']-receipt['visibleIdle']['privateBytesPeak']
        receipt['resourceGate']={'addedCpuPercentagePoints':added_cpu,'addedPrivateBytes':added_memory,'passed':added_cpu<1 and added_memory<=16*1024*1024 and receipt['trayIdle']['additionalProcesses']==0}
        assert receipt['resourceGate']['passed'],receipt['resourceGate']
    if mode=='normal':
        assert receipt['savedResult'] and 'CLOSED-UI-REPORT' in receipt['savedResult'][0]
        user.PostMessageW(handle,0x8000+75,0,0);receipt['quitExitCode']=app.wait(timeout=15);assert receipt['quitExitCode']==0,receipt;receipt['gracefulQuit']=True
    else:
        # Deliberate owned crash fixture: interrupted approval/effects never replay.
        identity=desktop.process_identity(app.pid);desktop.process_identity(app.pid,terminate_expected=identity);app.wait(timeout=8);receipt['ownedCrashFixture']=True
        app=launch();handle=desktop.window_visible(app.pid)
        if mode=='approval':
            wait(lambda:read()[0]['state']=='interrupted',25)
            assert (directory/'project/public-note.txt').read_text()=='Public approved-only note.'
            receipt['uncertainEffectNotReplayed']=True
        assert len(read())==1,'Restart duplicated the occurrence'
        receipt['restartPreservedOccurrence']=True
        user.PostMessageW(handle,0x8000+75,0,0);receipt['quitExitCode']=app.wait(timeout=15);assert receipt['quitExitCode']==0,receipt
    receipt['localFixtureRequests']=len(requests);receipt['mode']=mode;(directory/'receipt.json').write_text(json.dumps(receipt,indent=2));print(json.dumps(receipt))
finally:
    if receipt:(directory/'partial-receipt.json').write_text(json.dumps(receipt,indent=2))
    if app and app.poll() is None:
        identity=desktop.process_identity(app.pid);desktop.process_identity(app.pid,terminate_expected=identity);app.wait(timeout=8)
    if a.live_source:
        cleaned=subprocess.run([sys.executable,__file__,'--directory',str(directory),'--forget-live'],capture_output=True,text=True,encoding='utf-8')
        assert cleaned.returncode==0,'Isolated credential cleanup failed'
        assert digest(a.live_source)==original_before,'Original profile changed'
        receipt['isolatedCredentialRemoved']=True;receipt['originalProfileUnchanged']=True
        (directory/'partial-receipt.json').write_text(json.dumps(receipt,indent=2))
        if (directory/'receipt.json').exists():(directory/'receipt.json').write_text(json.dumps(receipt,indent=2))
    server.shutdown();server.server_close()
