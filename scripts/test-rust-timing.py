"""Timing gate must fail on one bad repetition or an accidentally empty filter."""
import importlib.util
from pathlib import Path
import subprocess
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('rust_timing', Path(__file__).with_name('check-rust-timing.py'))
timing = importlib.util.module_from_spec(spec)
spec.loader.exec_module(timing)


class TimingGateTests(unittest.TestCase):
    def test_a_failed_repetition_is_not_hidden_by_later_passes(self):
        results = [subprocess.CompletedProcess([], 1, 'synthetic deadline failure', '')] + [
            subprocess.CompletedProcess([], 0, 'test result: ok. 3 passed; 0 failed; 0 ignored;', '') for _ in range(3)]
        with patch.object(timing.subprocess, 'run', side_effect=results):
            with self.assertRaisesRegex(ValueError, 'repetition 1 failed; no retry'):
                timing.repeat(Path('synthetic-test-binary'), 4, 1)

    def test_empty_test_filter_refuses_then_correct_filter_recovers(self):
        with patch.object(timing.subprocess, 'run', return_value=subprocess.CompletedProcess([], 0, 'test result: ok. 0 passed; 0 failed; 0 ignored;', '')):
            with self.assertRaisesRegex(ValueError, 'failed; no retry'):
                timing.repeat(Path('synthetic-test-binary'), 1, 1)
        with patch.object(timing.subprocess, 'run', return_value=subprocess.CompletedProcess([], 0, 'test result: ok. 3 passed; 0 failed; 0 ignored;', '')):
            result = timing.repeat(Path('synthetic-test-binary'), 2, 1)
            self.assertEqual(result['testCases'], 6)
            self.assertEqual(result['retries'], 0)

    def test_mcp_gate_exercises_both_recovery_cases_and_refuses_missing_cases(self):
        with patch.object(timing.subprocess, 'run', return_value=subprocess.CompletedProcess([], 0, 'test result: ok. 3 passed; 0 failed; 0 ignored;', '')) as run:
            result = timing.repeat(Path('synthetic-test-binary'), 2, 1, 'mcp')
            self.assertEqual(result['suite'], 'mcp')
            self.assertIn('mcp::tests::', run.call_args.args[0])
        with patch.object(timing.subprocess, 'run', return_value=subprocess.CompletedProcess([], 0, 'test result: ok. 2 passed; 0 failed; 0 ignored;', '')):
            with self.assertRaisesRegex(ValueError, 'failed; no retry'):
                timing.repeat(Path('synthetic-test-binary'), 1, 1, 'mcp')

    def test_protocol_gate_refuses_whole_suite_or_ignored_case(self):
        for summary in ('11 passed; 0 failed; 0 ignored;', '1 passed; 0 failed; 1 ignored;'):
            with patch.object(timing.subprocess, 'run', return_value=subprocess.CompletedProcess([], 0, 'test result: ok. ' + summary, '')):
                with self.assertRaisesRegex(ValueError, 'failed; no retry'):
                    timing.repeat(Path('synthetic-test-binary'), 1, 1, 'mcp-protocol')
        with patch.object(timing.subprocess, 'run', return_value=subprocess.CompletedProcess([], 0, 'test result: ok. 1 passed; 0 failed; 0 ignored;', '')):
            self.assertEqual(timing.repeat(Path('synthetic-test-binary'), 1, 1, 'mcp-protocol')['testCases'], 1)

    def test_language_gate_requires_real_delay_timeout_and_recovery_cases(self):
        for count in (0, 2, 3):
            with patch.object(timing.subprocess, 'run', return_value=subprocess.CompletedProcess([], 0, f'test result: ok. {count} passed; 0 failed; 0 ignored;', '')):
                with self.assertRaisesRegex(ValueError, 'failed; no retry'):
                    timing.repeat(Path('synthetic-test-binary'), 1, 1, 'language')
        with patch.object(timing.subprocess, 'run', return_value=subprocess.CompletedProcess([], 0, 'test result: ok. 4 passed; 0 failed; 0 ignored;', '')) as run:
            result = timing.repeat(Path('synthetic-test-binary'), 2, 1, 'language')
            self.assertEqual(result['testCases'], 8)
            self.assertEqual(result['retries'], 0)
            self.assertIn('language::tests::', run.call_args.args[0])


if __name__ == '__main__':
    unittest.main()
