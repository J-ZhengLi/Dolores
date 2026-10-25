"""Regression checks for shared UI guidance drifting into feature/evidence logs."""
import importlib.util
from pathlib import Path
import tempfile
import unittest

spec=importlib.util.spec_from_file_location('check_docs',Path(__file__).with_name('check-docs.py'))
docs=importlib.util.module_from_spec(spec)
spec.loader.exec_module(docs)


class UiGuideScopeTests(unittest.TestCase):
    def check_text(self,text):
        with tempfile.TemporaryDirectory() as folder:
            path=Path(folder)/'UI.md'
            path.write_text(text,encoding='utf-8')
            return docs.ui_guide_issues(path)

    def test_current_shared_guide(self):
        self.assertEqual(docs.ui_guide_issues(docs.ROOT/'docs/UI.md'),[])

    def test_valid_feature_link_still_requires_owner_routing(self):
        issues=self.check_text('# UI guide\nSee [MCP](design/mcp-connection.md).\n')
        self.assertIn('feature-specific',issues[0]['issue'])
        self.assertEqual(self.check_text('# UI guide\nSee [specifications](README.md).\n'),[])

    def test_dated_evidence_and_unbounded_append_are_rejected(self):
        self.assertTrue(self.check_text('## Verified 2026-10-25\n'))
        self.assertTrue(self.check_text('# UI guide\n'+'A feature detail.\n'*180))


class WorkflowScopeTests(unittest.TestCase):
    def fixture(self, root):
        for name in docs.ENTRY_WORD_LIMITS:
            path=root/name
            path.parent.mkdir(parents=True,exist_ok=True)
            path.write_text('# Entry\n',encoding='utf-8')

    def test_current_entry_points_are_bounded(self):
        self.assertEqual(docs.workflow_issues(docs.ROOT),[])

    def test_diary_append_is_rejected_even_with_valid_links(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder)
            self.fixture(root)
            for heading in ('## UI pass — October 25','## Checked 2026-10-25'):
                (root/'docs/ACCEPTANCE.md').write_text(heading,encoding='utf-8')
                self.assertIn('current limitations',docs.workflow_issues(root)[0]['issue'])

    def test_growth_fails_then_focused_summary_recovers(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder)
            self.fixture(root)
            path=root/'docs/ACCEPTANCE.md'
            path.write_text('detail ' * 701,encoding='utf-8')
            self.assertIn('700 words',docs.workflow_issues(root)[0]['issue'])
            path.write_text('# Current limits\nModel reliability remains bounded.\n',encoding='utf-8')
            self.assertEqual(docs.workflow_issues(root),[])


if __name__=='__main__': unittest.main()
