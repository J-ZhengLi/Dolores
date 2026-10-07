"""Bounded public memory cases using configured models in a history-free profile.

Copies configuration only, never private conversations or credential values.
Reads the configured OS credential into memory for this launch only; does not
modify the original profile or vault.
"""
import argparse
import hashlib
import json
from pathlib import Path
import sqlite3
import subprocess
import sys
import time
import threading
import urllib.request
import urllib.error
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from desktop_test_support import NativeHost, ROOT
from memory_test_support import public_image

CONFIG_TABLES=('preferences','model_choices','model_contexts','request_settings','model_request_settings','model_images')
parser=argparse.ArgumentParser()
parser.add_argument('--directory',type=Path,required=True)
parser.add_argument('--source',type=Path)
parser.add_argument('--initialize',action='store_true')
parser.add_argument('--trace-public-extraction',action='store_true')
parser.add_argument('--image-only',action='store_true')
args=parser.parse_args();directory=args.directory.resolve()
assert directory.is_relative_to(ROOT/'output')
if args.initialize:
    host=NativeHost(directory);host.call('bootstrap');host.close();sys.exit(0)
directory.mkdir(parents=True,exist_ok=False)
subprocess.run([sys.executable,__file__,'--directory',str(directory),'--initialize'],check=True,capture_output=True)
original=args.source.resolve()
source=sqlite3.connect(original.as_uri()+'?mode=ro',uri=True)
def digest_tables(db):
    names=[r[0] for r in db.execute("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")]
    return {name:hashlib.sha256(repr(sorted(db.execute('SELECT * FROM '+name).fetchall(),key=repr)).encode()).hexdigest() for name in names}
before=digest_tables(source)
with sqlite3.connect(directory/'data/dolores.db') as target:
    for table in CONFIG_TABLES:
        rows=source.execute('SELECT * FROM '+table).fetchall()
        target.execute('DELETE FROM '+table)
        if rows:target.executemany('INSERT INTO '+table+' VALUES('+','.join('?' for _ in rows[0])+')',rows)
source.close()
host=NativeHost(directory);number=0;cases=[];usage=[];capture_usage=[]
traces=[];proxy=None
def chat(project=None):
    return host.call('createSession',kind='project' if project else 'side',**({'path':str(project)} if project else {}))['session']['id']
def policy(enabled):
    current=host.call('memories')['automaticPolicy']
    host.call('setAutomaticMemory',enabled=enabled,revision=current['revision'])
def run(session,prompt):
    global number
    number+=1
    host.call('start',id=number,session=session,input=prompt)
    def reject(event):
        if event['type']=='toolApproval':host.call('cancel',id=number)
    done,_=host.finish(number,callback=reject,seconds=15)
    if done.get('error'):return {'error':done['error']}
    message=host.call('messagesPage',session=session)['items'][-1]
    metadata=message.get('metadata') or {}
    if metadata.get('usage'):usage.append(metadata['usage'])
    activity=None
    if (done.get('memoryUpdate') or {}).get('status')=='queued':
        deadline=time.monotonic()+22
        while time.monotonic()<deadline:
            activity=host.call('memories',session=session)['automaticAttempt']
            if activity and activity['status']!='updating':break
            time.sleep(.03)
        if activity and activity.get('usage'):capture_usage.append(activity['usage'])
    return {'answer':message['content'][:500],'memory':activity}
def select(model):
    host.call('selectModel',model=model)
    # Only the disposable profile is changed. Lower limits are never imposed on
    # the original profile, and no model is silently substituted.
    with sqlite3.connect(directory/'data/dolores.db') as db:base=db.execute('SELECT base_url FROM preferences').fetchone()[0]
    host.call('setModelRequestSettings',preferences={'baseUrl':base,'model':model},settings={'maxOutputTokens':512,'timeoutSeconds':10,'reasoning':'deepseekThinkingOff' if model.startswith('deepseek') else 'providerDefault'})

try:
    host.call('bootstrap')
    import importlib.util
    credentials_spec=importlib.util.spec_from_file_location('configured_connection',ROOT/'scripts/test-workspace-editor.py')
    credentials_module=importlib.util.module_from_spec(credentials_spec);credentials_spec.loader.exec_module(credentials_module)
    base,key=credentials_module.connection(original.parent)
    with sqlite3.connect(directory/'data/dolores.db') as db:
        selected=db.execute('SELECT model FROM preferences').fetchone()[0]
        models=json.loads(db.execute('SELECT models FROM model_choices').fetchone()[0])
    if args.trace_public_extraction:
        remote_base=base;remote_key=key
        class PublicProbe(BaseHTTPRequestHandler):
            def log_message(self,*_):pass
            def do_POST(self):
                raw=self.rfile.read(int(self.headers['Content-Length']));payload=json.loads(raw)
                system=payload['messages'][0]['content'];extracting=system.startswith(('You extract automatic useful memory','Describe this explicitly shared image'))
                request=urllib.request.Request(remote_base.rstrip('/')+'/chat/completions',data=raw,headers={'Authorization':'Bearer '+remote_key,'Content-Type':'application/json'})
                answer='';reported=None
                try:
                    with urllib.request.urlopen(request,timeout=10) as response:
                        self.send_response(response.status);self.send_header('Content-Type','text/event-stream');self.end_headers()
                        for line in response:
                            self.wfile.write(line);self.wfile.flush()
                            if extracting and line.startswith(b'data: ') and line[6:].strip()!=b'[DONE]':
                                value=json.loads(line[6:]);reported=value.get('usage') or reported
                                for choice in value.get('choices',[]):answer+=(choice.get('delta') or {}).get('content') or ''
                except Exception as failure:
                    if extracting:answer='Transport diagnostic: '+type(failure).__name__
                finally:
                    if extracting:traces.append({'model':payload['model'],'kind':'image' if system.startswith('Describe') else 'text','publicResponse':answer[:8192],'reportedUsage':reported})
        with sqlite3.connect(directory/'data/dolores.db') as db:images=json.loads(db.execute('SELECT data FROM model_images WHERE base_url=?',(base,)).fetchone()[0])
        proxy=ThreadingHTTPServer(('127.0.0.1',0),PublicProbe);threading.Thread(target=proxy.serve_forever,daemon=True).start()
        base=f'http://127.0.0.1:{proxy.server_port}/v1'
    host.call('configure',preferences={'baseUrl':base,'model':selected},apiKey=key,rememberConnection=False,enabledModels=models)
    if proxy:host.call('setImageModels',models=images)
    del key
    # Set per-model bounded defaults through the supported connection command.
    if not args.image_only:
        select('Qwen/Qwen3.5-2B');policy(False)
        baseline=run(chat(),"What is our demo project codename? If unavailable, say you don't know.")
        cases.append({'case':'Qwen Memory Off baseline','result':baseline,'hit':'Cedar-742' in baseline.get('answer','')})
        policy(True);origin=chat()
        capture=run(origin,'Our demo project codename is Cedar-742. Acknowledge in one short sentence.')
        recall=run(chat(),"What is our demo project codename? If unavailable, say you don't know.")
        cases.append({'case':'Qwen useful fact and cross-chat recall','capture':capture,'result':recall,'hit':(capture.get('memory') or {}).get('saved',0)>0 and 'Cedar-742' in recall.get('answer','')})
        select('deepseek-v4.1-flash')
        projects=[]
        for name in ('A','B'):
            p=directory/name;p.mkdir();projects.append(p)
        a=chat(projects[0]);b=chat(projects[1])
        ac=run(a,'Our demo project codename is Aspen-318. Acknowledge briefly; no tools.')
        bc=run(b,'Our demo project codename is Maple-627. Acknowledge briefly; no tools.')
        correction=run(a,'Actually, our demo project codename is Birch-914 instead. Acknowledge briefly; no tools.')
        ar=run(chat(projects[0]),'What is our demo project codename? Give only the codename; no tools.')
        br=run(chat(projects[1]),'What is our demo project codename? Give only the codename; no tools.')
        cases.append({'case':'DeepSeek correction and project isolation','captures':[ac,bc,correction],'results':[ar,br],'hit':ar.get('answer','').strip('`* .\n')=='Birch-914' and br.get('answer','').strip('`* .\n')=='Maple-627','scopeLeak':'Maple-627' in ar.get('answer','') or 'Birch-914' in br.get('answer','')})
    else:
        select('deepseek-v4.1-flash');policy(True)
    # Public synthetic asset, explicitly supplied in the disposable conversation.
    image=directory/'blue-square.png';public_image(image)
    visual=chat();host.call('attachFile',session=visual,path=str(image))
    ic=run(visual,'What simple shape and colors are visible in this shared image? Be brief.')
    ir=run(chat(),'Which shared image showed a blue square on white? Describe it briefly and preserve uncertainty.')
    cases.append({'case':'DeepSeek supplied image and later recall','capture':ic,'result':ir,'hit':(ic.get('memory') or {}).get('saved',0)>0 and 'blue' in ir.get('answer','').lower() and 'square' in ir.get('answer','').lower() and not any(v in ir.get('answer','').lower() for v in ("don't have",'cannot confirm',"can't confirm"))})
finally:
    host.close()
    if proxy:proxy.shutdown();proxy.server_close()
after_db=sqlite3.connect(original.as_uri()+'?mode=ro',uri=True);after=digest_tables(after_db);after_db.close()
assert before==after,'Original profile changed during isolated qualification'
def totals(values):
    return {field:sum(v[field] for v in values) if values and all(v.get(field) is not None for v in values) else None for field in ('inputTokens','outputTokens','totalTokens')}
receipt={'cases':cases,'foregroundReportedUsage':totals(usage),'maintenanceReportedUsage':totals(capture_usage),'foregroundTurns':number,'originalProfileUnchanged':True,'configurationOnlyCopy':True,'generalReliabilityClaim':False}
if traces:receipt['publicExtractionDiagnostics']=traces
(directory/'receipt.json').write_text(json.dumps(receipt,indent=2,ensure_ascii=False),encoding='utf-8')
print(json.dumps(receipt,ensure_ascii=False))
