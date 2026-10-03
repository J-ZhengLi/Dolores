"""Recovery and honest-report checks for the runner itself; no native/model call."""
import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import time
import unittest

sys.dont_write_bytecode = True

spec = importlib.util.spec_from_file_location('regressions',Path(__file__).with_name('run-regressions.py'))
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)

class RunnerTests(unittest.TestCase):
    def test_receipts_and_incomplete_scope_do_not_pass(self):
        with tempfile.TemporaryDirectory() as temp:
            directory=Path(temp); log=directory/'receipt.log'
            for value in [{'ok':True,'stage':'restore','fixtureRequests':0,'liveRequests':0},
                          {'ok':True,'stage':'save','fixtureRequests':0,'liveRequests':1},
                          {'ok':True,'stage':'save','fixtureRequests':0,'liveRequests':False},
                          {'ok':True,'stage':'save','liveRequests':0}]:
                log.write_text(json.dumps(value),encoding='utf-8')
                with self.assertRaises(ValueError): runner.validate_receipt(log,'save')
            log.write_text('Optional tool banner\n{"ok":true,"stage":"save","fixtureRequests":2,"liveRequests":0}\n',encoding='utf-8')
            self.assertEqual(runner.validate_receipt(log,'save')['fixtureRequests'],2)
            report={'mode':'test','expectedSteps':2,'steps':[{'name':'one','status':'passed','elapsedSeconds':0}]}
            runner.write_report(directory,report)
            self.assertFalse(json.loads((directory/'report.json').read_text())['ok'])
            self.assertTrue((directory/'report.md').exists())

    def test_nonzero_invalid_receipt_timeout_and_fresh_retry_keep_evidence(self):
        with tempfile.TemporaryDirectory() as temp:
            directory=Path(temp)
            def run(name,code,timeout=5,stage=None):
                return runner.run_step(name,[sys.executable,'-c',code],directory,directory,timeout,stage)
            failed=run('nonzero','print("usable work",flush=True);raise SystemExit(2)')
            self.assertEqual(failed['status'],'failed')
            self.assertIn('usable work',(directory/'nonzero.log').read_text())
            self.assertEqual(run('malformed','print("{invalid")',stage='save')['status'],'failed')
            started=time.monotonic()
            child="import time;from pathlib import Path;time.sleep(3);Path('late-write').write_text('unexpected')"
            parent=f"import subprocess,sys,time;subprocess.Popen([sys.executable,'-c',{child!r}]);print('partial',flush=True);time.sleep(30)"
            self.assertEqual(run('timeout',parent,.5)['status'],'timedOut')
            self.assertLess(time.monotonic()-started,12)
            self.assertIn('partial',(directory/'timeout.log').read_text())
            time.sleep(3.2)
            self.assertFalse((directory/'late-write').exists())
            self.assertEqual(run('retry','print("explicit recovery")')['status'],'passed')
            self.assertTrue((directory/'nonzero.log').exists())

if __name__=='__main__': unittest.main()
