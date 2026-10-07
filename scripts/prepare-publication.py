"""Create a cleaned history copy under output/; never modify or push the source."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def command(root, *args):
    return subprocess.check_output(['git', '-C', str(root), *args]).decode().strip()


def prepare(directory):
    directory = directory.resolve()
    output = (ROOT / 'output').resolve()
    if directory == output or not directory.is_relative_to(output) or directory.exists():
        raise ValueError('Choose a fresh destination strictly under output/. Existing copies are retained.')
    if command(ROOT, 'status', '--porcelain'):
        raise ValueError('Commit the reviewed changes first; publication copies include committed work only.')
    if importlib.util.find_spec('git_filter_repo') is None:
        raise ValueError('Install git-filter-repo 2.47.0 in your Python environment before preparing history.')
    before = command(ROOT, 'show-ref')
    directory.mkdir(parents=True)
    repo = directory / 'dolores.git'
    subprocess.run(['git', 'clone', '--mirror', '--no-local', str(ROOT), str(repo)], check=True)
    for ref in command(repo, 'for-each-ref', '--format=%(refname)').splitlines():
        if not ref.startswith(('refs/heads/', 'refs/tags/')):
            command(repo, 'update-ref', '-d', ref)
    count = command(repo, 'rev-list', '--all', '--count')
    identities = sorted(command(repo, 'log', '--all', '--format=%an <%ae>|%cn <%ce>').splitlines())
    callback = '''from publication_rules import excluded, sanitize
name = filename.decode('utf-8')
if excluded(name): return (None, mode, blob_id)
if mode not in (b'100644', b'100755'): return (filename, mode, blob_id)
key = (filename, blob_id)
if key not in value.data:
    old = value.get_contents_by_identifier(blob_id)
    new = sanitize(name, old)
    value.data[key] = blob_id if new == old else value.insert_file_with_contents(new)
return (filename, mode, value.data[key])
'''
    env = dict(os.environ)
    env['PYTHONPATH'] = os.pathsep.join(filter(None, [str(ROOT / 'scripts'), env.get('PYTHONPATH')]))
    subprocess.run([sys.executable, '-m', 'git_filter_repo', '--force',
                    '--prune-empty', 'never', '--prune-degenerate', 'never',
                    '--file-info-callback', callback,
                    '--message-callback', "from publication_rules import sanitize; return sanitize('commit-message.txt', message)"],
                   cwd=repo, env=env, check=True)
    assert command(repo, 'rev-list', '--all', '--count') == count, 'Commit count changed'
    assert sorted(command(repo, 'log', '--all', '--format=%an <%ae>|%cn <%ce>').splitlines()) == identities, 'Attribution changed'
    assert command(ROOT, 'show-ref') == before, 'Source refs changed'
    subprocess.run([sys.executable, str(ROOT / 'scripts/check-publication.py'),
                    '--repository', str(repo), '--history', '--report', str(directory / 'audit.json')], check=True)
    command(repo, 'fsck', '--full')
    assert not command(repo, 'remote'), 'Publication copy must have no remote'
    receipt = {'commits': int(count), 'authorAttributionRetained': True, 'sourceRefsUnchanged': True,
               'historyAuditPassed': True, 'repository': str(repo), 'pushed': False,
               'note': 'All branch/tag history retained; cleaned contents change commit IDs. Local tooling refs excluded.'}
    (directory / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf-8')
    return receipt


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', type=Path, required=True)
    args = parser.parse_args()
    try:
        print(json.dumps(prepare(args.directory), indent=2))
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        parser.exit(1, f'Publication preparation refused: {error}\n')
