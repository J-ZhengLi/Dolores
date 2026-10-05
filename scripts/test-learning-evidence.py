"""Exercise post-chat learning through the native bridge with bounded provider failures."""
import argparse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import threading
from desktop_test_support import NativeHost, ROOT


class Provider(BaseHTTPRequestHandler):
    mode = 'command'
    requests = 0
    def log_message(self, *_):
        pass
    def do_POST(self):
        request = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        Provider.requests += 1
        # Independent repair trials fail explicitly; no fabricated passing trial.
        if Provider.requests > 2:
            self.send_response(503); self.end_headers(); return
        if Provider.requests == 1:
            calls = [
                {'index':0,'id':'read','type':'function','function':{'name':'read_text_file','arguments':json.dumps({'path':'package.json'})}},
                {'index':1,'id':'check','type':'function','function':{'name':'run_command','arguments':json.dumps({'program':'node','args':['obsolete.cjs']})}},
            ]
            events = [{'choices':[{'delta':{'tool_calls':calls},'finish_reason':None}]},
                      {'choices':[{'delta':{},'finish_reason':'tool_calls'}]}]
        else:
            events = [{'choices':[{'delta':{'content':'The obsolete check failed; inspect the saved receipt.'},'finish_reason':None}]},
                      {'choices':[{'delta':{},'finish_reason':'length' if Provider.mode=='limit' else 'stop'}]}]
        self.send_response(200); self.send_header('Content-Type','text/event-stream'); self.end_headers()
        self.wfile.write((''.join('data: '+json.dumps(event)+'\n\n' for event in events)+'data: [DONE]\n\n').encode())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', type=Path, required=True)
    args = parser.parse_args()
    directory = args.directory.resolve()
    if not args.directory.is_absolute() or not directory.is_relative_to(ROOT/'output') or directory.exists():
        parser.error('Choose a fresh absolute ignored output directory.')
    host = NativeHost(directory)
    server = ThreadingHTTPServer(('127.0.0.1',0), Provider)
    threading.Thread(target=server.serve_forever,daemon=True).start()
    try:
        host.call('configure', preferences={'baseUrl':f'http://127.0.0.1:{server.server_port}/v1','model':'fixture'}, apiKey='', rememberConnection=False)
        policy = host.call('memories')['automaticPolicy']
        host.call('setAutomaticMemory',enabled=False,revision=policy['revision'])
        host.call('setRequestSettings',settings={'maxOutputTokens':1024,'timeoutSeconds':3})
        for identity, mode in enumerate(['command','limit','denied'],1):
            folder = directory / mode; folder.mkdir(parents=True)
            (folder/'package.json').write_text('{"scripts":{"check":"node verify.cjs"}}',encoding='utf-8')
            (folder/'obsolete.cjs').write_text('process.exit(1);',encoding='utf-8')
            session = host.call('createSession',kind='project',path=str(folder))['session']['id']
            knowledge = host.call('projectKnowledge',session=session)
            host.call('setKnowledgePolicy',session=session,revision=knowledge['revision'],learning=True,share_feedback=False)
            host.call('createCheckWorkflow',session=session,command_text='node obsolete.cjs')
            learning = host.call('learningState',session=session)['state']
            host.call('setLearningPolicy',session=session,revision=learning['revision'],enabled=True,automatic=True,paused=False)
            Provider.mode=mode; Provider.requests=0
            def approve(event):
                if event['type']=='toolApproval':
                    request=event['request']
                    command=(request.get('command') or {}).get('invocation') or {}
                    allow=(request['name']=='read_text_file' and request['target']=='package.json' and mode!='denied') or (
                        request['name']=='run_command' and command=={'program':'node','args':['obsolete.cjs']})
                    host.call('approveTool',id=identity,callId=request['callId'],allow=allow)
            host.call('start',id=identity,session=session,input='Inspect package.json and enable config.json using the existing check workflow.',tools=True)
            done,_=host.finish(identity,callback=approve,seconds=15)
            assert not done.get('error'),done
            messages=host.call('messagesPage',session=session)['items']
            assert len(messages)==2 and 'obsolete check failed' in messages[-1]['content']
            knowledge=host.call('projectKnowledge',session=session)
            view=host.call('learningState',session=session)
            if mode=='command':
                assert messages[-1]['metadata']['paused']['reason']=='commandReview'
                assert any(fact['title']=='Project script: check' for fact in knowledge['facts']), 'Approved evidence was lost after command-review pause'
                event=view['state']['events'][-1]
                assert event['cause']=='skill' and event['status']=='inconclusive'
                assert len(view['trials'])==1 and view['trials'][0]['status']=='failed'
                assert Provider.requests==3
            else:
                assert knowledge['facts']==[] and view['trials']==[]
                assert not any(event['status']=='active' for event in view['state']['events'])
                assert Provider.requests==2
        print(json.dumps({'passed':True,'liveRequests':0,'commandReviewKeepsApprovedEvidence':True,
                          'failedTrialKeepsBaseline':True,'limitsAndDeniedSourcesExcluded':True,'savedRepliesRetained':True}))
    finally:
        host.close(); server.shutdown(); server.server_close()


if __name__=='__main__':
    main()
