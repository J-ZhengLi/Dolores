"""Collect versioned notices for an existing normal Windows bundle; no downloads."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import tarfile
import tomllib
from urllib.parse import unquote, urljoin, urlsplit

ROOT = Path(__file__).resolve().parents[1]
INPUTS = ('Cargo.lock', 'apps/dolores_flutter/pubspec.lock',
          'apps/dolores_flutter/pubspec.yaml', 'assets/LICENSE.material-icons')
NOTICE_NAME = re.compile(r'^(licen[cs]e|copying|copyright|notice)([._-]|$)', re.I)
MAX_NOTICE = 4 * 1024 * 1024

def digest(path):
    with path.open('rb') as source: return hashlib.file_digest(source, 'sha256').hexdigest()

def command(argv, cwd=ROOT):
    try:
        result = subprocess.run(argv, cwd=cwd, stdout=subprocess.PIPE,
                                stderr=subprocess.PIPE, timeout=120, check=True)
    except (OSError, subprocess.SubprocessError) as error:
        raise ValueError('Dependency metadata command failed; prepare the locked build dependencies and retry.') from error
    return result.stdout.decode('utf-8-sig').strip()

def production_closure(nodes, roots, children):
    seen = set(); todo = list(roots)
    while todo:
        key = todo.pop()
        if key in seen: continue
        if key not in nodes: raise ValueError('Incomplete dependency graph.')
        seen.add(key); todo.extend(children(nodes[key]))
    return sorted(seen)

def license_files(directory):
    found = []
    for base, dirs, files in os.walk(directory, followlinks=False):
        dirs[:] = sorted(d for d in dirs if not d.startswith('.') and d not in ('target', 'build', 'node_modules'))
        found.extend(Path(base)/name for name in sorted(files) if NOTICE_NAME.match(name))
    return sorted(found)

def package_directory(config, name):
    package = next((p for p in config['packages'] if p['name'] == name), None)
    if package is None: raise ValueError('Missing resolved Dart package: '+name)
    uri = urlsplit(urljoin(config['_uri'], package['rootUri']))
    if uri.scheme != 'file': raise ValueError('Dart package is not locally resolved: '+name)
    path = unquote(uri.path)
    if os.name == 'nt' and re.match(r'^/[A-Za-z]:', path): path = path[1:]
    return Path(path).resolve()

def collect(sdk, directory, bundle):
    directory = directory.resolve(); output = (ROOT/'output').resolve(); sdk = sdk.resolve()
    if directory == output or not directory.is_relative_to(output) or directory.is_relative_to(bundle.resolve()):
        raise ValueError('Use a fresh directory strictly beneath output/, outside the bundle.')
    if directory.exists(): raise ValueError('Directory exists; keep earlier evidence and choose a fresh directory.')
    spec = importlib.util.spec_from_file_location('portable', ROOT/'scripts/package-windows.py')
    portable = importlib.util.module_from_spec(spec); spec.loader.exec_module(portable)
    runtime = portable.payload_files(bundle)
    components = []; parts = ['Dolores third-party notices\n\n'
        'Windows x64 Flutter/Rust portable preview. Full upstream notice texts follow.\n'
        'The Rust normal/build dependency closure and Dart production closure are conservative: '
        'some build-only or unused platform packages may not contribute binary code. '
        'Other app shells and externally installed MCP/model software are outside this inventory.\n']
    def add(name, version, category, source, declared, files, base):
        if not files: raise ValueError('Missing full notice text for '+name+'; review before packaging.')
        records = []
        for path in files:
            value = path.read_bytes()
            if not value or len(value) > MAX_NOTICE: raise ValueError('Empty/oversized notice for '+name)
            text = value.decode('utf-8-sig')
            relative = path.relative_to(base).as_posix()
            # Metadata and notice contents must not expose local builder locations.
            for private in (str(ROOT), str(Path.home())):
                if private.lower().replace('\\', '/') in text.lower().replace('\\', '/'):
                    raise ValueError('Local build path in notice for '+name)
            records.append({'file': relative, 'sha256': hashlib.sha256(value).hexdigest()})
            parts.append('\n'+'='*72+'\n'+category+': '+name+' '+version+'\nSource: '+source+
                         '\nDeclared license: '+declared+'\nNotice: '+relative+'\n\n'+text+'\n')
        components.append({'name': name, 'version': version, 'category': category,
                           'source': source, 'declaredLicense': declared, 'notices': records})
    sqlite = None
    metadata = json.loads(command(['cargo', 'metadata', '--format-version', '1', '--locked', '--offline',
                                   '--filter-platform', 'x86_64-pc-windows-msvc']))
    packages = {p['id']: p for p in metadata['packages']}
    nodes = {p['id']: p for p in metadata['resolve']['nodes']}
    bridge = next(p['id'] for p in packages.values() if p['name'] == 'dolores-flutter-bridge')
    selected = production_closure(nodes, [bridge], lambda n:
        [d['pkg'] for d in n['deps'] if any(k['kind'] != 'dev' for k in d['dep_kinds'])])
    locked = tomllib.loads((ROOT/'Cargo.lock').read_text(encoding='utf-8'))['package']
    for key in selected:
        p = packages[key]
        if not p['source']: continue  # Dolores workspace is covered by the packaged project LICENSE.
        if not p['source'].startswith('registry+') or not p['license']:
            raise ValueError('Review a non-registry/custom-license Rust dependency: '+p['name'])
        base = Path(p['manifest_path']).parent
        files = license_files(base)
        if p.get('license_file') and base/p['license_file'] not in files: files.append(base/p['license_file'])
        add(p['name'], p['version'], 'Rust', 'https://crates.io/crates/'+p['name']+'/'+p['version'],
            p['license'], files, base)
        components[-1]['sourceArchive'] = 'https://crates.io/api/v1/crates/'+p['name']+'/'+p['version']+'/download'
        if p['license'] == 'MPL-2.0':
            # Preserve corresponding source in the notice bundle as well as the versioned upstream URL.
            crate = base.parent.parent.parent/'cache'/base.parent.name/(base.name+'.crate')
            expected = next(c['checksum'] for c in locked if c['name']==p['name'] and c['version']==p['version'])
            if digest(crate) != expected: raise ValueError('MPL source archive differs from Cargo.lock.')
            with tarfile.open(crate, 'r:gz') as archive:
                for member in archive.getmembers():
                    if member.isdir(): continue
                    if not member.isfile() or member.size>MAX_NOTICE: raise ValueError('Unsupported MPL source member.')
                    relative = Path(member.name).relative_to(base.name)
                    path = base/relative
                    if not path.resolve().is_relative_to(base.resolve()): raise ValueError('MPL source escapes package.')
                    value = archive.extractfile(member).read()
                    if path.read_bytes()!=value: raise ValueError('Changed MPL dependency source: '+p['name'])
                    parts.append('\nMPL-2.0 corresponding source: '+p['name']+' '+p['version']+'/'+relative.as_posix()+'\n\n'+
                                 value.decode('utf-8-sig')+'\n')
            components[-1]['correspondingSourceIncluded'] = True
        if p['name'] == 'libsqlite3-sys': sqlite = base/'sqlite3/sqlite3.c'
    if sqlite is None: raise ValueError('Review native SQLite notices after dependency changes.')
    sqlite_head = sqlite.read_text(encoding='utf-8')[:4096]
    sqlite_version = re.search(r'version ([0-9.]+)', sqlite_head)
    blessing = re.search(r'/\*\n\*\* [^\n]*\n\*\*\n\*\* The author disclaims copyright.*?\*/', sqlite_head, re.S)
    if not sqlite_version or not blessing: raise ValueError('Review changed SQLite copyright header.')
    parts.append('\nNative SQLite '+sqlite_version[1]+'\nSource: https://sqlite.org/src\n\n'+blessing[0]+'\n')
    components.append({'name':'SQLite amalgamation', 'version':sqlite_version[1], 'category':'Native',
                       'source':'https://sqlite.org/src', 'declaredLicense':'Public domain copyright disclaimer',
                       'notices':[{'file':'sqlite3/sqlite3.c copyright header',
                                   'sha256':hashlib.sha256(blessing[0].encode()).hexdigest()}]})
    app = ROOT/'apps/dolores_flutter'
    config_path = app/'.dart_tool/package_config.json'
    config = json.loads(config_path.read_text(encoding='utf-8')); config['_uri'] = config_path.as_uri()
    if package_directory(config, 'flutter') != (sdk/'packages/flutter').resolve():
        raise ValueError('Flutter SDK differs from resolved packages; rebuild using this SDK first.')
    dart = sdk/'bin/cache/dart-sdk/bin'/('dart.exe' if os.name == 'nt' else 'dart')
    deps = json.loads(command([str(dart), 'pub', 'deps', '--json'], app))
    nodes = {p['name']: p for p in deps['packages']}
    roots = [p['name'] for p in nodes.values() if p['kind'] == 'direct']
    version = json.loads((sdk/'bin/cache/flutter.version.json').read_text(encoding='utf-8'))
    engine_manifest = sdk/'bin/cache/artifacts/engine/windows-x64-release/license.windows_flutter.md'
    if version['engineRevision'] not in engine_manifest.read_text(encoding='utf-8'):
        raise ValueError('Windows engine notice revision differs from this SDK; prepare/rebuild the matching SDK.')
    for name in production_closure(nodes, roots, lambda n: n['dependencies']):
        p = nodes[name]; base = package_directory(config, name)
        source = ('https://github.com/flutter/flutter/tree/'+version['frameworkRevision'] if base.is_relative_to(sdk)
                  else 'https://pub.dev/packages/'+name+'/versions/'+p['version'])
        files = license_files(base)
        if not files and base.is_relative_to(sdk):
            # SDK packages such as flutter_web_plugins inherit the repository BSD license.
            files = [sdk/'LICENSE']; base = sdk
        add(name, p['version'], 'Dart', source, 'See full upstream notice', files, base)
    add('Flutter SDK', version['frameworkVersion'], 'SDK',
        'https://github.com/flutter/flutter/tree/'+version['frameworkRevision'], 'BSD-3-Clause', [sdk/'LICENSE'], sdk)
    engine = sdk/'bin/cache/pkg/sky_engine/LICENSE'
    add('Flutter engine and bundled native dependencies', version['engineRevision'], 'SDK',
        'https://github.com/flutter/engine/tree/'+version['engineRevision'], 'See consolidated upstream notice', [engine], sdk)
    add('Dart SDK', version['dartSdkVersion'], 'SDK', 'https://github.com/dart-lang/sdk',
        'BSD-3-Clause and upstream dependencies', [sdk/'bin/cache/dart-sdk/LICENSE'], sdk)
    sysroot = Path(command(['rustc', '--print', 'sysroot']))
    rust_doc = sysroot/'share/doc/rust'
    add('Rust standard library', command(['rustc', '--version']), 'SDK', 'https://github.com/rust-lang/rust',
        'See standard-library copyright inventory and license texts',
        [rust_doc/'COPYRIGHT-library.html', *sorted((rust_doc/'licenses').glob('*.txt'))], rust_doc)
    add('Dolores infinity artwork / Material Icons', 'frozen glyph all_inclusive_rounded', 'Artwork',
        'https://github.com/google/material-design-icons', 'CC-BY-4.0 / Apache-2.0; see attribution',
        [ROOT/'assets/LICENSE.material-icons'], ROOT/'assets')
    notices = '\n'.join(parts).encode('utf-8')
    if len(notices) > 64*1024*1024: raise ValueError('Notice collection exceeds 64 MiB.')
    inventory = {'format': 1, 'platform': 'windows-x64', 'scope': 'conservative production/build dependency closure',
                 'components': components, 'inputs': {name: digest(ROOT/name) for name in INPUTS},
                 'runtime': {name: digest(path) for name, path in sorted(runtime.items())},
                 'noticesSha256': hashlib.sha256(notices).hexdigest()}
    directory.mkdir(parents=True)
    (directory/'THIRD-PARTY-NOTICES.txt').write_bytes(notices)
    (directory/'DEPENDENCIES.json').write_text(json.dumps(inventory, indent=2)+'\n', encoding='utf-8')
    return inventory

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--flutter-sdk', type=Path, default=ROOT/'output/toolchains/flutter')
    parser.add_argument('--directory', type=Path, required=True)
    args = parser.parse_args()
    try:
        if not args.directory.is_absolute(): raise ValueError('Use an absolute output directory.')
        inventory = collect(args.flutter_sdk, args.directory,
                            ROOT/'apps/dolores_flutter/build/windows/x64/runner/Release')
        print(json.dumps({'ok': True, 'components': len(inventory['components']), 'liveRequests': 0}))
    except (OSError, ValueError, KeyError, UnicodeError, StopIteration) as error:
        parser.exit(1, 'Notices refused: '+str(error)+'\n')

if __name__ == '__main__': main()
