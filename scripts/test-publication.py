"""Synthetic publication history and explicit live-model selection checks."""
import importlib.util
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

from publication_rules import excluded, sanitize

ROOT = Path(__file__).resolve().parents[1]


class PublicationTests(unittest.TestCase):
    def test_portable_cleanup_and_binary_preservation(self):
        path = 'C:' + '/Users/' + 'example-person/.codex/RTK.md'
        original = ('@' + path + "\nUse the configured provider's Qwen3.5-2B model for routine live tests and DeepSeek V4.1 Flash for harder live cases.\n").encode()
        clean = sanitize('AGENTS.md', original)
        self.assertNotIn(path.encode(), clean)
        self.assertNotIn(b'Qwen', clean)
        self.assertEqual(sanitize('AGENTS.md', clean), clean)
        self.assertEqual(sanitize('art.ico', b'\0\xff\x01'), b'\0\xff\x01')
        self.assertTrue(excluded('docs/HANDOFF.md'))
        self.assertTrue(excluded('docs/design/developer-workspace-handoff.md'))
        self.assertFalse(excluded('docs/UI.md'))

    def test_live_checks_refuse_implicit_model_before_creating_output(self):
        ROOT.joinpath('output').mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(dir=ROOT / 'output') as temp:
            destination = Path(temp) / 'must-not-exist'
            for name in ('qualify-memory-live.py', 'qualify-scheduling-live.py'):
                result = subprocess.run([sys.executable, ROOT / 'scripts' / name,
                                         '--directory', destination, '--source', Path(temp) / 'missing.db'],
                                        capture_output=True, timeout=15)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(b'--model', result.stderr)
                self.assertFalse(destination.exists())
            result = subprocess.run([sys.executable, ROOT / 'scripts/qualify-background-scheduling.py',
                                     '--directory', destination, '--live-source', Path(temp) / 'missing.db'],
                                    capture_output=True, timeout=15)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn(b'--live-model', result.stderr)
            self.assertFalse(destination.exists())

    @unittest.skipUnless(importlib.util.find_spec('git_filter_repo'), 'Optional history tool not installed')
    def test_history_copy_removes_handoff_keeps_commits_and_attribution(self):
        spec = importlib.util.spec_from_file_location('prepare_publication', ROOT / 'scripts/prepare-publication.py')
        tool = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(tool)
        ROOT.joinpath('output').mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(dir=ROOT / 'output') as temp:
            source = Path(temp) / 'source'
            source.mkdir()
            def git(*args):
                return subprocess.check_output(['git', '-C', str(source), *args]).decode().strip()
            git('init', '-q')
            git('config', 'user.name', 'Example contributor')
            git('config', 'user.email', 'contributor@example.invalid')
            (source / 'scripts').mkdir()
            for name in ('check-publication.py', 'publication_rules.py'):
                shutil.copyfile(ROOT / 'scripts' / name, source / 'scripts' / name)
            (source / 'HANDOFF.md').write_text('Synthetic session detail; must disappear.\n')
            git('add', '.')
            git('commit', '-qm', 'Add synthetic files')
            (source / 'HANDOFF.md').write_text('Another synthetic session detail.\n')
            git('add', '.')
            git('commit', '-qm', 'Update synthetic handoff')
            before = git('show-ref')
            tool.ROOT = source
            destination = source / 'output/publication'
            receipt = tool.prepare(destination)
            self.assertEqual(receipt['commits'], 2)
            self.assertTrue(receipt['authorAttributionRetained'])
            self.assertEqual(git('show-ref'), before)
            self.assertEqual((source / 'HANDOFF.md').read_text(), 'Another synthetic session detail.\n')
            names = subprocess.check_output(['git', '-C', str(destination / 'dolores.git'),
                                             'log', '--all', '--format=', '--name-only'])
            self.assertNotIn(b'HANDOFF.md', names)
            with self.assertRaisesRegex(ValueError, 'fresh destination'):
                tool.prepare(destination)
            (source / 'new.txt').write_text('Uncommitted work stays local.')
            with self.assertRaisesRegex(ValueError, 'Commit'):
                tool.prepare(source / 'output/refused')
            self.assertFalse((source / 'output/refused').exists())


if __name__ == '__main__':
    unittest.main()
