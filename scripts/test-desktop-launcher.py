"""Build recovery and owned-launch safeguards, without a provider request."""
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('desktop', Path(__file__).with_name('desktop.py'))
desktop = importlib.util.module_from_spec(spec)
spec.loader.exec_module(desktop)


class DesktopTests(unittest.TestCase):
    def test_diagnostic_and_changed_binary_cannot_be_handed_off_as_normal(self):
        with tempfile.TemporaryDirectory() as temp:
            binary = Path(temp) / 'app.exe'; binary.write_bytes(b'fixture')
            state = Path(temp) / 'entry.json'; stat = binary.stat()
            state.write_text(json.dumps({'entry': 'smoke', 'size': stat.st_size, 'modifiedNs': stat.st_mtime_ns}))
            with patch.object(desktop, 'BUILD_STATE', state), patch.object(desktop, 'executable', return_value=binary):
                with self.assertRaisesRegex(ValueError, 'Diagnostic build'):
                    desktop.require_normal_build()
                self.assertEqual(desktop.require_normal_build(True), 'smoke')
                binary.write_bytes(b'changed fixture')
                with self.assertRaisesRegex(ValueError, 'identity changed'):
                    desktop.require_normal_build(True)

    def test_build_failure_restores_exact_package_configuration(self):
        with tempfile.TemporaryDirectory() as temp:
            app = Path(temp) / 'space & unicode 测试'; (app / '.dart_tool').mkdir(parents=True)
            path = app / '.dart_tool/package_config.json'
            original = b'{"packages":[{"name":"dolores_flutter","languageVersion":"3.13"}]}\n'
            path.write_bytes(original)
            with self.assertRaisesRegex(RuntimeError, 'build failed'):
                with desktop.registrant_alias(app):
                    self.assertEqual(json.loads(path.read_bytes())['packages'][-1]['name'], 'dolores_build_registrant')
                    raise RuntimeError('build failed')
            self.assertEqual(path.read_bytes(), original)

    def test_stale_or_foreign_preview_is_never_stopped(self):
        with tempfile.TemporaryDirectory() as temp:
            record = Path(temp) / 'preview.json'
            expected = {'pid': 123, 'executable': str(desktop.executable()), 'created': 100}
            record.write_text(json.dumps(expected))
            with patch.object(desktop, 'process_identity', return_value=dict(expected, created=101)), patch.object(desktop, 'checked') as command:
                with self.assertRaisesRegex(ValueError, 'identity changed'):
                    desktop.stop_owned(record)
                command.assert_not_called()

    @unittest.skipUnless(os.name == 'nt', 'Windows junctions')
    def test_plugin_junction_spaces_and_unexpected_directory_refusal(self):
        with tempfile.TemporaryDirectory() as temp:
            app = Path(temp) / 'app'; app.mkdir()
            target = Path(temp) / 'plugin space & 测试'; target.mkdir()
            (app / '.flutter-plugins-dependencies').write_text(json.dumps({'plugins': {'windows': [{'name': 'fixture', 'path': str(target)}]}}))
            desktop.plugin_links(app)
            desktop.plugin_links(app)
            link = app / 'windows/flutter/ephemeral/.plugin_symlinks/fixture'
            self.assertEqual(link.resolve(), target)
            # Removing a junction removes the link, not the plugin cache.
            link.rmdir(); link.mkdir(); (link / 'preserved').write_text('keep')
            with self.assertRaisesRegex(ValueError, 'Unexpected existing'):
                desktop.plugin_links(app)
            self.assertEqual((link / 'preserved').read_text(), 'keep')
            self.assertTrue(target.is_dir())


if __name__ == '__main__':
    unittest.main()
