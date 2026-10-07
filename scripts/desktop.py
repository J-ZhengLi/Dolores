"""Build and launch the normal Flutter desktop app; Python 3.11+, no PowerShell."""
import argparse
from contextlib import contextmanager
import ctypes
from ctypes import wintypes as w
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import sys
import time
import uuid

ROOT = Path(__file__).resolve().parents[1]
APP = ROOT / 'apps/dolores_flutter'
BUILD_STATE = APP / 'build/.dolores-entry.json'


def require_normal_build(allow_diagnostic=False):
    if not BUILD_STATE.is_file():
        raise ValueError('Build identity missing. Rebuild with python scripts/desktop.py build')
    state = json.loads(BUILD_STATE.read_text(encoding='utf-8'))
    binary = executable()
    if state.get('size') != binary.stat().st_size or state.get('modifiedNs') != binary.stat().st_mtime_ns:
        raise ValueError('Build identity changed. Rebuild with python scripts/desktop.py build')
    if state.get('entry') != 'main' and not allow_diagnostic:
        raise ValueError('Diagnostic build cannot serve as the normal app. Run python scripts/desktop.py build')
    return state['entry']


def platform_name():
    return {'win32': 'windows', 'linux': 'linux', 'darwin': 'macos'}[sys.platform]


def executable():
    return APP / {
        'windows': 'build/windows/x64/runner/Release/dolores_flutter.exe',
        'linux': 'build/linux/x64/release/bundle/dolores_flutter',
        'macos': 'build/macos/Build/Products/Release/dolores_flutter.app/Contents/MacOS/dolores_flutter',
    }[platform_name()]


def run(argv, cwd=ROOT, env=None, capture=False):
    result = subprocess.run([str(v) for v in argv], cwd=cwd, env=env,
                            text=True, encoding='utf-8', errors='replace',
                            stdout=subprocess.PIPE if capture else None,
                            stderr=subprocess.STDOUT if capture else None)
    if capture:
        print(result.stdout, end='', flush=True)
    return result


def checked(argv, **kwargs):
    result = run(argv, **kwargs)
    result.check_returncode()
    return result


def flutter_path(sdk=None):
    selected = sdk or os.environ.get('FLUTTER_SDK')
    local = ROOT / 'output/toolchains/flutter'
    if not selected and local.is_dir():
        selected = local
    name = 'flutter.bat' if os.name == 'nt' else 'flutter'
    path = Path(selected).resolve() / 'bin' / name if selected else shutil.which(name)
    if not path or not Path(path).is_file():
        raise ValueError('Flutter SDK missing. Set FLUTTER_SDK or pass --flutter-sdk.')
    return str(path)


def plugin_links(app=APP):
    metadata = json.loads((app / '.flutter-plugins-dependencies').read_text())
    for platform in ('windows', 'linux') if os.name == 'nt' else (platform_name(),):
        directory = app / platform / 'flutter/ephemeral/.plugin_symlinks'
        directory.mkdir(parents=True, exist_ok=True)
        if not directory.resolve().is_relative_to(app.resolve()):
            raise ValueError('Plugin link directory is outside the app.')
        for plugin in metadata['plugins'].get(platform, []):
            name = plugin['name']
            if not name or not name[0].islower() or any(c not in 'abcdefghijklmnopqrstuvwxyz0123456789_' for c in name):
                raise ValueError('Invalid plugin name.')
            target = Path(plugin['path']).resolve(strict=True)
            link = directory / name
            if os.path.lexists(link):
                is_link = link.is_symlink() or bool(getattr(link.lstat(), 'st_file_attributes', 0) & 0x400)
                if not is_link or link.resolve() != target:
                    raise ValueError(f'Unexpected existing plugin link: {name}. Retained for review.')
            elif os.name == 'nt':
                # CPython's junction helper does not require Developer Mode/elevation.
                import _winapi
                _winapi.CreateJunction(str(target), str(link))
            else:
                link.symlink_to(target, target_is_directory=True)


@contextmanager
def registrant_alias(app=APP):
    path = app / '.dart_tool/package_config.json'
    original = path.read_bytes()
    config = json.loads(original)
    packages = config['packages']
    if any(p['name'] == 'dolores_build_registrant' for p in packages):
        raise ValueError('Registrant alias already exists. Review the generated package config.')
    language = next(p['languageVersion'] for p in packages if p['name'] == 'dolores_flutter')
    packages.append({'name': 'dolores_build_registrant', 'rootUri': 'flutter_build/',
                     'packageUri': './', 'languageVersion': language})
    try:
        path.write_text(json.dumps(config), encoding='utf-8')
        yield
    finally:
        path.write_bytes(original)


def build(args):
    platform = platform_name()
    flutter = flutter_path(args.flutter_sdk)
    # Invalidate before work: a failed or partial build cannot inherit normal identity.
    BUILD_STATE.unlink(missing_ok=True)
    env = dict(os.environ, FLUTTER_SUPPRESS_ANALYTICS='true', DART_SUPPRESS_ANALYTICS='true')
    encoded = env.get('CARGO_ENCODED_RUSTFLAGS')
    flags = encoded.split('\x1f') if encoded else shlex.split(env.get('RUSTFLAGS', ''))
    for path, replacement in [(ROOT, '/dolores'), (Path.home(), '/build-user'),
                              (env.get('CARGO_HOME'), '/cargo'), (env.get('RUSTUP_HOME'), '/rustup')]:
        if path:
            for prefix in dict.fromkeys([str(path), str(path).replace('\\', '/')]):
                flags += ['--remap-path-prefix', f'{prefix}={replacement}']
    rust_env = dict(env, CARGO_ENCODED_RUSTFLAGS='\x1f'.join(flags))
    rust = ['cargo', 'build', '-p', 'dolores-flutter-bridge']
    if platform == 'windows':
        rust += ['-p', 'dolores-desktop-helper', '-p', 'dolores-native-update']
    checked(rust + ['--release', '--locked'], env=rust_env)
    base = [flutter, '--no-version-check', '--suppress-analytics']
    dependencies = run(base + ['pub', 'get'], cwd=APP, env=env, capture=True)
    if dependencies.returncode and 'Building with plugins requires symlink support' in dependencies.stdout:
        plugin_links()
        checked(base + ['pub', 'get'], cwd=APP, env=env)
    else:
        dependencies.check_returncode()
    plugin_links()
    entry = {'smoke': 'smoke', 'restart': 'restart_smoke', 'history': 'history_smoke', 'workspace': 'workspace_smoke', 'source-control': 'source_control_smoke', 'terminal': 'terminal_smoke', 'windows': 'window_smoke'}.get(args.diagnostic, 'main')
    symbols = ROOT / 'output/release-symbols' / uuid.uuid4().hex
    command = base + ['build', platform, '--release', '--no-pub',
                      f'--split-debug-info={symbols}', '--target', f'lib/{entry}.dart']
    cmake = args.cmake
    local_cmake = ROOT / 'output/toolchains/cmake-4.4.3-windows-x86_64/bin/cmake.exe'
    if platform == 'windows' and not cmake and local_cmake.is_file():
        cmake = str(local_cmake)
    with registrant_alias():
        # Invalidate only generated asset stamps, retaining compiler caches.
        cache = APP / '.dart_tool/flutter_build'
        for stamp in cache.glob(f'*/release_bundle_{platform}*_assets.stamp'):
            stamp.unlink()
        if platform == 'windows' and cmake:
            configured = run(command + ['--config-only'], cwd=APP, env=env, capture=True)
            if configured.returncode and 'Unable to find suitable Visual Studio toolchain' not in configured.stdout:
                configured.check_returncode()
            if not (APP / 'windows/flutter/ephemeral/generated_config.cmake').is_file():
                raise ValueError('Flutter did not generate build configuration.')
            checked([cmake, '-S', 'windows', '-B', 'build/windows/x64', '-G', args.generator,
                     '-A', 'x64', '-DFLUTTER_TARGET_PLATFORM=windows-x64'], cwd=APP, env=env)
            checked([cmake, '--build', 'build/windows/x64', '--config', 'Release', '--target', 'INSTALL'], cwd=APP, env=env)
        else:
            checked(command, cwd=APP, env=env)
    destination = executable().parent
    library = {'windows': 'dolores_flutter_bridge.dll', 'linux': 'libdolores_flutter_bridge.so',
               'macos': 'libdolores_flutter_bridge.dylib'}[platform]
    library_dir = destination if platform == 'windows' else destination / 'lib' if platform == 'linux' else destination.parent / 'Frameworks'
    shutil.copy2(ROOT / 'target/release' / library, library_dir / library)
    if platform == 'windows':
        shutil.copy2(ROOT / 'target/release/dolores-desktop-helper.exe', destination)
        shutil.copy2(ROOT / 'target/release/dolores-update-launcher.exe', destination)
    stat = executable().stat()
    BUILD_STATE.write_text(json.dumps({'entry': entry, 'size': stat.st_size,
                                      'modifiedNs': stat.st_mtime_ns}), encoding='utf-8')
    print(f'Built {entry} desktop entry: {executable()} (use the complete bundle).')


def process_identity(pid, terminate_expected=None):
    """Match Windows PID, binary and creation time before replacing an owned preview."""
    if os.name != 'nt':
        raise ValueError('Owned preview replacement currently supports Windows only.')
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    kernel.OpenProcess.argtypes = [w.DWORD, w.BOOL, w.DWORD]
    kernel.OpenProcess.restype = w.HANDLE
    kernel.CloseHandle.argtypes = [w.HANDLE]
    handle = kernel.OpenProcess(0x1000 | (1 if terminate_expected else 0), False, pid)
    if not handle:
        if ctypes.get_last_error() == 87:
            return None
        raise OSError('Cannot verify owned preview process.')
    try:
        exit_code = w.DWORD()
        get_exit_code = kernel.GetExitCodeProcess
        get_exit_code.argtypes = [w.HANDLE, ctypes.POINTER(w.DWORD)]
        if not get_exit_code(handle, ctypes.byref(exit_code)):
            raise OSError('Cannot verify owned preview state.')
        if exit_code.value != 259:  # STILL_ACTIVE
            return None
        buffer = ctypes.create_unicode_buffer(32768)
        size = w.DWORD(len(buffer))
        query = kernel.QueryFullProcessImageNameW
        query.argtypes = [w.HANDLE, w.DWORD, w.LPWSTR, ctypes.POINTER(w.DWORD)]
        times = [w.FILETIME() for _ in range(4)]
        get_times = kernel.GetProcessTimes
        get_times.argtypes = [w.HANDLE] + [ctypes.POINTER(w.FILETIME)] * 4
        if not query(handle, 0, buffer, ctypes.byref(size)) or not get_times(handle, *(ctypes.byref(v) for v in times)):
            raise OSError('Cannot read owned preview identity.')
        identity = {'pid': pid, 'executable': str(Path(buffer.value).resolve()),
                    'created': (times[0].dwHighDateTime << 32) + times[0].dwLowDateTime}
        if terminate_expected:
            if identity != terminate_expected or Path(identity['executable']) != executable().resolve():
                raise ValueError('Owned preview identity changed. Process left running.')
            terminate = kernel.TerminateProcess
            terminate.argtypes = [w.HANDLE, w.UINT]
            if not terminate(handle, 0):
                raise OSError('Could not stop the verified owned preview.')
        return identity
    finally:
        kernel.CloseHandle(handle)


def stop_owned(record):
    saved = json.loads(record.read_text())
    current = process_identity(saved['pid'])
    if current is None:
        return
    if current != saved or Path(current['executable']) != executable().resolve():
        raise ValueError('Owned preview identity changed. Process left running.')
    # Recheck and terminate through the same process handle, avoiding PID reuse.
    process_identity(saved['pid'], terminate_expected=saved)


def window_visible(pid, timeout=15):
    if os.name != 'nt':
        return None
    user = ctypes.WinDLL('user32', use_last_error=True)
    callback_type = ctypes.WINFUNCTYPE(w.BOOL, w.HWND, w.LPARAM)
    user.EnumWindows.argtypes = [callback_type, w.LPARAM]
    user.GetWindowThreadProcessId.argtypes = [w.HWND, ctypes.POINTER(w.DWORD)]
    user.IsWindowVisible.argtypes = [w.HWND]
    def scan(hwnd, _):
        owner = w.DWORD()
        user.GetWindowThreadProcessId(hwnd, ctypes.byref(owner))
        if owner.value == pid and user.IsWindowVisible(hwnd):
            found.append(int(hwnd))
        return True
    callback = callback_type(scan)
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        found = []
        user.EnumWindows(callback, 0)
        if found:
            return found[0]
        time.sleep(.2)
    raise RuntimeError('Dolores did not expose a visible window. Inspect the launch before retrying.')


def launch(args):
    binary = executable()
    if not binary.is_file():
        raise ValueError('Build the normal desktop release first: python scripts/desktop.py build')
    entry = require_normal_build(args.allow_diagnostic)
    if args.replace_owned and not args.pid_file:
        raise ValueError('--replace-owned requires an explicit --pid-file.')
    env = dict(os.environ)
    env.pop('DOLORES_SMOKE_DIR', None)
    if args.data_directory:
        path = Path(args.data_directory)
        if not path.is_absolute():
            raise ValueError('Choose an absolute application data directory.')
        env['DOLORES_DATA_DIR'] = str(path.resolve())
    record = Path(args.pid_file).resolve() if args.pid_file else None
    if record and not record.is_relative_to(ROOT / 'output'):
        raise ValueError('Owned preview records must stay under ignored output/.')
    if record and record.exists():
        if not args.replace_owned:
            raise ValueError('Preview record already exists. Review it or use --replace-owned.')
        stop_owned(record)
    app = subprocess.Popen([str(binary)], cwd=binary.parent, env=env,
                           stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                           start_new_session=os.name != 'nt')
    if record:
        record.parent.mkdir(parents=True, exist_ok=True)
        record.write_text(json.dumps(process_identity(app.pid)), encoding='utf-8')
    visible = window_visible(app.pid)
    if app.poll() is not None:
        raise RuntimeError('Dolores exited before opening its window.')
    print(json.dumps({'processId': app.pid, 'windowVisible': visible is not None,
                      'entry': entry,
                      'visibilityCheck': 'Windows window presence; inspect foreground separately' if os.name == 'nt' else 'Native visual verification required'}))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    builder = commands.add_parser('build')
    builder.add_argument('--flutter-sdk')
    builder.add_argument('--cmake')
    builder.add_argument('--generator', default='Visual Studio 18 2026')
    builder.add_argument('--diagnostic', choices=['smoke', 'restart', 'history', 'workspace', 'source-control', 'terminal', 'windows'])
    starter = commands.add_parser('launch')
    starter.add_argument('--data-directory')
    starter.add_argument('--pid-file')
    starter.add_argument('--replace-owned', action='store_true')
    starter.add_argument('--allow-diagnostic', action='store_true', help='Explicit diagnostic testing only; not a normal-app handoff.')
    stopper = commands.add_parser('stop-owned')
    stopper.add_argument('--pid-file', required=True, type=Path)
    args = parser.parse_args()
    try:
        if args.command == 'build':
            build(args)
        elif args.command == 'launch':
            launch(args)
        else:
            stop_owned(args.pid_file)
    except (OSError, ValueError, RuntimeError, subprocess.CalledProcessError) as error:
        parser.exit(1, f'{error}\n')


if __name__ == '__main__':
    main()
