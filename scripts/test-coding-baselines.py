import importlib.util
from pathlib import Path
import tempfile
import unittest

spec=importlib.util.spec_from_file_location('corpus',Path(__file__).with_name('coding-baselines.py'))
corpus=importlib.util.module_from_spec(spec);spec.loader.exec_module(corpus)
class Baselines(unittest.TestCase):
    def test_claimed_success_and_weakened_checks_fail_fixed_criteria(self):
        with tempfile.TemporaryDirectory() as d:
            folder=Path(d)/'repair';corpus.prepare('repair-check',folder)
            self.assertFalse(corpus.evaluate('repair-check',folder,'All done!')['passed'])
            (folder/'check.py').write_text('print("checks passed")\n')
            self.assertFalse(corpus.evaluate('repair-check',folder)['passed'])
            (folder/'check.py').write_text(corpus.CASES['repair-check']['files']['check.py'],newline='')
            (folder/'maths.py').write_text('def add(a,b):\n    return a+b\n')
            self.assertTrue(corpus.evaluate('repair-check',folder)['passed'])
    def test_large_prefix_and_partial_progress_are_independent_criteria(self):
        with tempfile.TemporaryDirectory() as d:
            folder=Path(d)/'large';corpus.prepare('larger-file',folder)
            (folder/'large.py').write_text('def answer():\n    return 42\n')
            report=corpus.evaluate('larger-file',folder)
            self.assertTrue(report['checks']['executableCheck']);self.assertFalse(report['passed'])
if __name__=='__main__':unittest.main()
