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


def check(generator='Visual Studio 17 2022', cmake=None, full=False):
    OUTPUT.mkdir(parents=True, exist_ok=True)
    # Check both source identity and usable SDK version before expensive compilation.
    flutter = desktop.flutter_path()
    run([flutter, '--no-version-check', '--suppress-analytics', '--version'])
    version = json.loads((sdk() / 'bin/cache/flutter.version.json').read_text())
    if version.get('frameworkRevision') != FLUTTER_REVISION:
        raise ValueError('Use the pinned Flutter 3.47.5 source revision for CI packages.')
    if version.get('frameworkVersion') != '3.47.5':
        raise ValueError('Flutter SDK version must be 3.47.5. Fetch the SDK release tags '
                         '(checkout fetch-depth: 0), then use a fresh SDK checkout/cache and rerun CI.')
    release.check(tag=tag())
    run([sys.executable, ROOT / 'scripts/check-publication.py'])
    run([sys.executable, ROOT / 'scripts/check-docs.py'])
    for name in ('test-ci-windows.py', 'test-local-ci.py', 'test-rust-timing.py', 'test-publication.py', 'test-release.py', 'test-installer.py', 'test-package-windows.py', 'test-windows-launcher.py', 'test-regression-runner.py'):
        run([sys.executable, ROOT / 'scripts' / name])
    run(['cargo', 'fmt', '--all', '--check'])
    run(['cargo', 'clippy', '--workspace', '--all-targets', '--locked', '--', '-D', 'warnings'])
    run(['cargo', 'test', '--workspace', '--locked', '--', '--test-threads=4'])
    run([sys.executable, ROOT / 'scripts/check-rust-timing.py'])
    run([sys.executable, ROOT / 'scripts/check-rust-timing.py', '--suite', 'mcp', '--software-hash', '--repeat', '8', '--jobs', '4'])
    run([sys.executable, ROOT / 'scripts/check-rust-timing.py', '--suite', 'mcp-protocol', '--software-hash', '--repeat', '4', '--jobs', '2'])
    run([shutil.which('npm') or 'npm', 'ci', '--ignore-scripts', '--no-audit', '--no-fund'], ROOT / 'adapters/browser')
    run(['node', '--test', 'adapters/browser/worker.test.cjs'])
    run([sys.executable, ROOT / 'scripts/test-browser-installer.py'])
    app = ROOT / 'apps/dolores_flutter'
    if full:
        run([flutter, '--no-version-check', '--suppress-analytics', 'precache', '--windows'])
        build = [sys.executable, ROOT / 'scripts/desktop.py', 'build', '--flutter-sdk', sdk(), '--generator', generator]
        if cmake:
            build += ['--cmake', cmake]
        run(build)
    else:
        run([flutter, '--no-version-check', '--suppress-analytics', 'pub', 'get'], app)
    run([flutter, '--no-version-check', '--suppress-analytics', 'analyze', '--no-pub'], app)
    run([flutter, '--no-version-check', '--suppress-analytics', 'test', '--no-pub'], app)
    if full:
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
    entries = release.sections((ROOT / 'CHANGELOG.md').read_text(encoding='utf-8'))
    note_version = version if tag() or (version in entries and not entries['Unreleased'][1]) else 'Unreleased'
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
    parser.add_argument('--full', action='store_true', help='Build the normal app and run native fixtures; release/manual builds only.')
    parser.add_argument('--directory', type=Path, default=OUTPUT,
                        help='Absolute output location under output/; use a fresh location when recovering a local run.')
    args = parser.parse_args()
    try:
        OUTPUT = args.directory.resolve()
        if not args.directory.is_absolute() or OUTPUT == (ROOT / 'output').resolve() or not OUTPUT.is_relative_to((ROOT / 'output').resolve()):
            raise ValueError('Use an absolute output directory strictly beneath output/.')
        if os.name != 'nt':
            raise ValueError('The release workflow supports Windows x64 only.')
        if args.command == 'check':
            check(args.generator, args.cmake, args.full)
        else:
            package()
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        parser.exit(1, 'Windows CI refused: ' + str(error) + '\n')
