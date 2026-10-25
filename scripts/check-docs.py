"""Check repository Markdown local file/heading links; external URLs stay external."""
import json
from pathlib import Path
import re
from urllib.parse import unquote, urlsplit

ROOT=Path(__file__).resolve().parents[1]

# Entry points are routers/current summaries, not growing delivery ledgers.
ENTRY_WORD_LIMITS = {
    'AGENTS.md': 350,
    'CONTRIBUTING.md': 900,
    'docs/README.md': 500,
    'docs/ACCEPTANCE.md': 700,
    'docs/ROADMAP.md': 500,
    'docs/IMPLEMENTATION_HISTORY.md': 400,
}

def plain_lines(path):
    fence=None
    for number,line in enumerate(path.read_text(encoding='utf-8-sig').splitlines(),1):
        marker=re.match(r'^\s*(`{3,}|~{3,})',line)
        if marker:
            if fence is None: fence=marker[1][0]
            elif fence==marker[1][0]: fence=None
            continue
        if fence is None: yield number,line

def anchors(path):
    result=set();counts={}
    for _,line in plain_lines(path):
        match=re.match(r'^#{1,6}\s+(.+?)\s*#*$',line)
        if not match: continue
        slug=re.sub(r'[^\w\- ]','',match[1].lower()).replace(' ','-')
        count=counts.get(slug,0);counts[slug]=count+1
        result.add(slug+('-'+str(count) if count else ''))
    return result

def ui_guide_issues(path):
    """Keep the shared guide small; route feature/evidence detail to its owner."""
    issues=[]
    if len(path.read_text(encoding='utf-8-sig').splitlines()) > 180:
        issues.append({'line':1,'issue':'UI guide exceeds 180 lines; disclose feature detail through docs/README.md'})
    for number,line in plain_lines(path):
        if re.match(r'^#{1,6}\s',line) and re.search(r'\b20\d{2}[-/]\d{2}[-/]\d{2}\b',line):
            issues.append({'line':number,'issue':'Dated UI sections belong in design rationale or acceptance'})
        for target in re.findall(r'\]\(([^\s)]+)',line):
            part=urlsplit(target).path
            if part.startswith(('design/','plans/','qualification/','research/')) and part.endswith('.md'):
                issues.append({'line':number,'target':target,'issue':'Route feature-specific references through docs/README.md; UI.md owns shared principles'})
    return issues


def workflow_issues(root):
    issues=[]
    for name,limit in ENTRY_WORD_LIMITS.items():
        path=root/name
        if not path.exists():
            issues.append({'file':name,'line':1,'issue':'Missing documentation entry point'})
            continue
        if len(path.read_text(encoding='utf-8-sig').split()) > limit:
            issues.append({'file':name,'line':1,'issue':f'Entry point exceeds {limit} words; keep current routing/limits here and use its owning record for detail'})
    path=root/'docs/ACCEPTANCE.md'
    if path.exists():
        for number,line in plain_lines(path):
            if re.match(r'^#{1,6}\s',line) and re.search(
                r'\b20\d{2}[-/]\d{2}[-/]\d{2}\b|\b(?:January|February|March|April|May|June|July|August|September|October|November|December)\s+\d{1,2}\b', line, re.I
            ):
                issues.append({'file':'docs/ACCEPTANCE.md','line':number,'issue':'Acceptance is a current limitations index; report dated runs in task results/CI'})
    return issues


def main():
    paths=sorted([ROOT/'README.md',*ROOT.glob('README_*.md'),ROOT/'CONTRIBUTING.md',ROOT/'apps/dolores_flutter/README.md',*(ROOT/'docs').rglob('*.md')])
    failures=[];checked=0;external=0
    for path in paths:
        for number,line in plain_lines(path):
            for match in re.finditer(r'!?\[[^\]]*\]\((<[^>]+>|[^\s)]+)(?:\s+"[^"]*")?\)',line):
                target=match[1].strip('<>');parts=urlsplit(target)
                if parts.scheme or parts.netloc: external+=1;continue
                checked+=1
                dest=(ROOT/parts.path.lstrip('/') if parts.path.startswith('/') else path.parent/unquote(parts.path)).resolve() if parts.path else path
                issue=None
                if not dest.is_relative_to(ROOT) or not dest.exists(): issue='missing/outside repository target'
                elif parts.fragment and dest.suffix=='.md' and unquote(parts.fragment) not in anchors(dest): issue='missing heading'
                if issue: failures.append({'file':str(path.relative_to(ROOT)),'line':number,'target':target,'issue':issue})
    failures.extend({'file':'docs/UI.md',**issue} for issue in ui_guide_issues(ROOT/'docs/UI.md'))
    failures.extend(workflow_issues(ROOT))
    print(json.dumps({'ok':not failures,'documents':len(paths),'localLinks':checked,'externalLinksNotChecked':external,'failures':failures},indent=2))
    return bool(failures)

if __name__=='__main__': raise SystemExit(main())
