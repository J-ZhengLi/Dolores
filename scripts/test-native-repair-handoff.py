"""Exercise the protected launcher with a trusted, qualified disposable build.

This constructs fixed test intents outside the product UI. It does not bypass
the product's native review or establish that a user has clicked its review.
Only the exact recorded idle fixture process is stopped for fixture cleanup.
"""
import importlib.util
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import sys
import time
import uuid
from desktop_test_support import ROOT

spec=importlib.util.spec_from_file_location('desktop',ROOT/'scripts/desktop.py')
desktop=importlib.util.module_from_spec(spec);spec.loader.exec_module(desktop)

def save(path,value):
    path.parent.mkdir(parents=True,exist_ok=True)
    temporary=path.with_suffix('.pending');temporary.write_text(json.dumps(value),encoding='utf-8');os.replace(temporary,path)

def identity(pid):return desktop.process_identity(pid)

def stop(process):
    # Reuse the same-handle creation/executable check, scoped to this exact owned fixture.
    path=Path(process['executable']).resolve()
    if not path.is_relative_to(ROOT/'output') and path!=desktop.executable().resolve():raise ValueError('Unowned fixture process')
    previous=desktop.executable;desktop.executable=lambda:path
    try:desktop.process_identity(process['pid'],terminate_expected=process)
    finally:desktop.executable=previous
    for _ in range(100):
        if desktop.process_identity(process['pid']) is None:return
        time.sleep(.05)
    raise AssertionError('Owned fixture did not stop')

def table_digest(profile):
    import hashlib
    db=sqlite3.connect(f'file:{profile / "dolores.db"}?mode=ro',uri=True)
    try:
        rows=[]
        for(name,)in db.execute("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"):
            values=[repr(tuple(r)) for r in db.execute('SELECT * FROM "'+name.replace('"','""')+'"')];rows.append((name,sorted(values)))
        return hashlib.sha256(repr(rows).encode()).hexdigest()
    finally:db.close()

def job(directory,build,old,operation,wrong_source=False):
    profile=directory/'data';root=profile/'repairs'/('build-'+build['id']);id=str(uuid.uuid4())
    source=build['sourceId'] if operation=='restore' or wrong_source else build['candidateSourceId']
    intent={'id':id,'build':build,'profile':str(profile),'buildRoot':str(root),'old':old,'child':None,'helper':None,'operation':operation,'stage':'waiting','note':'Trusted fixed normal-startup qualification only; no original task replay.','databaseId':None,'backupId':None,'expectedSource':source}
    path=profile/'native-updates'/('job-'+id)/'handoff.json';save(path,intent);save(profile/'native-updates/current.json',id)
    launcher=root/('bundle' if operation=='restore' else 'previous')/'dolores-update-launcher.exe'
    proc=subprocess.Popen([str(launcher),'--handoff',str(path)],creationflags=subprocess.CREATE_NO_WINDOW)
    return path,proc

def wait(path,proc,expected,seconds=95):
    deadline=time.monotonic()+seconds
    while time.monotonic()<deadline:
        try:
            j=json.loads(path.read_text())
        except PermissionError:
            # Windows can briefly deny a read while the helper atomically
            # publishes its receipt. Keep the existing overall wait bound.
            time.sleep(.05)
            continue
        if j['stage'] in ['applied','restored','rolledBack','failed','recoveryRequired']:
            assert j['stage']==expected,j;proc.wait(timeout=10);return j
        time.sleep(.1)
    raise AssertionError('Native startup/recovery fixture exceeded bound')

def main(directory):
    info=json.loads((directory/'qualification.json').read_text());profile=directory/'data';build=json.loads((profile/'repairs'/('build-'+info['buildId'])/'build.json').read_text())
    # Start the actual normal shell once so the old-process receipt is genuine.
    record=directory/('normal-'+str(uuid.uuid4())+'.json')
    subprocess.run([sys.executable,str(ROOT/'scripts/desktop.py'),'launch','--data-directory',str(profile),'--pid-file',str(record)],check=True)
    tracked=json.loads(record.read_text());old=identity(tracked['pid']);assert old
    stop(old)  # idle synthetic profile only; graceful product Close remains a separate UI gate.
    before=table_digest(profile)
    path,p=job(directory,build,old,'install');installed=wait(path,p,'applied');assert table_digest(profile)==before
    assert Path(installed['child']['executable']).parent==profile/'repairs'/('build-'+build['id'])/'bundle'
    save(directory/'installed.json',installed);stop(installed['child'])
    # Simulate acknowledged post-install user work through the supported candidate bridge.
    snippet='from pathlib import Path; from desktop_test_support import NativeHost; import sys; d=Path(sys.argv[1]); h=NativeHost(d,host=Path(sys.argv[2])); h.call("saveDraft",session=sys.argv[3],text="Work created after healthy installation must survive Restore. "+sys.argv[4]); h.close()'
    subprocess.run([sys.executable,'-c',snippet,str(directory),str(profile/'repairs'/('build-'+build['id'])/'bundle'),info['session'],str(uuid.uuid4())],cwd=ROOT/'scripts',check=True)
    after=table_digest(profile);assert after!=before
    path,p=job(directory,build,installed['child'],'restore');restored=wait(path,p,'restored');assert table_digest(profile)==after
    save(directory/'restored.json',restored)
    # Interrupt only our lazy helper while it waits for the genuine idle app.
    # Reopening its waiting intent must refuse replay and leave that app/history intact.
    interrupted_path,helper=job(directory,build,restored['child'],'install')
    deadline=time.monotonic()+10
    while time.monotonic()<deadline:
        interrupted=json.loads(interrupted_path.read_text())
        if interrupted['helper']:break
        time.sleep(.05)
    assert interrupted['helper'] and interrupted['stage']=='waiting',interrupted
    stop(interrupted['helper']);helper.wait(timeout=10)
    assert identity(restored['child']['pid'])==restored['child']
    assert table_digest(profile)==after
    launcher=profile/'repairs'/('build-'+build['id'])/'previous/dolores-update-launcher.exe'
    retry=subprocess.run([str(launcher),'--handoff',str(interrupted_path)],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
    assert retry.returncode!=0
    save(directory/'interrupted.json',interrupted)
    stop(restored['child'])
    path,p=job(directory,build,restored['child'],'install',wrong_source=True);rolled=wait(path,p,'rolledBack');assert table_digest(profile)==after
    save(directory/'rolled-back.json',rolled)
    # A used intent cannot be manually replayed with the same launcher command.
    launcher=profile/'repairs'/('build-'+build['id'])/'previous/dolores-update-launcher.exe'
    retry=subprocess.run([str(launcher),'--handoff',str(path)],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert retry.returncode!=0
    report={'normalStartup':'applied','reviewedCodeRestore':'restored','postInstallDraftPreserved':True,'interruptedWaitingHelper':'old app and history preserved; replay refused','failedStartup':'rolledBack','allHistoryPreserved':True,'sameIntentReplay':'refused','uiReviewAndGracefulClose':'not automated; separate acceptance gate','child':rolled['child']}
    save(directory/'handoff-report.json',report);print(json.dumps({k:v for k,v in report.items() if k!='child'}))

if __name__=='__main__':main(Path(sys.argv[1]).resolve())
