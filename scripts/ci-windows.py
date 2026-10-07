"""Credential-free Windows CI checks and an allowlisted unsigned package."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

import desktop
import release

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / 'output/ci-windows'
FLUTTER_REVISION = '6a19cca56475dbfba1478ee68d7bd0c2ef891da1'


def run(args, cwd=ROOT):
    subprocess.run([str(a) for a in args], cwd=cwd, check=True)


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


def tag():
    return os.environ.get('GITHUB_REF_NAME') if os.environ.get('GITHUB_REF_TYPE') == 'tag' else None


def sdk():
    return Path(desktop.flutter_path()).parent.parent


def check(generator='Visual Studio 17 2022', cmake=None):
    OUTPUT.mkdir(parents=True, exist_ok=True)
    release.check(tag=tag())
    run([sys.executable, ROOT / 'scripts/check-publication.py'])
    run([sys.executable, ROOT / 'scripts/check-docs.py'])
    for name in ('test-publication.py', 'test-release.py', 'test-package-windows.py', 'test-windows-launcher.py', 'test-regression-runner.py'):
        run([sys.executable, ROOT / 'scripts' / name])
    run(['cargo', 'fmt', '--all', '--check'])
    run(['cargo', 'clippy', '--workspace', '--all-targets', '--locked', '--', '-D', 'warnings'])
    run(['cargo', 'test', '--workspace', '--locked'])
    run([shutil.which('npm') or 'npm', 'ci', '--ignore-scripts', '--no-audit', '--no-fund'], ROOT / 'adapters/browser')
    run(['node', '--test', 'adapters/browser/worker.test.cjs'])
    run([sys.executable, ROOT / 'scripts/test-browser-installer.py'])
    flutter = desktop.flutter_path()
    run([flutter, '--no-version-check', '--suppress-analytics', 'precache', '--windows'])
    version = json.loads((sdk() / 'bin/cache/flutter.version.json').read_text())
    if version['frameworkRevision'] != FLUTTER_REVISION:
        raise ValueError('Use the pinned Flutter 3.47.5 source revision for CI packages.')
    build = [sys.executable, ROOT / 'scripts/desktop.py', 'build', '--flutter-sdk', sdk(), '--generator', generator]
    if cmake:
        build += ['--cmake', cmake]
    run(build)
    app = ROOT / 'apps/dolores_flutter'
    run([flutter, '--no-version-check', '--suppress-analytics', 'analyze', '--no-pub'], app)
    run([flutter, '--no-version-check', '--suppress-analytics', 'test', '--no-pub'], app)
    run([sys.executable, ROOT / 'scripts/run-regressions.py', '--native-only', '--directory', OUTPUT / 'regressions', '--flutter-sdk', sdk()])


def package():
    desktop.require_normal_build()
    version = release.check(tag=tag())
    portable = module('portable_ci', ROOT / 'scripts/package-windows.py')
    collector = module('notices_ci', ROOT / 'scripts/collect-windows-notices.py')
    runtime = OUTPUT / 'runtime'
    notices = OUTPUT / 'notices'
    destination = OUTPUT / 'package'
    if any(p.exists() for p in (runtime, notices, destination)):
        raise ValueError('CI package outputs already exist. Use a fresh checkout/output directory.')
    # Optional browser adapters are installed separately and do not enter this ZIP.
    bundle = portable.BUNDLE
    for p in bundle.rglob('*'):
        relative = p.relative_to(bundle).as_posix()
        if portable.is_link(p):
            raise ValueError('Build links cannot enter a release.')
        if p.is_file() and relative not in portable.RUNTIME and not relative.startswith('browser-adapter/'):
            raise ValueError('Unexpected build member: ' + relative)
    runtime.mkdir(parents=True)
    for name in sorted(portable.RUNTIME):
        source = bundle / name
        target = runtime / name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, target)
    collector.collect(sdk(), notices, runtime)
    archive = portable.package(runtime, destination, notices)
    portable.verify_archive(archive)
    note_version = version if tag() else 'Unreleased'
    (destination / 'RELEASE-NOTES.md').write_text(release.notes(ROOT, note_version), encoding='utf-8')
    checksum = archive.with_suffix('.zip.sha256').read_text().split()[0]
    assert hashlib.sha256(archive.read_bytes()).hexdigest() == checksum
    print(json.dumps({'ok': True, 'version': version, 'signed': False, 'published': False,
                      'archive': archive.name, 'runtimeFiles': len(portable.RUNTIME)}))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['check', 'package'])
    parser.add_argument('--generator', default='Visual Studio 17 2022', help='Installed CMake generator; CI uses Visual Studio 2022.')
    parser.add_argument('--cmake', help='Optional local CMake executable; hosted CI uses the runner toolchain.')
    args = parser.parse_args()
    try:
        if os.name != 'nt':
            raise ValueError('The release workflow supports Windows x64 only.')
        if args.command == 'check':
            check(args.generator, args.cmake)
        else:
            package()
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        parser.exit(1, 'Windows CI refused: ' + str(error) + '\n')
