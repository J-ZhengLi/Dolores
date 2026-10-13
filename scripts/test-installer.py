"""Installer payload refusal/recovery, install scopes and data retention contracts."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import installer

spec = importlib.util.spec_from_file_location('portable_tests', Path(__file__).with_name('test-package-windows.py'))
fixtures = importlib.util.module_from_spec(spec)
spec.loader.exec_module(fixtures)


class InstallerTests(unittest.TestCase):
    def test_verified_payload_compiles_without_overwriting_previous_installer(self):
        with tempfile.TemporaryDirectory(dir=installer.ROOT / 'output') as folder:
            root = Path(folder)
            fixture = fixtures.PackageTests()
            bundle = fixture.fixture(root)
            notices = fixture.notices(root, bundle)
            archive = fixtures.packaging.package(bundle, root / 'portable', notices)
            compiler = root / 'ISCC.exe'
            compiler.write_bytes(b'synthetic compiler fixture')
            version = installer.release.app_version()
            destination = root / 'installer'
            def compile_fixture(command, check):
                self.assertIn('/DAppVersion=' + version, command)
                (destination / f'Dolores-{version}-windows-x64-setup.exe').write_bytes(b'MZsynthetic fixture, not a real installer')
            with patch.object(installer.subprocess, 'run', side_effect=compile_fixture):
                asset = installer.windows(archive, destination, compiler)
            self.assertTrue(asset.with_suffix('.exe.sha256').is_file())
            self.assertEqual((destination / 'staging/Dolores/dolores_flutter.exe').read_bytes(), (bundle / 'dolores_flutter.exe').read_bytes())
            original = asset.read_bytes()
            with self.assertRaisesRegex(ValueError, 'exists'):
                installer.windows(archive, destination, compiler)
            self.assertEqual(asset.read_bytes(), original)
            with self.assertRaisesRegex(ValueError, 'compiler missing'):
                installer.windows(archive, root / 'missing-tool', root / 'missing.exe')
            self.assertFalse((root / 'missing-tool').exists())
            with self.assertRaisesRegex(ValueError, 'beneath output'):
                installer.fresh(installer.ROOT)

    def test_tampered_payload_refuses_before_compiler(self):
        with tempfile.TemporaryDirectory(dir=installer.ROOT / 'output') as folder:
            root = Path(folder)
            compiler = root / 'ISCC.exe'
            compiler.write_bytes(b'fixture')
            archive = root / 'bad.zip'
            archive.write_bytes(b'not a ZIP')
            with patch.object(installer.subprocess, 'run') as run:
                with self.assertRaises(Exception):
                    installer.windows(archive, root / 'bad', compiler)
                run.assert_not_called()
                self.assertFalse((root / 'bad').exists())

    def test_scopes_and_uninstall_are_opt_in_and_bounded(self):
        script = (installer.ROOT / 'packaging/windows/dolores.iss').read_text(encoding='utf-8')
        self.assertIn('PrivilegesRequiredOverridesAllowed=dialog', script)
        self.assertIn('DefaultDirName={autopf}\\Dolores', script)
        self.assertIn('{autoprograms}', script)
        self.assertIn('RemoveBox.Checked := False', script)
        self.assertIn('if UninstallSilent then exit', script)
        self.assertIn("{userappdata}\\dev.dolores.desktop", script)
        self.assertIn("CreateFileW(DataDirectory + '\\dolores.lock', $C0000000, 0", script)
        self.assertIn('SafeDataTree(DataDirectory)', script)
        self.assertNotIn('[UninstallDelete]', script)
        self.assertNotIn('{commonappdata}', script)

    def test_native_packages_refuse_wrong_host_and_foreign_notices(self):
        with tempfile.TemporaryDirectory(dir=installer.ROOT / 'output') as folder:
            root = Path(folder)
            bundle = root / 'bundle'
            bundle.mkdir()
            (bundle / 'binary').write_bytes(b'public fixture')
            notices = root / 'notices'
            notices.mkdir()
            (notices / 'DEPENDENCIES.json').write_text(json.dumps({'platform': 'windows-x64'}))
            (notices / 'THIRD-PARTY-NOTICES.txt').write_text('fixture')
            with self.assertRaisesRegex(ValueError, 'platform/runtime'):
                installer.native_notices(bundle, notices, 'linux-x64')
            with patch.object(installer.sys, 'platform', 'win32'):
                with self.assertRaisesRegex(ValueError, 'native operating system'):
                    installer.native('macos', root / 'macos', notices)


if __name__ == '__main__':
    unittest.main()
