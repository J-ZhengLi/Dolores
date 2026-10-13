"""Build Windows installers or experimental macOS/Linux packages on their native host."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import zipfile

import desktop
import release

ROOT = Path(__file__).resolve().parents[1]


def module(name):
    spec = importlib.util.spec_from_file_location(name.replace('-', '_'), ROOT / 'scripts' / (name + '.py'))
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


def fresh(directory):
    if not directory.is_absolute():
        raise ValueError('Use an absolute fresh directory beneath output/.')
    output = (ROOT / 'output').resolve()
    destination = directory.resolve()
    if destination == output or not destination.is_relative_to(output):
        raise ValueError('Use an absolute fresh directory beneath output/.')
    if destination.exists():
        raise ValueError('Destination exists. Keep previous artifacts and choose a fresh directory.')
    return destination


def checksum(asset):
    if not asset.is_file() or asset.stat().st_size == 0:
        raise ValueError('Installer compiler produced no package. Inspect its log and retry in a fresh directory.')
    with asset.open('rb') as stream:
        digest = hashlib.file_digest(stream, 'sha256').hexdigest()
    asset.with_suffix(asset.suffix + '.sha256').write_text(digest + '  ' + asset.name + '\n', encoding='utf-8')
    return asset


def windows(archive, directory, compiler):
    directory = fresh(directory)
    compiler = Path(compiler).resolve()
    if not compiler.is_file():
        raise ValueError('Inno Setup compiler missing. Run scripts/setup-inno.py or pass --compiler ISCC.exe.')
    portable = module('package-windows')
    manifest = portable.verify_archive(archive)
    version = release.app_version()
    expected = re.search(r'^version: (' + release.VERSION + r'\+[1-9][0-9]*)$',
                         (ROOT / 'apps/dolores_flutter/pubspec.yaml').read_text(), re.M)[1]
    if manifest['version'] != expected:
        raise ValueError('Archive version differs from source. Rebuild and collect matching notices before retrying.')
    directory.mkdir(parents=True)
    stage = directory / 'staging'
    # The verifier above requires an exact set of safe regular-file member paths.
    with zipfile.ZipFile(archive) as bundle:
        bundle.extractall(stage)
    payload = stage / 'Dolores'
    asset = directory / f'Dolores-{version}-windows-x64-setup.exe'
    subprocess.run([str(compiler), '/Qp', '/DAppVersion=' + version,
                    '/DNumericVersion=' + version.split('-')[0] + '.0',
                    '/DPayloadDirectory=' + str(payload), '/DOutputDirectory=' + str(directory),
                    str(ROOT / 'packaging/windows/dolores.iss')], check=True)
    if asset.read_bytes()[:2] != b'MZ':
        raise ValueError('Compiler output is not a Windows executable.')
    return checksum(asset)


def native_runtime(bundle):
    """Reject escaping links/private artifacts; preserve internal framework symlinks."""
    root = bundle.resolve()
    if bundle.is_symlink():
        raise ValueError('Bundle root must not be a link.')
    files = {}
    for base, dirs, names in os.walk(bundle, followlinks=False):
        for name in dirs + names:
            path = Path(base) / name
            relative = path.relative_to(bundle).as_posix()
            if name in ('.git', '.env') or name.endswith(('.db', '.db-wal', '.db-shm', '.pdb')):
                raise ValueError('Unexpected private/debug artifact in bundle: ' + relative)
            if path.is_symlink():
                if not path.resolve().is_relative_to(root):
                    raise ValueError('Bundle link escapes the runtime: ' + relative)
                files[relative] = 'symlink:' + os.readlink(path)
            elif path.is_file():
                with path.open('rb') as stream:
                    files[relative] = hashlib.file_digest(stream, 'sha256').hexdigest()
    if not files:
        raise ValueError('Runtime bundle is empty.')
    return files


def native_notices(bundle, notices, target):
    inventory = json.loads((notices / 'DEPENDENCIES.json').read_text(encoding='utf-8'))
    text = (notices / 'THIRD-PARTY-NOTICES.txt').read_bytes()
    if inventory.get('platform') != target or inventory.get('runtime') != native_runtime(bundle):
        raise ValueError('Notices do not match this platform/runtime. Collect native notices before packaging.')
    if inventory.get('noticesSha256') != hashlib.sha256(text).hexdigest() or not inventory.get('components'):
        raise ValueError('Dependency notices are incomplete or stale.')
    collector = module('collect-windows-notices')
    expected = {name: collector.digest(ROOT / name) for name in collector.INPUTS}
    if inventory.get('inputs') != expected:
        raise ValueError('Dependency inputs changed. Recollect native notices before packaging.')


def native(kind, directory, notices):
    host = {'macos': 'darwin', 'linux': 'linux'}[kind]
    if sys.platform != host:
        raise ValueError('Build this experimental installer on its native operating system.')
    directory = fresh(directory)
    desktop.require_normal_build()
    executable = desktop.executable()
    bundle = executable.parents[2] if kind == 'macos' else executable.parent
    machine = platform.machine().lower()
    arch = {'x86_64': 'x64', 'amd64': 'x64', 'aarch64': 'arm64', 'arm64': 'arm64'}.get(machine)
    if not arch:
        raise ValueError('Unsupported installer architecture.')
    native_notices(bundle, notices, kind + '-' + arch)
    tool = shutil.which('pkgbuild' if kind == 'macos' else 'dpkg-deb')
    if not tool:
        raise ValueError('Install pkgbuild (Xcode tools) or dpkg-deb on this native host, then retry.')
    directory.mkdir(parents=True)
    stage = directory / 'staging'
    version = release.app_version()
    if kind == 'macos':
        app = stage / 'Applications/Dolores.app'
        shutil.copytree(bundle, app, symlinks=True)
        documents = app / 'Contents/Resources/Dolores-notices'
        asset = directory / f'Dolores-{version}-macos-{arch}.pkg'
        command = [tool, '--root', str(stage), '--identifier', 'dev.dolores.desktop',
                   '--version', version.split('-')[0], '--install-location', '/', str(asset)]
    else:
        app = stage / 'opt/dolores'
        shutil.copytree(bundle, app, symlinks=True)
        documents = stage / 'usr/share/doc/dolores'
        desktop_file = stage / 'usr/share/applications/dolores.desktop'
        desktop_file.parent.mkdir(parents=True)
        desktop_file.write_text('[Desktop Entry]\nType=Application\nName=Dolores\nExec=/opt/dolores/dolores_flutter\nIcon=dolores\nCategories=Development;\nTerminal=false\n')
        icon = stage / 'usr/share/icons/hicolor/scalable/apps/dolores.svg'
        icon.parent.mkdir(parents=True)
        shutil.copyfile(ROOT / 'assets/dolores.svg', icon)
        deb_arch = 'amd64' if arch == 'x64' else 'arm64'
        control = stage / 'DEBIAN/control'
        control.parent.mkdir()
        control.write_text(f'Package: dolores\nVersion: {version.replace("-", "~", 1)}\nArchitecture: {deb_arch}\n'
                           'Maintainer: Dolores contributors\nDepends: libgtk-3-0, libsecret-1-0, libstdc++6\n'
                           'Description: Experimental Dolores desktop assistant\n')
        asset = directory / f'Dolores-{version}-linux-{arch}.deb'
        command = [tool, '--build', '--root-owner-group', str(stage), str(asset)]
    documents.mkdir(parents=True)
    for name in ('DEPENDENCIES.json', 'THIRD-PARTY-NOTICES.txt'):
        shutil.copyfile(notices / name, documents / name)
    for name in ('LICENSE', 'CHANGELOG.md'):
        shutil.copyfile(ROOT / name, documents / name)
    subprocess.run(command, check=True)
    return checksum(asset)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('platform', choices=('windows', 'macos', 'linux'))
    parser.add_argument('--directory', type=Path, required=True)
    parser.add_argument('--archive', type=Path, help='Verified Windows portable payload; retained as internal packaging input.')
    parser.add_argument('--compiler', default=str(ROOT / 'output/toolchains/inno-6.7.3/ISCC.exe'))
    parser.add_argument('--notices', type=Path, help='Native runtime-bound dependency notice collection for macOS/Linux.')
    args = parser.parse_args()
    try:
        if args.platform == 'windows':
            if os.name != 'nt' or not args.archive:
                raise ValueError('Windows installer needs Windows and --archive from the verified normal build.')
            asset = windows(args.archive, args.directory, args.compiler)
        else:
            if not args.notices:
                raise ValueError('Experimental native installer needs --notices for its exact runtime.')
            asset = native(args.platform, args.directory, args.notices)
        print(json.dumps({'ok': True, 'installer': str(asset), 'signed': False}))
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError, zipfile.BadZipFile) as error:
        parser.exit(1, 'Installer refused: ' + str(error) + '\n')


if __name__ == '__main__':
    main()
