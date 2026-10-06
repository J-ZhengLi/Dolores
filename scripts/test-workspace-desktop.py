"""Synthetic native workspace captures, latency and owned-process resources.

Build with desktop.py build --diagnostic workspace before running. This does
not qualify physical keyboard, IME or accessibility. Use a fresh output profile.
"""
import sys,os,time,json,subprocess,uuid,math
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'scripts'))
from desktop_resource_probe import memory,cpu_seconds,children
directory=ROOT/'output/workspace-native-16'/uuid.uuid4().hex
directory.mkdir(parents=True)
env=os.environ.copy();env['DOLORES_WORKSPACE_SMOKE_DIR']=str(directory)
subprocess.run([sys.executable,str(ROOT/'scripts/desktop.py'),'launch','--data-directory',str(directory/'data'),'--pid-file',str(directory/'process.json'),'--allow-diagnostic'],env=env,check=True)
record=json.loads((directory/'process.json').read_text());print(record,flush=True)
pid=record['pid'];samples={};last=None;start=time.monotonic();report={}
try:
 while time.monotonic()-start<140:
  try: report=json.loads((directory/'stage.json').read_text())
  except (OSError,ValueError): time.sleep(.1);continue
  stage=report['stage'];m=memory(pid)
  if stage!=last: print(stage,flush=True);last=stage
  if m: samples.setdefault(stage,[]).append({'t':time.monotonic(),'cpu':cpu_seconds(pid),**m})
  if stage in ('complete','failed'): break
  time.sleep(.05)
 result={'directory':str(directory),'editor':report,'stages':{}}
 for stage,values in samples.items():
  result['stages'][stage]={'samples':len(values),'durationSeconds':values[-1]['t']-values[0]['t'],'cpuSeconds':values[-1]['cpu']-values[0]['cpu'],'last':{k:values[-1][k] for k in ('workingBytes','privateBytes')},'peak':{k:max(v[k] for v in values) for k in ('workingBytes','privateBytes')}}
 for name,key in [('typingP95Ms','typingMs'),('validationP95Ms','validationMs'),('mutationP95Ms','mutationMs'),('frameBuildP95Ms','frameBuildMs'),('frameRasterP95Ms','frameRasterMs'),('ordinaryOpenP95Ms','openMs')]:
  values=sorted(report.get(key,[]) if key!='openMs' else report.get(key,[])[-9:])
  if values:result[name]=values[math.ceil(.95*len(values))-1]
 result['helpersAtEnd']=len(children(pid,None))
 result['passed']=report.get('stage')=='complete' and result.get('typingP95Ms',float('inf'))<32 and result.get('ordinaryOpenP95Ms',float('inf'))<250
 baseline=result['stages'].get('home-idle',{}).get('last',{})
 for stage,limit in [('one-large',128),('two-large',192),('four-large',224),('four-distinct',224)]:
  values=result['stages'].get(stage)
  if not values or not baseline:result['passed']=False;continue
  values['incrementMiB']={k:round((values['peak'][k]-baseline[k])/1048576,3) for k in baseline}
  result['passed']=result['passed'] and max(values['incrementMiB'].values())<=limit
 (directory/'qualification.json').write_text(json.dumps(result,indent=2))
 print(json.dumps({k:v for k,v in result.items() if k!='editor'}),flush=True)
finally:
 subprocess.run([sys.executable,str(ROOT/'scripts/desktop.py'),'stop-owned','--pid-file',str(directory/'process.json')],check=True)
if not result['passed']:raise SystemExit(1)
