"""Explicit isolated same-engine window trial; held backend is a recorded result."""
import json
import ctypes
from ctypes import wintypes as w
import os
from pathlib import Path
import statistics
import subprocess
import sys
import time
import uuid
from desktop_resource_probe import children, cpu_seconds, memory

ROOT = Path(__file__).resolve().parents[1]


def visible_windows(pid):
    user = ctypes.WinDLL('user32', use_last_error=True)
    callback_type = ctypes.WINFUNCTYPE(w.BOOL, w.HWND, w.LPARAM)
    user.EnumWindows.argtypes = [callback_type, w.LPARAM]
    user.GetWindowThreadProcessId.argtypes = [w.HWND, ctypes.POINTER(w.DWORD)]
    user.IsWindowVisible.argtypes = [w.HWND]
    found = []
    def scan(hwnd, _):
        owner = w.DWORD()
        user.GetWindowThreadProcessId(hwnd, ctypes.byref(owner))
        if owner.value == pid and user.IsWindowVisible(hwnd):
            found.append(int(hwnd))
        return True
    user.EnumWindows(callback_type(scan), 0)
    return len(found)


def main():
    directory = ROOT/'output/window-qualification'/uuid.uuid4().hex
    directory.mkdir(parents=True)
    record = directory/'process.json'
    env = dict(os.environ, DOLORES_WINDOW_SMOKE_DIR=str(directory))
    command = [sys.executable, str(ROOT/'scripts/desktop.py')]
    subprocess.run(command+['launch', '--allow-diagnostic', '--data-directory', str(directory/'data'), '--pid-file', str(record)], env=env, check=True)
    pid = json.loads(record.read_text())['pid']
    samples = {}; max_visible = 0
    try:
        deadline = time.monotonic()+90
        while time.monotonic()<deadline:
            file = directory/'stage.json'
            if file.exists():
                report = json.loads(file.read_text())
                stage = report['stage']
                if stage in ['complete', 'held']:
                    break
                max_visible = max(max_visible, visible_windows(pid))
                samples.setdefault(stage, []).append({'time': time.monotonic(), 'cpu': cpu_seconds(pid), **memory(pid)})
            time.sleep(.2)
        else:
            raise AssertionError('Window diagnostic did not complete within its deadline')
        for stage, values in samples.items():
            values = [v for v in values if v['time'] >= values[-1]['time']-4]
            report.setdefault('measurements', {})[stage] = {
                'privateMiB': round(statistics.median(v['privateBytes'] for v in values)/1048576, 2),
                'cpuOneCorePercent': round((values[-1]['cpu']-values[0]['cpu'])/(values[-1]['time']-values[0]['time'])*100, 3) if len(values)>1 else None,
            }
        if report.get('secondViewCreated'):
            base = report['measurements']['baseline']; full = report['measurements']['two-window-idle']
            report['addedPrivateMiB'] = round(full['privateMiB']-base['privateMiB'], 2)
            report['addedIdleCpuOneCorePercent'] = round(full['cpuOneCorePercent']-base['cpuOneCorePercent'], 3)
            report['resourceGatePassed'] = report['addedPrivateMiB'] <= 128 and report['addedIdleCpuOneCorePercent'] < 1
        report['remainingOwnedDescendants'] = len(children(pid, names=None))
        report['maxVisibleNativeWindows'] = max_visible
        report['finalVisibleNativeWindows'] = visible_windows(pid)
        (directory/'report.json').write_text(json.dumps(report, indent=2, ensure_ascii=False), encoding='utf-8')
        print(json.dumps({'directory': str(directory), **report}, ensure_ascii=False))
        assert report['primaryUsable'] and report['draftRetained'], 'Failed trial lost primary work'
        assert report['remainingOwnedDescendants'] == 0, 'Trial created an independent host/process'
        if report.get('passed'):
            assert max_visible >= 2, 'Flutter views did not establish two visible native windows'
    finally:
        subprocess.run(command+['stop-owned', '--pid-file', str(record)], check=True)


if __name__ == '__main__':
    main()
