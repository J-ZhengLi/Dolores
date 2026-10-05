"""Bounded live learning checks in a disposable profile; no keys or transcripts in receipts."""
import argparse
import json
import os
from pathlib import Path
from desktop_test_support import NativeHost, ROOT

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--directory', required=True)
    parser.add_argument('--model', required=True)
    parser.add_argument('--case', choices=['memory', 'skill'], required=True)
    args = parser.parse_args()
    directory = Path(args.directory).resolve()
    if directory.exists() or not directory.is_relative_to(ROOT / 'output'):
        raise ValueError('Fresh disposable output directory required')
    host = NativeHost(directory)
    result = {'case': args.case, 'model': args.model, 'live': True, 'passed': False}
    try:
        host.call('configure', preferences={'baseUrl': os.environ['DOLORES_TEST_BASE_URL'], 'model': args.model},
                  apiKey=os.environ['DOLORES_TEST_API_KEY'], rememberConnection=False, enabledModels=[args.model])
        host.call('setRequestSettings', settings={'maxOutputTokens': 1024, 'timeoutSeconds': 40})
        if args.case == 'skill':
            policy = host.call('memories')['automaticPolicy']
            host.call('setAutomaticMemory', enabled=False, revision=policy['revision'])
        session = host.call('createSession', kind='side')['session']['id']
        prompt = ('I prefer concise replies with one short example. Acknowledge in one sentence.'
                  if args.case == 'memory' else 'Give a reusable three-step procedure for reviewing a small code change. Keep it under 60 words.')
        host.call('start', id=1, session=session, input=prompt, tools=False)
        done, _ = host.finish(1, seconds=55)
        result['replyCompleted'] = not bool(done.get('error'))
        if not result['replyCompleted']:
            result['failure'] = 'reply failed within explicit allowance'
        elif args.case == 'memory':
            update = done.get('memoryUpdate') or {}
            result['learningStatus'] = update.get('status')
            result['saved'] = update.get('saved', 0)
            result['passed'] = result['saved'] > 0 and bool(host.call('memories', session=session)['items'])
        else:
            sources = host.call('reviewSkillExamples', session=session, scope='global')
            host.call('generateSkillDraft', id=2, session=session, token=sources['token'],
                      messageIds=[sources['examples'][0]['messageId']],
                      settings={'maxOutputTokens': 4096, 'timeoutSeconds': 90})
            drafted, _ = host.finish(2, seconds=100)
            result['draftParsed'] = bool(drafted.get('skillDraft'))
            result['notActivated'] = not host.call('projectSkills', session=session, scope='global')['items']
            result['passed'] = result['draftParsed'] and result['notActivated']
            if drafted.get('error'):
                result['failure'] = 'draft failed within explicit allowance'
    except Exception:
        result['failure'] = 'bounded live check failed; no automatic retry'
    finally:
        host.close()
    directory.mkdir(parents=True, exist_ok=True)
    (directory / 'result.json').write_text(json.dumps(result), encoding='utf-8')
    print(json.dumps(result))
    if not result['passed']:
        raise SystemExit(1)

if __name__ == '__main__':
    main()
