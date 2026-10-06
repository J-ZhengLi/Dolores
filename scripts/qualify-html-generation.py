"""Bounded configured-model HTML generation in a fresh disposable profile.

Credentials come only from the environment. Approvals cover files in the synthetic
folder, never model-proposed commands. Results report artifact/transport evidence,
not gameplay quality or general model reliability.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import re
from desktop_test_support import NativeHost, ROOT


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory',type=Path,required=True)
    parser.add_argument('--model',required=True)
    parser.add_argument('--max-output-tokens',type=int,default=16384)
    parser.add_argument('--timeout-seconds',type=int,default=180)
    parser.add_argument('--reasoning',choices=['providerDefault','deepseekThinkingOff','glmLow'],default='providerDefault')
    args=parser.parse_args()
    directory=args.directory.resolve()
    if not args.directory.is_absolute() or not directory.is_relative_to(ROOT/'output') or directory.exists():
        parser.error('Use a fresh absolute ignored output directory.')
    workspace=directory/'workspace'; workspace.mkdir(parents=True)
    host=NativeHost(directory)
    result={'model':args.model,'passed':False,'live':True,'scope':'standalone HTML generation and script syntax only','maxOutputTokens':args.max_output_tokens,'reasoning':args.reasoning,'requestTimeoutSeconds':args.timeout_seconds,'taskDeadlineSeconds':240}
    reviews=[]
    try:
        host.call('configure',preferences={'baseUrl':os.environ['DOLORES_TEST_BASE_URL'],'model':args.model},apiKey=os.environ['DOLORES_TEST_API_KEY'],rememberConnection=False,enabledModels=[args.model])
        host.call('setRequestSettings',settings={'maxOutputTokens':args.max_output_tokens,'timeoutSeconds':args.timeout_seconds,'reasoning':args.reasoning})
        policy=host.call('memories')['automaticPolicy']; host.call('setAutomaticMemory',enabled=False,revision=policy['revision'])
        session=host.call('createSession',kind='project',path=str(workspace))['session']['id']
        host.call('saveScopedSettings',session=session,scope='thread',revision=0,patch={'task':{'modelCalls':4,'toolCalls':6,'segments':1,'elapsedSeconds':240}})
        def approval(event):
            if event['type']!='toolApproval': return
            request=event['request']; target=request.get('target','')
            local_html=(isinstance(target,str) and Path(target).name==target and target.lower().endswith('.html'))
            allow=(request['name'] in ['create_text_file','edit_text_file','read_text_file'] and local_html) or (request['name']=='list_folder' and target=='.')
            reviews.append({'name':request['name'],'allowed':allow})
            host.call('approveTool',id=1,callId=request['callId'],allow=allow)
        host.call('start',id=1,session=session,input='Build me a Super Mario clone and output it as an HTML file',tools=True)
        done,events=host.finish(1,callback=approval,seconds=255)
        error=done.get('error') or ''
        result['errorCategory']=next((x for x in ['frame limit','oversized stream event','wire limit','byte limit','timed out','output limit'] if x in str(error)),'other' if error else None)
        result['replyCompleted']=not bool(error)
        messages=host.call('messagesPage',session=session)['items']
        result['pauseReason']=((messages[-1].get('metadata') or {}).get('paused') or {}).get('reason') if messages else None
        files=list(workspace.glob('*.html'))
        result['htmlFiles']=len(files); result['bytes']=[file.stat().st_size for file in files]
        scripts=[]
        if len(files)==1:
            text=files[0].read_text(encoding='utf-8')
            scripts=re.findall(r'<script\b[^>]*>(.*?)</script\s*>',text,re.I|re.S)
            result['standalone']=not bool(re.search(r'<(?:script|link)\b[^>]*(?:src|href)\s*=\s*[\"\'](?:https?:|//)',text,re.I))
            result['hasGameSurface']=bool(re.search(r'<canvas\b|requestAnimationFrame|keydown',text))
            syntax=[]
            for index,script in enumerate(scripts):
                path=directory/f'inline-{index}.js'; path.write_text(script,encoding='utf-8')
                check=subprocess.run(['node','--check',str(path)],capture_output=True,timeout=10)
                syntax.append(check.returncode==0)
            result['scriptSyntaxPass']=bool(syntax) and all(syntax)
        result['reviews']=reviews
        result['passed']=len(files)==1 and result.get('standalone',False) and result.get('hasGameSurface',False) and result.get('scriptSyntaxPass',False) and result['errorCategory'] is None and result['pauseReason'] is None
    except Exception as error:
        result['errorType']=type(error).__name__
    finally:
        host.close()
    (directory/'public-result.json').write_text(json.dumps(result,indent=2),encoding='utf-8')
    print(json.dumps(result))
    if not result['passed']: raise SystemExit(1)


if __name__=='__main__': main()
