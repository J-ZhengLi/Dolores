"""Validate versions and maintain the reviewed, human-written changelog."""
import argparse
from datetime import date
import json
import os
from pathlib import Path
import re
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[1]
NUMBER = r'(?:0|[1-9][0-9]*)'
IDENTIFIER = r'(?:0|[1-9][0-9]*|[0-9]*[A-Za-z-][0-9A-Za-z-]*)'
VERSION = NUMBER + r'\.' + NUMBER + r'\.' + NUMBER + r'(?:-' + IDENTIFIER + r'(?:\.' + IDENTIFIER + r')*)?'
HEADING = re.compile(r'^## \[([^\]]+)\](?: - ([0-9]{4}-[0-9]{2}-[0-9]{2}))?$', re.M)


def sections(text):
    matches = list(HEADING.finditer(text))
    result = {}
    for index, match in enumerate(matches):
        version, released = match.groups()
        if version in result:
            raise ValueError('Duplicate changelog section: ' + version)
        if version != 'Unreleased':
            if not re.fullmatch(VERSION, version) or not released:
                raise ValueError('Released versions require a SemVer version and an ISO date.')
            date.fromisoformat(released)
        elif released:
            raise ValueError('Unreleased must have no release date.')
        end = matches[index + 1].start() if index + 1 < len(matches) else len(text)
        result[version] = (match, text[match.end():end].strip())
    if not matches or matches[0].group(1) != 'Unreleased':
        raise ValueError('The first changelog section must be Unreleased.')
    return result


def app_version(root=ROOT):
    cargo = tomllib.loads((root / 'Cargo.toml').read_text())['workspace']['package']['version']
    flutter = re.search(r'^version: (' + VERSION + r')\+([1-9][0-9]*)$',
                        (root / 'apps/dolores_flutter/pubspec.yaml').read_text(), re.M)
    if not flutter or cargo != flutter.group(1):
        raise ValueError('Cargo and Flutter versions must match; Flutter needs a positive build number.')
    return cargo


def check(root=ROOT, tag=None):
    version = app_version(root)
    entries = sections((root / 'CHANGELOG.md').read_text(encoding='utf-8'))
    if tag:
        if tag != 'v' + version:
            raise ValueError('Release tag must equal v' + version)
        if version not in entries or not re.search(r'^- \S', entries[version][1], re.M):
            raise ValueError('Tag release requires dated, nonempty changelog notes for ' + version)
    return version


def prepare(root, version, released):
    previous = app_version(root)
    if not re.fullmatch(VERSION, version):
        raise ValueError('Use a SemVer version, such as 0.0.1-alpha or 0.0.1.')
    date.fromisoformat(released)
    p = root / 'CHANGELOG.md'
    text = p.read_text(encoding='utf-8')
    entries = sections(text)
    if version in entries:
        raise ValueError('Version already recorded; existing release notes are retained.')
    heading, body = entries['Unreleased']
    if not re.search(r'^- \S', body, re.M):
        raise ValueError('Unreleased has no user-visible changes to release.')
    new = text[:heading.end()] + '\n\n## [' + version + '] - ' + released + '\n\n' + text[heading.end():].lstrip()
    sections(new)
    cargo_path = root / 'Cargo.toml'
    flutter_path = root / 'apps/dolores_flutter/pubspec.yaml'
    cargo_text = cargo_path.read_text(encoding='utf-8')
    cargo_new, count = re.subn(r'(\[workspace\.package\]\s*\nversion\s*=\s*")[^"]+(")',
                               lambda m: m[1] + version + m[2], cargo_text, count=1)
    if count != 1:
        raise ValueError('Workspace version declaration is unavailable; align it before retrying.')
    flutter_text = flutter_path.read_text(encoding='utf-8')
    flutter_match = re.search(r'^version: (' + VERSION + r')\+([1-9][0-9]*)$', flutter_text, re.M)
    build = int(flutter_match[2]) + (version != previous)
    flutter_new = flutter_text[:flutter_match.start()] + f'version: {version}+{build}' + flutter_text[flutter_match.end():]
    updates = {cargo_path: cargo_new, flutter_path: flutter_new, p: new}
    lock_path = root / 'Cargo.lock'
    if lock_path.exists():
        members = tomllib.loads(cargo_text).get('workspace', {}).get('members', [])
        names = {tomllib.loads((root / member / 'Cargo.toml').read_text())['package']['name'] for member in members}
        def update_package(match):
            block = match[0]
            package = tomllib.loads(block)['package'][0]
            if package['name'] in names and 'source' not in package:
                if package['version'] != previous:
                    raise ValueError('Workspace lock versions are stale; refresh Cargo.lock before retrying.')
                block = re.sub(r'^version = "[^"]+"$', f'version = "{version}"', block, count=1, flags=re.M)
            return block
        lock_text = lock_path.read_text(encoding='utf-8')
        lock_new = re.sub(r'\[\[package\]\][\s\S]*?(?=\[\[package\]\]|\Z)', update_package, lock_text)
        if lock_new != lock_text:
            updates[lock_path] = lock_new
    originals = {path: path.read_bytes() for path in updates}
    try:
        for path, value in updates.items():
            atomic_write(path, value)
        check(root, 'v' + version)
    except Exception:
        for path, value in originals.items():
            path.write_bytes(value)
        raise


def atomic_write(path, text):
    fd, temporary = tempfile.mkstemp(dir=path.parent, prefix='.release-', suffix='.tmp')
    try:
        with os.fdopen(fd, 'w', encoding='utf-8', newline='\n') as stream:
            stream.write(text)
        os.replace(temporary, path)
    finally:
        Path(temporary).unlink(missing_ok=True)


def notes(root, version):
    text = (root / 'CHANGELOG.md').read_text(encoding='utf-8')
    entries = sections(text)
    if version not in entries:
        raise ValueError('No changelog section for ' + version)
    heading, body = entries[version]
    return heading.group(0) + '\n\n' + body + '\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    verify = commands.add_parser('check')
    verify.add_argument('--tag')
    promote = commands.add_parser('prepare')
    promote.add_argument('--version', required=True)
    promote.add_argument('--date', required=True)
    extract = commands.add_parser('notes')
    extract.add_argument('--version', required=True)
    args = parser.parse_args()
    try:
        if args.command == 'check':
            print(json.dumps({'ok': True, 'version': check(tag=args.tag)}))
        elif args.command == 'prepare':
            prepare(ROOT, args.version, args.date)
            print(json.dumps({'ok': True, 'prepared': args.version, 'published': False}))
        else:
            print(notes(ROOT, args.version), end='')
    except (OSError, ValueError, KeyError) as error:
        parser.exit(1, 'Release refused: ' + str(error) + '\n')


if __name__ == '__main__':
    main()
