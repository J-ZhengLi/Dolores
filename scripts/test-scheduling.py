"""Public, isolated normal-bundle C ABI scheduling corpus; no live credentials.

Run save then reopen in separate processes. The due-time injection is explicitly
a clock fixture; it is not evidence that a physical 21:00 desktop timer fired.
"""
import argparse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import sqlite3
import threading
import time
from desktop_test_support import NativeHost, ROOT

parser=argparse.ArgumentParser()
parser.add_argument('stage',choices=['save','reopen'])
parser.add_argument('--directory',type=Path,required=True)
args=parser.parse_args();directory=args.directory.resolve()
assert directory.is_relative_to(ROOT/'output')
if args.stage=='save':directory.mkdir(parents=True,exist_ok=False)
host=NativeHost(directory);state_file=directory/'state.json'
mode='normal';requests=[]
class Fixture(BaseHTTPRequestHandler):
    def log_message(self,*_):pass
    def do_POST(self):
        payload=json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        requests.append(payload)
        if mode=='offline':
            self.send_response(503);self.end_headers();return
        messages=payload['messages'];system=messages[0]['content']
        scheduled='This is an occurrence of the user' in system
        names=[t['function']['name'] for t in payload.get('tools',[])]
        first=messages[-1]['role']=='user'
        if scheduled and mode=='approval' and first:
            calls=[{'index':0,'id':'read','type':'function','function':{'name':'read_text_file','arguments':json.dumps({'path':'public-note.txt'})}}]
        elif not scheduled and first and mode!='changeSkill' and 'schedule_task' in names:
            call={'rule':{'kind':'weekdays','time':'21:00','date':None,'weekdays':[0,1,2,3,4],'zone':'Asia/Shanghai'},'skill':'daily-report','model':None}
            calls=[{'index':i,'id':f'create-{i}','type':'function','function':{'name':'schedule_task','arguments':json.dumps(call)}} for i in range(2)]
        elif not scheduled and first and mode=='changeSkill' and 'manage_scheduled_task' in names:
            calls=[{'index':0,'id':'update-skill','type':'function','function':{'name':'manage_scheduled_task','arguments':json.dumps({'task':changed_task['id'],'revision':changed_task['revision'],'action':'change','time':None,'model':None,'skill':'daily-report'})}}]
        else:calls=[]
        if scheduled:
            assert 'DAILY-REPORT-PINNED-V1' in system
            assert 'schedule_task' not in names and 'manage_scheduled_task' not in names
        elif first and 'schedule_task' in names:
            # Scheduling and ordinary work tools coexist; future occurrences
            # still exclude scheduling tools and use their pinned skill snapshot.
            assert 'manage_scheduled_task' in names and 'read_text_file' in names,names
            assert len(names)<=17,names
        delta={'tool_calls':calls} if calls else {'content':'DAILY-REPORT-PINNED-V1\nDone: public fixture report.\nNext: review tomorrow.' if scheduled else 'Task saved. Results appear in Scheduled.'}
        events=[{'choices':[{'delta':delta,'finish_reason':None}]},{'choices':[{'delta':{},'finish_reason':'tool_calls' if calls else 'stop'}]},{'choices':[],'usage':{'prompt_tokens':120,'completion_tokens':30,'total_tokens':150}}]
        self.send_response(200);self.send_header('Content-Type','text/event-stream');self.end_headers()
        self.wfile.write((''.join('data: '+json.dumps(e)+'\n\n' for e in events)+'data: [DONE]\n\n').encode())

def listing():return host.call('scheduledTasks')['items'][0]
def manage(action):
    task=listing()['task'];return host.call('scheduledManage',task=task['id'],revision=task['revision'],action=action)
def assert_start_identity(start):
    assert start['workspace']==host.call('workspace',session=start['session'])
    assert start['model']==listing()['receipt']['model']
def run_now(number,callback=None):
    o=manage('runNow')['claimed'][0]
    start=host.call('scheduledStart',occurrence=o['id'],id=number)
    assert_start_identity(start)
    done,events=host.finish(number,callback=callback,seconds=15)
    host.call('scheduledTick')
    return done,events,start

try:
    host.call('bootstrap')
    if args.stage=='reopen':
        state=json.loads(state_file.read_text())
        item=listing()
        assert any(o['id']==state['unfinished'] and o['state']=='interrupted' for o in item['occurrences'])
        assert any(o['state']=='succeeded' and o['session']==state['result'] for o in item['occurrences'])
        assert host.call('messagesPage',session=state['result'])['items'][-1]['content'].startswith('DAILY-REPORT-PINNED-V1')
        assert host.call('scheduledTick')['claimed']==[]
        (directory/'reopen.json').write_text(json.dumps({'ok':True,'restartInterruptedWithoutReplay':True,'savedResultPreserved':True},indent=2))
        print(json.dumps({'ok':True,'stage':'reopen'}))
    else:
        server=ThreadingHTTPServer(('127.0.0.1',0),Fixture);threading.Thread(target=server.serve_forever,daemon=True).start()
        host.call('configure',preferences={'baseUrl':f'http://127.0.0.1:{server.server_port}/v1','model':'fixture'},apiKey='',rememberConnection=True)
        policy=host.call('memories')['automaticPolicy'];host.call('setAutomaticMemory',enabled=False,revision=policy['revision'])
        project=directory/'project';skill=project/'.agents/skills/daily-report/SKILL.md';skill.parent.mkdir(parents=True)
        skill.write_text('---\nname: daily-report\ndescription: Write a short public daily report\n---\nReport exactly: DAILY-REPORT-PINNED-V1\n',encoding='utf-8')
        (project/'public-note.txt').write_text('Public fixture note.',encoding='utf-8')
        source=host.call('createSession',kind='project',path=str(project))['session']['id']
        review=host.call('reviewSkill',session=source,name='daily-report');host.call('activateSkill',session=source,token=review['token'])
        host.call('start',id=1,session=source,input='Every weekday at 9pm, write my daily report using skill daily-report')
        done,_=host.finish(1);assert not done.get('error'),done
        assert len(host.call('scheduledTasks')['items'])==1
        task=listing()['task']
        with sqlite3.connect(directory/'data/dolores.db') as db:
            row=json.loads(db.execute('SELECT data FROM scheduled_tasks WHERE id=?',(task['id'],)).fetchone()[0]);row['nextDue']=int(time.time())
            db.execute('UPDATE scheduled_tasks SET data=? WHERE id=?',(json.dumps(row),task['id']))
        tick=host.call('scheduledTick');assert len(tick['claimed'])==1
        assert host.call('scheduledTick')['claimed']==[]
        first=tick['claimed'][0];start=host.call('scheduledStart',occurrence=first['id'],id=2)
        assert_start_identity(start)
        done,_=host.finish(2);assert not done.get('error'),done
        host.call('scheduledTick');assert listing()['occurrences'][0]['state']=='succeeded'
        assert 'DAILY-REPORT-PINNED-V1' in host.call('messagesPage',session=start['session'])['items'][-1]['content']
        mode='approval'
        approved=[]
        def approve(e):
            if e['type']=='toolApproval':
                assert e['request']['name']=='read_text_file';approved.append(e['request']['callId'])
                host.call('approveTool',id=3,callId=e['request']['callId'],allow=True)
        done,_,_=run_now(3,approve);assert not done.get('error') and len(approved)==1
        def stop(e):
            if e['type']=='toolApproval':
                assert not host.envelope('scheduledManage',task=listing()['task']['id'],revision=listing()['task']['revision'],action='runNow')['ok']
                host.call('cancel',id=4)
        done,_,_=run_now(4,stop);assert done.get('error') and listing()['occurrences'][0]['state']=='cancelled'
        mode='offline';done,_,_=run_now(5);assert done.get('error') and listing()['occurrences'][0]['state']=='failed'
        mode='normal';manage('pause');assert listing()['task']['paused']
        manage('skip');assert any(o['state']=='skipped' for o in listing()['occurrences'])
        # Run now is deliberate even when recurrence is paused.
        done,_,_=run_now(6);assert not done.get('error')
        host.call('disableSkill',session=source,name='daily-report',revision=1)
        disabled=manage('runNow')['claimed'][0]
        denied=host.envelope('scheduledStart',occurrence=disabled['id'],id=7)
        assert not denied['ok'] and 'disabled' in denied['error'] and listing()['task']['paused']
        review=host.call('reviewSkill',session=source,name='daily-report');host.call('activateSkill',session=source,token=review['token'])
        changed_task=listing()['task'];mode='changeSkill'
        host.call('start',session=source,id=8,input='Update this scheduled task to use skill daily-report')
        done,_=host.finish(8);assert not done.get('error'),done
        mode='normal';done,_,_=run_now(9);assert not done.get('error'),done
        task=listing()['task'];assert not host.envelope('scheduledManage',task=task['id'],revision=1,action='resume')['ok']
        orphan=manage('runNow')['claimed'][0]
        samples=[]
        for _ in range(10):
            before=time.perf_counter();host.call('scheduledTick');samples.append((time.perf_counter()-before)*1000)
        summary=host.call('scheduledTasks');assert 'snapshot' not in json.dumps(summary) and 'SKILL.md' not in json.dumps(summary)
        state_file.write_text(json.dumps({'unfinished':orphan['id'],'result':start['session']}))
        report={'ok':True,'evidence':'normal packaged native C ABI with public provider and injected due clock','cases':['creation duplicate guard','due claim/report/result','reviewed file read','Stop while waiting','overlap refusal','offline recovery','pause/skip/run now/stale revision','disabled skill refusal and conversational version recovery','restart claim prepared'], 'requests':len(requests),'tickMaxMs':round(max(samples),3),'summaryBytes':len(json.dumps(summary).encode())}
        (directory/'save.json').write_text(json.dumps(report,indent=2));print(json.dumps(report))
        server.shutdown()
finally:host.close()
