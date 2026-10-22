"""Exercise rendered Flutter controls against the normal native Rust bridge.

Uses fresh output/ data and a public loopback provider fixture. Does not inspect
or control the Windows desktop, access credentials or call an external model.
"""
import argparse
import os
from pathlib import Path
import subprocess
import sys
import uuid

import desktop


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', type=Path)
    parser.add_argument('--flutter-sdk', type=Path)
    cases = {
        'local': ('test/native_local_ui_test.dart', 'receipt.json'),
        'scheduled': ('test/native_scheduled_ui_test.dart', 'schedule-receipt.json'),
        'stop': ('test/native_stop_ui_test.dart', 'stop-receipt.json'),
        'queue': ('test/native_queue_ui_test.dart', 'queue-receipt.json'),
        'goal': ('test/native_goal_ui_test.dart', 'goal-receipt.json'),
        'plan': ('test/native_plan_ui_test.dart', 'plan-receipt.json'),
        'chat-safeguards': ('test/native_chat_safeguards_ui_test.dart', 'chat-safeguards-receipt.json'),
        'commands': ('test/native_commands_ui_test.dart', 'commands-receipt.json'),
        'isolated-commands': ('test/native_isolated_commands_ui_test.dart', 'commands-isolated-receipt.json'),
    }
    parser.add_argument('--case', action='append', choices=cases, help='Run selected journeys; default: all.')
    args = parser.parse_args()
    if os.name != 'nt':
        parser.error('This qualification currently needs the Windows native bundle.')
    desktop.require_normal_build()
    base = (args.directory or desktop.ROOT / 'output/native-ui' / uuid.uuid4().hex).resolve()
    if base == desktop.ROOT / 'output' or not base.is_relative_to(desktop.ROOT / 'output'):
        parser.error('Choose a fresh directory strictly beneath output/.')
    if base.exists():
        parser.error('The directory already exists. Prior evidence was retained.')
    base.mkdir(parents=True)
    env = dict(os.environ, DOLORES_DATA_DIR=str(base / 'data'),
               DOLORES_GLOBAL_SKILLS_DIR=str(base / 'skills'),
               DOLORES_AUDIT_LIBRARY=str(desktop.executable().parent / 'dolores_flutter_bridge.dll'),
               FLUTTER_SUPPRESS_ANALYTICS='true', DART_SUPPRESS_ANALYTICS='true')
    selected = list(dict.fromkeys(args.case or cases))
    command = [desktop.flutter_path(args.flutter_sdk), '--no-version-check',
               'test', '--no-pub', '--concurrency=1',
               *(cases[name][0] for name in selected)]
    with (base / 'test.log').open('w', encoding='utf-8') as log:
        child = subprocess.Popen(command, cwd=desktop.APP, env=env,
                                 stdout=log, stderr=subprocess.STDOUT)
        try:
            status = child.wait(timeout=120)
        except subprocess.TimeoutExpired:
            # Only the process tree created above is stopped; never another app.
            subprocess.run(['taskkill', '/PID', str(child.pid), '/T', '/F'],
                           capture_output=True, check=False)
            child.wait(timeout=10)
            print(f'Native UI qualification timed out. Evidence retained at {base}.')
            return 1
    print((base / 'test.log').read_text(encoding='utf-8')[-2400:])
    if status or not all((base / cases[name][1]).is_file() for name in selected):
        print(f'Qualification incomplete. Evidence retained at {base}.')
        return 1
    print(f'Native bridge/rendered UI passed ({", ".join(selected)}); evidence: {base}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
