"""Package/verify an existing normal Windows x64 bundle without user data."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import struct
import zipfile

ROOT = Path(__file__).resolve().parents[1]
BUNDLE = ROOT/'apps/dolores_flutter/build/windows/x64/runner/Release'
RUNTIME = {
    'dolores-desktop-helper.exe',
    'dolores-update-launcher.exe',
    'dolores_flutter.exe', 'dolores_flutter_bridge.dll', 'flutter_windows.dll',
    'file_selector_windows_plugin.dll', 'screen_retriever_windows_plugin.dll',
    'pasteboard_plugin.dll',
    'window_manager_plugin.dll', 'data/app.so', 'data/icudtl.dat',
    'data/flutter_assets/AssetManifest.bin', 'data/flutter_assets/FontManifest.json',
    'data/flutter_assets/NativeAssetsManifest.json', 'data/flutter_assets/NOTICES.Z',
    'data/flutter_assets/fonts/MaterialIcons-Regular.otf',
    'data/flutter_assets/shaders/ink_sparkle.frag', 'data/flutter_assets/shaders/stretch_effect.frag',
}
DOCUMENTS = {'START-HERE.md','USER_GUIDE.md','PRIVACY.md','LICENSE','CHANGELOG.md',
             'Start-Dolores.cmd','THIRD-PARTY-NOTICES.txt','DEPENDENCIES.json'}
AUDIT_INPUTS = ('Cargo.lock', 'apps/dolores_flutter/pubspec.lock',
                'apps/dolores_flutter/pubspec.yaml', 'assets/LICENSE.material-icons',
                'assets/notices/fxhash-0.2.1/NOTICE.txt',
                'assets/notices/fxhash-0.2.1/LICENSE-APACHE',
                'assets/notices/fxhash-0.2.1/LICENSE-MIT',
                'assets/notices/mac-0.1.1/NOTICE.txt', 'assets/notices/mac-0.1.1/LICENSE-APACHE',
                'assets/notices/match_token-0.35.0/NOTICE.txt', 'assets/notices/match_token-0.35.0/LICENSE-APACHE',
                'assets/notices/selectors-0.31.0/NOTICE.txt', 'assets/notices/selectors-0.31.0/LICENSE-MPL-2.0')
PREFIX = 'Dolores/'
MAX_BYTES = 256*1024*1024

def private_prefixes():
    # This local identity check supplements secret scanning; it is not a PII classifier.
    values={str(path).replace('\\','/').lower() for path in (ROOT,Path.home())}
    values|={value.replace('/','\\') for value in values}
    return tuple(value.encode(encoding).lower() for value in values for encoding in ('utf-8','utf-16-le'))

PRIVATE_PREFIXES = private_prefixes()

def is_link(path):
    # Includes Windows junctions/reparse points on Python 3.11 as well as symlinks.
    return path.is_symlink() or bool(getattr(path.lstat(),'st_file_attributes',0)&0x400)

def sha(stream, name=None):
    digest=hashlib.sha256(); tail=b''
    while chunk:=stream.read(1024*1024):
        digest.update(chunk)
        if name:
            window=tail+chunk.lower()
            if any(prefix in window for prefix in PRIVATE_PREFIXES):
                raise ValueError('Local build path in '+name+'. Rebuild with python scripts/desktop.py build before packaging.')
            tail=window[-max(map(len,PRIVATE_PREFIXES)):]
    return digest.hexdigest()

def x64_binary(path):
    with path.open('rb') as stream:
        header = stream.read(64)
        if len(header)!=64 or header[:2]!=b'MZ': raise ValueError('Missing Windows PE header: '+path.name)
        offset = struct.unpack_from('<I',header,60)[0]
        if offset>1024*1024: raise ValueError('Invalid Windows PE offset: '+path.name)
        stream.seek(offset); pe = stream.read(6)
        if len(pe)!=6 or pe[:4]!=b'PE\0\0' or pe[4:]!=b'\x64\x86':
            raise ValueError('Expected an x64 Windows binary: '+path.name)

def payload_files(bundle):
    if is_link(bundle): raise ValueError('Bundle links are not allowed.')
    found = {}
    total = 0
    for base, directories, files in os.walk(bundle, followlinks=False):
        for name in directories+files:
            path=Path(base)/name
            if is_link(path): raise ValueError('Runtime links are not allowed.')
        for name in files:
            path=Path(base)/name; relative=path.relative_to(bundle).as_posix()
            if relative not in RUNTIME: raise ValueError('Unexpected runtime file; review the payload contract: '+relative)
            size=path.stat().st_size
            if not size: raise ValueError('Empty runtime file: '+relative)
            total+=size; found[relative]=path
    if found.keys()!=RUNTIME: raise ValueError('Missing required runtime files: '+', '.join(sorted(RUNTIME-found.keys())))
    if total>MAX_BYTES: raise ValueError('Runtime exceeds the 256 MiB package guard.')
    for name,path in found.items():
        if name.endswith(('.exe','.dll')): x64_binary(path)
        with path.open('rb') as stream: sha(stream,name)
    return found

def info(name):
    result=zipfile.ZipInfo(PREFIX+name, date_time=(1980,1,1,0,0,0))
    result.compress_type=zipfile.ZIP_DEFLATED
    result.external_attr=0o100644<<16
    return result

def validate_notices(inventory, notices, runtime):
    if not isinstance(inventory,dict) or inventory.get('format')!=1 or inventory.get('platform')!='windows-x64':
        raise ValueError('Invalid dependency inventory; collect notices again.')
    if inventory.get('runtime')!=runtime or inventory.get('noticesSha256')!=hashlib.sha256(notices).hexdigest():
        raise ValueError('Stale/mismatched dependency notices; collect notices for this bundle again.')
    components=inventory.get('components')
    if not isinstance(components,list) or not components or any(
        not isinstance(c,dict) or not c.get('name') or not c.get('source') or not c.get('notices') for c in components):
        raise ValueError('Incomplete dependency inventory; review missing notices.')

def notice_documents(directory, files):
    if is_link(directory): raise ValueError('Notice directory links are not allowed.')
    values={}
    for name in ('DEPENDENCIES.json','THIRD-PARTY-NOTICES.txt'):
        path=directory/name
        if is_link(path) or path.stat().st_size>64*1024*1024: raise ValueError('Invalid notice file: '+name)
        values[name]=path.read_bytes()
    inventory=json.loads(values['DEPENDENCIES.json'])
    runtime={}
    for name,path in files.items():
        with path.open('rb') as stream: runtime[name]=sha(stream)
    validate_notices(inventory,values['THIRD-PARTY-NOTICES.txt'],runtime)
    inputs={name:hashlib.sha256((ROOT/name).read_bytes()).hexdigest() for name in AUDIT_INPUTS}
    if inventory.get('inputs')!=inputs: raise ValueError('Dependency inputs changed; collect notices again before packaging.')
    return values

def verify_archive(path):
    with zipfile.ZipFile(path) as archive:
        entries=archive.infolist()
        expected={PREFIX+n for n in RUNTIME|DOCUMENTS|{'MANIFEST.json'}}
        if len(entries)!=len(expected) or {e.filename for e in entries}!=expected:
            raise ValueError('Archive has missing, duplicate or unexpected entries.')
        if sum(e.file_size for e in entries)>MAX_BYTES or any(e.flag_bits&1 for e in entries):
            raise ValueError('Archive exceeds its size guard or is encrypted.')
        if any(((e.external_attr>>16)&0o170000) not in (0,0o100000) for e in entries):
            raise ValueError('Archive entries must be regular files.')
        if archive.getinfo(PREFIX+'MANIFEST.json').file_size>16384: raise ValueError('Oversized manifest.')
        manifest=json.loads(archive.read(PREFIX+'MANIFEST.json'))
        if not isinstance(manifest,dict) or manifest.get('format')!=1 or manifest.get('platform')!='windows-x64' or manifest.get('signed') is not False:
            raise ValueError('Unsupported manifest.')
        if not isinstance(manifest.get('files'),dict) or manifest['files'].keys()!=RUNTIME|DOCUMENTS:
            raise ValueError('Manifest does not describe the exact payload.')
        for name,record in manifest['files'].items():
            entry=archive.getinfo(PREFIX+name)
            if not isinstance(record,dict) or type(record.get('bytes')) is not int or record['bytes']!=entry.file_size or not isinstance(record.get('sha256'),str) or not re.fullmatch('[0-9a-f]{64}',record['sha256']):
                raise ValueError('Invalid file record: '+name)
            with archive.open(entry) as stream:
                if sha(stream,name)!=record['sha256']: raise ValueError('Archive content hash differs: '+name)
        inventory=json.loads(archive.read(PREFIX+'DEPENDENCIES.json'))
        validate_notices(inventory,archive.read(PREFIX+'THIRD-PARTY-NOTICES.txt'),
                         {name:manifest['files'][name]['sha256'] for name in RUNTIME})
        return manifest

def package(bundle, directory, notice_directory):
    if is_link(bundle): raise ValueError('Bundle links are not allowed.')
    output=(ROOT/'output').resolve(); directory=directory.resolve(); bundle=bundle.resolve()
    if directory==output or not directory.is_relative_to(output) or directory.is_relative_to(bundle):
        raise ValueError('Choose a fresh destination strictly beneath output/ and outside the bundle.')
    if directory.exists(): raise ValueError('Destination exists; choose a fresh directory to retain earlier artifacts.')
    files=payload_files(bundle)
    notices=notice_documents(notice_directory,files)
    documents={
        'LICENSE':(ROOT/'LICENSE').read_bytes(),
        'CHANGELOG.md':(ROOT/'CHANGELOG.md').read_bytes(),
        'PRIVACY.md':(ROOT/'docs/PRIVACY.md').read_bytes(),
        'USER_GUIDE.md':(ROOT/'docs/USER_GUIDE.md').read_text(encoding='utf-8').replace(
            'See [privacy](PRIVACY.md) and [tested limitations](ACCEPTANCE.md).',
            'See [privacy](PRIVACY.md). Native accessibility, other platforms and low-end acceptance remain open.').encode(),
        'START-HERE.md':b'# Dolores Windows portable preview\n\nExtract the whole ZIP, keep these files together, and open Start-Dolores.cmd. It explains missing app/C++ runtime files before launching Dolores.\n\nRead [User guide](USER_GUIDE.md) and [Privacy](PRIVACY.md). [Third-party notices](THIRD-PARTY-NOTICES.txt) and [versioned inventory](DEPENDENCIES.json) are included.\n\nRequires Microsoft Visual C++ x64 runtime: https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist\n\nIf a loader error still appears, install/repair that runtime and extract a fresh complete ZIP. The launcher checks file presence, not version/loadability; it downloads nothing.\n\nThis unsigned preview does not install a model, dependencies or an updater. Conversations/settings use your account application-data folder, not this extracted folder. Signing, clean-machine tests and low-end/input acceptance remain open.\n',
        'Start-Dolores.cmd':(ROOT/'scripts/Start-Dolores.cmd').read_bytes(),
        **notices,
    }
    version=re.search(r'^version: ([0-9.]+\+[0-9]+)$',(ROOT/'apps/dolores_flutter/pubspec.yaml').read_text(),re.M)[1]
    manifest={'format':1,'version':version,'platform':'windows-x64','signed':False,
              'buildFreshness':'existing bundle; build the normal entry point before packaging',
              'prerequisite':'Microsoft Visual C++ x64 runtime installed separately','files':{}}
    directory.mkdir(parents=True)
    pending=directory/'package.pending'
    archive_path=directory/('Dolores-'+version.replace('+','-')+'-windows-x64.zip')
    try:
        with zipfile.ZipFile(pending,'x',compression=zipfile.ZIP_DEFLATED,compresslevel=6) as archive:
            for name in sorted(files.keys()|documents.keys()):
                if name in documents:
                    value=documents[name]; archive.writestr(info(name),value)
                    manifest['files'][name]={'bytes':len(value),'sha256':hashlib.sha256(value).hexdigest()}
                else:
                    path=files[name]; before=path.stat(); digest=hashlib.sha256(); size=0
                    with path.open('rb') as source, archive.open(info(name),'w') as target:
                        while chunk:=source.read(1024*1024): target.write(chunk);digest.update(chunk);size+=len(chunk)
                    after=path.stat()
                    if (before.st_size,before.st_mtime_ns)!=(after.st_size,after.st_mtime_ns) or size!=before.st_size:
                        raise ValueError('Bundle changed while packaging. Rebuild and use a fresh destination.')
                    manifest['files'][name]={'bytes':size,'sha256':digest.hexdigest()}
            archive.writestr(info('MANIFEST.json'),json.dumps(manifest,indent=2).encode())
        verify_archive(pending)
        os.link(pending,archive_path)  # No replacement, even if a destination appears concurrently.
        with archive_path.open('rb') as source: digest=sha(source)
        with archive_path.with_suffix('.zip.sha256').open('x',encoding='utf-8') as stream:
            stream.write(digest+'  '+archive_path.name+'\n')
        return archive_path
    finally:
        # This exact owned temporary file stays within the checked fresh output directory.
        pending.unlink(missing_ok=True)

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    group=parser.add_mutually_exclusive_group(required=True)
    group.add_argument('--directory',type=Path,help='Fresh absolute output destination; normal bundle only.')
    group.add_argument('--verify',type=Path,help='Verify a ZIP without extracting or executing it.')
    parser.add_argument('--notices',type=Path,help='Notice collection directory bound to this bundle and locked dependencies.')
    args=parser.parse_args()
    try:
        if args.verify:
            manifest=verify_archive(args.verify)
            print(json.dumps({'ok':True,'files':len(manifest['files']),'platform':manifest['platform']}))
        else:
            if args.notices is None: raise ValueError('Collect dependency notices and pass their directory with --notices.')
            if os.name!='nt': raise ValueError('Windows packaging must run on Windows.')
            if not args.directory.is_absolute(): raise ValueError('Use an absolute output directory.')
            # Check the lexical bundle path before resolving links.
            if is_link(BUNDLE): raise ValueError('Bundle links are not allowed.')
            path=package(BUNDLE,args.directory,args.notices)
            print(json.dumps({'ok':True,'archive':str(path),'bytes':path.stat().st_size,'files':len(RUNTIME|DOCUMENTS)}))
    except (OSError, ValueError, zipfile.BadZipFile, KeyError, TypeError) as error:
        parser.exit(1,'Package refused: '+str(error)+'\n')

if __name__=='__main__': main()
