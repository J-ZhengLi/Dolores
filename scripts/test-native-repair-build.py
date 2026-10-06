"""Trusted fixed parser repair: frozen fault reproduction, regressions and reviewed build.

Uses only a disposable profile and a local synthetic provider. This does not
establish real-model repair reliability or approve any provider-selected code.
"""
import json
from pathlib import Path
import subprocess
import sys
import threading
import time
import uuid
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from desktop_test_support import NativeHost, ROOT

SOURCE='crates/dolores-provider-openai/src/agent_stream.rs'
OLD='''            let call = ToolCall {
                id: part.id,
                name: part.name,
                arguments: part.arguments,
            };'''
NEW='''            let arguments = match serde_json::from_str::<Value>(&part.arguments) {
                Ok(Value::String(inner)) if serde_json::from_str::<Value>(&inner).is_ok_and(|v| v.is_object()) => inner,
                _ => part.arguments,
            };
            let call = ToolCall {
                id: part.id,
                name: part.name,
                arguments,
            };'''
REPRODUCTION=r'''use dolores_core::{AgentMessage,ModelProvider,ConnectionPreferences};
use dolores_provider_openai::OpenAiProvider;
use serde_json::{json,Value};
use std::{io::{Read,Write},net::TcpListener};
async fn response(args:&str,finish:&str)->Result<dolores_core::AgentTurn,String>{
 let listener=TcpListener::bind("127.0.0.1:0").unwrap();let addr=listener.local_addr().unwrap();
 let frame=json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"one","type":"function","function":{"name":"read_text_file","arguments":args}}]},"finish_reason":finish}]});
 let body=format!("data: {frame}\n\ndata: [DONE]\n\n");
 let server=std::thread::spawn(move||{let(mut s,_)=listener.accept().unwrap();s.set_read_timeout(Some(std::time::Duration::from_secs(3))).unwrap();let mut b=[0;8192];s.read(&mut b).unwrap();write!(s,"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",body.len(),body).unwrap();});
 let provider=OpenAiProvider::new(&ConnectionPreferences{base_url:format!("http://{addr}/v1"),model:"fixture".into()},String::new()).unwrap();
 let (tx,_rx)=tokio::sync::mpsc::channel(8);
 let result=provider.stream_tool_turn(&[AgentMessage{role:"user".into(),content:"Read fixture".into(),parts:vec![],calls:vec![],call_id:None}],&[],tx,tokio_util::sync::CancellationToken::new()).await;server.join().unwrap();result
}
#[tokio::test] async fn one_encoded_object_is_a_valid_argument(){let inner=r#"{"path":"a.txt"}"#;let args=serde_json::to_string(inner).unwrap();let turn=response(&args,"tool_calls").await.expect("one JSON string wrapper must normalize");assert_eq!(turn.calls.len(),1);assert_eq!(serde_json::from_str::<Value>(&turn.calls[0].arguments).unwrap(),json!({"path":"a.txt"}));}
#[tokio::test] async fn malformed_recursive_and_non_objects_stay_invalid(){for args in ["{", "[1]", "null", "\"[1]\"", "\"\\\"{}\\\"\""]{assert!(response(args,"tool_calls").await.is_err(),"{args}");}}
#[tokio::test] async fn exhausted_output_never_publishes_a_partial_call(){let turn=response("{","length").await.unwrap();assert!(turn.output_limit);assert!(turn.calls.is_empty());}
'''

def main(directory):
    host=NativeHost(directory);call=host.call
    project=directory/'unrelated-project';project.mkdir();(project/'keep.txt').write_text('preserved')
    session=call('createSession',kind='project',path=str(project))['session']['id']
    inventory=call('harnessInventory',session=session)
    source=call('harnessNavigation',query={'action':'read','source':SOURCE,'startLine':160,'lineCount':55})
    # Read metadata through the same supported source operation used by the model.
    if 'sourceId' not in source:
        source=call('harnessSource',source=SOURCE,start_line=160,line_count=55)
    mode='prepare';repair_id=None;evaluation_id=None;reviews=[];durations={}
    class Provider(BaseHTTPRequestHandler):
        def log_message(self,*args):pass
        def do_POST(self):
            request=json.loads(self.rfile.read(int(self.headers['Content-Length'])))
            if any(m['role']=='tool' for m in request['messages']):delta={'content':'Fixed qualification evidence retained. No automatic installation or replay.'};finish='stop'
            else:
                name='harness_repair';args={'action':'prepare','source':SOURCE,'sourceId':source['sourceId'],'bundleId':inventory['sourceBundle']['bundleId']}
                if mode=='patch':
                    state=call('harnessRepairs',session=session,repairId=repair_id)
                    args={'action':'propose','repairId':repair_id,'revision':state['revision'],'source':SOURCE,'sourceId':state['files'][0]['candidateId'],'oldText':OLD,'newText':NEW}
                elif mode=='evaluate':name='test_harness_repair';args={'repairId':repair_id,'revision':2,'package':'dolores-provider-openai','reproduction':REPRODUCTION}
                elif mode.startswith('build'):
                    name='build_harness_repair';args={'repairId':repair_id,'revision':1 if mode=='build-stale' else 2,'evaluationId':evaluation_id}
                delta={'tool_calls':[{'index':0,'id':f'{mode}-call','type':'function','function':{'name':name,'arguments':json.dumps(args)}}]};finish='tool_calls'
            self.send_response(200);self.send_header('Content-Type','text/event-stream');self.end_headers();self.wfile.write(('data: '+json.dumps({'choices':[{'delta':delta,'finish_reason':finish}]})+'\n\ndata: [DONE]\n\n').encode())
    server=ThreadingHTTPServer(('127.0.0.1',0),Provider);threading.Thread(target=server.serve_forever,daemon=True).start()
    def review(event):
        if event['type']!='toolApproval':return
        r=event['request'];reviews.append(r['name'])
        assert r['name'] in ['harness_repair','test_harness_repair','build_harness_repair']
        if r['name']=='test_harness_repair':assert REPRODUCTION in r['diff'].replace('\n+','\n')
        if r['name']=='build_harness_repair':
            q=json.loads(r['query']);assert q['build']['evaluationId']==evaluation_id;assert q['command']['invocation']['args'][:6]==['build','--offline','--locked','--release','-p','dolores-flutter-bridge'];assert q['command']['invocation']['args'][6]=='--target-dir';assert q['command']['timeout_seconds']==300
        call('approveTool',id=identity,callId=r['callId'],allow=mode!='build-decline')
    try:
        policy=call('memories')['automaticPolicy'];call('setAutomaticMemory',enabled=False,revision=policy['revision'])
        call('configure',preferences={'baseUrl':f'http://127.0.0.1:{server.server_port}/v1','model':'fixture'},apiKey='',rememberConnection=False)
        for identity,mode in enumerate(['prepare','patch','evaluate','build-stale','build-decline','build'],1):
            start=time.monotonic();call('start',id=identity,session=session,input=f'Fixed native qualification: {mode}')
            done,events=host.finish(identity,review,seconds=940 if mode=='evaluate' else 340);durations[mode]=round(time.monotonic()-start,2);assert not done.get('error'),done
            if mode=='prepare':repair_id=call('harnessRepairs',session=session)['repairIds'][0]
            elif mode=='evaluate':
                e=call('harnessRepairs',session=session,repairId=repair_id)['evaluations'][0];assert e['status']=='qualified',e;evaluation_id=e['evaluationId']
            elif mode=='build-stale':assert any(e['type']=='toolResult' and e['record']['status']=='blocked' for e in events)
            elif mode=='build-decline':assert not call('nativeRepairs',session=session)['builds']
            print(json.dumps({'stage':mode,'seconds':durations[mode]}),flush=True)
        builds=call('nativeRepairs',session=session)['builds'];assert len(builds)==1,builds;assert builds[0]['status']=='ready',builds
        # Direct user installation authority is unavailable to this C ABI driver.
        denied=host.envelope('reviewNativeUpdate',session=session,buildId=builds[0]['id'],restore=False);assert not denied['ok'] and 'normal Dolores app' in denied['error'],denied
        call('saveDraft',session=session,text='Unsaved qualification draft preserved across native restart.')
        assert (project/'keep.txt').read_text()=='preserved'
        (directory/'qualification.json').write_text(json.dumps({'session':session,'buildId':builds[0]['id'],'repairId':repair_id,'evaluationId':evaluation_id,'durations':durations,'reviews':reviews,'nativeInstallations':0,'projectPreserved':True},indent=2))
        print(json.dumps({'build':'ready','baselineFault':'reproduced','candidateAndRegressions':'passed','staleAndDeniedBuilds':'refused','driverInstall':'refused','durations':durations}))
    finally:server.shutdown();host.close()

if __name__=='__main__':
    if len(sys.argv)==3 and sys.argv[1]=='--child':main(Path(sys.argv[2]).resolve())
    else:
        directory=ROOT/'output/native-build-qualification'/str(uuid.uuid4());directory.mkdir(parents=True)
        subprocess.run([sys.executable,__file__,'--child',str(directory)],check=True);print(f'Disposable evidence: {directory}')
