"""Credential-free checks of explicit selection and truthful live receipts."""
import contextlib
import importlib.util
import io
import json
from pathlib import Path
import sqlite3
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import Mock, patch

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('runner', ROOT/'scripts/qualify-companionship-live.py')
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


class RunnerTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='companion-runner-', dir=ROOT/'output')
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        self.source = self.directory/'source.db'
        with contextlib.closing(sqlite3.connect(self.source)) as db, db:
            db.execute('CREATE TABLE model_choices(models TEXT)')
            db.execute('INSERT INTO model_choices VALUES(?)', (json.dumps(['configured-model']),))
        self.output = self.directory/'receipt.json'
        self.argv = ['--source', str(self.source), '--output', str(self.output), '--model', 'configured-model']
        self.support = SimpleNamespace(digest=Mock(return_value=('unchanged', 1)),
                                       connection=Mock(return_value=('http://public.invalid/v1', 'synthetic')))
        self.data = {'model': 'configured-model', 'reasoning': 'providerDefault', 'requests': 2, 'tools': 0,
                     'cases': [{'kind': 'chat', 'pass': True}, {'kind': 'memory', 'pass': True}]}

    def run_receipt(self, data=None, returncode=0):
        data = self.data if data is None else data
        result = SimpleNamespace(stdout='COMPANION_LIVE='+json.dumps(data), returncode=returncode)
        with patch.object(runner, 'connection_support', return_value=self.support), \
             patch.object(runner.subprocess, 'run', return_value=result) as child, \
             contextlib.redirect_stdout(io.StringIO()):
            status = runner.main(self.argv)
        self.assertEqual(child.call_args.kwargs['env']['DOLORES_COMPANION_LIVE_MODEL'], 'configured-model')
        self.assertNotIn('DOLORES_COMPANION_LIVE_KEY', child.call_args.kwargs['env'])
        receipt = json.loads(self.output.read_text())
        self.assertNotIn('synthetic', self.output.read_text())
        return status, receipt

    def test_explicit_enabled_model_and_preserved_source(self):
        status, receipt = self.run_receipt()
        self.assertEqual(status, 0)
        self.assertTrue(receipt['passed'] and receipt['originalProfileUnchanged'])

    def test_reasoning_is_explicit_and_mismatched_receipt_is_not_success(self):
        self.argv += ['--reasoning', 'deepseekThinkingOff']
        status, receipt = self.run_receipt()
        self.assertEqual(status, 1)
        self.assertFalse(receipt['passed'])
        self.output.unlink()
        self.data['reasoning'] = 'deepseekThinkingOff'
        status, receipt = self.run_receipt()
        self.assertEqual(status, 0)
        self.assertEqual(receipt['reasoning'], 'deepseekThinkingOff')

    def test_invalid_model_or_existing_receipt_never_reads_credentials_or_starts_child(self):
        for argv in [self.argv[:-1]+['missing-model'], self.argv]:
            if argv == self.argv:
                self.output.write_text('retain previous evidence')
            with patch.object(runner, 'connection_support') as support, \
                 patch.object(runner.subprocess, 'run') as child, \
                 contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit):
                runner.main(argv)
            support.assert_not_called()
            child.assert_not_called()
        self.assertEqual(self.output.read_text(), 'retain previous evidence')

    def test_failed_case_or_child_or_changed_profile_cannot_report_success(self):
        for kind in ('case', 'child', 'profile'):
            with self.subTest(kind=kind):
                data = json.loads(json.dumps(self.data))
                if kind == 'case':
                    data['cases'][1]['pass'] = False
                if kind == 'profile':
                    self.support.digest.side_effect = [('before', 1), ('after', 1)]
                status, receipt = self.run_receipt(data, returncode=1 if kind == 'child' else 0)
                self.assertEqual(status, 1)
                self.assertFalse(receipt['passed'])
                self.output.unlink()


if __name__ == '__main__':
    unittest.main()
