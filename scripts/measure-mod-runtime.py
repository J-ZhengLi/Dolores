"""Sample prebuilt experimental Wasm/Rhai probes on Windows; not app qualification."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
from desktop import ROOT
from desktop_resource_probe import memory


def main():
    if os.name != 'nt':
        raise SystemExit('Windows memory observation only.')
    for name in ('wasm_probe', 'rhai_probe'):
        binary = ROOT / f'target/release/examples/{name}.exe'
        if not binary.is_file():
            raise SystemExit('Build the mod-runtime release examples first.')
        samples = []
        for _ in range(3):
            process = subprocess.Popen([str(binary)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, creationflags=0x08000000)
            try:
                time.sleep(.2); samples.append(memory(process.pid))
                if process.wait(timeout=15):
                    raise RuntimeError('Runtime probe failed.')
            finally:
                if process.poll() is None:
                    process.kill(); process.wait()
        print(json.dumps({'runtime':name, 'exeBytes':binary.stat().st_size, 'samples':samples,
                          'limitation':'Point-in-time probe samples, not peak or normal desktop memory.'}))
    with tempfile.TemporaryDirectory(prefix='dolores-worker-') as directory:
        fixture = Path(directory) / 'boundary.txt'; fixture.write_text('worker-can-read')
        subprocess.run([sys.executable, str(ROOT / 'scripts/mod-worker-probe.py'), str(fixture)], check=True, timeout=15)


if __name__ == '__main__':
    main()
