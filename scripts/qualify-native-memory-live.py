"""Opt-in two-chat memory UI check with an explicitly selected configured model.

Original history is never copied. The source connection is read-only; its key
exists only in the child environment and disposable in-memory connection.
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

import desktop


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--model', required=True)
    parser.add_argument('--directory', type=Path, required=True)
    parser.add_argument('--reasoning', choices=['providerDefault', 'deepseekThinkingOff', 'openaiLow', 'openaiMedium', 'openaiHigh'], required=True)
    args = parser.parse_args()
    if os.name != 'nt':
        parser.error('The normal Windows bundle is required.')
    desktop.require_normal_build()
    source, base = args.source.resolve(), args.directory.resolve()
    if base == desktop.ROOT / 'output' or not base.is_relative_to(desktop.ROOT / 'output') or base.exists():
        parser.error('Choose a fresh directory strictly beneath output/.')
    with closing(sqlite3.connect(source.as_uri()+'?mode=ro', uri=True)) as db:
        models = json.loads(db.execute('SELECT models FROM model_choices').fetchone()[0])
    if args.model not in models:
        parser.error('Choose an enabled model from the source connection.')
    spec = importlib.util.spec_from_file_location('configured', desktop.ROOT / 'scripts/test-workspace-editor.py')
    support = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(support)
    before = support.digest(source.parent)
    endpoint, key = support.connection(source.parent)
    base.mkdir(parents=True)
    env = dict(os.environ, DOLORES_DATA_DIR=str(base / 'data'),
               DOLORES_GLOBAL_SKILLS_DIR=str(base / 'skills'),
               DOLORES_AUDIT_LIBRARY=str(desktop.executable().parent / 'dolores_flutter_bridge.dll'),
               DOLORES_AUDIT_LIVE_BASE=endpoint, DOLORES_AUDIT_LIVE_KEY=key,
               DOLORES_AUDIT_LIVE_MODEL=args.model, DOLORES_AUDIT_LIVE_REASONING=args.reasoning)
    del key
    try:
        with (base / 'test.log').open('w', encoding='utf-8') as log:
            child = subprocess.Popen([desktop.flutter_path(), '--no-version-check', 'test', '--no-pub',
                                      'test/native_memory_live_ui_test.dart'], cwd=desktop.APP, env=env,
                                     stdout=log, stderr=subprocess.STDOUT)
            try:
                status = child.wait(timeout=120)
            except subprocess.TimeoutExpired:
                subprocess.run(['taskkill', '/PID', str(child.pid), '/T', '/F'], capture_output=True, check=False)
                child.wait(timeout=10)
                status = 1
    finally:
        env.pop('DOLORES_AUDIT_LIVE_KEY', None)
    unchanged = support.digest(source.parent) == before
    receipt = base / 'memory-live-receipt.json'
    if receipt.is_file():
        result = json.loads(receipt.read_text(encoding='utf-8'))
        result['originalProfileUnchanged'] = unchanged
        receipt.write_text(json.dumps(result, indent=2), encoding='utf-8')
    print(json.dumps({'passed': status == 0 and receipt.is_file() and unchanged,
                      'model': args.model, 'originalProfileUnchanged': unchanged,
                      'receipt': str(receipt) if receipt.is_file() else None,
                      'log': str(base / 'test.log')}))
    return 0 if status == 0 and receipt.is_file() and unchanged else 1


if __name__ == '__main__':
    sys.exit(main())
