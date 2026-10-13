"""Repeat stream, MCP and language recovery tests; any failure fails CI."""
import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
import json
from pathlib import Path
import re
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]


def binary(software_hash=False, suite='stream'):
    crate = 'dolores-tools-mcp' if suite == 'mcp-protocol' else 'dolores-flutter-bridge'
    command = ['cargo', 'test', '-p', crate, '--lib', '--locked',
               '--no-run', '--message-format=json']
    if software_hash:
        command += ['--features', 'sha2/force-soft']
    result = subprocess.run(command, cwd=ROOT,
                            capture_output=True, text=True, encoding='utf-8', errors='replace', check=True)
    for line in result.stdout.splitlines():
        item = json.loads(line)
        if (item.get('reason') == 'compiler-artifact' and item.get('profile', {}).get('test')
                and item.get('target', {}).get('name') == crate.replace('-', '_')
                and item.get('executable')):
            path = Path(item['executable']).resolve(strict=True)
            if not path.is_relative_to((ROOT / 'target').resolve()):
                raise ValueError('Timing test binary must be under the repository target directory.')
            return path
    raise ValueError('Cargo did not return the bridge test binary.')


def repeat(executable, count, jobs, suite='stream'):
    started = time.monotonic()
    test_filter, cases = {'stream': ('stream_deadline', 3), 'mcp': ('mcp::tests::', 3),
                         'mcp-protocol': ('tests::pages_and_unsupported_client_requests_follow_protocol', 1),
                         'language': ('language::tests::', 4)}[suite]
    def sample(number):
        result = subprocess.run([str(executable), test_filter, '--nocapture', '--test-threads=2'],
                                cwd=ROOT, capture_output=True, text=True, encoding='utf-8', errors='replace', timeout=120)
        summary = re.search(r'(?m)^test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;', result.stdout)
        if result.returncode or not summary or tuple(map(int, summary.groups())) != (cases, 0, 0):
            raise ValueError(f'{suite} repetition {number} failed; no retry:\n{result.stdout}\n{result.stderr}')
    with ThreadPoolExecutor(max_workers=jobs) as pool:
        futures = [pool.submit(sample, number) for number in range(1, count + 1)]
        for future in as_completed(futures):
            future.result()
    return {'ok': True, 'suite': suite, 'repetitions': count, 'testCases': count * cases, 'concurrentProcesses': jobs,
            'elapsedSeconds': round(time.monotonic() - started, 3), 'retries': 0}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repeat', type=int, default=20)
    parser.add_argument('--jobs', type=int, default=2)
    parser.add_argument('--suite', choices=['stream', 'mcp', 'mcp-protocol', 'language'], default='stream')
    parser.add_argument('--software-hash', action='store_true', help='Exercise SHA-256 without hardware acceleration; test binaries only.')
    args = parser.parse_args()
    try:
        if not 1 <= args.repeat <= 200 or not 1 <= args.jobs <= 16:
            raise ValueError('Use 1-200 repetitions and 1-16 concurrent processes.')
        print(json.dumps(repeat(binary(args.software_hash, args.suite), args.repeat, args.jobs, args.suite)))
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        parser.exit(1, 'Timing checks failed: ' + str(error) + '\n')
