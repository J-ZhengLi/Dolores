"""Package completeness, integrity and private-file/missing-runtime recovery."""
import importlib.util
import io
import hashlib
import json
from pathlib import Path
import struct
import sys
import tempfile
import unittest
import zipfile

sys.dont_write_bytecode=True
spec=importlib.util.spec_from_file_location('packaging',Path(__file__).with_name('package-windows.py'))
packaging=importlib.util.module_from_spec(spec);spec.loader.exec_module(packaging)

class PackageTests(unittest.TestCase):
    def test_private_path_across_stream_chunks(self):
        for path in (packaging.ROOT,Path.home()):
            for encoding in ('utf-8','utf-16-le'):
                with self.subTest(encoding=encoding):
                    prefix=str(path).upper().encode(encoding)
                    source=io.BytesIO(b'x'*(1024*1024-len(prefix)//2)+prefix+b'\0')
                    with self.assertRaisesRegex(ValueError,'Local build path'): packaging.sha(source,'synthetic.bin')

    def fixture(self,directory):
        bundle=directory/'bundle';bundle.mkdir()
        for name in packaging.RUNTIME:
            path=bundle/name;path.parent.mkdir(parents=True,exist_ok=True)
            value=b'runtime fixture'
            if name.endswith(('.exe','.dll')):
                value=bytearray(134);value[:2]=b'MZ';struct.pack_into('<I',value,60,128);value[128:]=b'PE\0\0\x64\x86'
            path.write_bytes(value)
        return bundle

    def notices(self,directory,bundle):
        result=directory/'notices';result.mkdir()
        text=b'SYNTHETIC MIT license fixture, not a real dependency audit.'
        inventory={'format':1,'platform':'windows-x64','components':[
            {'name':'synthetic','source':'https://example.invalid','notices':[{'file':'LICENSE'}]}],
            'runtime':{n:hashlib.sha256((bundle/n).read_bytes()).hexdigest() for n in packaging.RUNTIME},
            'inputs':{n:hashlib.sha256((packaging.ROOT/n).read_bytes()).hexdigest() for n in packaging.AUDIT_INPUTS},
            'noticesSha256':hashlib.sha256(text).hexdigest()}
        (result/'DEPENDENCIES.json').write_text(json.dumps(inventory),encoding='utf-8')
        (result/'THIRD-PARTY-NOTICES.txt').write_bytes(text)
        return result

    def test_complete_archive_hashes_and_no_overwrite(self):
        with tempfile.TemporaryDirectory(dir=packaging.ROOT/'output') as temp:
            directory=Path(temp);bundle=self.fixture(directory);destination=directory/'release'
            notices=self.notices(directory,bundle)
            archive=packaging.package(bundle,destination,notices)
            manifest=packaging.verify_archive(archive)
            self.assertEqual(len(manifest['files']),len(packaging.RUNTIME|packaging.DOCUMENTS))
            self.assertFalse(manifest['signed'])
            self.assertTrue(archive.with_suffix('.zip.sha256').is_file())
            prior=archive.read_bytes()
            with self.assertRaisesRegex(ValueError,'exists'): packaging.package(bundle,destination,notices)
            self.assertEqual(archive.read_bytes(),prior)
            damaged=directory/'damaged.zip'
            with zipfile.ZipFile(archive) as source,zipfile.ZipFile(damaged,'w') as target:
                for entry in source.infolist():
                    target.writestr(entry,b'changed' if entry.filename.endswith('USER_GUIDE.md') else source.read(entry))
            with self.assertRaises(ValueError): packaging.verify_archive(damaged)
            malformed=directory/'malformed.zip'
            with zipfile.ZipFile(archive) as source,zipfile.ZipFile(malformed,'w') as target:
                for entry in source.infolist():
                    target.writestr(entry,b'[]' if entry.filename.endswith('MANIFEST.json') else source.read(entry))
            with self.assertRaisesRegex(ValueError,'manifest'): packaging.verify_archive(malformed)

    def test_missing_private_and_wrong_arch_refuse_then_explicit_recovery(self):
        with tempfile.TemporaryDirectory(dir=packaging.ROOT/'output') as temp:
            directory=Path(temp);bundle=self.fixture(directory);bridge=bundle/'dolores_flutter_bridge.dll'
            notices=self.notices(directory,bundle)
            original=bridge.read_bytes();bridge.unlink()
            with self.assertRaisesRegex(ValueError,'Missing'): packaging.package(bundle,directory/'missing',notices)
            self.assertFalse((directory/'missing').exists())
            bridge.write_bytes(original)
            private=bundle/'data/dolores.db';private.write_bytes(b'SYNTHETIC PRIVATE DATA')
            with self.assertRaisesRegex(ValueError,'Unexpected'): packaging.package(bundle,directory/'private',notices)
            self.assertFalse((directory/'private').exists());private.unlink()
            invalid=bytearray(original);invalid[-2:]=b'\x4c\x01';bridge.write_bytes(invalid)
            with self.assertRaisesRegex(ValueError,'x64'): packaging.package(bundle,directory/'wrong-arch',notices)
            bridge.write_bytes(original)
            bridge.write_bytes(original+str(packaging.ROOT).encode('utf-16-le'))
            with self.assertRaisesRegex(ValueError,'Local build path'): packaging.package(bundle,directory/'build-path',notices)
            self.assertFalse((directory/'build-path').exists())
            bridge.write_bytes(original)
            self.assertTrue(packaging.package(bundle,directory/'recovered',notices).is_file())

    def test_stale_notices_and_changed_locks_refuse_then_recover(self):
        with tempfile.TemporaryDirectory(dir=packaging.ROOT/'output') as temp:
            directory=Path(temp);bundle=self.fixture(directory);notices=self.notices(directory,bundle)
            inventory_path=notices/'DEPENDENCIES.json';original=inventory_path.read_bytes()
            bridge=bundle/'dolores_flutter_bridge.dll';prior=bridge.read_bytes();bridge.write_bytes(prior+b'new build')
            with self.assertRaisesRegex(ValueError,'Stale'): packaging.package(bundle,directory/'stale',notices)
            self.assertFalse((directory/'stale').exists());bridge.write_bytes(prior)
            inventory=json.loads(original);inventory['inputs']['Cargo.lock']='0'*64
            inventory_path.write_text(json.dumps(inventory),encoding='utf-8')
            with self.assertRaisesRegex(ValueError,'inputs changed'): packaging.package(bundle,directory/'locks',notices)
            self.assertFalse((directory/'locks').exists());inventory_path.write_bytes(original)
            text=notices/'THIRD-PARTY-NOTICES.txt';prior_text=text.read_bytes();text.write_bytes(b'cut off')
            with self.assertRaisesRegex(ValueError,'mismatched'): packaging.package(bundle,directory/'truncated-notice',notices)
            text.write_bytes(prior_text)
            archive=packaging.package(bundle,directory/'recovered',notices)
            self.assertTrue(archive.is_file());self.assertEqual(inventory_path.read_bytes(),original)

if __name__=='__main__': unittest.main()
