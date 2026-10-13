"""Install the pinned build-only Inno compiler under ignored output/toolchains/."""
import hashlib
from pathlib import Path
import subprocess
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
VERSION = '6.7.3'
# Published installer digest in Microsoft's WinGet manifest for this version.
DIGEST = '9c73c3bae7ed48d44112a0f48e66742c00090bdb5bef71d9d3c056c66e97b732'
URL = 'https://github.com/jrsoftware/issrc/releases/download/is-6_7_3/innosetup-6.7.3.exe'

if __name__ == '__main__':
    directory = ROOT / 'output/toolchains/inno-6.7.3'
    if (directory / 'ISCC.exe').is_file():
        print('Pinned Inno Setup compiler already available.')
    else:
        bootstrap = ROOT / 'output/toolchains/inno-bootstrap'
        bootstrap.mkdir(parents=True, exist_ok=True)
        archive = bootstrap / 'innosetup-6.7.3.exe'
        if not archive.exists():
            with urllib.request.urlopen(URL, timeout=60) as response, archive.open('xb') as target:
                while chunk := response.read(1024 * 1024):
                    target.write(chunk)
        if hashlib.sha256(archive.read_bytes()).hexdigest() != DIGEST:
            raise SystemExit('Inno download hash differs. Retained for review; remove this exact download before retrying.')
        subprocess.run([str(archive), '/CURRENTUSER', '/VERYSILENT', '/SUPPRESSMSGBOXES',
                        '/NORESTART', '/SP-', '/DIR=' + str(directory)], check=True, timeout=120)
        if not (directory / 'ISCC.exe').is_file():
            raise SystemExit('Inno compiler missing after setup; inspect output/toolchains/inno-bootstrap.')
        print('Pinned Inno Setup compiler installed.')
