"""Local context cost in an existing public, disposable memory test profile."""
import argparse
import json
from pathlib import Path
import sqlite3
import statistics
import time
from desktop_test_support import NativeHost,ROOT

parser=argparse.ArgumentParser();parser.add_argument('--directory',type=Path,required=True)
args=parser.parse_args();directory=args.directory.resolve()
assert directory.is_relative_to(ROOT/'output')
assert json.loads((directory/'receipt.json').read_text(encoding='utf-8'))['configurationOnlyCopy']
host=NativeHost(directory)
with sqlite3.connect(directory/'data/dolores.db') as db:
    session=db.execute('SELECT id FROM sessions ORDER BY updated_at DESC LIMIT 1').fetchone()[0]
    values=[r[0] for r in db.execute('SELECT data FROM memory_preferences')]
    payload=sum(len(v.encode()) for v in values)
    disk=db.execute('PRAGMA page_count').fetchone()[0]*db.execute('PRAGMA page_size').fetchone()[0]
policy=host.call('memories')['automaticPolicy'];original=policy['enabled']
results={}
try:
    for enabled in (False,True):
        policy=host.call('memories')['automaticPolicy'];host.call('setAutomaticMemory',enabled=enabled,revision=policy['revision'])
        elapsed=[]
        for _ in range(40):
            start=time.perf_counter();context=host.call('context',session=session,input='What is our demo project codename, and which shared image showed a blue square?')
            elapsed.append((time.perf_counter()-start)*1000)
        report=context.get('memory') or {}
        results['on' if enabled else 'off']={'samples':40,'p50Milliseconds':round(statistics.median(elapsed),3),'p95Milliseconds':round(sorted(elapsed)[37],3),'memoryTextBytes':report.get('textBytes',0),'usedEntries':len(report.get('used',[]))}
finally:
    policy=host.call('memories')['automaticPolicy'];host.call('setAutomaticMemory',enabled=original,revision=policy['revision']);host.close()
receipt={'diagnosticOnly':True,'liveRequests':0,'recordCount':len(values),'recordPayloadBytes':payload,'wholeFixtureDatabaseBytes':disk,'context':results,'scope':'one public corpus on this machine; no general performance claim'}
(directory/'cost.json').write_text(json.dumps(receipt,indent=2),encoding='utf-8');print(json.dumps(receipt))
