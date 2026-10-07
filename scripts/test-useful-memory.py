"""Public synthetic C ABI corpus; no credentials or live provider requests."""
import argparse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import threading
import time
from desktop_test_support import NativeHost, ROOT

parser=argparse.ArgumentParser()
parser.add_argument('--directory',type=Path,required=True)
args=parser.parse_args()
directory=args.directory.resolve()
assert directory.is_relative_to(ROOT/'output')
directory.mkdir(exist_ok=False,parents=True)
corpus=json.loads((ROOT/'docs/fixtures/memory-corpus.json').read_text(encoding='utf-8'))
titles=['Fact: project codename','Decision: local task index','Outcome: release migration','Open work: recovery flow','Response style','Decision: 本地任务索引']
requests=[]
mode='valid'
class Provider(BaseHTTPRequestHandler):
    def log_message(self,*_):pass
    def do_POST(self):
        request=json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        requests.append(request)
        if request['messages'][0]['content'].startswith('You extract automatic useful memory'):
            source=json.loads(request['messages'][1]['content'])['sources'][0]
            text=source['text']
            title=next((titles[n] for n,item in enumerate(corpus['eligible']) if item['text']==text),'Fact: project codename')
            answer=json.dumps({'suggestions':[{'title':title,'text':text,'quote':text,'messageId':source['messageId']}]},ensure_ascii=False)
            if mode=='malformed':answer='invalid'
        else:answer='Acknowledged.'
        self.send_response(200);self.send_header('Content-Type','text/event-stream');self.end_headers()
        events=[{'choices':[{'delta':{'content':answer},'finish_reason':None}]},{'choices':[{'delta':{},'finish_reason':'stop'}]},{'choices':[],'usage':{'prompt_tokens':100,'completion_tokens':25,'total_tokens':125}}]
        self.wfile.write((''.join('data: '+json.dumps(e,ensure_ascii=False)+'\n\n' for e in events)+'data: [DONE]\n\n').encode())

host=NativeHost(directory)
server=ThreadingHTTPServer(('127.0.0.1',0),Provider)
threading.Thread(target=server.serve_forever,daemon=True).start()
number=0
def run(session,text):
    global number
    number+=1
    host.call('start',id=number,session=session,input=text)
    done,_=host.finish(number)
    assert not done.get('error'),done
    if (done.get('memoryUpdate') or {}).get('status')=='queued':
        deadline=time.monotonic()+12
        while time.monotonic()<deadline:
            activity=host.call('memories',session=session)['automaticAttempt']
            if activity and activity['status']!='updating':return activity
            time.sleep(.01)
        raise AssertionError('Memory deadline')
    return done.get('memoryUpdate')

try:
    host.call('configure',preferences={'baseUrl':f'http://127.0.0.1:{server.server_port}/v1','model':'fixture'},apiKey='',rememberConnection=True)
    host.call('setAutomaticMemory',enabled=True,revision=1)
    projects=[]
    for name in ['A','B']:
        path=directory/name;path.mkdir()
        projects.append(str(path))
    a=host.call('createSession',kind='project',path=projects[0])['session']['id']
    next_a=host.call('createSession',kind='project',path=projects[0])['session']['id']
    b=host.call('createSession',kind='project',path=projects[1])['session']['id']
    for item in corpus['eligible']:
        assert run(a,item['text'])['saved']==1
    records=host.call('memories',session=a)['items']
    for n,item in enumerate(corpus['eligible']):
        record=next(r for r in records if r['text']==item['text'])
        context=host.call('context',session=next_a,input=item['cue'])
        assert context['memory']['used'][0]['id']==record['id'],item['cue']
        evidence=host.call('memoryEvidence',session=next_a,id=record['id'],revision=record['revision'])
        assert evidence['text']==item['text']
    assert run(b,corpus['otherProject'])['saved']==1
    context=host.call('context',session=b,input=corpus['eligible'][0]['cue'])
    assert 'Maple' in context['messages'][0]['content'] and 'Cedar' not in json.dumps(context)
    for text in corpus['negative']:
        assert run(a,text)['status']=='skipped'
    assert len(host.call('memories',session=a)['items'])==6
    mode='malformed'
    assert run(a,'Our project codename is Fir.')['status']=='failed'
    assert len(host.call('memories',session=a)['items'])==6
    host.call('delete',session=a)
    unavailable=host.call('memories',session=next_a)['items']
    assert all(not r['originAvailable'] for r in unavailable)
    assert not host.call('context',session=next_a,input=corpus['eligible'][0]['cue'])['memoryEntries']
    receipt={'ok':True,'fixtureRequests':len(requests),'liveRequests':0,'captures':6,'cueHits':6,'negativeCaptures':0,'scopeLeaks':0,'missingSourcesExcluded':6}
    (directory/'receipt.json').write_text(json.dumps(receipt,indent=2),encoding='utf-8')
    print(json.dumps(receipt))
finally:
    host.close();server.shutdown();server.server_close()
