"""Public C ABI editor/recovery fixtures; optional bounded configured-model probe.

Only disposable output profiles and synthetic files are mutated. The optional
Windows credential is read into memory, never printed, exported or persisted.
This script does not qualify physical editor input or Flutter performance.
"""
import argparse
import ctypes
from ctypes import wintypes as w
import hashlib
import json
from pathlib import Path
import sqlite3
import subprocess
import sys
import time
import uuid
from desktop_test_support import NativeHost, ROOT


def digest(profile):
    with sqlite3.connect((profile / 'dolores.db').as_uri()+'?mode=ro', uri=True) as db:
        rows=[]
        for(name,)in db.execute("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"):
            values=sorted(repr(tuple(r)) for r in db.execute('SELECT * FROM "'+name.replace('"','""')+'"'))
            rows.append((name, values))
        return hashlib.sha256(repr(rows).encode()).hexdigest(), len(rows)


def connection(profile):
    with sqlite3.connect((profile / 'dolores.db').as_uri()+'?mode=ro', uri=True) as db:
        base, _=db.execute('SELECT base_url,model FROM preferences').fetchone()
        saved=json.loads(db.execute('SELECT data FROM remembered_connection').fetchone()[0])
    scope=hashlib.sha256(('\\\\?\\'+str(profile.resolve())).lower().encode()).hexdigest()
    target=scope+':'+saved['credentialId']+'.dev.dolores.desktop.connection'
    class Credential(ctypes.Structure):
        _fields_=[('Flags',w.DWORD),('Type',w.DWORD),('TargetName',w.LPWSTR),('Comment',w.LPWSTR),('LastWritten',w.FILETIME),('CredentialBlobSize',w.DWORD),('CredentialBlob',ctypes.POINTER(ctypes.c_ubyte)),('Persist',w.DWORD),('AttributeCount',w.DWORD),('Attributes',ctypes.c_void_p),('TargetAlias',w.LPWSTR),('UserName',w.LPWSTR)]
    api=ctypes.WinDLL('advapi32',use_last_error=True)
    api.CredReadW.argtypes=[w.LPCWSTR,w.DWORD,w.DWORD,ctypes.POINTER(ctypes.POINTER(Credential))]
    api.CredReadW.restype=w.BOOL;api.CredFree.argtypes=[ctypes.c_void_p]
    pointer=ctypes.POINTER(Credential)()
    if not api.CredReadW(target,1,0,ctypes.byref(pointer)):raise RuntimeError('Configured test credential unavailable.')
    try:secret=json.loads(ctypes.string_at(pointer.contents.CredentialBlob,pointer.contents.CredentialBlobSize).decode('utf-16-le'))
    finally:api.CredFree(pointer)
    assert secret['base_url']==base,'Credential endpoint mismatch.'
    return base,secret['api_key']


def exercise(directory, profile):
    host=NativeHost(directory);call=host.call;results={}
    a=directory/'A';b=directory/'B';a.mkdir();b.mkdir()
    original=b'\xef\xbb\xbfmarker = OLD\r\n'
    (a/'same.txt').write_bytes(original);(b/'same.txt').write_text('Project B remains unchanged.\n')
    (a/'binary.bin').write_bytes(b'\x00\xff\x01')
    (a/'long.txt').write_text('x'*100*1024)
    sessions={name:call('createSession',kind='project',path=str(root))['session']['id'] for name,root in [('A',a),('B',b)]}
    projects={name:call('editor',session=session,request={'action':'workspace'})['project'] for name,session in sessions.items()}
    def editor(action, name='A', **fields):return call('editor',session=sessions[name],request={'action':action,'project':projects[name],**fields})
    def mutate(d, action, **fields):return editor(action,document=d['document'],version=d['version'],**fields)
    try:
        d=editor('open',path='same.txt')
        assert d['snapshot']['bom'] and d['snapshot']['newline']=='crlf'
        mutate(d,'edit',edits=[{'start':9,'end':12,'text':'DOLORES-SAVED-73'}]);d=editor('open',path='same.txt')
        assert (a/'same.txt').read_bytes()==original
        assert not host.envelope('editor',session=sessions['B'],request={'action':'save','project':projects['A'],'document':d['document'],'version':d['version']})['ok']
        d=mutate(d,'save');assert (a/'same.txt').read_bytes()==b'\xef\xbb\xbfmarker = DOLORES-SAVED-73\r\n'
        results['bomCrlfAndProjectIsolation']='passed'
        mutate(d,'edit',edits=[{'start':0,'end':0,'text':'local '}]);d=editor('open',path='same.txt')
        (a/'same.txt').write_bytes(b'\xef\xbb\xbfexternal disk base\r\n')
        refused=host.envelope('editor',session=sessions['A'],request={'action':'save','project':projects['A'],'document':d['document'],'version':d['version']})
        assert not refused['ok'] and 'changed' in refused['error'].lower()
        compared=mutate(d,'compare');d=mutate(d,'rebase',revision=compared['disk']['revision'])
        assert d['dirty'] and d['text'].startswith('local ')
        d=mutate(d,'save');results['staleSaveCompareAndRebase']='passed'
        for path in ['binary.bin','long.txt']:
            preview=editor('open',path=path);assert preview['snapshot']['readonly'];mutate(preview,'close',discard=False)
        results['binaryAndLongLine']='read-only; original bytes retained'
        # Fail only the private checkpoint table; physical saves must report success.
        with sqlite3.connect(directory/'data/dolores.db') as db:
            db.execute("CREATE TRIGGER fail_editor_checkpoint BEFORE INSERT ON workspace_editor_state BEGIN SELECT RAISE(FAIL,'fixture recovery unavailable'); END")
        mutate(d,'edit',edits=[{'start':0,'end':6,'text':''}]);d=editor('open',path='same.txt')
        saved=mutate(d,'save');assert 'saved' in saved and 'recoveryWarning' in saved;d=saved['saved']
        assert not d['dirty'] and (a/'same.txt').read_bytes()==b'\xef\xbb\xbfmarker = DOLORES-SAVED-73\r\n'
        renamed=mutate(d,'rename',path='renamed.txt');assert 'saved' in renamed and renamed['saved']['snapshot']['path']=='renamed.txt';d=renamed['saved']
        assert not(a/'same.txt').exists() and(a/'renamed.txt').exists()
        # Restore the original file name for the saved-file live case.
        renamed=mutate(d,'rename',path='same.txt');d=renamed['saved'];results['completedActionsWithFailedRecovery']='saved and renamed with explicit warnings'
        disposable=editor('create',path='delete-me.txt');deleted=mutate(disposable,'delete')
        assert deleted['deleted'] and deleted['recoveryWarning'] and not(a/'delete-me.txt').exists()
        results['deletedWithFailedRecovery']='deleted with explicit warning; no false failure'
        with sqlite3.connect(directory/'data/dolores.db') as db:db.execute('DROP TRIGGER fail_editor_checkpoint')
        editor('checkpoint')
        if profile:
            before, count=digest(profile);base,key=connection(profile)
            call('configure',preferences={'baseUrl':base,'model':'Qwen/Qwen3.5-2B'},apiKey=key,rememberConnection=False,enabledModels=['Qwen/Qwen3.5-2B'],modelContexts={'Qwen/Qwen3.5-2B':32768});del key
            call('setRequestSettings',settings={'maxOutputTokens':512,'timeoutSeconds':60})
            policy=call('memories')['automaticPolicy'];call('setAutomaticMemory',revision=policy['revision'],enabled=False)
            # Keep an unsaved variant; the model must see saved bytes only.
            mutate(d,'edit',edits=[{'start':9,'end':25,'text':'UNSAVED-MUST-NOT-SHARE'}]);d=editor('open',path='same.txt')
            approvals=[];tools=[];identity=7101;start=time.monotonic()
            call('start',id=identity,session=sessions['A'],tools=True,input='Call read_text_file exactly once with {"path":"same.txt"}. Report the marker value found in the saved file. Do not use other tools.')
            def approve(event):
                if event['type']=='toolApproval':
                    r=event['request'];allowed=r['name']=='read_text_file' and r.get('target')=='same.txt';approvals.append(allowed)
                    call('approveTool',id=identity,callId=r['callId'],allow=allowed)
                elif event['type']=='toolResult':tools.append(event['record'])
            done,events=host.finish(identity,approve,seconds=70)
            passed=not done.get('error') and approvals==[True] and len(tools)==1 and tools[0]['status']=='read' and 'DOLORES-SAVED-73' in done.get('answer','') and 'UNSAVED-MUST-NOT-SHARE' not in done.get('answer','')
            results['qwenSavedFile']={'passed':passed,'seconds':round(time.monotonic()-start,2),'approvedReads':sum(approvals),'toolStatuses':[r['status'] for r in tools],'outputTokens':sum((e.get('usage') or {}).get('outputTokens',0) for e in events if e['type']=='modelFinished'),'error':done.get('error'),'fixtureAnswer':done.get('answer'),'eventTypes':sorted(set(e['type'] for e in events))}
            assert digest(profile)==(before,count),'Original profile changed.'
            results['originalProfilePreserved']={'tables':count,'unchanged':True}
            if not passed:
                (directory/'report.json').write_text(json.dumps(results,indent=2))
                raise AssertionError('Configured saved-file probe failed; inspect the public fixture report.')
        # Checkpoint one dirty draft for a fresh-process recovery verification.
        d=editor('open',path='same.txt')
        if not d['dirty']:mutate(d,'edit',edits=[{'start':0,'end':0,'text':'recoverable '}]);d=editor('open',path='same.txt')
        editor('checkpoint');(directory/'recovery-expected.json').write_text(json.dumps({'session':sessions['A'],'project':projects['A'],'text':d['text'],'bytes':list((a/'same.txt').read_bytes())}))
        assert (b/'same.txt').read_text()=='Project B remains unchanged.\n'
        (directory/'report.json').write_text(json.dumps(results,indent=2));print(json.dumps(results),flush=True)
    finally:host.close()


def recover(directory):
    expected=json.loads((directory/'recovery-expected.json').read_text());host=NativeHost(directory)
    try:
        value=host.call('editor',session=expected['session'],request={'action':'recover','project':expected['project'],'path':'same.txt'})
        assert value['dirty'] and value['text']==expected['text']
        assert list((directory/'A/same.txt').read_bytes())==expected['bytes']
        results=json.loads((directory/'report.json').read_text());results['freshProcessRecovery']='passed; source bytes unchanged'
        (directory/'report.json').write_text(json.dumps(results,indent=2));print(json.dumps({'freshProcessRecovery':'passed'}))
    finally:host.close()


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--connection-profile',type=Path);parser.add_argument('--recover',type=Path);parser.add_argument('--exercise',type=Path);args=parser.parse_args()
    if args.recover:recover(args.recover.resolve())
    elif args.exercise:exercise(args.exercise.resolve(),args.connection_profile.resolve() if args.connection_profile else None)
    else:
        directory=ROOT/'output/workspace-editor-qualification'/str(uuid.uuid4());directory.mkdir(parents=True)
        command=[sys.executable,__file__,'--exercise',str(directory)]
        if args.connection_profile:command+=['--connection-profile',str(args.connection_profile.resolve())]
        subprocess.run(command,check=True)
        subprocess.run([sys.executable,__file__,'--recover',str(directory)],check=True)
        print('Disposable report: '+str(directory/'report.json'))
