"""Isolated real-FFI/SQLite/HTTP larger-file coding diagnostic.

Build the Flutter bundle first, then run save and restore in separate processes
with the same fresh absolute --directory under output/. No live API is used. An installed Node executable validates synthetic files.
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
loader = os.add_dll_directory(str(bundle)) if os.name == 'nt' else None
native = ctypes.CDLL(str(bundle / ('dolores_flutter_bridge.dll' if os.name == 'nt' else 'lib/libdolores_flutter_bridge.so')))
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
levels = ',\n'.join((f'  {{x:{i * 24},y:80,width:20,height:20,label:"checkpoint {i} 世界"}}' for i in range(80)))
game = "(function () {\n  'use strict';\n  function intersects(a, b) {\n    return a.x < b.x + b.width && a.x + a.width > b.x &&\n      a.y < b.y + b.height && a.y + a.height > b.y;\n  }\n  function step(player, input, dt) {\n    const x = player.x + input * 100 * dt;\n    const velocity = player.velocity + 300 * dt;\n    const y = Math.min(80, player.y + velocity * dt);\n    return {...player, x, y, velocity: y === 80 ? 0 : velocity};\n  }\n  const checkpoints = [\n" + levels + "\n  ];\n  const api = {intersects, step, checkpoints};\n  if (typeof module !== 'undefined') module.exports = api;\n  else window.DemoGame = api;\n}());\n"
html = '<!doctype html><html lang="en"><meta charset="utf-8"><title>Synthetic platform demo</title>\n<canvas id="game" width="640" height="160"></canvas><p>Use arrow keys to move.</p>\n<script src="game.cjs"></script><script>\nconst canvas=document.getElementById(\'game\'), ctx=canvas.getContext(\'2d\');\nlet player={x:20,y:10,width:20,height:20,velocity:0}, input=0;\nwindow.onkeydown=e=>{input=e.key===\'ArrowRight\'?1:e.key===\'ArrowLeft\'?-1:0;};\nwindow.onkeyup=()=>{input=0;};\nfunction frame(){player=DemoGame.step(player,input,1/60);ctx.clearRect(0,0,640,160);\nctx.fillStyle=\'#a0b2ff\';ctx.fillRect(player.x,player.y,20,20);ctx.fillStyle=\'#555\';\nctx.fillRect(0,100,640,5);requestAnimationFrame(frame);}frame();</script></html>\n'
checks = "const assert=require('node:assert/strict');\nconst game=require('./game.cjs');\nconst p={x:0,y:0,width:20,height:20,velocity:0};\nassert.equal(game.checkpoints.length,80);\nassert.equal(game.intersects(p,{...p,x:10}),true);\nassert.equal(game.intersects(p,{...p,x:20}),false);\nassert.equal(game.step(p,1,0.1).x,10);\nassert.equal(game.step({...p,y:79},0,1).y,80);\nassert.equal(game.step({...p,y:79},0,1).velocity,0);\nconsole.log('6 synthetic physics checks passed');\n"
files = {'game.cjs': game, 'index.html': html, 'check.cjs': checks}
assert 4096 < len(game.encode()) < 16000
requests = []

class ModelFixture(BaseHTTPRequestHandler):

    def log_message(self, *_):
        pass

    def do_POST(self):
        payload = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        requests.append(payload)
        assert 'small runnable slice' in payload['messages'][0]['content']
        user = next((m['content'] for m in reversed(payload['messages']) if m['role'] == 'user'))
        count = sum((m['role'] == 'tool' for m in payload['messages']))
        calls = []

        def propose(name, args):
            calls.append({'id': f'call-{len(requests)}-{len(calls)}', 'type': 'function', 'function': {'name': name, 'arguments': json.dumps(args)}})
        if user == 'build':
            if count == 0:
                for path, text in files.items():
                    propose('create_text_file', {'path': path, 'content': text})
            elif count == 3:
                propose('run_command', {'program': 'node', 'args': ['check.cjs']})
        elif user == 'oversized':
            if count == 0:
                propose('create_text_file', {'path': 'oversized.js', 'content': 'x' * (16 * 1024 + 1)})
            elif count == 1:
                assert '16 KiB' in payload['messages'][-1]['content']
                propose('create_text_file', {'path': 'small.js', 'content': 'module.exports = 42;\n'})
            elif count == 2:
                propose('run_command', {'program': 'node', 'args': ['--check', 'small.js']})
        elif user == 'cancel' and count == 0:
            propose('create_text_file', {'path': 'kept.js', 'content': game})
            propose('create_text_file', {'path': 'never.js', 'content': 'module.exports = 0;'})
        elif user == 'recover':
            if count == 0:
                propose('read_text_file', {'path': 'kept.js'})
            elif count == 1:
                propose('run_command', {'program': 'node', 'args': ['--check', 'kept.js']})
        frames = []
        if calls:
            for index, call in enumerate(calls):
                arguments = call['function'].pop('arguments')
                frames.append({'choices': [{'index': 0, 'delta': {'tool_calls': [{'index': index, **call}]}, 'finish_reason': None}]})
                for start in range(0, len(arguments), 100):
                    frames.append({'choices': [{'index': 0, 'delta': {'tool_calls': [{'index': index, 'function': {'arguments': arguments[start:start + 100]}}]}, 'finish_reason': None}]})
            frames.append({'choices': [{'index': 0, 'delta': {}, 'finish_reason': 'tool_calls'}]})
        else:
            frames.append({'choices': [{'index': 0, 'delta': {'content': 'Synthetic work validated.'}, 'finish_reason': 'stop'}]})
        frames.append({'choices': [], 'usage': {'prompt_tokens': 100, 'completion_tokens': 200}})
        self.send_response(200)
        self.send_header('Content-Type', 'text/event-stream')
        self.end_headers()
        try:
            self.wfile.write((''.join(('data: ' + json.dumps(frame) + '\n\n' for frame in frames)) + 'data: [DONE]\n\n').encode())
        except (BrokenPipeError, ConnectionResetError):
            pass
approvals = []

def run(number, session, text, cancel_second=False):
    call('start', id=number, session=session, input=text)
    deadline = time.monotonic() + 25
    seen = 0
    while time.monotonic() < deadline:
        for event in call('poll', id=number):
            if event['type'] == 'toolApproval':
                seen += 1
                request = event['request']
                approvals.append(request)
                assert request['name'] in ['create_text_file', 'read_text_file', 'run_command']
                if request['name'] == 'run_command':
                    assert request['command']['invocation']['program'] == 'node'
                if cancel_second and seen == 2:
                    call('cancel', id=number)
                else:
                    call('approveTool', id=number, callId=request['callId'], allow=True)
            if event['type'] == 'done':
                return event
        time.sleep(0.01)
    raise AssertionError('Coding diagnostic timed out')

def messages(session):
    return call('messagesPage', session=session)['items']
call('bootstrap')
server = ThreadingHTTPServer(('127.0.0.1', 0), ModelFixture)
threading.Thread(target=server.serve_forever, daemon=True).start()
try:
    policy = call('memories')['automaticPolicy']
    call('setAutomaticMemory', enabled=False, revision=policy['revision'])
    call('configure', preferences={'baseUrl': f'http://127.0.0.1:{server.server_port}/v1', 'model': 'fixture'}, apiKey='', rememberConnection=False)
    state_file = directory / 'state.json'
    if args.stage == 'save':
        project = directory / 'project'
        project.mkdir()
        session = call('createSession', kind='project', path=str(project))['session']['id']
        assert not run(1, session, 'build').get('error')
        records = messages(session)[-1]['metadata']['agent']['tools']
        assert len(records) == 4 and all((r['status'] in ['created', 'completed'] for r in records))
        result = json.loads(records[-1]['content'])
        assert result['exitCode'] == 0 and '6 synthetic physics checks passed' in result['stdout']
        assert len(approvals) == 4
        for path, text in files.items():
            assert (project / path).read_bytes() == text.encode()
        assert json.loads(records[0]['content'])['bytesAfter'] > 4096
        assert len(call('changesPage', session=session)['items']) == 3
        prior = len(approvals)
        assert not run(2, session, 'oversized').get('error')
        records = messages(session)[-1]['metadata']['agent']['tools']
        assert records[0]['status'] == 'blocked' and '16 KiB' in records[0]['content']
        assert not (project / 'oversized.js').exists()
        assert len(approvals) - prior == 2 and json.loads(records[-1]['content'])['exitCode'] == 0
        before = messages(session)
        assert run(3, session, 'cancel', True).get('error')
        assert messages(session) == before and (project / 'kept.js').read_bytes() == game.encode()
        assert not (project / 'never.js').exists()
        assert not run(4, session, 'recover').get('error')
        assert (project / 'kept.js').read_bytes() == game.encode() and (not (project / 'never.js').exists())
        assert json.loads(messages(session)[-1]['metadata']['agent']['tools'][-1]['content'])['exitCode'] == 0
        state_file.write_text(json.dumps({'session': session, 'messages': messages(session)}), encoding='utf-8')
    else:
        state = json.loads(state_file.read_text(encoding='utf-8'))
        session = state['session']
        assert messages(session) == state['messages']
        for path, text in files.items():
            assert (directory / 'project' / path).read_bytes() == text.encode()
        records = call('changesPage', session=session)['items']
        assert len(records) == 5
        assert len(messages(session)) == 6 and (not (directory / 'project/never.js').exists())
finally:
    call('shutdown')
    server.shutdown()
print(json.dumps({'ok': True, 'stage': args.stage, 'fixtureRequests': len(requests), 'approvals': len(approvals), 'liveRequests': 0, 'gameBytes': len(game.encode())}))
