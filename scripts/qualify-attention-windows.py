"""Compile/run actual Windows notification channel code in an owned fixture.

Injects deterministic Shell outcomes for callback/privacy/failure checks, then
queries real OS policy and registers/removes one owned icon without a balloon.
This is diagnostic native evidence, not physical-click/display acceptance.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT=Path(__file__).resolve().parents[1]

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory',type=Path,required=True)
    parser.add_argument('--cmake',type=Path,required=True)
    args=parser.parse_args()
    if os.name!='nt':parser.error('Windows x64 is required.')
    out=args.directory.resolve()
    if not out.is_relative_to(ROOT/'output') or out==ROOT/'output' or out.exists():parser.error('Choose a fresh directory strictly under output/.')
    out.mkdir(parents=True)
    ephemeral=ROOT/'apps/dolores_flutter/windows/flutter/ephemeral'
    wrapper=ROOT/'apps/dolores_flutter/build/windows/x64/flutter/Release/flutter_wrapper_app.lib'
    source=ROOT/'scripts/fixtures/attention_notifications.cpp'
    cmake=f'''cmake_minimum_required(VERSION 3.14)
project(attention_fixture LANGUAGES CXX)
add_executable(attention_fixture "{source.as_posix()}")
target_compile_features(attention_fixture PRIVATE cxx_std_17)
target_compile_definitions(attention_fixture PRIVATE UNICODE _UNICODE NOMINMAX)
target_compile_options(attention_fixture PRIVATE /UNDEBUG /utf-8)
target_include_directories(attention_fixture PRIVATE "{(ephemeral/'cpp_client_wrapper/include').as_posix()}" "{ephemeral.as_posix()}")
target_link_libraries(attention_fixture PRIVATE "{wrapper.as_posix()}" "{(ephemeral/'flutter_windows.dll.lib').as_posix()}" shell32 ole32 user32 uuid)
'''
    (out/'CMakeLists.txt').write_text(cmake,encoding='utf-8')
    with (out/'build.log').open('w',encoding='utf-8') as log:
        for command in [[str(args.cmake),'-S',str(out),'-B',str(out/'build'),'-G','Visual Studio 18 2026','-A','x64'],[str(args.cmake),'--build',str(out/'build'),'--config','Release']]:
            result=subprocess.run(command,stdout=log,stderr=subprocess.STDOUT,timeout=120)
            if result.returncode:print(f'Native fixture compile failed; evidence: {out}');return result.returncode
    env={**os.environ,'PATH':str(ephemeral)+os.pathsep+os.environ.get('PATH','')}
    result=subprocess.run([str(out/'build/Release/attention_fixture.exe')],capture_output=True,text=True,env=env,timeout=30)
    (out/'run.log').write_text(result.stdout+result.stderr,encoding='utf-8')
    if result.returncode:return result.returncode
    receipt=json.loads(result.stdout.strip())
    (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(receipt,indent=2))
    return 0 if receipt.get('passed') and receipt.get('actualPolicyRead') and receipt.get('actualRegistration') else 1

if __name__=='__main__':sys.exit(main())
