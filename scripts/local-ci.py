"""Run the Windows CI checks or installer pipeline locally, without a GitHub tag."""
import argparse
import ctypes
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]


def affinity(cpus):
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    kernel.GetCurrentProcess.restype = ctypes.c_void_p
    kernel.GetProcessAffinityMask.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_size_t), ctypes.POINTER(ctypes.c_size_t)]
    kernel.SetProcessAffinityMask.argtypes = [ctypes.c_void_p, ctypes.c_size_t]
    process = kernel.GetCurrentProcess()
    allowed, system = ctypes.c_size_t(), ctypes.c_size_t()
    if not kernel.GetProcessAffinityMask(process, ctypes.byref(allowed), ctypes.byref(system)):
        raise ctypes.WinError(ctypes.get_last_error())
    bits = [1 << n for n in range(ctypes.sizeof(allowed) * 8) if allowed.value & (1 << n)]
    if cpus > len(bits):
        raise ValueError(f'Only {len(bits)} CPUs are available; reduce --cpus.')
    if not kernel.SetProcessAffinityMask(process, sum(bits[:cpus])):
        raise ctypes.WinError(ctypes.get_last_error())


def output(path):
    resolved = path.resolve()
    if not path.is_absolute() or not resolved.is_relative_to((ROOT / 'output').resolve()) or resolved == (ROOT / 'output').resolve():
        raise ValueError('Use an absolute directory strictly beneath output/.')
    resolved.mkdir(parents=True, exist_ok=False)
    return resolved


def run(args, directory, env, steps):
    log = directory / f'{len(steps) + 1:02d}.log'
    step = {'command': list(map(str, args)), 'log': log.name, 'status': 'running'}
    steps.append(step)
    started = time.monotonic()
    print(f'Running {Path(str(args[0])).name}; log: {log}', flush=True)
    with log.open('w', encoding='utf-8') as stream:
        result = subprocess.run(list(map(str, args)), cwd=ROOT, env=env, stdout=stream, stderr=subprocess.STDOUT)
    step.update(status='passed' if result.returncode == 0 else 'failed', exitCode=result.returncode,
                elapsedSeconds=round(time.monotonic() - started, 3))
    if result.returncode:
        raise ValueError(f'Step failed ({result.returncode}). Inspect {log}; fix the cause and start a fresh run.')


def execute(args):
    if os.name != 'nt':
        raise ValueError('This pipeline requires Windows x64 and the Windows desktop toolchain.')
    if not 1 <= args.cpus <= 64:
        raise ValueError('Use 1-64 CPUs.')
    affinity(args.cpus)
    directory = output(args.directory)
    report = {'status': 'running', 'mode': args.command, 'host': platform.platform(),
              'cpus': args.cpus, 'githubHostedImageEmulated': False, 'published': False, 'steps': []}
    env = dict(os.environ, RUSTUP_TOOLCHAIN='1.95.0', FLUTTER_SUPPRESS_ANALYTICS='true',
               DART_SUPPRESS_ANALYTICS='true', PYTHONUTF8='1', CARGO_TERM_COLOR='never')
    # A local release build must never infer publication or an old tag from its shell.
    for name in ('GITHUB_REF_TYPE', 'GITHUB_REF_NAME'):
        env.pop(name, None)
    if args.flutter_sdk:
        env['FLUTTER_SDK'] = str(args.flutter_sdk.resolve())
    try:
        if sys.version_info[:2] != (3, 13):
            raise ValueError('Use Python 3.13, matching GitHub Actions.')
        node = subprocess.check_output(['node', '--version'], env=env, text=True).strip()
        if not node.startswith('v24.'):
            raise ValueError('Use Node 24, matching GitHub Actions.')
        rust = subprocess.check_output(['rustc', '--version'], env=env, text=True).strip()
        if not rust.startswith('rustc 1.95.0 '):
            raise ValueError('Install Rust 1.95.0 with rustfmt and Clippy.')
        report['tools'] = {'python': platform.python_version(), 'node': node, 'rust': rust}
        report['revision'] = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
        report['workingTreeDirty'] = bool(subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT, text=True).strip())
        # Isolate the pinned history-audit dependency; never silently skip that fixture.
        run([sys.executable, '-m', 'venv', directory / 'venv'], directory, env, report['steps'])
        python = directory / 'venv/Scripts/python.exe'
        run([python, '-m', 'pip', '--isolated', 'install', '--require-hashes', '-r', ROOT / '.github/requirements-ci.txt'], directory, env, report['steps'])
        common = ['--directory', directory / 'pipeline', '--generator', args.generator]
        if args.cmake:
            common += ['--cmake', args.cmake]
        check = [python, ROOT / 'scripts/ci-windows.py', 'check', *common]
        if args.command == 'release':
            check += ['--full']
        run(check, directory, env, report['steps'])
        if args.command == 'release':
            run([python, ROOT / 'scripts/ci-windows.py', 'package', *common], directory, env, report['steps'])
            run([python, ROOT / 'scripts/setup-inno.py'], directory, env, report['steps'])
            payloads = list((directory / 'pipeline/package').glob('*.zip'))
            if len(payloads) != 1:
                raise ValueError('Expected exactly one verified installer payload.')
            run([python, ROOT / 'scripts/installer.py', 'windows', '--archive', payloads[0], '--directory', directory / 'pipeline/installer'], directory, env, report['steps'])
            shutil.copyfile(directory / 'pipeline/package/RELEASE-NOTES.md', directory / 'pipeline/installer/RELEASE-NOTES.md')
            run([python, ROOT / 'scripts/test-installer-native.py', '--archive', payloads[0], '--directory', directory / 'pipeline/installer-fixture'], directory, env, report['steps'])
            run([python, ROOT / 'scripts/test-installer-native.py', '--archive', payloads[0], '--directory', directory / 'pipeline/installer-fixture-rapid', '--rapid-reinstall'], directory, env, report['steps'])
        report['status'] = 'passed'
    except KeyboardInterrupt:
        report['status'] = 'interrupted'
        raise
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        report['status'] = 'failed'
        report['failure'] = str(error)
        raise
    finally:
        (directory / 'RESULT.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
        print(f"Local CI {report['status']}: {directory / 'RESULT.json'}", flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['checks', 'release'])
    parser.add_argument('--cpus', type=int, default=4, help='Restrict this pipeline and children to this many CPUs; use 2 for stronger contention.')
    parser.add_argument('--directory', type=Path, default=ROOT / 'output/local-ci' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ'))
    parser.add_argument('--flutter-sdk', type=Path, help='Pinned Flutter 3.47.5 checkout including release tags.')
    parser.add_argument('--generator', default='Visual Studio 17 2022', help='Use your installed generator; hosted CI uses Visual Studio 17 2022.')
    parser.add_argument('--cmake', help='Optional installed CMake executable.')
    args = parser.parse_args()
    try:
        execute(args)
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        parser.exit(1, str(error) + '\n')
