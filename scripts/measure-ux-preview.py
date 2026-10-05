"""Launch the normal Windows app with an owned synthetic profile and sample resources.

Reports observations, not an added-cost comparison or a low-end qualification.
Leaves the app visible for native review. Never reads the user's profile.
"""
import argparse
import ctypes
from ctypes import wintypes as w
import json
import os
from pathlib import Path
import subprocess
import time
from desktop_resource_probe import ResourceProbe
from desktop_test_support import ROOT


def cpu_seconds(pid):
    kernel = ctypes.windll.kernel32
    kernel.OpenProcess.restype = w.HANDLE
    kernel.CloseHandle.argtypes = [w.HANDLE]
    handle = kernel.OpenProcess(0x1000, False, pid)
    if not handle:
        raise RuntimeError('Owned preview exited during sampling')
    try:
        values = [w.FILETIME() for _ in range(4)]
        fn = kernel.GetProcessTimes
        fn.argtypes = [w.HANDLE] + [ctypes.POINTER(w.FILETIME)] * 4
        if not fn(handle, *(ctypes.byref(v) for v in values)):
            raise OSError('Process time query failed')
        return sum((v.dwHighDateTime << 32) + v.dwLowDateTime for v in values[2:]) / 10_000_000
    finally:
        kernel.CloseHandle(handle)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--directory', required=True, type=Path)
    args = p.parse_args()
    directory = args.directory.resolve()
    if os.name != 'nt' or not directory.is_relative_to(ROOT / 'output'):
        raise ValueError('Use a disposable Windows preview under output/')
    if not (directory / 'data' / 'dolores.db').is_file():
        raise ValueError('Prepare the synthetic preview first')
    executable = ROOT / 'apps/dolores_flutter/build/windows/x64/runner/Release/dolores_flutter.exe'
    env = dict(os.environ, DOLORES_DATA_DIR=str(directory / 'data'),
               DOLORES_GLOBAL_SKILLS_DIR=str(directory / 'skills'))
    env.pop('DOLORES_SMOKE_DIR', None)
    started = time.monotonic()
    app = subprocess.Popen([str(executable)], cwd=executable.parent, env=env)
    (directory / 'preview.pid').write_text(str(app.pid))
    time.sleep(3)
    if app.poll() is not None:
        raise RuntimeError('Normal preview failed to start')
    warmup = time.monotonic() - started
    before = cpu_seconds(app.pid)
    probe = ResourceProbe(app.pid).start()
    sample_started = time.monotonic()
    time.sleep(15)
    resources = probe.finish()
    elapsed = time.monotonic() - sample_started
    cpu = cpu_seconds(app.pid) - before
    resources['limit'] = 'Normal Flutter release; sampled peaks may miss short-lived helpers.'
    result = {'normalRelease': True, 'syntheticProfile': True,
              'aliveAfterWarmupSeconds': round(warmup, 2),
              'sampleSeconds': round(elapsed, 2), 'cpuSeconds': round(cpu, 3),
              'cpuPercentOneCore': round(cpu / elapsed * 100, 2), 'resources': resources,
              'limitations': 'Alive-after-warmup is not first-frame latency. Record concurrent activity separately before calling this idle cost. No matched pre-milestone baseline, other OS or low-end host.'}
    (directory / 'public-performance.json').write_text(json.dumps(result, indent=2))
    print(json.dumps(result))


if __name__ == '__main__':
    main()
