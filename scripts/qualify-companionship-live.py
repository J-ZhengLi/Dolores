"""Two public no-tool invitations through the actual bounded Rust collector.

Presence/admission/publication are separate fixtures, not claimed by this probe.
Credentials exist only in the child environment; no original setting is changed.
"""
import argparse, importlib.util, json, os, sqlite3, subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser();p.add_argument('--source',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
original=a.source.resolve();output=a.output.resolve();assert output.is_relative_to(ROOT/'output')
spec=importlib.util.spec_from_file_location('configured',ROOT/'scripts/test-workspace-editor.py');module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
base,key=module.connection(original.parent)
with sqlite3.connect(original.as_uri()+'?mode=ro',uri=True) as db:
    models=json.loads(db.execute('SELECT models FROM model_choices').fetchone()[0])
model=next(m for m in models if 'qwen3.5-2b' in m.lower())
env=dict(os.environ,DOLORES_COMPANION_LIVE_BASE=base,DOLORES_COMPANION_LIVE_MODEL=model,DOLORES_COMPANION_LIVE_KEY=key);del key
r=subprocess.run(['cargo','test','-p','dolores-flutter-bridge','companionship::tests::configured_weaker_model_generates_bounded_invitations','--','--ignored','--nocapture'],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace');env.pop('DOLORES_COMPANION_LIVE_KEY',None)
summary=next((line.split('COMPANION_LIVE=',1)[1] for line in r.stdout.splitlines() if 'COMPANION_LIVE=' in line),None)
if summary is None:raise RuntimeError('Qualification did not complete; inspect native tests. Provider details were not logged.')
data=json.loads(summary);output.parent.mkdir(parents=True,exist_ok=True);output.write_text(json.dumps(data,indent=2),encoding='utf-8');print(json.dumps(data));r.check_returncode()
