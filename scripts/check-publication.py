"""Audit tracked files or reachable history; print locations, never secret values."""
import argparse
import json
from pathlib import Path
import re
import subprocess

from publication_rules import excluded

RULES = {
    'personal-machine-path': re.compile(r'[A-Za-z]:[\\/]+(?:Users[\\/]+[^\\/\s\"\x27`<>]+|Workspace[\\/]+(?:project_dolores|mario_clone))', re.I),
    'access-token': re.compile(r'\b(?:ghp_|github_pat_|sk-proj-)[A-Za-z0-9_\-]{20,}'),
    'private-key': re.compile(r'-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----\s+[A-Za-z0-9+/]{32,}'),
    'personal-model-instruction': re.compile(r'(?:routine (?:live tests|model probes).*(?:Qwen|DeepSeek)|Qwen.*routine.*DeepSeek|configured provider\x27s Qwen)', re.I),
}


def findings(path, data):
    if excluded(path):
        return [{'file': path, 'rule': 'private-artifact', 'line': 0}]
    try:
        text = data.decode('utf-8')
    except UnicodeError:
        return []
    return [{'file': path, 'rule': label, 'line': text.count('\n', 0, match.start()) + 1}
            for label, pattern in RULES.items()
            if label != 'personal-model-instruction' or (
                path.endswith('.md') and path != 'docs/ACCEPTANCE.md'
                and not path.startswith('docs/qualification/'))
            for match in pattern.finditer(text)]


def git(root, *args):
    return subprocess.check_output(['git', '-C', str(root), *args])


def scan(root, history=False):
    result = []
    if not history:
        names = git(root, 'ls-files', '-z').decode().split('\0')
        for name in filter(None, names):
            p = root / name
            if p.is_file():
                result.extend(findings(name, p.read_bytes()))
        return result, len(names) - 1
    records = git(root, 'rev-list', '--objects', '--all').decode().splitlines()
    objects = {line.split(' ', 1)[0]: line.split(' ', 1)[1] if ' ' in line else '' for line in records}
    # Batch avoids process-per-blob overhead and preserves length-delimited contents.
    output = b'' if not objects else subprocess.run(
        ['git', '-C', str(root), 'cat-file', '--batch'],
        input=('\n'.join(objects) + '\n').encode(), capture_output=True, check=True).stdout
    pos = 0
    checked = 0
    for oid, name in objects.items():
        end = output.index(b'\n', pos)
        _, kind, size = output[pos:end].split()
        pos = end + 1
        data = output[pos:pos + int(size)]
        pos += int(size) + 1
        if kind == b'blob':
            checked += 1
            result.extend(dict(v, object=oid) for v in findings(name, data))
        elif kind in (b'commit', b'tag'):
            # Author attribution is deliberately retained; audit only the message.
            message = data.split(b'\n\n', 1)[-1]
            result.extend(dict(v, object=oid) for v in findings('commit-message.txt', message))
    names = set(git(root, 'log', '--all', '--format=', '--name-only').decode().splitlines())
    result.extend({'file': name, 'rule': 'private-historical-path', 'line': 0}
                  for name in sorted(names) if excluded(name))
    return result, checked


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repository', type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument('--history', action='store_true')
    parser.add_argument('--report', type=Path)
    args = parser.parse_args()
    items, count = scan(args.repository.resolve(), args.history)
    report = {'ok': not items, 'history': args.history, 'checked': count, 'findings': items,
              'scope': 'Known path, credential and instruction patterns; manual review still required. Author attribution retained.'}
    if args.report:
        args.report.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(dict(report, findings=items[:30]), indent=2))
    return 1 if items else 0


if __name__ == '__main__':
    raise SystemExit(main())
