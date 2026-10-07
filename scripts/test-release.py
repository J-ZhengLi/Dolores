"""Release notes, version mismatch and duplicate-release recovery fixtures."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('release', Path(__file__).with_name('release.py'))
release = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release)


class ReleaseTests(unittest.TestCase):
    def fixture(self, root):
        (root / 'apps/dolores_flutter').mkdir(parents=True)
        (root / 'Cargo.toml').write_text('[workspace.package]\nversion="0.1.0"\n')
        (root / 'package.json').write_text(json.dumps({'version': '0.1.0'}))
        (root / 'apps/dolores_flutter/pubspec.yaml').write_text('version: 0.1.0+1\n')
        (root / 'CHANGELOG.md').write_text('# Changelog\n\n## [Unreleased]\n\n### Fixed\n\n- Preserve drafts.\n')

    def test_prepare_check_and_exact_notes(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            self.fixture(root)
            release.prepare(root, '0.1.0', '2026-10-08')
            self.assertEqual(release.check(root, 'v0.1.0'), '0.1.0')
            self.assertIn('- Preserve drafts.', release.notes(root, '0.1.0'))
            self.assertEqual(release.notes(root, 'Unreleased').strip(), '## [Unreleased]')
            original = (root / 'CHANGELOG.md').read_bytes()
            with self.assertRaisesRegex(ValueError, 'already recorded'):
                release.prepare(root, '0.1.0', '2026-10-09')
            self.assertEqual((root / 'CHANGELOG.md').read_bytes(), original)

    def test_version_or_tag_mismatch_refuses_then_recovers(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            self.fixture(root)
            (root / 'package.json').write_text('{"version":"0.2.0"}')
            with self.assertRaisesRegex(ValueError, 'must match'):
                release.prepare(root, '0.1.0', '2026-10-08')
            (root / 'package.json').write_text('{"version":"0.1.0"}')
            with self.assertRaisesRegex(ValueError, 'dated'):
                release.check(root, 'v0.1.0')
            release.prepare(root, '0.1.0', '2026-10-08')
            with self.assertRaisesRegex(ValueError, 'tag'):
                release.check(root, 'v0.2.0')
            self.assertEqual(release.check(root, 'v0.1.0'), '0.1.0')


if __name__ == '__main__':
    unittest.main()
