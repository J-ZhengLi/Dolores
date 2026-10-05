"""Bounded real-provider check of the existing config-check-v1 repair, in isolated data.

No synthetic model responses or injected database receipts. Read-only trials use
the existing in-memory suite; real project commands are explicitly reviewed here.
This qualifies only the narrow check-command workflow, not general self-evolution.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
from desktop_test_support import NativeHost, ROOT


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', type=Path, required=True)
    parser.add_argument('--model', required=True)
    parser.add_argument('--max-output-tokens', type=int, default=1024)
    args = parser.parse_args()
    if not 512 <= args.max_output_tokens <= 4096:
        parser.error('Use an explicit bounded output allowance from 512 to 4096.')
    directory = args.directory.resolve()
    if not args.directory.is_absolute() or not directory.is_relative_to(ROOT / 'output') or directory.exists():
        parser.error('Choose a fresh absolute ignored output directory.')
    host = NativeHost(directory)
    result = {'model':args.model, 'live':True, 'passed':False, 'scope':'config-check-v1 only', 'maxOutputTokens':args.max_output_tokens}
    try:
        result['stage']='prepare'; folder = directory / 'workspace'; folder.mkdir(parents=True)
        files = {
            'package.json':'{"scripts":{"check":"node verify.cjs"}}',
            'config.json':'{"enabled":false,"theme":"system","port":3000}',
            'obsolete.cjs':'process.stderr.write("Obsolete check binding\\n"); process.exit(1);',
            'verify.cjs':'const fs=require("node:fs");const c=JSON.parse(fs.readFileSync("config.json","utf8"));process.exit(c.enabled===true&&c.theme==="system"&&c.port===3000?0:1);',
            'protected.txt':'Preserve the unrelated project file.\n',
        }
        for name, text in files.items():
            (folder / name).write_text(text, encoding='utf-8')
        protected = {name:hashlib.sha256((folder / name).read_bytes()).hexdigest() for name in files if name != 'config.json'}
        result['stage']='configure'
        host.call('configure', preferences={'baseUrl':os.environ['DOLORES_TEST_BASE_URL'], 'model':args.model},
                  apiKey=os.environ['DOLORES_TEST_API_KEY'], rememberConnection=False, enabledModels=[args.model])
        host.call('setRequestSettings', settings={'maxOutputTokens':args.max_output_tokens, 'timeoutSeconds':30})
        policy = host.call('memories')['automaticPolicy']; host.call('setAutomaticMemory', enabled=False, revision=policy['revision'])
        session = host.call('createSession', kind='project', path=str(folder))['session']['id']
        host.call('saveScopedSettings', session=session, scope='thread', revision=0,
                  patch={'task':{'modelCalls':6, 'toolCalls':8, 'segments':1, 'elapsedSeconds':60}})
        result['stage']='learning setup'
        knowledge = host.call('projectKnowledge', session=session)
        host.call('setKnowledgePolicy', session=session, revision=knowledge['revision'], learning=True, share_feedback=False)
        host.call('createCheckWorkflow', session=session, command_text='node obsolete.cjs')
        state = host.call('learningState', session=session)['state']
        host.call('setLearningPolicy', session=session, revision=state['revision'], enabled=True, automatic=True, paused=False)
        reviewed = []
        def approval(event):
            if event['type'] != 'toolApproval':
                return
            request = event['request']; command = (request.get('command') or {}).get('invocation') or {}
            allow = (request['name']=='read_text_file' and request['target'] in files) or (
                request['name']=='list_folder' and request['target']=='.') or (
                request['name']=='edit_text_file' and request['target']=='config.json') or (
                request['name']=='run_command' and command.get('program')=='node' and command.get('args') in [['obsolete.cjs'], ['verify.cjs']])
            reviewed.append(allow)
            host.call('approveTool', id=1, callId=request['callId'], allow=allow)
        result['stage']='live task and trials'
        host.call('start', id=1, session=session, input='Read package.json and config.json. Enable config.json by changing only its enabled flag to true. Use the existing project-check workflow and its literal check command. Report the actual check outcome briefly; do not change scripts or unrelated files.', tools=True)
        done, _ = host.finish(1, callback=approval, seconds=185)
        result['stage']='evaluate'
        view = host.call('learningState', session=session)
        messages = host.call('messagesPage', session=session)['items']
        metadata = messages[-1].get('metadata') or {} if messages else {}
        events = view['state']['events']; active = next((event for event in events if event['status']=='active'), None)
        result.update({'replySaved':bool(messages),
                       'replyError':bool(done.get('error')), 'eventStatuses':[event['status'] for event in events],
                       'pauseReason':(metadata.get('paused') or {}).get('reason'),
                       'declarationLearned':any(fact['title']=='Project script: check' for fact in host.call('projectKnowledge',session=session)['facts']),
                       'automaticActivation':bool(active), 'reviewedProjectOperations':len(reviewed),
                       'rejectedOperations':reviewed.count(False),
                       'preservedFiles':all(hashlib.sha256((folder/name).read_bytes()).hexdigest()==digest for name,digest in protected.items()),
                       'configCorrect':json.loads((folder/'config.json').read_bytes())=={'enabled':True,'theme':'system','port':3000},
                       'trials':[{'status':trial['status'],'improved':trial['improved'],
                                  'cases':[{'candidate':case['candidate'],'complete':case['complete'],'passed':case['passed']} for case in trial['results']]} for trial in view['trials']]})
        if active:
            restored = host.call('restoreLearning', session=session, revision=view['state']['revision'], event=active['id'])
            result['manualRestore'] = restored['state']['events'][-1]['status']=='restored'
        result['passed'] = bool(active) and result.get('manualRestore',False) and result['preservedFiles'] and result['configCorrect']
        result['automaticRegressionRestoreQualified'] = False
    except Exception as error:
        result['errorType']=type(error).__name__
        result['failure']='Bounded live qualification failed; no retry or manual activation.'
    finally:
        host.close()
    (directory/'public-result.json').write_text(json.dumps(result,indent=2),encoding='utf-8')
    print(json.dumps(result))
    if not result['passed']:
        raise SystemExit(1)


if __name__=='__main__':
    main()
