"""Isolated native FFI/HTTP proof of reviewed skill drafts and frozen promotion."""
import argparse
import ctypes
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import sqlite3
import threading
import time

parser = argparse.ArgumentParser()
parser.add_argument('stage', choices=['save', 'restore'])
parser.add_argument('--directory', type=Path, required=True)
args = parser.parse_args()
root = Path(__file__).resolve().parents[1]
fixture = args.directory
if not fixture.is_absolute() or not fixture.resolve().is_relative_to(root / 'output'):
    raise SystemExit('Use an absolute isolated directory under output/.')
if args.stage == 'save': fixture.mkdir(parents=True, exist_ok=False)
os.environ['DOLORES_DATA_DIR'] = str(fixture / 'data')
os.environ['DOLORES_GLOBAL_SKILLS_DIR'] = str(fixture / 'global-skills')
bundle = root / 'apps/dolores_flutter/build/windows/x64/runner/Release'
loader = os.add_dll_directory(str(bundle)) if os.name == 'nt' else None
native = ctypes.CDLL(str(bundle / ('dolores_flutter_bridge.dll' if os.name == 'nt' else 'lib/libdolores_flutter_bridge.so')))
native.dolores_call.argtypes = [ctypes.c_void_p, ctypes.c_size_t]
native.dolores_call.restype = ctypes.c_void_p
native.dolores_free.argtypes = [ctypes.c_void_p]

def envelope(command, **fields):
    payload = json.dumps({'command': command, **fields}).encode()
    buffer = ctypes.create_string_buffer(payload)
    pointer = native.dolores_call(buffer, len(payload))
    try: return json.loads(ctypes.string_at(pointer))
    finally: native.dolores_free(pointer)

def call(command, **fields):
    result = envelope(command, **fields)
    assert result['ok'], result.get('error')
    return result.get('result')

requests = []
mode = 'normal'
class Fixture(BaseHTTPRequestHandler):
    def log_message(self, *_): pass
    def do_POST(self):
        request = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        requests.append(request)
        if mode == 'slow': time.sleep(2)
        if mode == 'long': time.sleep(31)
        assert request['model'] == 'fixture'
        assert 'tools' not in request and 'tool_choice' not in request
        assert len(request['messages']) == 2
        if request['messages'][0]['content'].startswith('You draft reusable instructions for Dolores'):
            assert request['max_tokens'] in (2048, 4096)
            example = json.loads(request['messages'][1]['content'])['examples'][0]
            text = json.dumps({'name': 'review', 'description': 'Use when reviewing synthetic work.', 'instructions': 'Include SKILL_PASS in the review.', 'evidence': [{'messageId': example['messageId'], 'quote': 'Use focused tests.'}]})
        elif request['messages'][-1]['content'].startswith('skill-test'):
            assert request['max_tokens'] == 512
            text = 'SKILL_PASS' if mode == 'tie' or 'Include SKILL_PASS' in request['messages'][0]['content'] else 'Ordinary response'
            if mode == 'regress': text = 'FAIL'
        else:
            text = 'Use focused tests.'
        truncated = mode == 'length' and request['max_tokens'] < 4096
        if truncated: text = text[:50]
        events = [{'choices': [{'delta': {'content': text}, 'finish_reason': None}]}, {'choices': [{'delta': {}, 'finish_reason': 'length' if truncated else 'stop'}]}, {'choices': [], 'usage': {'prompt_tokens': 100, 'completion_tokens': 20, 'total_tokens': 120}}]
        try:
            self.send_response(200); self.send_header('Content-Type', 'text/event-stream'); self.end_headers()
            self.wfile.write((''.join('data: '+json.dumps(v)+'\n\n' for v in events)+'data: [DONE]\n\n').encode())
        except (BrokenPipeError, ConnectionResetError): pass

def finish(number, error=False, timeout=15):
    deadline = time.monotonic()+timeout
    while time.monotonic() < deadline:
        for event in call('poll', id=number):
            if event['type'] == 'done':
                assert ('error' in event) == error, event
                return event
        time.sleep(.01)
    raise AssertionError('Native request timed out')

state_file = fixture / 'state.json'
if args.stage == 'save':
    server = ThreadingHTTPServer(('127.0.0.1', 0), Fixture)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    try:
        call('bootstrap')
        policy = call('memories')['automaticPolicy']
        call('setAutomaticMemory', enabled=False, revision=policy['revision'])
        call('configure', preferences={'baseUrl': f'http://127.0.0.1:{server.server_port}/v1', 'model': 'fixture'}, apiKey='', rememberConnection=True)
        session = call('createSession', kind='side')['session']['id']
        call('start', id=1, session=session, input='Review synthetic work'); finish(1)
        original = call('messagesPage', session=session)
        sources = call('reviewSkillExamples', session=session, scope='global')
        assert len(sources['examples']) == 1 and len(requests) == 1
        assert not envelope('reviewSkillExamples', session=session, scope='project')['ok']
        assert not envelope('promoteSkillDraft', session=session, token=sources['token'])['ok']
        assert not envelope('generateSkillDraft', id=2, session=session, token=sources['token'], messageIds=[999])['ok']
        assert sources['settings'] == {'maxOutputTokens': 2048, 'timeoutSeconds': 180}
        mode = 'length'
        call('generateSkillDraft', id=2, session=session, token=sources['token'], messageIds=[sources['examples'][0]['messageId']])
        exhausted = finish(2, error=True)
        assert '2048-token output limit' in exhausted['error'] and 'Increase Draft output tokens' in exhausted['error']
        assert len(requests) == 2 and not call('projectSkills', session=session, scope='global')['items']
        assert not envelope('generateSkillDraft', id=2, session=session, token=sources['token'], messageIds=[sources['examples'][0]['messageId']], settings={'maxOutputTokens': 0, 'timeoutSeconds': 90})['ok']
        assert len(requests) == 2
        mode = 'long'
        call('generateSkillDraft', id=2, session=session, token=sources['token'], messageIds=[sources['examples'][0]['messageId']], settings={'maxOutputTokens': 4096, 'timeoutSeconds': 90})
        assert not envelope('reviewSkillExamples', session=session, scope='global')['ok']
        draft = finish(2, timeout=45)['skillDraft']
        assert requests[-1]['max_tokens'] == 4096 and draft['settings'] == {'maxOutputTokens': 4096, 'timeoutSeconds': 90}
        mode = 'normal'
        assert call('bootstrap')['requestSettings'] == {'maxOutputTokens': 2048, 'timeoutSeconds': 180}
        assert not call('projectSkills', session=session, scope='global')['items']
        trials = [{'prompt': 'skill-test-one', 'required': ['SKILL_PASS'], 'forbidden': ['FAIL']}, {'prompt': 'skill-test-two', 'required': ['SKILL_PASS'], 'forbidden': []}]
        bad = {**draft['draft'], 'evidence': [{'messageId': 999, 'quote': 'Invented'}]}
        assert not envelope('evaluateSkillDraft', id=3, session=session, token=draft['token'], draft=bad, trials=trials)['ok']
        mode = 'tie'
        call('evaluateSkillDraft', id=3, session=session, token=draft['token'], draft=draft['draft'], trials=trials)
        tied = finish(3)['skillEvaluation']
        assert not tied['promotable'] and tied['baselinePassed'] == tied['candidatePassed'] == 2
        assert not envelope('promoteSkillDraft', session=session, token=tied['token'])['ok']
        mode = 'regress'
        call('evaluateSkillDraft', id=4, session=session, token=tied['token'], draft=draft['draft'], trials=trials)
        failed = finish(4)['skillEvaluation']
        assert not failed['promotable'] and failed['candidatePassed'] == 0
        mode = 'normal'
        call('evaluateSkillDraft', id=5, session=session, token=failed['token'], draft=draft['draft'], trials=trials)
        tested = finish(5)['skillEvaluation']
        assert tested['promotable'] and tested['candidatePassed'] == 2 and tested['baselinePassed'] == 0
        evaluation = tested['evaluation']
        for result, base, candidate in zip(evaluation['results'], requests[-4::2], requests[-3::2]):
            assert result['baselineMessages'] == base['messages'] and result['candidateMessages'] == candidate['messages']
            assert result['baselineUsage']['inputTokens'] == result['candidateUsage']['inputTokens'] == 100
        active = call('promoteSkillDraft', session=session, token=tested['token'])
        assert active['versions'][-1]['evaluation'] == evaluation
        assert not envelope('promoteSkillDraft', session=session, token=tested['token'])['ok']
        assert not (fixture / 'global-skills/review/SKILL.md').exists()
        saved = call('reviewSkill', session=session, scope='global', name='review', version=1)
        assert saved['generated'] and saved['problem'] is None and saved['evaluation'] == evaluation
        call('cancelSkillReview', token=saved['token'])
        preview = call('context', session=session, input='next')
        assert 'Include SKILL_PASS' in preview['messages'][0]['content']
        assert 'Ordinary response' not in preview['messages'][0]['content']
        assert call('messagesPage', session=session) == original
        call('disableSkill', session=session, scope='global', name='review', revision=1)
        sources = call('reviewSkillExamples', session=session, scope='global')
        call('generateSkillDraft', id=6, session=session, token=sources['token'], messageIds=[sources['examples'][0]['messageId']])
        draft = finish(6)['skillDraft']
        mode = 'slow'; count = len(requests)
        call('evaluateSkillDraft', id=7, session=session, token=draft['token'], draft=draft['draft'], trials=trials)
        deadline = time.monotonic()+5
        while len(requests) == count and time.monotonic() < deadline: time.sleep(.01)
        call('cancel', id=7); finish(7, error=True)
        assert not envelope('promoteSkillDraft', session=session, token=draft['token'])['ok']
        assert len(requests) == count+1
        mode = 'normal'
        call('evaluateSkillDraft', id=8, session=session, token=draft['token'], draft=draft['draft'], trials=trials)
        tested = finish(8)['skillEvaluation']
        # A concurrent local skill write invalidates the frozen comparison.
        path = fixture / 'global-skills/other'; path.mkdir(parents=True)
        (path / 'SKILL.md').write_text('---\nname: other\ndescription: Other workflow\n---\nOther guidance', encoding='utf-8')
        review = call('reviewSkill', session=session, scope='global', name='other')
        call('activateSkill', session=session, token=review['token'])
        assert not envelope('promoteSkillDraft', session=session, token=tested['token'])['ok']
        call('forgetSkill', session=session, scope='global', name='other', revision=1)
        call('discardSkillDraft', token=tested['token'])
        state_file.write_text(json.dumps({'session': session, 'evaluation': evaluation}), encoding='utf-8')
        with sqlite3.connect(fixture / 'data/dolores.db') as db: assert db.execute('PRAGMA user_version').fetchone()[0] == 15
        print(json.dumps({'ok': True, 'stage': 'save', 'requests': len(requests), 'checks': 'output-limit refusal, explicit larger retry, 31-second draft, unchanged settings, tool-free wire, frozen tests, tie/regression gate, one-use promotion, edits, Stop, unchanged transcript/schema'}))
    finally:
        call('shutdown'); server.shutdown(); server.server_close()
else:
    state = json.loads(state_file.read_text(encoding='utf-8'))
    call('bootstrap')
    review = call('reviewSkill', session=state['session'], scope='global', name='review', version=1)
    assert review['evaluation'] == state['evaluation'] and not review['enabled']
    saved = call('activateSkill', session=state['session'], token=review['token'])
    assert saved['versions'][-1]['rollbackFrom'] == 1 and saved['versions'][-1]['evaluation'] == state['evaluation']
    assert not (fixture / 'global-skills/review/SKILL.md').exists()
    print(json.dumps({'ok': True, 'stage': 'restore', 'checks': 'separate-process receipt, disabled state, reviewed rollback'}))
    call('shutdown')
