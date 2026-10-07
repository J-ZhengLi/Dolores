"""Public shared-image C ABI qualification. Local HTTP fixture only."""
import argparse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import struct
import threading
import time
import zlib
from desktop_test_support import NativeHost, ROOT

def public_image(path):
    def chunk(kind, data):
        return struct.pack('>I', len(data))+kind+data+struct.pack('>I', zlib.crc32(kind+data))
    rows=b''.join(b'\0'+b''.join(bytes((20,90,220)) if 16<=x<48 and 16<=y<48 else bytes((255,255,255)) for x in range(64)) for y in range(64))
    path.write_bytes(b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',64,64,8,2,0,0,0))+chunk(b'IDAT',zlib.compress(rows))+chunk(b'IEND',b''))

parser=argparse.ArgumentParser()
parser.add_argument('stage',choices=['save','restore'])
parser.add_argument('--directory',type=Path,required=True)
args=parser.parse_args();directory=args.directory.resolve()
assert directory.is_relative_to(ROOT/'output')
if args.stage=='save':directory.mkdir(parents=True,exist_ok=False)
host=NativeHost(directory);server=None;requests=[];mode='valid';number=0
class Provider(BaseHTTPRequestHandler):
    def log_message(self,*_):pass
    def do_POST(self):
        request=json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        requests.append(request)
        system=request['messages'][0]['content']
        if system.startswith('Describe this explicitly shared image'):
            assert any(p.get('type')=='image_url' for m in request['messages'] if isinstance(m['content'],list) for p in m['content'])
            answer=json.dumps({'title':'Image: blue square','description':'A blue square on a white background.','uncertainty':'Simple shape; other context is unknown.'}) if mode=='valid' else 'malformed'
        elif system.startswith('You extract automatic useful memory'):
            source=json.loads(request['messages'][1]['content'])['sources'][0]
            answer=json.dumps({'suggestions':[{'title':'Decision: local database','text':source['text'],'quote':source['text'],'messageId':source['messageId']}]})
        else:answer='The shared image shows a blue square on white.'
        self.send_response(200);self.send_header('Content-Type','text/event-stream');self.end_headers()
        chunks=[{'choices':[{'delta':{'content':answer},'finish_reason':'stop'}]},{'choices':[],'usage':{'prompt_tokens':100,'completion_tokens':25,'total_tokens':125}}]
        self.wfile.write((''.join('data: '+json.dumps(c)+'\n\n' for c in chunks)+'data: [DONE]\n\n').encode())

def run(session,text):
    global number
    number+=1;host.call('start',id=number,session=session,input=text)
    done,_=host.finish(number);assert not done.get('error'),done
    deadline=time.monotonic()+22
    while time.monotonic()<deadline:
        value=host.call('memories',session=session)['automaticAttempt']
        if value and value['status']!='updating':return value
        time.sleep(.01)
    raise AssertionError('Memory did not finish within its bounds')

try:
    if args.stage=='save':
        server=ThreadingHTTPServer(('127.0.0.1',0),Provider);threading.Thread(target=server.serve_forever,daemon=True).start()
        host.call('configure',preferences={'baseUrl':f'http://127.0.0.1:{server.server_port}/v1','model':'fixture-vision'},enabledModels=['fixture-vision'],apiKey='',rememberConnection=False)
        host.call('setImageModels',models=['fixture-vision']);host.call('setAutomaticMemory',enabled=True,revision=1)
        folder=directory/'project';folder.mkdir()
        source=host.call('createSession',kind='project',path=str(folder))['session']['id']
        next_chat=host.call('createSession',kind='project',path=str(folder))['session']['id']
        image=directory/'blue-square.png';public_image(image)
        host.call('attachFile',session=source,path=str(image));assert run(source,'What does this shared image show?')['saved']==1
        record=host.call('memories',session=source)['items'][0]
        assert record['image']['digest'] and 'uncertain' in record['confidence']
        context=host.call('context',session=next_chat,input='Which shared image showed a blue square?')
        assert context['messages'][-1]['parts'][0]['digest']==record['image']['digest']
        before=len(requests);run(next_chat,'Which shared image showed a blue square?')
        assert any(p.get('type')=='image_url' for m in requests[before]['messages'] if isinstance(m['content'],list) for p in m['content'])
        assert host.call('memoryEvidence',session=next_chat,id=record['id'],revision=record['revision'])['imageBase64']
        # Bad image extraction must preserve independently valid text memory.
        mode='malformed';host.call('attachFile',session=source,path=str(image))
        activity=run(source,'We decided to use SQLite for the local database.')
        assert activity['saved']==1 and 'Image memory unavailable' in activity['note'],activity
        assert len(host.call('memories',session=source)['items'])==2
        state={'source':source,'next':next_chat,'record':record}
        (directory/'state.json').write_text(json.dumps(state),encoding='utf-8')
    else:
        state=json.loads((directory/'state.json').read_text(encoding='utf-8'));source=state['source'];next_chat=state['next'];record=state['record']
        assert len(host.call('memories',session=next_chat)['items'])==2
        # Without a connected model, local inspection is still usable after restart.
        assert host.call('memoryEvidence',session=next_chat,id=record['id'],revision=1)['imageBase64']
        host.call('setImageModels',models=[])
        context=host.call('context',session=next_chat,input='blue square image')
        assert not context['messages'][-1].get('parts') and 'blue square' in context['messages'][0]['content']
        host.call('delete',session=source)
        missing=host.envelope('memoryEvidence',session=next_chat,id=record['id'],revision=1)
        assert not missing['ok'] and 'unavailable' in missing['error']
        assert not host.call('context',session=next_chat,input='blue square image')['memoryEntries']
        host.call('deleteMemory',session=next_chat,scope='folder',id=record['id'],revision=1)
        assert all(r['id']!=record['id'] for r in host.call('memories',session=next_chat)['items'])
    receipt={'ok':True,'stage':args.stage,'fixtureRequests':len(requests),'liveRequests':0,'uncertainSourceLinkedCaption':True,'boundedRelevantImageRecall':True,'malformedCaptionKeepsText':True,'restartMissingForget':args.stage=='restore'}
    (directory/f'receipt-{args.stage}.json').write_text(json.dumps(receipt,indent=2),encoding='utf-8');print(json.dumps(receipt))
finally:
    host.close()
    if server:server.shutdown();server.server_close()
