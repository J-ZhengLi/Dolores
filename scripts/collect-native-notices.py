"""Experimental macOS/Linux notice collection after a native normal build."""
import argparse
import json
from pathlib import Path
import platform
import sys

import desktop
import installer

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--flutter-sdk', type=Path, required=True)
parser.add_argument('--directory', type=Path, required=True)
if __name__ == '__main__':
    args = parser.parse_args()
    try:
        if sys.platform not in ('darwin', 'linux'):
            raise ValueError('Use this collector on a native macOS/Linux host.')
        installer.fresh(args.directory)
        desktop.require_normal_build()
        kind = 'macos' if sys.platform == 'darwin' else 'linux'
        arch = {'x86_64': 'x64', 'amd64': 'x64', 'aarch64': 'arm64', 'arm64': 'arm64'}[platform.machine().lower()]
        binary = desktop.executable()
        bundle = binary.parents[2] if kind == 'macos' else binary.parent
        result = installer.module('collect-windows-notices').collect(args.flutter_sdk, args.directory, bundle, kind + '-' + arch)
        print(json.dumps({'ok': True, 'platform': result['platform'], 'components': len(result['components'])}))
    except (OSError, ValueError, KeyError) as error:
        parser.exit(1, 'Native notices refused: ' + str(error) + '\n')
