"""Windows regression runner: fresh data, bounded stages, explicit coverage gaps."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import uuid

ROOT = Path(__file__).resolve().parents[1]
CASES = ('task-feedback', 'context-comparisons', 'continuation', 'generation-profiles',
         'coding-repair', 'exact-edits', 'automatic-memory', 'skill-drafts')

def validate_receipt(log, stage):
    # RTK adds a banner; the fixture's final nonblank line must be its receipt.
    with log.open('rb') as stream:
        stream.seek(max(0, log.stat().st_size - 16384))
        lines = stream.read().decode('utf-8', errors='replace').splitlines()
    value = json.loads(next(line for line in reversed(lines) if line.strip()))
    if not isinstance(value, dict) or value.get('ok') is not True or value.get('stage') != stage:
        raise ValueError('Fixture did not report successful completion of the requested stage.')
    if type(value.get('liveRequests')) is not int or value['liveRequests'] != 0:
        raise ValueError('Fixture must explicitly report zero live requests.')
    if type(value.get('fixtureRequests')) is not int or value['fixtureRequests'] < 0:
        raise ValueError('Fixture request count is missing or invalid.')
    return value

def stop_process(process):
    if os.name == 'nt':
        # Only the subprocess owned by this runner and its current descendants.
        stopped = subprocess.run(['taskkill','/PID',str(process.pid),'/T','/F'],
                                 stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=10)
        if stopped.returncode and process.poll() is None: process.kill()
    else:
        import signal
        os.killpg(process.pid, signal.SIGKILL)
    process.wait(timeout=10)

def run_step(name, argv, cwd, logs, timeout, stage=None):
    result = {'name':name, 'status':'failed', 'deadlineSeconds':timeout,
              'log':str(Path('logs') / (name+'.log'))}
    started = time.monotonic()
    process = None
    log = logs / (name+'.log')
    with log.open('wb') as stream:
        try:
            process = subprocess.Popen(argv, cwd=cwd, stdout=stream, stderr=subprocess.STDOUT,
                                       start_new_session=os.name!='nt')
            result['exitCode'] = process.wait(timeout=timeout)
            if result['exitCode'] == 0:
                if stage: result['receipt'] = validate_receipt(log, stage)
                result['status'] = 'passed'
            else: result['recovery'] = 'Inspect the retained log, fix the failure, then start a fresh run.'
        except subprocess.TimeoutExpired:
            stop_process(process)
            result['status'] = 'timedOut'
            result['recovery'] = 'The owned process tree was stopped. Inspect retained evidence and retry explicitly in a fresh directory.'
        except KeyboardInterrupt:
            if process is not None and process.poll() is None: stop_process(process)
            result['status'] = 'interrupted'
            result['recovery'] = 'Earlier reports and fixture data remain. Start a new run to repeat checks.'
        except (OSError, ValueError, StopIteration) as error:
            result['recovery'] = f'{type(error).__name__}: stage or receipt was invalid. Inspect the retained log before a fresh run.'
    result['elapsedSeconds'] = round(time.monotonic()-started, 3)
    return result

def write_report(directory, report):
    report['ok'] = all(s['status']=='passed' for s in report['steps']) and len(report['steps'])==report['expectedSteps']
    temporary = directory/'report.pending'
    temporary.write_text(json.dumps(report, indent=2), encoding='utf-8')
    os.replace(temporary, directory/'report.json')
    lines = ['# Dolores selected regression results', '',
             f"Completed selected scope: {report['ok']}", f"Mode: {report['mode']}", '',
             '| Check | Status | Seconds |', '| --- | --- | --- |']
    lines += [f"| {s['name']} | {s['status']} | {s['elapsedSeconds']} |" for s in report['steps']]
    lines += ['', 'Not acceptance: live model competence, native keyboard/IME/accessibility, macOS/Linux, low-end resources, packaging/signing.',
              'Synthetic fixture data and full logs remain local. Failed/unfinished checks do not authorize instruction promotion.',
              'Recovery: inspect the failed stage log, repair its cause, then run again with a fresh output directory. No automatic retry.']
    lines += [''] + [f"{s['name']}: {s['recovery']}" for s in report['steps'] if 'recovery' in s]
    (directory/'report.md').write_text('\n'.join(lines)+'\n', encoding='utf-8')

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', type=Path, help='Fresh absolute directory strictly beneath output/.')
    parser.add_argument('--native-only', action='store_true', help='Explicitly skip build/lint/unit checks; report existing bundle hash, no freshness claim.')
    parser.add_argument('--case', action='append', choices=CASES, dest='cases', help='Run only selected fixture pairs; default is all eight.')
    parser.add_argument('--flutter-sdk', type=Path, default=ROOT/'output/toolchains/flutter')
    args = parser.parse_args()
    if os.name != 'nt': parser.error('This native bundle runner is currently Windows-only.')
    if args.directory is not None and not args.directory.is_absolute(): parser.error('Use an absolute output directory.')
    directory = args.directory or ROOT/'output/regressions'/('run-'+uuid.uuid4().hex[:12])
    directory = directory.resolve()
    output = (ROOT/'output').resolve()
    if directory == output or not directory.is_relative_to(output): parser.error('Use a fresh directory strictly beneath output/.')
    if directory.exists(): parser.error('Directory exists. Choose a fresh run; previous evidence is retained.')
    directory.mkdir(parents=True)
    logs = directory/'logs'; logs.mkdir()
    cases = list(dict.fromkeys(args.cases or CASES))
    flutter = (args.flutter_sdk/'bin/flutter.bat').resolve()
    steps = []
    if not args.native_only:
        steps = [
            ('runner-tests',[sys.executable,'-I','-B',str(ROOT/'scripts/test-regression-runner.py')],ROOT,60,None),
            ('rust-format',['cargo','fmt','--all','--check'],ROOT,60,None),
            ('rust-lint',['cargo','clippy','--workspace','--all-targets','--locked','--','-D','warnings'],ROOT,300,None),
            ('rust-tests',['cargo','test','--workspace','--locked'],ROOT,300,None),
            ('normal-build',[sys.executable,str(ROOT/'scripts/desktop.py'),'build','--flutter-sdk',str(args.flutter_sdk.resolve())],ROOT,900,None),
            ('flutter-analysis',[str(flutter),'--no-version-check','--suppress-analytics','analyze','--no-pub'],ROOT/'apps/dolores_flutter',120,None),
            ('flutter-tests',[str(flutter),'--no-version-check','--suppress-analytics','test','--no-pub'],ROOT/'apps/dolores_flutter',180,None),
        ]
    for case in cases:
        for stage in ('save','restore'):
            steps.append((case+'-'+stage,[sys.executable,'-I','-B',str(ROOT/'scripts'/('test-'+case+'.py')),stage,'--directory',str(directory/case)],ROOT,120,stage))
    report = {'version':1, 'createdAt':datetime.now(timezone.utc).isoformat(),
              'mode':'existing-bundle-native-only' if args.native_only else 'build-and-selected-regressions',
              'cases':cases, 'expectedSteps':len(steps), 'steps':[], 'liveModelTests':'not run',
              'coverageGaps':['general model competence','native input/accessibility','macOS/Linux','low-end resources','packaging/signing']}
    # Working-copy identities make a local report traceable even before a commit.
    report['sources'] = {}
    for source in [Path(__file__), ROOT/'scripts/test-regression-runner.py', ROOT/'scripts/desktop.py',
                   ROOT/'Cargo.lock', ROOT/'apps/dolores_flutter/pubspec.lock',
                   *[ROOT/'scripts'/('test-'+case+'.py') for case in cases]]:
        with source.open('rb') as stream:
            report['sources'][str(source.relative_to(ROOT))] = hashlib.file_digest(stream,'sha256').hexdigest()
    write_report(directory,report)
    os.environ['FLUTTER_SUPPRESS_ANALYTICS'] = 'true'
    os.environ['DART_SUPPRESS_ANALYTICS'] = 'true'
    bundle = ROOT/'apps/dolores_flutter/build/windows/x64/runner/Release'
    for name,argv,cwd,timeout,stage in steps:
        # No inherited user data or global skills. Every fixture receives explicit fresh paths.
        os.environ['DOLORES_DATA_DIR'] = str(directory/'runner-data')
        os.environ['DOLORES_GLOBAL_SKILLS_DIR'] = str(directory/'runner-skills')
        if stage and 'bundle' not in report:
            bridge = bundle/'dolores_flutter_bridge.dll'
            if not bridge.exists():
                report['steps'].append({'name':name,'status':'blocked','elapsedSeconds':0,'recovery':'Build the normal Windows bundle first.'})
                write_report(directory,report); break
            with bridge.open('rb') as stream:
                report['bundle'] = {'bridgeSha256':hashlib.file_digest(stream,'sha256').hexdigest(),
                                    'freshBuild':not args.native_only}
        print(f'Checking {name}...', flush=True)
        result = run_step(name,argv,cwd,logs,timeout,stage)
        report['steps'].append(result); write_report(directory,report)
        print(f"{name}: {result['status']}", flush=True)
        if result['status'] != 'passed': break
    print(json.dumps({'ok':report['ok'],'completedSteps':len(report['steps']),'expectedSteps':len(steps),
                      'report':str(directory/'report.json'),'liveRequests':0}))
    return 0 if report['ok'] else 1

if __name__ == '__main__': sys.exit(main())
