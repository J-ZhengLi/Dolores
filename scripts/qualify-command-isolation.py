"""Retain public Windows LPAC transport evidence; no product-mode qualification.

Runs deterministic native command tests without a model/provider or credentials.
Temporary workspaces and any interrupted cleanup journals stay in fresh output/.
No firewall, exemptions, feature installation or installed runtime changes.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import uuid

ROOT = Path(__file__).resolve().parents[1]
CASES = [
    'lpac_native_transport_enforces_grants_and_network_with_exact_unicode_pipes',
    'failed_setup_and_path_aliases_never_launch_and_cleanup_allows_a_fresh_review',
    'concurrent_package_grants_preserve_each_other_and_cleanup_resumes_after_a_partial_delete',
    'hard_owner_crash_kills_container_descendants_and_recovery_revokes_without_replay',
    'container_cannot_use_user_ownership_to_expand_readonly_or_runtime_acl_rights',
    'lpac_denies_ipv4_ipv6_udp_and_inbound_bind_after_matching_native_controls',
    'cleanup_refuses_a_new_junction_before_acl_propagation_and_preserves_its_outside_target',
    'reviewed_grants_reject_replacement_and_stop_before_child_effects',
]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', type=Path)
    parser.add_argument('--public-network', action='store_true', help='Only run the opt-in LAN, public TCP 1.1.1.1:443 and example.com system-DNS controls; no payload or credentials.')
    args = parser.parse_args()
    if os.name != 'nt':
        parser.error('Windows x64 transport qualification only.')
    directory = (args.directory or ROOT / 'output/isolation-qualification' / uuid.uuid4().hex).resolve()
    if directory == ROOT / 'output' or not directory.is_relative_to((ROOT / 'output').resolve()):
        parser.error('Choose a fresh directory strictly beneath output/.')
    directory.mkdir(parents=True, exist_ok=False)
    temporary = directory / 'temporary-fixtures'
    temporary.mkdir()
    selected = ['public_network_and_system_dns_qualification'] if args.public_network else CASES
    command = ['cargo', 'test', '-p', 'dolores-tools-command', '--locked',
               'isolation::tests::public_network_and_system_dns_qualification' if args.public_network else 'isolation::tests::',
               '--', '--test-threads=1', *(['--ignored', '--nocapture'] if args.public_network else [])]
    env = dict(os.environ, TEMP=str(temporary), TMP=str(temporary))
    timed_out = False
    with (directory / 'test.log').open('w', encoding='utf-8') as log:
        child = subprocess.Popen(command, cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT)
        try:
            status = child.wait(timeout=120)
        except subprocess.TimeoutExpired:
            timed_out = True
            subprocess.run(['taskkill', '/PID', str(child.pid), '/T', '/F'], capture_output=True, check=False)
            child.wait(timeout=10)
            status = 1
    text = (directory / 'test.log').read_text(encoding='utf-8')
    # --nocapture can place diagnostics between the test name and its final ok.
    cases = {name: (f'test isolation::tests::{name} ... ok' in text if not args.public_network
                    else status == 0 and f'test isolation::tests::{name} ... ' in text and '1 passed; 0 failed' in text)
             for name in selected}
    # Even a successful test exit is insufficient if a crash left owned intent.
    journals = [str(p.relative_to(directory)) for p in temporary.rglob('journal.json')]
    sources = ['src/isolation.rs', 'src/isolation/acl.rs', 'src/isolation/review.rs', 'src/isolation/tests.rs', 'src/windows.rs']
    receipt = {
        'scope': ('Public-network controls: own LAN listener, TCP handshake only to 1.1.1.1:443 and example.com system resolver. No provider, credentials or private names.' if args.public_network else 'Native LPAC transport fixtures only; product routing, UI, external helpers and broader network/toolchain coverage remain separate.'),
        'platform': platform.platform(), 'command': command,
        'passed': status == 0 and not timed_out and all(cases.values()) and not journals,
        'exitCode': status, 'timedOut': timed_out, 'cases': cases, 'retainedCleanupJournals': journals,
        'sourceSha256': {name: hashlib.sha256((ROOT / 'crates/dolores-tools-command' / name).read_bytes()).hexdigest() for name in sources},
        'ignoredWorkerTests': None if args.public_network else 'crash_worker and rights_worker are deliberately invoked by their passing parent cases.',
        'publicNetwork': args.public_network,
    }
    (directory / 'receipt.json').write_text(json.dumps(receipt, indent=2), encoding='utf-8')
    print(text[-2000:])
    print(f'LPAC transport evidence: {directory / "receipt.json"}')
    if not receipt['passed']:
        print('Transport qualification incomplete. Inspect retained logs/journals; no operation was replayed.')
    return 0 if receipt['passed'] else 1


if __name__ == '__main__':
    sys.exit(main())
