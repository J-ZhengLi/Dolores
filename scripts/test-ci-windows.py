"""Pinned SDK metadata, early refusal and release-tag checkout regressions."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('windows_ci', Path(__file__).with_name('ci-windows.py'))
ci = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ci)


class WindowsCiTests(unittest.TestCase):
    def check_sdk(self, root, version, revision, full=False):
        cache = root / 'sdk/bin/cache'
        cache.mkdir(parents=True, exist_ok=True)
        (cache / 'flutter.version.json').write_text(json.dumps({
            'frameworkVersion': version, 'frameworkRevision': revision,
        }), encoding='utf-8')
        with patch.object(ci, 'OUTPUT', root / 'evidence'), \
                patch.object(ci, 'sdk', return_value=root / 'sdk'), \
                patch.object(ci.desktop, 'flutter_path', return_value='flutter.bat'), \
                patch.object(ci.release, 'check'), patch.object(ci, 'run') as run:
            try:
                ci.check(full=full)
            finally:
                self.calls = [str(call.args[0][0]) for call in run.call_args_list]
                self.commands = [list(map(str, call.args[0])) for call in run.call_args_list]

    def test_unknown_version_refuses_before_compilation_then_recovers(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            with self.assertRaisesRegex(ValueError, 'release tags'):
                self.check_sdk(root, '0.0.0-unknown', ci.FLUTTER_REVISION)
            self.assertNotIn('cargo', self.calls)
            self.assertEqual(self.calls, ['flutter.bat'])
            self.check_sdk(root, '3.47.5', ci.FLUTTER_REVISION)
            self.assertIn('cargo', self.calls)

    def test_wrong_revision_refuses_before_compilation(self):
        with tempfile.TemporaryDirectory() as folder:
            with self.assertRaisesRegex(ValueError, 'pinned Flutter'):
                self.check_sdk(Path(folder), '3.47.5', '0' * 40)
            self.assertNotIn('cargo', self.calls)

    def test_pinned_flutter_checkout_includes_release_tags(self):
        for name in ('windows.yml', 'release.yml'):
            workflow = (ci.ROOT / '.github/workflows' / name).read_text(encoding='utf-8')
            sdk_checkout = workflow.split('repository: flutter/flutter', 1)[1].split('- name:', 1)[0]
            self.assertIn('ref: ' + ci.FLUTTER_REVISION, sdk_checkout)
            self.assertRegex(sdk_checkout, r'(?m)^\s+fetch-depth: 0\s*$')

    def test_ordinary_checks_skip_release_build_and_native_fixtures(self):
        with tempfile.TemporaryDirectory() as folder:
            self.check_sdk(Path(folder), '3.47.5', ci.FLUTTER_REVISION)
            self.assertFalse(any('build' in c or '--native-only' in c for c in self.commands))
            self.check_sdk(Path(folder), '3.47.5', ci.FLUTTER_REVISION, full=True)
            self.assertTrue(any('build' in c for c in self.commands))
            self.assertTrue(any('--native-only' in c for c in self.commands))

    def test_only_release_workflow_packages_and_uploads_installers(self):
        checks = (ci.ROOT / '.github/workflows/windows.yml').read_text(encoding='utf-8')
        self.assertNotIn("tags: ['v*']", checks)
        self.assertNotIn('ci-windows.py package', checks)
        self.assertNotIn('installer.py windows', checks)
        workflow = (ci.ROOT / '.github/workflows/release.yml').read_text(encoding='utf-8')
        self.assertIn("tags: ['v*']", workflow)
        self.assertNotIn("branches: ['**']", workflow)
        self.assertIn('ci-windows.py check --full', workflow)
        self.assertIn('installer.py windows', workflow)
        self.assertIn('upload-artifact@', workflow)
        self.assertIn('if: github.ref_type == \'tag\'', workflow)
        self.assertIn('--verify-tag', workflow)


if __name__ == '__main__':
    unittest.main()
