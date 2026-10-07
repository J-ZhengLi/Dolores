"""Maximum retained scheduler state with public synthetic records; no provider work."""
import argparse
import copy
import json
from pathlib import Path
import sqlite3
import subprocess
import sys
import time
from desktop_test_support import NativeHost, ROOT

p=argparse.ArgumentParser();p.add_argument('--directory',type=Path,required=True);p.add_argument('--source',type=Path);p.add_argument('--initialize',action='store_true');a=p.parse_args()
directory=a.directory.resolve();assert directory.is_relative_to(ROOT/'output')
if a.initialize:
    h=NativeHost(directory);h.call('bootstrap');h.close();sys.exit(0)
directory.mkdir(parents=True,exist_ok=False)
subprocess.run([sys.executable,__file__,'--directory',str(directory),'--initialize'],check=True,capture_output=True)
source=a.source.resolve();assert source.is_relative_to(ROOT/'output')
with sqlite3.connect((source/'data/dolores.db').as_uri()+'?mode=ro',uri=True) as db:
    template=json.loads(db.execute('SELECT data FROM scheduled_tasks LIMIT 1').fetchone()[0])
template['paused']=True;template['nextDue']=None;template['prompt']='P'*4096;template['title']='T'*160
template['skill']['versions']=[template['skill']['versions'][-1]]
template['skill']['versions'][0]['document']['text']='S'*8192
with sqlite3.connect(directory/'data/dolores.db') as db:
    for i in range(32):
        task=copy.deepcopy(template);task['id']=f'public-{i}';task['sourceKey']=task['id']
        db.execute('INSERT INTO scheduled_tasks VALUES(?,?,?)',(task['id'],task['sourceKey'],json.dumps(task)))
        for j in range(50):
            occurrence={'id':f'{i}:{j}','task':task['id'],'due':1000+j,'state':'succeeded','session':None,'run':None,'leaseUntil':0,'snapshot':task,'error':None}
            db.execute('INSERT INTO scheduled_occurrences VALUES(?,?,?,?)',(occurrence['id'],task['id'],occurrence['due'],json.dumps(occurrence)))
    stored=db.execute('SELECT SUM(length(data)) FROM scheduled_occurrences').fetchone()[0]+db.execute('SELECT SUM(length(data)) FROM scheduled_tasks').fetchone()[0]
host=NativeHost(directory)
try:
    host.call('bootstrap');ticks=[];lists=[]
    for _ in range(10):
        start=time.perf_counter();result=host.call('scheduledTick');ticks.append((time.perf_counter()-start)*1000);assert not result['claimed']
        start=time.perf_counter();listing=host.call('scheduledTasks');lists.append((time.perf_counter()-start)*1000)
    assert len(listing['items'])==32 and all(len(i['occurrences'])==50 for i in listing['items'])
    assert 'snapshot' not in json.dumps(listing)
    receipt={'syntheticMaximumRetention':True,'tasks':32,'occurrences':1600,'storedJsonBytes':stored,'listBytes':len(json.dumps(listing).encode()),'tickMaxMs':round(max(ticks),3),'listMaxMs':round(max(lists),3),'providerRequests':0,'idleCpuAcceptance':False}
    (directory/'receipt.json').write_text(json.dumps(receipt,indent=2));print(json.dumps(receipt))
finally:host.close()
