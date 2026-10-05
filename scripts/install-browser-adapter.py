"""Install Dolores's pinned optional browser adapter, without PowerShell or browser downloads."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'adapters/browser'
FILES = ('package.json', 'package-lock.json')


def tools():
    node = shutil.which('node')
    npm = shutil.which('npm.cmd' if os.name == 'nt' else 'npm')
    if not node or not npm:
        raise ValueError('Install Node 20 or newer, then run this command again.')
    version = subprocess.run([node, '--version'], capture_output=True, text=True, check=True).stdout.strip()
    if int(version.lstrip('v').split('.')[0]) < 20:
        raise ValueError('Node 20 or newer is required.')
    # Invoke the CLI with Node directly: no shell wrapper or child terminal.
    cli = Path(npm).parent / 'node_modules/npm/bin/npm-cli.js' if os.name == 'nt' else Path(npm).resolve()
    if not cli.is_file():
        raise ValueError('npm CLI missing. Repair the Node installation and try again.')
    return node, cli


def install(destination, node, npm_cli, runner=subprocess.run):
    destination = Path(destination).absolute()
    if destination.is_symlink() or destination.resolve() == SOURCE.resolve():
        raise ValueError('Choose a separate directory; source and linked targets are retained.')
    manifests = {name: (SOURCE / name).read_bytes() for name in FILES}
    pinned = json.loads(manifests['package.json'])['dependencies']['playwright-core']
    if destination.exists():
        try:
            matching = all((destination / name).read_bytes() == contents for name, contents in manifests.items())
            version = json.loads((destination / 'node_modules/playwright-core/package.json').read_bytes())['version']
            if matching and version == pinned:
                return 'Already installed'
        except (OSError, ValueError, KeyError):
            pass
        raise ValueError('Destination already contains other or incomplete files. Retained; choose a new empty destination.')
    destination.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='.dolores-browser-', dir=destination.parent) as temporary:
        staging = Path(temporary) / 'adapter'; staging.mkdir()
        for name, contents in manifests.items():
            (staging / name).write_bytes(contents)
        user_config = Path(temporary) / 'user.npmrc'; user_config.write_text('')
        global_config = Path(temporary) / 'global.npmrc'; global_config.write_text('')
        env = {key: value for key, value in os.environ.items()
               if key != 'NODE_OPTIONS' and not key.upper().startswith(('NPM_CONFIG_', 'DOLORES_'))}
        try:
            result = runner([str(node), str(npm_cli), 'ci', '--ignore-scripts', '--no-audit', '--no-fund',
                             '--registry=https://registry.npmjs.org', f'--userconfig={user_config}', f'--globalconfig={global_config}'],
                            cwd=staging, env=env, timeout=90, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        except subprocess.TimeoutExpired as error:
            raise ValueError('Installation timed out. Destination was not changed; check the network and explicitly try again.') from error
        if result.returncode:
            raise ValueError('Installation failed. Destination was not changed; check the network and explicitly try again.')
        version = json.loads((staging / 'node_modules/playwright-core/package.json').read_bytes())['version']
        if version != pinned:
            raise ValueError('Installed adapter version differs from the pinned version. Nothing activated.')
        if destination.exists():
            raise ValueError('Destination appeared during installation. Retained; choose a new destination.')
        staging.rename(destination)
    return 'Installed'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--destination', type=Path)
    args = parser.parse_args()
    try:
        destination = args.destination
        if destination is None:
            spec = importlib.util.spec_from_file_location('desktop', Path(__file__).with_name('desktop.py'))
            desktop = importlib.util.module_from_spec(spec); spec.loader.exec_module(desktop)
            destination = desktop.executable().parent / 'browser-adapter'
        node, npm = tools()
        print(install(destination, node, npm) + '. Refresh Browser in Dolores. Edge/Chrome must already be installed; no browser or existing profile is downloaded or used.')
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        parser.exit(1, f'{error}\n')


if __name__ == '__main__':
    main()
