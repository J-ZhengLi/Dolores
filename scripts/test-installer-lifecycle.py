"""Installer fixture must wait for late cleanup and execute its registered owner."""
import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('native_installer_fixture', Path(__file__).with_name('test-installer-native.py'))
native = importlib.util.module_from_spec(spec)
spec.loader.exec_module(native)


class LifecycleTests(unittest.TestCase):
    def test_app_removed_before_uninstaller_must_not_admit_reinstall(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            installed = root / 'installed'
            installed.mkdir()
            late = installed / 'unins000.exe'
            late.write_bytes(b'public late-cleanup fixture')
            data = root / 'data'
            data.mkdir()
            (data / 'retain.txt').write_text('retained public fixture')
            def complete_cleanup(_):
                late.unlink()
                installed.rmdir()
            with patch.object(native.time, 'sleep', side_effect=complete_cleanup) as sleep:
                native.wait_cleanup(installed, data, False)
            self.assertFalse(installed.exists(), 'Reinstallation must wait for the old uninstaller cleanup')
            sleep.assert_called_once()
            self.assertTrue((data / 'retain.txt').is_file())

    def test_numbered_registration_is_used_and_external_or_missing_owner_refuses(self):
        with tempfile.TemporaryDirectory() as folder:
            installed = Path(folder) / 'installed'
            installed.mkdir()
            executable = installed / 'unins001.exe'
            executable.write_bytes(b'public selected uninstaller fixture')
            executable.with_suffix('.dat').write_bytes(b'public matching uninstall log fixture')
            command = '"' + str(executable) + '"'
            self.assertEqual(native.validate_uninstaller(installed, str(installed), command), executable.resolve())
            for location, value in ((str(installed.parent), command),
                                    (str(installed), '"' + str(installed.parent / 'unins001.exe') + '"'),
                                    (str(installed), command + ' /unreviewed')):
                with self.assertRaisesRegex(ValueError, 'registration'):
                    native.validate_uninstaller(installed, location, value)
            executable.with_suffix('.dat').unlink()
            with self.assertRaisesRegex(ValueError, 'incomplete'):
                native.validate_uninstaller(installed, str(installed), command)

    def test_unrelated_nonzero_is_not_accepted_as_guard_refusal(self):
        with patch.object(native.subprocess, 'run', return_value=subprocess.CompletedProcess([], 5)):
            with self.assertRaisesRegex(ValueError, 'expected exit 1, got 5'):
                native.run(['public-fixture-command'], expected=1)

    def test_incomplete_cleanup_fails_instead_of_retrying_and_purge_must_finish(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            installed = root / 'installed'
            installed.mkdir()
            with patch.object(native.time, 'monotonic', side_effect=[0, 11]):
                with self.assertRaisesRegex(ValueError, 'cleanup is incomplete'):
                    native.wait_cleanup(installed, root / 'data', False)
            self.assertTrue(installed.is_dir())
            installed.rmdir()
            data = root / 'data'
            data.mkdir()
            with patch.object(native.time, 'monotonic', side_effect=[0, 11]):
                with self.assertRaisesRegex(ValueError, 'cleanup is incomplete'):
                    native.wait_cleanup(installed, data, True)
            self.assertTrue(data.is_dir())


if __name__ == '__main__':
    unittest.main()
