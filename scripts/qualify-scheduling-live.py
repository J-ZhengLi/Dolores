"""Bounded public scheduling cases in a configuration-only disposable profile.

No original history is copied; the configured vault key stays in memory. Task
execution uses an explicit Run now, not a claim of a physical clock firing.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import sqlite3
import subprocess
import sys
from desktop_test_support import NativeHost, ROOT

parser=argparse.ArgumentParser()
parser.add_argument('--directory',type=Path,required=True)
parser.add_argument('--source',type=Path)
parser.add_argument('--initialize',action='store_true')
parser.add_argument('--model',help='Enabled primary model ID for this optional live check.')
parser.add_argument('--secondary-model',help='Optional enabled model for harder cases; defaults to --model.')
parser.add_argument('--reasoning',choices=['providerDefault','deepseekThinkingOff','openaiLow','openaiMedium','openaiHigh'],default='providerDefault')
args=parser.parse_args();directory=args.directory.resolve()
assert directory.is_relative_to(ROOT/'output')
if args.initialize:
    host=NativeHost(directory);host.call('bootstrap');host.close();sys.exit(0)
if not args.source or not args.model:parser.error('--source and --model are required for live checks')
directory.mkdir(parents=True,exist_ok=False)
subprocess.run([sys.executable,__file__,'--directory',str(directory),'--initialize'],check=True,capture_output=True)
original=args.source.resolve()
def digest(db):
    names=[r[0] for r in db.execute("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")]
    return {n:hashlib.sha256(repr(sorted(db.execute('SELECT * FROM '+n).fetchall(),key=repr)).encode()).hexdigest() for n in names}
with sqlite3.connect(original.as_uri()+'?mode=ro',uri=True) as source:
    before=digest(source)
    with sqlite3.connect(directory/'data/dolores.db') as target:
        for table in ('preferences','model_choices','model_contexts','request_settings','model_request_settings','model_images'):
            rows=source.execute('SELECT * FROM '+table).fetchall();target.execute('DELETE FROM '+table)
            if rows:target.executemany('INSERT INTO '+table+' VALUES('+','.join('?' for _ in rows[0])+')',rows)
host=NativeHost(directory);number=0;cases=[];usage=[]
def run(session,prompt,occurrence=None):
    global number
    number+=1
    if occurrence:host.call('scheduledStart',occurrence=occurrence,id=number)
    else:host.call('start',id=number,session=session,input=prompt)
    def reject(event):
        if event['type']=='toolApproval':host.call('cancel',id=number)
    done,events=host.finish(number,callback=reject,seconds=35)
    session=next((e.get('session') for e in events if e.get('session')),session)
    # Scheduled Start returns the actual destination before polling.
    if occurrence:
        host.call('scheduledTick')
        session=next(o['session'] for item in host.call('scheduledTasks')['items'] for o in item['occurrences'] if o['id']==occurrence)
    messages=host.call('messagesPage',session=session)['items']
    answer=messages[-1] if not done.get('error') and messages and messages[-1]['role']=='assistant' else {}
    metadata=answer.get('metadata') or {}
    reported=metadata.get('usage')
    if reported:usage.append(reported)
    else:usage.extend(v for v in (metadata.get('agent') or {}).get('usage_by_call',[]) if v)
    return {'error':done.get('error'),'answer':answer.get('content','')[:1200]}
def select(model):
    host.call('selectModel',model=model)
    host.call('setModelRequestSettings',preferences={'baseUrl':base,'model':model},settings={'maxOutputTokens':1024,'timeoutSeconds':20,'reasoning':args.reasoning})
def tasks():return host.call('scheduledTasks')['items']
try:
    host.call('bootstrap')
    spec=importlib.util.spec_from_file_location('configured_connection',ROOT/'scripts/test-workspace-editor.py')
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    base,key=module.connection(original.parent)
    with sqlite3.connect(directory/'data/dolores.db') as db:
        selected=db.execute('SELECT model FROM preferences').fetchone()[0]
        models=json.loads(db.execute('SELECT models FROM model_choices').fetchone()[0])
    host.call('configure',preferences={'baseUrl':base,'model':selected},apiKey=key,rememberConnection=False,enabledModels=models);del key
    policy=host.call('memories')['automaticPolicy'];host.call('setAutomaticMemory',enabled=False,revision=policy['revision'])
    project=directory/'public-project';skill=project/'.agents/skills/daily-report/SKILL.md';skill.parent.mkdir(parents=True)
    skill.write_text('---\nname: daily-report\ndescription: Write a short public daily report\n---\nUse no tools or file access for this public demonstration. Output PUBLIC-SCHEDULE-REPORT and then two short lines: Done: scheduling demo. Next: review tomorrow.\n',encoding='utf-8')
    source=host.call('createSession',kind='project',path=str(project))['session']['id']
    review=host.call('reviewSkill',session=source,name='daily-report');host.call('activateSkill',session=source,token=review['token'])
    select(args.model)
    created=run(source,'Every weekday at 9pm, write my daily report using skill daily-report')
    model_label=args.model
    items=tasks();cases.append({'case':model_label+' direct weekday task','result':created,'pass':len(items)==1 and items[0]['receipt']['schedule'].startswith('Mon, Tue, Wed, Thu, Fri at 21:00')})
    if items:
        task=items[0]['task'];occurrence=host.call('scheduledManage',task=task['id'],revision=task['revision'],action='runNow')['claimed'][0]
        report=run(source,'',occurrence['id'])
        cases.append({'case':model_label+' pinned skill Run now result','result':report,'pass':not report['error'] and 'PUBLIC-SCHEDULE-REPORT' in report['answer']})
    select(args.secondary_model or args.model)
    if not tasks() and args.secondary_model:
        created=run(source,'Every weekday at 9pm, write my daily report using skill daily-report with model '+args.model)
        cases.append({'case':'Selected model direct creation with explicit Primary model execution model','result':created,'pass':len(tasks())==1})
        if tasks():
            task=tasks()[0]['task'];occurrence=host.call('scheduledManage',task=task['id'],revision=task['revision'],action='runNow')['claimed'][0]
            report=run(source,'',occurrence['id'])
            cases.append({'case':'Primary model pinned skill Run now after explicit Selected model creation','result':report,'pass':not report['error'] and 'PUBLIC-SCHEDULE-REPORT' in report['answer']})
    if tasks():
        changed=run(source,'Change it to 10pm')
        cases.append({'case':'Selected model conversational time change','result':changed,'pass':'22:00' in tasks()[0]['receipt']['schedule']})
    count=len(tasks())
    quoted=run(source,'Explain this quoted example: "Every weekday at 9pm, write my daily report using skill daily-report". Do not create any task.')
    cases.append({'case':'Selected model quoted example creates no task','result':quoted,'pass':len(tasks())==count})
finally:host.close()
with sqlite3.connect(original.as_uri()+'?mode=ro',uri=True) as db:assert before==digest(db),'Original profile changed'
receipt={'cases':cases,'turns':number,'reportedUsage':{field:sum(v[field] for v in usage) if usage and all(v.get(field) is not None for v in usage) else None for field in ('inputTokens','outputTokens','totalTokens')},'originalProfileUnchanged':True,'configurationOnlyCopy':True,'generalReliabilityClaim':False}
(directory/'receipt.json').write_text(json.dumps(receipt,indent=2),encoding='utf-8');print(json.dumps(receipt))
