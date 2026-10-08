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
VERSION = r'(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)'
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
                raise ValueError('Released versions require MAJOR.MINOR.PATCH and an ISO date.')
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
    if app_version(root) != version:
        raise ValueError('Set and align the source versions before preparing this release.')
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
    fd, temporary = tempfile.mkstemp(dir=p.parent, prefix='.changelog-', suffix='.tmp')
    try:
        with os.fdopen(fd, 'w', encoding='utf-8', newline='\n') as stream:
            stream.write(new)
        os.replace(temporary, p)
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
