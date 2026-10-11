"""Two public no-tool invitations through the bounded Rust collector.

Presence/admission/publication are separate fixtures. Select an enabled model
explicitly; credentials stay in the child environment and no original data changes.
"""
import argparse
from contextlib import closing
import importlib.util
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def connection_support():
    spec = importlib.util.spec_from_file_location('configured', ROOT/'scripts/test-workspace-editor.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--model', required=True, help='Explicit enabled model ID.')
    parser.add_argument('--reasoning', choices=['providerDefault', 'deepseekThinkingOff', 'openaiLow', 'openaiMedium', 'openaiHigh'],
                        default='providerDefault', help='Explicit control for this isolated qualification only.')
    args = parser.parse_args(argv)
    original, output = args.source.resolve(), args.output.resolve()
    if output == ROOT/'output' or not output.is_relative_to(ROOT/'output') or output.exists():
        parser.error('Choose a fresh receipt file strictly beneath output/.')
    with closing(sqlite3.connect(original.as_uri()+'?mode=ro', uri=True)) as db:
        models = json.loads(db.execute('SELECT models FROM model_choices').fetchone()[0])
    if args.model not in models:
        parser.error('Choose an enabled model from the source profile.')
    support = connection_support()
    before = support.digest(original.parent)
    base, key = support.connection(original.parent)
    env = dict(os.environ, DOLORES_COMPANION_LIVE_BASE=base,
               DOLORES_COMPANION_LIVE_MODEL=args.model, DOLORES_COMPANION_LIVE_KEY=key,
               DOLORES_COMPANION_LIVE_REASONING=json.dumps(args.reasoning))
    del key
    try:
        result = subprocess.run([
            'cargo', 'test', '-p', 'dolores-flutter-bridge',
            'companionship::tests::configured_weaker_model_generates_bounded_invitations',
            '--', '--ignored', '--nocapture'], cwd=ROOT, env=env, capture_output=True,
            text=True, encoding='utf-8', errors='replace', timeout=300)
    finally:
        env.pop('DOLORES_COMPANION_LIVE_KEY', None)
    summary = next((line.split('COMPANION_LIVE=', 1)[1] for line in result.stdout.splitlines()
                    if 'COMPANION_LIVE=' in line), None)
    if summary is None:
        raise RuntimeError('Qualification did not complete. Provider details were not logged.')
    data = json.loads(summary)
    cases = data.get('cases', [])
    data['originalProfileUnchanged'] = support.digest(original.parent) == before
    data['passed'] = (result.returncode == 0 and data.get('model') == args.model
                      and data.get('reasoning') == args.reasoning
                      and data.get('requests') == 2 and data.get('tools') == 0
                      and len(cases) == 2 and {c.get('kind') for c in cases} == {'chat', 'memory'}
                      and all(c.get('pass') is True for c in cases)
                      and data['originalProfileUnchanged'])
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(data, indent=2), encoding='utf-8')
    print(json.dumps(data))
    return 0 if data['passed'] else 1


if __name__ == '__main__':
    sys.exit(main())
