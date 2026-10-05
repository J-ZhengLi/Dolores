"""Bounded model setup/chat check in a fresh synthetic profile. Never records keys or transcripts."""
import argparse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import threading
from desktop_test_support import NativeHost, ROOT

class Handler(BaseHTTPRequestHandler):
    def log_message(self, *args): pass
    def do_GET(self):
        data=json.dumps({'data':[{'id':'fixture'}]}).encode()
        self.send_response(200); self.send_header('Content-Type','application/json'); self.end_headers(); self.wfile.write(data)
    def do_POST(self):
        request=json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        assert request['max_tokens']==512
        data='data: '+json.dumps({'choices':[{'index':0,'delta':{'content':'READY'},'finish_reason':None}]})+'\n\n'
        data+='data: '+json.dumps({'choices':[{'index':0,'delta':{},'finish_reason':'stop'}], 'usage':{'prompt_tokens':5,'completion_tokens':1,'total_tokens':6}})+'\n\ndata: [DONE]\n\n'
        self.send_response(200); self.send_header('Content-Type','text/event-stream'); self.end_headers(); self.wfile.write(data.encode())

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--directory',required=True);parser.add_argument('--model');args=parser.parse_args()
    directory=Path(args.directory).resolve()
    if not directory.is_relative_to(ROOT/'output') or directory.exists():raise ValueError('Fresh owned output directory required')
    server=None
    if args.model:
        endpoint=os.environ['DOLORES_TEST_BASE_URL'];key=os.environ['DOLORES_TEST_API_KEY'];model=args.model
    else:
        server=ThreadingHTTPServer(('127.0.0.1',0),Handler);threading.Thread(target=server.serve_forever,daemon=True).start()
        endpoint=f'http://127.0.0.1:{server.server_port}/v1';key='';model='fixture'
    host=NativeHost(directory)
    try:
        models=host.call('listModels',baseUrl=endpoint,apiKey=key)
        assert model in models,'Configured test model unavailable in discovery'
        preferences={'baseUrl':endpoint,'model':model}
        host.call('configure',preferences=preferences,apiKey=key,rememberConnection=False,enabledModels=[model])
        expected=host.call('modelDetails',preferences=preferences)
        details={'contextWindowTokens':None,'imageInput':False,'requestSettings':{'maxOutputTokens':512,'timeoutSeconds':30}}
        assert host.call('setModelDetails',preferences=preferences,expected=expected,details=details)==details
        assert host.call('bootstrap')['preferences']==preferences
        sid=host.call('createSession',kind='side')['session']['id']
        host.call('start',id=1,session=sid,input='Reply with only READY.',tools=False)
        done,events=host.finish(1,seconds=40)
        assert not done.get('error'), 'Model chat did not complete; inspect the actual bounded failure separately'
        assert 'READY' in ''.join(e.get('text','') for e in events if e['type']=='delta'), 'Model did not return the requested confirmation within its allowance'
        host.call('setModelDetails',preferences=preferences,expected=details,details=expected)
        result={'provider':'live' if args.model else 'fixture','model':model,'discovered':True,'atomicDetails':True,'chat':True,'restoredProfile':True}
        (directory/'result.json').write_text(json.dumps(result),encoding='utf-8');print(json.dumps(result))
    finally:
        host.close()
        if server:server.shutdown()
if __name__=='__main__':main()
