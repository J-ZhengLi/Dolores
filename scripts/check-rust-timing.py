"""Repeat deterministic bridge deadline tests; any failed repetition fails CI."""
import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
import json
from pathlib import Path
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]


def binary():
    result = subprocess.run(['cargo', 'test', '-p', 'dolores-flutter-bridge', '--lib', '--locked',
                             '--no-run', '--message-format=json'], cwd=ROOT,
                            capture_output=True, text=True, encoding='utf-8', errors='replace', check=True)
    for line in result.stdout.splitlines():
        item = json.loads(line)
        if (item.get('reason') == 'compiler-artifact' and item.get('profile', {}).get('test')
                and item.get('target', {}).get('name') == 'dolores_flutter_bridge'
                and item.get('executable')):
            path = Path(item['executable']).resolve(strict=True)
            if not path.is_relative_to((ROOT / 'target').resolve()):
                raise ValueError('Timing test binary must be under the repository target directory.')
            return path
    raise ValueError('Cargo did not return the bridge test binary.')


def repeat(executable, count, jobs):
    started = time.monotonic()
    def sample(number):
        result = subprocess.run([str(executable), 'stream_deadline', '--nocapture', '--test-threads=1'],
                                cwd=ROOT, capture_output=True, text=True, encoding='utf-8', errors='replace', timeout=30)
        if result.returncode or '3 passed; 0 failed' not in result.stdout:
            raise ValueError(f'Deadline repetition {number} failed; no retry:\n{result.stdout}\n{result.stderr}')
    with ThreadPoolExecutor(max_workers=jobs) as pool:
        futures = [pool.submit(sample, number) for number in range(1, count + 1)]
        for future in as_completed(futures):
            future.result()
    return {'ok': True, 'repetitions': count, 'testCases': count * 3, 'concurrentProcesses': jobs,
            'elapsedSeconds': round(time.monotonic() - started, 3), 'retries': 0}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repeat', type=int, default=20)
    parser.add_argument('--jobs', type=int, default=2)
    args = parser.parse_args()
    try:
        if not 1 <= args.repeat <= 200 or not 1 <= args.jobs <= 16:
            raise ValueError('Use 1-200 repetitions and 1-16 concurrent processes.')
        print(json.dumps(repeat(binary(), args.repeat, args.jobs)))
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        parser.exit(1, 'Timing checks failed: ' + str(error) + '\n')
