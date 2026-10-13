"""Local CI preserves failed evidence, refuses reused output and cannot publish."""
import argparse
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('local_ci', Path(__file__).with_name('local-ci.py'))
local = importlib.util.module_from_spec(spec)
spec.loader.exec_module(local)


class LocalCiTests(unittest.TestCase):
    def test_failed_step_stops_and_retains_the_report(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            args = argparse.Namespace(command='release', cpus=2, directory=root / 'output/run', flutter_sdk=None,
                                      generator='Visual Studio 17 2022', cmake=None)
            with patch.object(local, 'ROOT', root), patch.object(local, 'affinity'), \
                    patch.object(local.sys, 'version_info', (3, 13)), \
                    patch.object(local.subprocess, 'check_output', side_effect=['v24.15.0', 'rustc 1.95.0 fixture', 'a' * 40, 'M owned-fixture']), \
                    patch.object(local.subprocess, 'run', return_value=subprocess.CompletedProcess([], 1)) as run, \
                    patch.dict(local.os.environ, {'GITHUB_REF_TYPE': 'tag', 'GITHUB_REF_NAME': 'v0.0.1-alpha'}):
                with self.assertRaisesRegex(ValueError, 'start a fresh run'):
                    local.execute(args)
                self.assertEqual(run.call_count, 1)
                self.assertNotIn('GITHUB_REF_TYPE', run.call_args.kwargs['env'])
                report = json.loads((args.directory / 'RESULT.json').read_text())
                self.assertEqual(report['status'], 'failed')
                self.assertFalse(report['published'])
                self.assertEqual(report['steps'][0]['exitCode'], 1)
                self.assertTrue((args.directory / '01.log').is_file())
                with self.assertRaises(FileExistsError):
                    local.output(args.directory)
                fresh = local.output(root / 'output/recovery')
                self.assertTrue(fresh.is_dir())
                self.assertTrue((args.directory / 'RESULT.json').is_file())

    def test_output_outside_owned_directory_is_refused(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            with patch.object(local, 'ROOT', root):
                for path in (root, root / 'output', root / 'unrelated'):
                    with self.assertRaisesRegex(ValueError, 'strictly beneath output'):
                        local.output(path)
                self.assertFalse((root / 'unrelated').exists())


if __name__ == '__main__':
    unittest.main()
