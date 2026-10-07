"""Synthetic release C ABI check: held capability and saved choice across processes."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import uuid
from desktop_test_support import NativeHost, ROOT
from desktop_resource_probe import children


def phase(directory, restore):
    host = NativeHost(directory)
    try:
        view = host.call('experimentalPreferences')
        assert view['multipleWindowCapability']['available'] is False
        assert view['multipleWindowCapability']['status'] == 'held'
        assert view['preferences']['multipleWindow'] is (not restore)
        assert host.call('terminal', request={'action': 'list'}) == []
        if not restore:
            original = view['preferences']
            saved = host.call('saveExperimentalPreferences', preferences={**original, 'multipleWindow': False})
            assert saved['preferences']['multipleWindow'] is False
            stale = host.envelope('saveExperimentalPreferences', preferences=original)
            assert not stale['ok'] and 'Refresh' in stale['error']
            assert host.call('experimentalPreferences')['preferences']['multipleWindow'] is False
        assert children(os.getpid(), names=None) == [], 'Preference activated a child host or shell'
    finally:
        host.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', type=Path)
    parser.add_argument('--phase', choices=['save', 'restore'])
    args = parser.parse_args()
    if args.phase:
        assert args.directory and args.directory.resolve().is_relative_to(ROOT/'output')
        phase(args.directory, args.phase == 'restore')
        return
    directory = ROOT/'output/window-fallback-qualification'/uuid.uuid4().hex
    directory.mkdir(parents=True)
    for step in ['save', 'restore']:
        subprocess.run([sys.executable, __file__, '--directory', str(directory), '--phase', step], check=True, timeout=45)
    report = {'passed': True, 'capability': 'held', 'defaultOn': True, 'offRestoredAcrossProcesses': True,
              'staleSaveRefused': True, 'ownedShells': 0, 'ownedChildHosts': 0, 'modelRequests': 0,
              'evidence': 'Synthetic release C ABI; not native physical UI qualification'}
    (directory/'report.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
    print(json.dumps({'directory': str(directory), **report}))


if __name__ == '__main__':
    main()
