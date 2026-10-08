"""Check repository Markdown local file/heading links; external URLs stay external."""
import json
from pathlib import Path
import re
from urllib.parse import unquote, urlsplit

ROOT=Path(__file__).resolve().parents[1]

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
    print(json.dumps({'ok':not failures,'documents':len(paths),'localLinks':checked,'externalLinksNotChecked':external,'failures':failures},indent=2))
    return bool(failures)

if __name__=='__main__': raise SystemExit(main())
