"""Qualify general UIA targets against an owned Win32 form, without a model.

The fixture contains public controls only. Never selects or inputs into another
app. Tests actual capture, target dispatch, changed-control refusal and recovery.
"""
import argparse
import ctypes
from ctypes import wintypes as w
import json
from pathlib import Path
import subprocess
import sys
import threading
import time

ROOT = Path(__file__).resolve().parents[1]
TITLE = 'Public accessibility fixture'


def fixture(directory):
    user = ctypes.windll.user32
    kernel = ctypes.windll.kernel32
    callback = ctypes.WINFUNCTYPE(ctypes.c_ssize_t, w.HWND, w.UINT, w.WPARAM, w.LPARAM)

    class WindowClass(ctypes.Structure):
        _fields_ = [('style', w.UINT), ('procedure', callback), ('classExtra', ctypes.c_int),
                    ('windowExtra', ctypes.c_int), ('instance', w.HINSTANCE), ('icon', w.HICON),
                    ('cursor', w.HANDLE), ('background', w.HBRUSH), ('menu', w.LPCWSTR), ('name', w.LPCWSTR)]

    user.CreateWindowExW.restype = w.HWND
    user.CreateWindowExW.argtypes = [w.DWORD, w.LPCWSTR, w.LPCWSTR, w.DWORD, ctypes.c_int, ctypes.c_int,
                                   ctypes.c_int, ctypes.c_int, w.HWND, w.HMENU, w.HINSTANCE, ctypes.c_void_p]
    user.DefWindowProcW.restype = ctypes.c_ssize_t
    user.DefWindowProcW.argtypes = [w.HWND, w.UINT, w.WPARAM, w.LPARAM]
    user.GetWindowTextW.argtypes = [w.HWND, w.LPWSTR, ctypes.c_int]
    user.MoveWindow.argtypes = [w.HWND, ctypes.c_int, ctypes.c_int, ctypes.c_int, ctypes.c_int, w.BOOL]
    user.EnableWindow.argtypes = [w.HWND, w.BOOL]
    user.SetForegroundWindow.argtypes = [w.HWND]
    user.SetFocus.argtypes = [w.HWND]
    user.DestroyWindow.argtypes = [w.HWND]
    user.SetTimer.argtypes = [w.HWND, ctypes.c_size_t, w.UINT, ctypes.c_void_p]
    kernel.GetModuleHandleW.restype = w.HMODULE
    kernel.GetModuleHandleW.argtypes = [w.LPCWSTR]
    user.SetProcessDpiAwarenessContext(ctypes.c_void_p(-4))
    state = {'clicks': 0, 'command': None, 'moved': False}
    controls = {}
    started = time.monotonic()

    def publish():
        buffer = ctypes.create_unicode_buffer(512)
        user.GetWindowTextW(controls['edit'], buffer, len(buffer))
        data = {**state, 'text': buffer.value}
        temporary = directory / 'state.tmp'
        temporary.write_text(json.dumps(data), encoding='utf-8')
        try:
            temporary.replace(directory / 'state.json')
        except PermissionError:
            pass

    @callback
    def procedure(hwnd, message, wp, lp):
        if message == 0x111 and wp & 0xffff == 2:  # WM_COMMAND / public Save button
            state['clicks'] += 1
        elif message == 0x113:  # WM_TIMER
            command = directory / 'command.json'
            if command.is_file():
                try:
                    value = json.loads(command.read_text())
                except (OSError, ValueError):
                    value = None
                if value and value != state['command']:
                    state['command'] = value
                    if value['operation'] == 'move':
                        user.MoveWindow(controls['save'], 320, 180, 200, 40, True)
                        state['moved'] = True
                    elif value['operation'] == 'restore':
                        user.MoveWindow(controls['save'], 40, 180, 200, 40, True)
                        user.SetForegroundWindow(hwnd)
                        user.SetFocus(controls['edit'])
                        state['moved'] = False
                    elif value['operation'] == 'close':
                        user.DestroyWindow(hwnd)
                        return 0
            publish()
            if time.monotonic() - started > 180:
                user.DestroyWindow(hwnd)
        elif message == 2:
            user.PostQuitMessage(0)
            return 0
        return user.DefWindowProcW(hwnd, message, wp, lp)

    instance = kernel.GetModuleHandleW(None)
    wc = WindowClass(0, procedure, 0, 0, instance, None, None, 6, None, 'DoloresPublicUIAFixture')
    assert user.RegisterClassW(ctypes.byref(wc))
    hwnd = user.CreateWindowExW(0, wc.name, TITLE, 0x10cf0000, 80, 80, 640, 360, None, None, instance, None)
    assert hwnd
    child = 0x50000000
    controls['edit'] = user.CreateWindowExW(0x200, 'EDIT', '', child | 0x80, 40, 80, 400, 36, hwnd, 1, instance, None)
    controls['save'] = user.CreateWindowExW(0, 'BUTTON', 'Save note', child | 0x10000, 40, 180, 200, 40, hwnd, 2, instance, None)
    controls['disabled'] = user.CreateWindowExW(0, 'BUTTON', 'Disabled action', child | 0x08000000, 40, 230, 200, 40, hwnd, 3, instance, None)
    controls['secret'] = user.CreateWindowExW(0x200, 'EDIT', '', child | 0x20, 40, 125, 400, 36, hwnd, 4, instance, None)
    user.SetForegroundWindow(hwnd)
    user.SetFocus(controls['edit'])
    user.SetTimer(hwnd, 1, 50, None)
    publish()
    message = w.MSG()
    while user.GetMessageW(ctypes.byref(message), None, 0, 0) > 0:
        user.TranslateMessage(ctypes.byref(message))
        user.DispatchMessageW(ctypes.byref(message))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', required=True, type=Path)
    parser.add_argument('--helper', type=Path, default=ROOT / 'apps/dolores_flutter/build/windows/x64/runner/Release/dolores-desktop-helper.exe')
    parser.add_argument('--fixture', action='store_true')
    args = parser.parse_args()
    directory = args.directory.resolve()
    assert args.directory.is_absolute() and directory.is_relative_to(ROOT / 'output')
    if args.fixture:
        fixture(directory)
        return
    directory.mkdir(parents=True, exist_ok=False)
    child = subprocess.Popen([sys.executable, __file__, '--fixture', '--directory', str(directory)], creationflags=subprocess.CREATE_NO_WINDOW)

    def wait(predicate):
        deadline = time.monotonic() + 8
        while time.monotonic() < deadline:
            try:
                if predicate():
                    return
            except (OSError, ValueError):
                pass
            time.sleep(.05)
        raise AssertionError('Owned fixture did not reach its expected state')

    def state():
        return json.loads((directory / 'state.json').read_text())

    def command(operation):
        value = {'operation': operation, 'id': time.monotonic_ns()}
        (directory / 'command.json').write_text(json.dumps(value))
        wait(lambda: state()['command'] == value)

    def observe(request):
        result = subprocess.run([str(args.helper)], input=json.dumps(request), text=True, capture_output=True, timeout=8, check=True)
        value = json.loads(result.stdout)
        assert value['ok'], value
        return value['result']

    def capture(target):
        observation = observe({'operation': 'capture', 'target': target})
        observation.pop('imageBase64')
        return observation

    def click(target, observation, element):
        request = {'target': target, 'observation': observation, 'action': {'operation': 'click', 'x': element['x'], 'y': element['y']},
                   'element': element, 'hostPid': child.pid, 'automatic': False}
        process = subprocess.Popen([str(args.helper), '--input'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        timer = threading.Timer(8, process.kill)
        timer.daemon = True
        timer.start()
        try:
            process.stdin.write(json.dumps(request) + '\n')
            process.stdin.flush()
            first = json.loads(process.stdout.readline())
            if first.get('ready'):
                process.stdin.write('{"dispatch":true}\n')
                process.stdin.flush()
                result = json.loads(process.stdout.readline())
            else:
                result = first
            process.wait(timeout=8)
            return result
        finally:
            timer.cancel()
            if process.poll() is None:
                process.kill()
                process.wait()

    try:
        wait(lambda: (directory / 'state.json').is_file())
        target = next(w for w in observe({'operation': 'list'})['windows'] if w['title'] == TITLE and w['pid'] == child.pid)
        first = capture(target)
        elements = first['accessibility']['elements']
        save = next(e for e in elements if e['name'] == 'Save note')
        assert save['role'] == 'button'
        assert not any(e['name'] == 'Disabled action' for e in elements)
        assert len([e for e in elements if e['role'] == 'edit']) == 1, 'Password control leaked into targets'
        assert click(target, first, save)['ok']
        wait(lambda: state()['clicks'] == 1)
        second = capture(target)
        old = next(e for e in second['accessibility']['elements'] if e['name'] == 'Save note')
        command('move')
        refused = click(target, second, old)
        assert not refused['ok'] and 'Observe again' in refused['error'], refused
        assert state()['clicks'] == 1
        command('restore')
        recovered = capture(target)
        current = next(e for e in recovered['accessibility']['elements'] if e['name'] == 'Save note')
        assert click(target, recovered, current)['ok']
        wait(lambda: state()['clicks'] == 2)
        receipt = {'passed': True, 'controls': len(elements), 'targetClicks': 2, 'changedControlRefused': True,
                   'recoveryAfterObserve': True, 'disabledAndPasswordExcluded': True, 'providerRequests': 0}
        (directory / 'result.json').write_text(json.dumps(receipt, indent=2))
        print(json.dumps(receipt))
    finally:
        if child.poll() is None:
            (directory / 'command.json').write_text(json.dumps({'operation': 'close', 'id': time.monotonic_ns()}))
            try:
                child.wait(timeout=5)
            except subprocess.TimeoutExpired:
                child.terminate()
                child.wait()


if __name__ == '__main__':
    main()
