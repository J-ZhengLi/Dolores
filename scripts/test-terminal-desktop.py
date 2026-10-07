"""Measure and inspect the explicit disposable Flutter terminal diagnostic."""
import json
import os
from pathlib import Path
import statistics
import subprocess
import sys
import time
import uuid

from desktop_resource_probe import children, cpu_seconds, memory

ROOT = Path(__file__).resolve().parents[1]


def main():
    directory = ROOT/'output/terminal-desktop-qualification'/uuid.uuid4().hex
    directory.mkdir(parents=True)
    record = directory/'process.json'
    env = dict(os.environ, DOLORES_TERMINAL_SMOKE_DIR=str(directory))
    command = [sys.executable, str(ROOT/'scripts/desktop.py')]
    subprocess.run(command+['launch', '--allow-diagnostic', '--data-directory',str(directory/'data'), '--pid-file',str(record)],env=env,check=True)
    pid = json.loads(record.read_text())['pid']
    samples = {}; report = {}
    try:
        deadline = time.monotonic()+120
        while time.monotonic()<deadline:
            file = directory/'stage.json'
            if file.exists():
                report = json.loads(file.read_text())
                stage = report['stage']
                if stage in ['complete','failed']: break
                samples.setdefault(stage,[]).append({'time':time.monotonic(),'cpu':cpu_seconds(pid),**memory(pid)})
            time.sleep(.2)
        else: raise AssertionError('Diagnostic deadline exceeded')
        for stage,values in samples.items():
            # Settled tail samples omit page transition/render/startup activity.
            values=[v for v in values if v['time']>=values[-1]['time']-4]
            report.setdefault('measurements',{})[stage]={
                'samples':len(values),
                'privateMiB':round(statistics.median(v['privateBytes'] for v in values)/1048576,2),
                'workingMiB':round(statistics.median(v['workingBytes'] for v in values)/1048576,2),
                'cpuOneCorePercent':round((values[-1]['cpu']-values[0]['cpu'])/(values[-1]['time']-values[0]['time'])*100,3) if len(values)>1 else None,
            }
        report['ownedProcessesAfterStop']=len(children(pid,names=None))
        if report.get('passed'):
            baseline=report['measurements']['baseline']; full=report['measurements']['two-full-scrollback-idle']
            report['addedPrivateMiBPerTerminal']=round((full['privateMiB']-baseline['privateMiB'])/2,2)
            report['addedIdleCpuOneCorePercent']=round(full['cpuOneCorePercent']-baseline['cpuOneCorePercent'],3)
            report['resourceGatePassed']=report['addedPrivateMiBPerTerminal']<64 and report['addedIdleCpuOneCorePercent']<1
        (directory/'report.json').write_text(json.dumps(report,indent=2,ensure_ascii=False),encoding='utf-8')
        print(json.dumps({'directory':str(directory),**report},ensure_ascii=False))
        assert report.get('passed'),report.get('error')
        assert report['resourceGatePassed'],'Terminal UI resource ceiling exceeded'
        assert report['ownedProcessesAfterStop']==0,'Owned PTY descendants remain'
    finally:
        subprocess.run(command+['stop-owned','--pid-file',str(record)],check=True)


if __name__ == '__main__': main()
