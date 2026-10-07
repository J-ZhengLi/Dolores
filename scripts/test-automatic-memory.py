"""Synthetic native end-to-end automatic learning and separate-process restart."""
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
parser.add_argument("stage", choices=["save", "restore"])
parser.add_argument("--directory", type=Path, required=True)
args = parser.parse_args()
root = Path(__file__).resolve().parents[1]
fixture = args.directory
if not fixture.is_absolute() or not fixture.resolve().is_relative_to(root / "output"):
    raise SystemExit("Use an absolute isolated directory under output/.")
if args.stage == "save":
    fixture.mkdir(parents=True, exist_ok=False)
os.environ["DOLORES_DATA_DIR"] = str(fixture / "data")
os.environ["DOLORES_GLOBAL_SKILLS_DIR"] = str(fixture / "global-skills")
bundle = root / "apps/dolores_flutter/build/windows/x64/runner/Release"
loader = os.add_dll_directory(str(bundle)) if os.name == "nt" else None
native = ctypes.CDLL(str(bundle / ("dolores_flutter_bridge.dll" if os.name == "nt" else "lib/libdolores_flutter_bridge.so")))
native.dolores_call.argtypes = [ctypes.c_void_p, ctypes.c_size_t]
native.dolores_call.restype = ctypes.c_void_p
native.dolores_free.argtypes = [ctypes.c_void_p]

def envelope(command, **fields):
    payload = json.dumps({"command": command, **fields}).encode()
    buffer = ctypes.create_string_buffer(payload)
    pointer = native.dolores_call(buffer, len(payload))
    try:
        return json.loads(ctypes.string_at(pointer))
    finally:
        native.dolores_free(pointer)

def call(command, **fields):
    response = envelope(command, **fields)
    assert response["ok"], response.get("error")
    return response.get("result")

requests, mode = [], "valid"
learning_started = threading.Event()
class Fixture(BaseHTTPRequestHandler):
    def log_message(self, *_): pass
    def do_POST(self):
        request = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        requests.append(request)
        learning = request["messages"][0]["content"].startswith("You extract automatic useful memory")
        if learning:
            learning_started.set()
            if mode == "denied":
                self.send_response(401); self.end_headers(); self.wfile.write(b'AUTOMATIC_RAW_DENIAL_DO_NOT_EXPOSE'); return
            source = json.loads(request["messages"][1]["content"])["sources"][0]
            suggestion = {"title":"Response style", "text":source["text"], "quote":source["text"], "messageId":source["messageId"]}
            if mode == "invented": suggestion["quote"] = "I prefer fabricated replies."
            if mode == "broadened": suggestion["text"] = "Always give short replies."
            answer = json.dumps({"suggestions": [] if mode == "empty" else [suggestion]})
            if mode == "malformed": answer = "broken json"
            if mode == "oversized": answer = "x" * 8193
        else:
            answer = "ASSISTANT_EXCLUDED"
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.end_headers()
        if learning and mode == "slow": time.sleep(2)
        events = [{"choices":[{"delta":{"content":answer},"finish_reason":None}]},
                  {"choices":[{"delta":{},"finish_reason":"stop"}]},
                  {"choices":[],"usage":{"prompt_tokens":100,"completion_tokens":25,"total_tokens":125}}]
        try: self.wfile.write(("".join("data: "+json.dumps(v)+"\n\n" for v in events)+"data: [DONE]\n\n").encode())
        except (BrokenPipeError, ConnectionResetError, ConnectionAbortedError): pass

def done(number, stop_learning=False):
    deadline = time.monotonic() + 15
    while time.monotonic() < deadline:
        for event in call("poll", id=number):
            if event["type"] == "done":
                assert not event.get("error"), event
                if (event.get("memoryUpdate") or {}).get("status") == "queued":
                    if stop_learning:
                        assert learning_started.wait(3)
                        set_policy(False)
                    memory_deadline=time.monotonic()+12
                    while time.monotonic()<memory_deadline:
                        attempt=call("memories",session=current_session)["automaticAttempt"]
                        if attempt and attempt["status"]!="updating":
                            event["memoryUpdate"]=attempt
                            break
                        time.sleep(.01)
                    else: raise AssertionError("Background memory did not settle")
                return event
        time.sleep(.01)
    call("cancel", id=number)
    raise AssertionError("Native fixture deadline")

def run(number, session, text):
    global current_session
    current_session=session
    call("start", id=number, session=session, input=text)
    return done(number)

def items(session): return call("memories", session=session)["items"]

def set_policy(enabled):
    policy=call("memories")["automaticPolicy"]
    return call("setAutomaticMemory",enabled=enabled,revision=policy["revision"])

if args.stage == "save":
    server = ThreadingHTTPServer(("127.0.0.1", 0), Fixture)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    try:
        call("configure", preferences={"baseUrl":f"http://127.0.0.1:{server.server_port}/v1","model":"fixture"},apiKey="",rememberConnection=True)
        call("setRequestSettings",settings={"maxOutputTokens":4096,"timeoutSeconds":8})
        assert not call("memories")["automaticPolicy"]["enabled"]
        set_policy(True)
        project = fixture / "project"; project.mkdir()
        session = call("createSession",kind="project",path=str(project))["session"]["id"]
        side = call("createSession",kind="side")["session"]["id"]
        result = run(1,session,"I prefer concise replies with examples.")
        learned = items(session)[0]
        assert result["memoryUpdate"]["saved"] == 1 and learned["source"] == "automatic"
        assert learned["text"] == learned["origin"]["quote"] and learned["autoUpdate"]
        assert items(side) == []
        assert len(requests) == 1 and result["memoryUpdate"].get("usage") is None
        assert learned["origin"]["model"] == "local-excerpt"
        preview = call("context",session=session,input="Follow up")
        assert learned["text"] in preview["messages"][0]["content"]
        assert preview["memory"]["used"][0]["source"] == "automatic"
        assert run(2,session,learned["text"])["memoryUpdate"]["skipped"] == 1
        assert run(3,session,"I prefer detailed replies with examples.")["memoryUpdate"]["saved"] == 0
        assert run(4,session,"From now on I prefer detailed replies instead.")["memoryUpdate"]["saved"] == 1
        corrected = items(session)[0]; assert corrected["id"] == learned["id"] and corrected["revision"] == 2
        call("saveMemory",session=session,scope="folder",id=corrected["id"],revision=2,title=corrected["title"],text="USER_CORRECTED",enabled=True)
        assert run(5,session,"From now on I prefer concise replies instead.")["memoryUpdate"]["skipped"] == 1
        assert items(session)[0]["text"] == "USER_CORRECTED" and not items(session)[0]["autoUpdate"]
        before = len(requests)
        for n,text in enumerate(["Read this file.","> I prefer concise replies.","I prefer api_key=DO_NOT_SAVE", "I want you to read a file.","For this answer I prefer brief replies."],6):
            assert run(n,session,text)["memoryUpdate"]["status"] == "skipped"
        assert len(requests) == before + 5
        policy = call("memories",session=session)["automaticPolicy"]
        call("setAutomaticMemory",enabled=False,revision=policy["revision"])
        before=len(requests);assert run(11,session,"I prefer concise replies.")["memoryUpdate"] is None
        assert len(requests)==before+1
        set_policy(True)
        # A side-chat preference learns in All chats, with independent usage.
        result=run(12,side,"I prefer clear examples with assumptions labeled.")
        assert result["memoryUpdate"]["saved"]==1 and result["memoryUpdate"]["usage"]["inputTokens"]==100
        global_pref=items(side)[0]
        call("deleteMemory",scope="all",id=global_pref["id"],revision=1)
        for number,bad in enumerate(["invented","broadened","malformed","empty"],13):
            mode=bad;result=run(number,side,"I prefer clear examples with assumptions labeled.")
            assert items(side)==[] and len(call("messagesPage",session=side)["items"])==(number-11)*2
            assert result["memoryUpdate"]["saved"]==0
        mode="slow";learning_started.clear()
        current_session=side
        call("start",id=17,session=side,input="I prefer clear examples with assumptions labeled.")
        assert done(17,stop_learning=True)["memoryUpdate"]["status"]=="stopped"
        assert items(side)==[]
        set_policy(True)
        call("setRequestSettings",settings={"maxOutputTokens":512,"timeoutSeconds":1})
        assert run(18,side,"I prefer clear examples with assumptions labeled.")["memoryUpdate"]["status"]=="failed"
        assert items(side)==[]
        mode="valid"
        # Lower context capacity refuses extraction without making the extra request.
        call("configure",preferences={"baseUrl":f"http://127.0.0.1:{server.server_port}/v1","model":"fixture"},apiKey="",rememberConnection=True,modelContexts={"fixture":1024})
        bounded=call("createSession",kind="side")["session"]["id"]
        capacity=call("saveMemory",scope="all",title="Capacity fixture",text="x"*1024,enabled=False)
        before=len(requests)
        assert run(19,bounded,"I prefer concise replies with assumptions labeled.")["memoryUpdate"]["status"]=="failed"
        assert len(requests)==before+1
        call("deleteMemory",scope="all",id=capacity["id"],revision=1)
        call("configure",preferences={"baseUrl":f"http://127.0.0.1:{server.server_port}/v1","model":"fixture"},apiKey="",rememberConnection=True,modelContexts={"fixture":131072})
        for number,bad in enumerate(["oversized","denied"],20):
            mode=bad; result=run(number,side,"I prefer clear examples with assumptions labeled.")
            assert result["memoryUpdate"]["status"]=="failed" and items(side)==[]
            assert "AUTOMATIC_RAW_DENIAL" not in json.dumps(result)
        call("export",session=session,path=str(fixture/"conversation.json"),format="json")
        exported=json.loads((fixture/"conversation.json").read_text())
        assert any(m.get("metadata",{}).get("context",{}).get("memory",{}).get("used",[]) for m in exported["messages"] if m["role"]=="assistant")
        (fixture/"state.json").write_text(json.dumps({"session":session,"side":side,"id":corrected["id"]}),encoding="utf-8")
        print(json.dumps({"ok":True,"stage":"save","fixtureRequests":len(requests),"liveRequests":0,"scopedAutomaticLearning":True,"correctionAndProtection":True,"failureKeepsReplies":True}))
    finally: call("shutdown");server.shutdown();server.server_close()
else:
    state=json.loads((fixture/"state.json").read_text())
    assert call("memories",session=state["session"])["automaticPolicy"]["enabled"]
    learned=items(state["session"])[0]
    assert learned["id"]==state["id"] and learned["text"]=="USER_CORRECTED" and learned["revision"]==3
    assert learned["origin"]["quote"]=="From now on I prefer detailed replies instead."
    assert items(state["side"])==[]
    call("deleteMemory",session=state["session"],scope="folder",id=learned["id"],revision=3)
    assert items(state["session"])==[]
    with sqlite3.connect(fixture/"data/dolores.db") as db:
        assert db.execute("PRAGMA integrity_check").fetchone()[0]=="ok"
    print(json.dumps({"ok":True,"stage":"restore","fixtureRequests":len(requests),"liveRequests":0,"policyAndProvenancePreserved":True}))
    call("shutdown")
