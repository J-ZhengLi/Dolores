"""Windows batch preflight: incomplete extraction and missing runtime with explicit recovery."""
import importlib.util
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('portable',ROOT/'scripts/package-windows.py')
portable=importlib.util.module_from_spec(spec);spec.loader.exec_module(portable)

@unittest.skipUnless(os.name=='nt','Native batch behavior requires Windows')
class LauncherTests(unittest.TestCase):
    def test_missing_app_and_runtime_recovery_preserves_data(self):
        # Fake SystemRoot exists only in the child environment; no Windows DLL is changed.
        with tempfile.TemporaryDirectory(dir=ROOT/'output',prefix='launch-') as temp:
            root=Path(temp);app=root/'space & unicode 测试 (preview)';app.mkdir()
            launcher=app/'Start-Dolores.cmd';shutil.copyfile(ROOT/'scripts/Start-Dolores.cmd',launcher)
            system=root/'fake-system';(system/'System32').mkdir(parents=True)
            env=os.environ.copy();env['SystemRoot']=str(system)
            env['PROCESSOR_ARCHITECTURE']='AMD64';env.pop('PROCESSOR_ARCHITEW6432',None)
            history=root/'data.db';history.write_bytes(b'SYNTHETIC HISTORY')
            def check():
                # /s needs the outer quote pair to preserve a quoted batch path containing &/spaces.
                line='"'+os.environ.get('COMSPEC','cmd.exe')+'" /d /s /c ""'+str(launcher)+'" --check"'
                return subprocess.run(line,
                                      env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,timeout=10)
            result=check();self.assertEqual(result.returncode,2,result.stdout)
            self.assertIn(b'Extract the whole ZIP',result.stdout)
            for name in portable.RUNTIME:
                path=app/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(b'fixture')
            result=check();self.assertEqual(result.returncode,3)
            self.assertIn(b'Redistributable for x64',result.stdout);self.assertIn(b'learn.microsoft.com',result.stdout)
            for name in ('msvcp140.dll','vcruntime140.dll','vcruntime140_1.dll'):
                (system/'System32'/name).write_bytes(b'presence-only runtime fixture')
            result=check();self.assertEqual(result.returncode,0)
            self.assertIn(b'does not verify runtime versions',result.stdout)
            bridge=app/'dolores_flutter_bridge.dll';bridge.write_bytes(b'')
            self.assertEqual(check().returncode,2);bridge.write_bytes(b'fixture')
            self.assertEqual(check().returncode,0);self.assertEqual(history.read_bytes(),b'SYNTHETIC HISTORY')
            self.assertFalse(any(p.name.endswith('.log') for p in app.rglob('*')))

if __name__=='__main__': unittest.main()
