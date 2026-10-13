"""Diagnostic Inno lifecycle: isolated app ID, shortcuts, install path and synthetic data."""
import argparse
import ctypes
import json
from pathlib import Path
import subprocess
import time
import uuid
import zipfile

import installer


def run(command, expected=0):
    result = subprocess.run(list(map(str, command)), timeout=120)
    if expected == 0 and result.returncode != 0:
        raise ValueError('Installer fixture command failed; inspect retained logs.')
    if expected != 0 and result.returncode == 0:
        raise ValueError('Installer fixture unexpectedly accepted unsafe data removal.')


def exercise(archive, directory, compiler):
    root = installer.fresh(directory)
    installer.module('package-windows').verify_archive(archive)
    root.mkdir(parents=True)
    with zipfile.ZipFile(archive) as bundle:
        bundle.extractall(root / 'payload')
    data = root / 'synthetic-user-data'
    data.mkdir()
    sentinel = data / 'retain.txt'
    sentinel.write_text('PUBLIC SYNTHETIC UNINSTALL FIXTURE')
    identity = str(uuid.uuid4())
    version = installer.release.app_version()
    run([compiler, '/Qp', '/DAppVersion=' + version,
         '/DNumericVersion=' + version.split('-')[0] + '.0',
         '/DInstallerAppId=' + identity, '/DShortcutName=Dolores Fixture ' + identity,
         '/DUninstallFixtureDirectory=' + str(data),
         '/DPayloadDirectory=' + str(root / 'payload/Dolores'), '/DOutputDirectory=' + str(root),
         installer.ROOT / 'packaging/windows/dolores.iss'])
    setup = root / f'Dolores-{version}-windows-x64-setup.exe'
    installed = root / 'installed'
    def install(case):
        run([setup, '/CURRENTUSER', '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', '/SP-',
             '/DIR=' + str(installed), '/TASKS=', '/LOG=' + str(root / (case + '-install.log'))])
        if not (installed / 'dolores_flutter.exe').is_file():
            raise ValueError('Fixture installation is incomplete.')
    def uninstall(case, purge=False, refused=False):
        command = [installed / 'unins000.exe', '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART',
                   '/LOG=' + str(root / (case + '-uninstall.log'))]
        if purge:
            command += ['/REMOVEFIXTUREDATA=1']
        run(command, expected=1 if refused else 0)
        if not refused:
            # The uninstaller's temporary copy can finish file removal after its parent returns.
            deadline = time.monotonic() + 10
            while (installed / 'dolores_flutter.exe').exists() and time.monotonic() < deadline:
                time.sleep(0.1)
            if (installed / 'dolores_flutter.exe').exists():
                raise ValueError('Fixture application was not removed.')
            if purge:
                while data.exists() and time.monotonic() < deadline:
                    time.sleep(0.1)
    install('retention')
    uninstall('retention')
    assert sentinel.read_text() == 'PUBLIC SYNTHETIC UNINSTALL FIXTURE'
    install('locked')
    (data / 'dolores.lock').write_text('public fixture lock')
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    kernel.CreateFileW.argtypes = [ctypes.c_wchar_p, ctypes.c_uint32, ctypes.c_uint32, ctypes.c_void_p,
                                   ctypes.c_uint32, ctypes.c_uint32, ctypes.c_void_p]
    kernel.CreateFileW.restype = ctypes.c_void_p
    kernel.CloseHandle.argtypes = [ctypes.c_void_p]
    handle = kernel.CreateFileW(str(data / 'dolores.lock'), 0xC0000000, 0, None, 3, 0x80, None)
    if handle == ctypes.c_void_p(-1).value:
        raise ValueError('Could not establish fixture data lock.')
    try:
        uninstall('locked', purge=True, refused=True)
        assert sentinel.is_file() and (installed / 'dolores_flutter.exe').is_file()
    finally:
        kernel.CloseHandle(handle)
    outside = root / 'outside-project'
    outside.mkdir()
    (outside / 'retain.txt').write_text('PUBLIC EXTERNAL PROJECT FIXTURE')
    import _winapi
    link = data / 'linked-project'
    _winapi.CreateJunction(str(outside), str(link))
    uninstall('linked', purge=True, refused=True)
    assert sentinel.is_file() and (outside / 'retain.txt').is_file()
    # Remove only this exact fixture junction, never traverse its external target.
    link.rmdir()
    uninstall('purge-recovery', purge=True)
    assert not data.exists() and (outside / 'retain.txt').is_file()
    result = {'ok': True, 'diagnosticInstaller': True, 'productionCheckboxInteraction': False,
              'cases': ['current-user installation', 'silent data retention', 'locked-data refusal',
                        'junction refusal', 'explicit synthetic purge recovery']}
    (root / 'RESULT.json').write_text(json.dumps(result, indent=2), encoding='utf-8')
    print(json.dumps(result))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path, required=True)
    parser.add_argument('--directory', type=Path, required=True)
    parser.add_argument('--compiler', default=str(installer.ROOT / 'output/toolchains/inno-6.7.3/ISCC.exe'))
    args = parser.parse_args()
    exercise(args.archive, args.directory, args.compiler)
