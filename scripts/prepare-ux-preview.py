"""Prepare a public synthetic profile for normal native UI inspection, without real credentials."""
import argparse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import threading
from desktop_test_support import NativeHost, ROOT

class Handler(BaseHTTPRequestHandler):
    def log_message(self, *args): pass
    def do_POST(self):
        self.rfile.read(int(self.headers['Content-Length']))
        events = [{'choices':[{'delta':{'content':'The notes board is ready for review. Check the empty state, long titles and keyboard navigation.'},'finish_reason':None}]},
                  {'choices':[{'delta':{},'finish_reason':'stop'}],'usage':{'prompt_tokens':24,'completion_tokens':20,'total_tokens':44}}]
        self.send_response(200); self.send_header('Content-Type','text/event-stream'); self.end_headers()
        self.wfile.write((''.join('data: '+json.dumps(v)+'\n\n' for v in events)+'data: [DONE]\n\n').encode())

def main():
    p=argparse.ArgumentParser();p.add_argument('--directory',required=True);args=p.parse_args()
    directory=Path(args.directory).resolve()
    if directory.exists() or not directory.is_relative_to(ROOT/'output'):
        raise ValueError('Fresh disposable output directory required')
    server=ThreadingHTTPServer(('127.0.0.1',0),Handler);threading.Thread(target=server.serve_forever,daemon=True).start()
    host=NativeHost(directory)
    try:
        host.call('configure',preferences={'baseUrl':f'http://127.0.0.1:{server.server_port}/v1','model':'Demo model'},enabledModels=['Demo model'],apiKey='',rememberConnection=True)
        policy=host.call('memories')['automaticPolicy'];host.call('setAutomaticMemory',enabled=False,revision=policy['revision'])
        project=directory/'notes-board';project.mkdir()
        sid=host.call('createSession',kind='project',path=str(project))['session']['id']
        host.call('start',id=1,session=sid,input='Review a notes board with empty and long-title states.',tools=False)
        done,_=host.finish(1);assert not done.get('error')
        host.call('saveMemory',scope='all',title='Examples',text='Use short examples when explaining.',enabled=True)
        review=host.call('reviewSkillText',session='',scope='global',name='review',text='---\nname: review\ndescription: Review a small change\n---\nCheck basic behavior and one realistic edge case.')
        host.call('activateSkill',session='',token=review['token'])
        host.call('saveAppearance',theme='dark')
        print(json.dumps({'prepared':True,'privateData':False,'profile':str(directory/'data')}))
    finally:
        host.close();server.shutdown();server.server_close()

if __name__=='__main__':main()
