"""Normal native bundle: complete-boundary forks and bounded compaction recovery."""
import ctypes
import json
import os
from pathlib import Path
import sqlite3
import sys
import subprocess
import tempfile
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

root = Path(__file__).resolve().parents[1]
if len(sys.argv) == 1:
    with tempfile.TemporaryDirectory(prefix="dolores-threads-") as path:
        subprocess.run([sys.executable, __file__, path], check=True, timeout=40)
    raise SystemExit(0)
directory = Path(sys.argv[1])
os.environ["DOLORES_DATA_DIR"] = str(directory / "data")
os.environ["DOLORES_GLOBAL_SKILLS_DIR"] = str(directory / "skills")
bundle = root / "apps/dolores_flutter/build/windows/x64/runner/Release"
loader = os.add_dll_directory(str(bundle)) if os.name == "nt" else None
native = ctypes.CDLL(str(bundle / ("dolores_flutter_bridge.dll" if os.name == "nt" else "lib/libdolores_flutter_bridge.so")))
native.dolores_call.argtypes = [ctypes.c_void_p, ctypes.c_size_t]
native.dolores_call.restype = ctypes.c_void_p
native.dolores_free.argtypes = [ctypes.c_void_p]

def envelope(command, **fields):
    raw = json.dumps({"command": command, **fields}).encode()
    buffer = ctypes.create_string_buffer(raw)
    pointer = native.dolores_call(buffer, len(raw))
    try:
        return json.loads(ctypes.string_at(pointer))
    finally:
        native.dolores_free(pointer)

def call(command, **fields):
    result = envelope(command, **fields)
    assert result["ok"], result.get("error")
    return result.get("result")

requests = []
invalid = False
class Handler(BaseHTTPRequestHandler):
    def log_message(self, *_): pass
    def do_POST(self):
        payload = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        requests.append(payload)
        summary = payload["messages"][0]["content"].startswith("You draft a session summary")
        if summary:
            assert "tools" not in payload
            assert payload["max_tokens"] <= 1024
            text = "x" * 8193 if invalid else "Goal: build a parser. Pending: verify Unicode errors."
        else:
            assert "Original goal: Build parser" in payload["messages"][0]["content"]
            text = "Grounded fixture reply."
        self.send_response(200); self.send_header("Content-Type", "text/event-stream"); self.end_headers()
        for value in [{"choices":[{"delta":{"content":text},"finish_reason":None}]}, {"choices":[{"delta":{},"finish_reason":"stop"}]}, {"choices":[],"usage":{"prompt_tokens":120,"completion_tokens":20,"total_tokens":140}}]:
            self.wfile.write(("data: " + json.dumps(value) + "\n\n").encode())
        self.wfile.write(b"data: [DONE]\n\n")

def seed(session, count):
    with sqlite3.connect(directory / "data/dolores.db") as db:
        for number in range(count):
            db.executemany("INSERT INTO messages(session_id,role,content) VALUES(?,?,?)", [(session,"user",f"Build parser {number}. " + "u"*380),(session,"assistant","a"*400)])

def finish(number):
    deadline = time.monotonic() + 15
    events = []
    while time.monotonic() < deadline:
        for event in call("poll",id=number):
            events.append(event)
            assert event["type"] != "toolApproval"
            if event["type"] == "done": return event, events
        time.sleep(.02)
    call("cancel",id=number)
    raise AssertionError("Bounded fixture timeout")

server = ThreadingHTTPServer(("127.0.0.1",0), Handler)
threading.Thread(target=server.serve_forever,daemon=True).start()
try:
    call("bootstrap")
    policy = call("memories")["automaticPolicy"]
    call("setAutomaticMemory",enabled=False,revision=policy["revision"])
    preferences = {"baseUrl":f"http://127.0.0.1:{server.server_port}/v1","model":"fixture"}
    # 8K admits a complete source batch plus the current fixed host context.
    # The separately exercised 4K case must stop rather than omit more history.
    call("configure",preferences=preferences,apiKey="",rememberConnection=False,modelContexts={"fixture":8192})
    call("setRequestSettings",settings={"maxOutputTokens":128,"timeoutSeconds":60})
    session = call("createSession",kind="side")["session"]["id"]
    seed(session,30)
    call("saveScopedSettings",session=session,scope="thread",revision=0,patch={"task":{"modelCalls":4,"toolCalls":4,"segments":4,"elapsedSeconds":180}})
    call("saveDraft",session=session,text="keep draft")
    page = call("messagesPage",session=session)["items"]
    assert not envelope("forkSession",session=session,through=page[0]["id"])["ok"]
    fork = call("forkSession",session=session,through=page[1]["id"])["session"]["id"]
    assert len(call("messagesPage",session=fork)["items"]) == 2
    assert call("savedDraft",session=fork) == ""
    assert call("runs",session=fork) == []
    assert call("taskPermissions",session=fork)["revision"] == 0
    # New chats and old opt-out rows both keep automatic context protection.
    assert call("reviewSummary",session=session)["autoCompact"] is True
    with sqlite3.connect(directory / "data/dolores.db") as db:
        db.execute("INSERT INTO thread_context(session_id,auto_compact) VALUES(?,0)", (session,))
    assert call("reviewSummary",session=session)["autoCompact"] is True
    assert not envelope("setAutoCompact",session=session,enabled=False)["ok"]
    preview = call("context",session=session,input="continue",tools=False)
    assert preview["omittedTurns"] > 0 and len(requests) == 0
    call("start",id=1,session=session,input="continue")
    done, events = finish(1)
    assert not done.get("error"), done
    assert len(requests) == 2
    assert any(e["type"] == "compacted" for e in events)
    saved = call("reviewSummary",session=session)["summary"]
    assert saved["text"].startswith("Original goal: Build parser")
    evidence = call("runEvents",session=session,runId=done["runId"])
    assert any(e["kind"] == "compacted" and e["data"]["usage"]["inputTokens"] == 120 for e in evidence)
    seed(session,20); invalid = True
    before = len(requests)
    call("start",id=2,session=session,input="finish tests")
    failed, events = finish(2)
    assert "Last valid summary" in failed["error"]
    assert len(requests) == before + 1
    assert call("reviewSummary",session=session)["summary"] == saved
    assert call("savedDraft",session=session) == "finish tests"
    # A failed automatic summary never retries by itself. Explicit send recovers.
    invalid = False
    before = len(requests)
    call("start",id=3,session=session,input="finish tests")
    recovered, events = finish(3)
    assert not recovered.get("error"), recovered
    assert len(requests) == before + 2
    assert call("reviewSummary",session=session)["summary"]["provenance"]["revision"] > saved["provenance"]["revision"]
    tiny = call("createSession",kind="side")["session"]["id"]
    seed(tiny,30)
    call("configure",preferences=preferences,apiKey="",rememberConnection=False,modelContexts={"fixture":4096})
    before = len(requests)
    call("start",id=4,session=tiny,input="continue tiny window")
    limited, events = finish(4)
    assert "valid summary but this window still omits history" in limited["error"], limited
    checkpoint = call("reviewSummary",session=tiny)["summary"]
    assert checkpoint and len(requests) == before + 1
    assert call("savedDraft",session=tiny) == "continue tiny window"
    with sqlite3.connect(directory / "data/dolores.db") as db:
        assert db.execute("SELECT count(*) FROM messages WHERE session_id=?", (tiny,)).fetchone()[0] == 60
    call("configure",preferences=preferences,apiKey="",rememberConnection=False,modelContexts={"fixture":8192})
    call("start",id=5,session=tiny,input="continue tiny window")
    resumed, events = finish(5)
    assert not resumed.get("error"), resumed
    assert len(requests) == before + 2
    assert call("reviewSummary",session=tiny)["summary"] == checkpoint
    (directory / "receipt.json").write_text(json.dumps({"tinyWindowFailure":limited,"explicitRecovery":resumed,"providerRequests":len(requests)},indent=2),encoding="utf-8")
    print(json.dumps({"passed":True,"forkBoundary":True,"oneCompaction":True,"usageRecorded":True,"failedSummaryPreserved":True,"draftPreserved":True}))
finally:
    call("shutdown");server.shutdown()
