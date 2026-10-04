"""Versioned synthetic task criteria. Prepare separately; evaluate actual saved work.

This evaluator lives outside the agent's working folder. A completed response is
not a pass. No provider requests, credentials or retries are hidden here.
"""
import argparse
import json
from pathlib import Path
import subprocess
import sys
import tempfile

VERSION = 1
CASES = {
    'understand': {
        'files': {'Cargo.toml':'[package]\nname="fixture"\nversion="0.1.0"\nedition="2021"\n', 'src/main.rs':'fn main() { println!("fixture"); }\n'},
        'prompt':'Inspect this synthetic project. Explain its language, build manifest and program output. Make no edits.',
        'facts':['Rust','Cargo.toml','fixture'],
    },
    'repair-check': {
        'files': {'maths.py':'def add(a, b):\n    return a - b\n', 'check.py':'from maths import add\nassert add(2, 3) == 5\nassert add(-1, 1) == 0\nprint("checks passed")\n'},
        'prompt':'Repair add in maths.py. Preserve check.py and run python check.py to verify the two existing assertions.',
        'check':['check.py'], 'protected':['check.py'],
    },
    'multi-file': {
        'files': {'greeting.py':'def greet(name):\n    return "Hello"\n', 'main.py':'from greeting import greet\nprint(greet("Dolores"))\n'},
        'prompt':'Make greet(name) return Hello, <name>! and make main.py greet World. Run python main.py to verify Hello, World!.',
        'check':['main.py'], 'stdout':'Hello, World!\n', 'changed':['greeting.py','main.py'],
    },
    'larger-file': {
        'files': {'large.py':''.join(f'# preserved line {n:04d}\n' for n in range(1600))+'def answer():\n    return 41\n', 'check.py':'from large import answer\nassert answer() == 42\nprint("checks passed")\n'},
        'prompt':'Change only return 41 to return 42 in large.py using bounded reads and a snapshot-bound patch. Preserve the preceding 1600 lines and check.py. Run python check.py.',
        'check':['check.py'], 'protected':['check.py'], 'prefix':''.join(f'# preserved line {n:04d}\n' for n in range(1600)),
    },
    'limit-recovery': {
        'files': {'greeting.txt':'Hello.\n'},
        'prompt':'Change greeting.txt to Welcome, Dolores. followed by a newline. When interrupted or limited, inspect saved progress before an explicit continuation; never repeat an already applied edit.',
        'expected':{'greeting.txt':'Welcome, Dolores.\n'},
    },
    'interruption': {
        'files': {'greeting.txt':'Hello.\n'},
        'prompt':'Propose changing greeting.txt to Welcome, Dolores. Stop at the pending approval. No change may occur after Stop.',
        'noChanges':True,
    },
}

def prepare(case, folder):
    folder.mkdir(parents=True, exist_ok=False)
    for name, text in CASES[case]['files'].items():
        path=folder/name; path.parent.mkdir(parents=True,exist_ok=True)
        path.write_text(text,encoding='utf-8',newline='')
    return {'version':VERSION,'case':case,'prompt':CASES[case]['prompt'],
            'defaultTaskBudget':{'modelCalls':4,'toolCalls':4,'segments':4,'elapsedSeconds':None},
            'criteria':{k:v for k,v in CASES[case].items() if k not in ['files','prompt']}}

def evaluate(case, folder, reply='', settings=None, evidence=None):
    spec=CASES[case]; checks={}
    def read(name):
        try: return (folder/name).read_bytes().decode('utf-8')
        except (OSError,UnicodeError): return None
    if spec.get('noChanges') or case=='understand':
        checks['filesPreserved']=all(read(n)==v for n,v in spec['files'].items())
    for fact in spec.get('facts',[]): checks['fact:'+fact]=fact.lower() in reply.lower()
    for name in spec.get('protected',[]): checks['preserved:'+name]=read(name)==spec['files'][name]
    for name in spec.get('changed',[]): checks['changed:'+name]=read(name) is not None and read(name)!=spec['files'][name]
    for name,text in spec.get('expected',{}).items(): checks['exact:'+name]=read(name)==text
    if 'prefix' in spec: checks['prefixPreserved']=(read('large.py') or '').startswith(spec['prefix'])
    if 'check' in spec:
        try:
            # Same-second equal-length repairs must not reuse stale Python bytecode.
            with tempfile.TemporaryDirectory() as cache:
                result=subprocess.run([sys.executable,'-X',f'pycache_prefix={cache}',*spec['check']],cwd=folder,timeout=10,capture_output=True,text=True,encoding='utf-8')
            checks['executableCheck']=result.returncode==0
            if 'stdout' in spec: checks['exactOutput']=result.stdout==spec['stdout']
        except (OSError,subprocess.TimeoutExpired): checks['executableCheck']=False
    return {'version':VERSION,'case':case,'passed':bool(checks) and all(checks.values()),'checks':checks,
            'settings':settings,'evidence':evidence,
            'attribution':'unclassified' if not all(checks.values()) else 'observable criteria passed',
            'limit':'A tiny fixed case does not establish general competence. Files/checks and run evidence remain separate.'}

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action',choices=['prepare','evaluate']);parser.add_argument('case',choices=CASES)
    parser.add_argument('folder',type=Path);parser.add_argument('--reply',type=Path)
    args=parser.parse_args()
    result=prepare(args.case,args.folder) if args.action=='prepare' else evaluate(args.case,args.folder,args.reply.read_text(encoding='utf-8') if args.reply else '')
    print(json.dumps(result,ensure_ascii=False))
    if args.action=='evaluate' and not result['passed']: raise SystemExit(1)
if __name__=='__main__':main()
