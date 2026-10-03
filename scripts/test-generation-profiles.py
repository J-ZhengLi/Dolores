"""Isolated generation-profile diagnostic through native FFI, SQLite and HTTP.

Build the Windows Flutter bundle first. Run save and restore in separate
processes using the same fresh absolute --directory under output/. No live API.
"""
import argparse
import ctypes
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import threading
import time

parser = argparse.ArgumentParser()
parser.add_argument('stage', choices=['save', 'restore'])
parser.add_argument('--directory', type=Path, required=True)
args = parser.parse_args()
root = Path(__file__).resolve().parents[1]
directory = args.directory
if not directory.is_absolute() or not directory.resolve().is_relative_to(root / 'output'):
    raise SystemExit('Use an absolute isolated directory under output/.')
if args.stage == 'save':
    directory.mkdir(parents=True, exist_ok=False)
os.environ['DOLORES_DATA_DIR'] = str(directory / 'data')
os.environ['DOLORES_GLOBAL_SKILLS_DIR'] = str(directory / 'global-skills')
bundle = root / 'apps/dolores_flutter/build/windows/x64/runner/Release'
loader = os.add_dll_directory(str(bundle))
native = ctypes.CDLL(str(bundle / 'dolores_flutter_bridge.dll'))
native.dolores_call.argtypes = [ctypes.c_void_p, ctypes.c_size_t]
native.dolores_call.restype = ctypes.c_void_p
native.dolores_free.argtypes = [ctypes.c_void_p]

def envelope(command, **fields):
    payload = json.dumps({'command': command, **fields}).encode()
    buffer = ctypes.create_string_buffer(payload)
    pointer = native.dolores_call(buffer, len(payload))
    try:
        return json.loads(ctypes.string_at(pointer))
    finally:
        native.dolores_free(pointer)

def call(command, **fields):
    result = envelope(command, **fields)
    assert result['ok'], result.get('error')
    return result.get('result')

requests = []
class Fixture(BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass
    def do_POST(self):
        payload = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        requests.append(payload)
        user = next(m['content'] for m in reversed(payload['messages']) if m['role'] == 'user')
        if payload['model'] == 'fast':
            assert payload['reasoning_effort'] == 'low' and payload['max_completion_tokens'] == 4096
            assert 'max_tokens' not in payload and 'thinking' not in payload
        elif user not in ['basic']:
            assert payload['thinking'] == {'type':'disabled'} and payload['max_tokens'] == 8192
        else:
            assert 'thinking' not in payload and 'reasoning_effort' not in payload and payload['max_tokens'] == 2048
        if user == 'unsupported':
            body = json.dumps({'error':{'param':'thinking','message':'unsupported thinking and stream_options PRIVATE_FIXTURE'}}).encode()
            self.send_response(400)
            self.send_header('Content-Length', str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if user == 'malformed':
            count = sum(m['role'] == 'tool' for m in payload['messages'])
            arguments = json.dumps({'path':'kept.js','content':'module.exports = 42;\n'}) if count == 0 else '{invalid'
            delta = {'tool_calls':[{'index':0,'id':'one' if count == 0 else 'two','type':'function','function':{'name':'create_text_file','arguments':arguments}}]}
            frames = [{'choices':[{'delta':delta,'finish_reason':'tool_calls'}]}]
        else:
            frames = [{'choices':[{'delta':{'content':'Partial progress' if user == 'paused' else 'Complete answer'},'finish_reason':'length' if user == 'paused' else 'stop'}]}]
        frames.append({'choices':[],'usage':{'prompt_tokens':100,'completion_tokens':8192 if user == 'paused' else 20,'completion_tokens_details':{'reasoning_tokens':8192 if user == 'paused' else 0}}})
        self.send_response(200)
        self.send_header('Content-Type','text/event-stream')
        self.end_headers()
        self.wfile.write((''.join('data: '+json.dumps(frame)+'\n\n' for frame in frames)+'data: [DONE]\n\n').encode())

approvals = []
def run(number, session, text):
    call('start', id=number, session=session, input=text)
    until = time.monotonic() + 25
    while time.monotonic() < until:
        for event in call('poll', id=number):
            if event['type'] == 'toolApproval':
                request = event['request']
                assert request['name'] == 'create_text_file' and request['target'] == 'kept.js'
                approvals.append(request)
                call('approveTool', id=number, callId=request['callId'], allow=True)
            if event['type'] == 'done':
                return event
        time.sleep(.01)
    raise AssertionError('Generation profile diagnostic timed out')

server = ThreadingHTTPServer(('127.0.0.1',0),Fixture)
threading.Thread(target=server.serve_forever,daemon=True).start()
try:
    state_file = directory / 'state.json'
    initial = call('bootstrap')
    if args.stage == 'save':
        policy = call('memories')['automaticPolicy']
        call('setAutomaticMemory',enabled=False,revision=policy['revision'])
        preferences = {'baseUrl':f'http://127.0.0.1:{server.server_port}/v1','model':'fast'}
        call('configure', preferences=preferences, enabledModels=['fast','reasoning'], apiKey='',rememberConnection=False)
        fast = {'maxOutputTokens':4096,'timeoutSeconds':60,'reasoning':'openaiLow'}
        hard = {'maxOutputTokens':8192,'timeoutSeconds':120,'reasoning':'deepseekThinkingOff'}
        call('setModelRequestSettings',preferences=preferences,settings=fast)
        hard_preferences = {**preferences,'model':'reasoning'}
        call('setModelRequestSettings',preferences=hard_preferences,settings=hard)
        side = call('createSession',kind='side')['session']['id']
        assert not run(1,side,'basic').get('error')
        call('selectModel',model='reasoning')
        assert call('bootstrap')['requestSettings'] == hard
        assert not run(2,side,'paused').get('error')
        before = call('messagesPage',session=side)['items']
        metadata = before[-1]['metadata']
        assert metadata['paused']['reason'] == 'outputLimit' and metadata['usage']['reasoningTokens'] == 8192
        assert metadata['requestSettings'] == hard
        result = run(3,side,'unsupported')
        assert result['recovery']['kind'] == 'generationSettings' and not result['recovery']['retryable']
        assert 'PRIVATE_FIXTURE' not in json.dumps(result)
        assert call('messagesPage',session=side)['items'] == before
        assert call('bootstrap')['requestSettings'] == hard
        project = directory / 'project'
        project.mkdir()
        session = call('createSession',kind='project',path=str(project))['session']['id']
        result = run(4,session,'malformed')
        assert result['recovery']['kind'] == 'malformedTools'
        assert (project/'kept.js').read_bytes() == b'module.exports = 42;\n'
        assert call('messagesPage',session=session)['items'] == [] and len(approvals) == 1
        assert len(call('changesPage',session=session)['items']) == 1
        call('setModelRequestSettings',preferences=hard_preferences,settings=None)
        assert not run(5,side,'basic').get('error')
        call('selectModel',model='fast')
        assert not run(6,side,'basic').get('error')
        assert len(requests) == 7
        state_file.write_text(json.dumps({'profiles':call('bootstrap')['modelRequestSettings'],'settings':fast,'side':side,'project':session,'messages':call('messagesPage',session=side)['items']}),encoding='utf-8')
    else:
        saved = json.loads(state_file.read_text(encoding='utf-8'))
        assert initial['modelRequestSettings'] == saved['profiles'] and initial['requestSettings'] == saved['settings']
        assert call('messagesPage',session=saved['side'])['items'] == saved['messages']
        assert len(call('changesPage',session=saved['project'])['items']) == 1
        assert (directory/'project/kept.js').read_bytes() == b'module.exports = 42;\n'
        assert not requests
finally:
    call('shutdown')
    server.shutdown()
print(json.dumps({'ok':True,'stage':args.stage,'fixtureRequests':len(requests),'approvals':len(approvals),'liveRequests':0}))
