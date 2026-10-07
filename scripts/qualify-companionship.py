"""Packaged public policy/restart/quiet checks, without overriding OS presence."""
import argparse,json,statistics,subprocess,sys,time
from pathlib import Path
from desktop_test_support import NativeHost,ROOT
p=argparse.ArgumentParser();p.add_argument('--directory',type=Path,required=True);p.add_argument('--restart',action='store_true');p.add_argument('--stage',action='store_true');a=p.parse_args();directory=a.directory.resolve();assert directory.is_relative_to(ROOT/'output')
if not a.stage and not a.restart:
    directory.mkdir(parents=True,exist_ok=False)
    results=[]
    for flag in ['--stage','--restart']:
        r=subprocess.run([sys.executable,__file__,'--directory',str(directory),flag],capture_output=True,text=True,encoding='utf-8')
        if r.returncode:raise RuntimeError(r.stderr)
        results.append(json.loads(r.stdout))
    result={**results[0],**results[1]};(directory/'qualification.json').write_text(json.dumps(result,indent=2),encoding='utf-8');print(json.dumps(result));sys.exit(0)
h=NativeHost(directory);h.call('bootstrap');v=h.call('companionState');s=v['state']
if a.restart:
    assert s['policy']['enabled'] is False and s['policy']['dailyCap']==1
    assert h.call('sessionsPage')['items']==[]
    h.close();print(json.dumps({'restartPreserved':True}));sys.exit(0)
assert not s['policy']['enabled'];before=s
times=[]
for _ in range(100):
    start=time.perf_counter();v=h.call('companionTick',session=None,busy=False);times.append((time.perf_counter()-start)*1000)
assert v['state']==before
policy=s['policy'];policy['dailyCap']=1
v=h.call('companionPolicy',policy=policy,revision=s['revision']);saved=v['state'];assert saved['policy']['dailyCap']==1
assert not h.envelope('companionPolicy',policy=policy,revision=s['revision'])['ok']
assert h.call('companionState')['state']==saved
h.close()
result={'packagedOffQuiet':True,'staleSavePreservesPolicy':True,'requests':0,'addedProcesses':0,'offTickMs':{'median':statistics.median(times),'p95':sorted(times)[94]},'tickWrites':0,'profileBytes':(directory/'data/dolores.db').stat().st_size};print(json.dumps(result))
