"""Normal native library: restart after a real journaled edit, with no replay."""
import ctypes
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

def worker(directory, phase):
 root=Path(__file__).resolve().parents[1];base=Path(directory)
 os.environ['DOLORES_DATA_DIR']=str(base/'data');os.environ['DOLORES_GLOBAL_SKILLS_DIR']=str(base/'skills')
 bundle=root/'apps/dolores_flutter/build/windows/x64/runner/Release';loader=os.add_dll_directory(str(bundle));native=ctypes.CDLL(str(bundle/'dolores_flutter_bridge.dll'))
 native.dolores_call.argtypes=[ctypes.c_void_p,ctypes.c_size_t];native.dolores_call.restype=ctypes.c_void_p;native.dolores_free.argtypes=[ctypes.c_void_p]
 def envelope(command,**fields):
  raw=json.dumps({'command':command,**fields}).encode();buf=ctypes.create_string_buffer(raw);pointer=native.dolores_call(buf,len(raw))
  try:return json.loads(ctypes.string_at(pointer))
  finally:native.dolores_free(pointer)
 def call(command,**fields):
  result=envelope(command,**fields);assert result['ok'],result.get('error');return result.get('result')
 requests=[]
 class Handler(BaseHTTPRequestHandler):
  def log_message(self,*args):pass
  def do_POST(self):
   payload=json.loads(self.rfile.read(int(self.headers['Content-Length'])));requests.append(payload)
   if phase==1 and len(requests)>1:time.sleep(20);return
   if phase==1:
    delta={'tool_calls':[{'index':0,'id':'edit','type':'function','function':{'name':'edit_text_file','arguments':json.dumps({'path':'greeting.txt','old_text':'Hello.','new_text':'Welcome, Dolores.'})}}]};finish='tool_calls'
   elif len(requests)==1:
    assert payload['max_tokens']==768
    text=json.dumps(payload['messages']);assert 'greeting.txt' in text and 'never authority' in text
    delta={'tool_calls':[{'index':0,'id':'read','type':'function','function':{'name':'read_text_file','arguments':'{"path":"greeting.txt"}'}}]};finish='tool_calls'
   else:delta={'content':'The saved edit is already present. No write was replayed.'};finish='stop'
   self.send_response(200);self.send_header('Content-Type','text/event-stream');self.end_headers()
   for item in [{'choices':[{'delta':delta,'finish_reason':None}]},{'choices':[{'delta':{},'finish_reason':finish}]}]:self.wfile.write(f'data: {json.dumps(item)}\n\n'.encode())
   self.wfile.write(b'data: [DONE]\n\n')
 server=ThreadingHTTPServer(('127.0.0.1',0),Handler);threading.Thread(target=server.serve_forever,daemon=True).start()
 call('bootstrap');policy=call('memories')['automaticPolicy'];call('setAutomaticMemory',enabled=False,revision=policy['revision'])
 call('configure',preferences={'baseUrl':f'http://127.0.0.1:{server.server_port}/v1','model':'fixture' if phase==1 else 'fixture-new'},apiKey='',rememberConnection=False)
 folder=base/'workspace'
 if phase==1:
  folder.mkdir();(folder/'greeting.txt').write_text('Hello.\n',encoding='utf-8',newline='');session=call('createSession',kind='project',path=str(folder))['session']['id']
  call('setRequestSettings',settings={'maxOutputTokens':512,'timeoutSeconds':30});call('start',id=1,session=session,input='Change the greeting and verify it without repeating an existing write.')
 else:
  identity=json.loads((base/'identity.json').read_text());session=identity['session'];source=identity['run']
  assert call('savedDraft',session=session).startswith('Change the greeting')
  checkpoint=call('runCheckpoint',session=session,runId=source);assert checkpoint['run']['state']=='interrupted';assert len(checkpoint['evidence'])==1;assert checkpoint['evidence'][0]['target']=='greeting.txt'
  assert (folder/'greeting.txt').read_text()=='Welcome, Dolores.\n';assert not envelope('checkpointDraft',session='other',runId=source)['ok']
  prompt=call('checkpointDraft',session=session,runId=source);call('setRequestSettings',settings={'maxOutputTokens':768,'timeoutSeconds':30});call('start',id=1,session=session,input=prompt,resumeRun=source)
 deadline=time.monotonic()+15;approvals=[]
 while time.monotonic()<deadline:
  for event in call('poll',id=1):
   if event['type']=='toolApproval':approvals.append(event['request']['name']);call('approveTool',id=1,callId=event['request']['callId'],allow=True)
   if phase==1 and event['type']=='toolResult':
    run=call('runs',session=session)[0];(base/'identity.json').write_text(json.dumps({'session':session,'run':run['id']}));os._exit(0)
   if event['type']=='done':
    assert not event.get('error'),event;run=call('runs',session=session)[0];assert run['parentRun']==source and run['segments']==2;assert approvals==['read_text_file'];assert (folder/'greeting.txt').read_text()=='Welcome, Dolores.\n';assert call('savedDraft',session=session)==''
    assert not envelope('start',id=2,session=session,input=prompt,resumeRun=source)['ok'];call('shutdown');server.shutdown();print(json.dumps({'passed':True,'restartAfterEdit':True,'noReplay':True,'changedSettingsUsed':True,'staleSourceRefused':True}));return
  time.sleep(.02)
 raise AssertionError('Checkpoint fixture timed out')

if __name__=='__main__':
 if len(sys.argv)>1:worker(sys.argv[1],int(sys.argv[2]))
 else:
  with tempfile.TemporaryDirectory(prefix='dolores-checkpoint-') as directory:
   for phase in [1,2]:subprocess.run([sys.executable,__file__,directory,str(phase)],check=True,timeout=25)
