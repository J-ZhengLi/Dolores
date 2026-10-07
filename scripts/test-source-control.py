"""Synthetic release C ABI Git qualification. Never uses the user's repository/remotes."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import time
import uuid
from desktop_test_support import NativeHost, ROOT
from desktop_resource_probe import memory, children

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--host',type=Path);args=parser.parse_args()
    directory=ROOT/'output'/('source-control-17-'+uuid.uuid4().hex);directory.mkdir()
    host=NativeHost(directory,args.host);times={};report={'evidence':'synthetic public C ABI fixture; not physical input or model reliability','checks':{}}
    def shell(folder,*argv):
        out=subprocess.run(['git',*argv],cwd=folder,stdin=subprocess.DEVNULL,capture_output=True,timeout=20)
        assert out.returncode==0,out.stderr.decode(errors='replace');return out.stdout
    def start(session,action,**fields):return host.call('git',session=session,request={'action':action,**fields})['job']
    def finish(session,job):
        deadline=time.monotonic()+65
        while time.monotonic()<deadline:
            r=host.envelope('git',session=session,request={'action':'poll','job':job})
            if not r['ok']:raise AssertionError(r['error'])
            if r['result']['done']:return r['result']['value']
            time.sleep(.01)
        raise AssertionError('Git fixture deadline exceeded')
    def git(session,action,**fields):
        t=time.monotonic();v=finish(session,start(session,action,**fields));times.setdefault(action,[]).append(round((time.monotonic()-t)*1000,2));return v
    def review(session,operation):
        status=git(session,'status');return git(session,'review',repo=status['repo'],revision=status['revision'],operation=operation)
    def apply(session,r):return git(session,'apply',repo=r['repo'],token=r['token'])
    try:
        sessions={};roots={};
        for name in ['A','B']:
            folder=directory/name;folder.mkdir();roots[name]=folder;shell(folder,'init','-b','main');shell(folder,'config','user.name','Public Fixture');shell(folder,'config','user.email','fixture@example.invalid');shell(folder,'config','core.autocrlf','false')
            (folder/'same.txt').write_text('base '+name+'\n',encoding='utf-8',newline='');shell(folder,'add','.');shell(folder,'commit','-m','base')
            sessions[name]=host.call('createSession',kind='project',path=str(folder))['session']['id']
        a,b=sessions['A'],sessions['B'];before=memory(os.getpid());jobs={n:start(s,'status')for n,s in sessions.items()};statuses={n:finish(sessions[n],j)for n,j in jobs.items()};assert statuses['A']['repo']!=statuses['B']['repo'];report['checks']['twoRepositoryOwnership']=True
        for i in range(35):shell(roots['A'],'commit','--allow-empty','-m',f'page-{i}')
        s=git(a,'status');page=git(a,'history',repo=s['repo'],head=s['head'],cursor=0);assert len(page['items'])==30 and page['next']==30;page2=git(a,'history',repo=s['repo'],head=s['head'],cursor=30);assert len(page2['items'])==6;report['checks']['pinnedHistoryPaging']=True
        (roots['A']/'same.txt').write_text('working A\n',encoding='utf-8',newline='');s=git(a,'status');d=git(a,'diff',repo=s['repo'],revision=s['revision'],path='same.txt',basis='working');assert d['left']=='base A\n' and d['right']=='working A\n'
        (roots['A']/'new.txt').write_text('reviewed new content\n',encoding='utf-8',newline='');r=review(a,{'kind':'stage','paths':['new.txt']});assert 'reviewed new content' in r['patch'];apply(a,r);report['checks']['newFileContentReview']=True
        project=host.call('editor',session=a,request={'action':'workspace'})['project'];ed=host.call('editor',session=a,request={'action':'open','project':project,'path':'same.txt'});host.call('editor',session=a,request={'action':'edit','project':project,'document':ed['document'],'version':ed['version'],'edits':[{'start':0,'end':0,'text':'unsaved '}]})
        r=review(a,{'kind':'discard','path':'same.txt'})
        try:apply(a,r);raise AssertionError('Dirty editor was overwritten')
        except AssertionError as e:assert 'unsaved' in str(e).lower() or 'save' in str(e).lower()
        ed=host.call('editor',session=a,request={'action':'open','project':project,'path':'same.txt'});host.call('editor',session=a,request={'action':'close','project':project,'document':ed['document'],'version':ed['version'],'discard':True});report['checks']['nativeDirtyBufferGuard']=True
        apply(a,review(a,{'kind':'stage','paths':['same.txt']}))
        hook=roots['A']/'.git/hooks/pre-commit';hook.write_text('#!/bin/sh\necho fixture-hook-refused >&2\nexit 1\n');r=review(a,{'kind':'commit','message':'retained review message'})
        try:apply(a,r);raise AssertionError('Hook was bypassed')
        except AssertionError as e:assert 'fixture-hook-refused' in str(e)
        hook.unlink();apply(a,review(a,{'kind':'commit','message':'retained review message'}));report['checks']['hookRefusalFreshReviewRecovery']=True
        (roots['A']/'same.txt').write_text('stashed A\n',encoding='utf-8',newline='');apply(a,review(a,{'kind':'stashCreate','paths':['same.txt'],'message':'public selected'}));s=git(a,'status');local=git(a,'localState',repo=s['repo']);apply(a,review(a,{'kind':'stashApply','stash':local['stashes'][0]['id'],'pop':True}));apply(a,review(a,{'kind':'discard','path':'same.txt'}));report['checks']['selectedStashAndDiscard']=True
        bare=directory/'remote.git';shell(directory,'init','--bare',str(bare));shell(roots['A'],'remote','add','fixture',str(bare));shell(roots['A'],'config','branch.main.remote','fixture');shell(roots['A'],'config','branch.main.merge','refs/heads/main');apply(a,review(a,{'kind':'push','remote':'fixture','branch':'refs/heads/main'}));apply(a,review(a,{'kind':'fetch','remote':'fixture'}));report['checks']['localBareRemoteOnly']=True
        # A sleeping owned hook provides a real cancellation and concurrent-read probe.
        (roots['A']/'same.txt').write_text('cancel retained\n',encoding='utf-8',newline='');apply(a,review(a,{'kind':'stage','paths':['same.txt']}));hook.write_text('#!/bin/sh\nsleep 20\n');r=review(a,{'kind':'commit','message':'canceled message'});job=start(a,'apply',repo=r['repo'],token=r['token']);time.sleep(.4);assert git(b,'status')['repo']==statuses['B']['repo'];t=time.monotonic();host.call('git',session=a,request={'action':'cancel','job':job})
        try:finish(a,job);raise AssertionError('Cancel unexpectedly committed')
        except AssertionError as e:assert 'stop' in str(e).lower()
        report['cancelReapMs']=round((time.monotonic()-t)*1000,2);hook.unlink();assert children(os.getpid(),None)==[];apply(a,review(a,{'kind':'commit','message':'fresh after stop'}));report['checks']['ownedHookCancelAndOtherRepositoryRead']=True
        assert (roots['B']/'same.txt').read_text()=='base B\n';after=memory(os.getpid());report.update(timingsMs=times,hostPrivateIncrementBytes=after['privateBytes']-before['privateBytes'],ownedProcessesAfterJobs=len(children(os.getpid(),None)))
        time.sleep(.3);assert children(os.getpid(),None)==[];report['checks']['noIdleGitProcess']=True
        # Real C ABI viewer regressions: whole blobs never enter the render DTO.
        large=roots['A']/'large.txt';context='unchanged context\n'*1_050_000
        large.write_text('old value\n'+context,encoding='utf-8',newline='');shell(roots['A'],'add','large.txt');shell(roots['A'],'commit','-m','large public base')
        large.write_text('new value\n'+context,encoding='utf-8',newline='');s=git(a,'status');d=git(a,'diff',repo=s['repo'],revision=s['revision'],path='large.txt',basis='working')
        assert d['reason'] is None and '+new value' in d['patch'] and d['next'] is None
        assert any(row['kind']=='remove' and row['oldLine']==1 for row in d['rows']) and any(row['kind']=='add' and row['newLine']==1 for row in d['rows']);report['checks']['largeFileSmallDiff']=True
        pages=roots['A']/'pages.txt';old=''.join(f'old content {i}\n'for i in range(30000));new=old.replace('old content','new content')
        pages.write_text(old,encoding='utf-8',newline='');shell(roots['A'],'add','pages.txt');shell(roots['A'],'commit','-m','public patch base');pages.write_text(new,encoding='utf-8',newline='')
        s=git(a,'status');first=git(a,'diff',repo=s['repo'],revision=s['revision'],path='pages.txt',basis='working');assert len(first['rows'])==256 and first['totalRows']>60000
        last=git(a,'diff',repo=s['repo'],revision=s['revision'],path='pages.txt',basis='working',cursor=first['totalRows']-32,digest=first['digest']);assert last['next'] is None and '+new content 29999' in last['patch'];report['checks']['largePatchInAppPaging']=True
        pages.write_text('changed during viewing\n',encoding='utf-8',newline='')
        try:git(a,'diff',repo=s['repo'],revision=s['revision'],path='pages.txt',basis='working',cursor=first['next'],digest=first['digest']);raise AssertionError('Stale diff page was accepted')
        except AssertionError as e:assert 'basis changed' in str(e)
        s=git(a,'status');fresh=git(a,'diff',repo=s['repo'],revision=s['revision'],path='pages.txt',basis='working')
        recovered=git(a,'diff',repo=s['repo'],revision=s['revision'],path='pages.txt',basis='working',cursor=max(0,fresh['totalRows']-32),digest=fresh['digest']);assert '+changed during viewing' in recovered['patch'];report['checks']['stalePageRefreshRecovery']=True
        report['viewerFixtures']='Public 17 MiB file/small edit and over-512-KiB patch; pre-viewer resource measurements above remain separate.'
        assert children(os.getpid(),None)==[]
    finally:host.close()
    assert all(report['checks'].values());(directory/'result.json').write_text(json.dumps(report,indent=2));print(json.dumps({'directory':str(directory),'checks':report['checks'],'cancelReapMs':report['cancelReapMs'],'hostPrivateIncrementBytes':report['hostPrivateIncrementBytes']}))
if __name__=='__main__':main()
