"""Pinned install, failed-download recovery and existing-directory preservation."""
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from types import SimpleNamespace

spec = importlib.util.spec_from_file_location('installer', Path(__file__).with_name('install-browser-adapter.py'))
installer = importlib.util.module_from_spec(spec); spec.loader.exec_module(installer)


class InstallerTests(unittest.TestCase):
    def test_pinned_install_is_atomic_and_repeat_is_offline(self):
        with tempfile.TemporaryDirectory() as temporary:
            destination = Path(temporary) / 'adapter space 测试'
            calls = []
            def download(argv, **kwargs):
                calls.append(argv)
                metadata = kwargs['cwd'] / 'node_modules/playwright-core/package.json'
                metadata.parent.mkdir(parents=True)
                pinned = json.loads((installer.SOURCE / 'package.json').read_bytes())['dependencies']['playwright-core']
                metadata.write_text(json.dumps({'version': pinned}))
                self.assertFalse(destination.exists())
                self.assertIn('--ignore-scripts', argv)
                return SimpleNamespace(returncode=0)
            self.assertEqual(installer.install(destination, 'node', 'npm-cli.js', download), 'Installed')
            self.assertEqual(installer.install(destination, 'node', 'npm-cli.js', download), 'Already installed')
            self.assertEqual(len(calls), 1)

    def test_download_failure_and_timeout_leave_no_partial_destination(self):
        with tempfile.TemporaryDirectory() as temporary:
            destination = Path(temporary) / 'adapter'
            for runner in [lambda *a, **kw: SimpleNamespace(returncode=1),
                           lambda *a, **kw: (_ for _ in ()).throw(subprocess.TimeoutExpired('npm', 90))]:
                with self.assertRaisesRegex(ValueError, 'Destination was not changed'):
                    installer.install(destination, 'node', 'npm-cli.js', runner)
                self.assertFalse(destination.exists())
                self.assertEqual(list(Path(temporary).iterdir()), [])

    def test_existing_foreign_directory_is_retained(self):
        with tempfile.TemporaryDirectory() as temporary:
            destination = Path(temporary) / 'adapter'; destination.mkdir()
            (destination / 'keep').write_text('untouched')
            with self.assertRaisesRegex(ValueError, 'Retained'):
                installer.install(destination, 'node', 'npm-cli.js')
            self.assertEqual((destination / 'keep').read_text(), 'untouched')


if __name__ == '__main__':
    unittest.main()
