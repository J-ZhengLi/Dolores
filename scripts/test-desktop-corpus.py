"""Check qualification rejects wrong card mappings and invalid generated views."""
import json,runpy,subprocess,sys,tempfile,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
visual_checks=runpy.run_path(str(ROOT/'scripts/qualify-desktop.py'))['visual_checks']

class CorpusChecks(unittest.TestCase):
    def test_numbers_must_match_their_card(self):
        self.assertTrue(all(visual_checks('PulseBoard Ready; Completed — 42, In progress — 7, Blocked — 3.').values()))
        self.assertFalse(all(visual_checks('PulseBoard Ready; Completed — 3, In progress — 42, Blocked — 7.').values()))

    def test_unexpected_or_oversized_view_never_launches(self):
        with tempfile.TemporaryDirectory(dir=ROOT/'output',prefix='corpus-check-') as name:
            directory=Path(name).resolve()
            self.assertTrue(directory.is_relative_to((ROOT/'output').resolve()))
            for raw in [json.dumps({'title':'PulseBoard','script':'untrusted'}), ' '*4097]:
                with self.subTest(bytes=len(raw)):
                    (directory/'generated-view.json').write_text(raw,encoding='utf-8')
                    result=subprocess.run([sys.executable,str(ROOT/'scripts/desktop-generated-fixture.py'),'--directory',str(directory)],capture_output=True,text=True,timeout=5)
                    self.assertNotEqual(result.returncode,0)
                    self.assertIn('Generated view',result.stderr)

if __name__=='__main__':unittest.main()
