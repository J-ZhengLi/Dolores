"""Public isolated C ABI checks; no model, credentials or existing profile."""
import argparse
import json
from pathlib import Path
import subprocess
import sys
import time
from desktop_test_support import NativeHost, ROOT

p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--directory',type=Path,required=True)
p.add_argument('--stage',choices=['save','restore'])
p.add_argument('--install-browser',action='store_true',help='Explicit pinned npm adapter download, never a browser/model request.')
a=p.parse_args();directory=a.directory.resolve()
assert directory.is_relative_to(ROOT/'output') and directory!=ROOT/'output'
if a.stage is None:
    for stage in ['save','restore']:
        subprocess.run([sys.executable,__file__,'--directory',str(directory),'--stage',stage,*(['--install-browser'] if a.install_browser else [])],check=True)
    print((directory/'receipt.json').read_text(encoding='utf-8'));sys.exit(0)
if a.stage=='save':directory.mkdir(parents=True,exist_ok=False)
host=NativeHost(directory)
try:
    state=host.call('bootstrap')
    if a.stage=='restore':
        saved=json.loads((directory/'state.json').read_text(encoding='utf-8'))
        assert host.call('openSession',kind='project',path=str(directory/'project'))['session']['id']==saved['empty']
        assert host.call('savedDraft',session=saved['draft'])=='PUBLIC_DRAFT'
        assert not host.call('memories')['automaticPolicy']['enabled']
        knowledge=host.call('projectKnowledge',session=saved['empty'])
        assert not knowledge['learning'] and not knowledge['shareFeedback']
        receipt=json.loads((directory/'receipt.json').read_text(encoding='utf-8'));receipt['restartPreservedChoicesAndChats']=True
        (directory/'receipt.json').write_text(json.dumps(receipt,indent=2),encoding='utf-8')
    else:
        project=directory/'project';project.mkdir()
        opened=host.call('openSession',kind='project',path=str(project));first=opened['session']['id']
        for _ in range(15):assert host.call('openSession',kind='project',path=str(project))['session']['id']==first
        host.call('saveDraft',session=first,text='PUBLIC_DRAFT')
        second=host.call('openSession',kind='project',path=str(project))['session']['id'];assert second!=first
        assert host.call('openSession',kind='project',path=str(project/'.'))['session']['id']==second
        automatic=host.call('memories')['automaticPolicy'];assert automatic['enabled']
        host.call('setAutomaticMemory',enabled=False,revision=automatic['revision'])
        knowledge=host.call('projectKnowledge',session=second);assert knowledge['learning'] and knowledge['shareFeedback']
        host.call('setKnowledgePolicy',session=second,revision=knowledge['revision'],learning=False,share_feedback=False)
        browser=host.call('browserSettings');installed=False
        if a.install_browser:
            assert browser['prerequisites']['nodeReady'] and browser['prerequisites']['browserReady'],browser['prerequisites']
            status=host.call('browserSetup',action='start');deadline=time.monotonic()+105
            while status['phase']=='installing' and time.monotonic()<deadline:
                time.sleep(.15);status=host.call('browserSetup',action='status')
            assert status['phase']=='ready',status
            assert host.call('browserSettings')['available']
            installed=True
        (directory/'state.json').write_text(json.dumps({'draft':first,'empty':second}),encoding='utf-8')
        (directory/'receipt.json').write_text(json.dumps({'emptySessionReuse':True,'draftPreserved':True,'defaultLearningEnabled':True,'savedOffPreserved':True,'browserAdapterInstalled':installed,'modelRequests':0,'browserLaunches':0},indent=2),encoding='utf-8')
finally:host.close()
