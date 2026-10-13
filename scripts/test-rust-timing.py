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
            subprocess.CompletedProcess([], 0, '3 passed; 0 failed', '') for _ in range(3)]
        with patch.object(timing.subprocess, 'run', side_effect=results):
            with self.assertRaisesRegex(ValueError, 'repetition 1 failed; no retry'):
                timing.repeat(Path('synthetic-test-binary'), 4, 1)

    def test_empty_test_filter_refuses_then_correct_filter_recovers(self):
        with patch.object(timing.subprocess, 'run', return_value=subprocess.CompletedProcess([], 0, '0 passed; 0 failed', '')):
            with self.assertRaisesRegex(ValueError, 'failed; no retry'):
                timing.repeat(Path('synthetic-test-binary'), 1, 1)
        with patch.object(timing.subprocess, 'run', return_value=subprocess.CompletedProcess([], 0, '3 passed; 0 failed', '')):
            result = timing.repeat(Path('synthetic-test-binary'), 2, 1)
            self.assertEqual(result['testCases'], 6)
            self.assertEqual(result['retries'], 0)


if __name__ == '__main__':
    unittest.main()
