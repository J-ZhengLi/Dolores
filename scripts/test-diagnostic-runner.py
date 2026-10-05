"""Diagnostic failure recovery owns only its newly launched process."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import Mock, patch

spec = importlib.util.spec_from_file_location('diagnostic', Path(__file__).with_name('test-desktop-diagnostic.py'))
diagnostic = importlib.util.module_from_spec(spec); spec.loader.exec_module(diagnostic)


class RunnerTests(unittest.TestCase):
    def test_failed_report_stops_own_process_and_keeps_report(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary); report = directory / 'report.json'
            report.write_text(json.dumps({'ok':False, 'error':'synthetic failure'}))
            process = Mock(); process.poll.return_value = None
            with patch.object(diagnostic.subprocess, 'Popen', return_value=process):
                with self.assertRaisesRegex(RuntimeError, 'reported failure'):
                    diagnostic.run_phase(directory, {}, 'report.json', 10)
            process.terminate.assert_called_once(); process.wait.assert_called_once()
            self.assertTrue(report.exists())

    def test_deadline_stops_process_without_false_success(self):
        with tempfile.TemporaryDirectory() as temporary:
            process = Mock(); process.poll.return_value = None
            with patch.object(diagnostic.subprocess, 'Popen', return_value=process), patch.object(diagnostic.time, 'monotonic', side_effect=[0, 11]):
                with self.assertRaisesRegex(RuntimeError, 'deadline reached'):
                    diagnostic.run_phase(Path(temporary), {}, 'report.json', 10)
            process.terminate.assert_called_once(); process.wait.assert_called_once()


if __name__ == '__main__':
    unittest.main()
