"""Read-only resource sample of a recorded owned Windows preview."""
import argparse
import json
from pathlib import Path
import time
from desktop import ROOT, process_identity, executable, require_normal_build
from desktop_resource_probe import cpu_seconds, ResourceProbe


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pid-file',type=Path,required=True)
    parser.add_argument('--seconds',type=float,default=15)
    parser.add_argument('--max-cpu-percent',type=float)
    parser.add_argument('--result',type=Path)
    parser.add_argument('--diagnostic',action='store_true')
    args=parser.parse_args()
    entry=require_normal_build(args.diagnostic)
    if not 1<=args.seconds<=60:parser.error('Sample duration must be 1–60 seconds.')
    identity=json.loads(args.pid_file.read_text())
    if process_identity(identity['pid'])!=identity or Path(identity['executable'])!=executable().resolve():
        parser.error('Recorded preview identity changed; no measurement taken.')
    if args.result and not args.result.resolve().is_relative_to(ROOT/'output'):
        parser.error('Keep raw measurements under ignored output/.')
    pid=identity['pid'];before=cpu_seconds(pid);probe=ResourceProbe(pid).start();started=time.monotonic()
    try:time.sleep(args.seconds)
    finally:resources=probe.finish()
    elapsed=time.monotonic()-started;cpu=cpu_seconds(pid)-before
    result={'sampleSeconds':round(elapsed,2),'cpuSeconds':round(cpu,3),
            'entry':entry,'normalRelease':entry=='main',
            'cpuPercentOneCore':round(cpu/elapsed*100,2),'resources':resources,
            'limits':'Sampled peaks may miss short-lived helpers. Record focus and concurrent activity separately. Not startup/low-end qualification; --max-cpu-percent is an optional diagnostic threshold, not a release acceptance target.'}
    if args.result:args.result.write_text(json.dumps(result,indent=2),encoding='utf-8')
    print(json.dumps(result))
    if args.max_cpu_percent is not None and cpu/elapsed*100>args.max_cpu_percent:
        parser.exit(1,'CPU diagnostic threshold exceeded. Inspect the retained sample.\n')


if __name__=='__main__':main()
