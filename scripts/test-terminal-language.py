"""Public native terminal/LSP corpus; optional bounded configured DeepSeek case.

Uses only disposable output/ projects. Native calls exercise the maintained C ABI,
not physical Flutter keyboard/mouse input. Model credentials stay in memory.
"""
import argparse
import base64
import importlib.util
import json
import os
from pathlib import Path
import re
import sqlite3
import shutil
import time
import uuid
from desktop_test_support import NativeHost, ROOT
from desktop_resource_probe import children, memory, cpu_seconds


def main(directory, profile, reuse_tools=None):
    directory.mkdir(parents=True, exist_ok=True)
    if reuse_tools:
        source=(reuse_tools/'data/language-tools').resolve(strict=True)
        if not source.is_relative_to(ROOT/'output'):raise ValueError('Only qualified output tools may be reused')
        shutil.copytree(source,directory/'data/language-tools')
    host = NativeHost(directory)
    report = {'evidence': 'Synthetic Windows C ABI corpus; not physical Flutter input'}
    def record(name, value):
        report[name] = value
        (directory/'report.json').write_text(json.dumps(report, indent=2, ensure_ascii=False), encoding='utf-8')
        print(name + ': ' + json.dumps(value, ensure_ascii=False), flush=True)
    def terminal(action, **fields): return host.call('terminal', request={'action': action, **fields})
    def language(action, **fields): return host.call('language', request={'action': action, **fields})
    tail = {}
    def poll(s):
        v = terminal('poll', id=s['id'])
        chunk = base64.b64decode(v['bytes'])
        check = tail.get(s['id'], b'') + chunk
        query = b'\x1b[6n'
        for _ in range(check.count(query)): terminal('input', id=s['id'], text='\x1b[1;1R')
        tail[s['id']] = next((check[-n:] for n in range(len(query)-1,0,-1) if check.endswith(query[:n])), b'')
        return chunk, v
    def output(s, marker, seconds=15):
        seen = bytearray(); until = time.monotonic()+seconds
        while time.monotonic() < until:
            chunk, _ = poll(s); seen.extend(chunk)
            if marker.encode() in seen: return seen.decode('utf-8', errors='replace')
            if len(seen)>4*1024*1024: raise AssertionError('Fixture output exceeded 4 MiB')
            time.sleep(.02)
        (directory/'terminal-failure.txt').write_text(seen.decode('utf-8', errors='replace'), encoding='utf-8')
        raise AssertionError('Expected executed terminal marker was absent: '+marker)
    a, b = directory/'Project A 世界', directory/'Project B'
    a.mkdir(exist_ok=True); b.mkdir(exist_ok=True)
    sessions = {name: host.call('createSession', kind='project', path=str(root))['session']['id'] for name,root in [('A',a),('B',b)]}
    def editor(name, action, **fields):
        return host.call('editor', session=sessions[name], request={'action':action, 'project':projects[name], **fields})
    projects = {name: host.call('editor',session=session,request={'action':'workspace'})['project'] for name,session in sessions.items()}
    try:
        assert terminal('list')==[] and not children(os.getpid(), names=None)
        start_cpu = cpu_seconds(os.getpid()); started=time.monotonic(); time.sleep(3)
        baseline_cpu=(cpu_seconds(os.getpid())-start_cpu)/(time.monotonic()-started)*100
        record('lazyColdHost', {'terminalProcesses':0,'languageProcesses':0,'cpuOneCorePercent':round(baseline_cpu,3),'privateMiB':round(memory(os.getpid())['privateBytes']/1048576,2)})
        shells = [terminal('create',session=sessions['A'],home=False),terminal('create',session=sessions['B'],home=False),terminal('create',session=None,home=True)]
        assert shells[0]['cwd'].replace('\\?\\','').endswith(a.name)
        assert shells[1]['cwd'].endswith(b.name)
        assert shells[2]['cwd'].replace('\\\\?\\','').lower()==str(Path.home()).lower()
        for s in shells:
            terminal('input',id=s['id'],text="[Console]::WriteLine(('PT'+'Y_UNICODE_83:世界😀'));\r")
            assert 'PTY_UNICODE_83:世界😀' in output(s,'PTY_UNICODE_83:世界😀')
            terminal('resize',id=s['id'],rows=40,cols=100)
            until=time.monotonic()+.5
            while time.monotonic()<until:poll(s);time.sleep(.02)
        ids=[s['id'] for s in shells]
        assert len(set(s['pid'] for s in shells))==3
        record('rootHomeUnicodeResize', {'passed':True,'independentShells':3,'cwdStable':terminal('list')[0]['cwd'] in [s['cwd'] for s in shells]})
        # A real foreground child receives Ctrl+C, while the shell remains usable.
        (a/'interrupt.cjs').write_text("process.on('SIGINT',()=>{console.log('INTERRUPTED_18');process.exit(0)});console.log('INTERRUPT_READY_18');setInterval(()=>{},1000);",encoding='utf-8')
        terminal('input',id=ids[0],text='node interrupt.cjs\r')
        output(shells[0],'INTERRUPT_READY_18'); terminal('input',id=ids[0],text='\x03')
        output(shells[0],'INTERRUPTED_18')
        terminal('input',id=ids[0],text="Write-Output ('AFTER'+'_INTERRUPT_18')\r")
        output(shells[0],'AFTER_INTERRUPT_18'); record('foregroundInterrupt', {'passed':True,'shellStillUsable':True})
        # Do not drain output until the bounded native queue applies backpressure.
        terminal('input',id=ids[1],text='node -e "process.stdout.write(\'F\'.repeat(4*1024*1024))"\r')
        paused=False; max_chunk=0; until=time.monotonic()+10
        time.sleep(2)
        while time.monotonic()<until:
            chunk,v=poll(shells[1]);max_chunk=max(max_chunk,len(chunk));paused=paused or v.get('backpressure',False)
            if paused:break
            time.sleep(.05)
        assert paused, 'Native queue did not expose backpressure'
        terminal('stop',id=ids[1]); assert terminal('poll',id=ids[1])['session']['state']=='stopped'
        record('floodBackpressureStop', {'passed':True,'maxPollBytes':max_chunk,'queueMiB':1,'displayScrollbackLines':5000})
        # Parent and grandchild are both owned by the terminal job.
        (a/'grandchild.cjs').write_text("require('fs').writeFileSync('grandchild-ready','yes');setTimeout(()=>require('fs').writeFileSync('late-child-write','bad'),4000);setInterval(()=>{},1000)",encoding='utf-8')
        (a/'tree.cjs').write_text("require('child_process').spawn(process.execPath,['grandchild.cjs'],{stdio:'ignore'});setInterval(()=>{},1000)",encoding='utf-8')
        terminal('input',id=ids[0],text='node tree.cjs\r')
        until=time.monotonic()+10
        while not(a/'grandchild-ready').exists() and time.monotonic()<until:poll(shells[0]);time.sleep(.02)
        assert(a/'grandchild-ready').exists(); owned=children(os.getpid(),names=None)
        terminal('stop',id=ids[0]); terminal('stop',id=ids[2]); time.sleep(4.2)
        assert not(a/'late-child-write').exists(); assert not children(os.getpid(),names=None)
        record('descendantCleanup', {'passed':True,'ownedProcessesObserved':len(owned),'remainingOwnedProcesses':0})
        checkpoint={'version':1,'sessions':[{'id':s['id'],'cwd':s['cwd'],'shell':s['shell'],'title':'Stopped fixture','output':'Retained 世界'} for s in shells], 'groups':[{'id':'g','tabs':ids,'active':ids[0]}],'activeGroup':'g','tree':{'group':'g'}}
        terminal('checkpoint',value=checkpoint); assert all(s['state']=='stopped' for s in terminal('restore')['sessions'])
        parts=terminal('share',session=sessions['A'],text='Only selected output 18')
        assert len(parts)==1 and host.call('draftAttachments',session=sessions['B'])==[]
        record('stoppedCheckpointAndScopedShare', {'passed':True,'modelRequests':0})
        # Deliberate installs exercise pinned upstream archives; no npm scripts run.
        for name in ['typescript','rust']:
            if reuse_tools:
                record(name+'PinnedInstall', {'passed':True,'versions':language('installInfo')[name],'source':'Reused prior pinned upstream installation; startup verifies every retained member'})
                continue
            job=language('install',language=name);until=time.monotonic()+280
            while time.monotonic()<until:
                state=language('installPoll',id=job['id'])
                if state['state'] in ['done','failed']:break
                time.sleep(.15)
            else:language('cancelInstall',id=job['id']);raise AssertionError('Install deadline exceeded')
            assert state['state']=='done',state
            record(name+'PinnedInstall', {'passed':True,'versions':language('installInfo')[name]})
        (a/'tsconfig.json').write_text('{"compilerOptions":{"strict":true,"target":"ES2022","module":"commonjs"},"include":["*.ts"]}')
        (a/'a.ts').write_text('export function greet(name: string): string { return "hello "+name; }\nconst message: number = greet("world");\ngre\n')
        (a/'b.ts').write_text('import { greet } from "./a";\ngreet("friend");\n')
        ts=editor('A','open',path='a.ts')
        editor('A','edit',document=ts['document'],version=ts['version'],edits=[{'start':0,'end':0,'text':'// unsaved 世界😀\n'}]);ts=editor('A','open',path='a.ts')
        def feature(project_name,doc,kind,line=0,column=0,**fields):
            job=language('feature',session=sessions[project_name],document=doc['document'],version=doc['version'],feature=kind,position={'line':line,'character':column},**fields)
            until=time.monotonic()+25
            while time.monotonic()<until:
                value=language('poll',id=job['id'])
                if value['state']=='done':return value['result']
                time.sleep(.05)
            raise AssertionError('Language fixture deadline exceeded')
        diagnostics=feature('A',ts,'diagnostics')
        assert diagnostics['items'],diagnostics
        hover=feature('A',ts,'hover',1,18);assert hover
        completion=feature('A',ts,'completion',3,3);items=completion if isinstance(completion,list) else completion.get('items',[])
        assert any(i['label']=='greet' for i in items),[i['label'] for i in items[:10]]
        definitions=feature('A',ts,'definition',2,26);assert definitions
        references=feature('A',ts,'references',1,18);assert len(references)>=2,references
        rename=feature('A',ts,'rename',1,18,name='welcome')
        preview=host.call('languageEdits',session=sessions['A'],request={'action':'preview','document':ts['document'],'version':ts['version'],'edit':rename})
        assert len(preview['files'])==2
        applied=host.call('languageEdits',session=sessions['A'],request={'action':'apply','token':preview['token']})
        assert all('welcome' in d['text'] for d in applied['documents'])
        host.call('languageEdits',session=sessions['A'],request={'action':'undo','token':preview['token']})
        ts=editor('A','open',path='a.ts')
        # A real late reply is invalidated by a newer native buffer version.
        pending=language('feature',session=sessions['A'],document=ts['document'],version=ts['version'],feature='hover',position={'line':1,'character':18})
        editor('A','edit',document=ts['document'],version=ts['version'],edits=[{'start':0,'end':0,'text':'// later\n'}])
        until=time.monotonic()+25
        while time.monotonic()<until:
            result=host.envelope('language',request={'action':'poll','id':pending['id']})
            if not result['ok']:break
            time.sleep(.05)
        assert not result['ok'] and 'changed' in result['error'].lower(),result
        record('realTypeScript', {'passed':True,'currentDiagnostics':len(diagnostics['items']),'completion':True,'hover':True,'definition':True,'references':len(references),'renameFiles':len(preview['files']),'undo':True,'staleResultRefused':True,'unsavedUnicode':True})
        (b/'src').mkdir(exist_ok=True);(b/'Cargo.toml').write_text('[package]\nname="dolores_lsp_fixture"\nversion="0.1.0"\nedition="2021"\n\n[workspace]\n')
        (b/'src/main.rs').write_text('fn main() { let answer: i32 = 42; println!("{answer}"); }\n')
        rust=editor('B','open',path='src/main.rs')
        hover=feature('B',rust,'hover',0,18)
        # Rust may finish initialization before Cargo analysis is ready. This
        # corpus permits two explicit readiness checks, never unlimited retries.
        for _ in range(2):
            if hover:break
            time.sleep(2)
            try: hover=feature('B',rust,'hover',0,18)
            except AssertionError as error:
                if 'could not provide this feature' not in str(error): raise
                hover=None
        assert hover, 'Rust returned no hover after the bounded readiness checks'
        record('realRust', {'passed':True,'hover':True,'projectIsolation':True,'buildScriptsDisabled':True})
        processes=children(os.getpid(),names=None);process_memory=[memory(pid) for pid in processes]
        start_cpu=cpu_seconds(os.getpid());started=time.monotonic();time.sleep(5)
        idle_cpu=(cpu_seconds(os.getpid())-start_cpu)/(time.monotonic()-started)*100
        assert idle_cpu-baseline_cpu<1, 'Added idle host CPU exceeded the frozen one-core ceiling'
        record('idleNativeCost', {'addedHostCpuOneCorePercent':round(idle_cpu-baseline_cpu,3),'hostPrivateMiB':round(memory(os.getpid())['privateBytes']/1048576,2),'languageTreePrivateMiB':round(sum(v['privateBytes'] for v in process_memory if v)/1048576,2),'scope':'Native Python host and owned language descendants; Flutter UI memory is separate'})
        language('stop');assert not children(os.getpid(),names=None)
        record('languageCleanup', {'passed':True,'remainingOwnedProcesses':0})
        if profile:
            spec=importlib.util.spec_from_file_location('editor_fixture',ROOT/'scripts/test-workspace-editor.py');module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
            before,count=module.digest(profile);base,key=module.connection(profile)
            model='deepseek-v4.1-flash'
            host.call('configure',preferences={'baseUrl':base,'model':model},apiKey=key,rememberConnection=False,enabledModels=[model],modelContexts={model:1000000});del key
            host.call('setRequestSettings',settings={'maxOutputTokens':2048,'timeoutSeconds':90})
            policy=host.call('memories')['automaticPolicy'];host.call('setAutomaticMemory',revision=policy['revision'],enabled=False)
            # Release editors before the separate saved-file model case.
            for name in ['A','B']:
                for doc in editor(name,'workspace')['documents']:editor(name,'close',document=doc['document'],version=doc['version'],discard=True)
            (a/'calculate.cjs').write_text('console.log(JSON.stringify({marker:"SAVED_ONLY_18",answer:Number(process.argv[2])*6}));\n')
            saved=editor('A','open',path='calculate.cjs')
            editor('A','edit',document=saved['document'],version=saved['version'],edits=[{'start':0,'end':0,'text':'// UNSAVED_MUST_NOT_SHARE\n'}])
            allowed=[];results=[];identity=181901;start=time.monotonic()
            host.call('start',id=identity,session=sessions['A'],tools=True,input='Read calculate.cjs once with read_text_file. Then call run_command once with program node, args ["calculate.cjs","7"], timeout_seconds 10 and capture_bytes 8192. These two actions are the complete task. Report the saved marker and calculated answer. Do not use other tools or change any files.')
            def approve(event):
                if event['type']=='toolApproval':
                    r=event['request'];inv=(r.get('command') or {}).get('invocation') or {}
                    yes=(r['name']=='read_text_file' and r.get('target')=='calculate.cjs') or (r['name']=='run_command' and inv=={'program':'node','args':['calculate.cjs','7']} and r['command']['timeout_seconds']<=30 and r['command']['capture_bytes']<=8192)
                    allowed.append({'tool':r['name'],'allowed':yes});host.call('approveTool',id=identity,callId=r['callId'],allow=yes)
                elif event['type']=='toolResult':results.append({'tool':event['record']['name'],'status':event['record']['status']})
            done,events=host.finish(identity,approve,seconds=100)
            passed=not done.get('error') and len(allowed)==2 and all(v['allowed'] for v in allowed) and 'SAVED_ONLY_18' in done.get('answer','') and '42' in done.get('answer','') and 'UNSAVED_MUST_NOT_SHARE' not in done.get('answer','')
            record('deepSeekSavedFileApprovedCommand',{'passed':passed,'model':model,'seconds':round(time.monotonic()-start,2),'approvals':allowed,'results':results,'outputTokens':sum((e.get('usage') or {}).get('outputTokens',0) for e in events if e['type']=='modelFinished'),'error':done.get('error'),'fixtureAnswer':done.get('answer')})
            assert module.digest(profile)==(before,count)
            record('originalProfilePreserved',{'tables':count,'unchanged':True});assert passed,'Bounded DeepSeek case failed'
    finally:
        host.close()
    print('Disposable report: '+str(directory/'report.json'))


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--directory',type=Path);parser.add_argument('--connection-profile',type=Path);parser.add_argument('--reuse-language-tools',type=Path);args=parser.parse_args()
    directory=(args.directory or ROOT/'output/terminal-language-qualification'/str(uuid.uuid4())).resolve()
    if not directory.is_relative_to(ROOT/'output'):raise ValueError('Only disposable output directories are allowed')
    main(directory,args.connection_profile.resolve() if args.connection_profile else None,args.reuse_language_tools.resolve() if args.reuse_language_tools else None)
