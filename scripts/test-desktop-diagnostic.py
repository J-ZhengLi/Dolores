"""Run a built Flutter smoke/history/restart diagnostic in a fresh ignored profile.

Build with desktop.py build --diagnostic <kind> first. Rebuild normally afterward.
This runner does not satisfy the normal visible-app verification requirement.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import time
from desktop import ROOT, executable, require_normal_build


def run_phase(directory, env, report_name, timeout, compact=False):
    directory.mkdir(parents=True, exist_ok=True)
    env = dict(env, DOLORES_SMOKE_DIR=str(directory))
    process = subprocess.Popen([str(executable()), *(['--compact'] if compact else [])],
                               cwd=executable().parent, env=env,
                               stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                               creationflags=0x08000000 if os.name == 'nt' else 0)
    try:
        report = directory / report_name; deadline = time.monotonic() + timeout
        while not report.is_file():
            if process.poll() is not None:
                raise RuntimeError('Diagnostic exited before producing its report.')
            if time.monotonic() >= deadline:
                raise RuntimeError('Diagnostic deadline reached; rebuild the matching entry before retrying.')
            time.sleep(.2)
        result = json.loads(report.read_text(encoding='utf-8'))
        if not result.get('ok'):
            raise RuntimeError('Diagnostic reported failure. Inspect its ignored local report.')
        return result
    finally:
        if process.poll() is None:
            process.terminate()
        process.wait(timeout=5)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('kind', choices=['smoke', 'history', 'restart'])
    parser.add_argument('--directory', type=Path, required=True)
    parser.add_argument('--compact', action='store_true')
    parser.add_argument('--fixture-url', default='http://127.0.0.1:19421/v1')
    parser.add_argument('--timeout', type=int, default=40)
    args = parser.parse_args()
    expected = {'smoke':'smoke', 'history':'history_smoke', 'restart':'restart_smoke'}[args.kind]
    if require_normal_build(True) != expected:
        parser.error(f'Build the {args.kind} diagnostic first; normal apps are never used as diagnostics.')
    directory = args.directory.resolve()
    if not args.directory.is_absolute() or not directory.is_relative_to(ROOT / 'output') or directory.exists():
        parser.error('Choose a fresh absolute directory under ignored output/.')
    if not 10 <= args.timeout <= 120:
        parser.error('Choose a 10–120 second deadline.')
    env = dict(os.environ, DOLORES_DATA_DIR=str(directory / 'data'),
               DOLORES_GLOBAL_SKILLS_DIR=str(directory / 'skills'),
               DOLORES_SMOKE_PROVIDER=args.fixture_url)
    if args.kind == 'history':
        subprocess.run(['node', str(ROOT / 'scripts/seed-history-fixture.mjs'), str(directory / 'data')], check=True)
    if args.kind == 'restart':
        saved = False; forgotten = False; results = []
        try:
            for phase in range(1, 5):
                saved = True
                results.append(run_phase(directory, dict(env, DOLORES_RESTART_PHASE=str(phase)),
                                         f'phase-{phase}.json', args.timeout))
                if phase == 3:
                    forgotten = True
            for path in (directory / 'data').iterdir():
                if path.is_file() and b'dolores-generated-restart-test' in path.read_bytes():
                    raise RuntimeError('Generated test key appeared in a data file.')
            result = {'ok':True, 'phases':results, 'keyAbsentFromDataFiles':True}
        finally:
            if saved and not forgotten:
                run_phase(directory / 'cleanup', dict(env, DOLORES_RESTART_PHASE='3'), 'phase-3.json', args.timeout)
    else:
        result = run_phase(directory, env, 'report.json', args.timeout, args.compact)
        if args.kind == 'smoke' and abs(result['logicalWidth'] - (620 if args.compact else 1120)) > 1:
            raise RuntimeError('Unexpected diagnostic client width.')
    (directory / 'report.json').write_text(json.dumps(result, indent=2), encoding='utf-8')
    print(json.dumps({'ok':True, 'diagnostic':args.kind, 'normalHandoff':False}))


if __name__ == '__main__':
    main()
