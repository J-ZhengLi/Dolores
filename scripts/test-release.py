"""Release notes, version mismatch and duplicate-release recovery fixtures."""
import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('release', Path(__file__).with_name('release.py'))
release = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release)


class ReleaseTests(unittest.TestCase):
    def fixture(self, root):
        (root / 'apps/dolores_flutter').mkdir(parents=True)
        (root / 'Cargo.toml').write_text('[workspace.package]\nversion="0.1.0"\n')
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
            (root / 'apps/dolores_flutter/pubspec.yaml').write_text('version: 0.2.0+1\n')
            with self.assertRaisesRegex(ValueError, 'must match'):
                release.prepare(root, '0.1.0', '2026-10-08')
            (root / 'apps/dolores_flutter/pubspec.yaml').write_text('version: 0.1.0+1\n')
            with self.assertRaisesRegex(ValueError, 'dated'):
                release.check(root, 'v0.1.0')
            release.prepare(root, '0.1.0', '2026-10-08')
            with self.assertRaisesRegex(ValueError, 'tag'):
                release.check(root, 'v0.2.0')
            self.assertEqual(release.check(root, 'v0.1.0'), '0.1.0')

    def test_alpha_preparation_aligns_versions_and_retains_previous_notes(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            self.fixture(root)
            (root / 'Cargo.lock').write_text('version = 4\n\n[[package]]\nname = "external"\nversion = "0.1.0"\nsource = "registry+https://example.invalid"\n')
            lock = (root / 'Cargo.lock').read_bytes()
            release.prepare(root, '0.0.1-alpha', '2026-10-13')
            self.assertEqual(release.check(root, 'v0.0.1-alpha'), '0.0.1-alpha')
            self.assertIn('version: 0.0.1-alpha+2', (root / 'apps/dolores_flutter/pubspec.yaml').read_text())
            self.assertEqual((root / 'Cargo.lock').read_bytes(), lock)
            first = release.notes(root, '0.0.1-alpha')
            with self.assertRaisesRegex(ValueError, 'already recorded'):
                release.prepare(root, '0.0.1-alpha', '2026-10-14')
            self.assertEqual(release.notes(root, '0.0.1-alpha'), first)

    def test_invalid_version_and_partial_write_preserve_sources(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            self.fixture(root)
            paths = [root / 'Cargo.toml', root / 'apps/dolores_flutter/pubspec.yaml', root / 'CHANGELOG.md']
            original = {p: p.read_bytes() for p in paths}
            for version in ('0.0.1-01', '01.0.1', '0.0.1-alpha/other'):
                with self.assertRaisesRegex(ValueError, 'SemVer'):
                    release.prepare(root, version, '2026-10-13')
            write = release.atomic_write
            calls = 0
            def interrupted(path, value):
                nonlocal calls
                calls += 1
                if calls == 2:
                    raise OSError('synthetic write interruption')
                write(path, value)
            with patch.object(release, 'atomic_write', side_effect=interrupted):
                with self.assertRaisesRegex(OSError, 'interruption'):
                    release.prepare(root, '0.0.1-alpha', '2026-10-13')
            self.assertEqual({p: p.read_bytes() for p in paths}, original)


if __name__ == '__main__':
    unittest.main()
