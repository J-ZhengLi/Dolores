"""Diagnostic Inno lifecycle: isolated app ID, shortcuts, install path and synthetic data."""
import argparse
import ctypes
import json
from pathlib import Path
import re
import subprocess
import time
import uuid
import zipfile

import installer


def run(command, expected=0):
    result = subprocess.run(list(map(str, command)), timeout=120)
    if result.returncode != expected:
        raise ValueError(f'Installer fixture expected exit {expected}, got {result.returncode}; inspect retained logs.')


def registered_uninstaller(installed, identity):
    import winreg
    key = 'Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\' + identity + '_is1'
    with winreg.OpenKey(winreg.HKEY_CURRENT_USER, key, 0, winreg.KEY_READ | winreg.KEY_WOW64_64KEY) as entry:
        location = winreg.QueryValueEx(entry, 'InstallLocation')[0]
        command = winreg.QueryValueEx(entry, 'UninstallString')[0]
    return validate_uninstaller(installed, location, command)


def validate_uninstaller(installed, location, command):
    owned = installed.resolve()
    match = re.fullmatch(r'"([^"]+)"|([^"\s]+)', command)
    if not match or Path(location).resolve() != owned:
        raise ValueError('Fixture uninstall registration does not match its owned install directory.')
    executable = Path(match[1] or match[2]).resolve()
    if executable.parent != owned or not re.fullmatch(r'unins[0-9]{3}\.exe', executable.name, re.I):
        raise ValueError('Fixture uninstall registration points outside its owned uninstaller.')
    if not executable.is_file() or not executable.with_suffix('.dat').is_file():
        raise ValueError('Registered fixture uninstaller is incomplete; inspect the install log.')
    return executable


def wait_cleanup(installed, data, purge):
    # Inno's first process exits before the temporary second phase deletes the
    # original uninstaller. Do not reinstall until that entire directory is gone.
    deadline = time.monotonic() + 10
    while (installed.exists() or (purge and data.exists())) and time.monotonic() < deadline:
        time.sleep(0.1)
    if installed.exists() or (purge and data.exists()):
        raise ValueError('Fixture uninstall cleanup is incomplete; retain logs and start a fresh run after fixing the cause.')


def exercise(archive, directory, compiler, rapid_reinstall=False):
    root = installer.fresh(directory)
    installer.module('package-windows').verify_archive(archive)
    root.mkdir(parents=True)
    with zipfile.ZipFile(archive) as bundle:
        bundle.extractall(root / 'payload')
    payload = root / 'payload/Dolores'
    if rapid_reinstall:
        # Diagnostic-only small payload reproduces a reinstall finishing before
        # Inno's delayed deletion of the previous uninstaller. Never a release.
        import shutil
        payload = root / 'rapid-payload'
        payload.mkdir()
        for name in ('LICENSE', 'dolores_flutter.exe', 'Start-Dolores.cmd'):
            shutil.copyfile(root / 'payload/Dolores' / name, payload / name)
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
         '/DPayloadDirectory=' + str(payload), '/DOutputDirectory=' + str(root),
         installer.ROOT / 'packaging/windows/dolores.iss'])
    setup = root / f'Dolores-{version}-windows-x64-setup.exe'
    installed = root / 'installed'
    def install(case):
        run([setup, '/CURRENTUSER', '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', '/SP-',
             '/DIR=' + str(installed), '/TASKS=', '/LOG=' + str(root / (case + '-install.log'))])
        if not (installed / 'dolores_flutter.exe').is_file():
            raise ValueError('Fixture installation is incomplete.')
        executable = registered_uninstaller(installed, identity)
        print(f'{case}: registered fixture uninstaller {executable.name}', flush=True)
    def uninstall(case, purge=False, refused=False):
        executable = registered_uninstaller(installed, identity)
        command = [executable, '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART',
                   '/LOG=' + str(root / (case + '-uninstall.log'))]
        if purge:
            command += ['/REMOVEFIXTUREDATA=1']
        run(command, expected=1 if refused else 0)
        if refused:
            if registered_uninstaller(installed, identity) != executable:
                raise ValueError('Refused uninstall changed its registered executable.')
        else:
            wait_cleanup(installed, data, purge)
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
              'rapidReinstall': rapid_reinstall, 'completePayload': not rapid_reinstall,
              'cases': ['current-user installation', 'silent data retention', 'locked-data refusal',
                        'junction refusal', 'explicit synthetic purge recovery']}
    (root / 'RESULT.json').write_text(json.dumps(result, indent=2), encoding='utf-8')
    print(json.dumps(result))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path, required=True)
    parser.add_argument('--directory', type=Path, required=True)
    parser.add_argument('--compiler', default=str(installer.ROOT / 'output/toolchains/inno-6.7.3/ISCC.exe'))
    parser.add_argument('--rapid-reinstall', action='store_true', help='Diagnostic small payload to exercise the old cleanup/reinstall race; never release this fixture installer.')
    args = parser.parse_args()
    exercise(args.archive, args.directory, args.compiler, args.rapid_reinstall)
