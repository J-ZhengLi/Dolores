"""Package completeness, integrity and private-file/missing-runtime recovery."""
import importlib.util
import io
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

    def test_complete_archive_hashes_and_no_overwrite(self):
        with tempfile.TemporaryDirectory(dir=packaging.ROOT/'output') as temp:
            directory=Path(temp);bundle=self.fixture(directory);destination=directory/'release'
            archive=packaging.package(bundle,destination)
            manifest=packaging.verify_archive(archive)
            self.assertEqual(len(manifest['files']),len(packaging.RUNTIME|packaging.DOCUMENTS))
            self.assertFalse(manifest['signed'])
            self.assertTrue(archive.with_suffix('.zip.sha256').is_file())
            prior=archive.read_bytes()
            with self.assertRaisesRegex(ValueError,'exists'): packaging.package(bundle,destination)
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
            original=bridge.read_bytes();bridge.unlink()
            with self.assertRaisesRegex(ValueError,'Missing'): packaging.package(bundle,directory/'missing')
            self.assertFalse((directory/'missing').exists())
            bridge.write_bytes(original)
            private=bundle/'data/dolores.db';private.write_bytes(b'SYNTHETIC PRIVATE DATA')
            with self.assertRaisesRegex(ValueError,'Unexpected'): packaging.package(bundle,directory/'private')
            self.assertFalse((directory/'private').exists());private.unlink()
            invalid=bytearray(original);invalid[-2:]=b'\x4c\x01';bridge.write_bytes(invalid)
            with self.assertRaisesRegex(ValueError,'x64'): packaging.package(bundle,directory/'wrong-arch')
            bridge.write_bytes(original)
            bridge.write_bytes(original+str(packaging.ROOT).encode('utf-16-le'))
            with self.assertRaisesRegex(ValueError,'Local build path'): packaging.package(bundle,directory/'build-path')
            self.assertFalse((directory/'build-path').exists())
            bridge.write_bytes(original)
            self.assertTrue(packaging.package(bundle,directory/'recovered').is_file())

if __name__=='__main__': unittest.main()
