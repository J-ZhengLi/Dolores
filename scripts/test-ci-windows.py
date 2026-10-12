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
    def check_sdk(self, root, version, revision):
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
                ci.check()
            finally:
                self.calls = [str(call.args[0][0]) for call in run.call_args_list]

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
        workflow = (ci.ROOT / '.github/workflows/windows.yml').read_text(encoding='utf-8')
        sdk_checkout = workflow.split('repository: flutter/flutter', 1)[1].split('- name:', 1)[0]
        self.assertIn('ref: ' + ci.FLUTTER_REVISION, sdk_checkout)
        self.assertRegex(sdk_checkout, r'(?m)^\s+fetch-depth: 0\s*$')


if __name__ == '__main__':
    unittest.main()
