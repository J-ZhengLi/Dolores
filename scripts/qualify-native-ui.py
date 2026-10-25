"""Exercise rendered Flutter controls against the normal native Rust bridge.

Uses fresh output/ data and public loopback provider fixtures. Does not inspect
or control the Windows desktop, use personal credentials or call an external
model. MCP qualification uses generated native-vault entries and opens its local
OAuth authorization fixture in the default browser.
"""
import argparse
import os
from pathlib import Path
import subprocess
import sys
import uuid
import json
import zipfile

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
        'worktrees': ('test/native_worktrees_ui_test.dart', 'worktrees-receipt.json'),
        'review-findings': ('test/native_review_findings_ui_test.dart', 'review-findings-receipt.json'),
        'side-conversations': ('test/native_side_conversations_ui_test.dart', 'side-conversations-receipt.json'),
        'prompt-branches': ('test/native_prompt_branches_ui_test.dart', 'prompt-branches-receipt.json'),
        'managed-prompt-branches': ('test/native_managed_prompt_branches_ui_test.dart', 'prompt-managed-branches-receipt.json'),
        'history-search': ('test/native_history_search_ui_test.dart', 'history-search-receipt.json'),
        'attention': ('test/native_attention_ui_test.dart', 'attention-receipt.json'),
        'skill-packages': ('test/native_skill_packages_ui_test.dart', 'skill-packages-receipt.json'),
        'mcp-remote': ('test/native_mcp_remote_ui_test.dart', 'mcp-remote-receipt.json'),
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
    if 'skill-packages' in selected:
        for version in range(1, 7):
            with zipfile.ZipFile(base / f'review-v{version}.zip', 'w', zipfile.ZIP_DEFLATED) as archive:
                archive.writestr('SKILL.md', '---\nname: review\ndescription: Review public code\n---\nRead references/check.md. Never execute scripts during package installation.\n')
                archive.writestr('dolores-package.json', json.dumps({'schema': 1, 'version': f'{version}.0', 'origin': 'public loopback fixture', 'dependencies': [{'name': 'python', 'requirement': 'inspection only; never install'}]}))
                archive.writestr('references/check.md', f'PUBLIC_PACKAGE_V{version}\nVerify the dock 世界.\n')
                archive.writestr('scripts/check.py', "raise RuntimeError('must never execute during package management')\n")
                archive.writestr('assets/image.bin', b'\x00\xff\x01')
                archive.writestr('assets/empty/', b'')
    command = [desktop.flutter_path(args.flutter_sdk), '--no-version-check',
               'test', '--no-pub', '--concurrency=1',
               *(cases[name][0] for name in selected)]
    if 'mcp-remote' in selected:
        # Separate suites own separate FFI engines; Shutdown is terminal.
        command.append('test/native_mcp_oauth_ui_test.dart')
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
    required = [base / cases[name][1] for name in selected]
    if 'mcp-remote' in selected:
        required.append(base / 'mcp-remote-oauth-receipt.json')
    if status or not all(path.is_file() for path in required):
        print(f'Qualification incomplete. Evidence retained at {base}.')
        return 1
    print(f'Native bridge/rendered UI passed ({", ".join(selected)}); evidence: {base}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
