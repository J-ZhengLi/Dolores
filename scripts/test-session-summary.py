"""Native summary lifecycle and exact provider wire, using isolated synthetic data.

Run save and restore in separate processes after building the Flutter bundle.
"""
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


requests = []
mode = "valid"


class Fixture(BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def do_POST(self):
        request = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        requests.append(request)
        extraction = request["messages"][0]["content"].startswith("You draft a session summary")
        answer = "Goal: build a Unicode parser. Next: test errors." if extraction else "ASSISTANT_TASK_RESULT"
        if extraction and mode == "credential":
            answer = "api_key=DO_NOT_SAVE_THIS"
        if extraction and mode == "oversized":
            answer = "x" * 8193
        if extraction and mode == "empty":
            answer = ""
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.end_headers()
        if extraction and mode == "slow":
            time.sleep(2)
        events = [{"choices": [{"delta": {"content": answer}, "finish_reason": None}]},
                  {"choices": [{"delta": {}, "finish_reason": "stop"}]},
                  {"choices": [], "usage": {"prompt_tokens": 100, "completion_tokens": 25, "total_tokens": 125}}]
        body = "".join("data: " + json.dumps(v) + "\n\n" for v in events) + "data: [DONE]\n\n"
        try:
            self.wfile.write(body.encode())
        except (BrokenPipeError, ConnectionResetError, ConnectionAbortedError):
            pass


def done(number):
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        for event in call("poll", id=number):
            assert event["type"] != "toolApproval", "Summary cannot use tools"
            if event["type"] == "done":
                return event
        time.sleep(0.01)
    call("cancel", id=number)
    raise AssertionError("Native fixture deadline")


def generate(number, session):
    review = call("reviewSummary", session=session)
    call("generateSummary", id=number, session=session, token=review["token"])
    return review, done(number)


if args.stage == "save":
    server = ThreadingHTTPServer(("127.0.0.1", 0), Fixture)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        call("configure", preferences={"baseUrl": f"http://127.0.0.1:{server.server_port}/v1", "model": "fixture"}, apiKey="", rememberConnection=True)
        call("setRequestSettings", settings={"maxOutputTokens":4096,"timeoutSeconds":8})
        session = call("createSession", kind="side")["session"]["id"]
        other = call("createSession", kind="side")["session"]["id"]
        for number in range(1, 23):
            call("start", id=number, session=session, input=f"Goal {number}: build a Unicode parser. 世界.")
            assert not done(number).get("error")
        call("saveMemory",scope="all",title="Existing",text="PREFERENCE_EXCLUDED_FROM_SUMMARY",enabled=False)
        before = len(requests)
        review = call("reviewSummary", session=session)
        assert len(requests) == before and len(review["messages"]) == 40 and review["hasMore"]
        assert not envelope("saveSummary",session=session,token=review["token"],text="Forged")["ok"]
        call("configure",preferences={"baseUrl":f"http://127.0.0.1:{server.server_port}/v1","model":"fixture"},apiKey="",rememberConnection=True,modelContexts={"fixture":1024})
        call("setRequestSettings",settings={"maxOutputTokens":512,"timeoutSeconds":8})
        before = len(requests)
        review = call("reviewSummary",session=session)
        assert not envelope("generateSummary",id=29,session=session,token=review["token"])["ok"]
        assert len(requests) == before, "Over-capacity source must never reach provider"
        call("configure",preferences={"baseUrl":f"http://127.0.0.1:{server.server_port}/v1","model":"fixture"},apiKey="",rememberConnection=True,modelContexts={"fixture":131072})
        call("setRequestSettings",settings={"maxOutputTokens":4096,"timeoutSeconds":8})
        review, event = generate(30, session)
        candidate = event["summaryDraft"]
        request = requests[-1]
        assert "tools" not in request and request["max_tokens"] == 1024 and request["stream"]
        body = json.loads(request["messages"][1]["content"])
        assert body == {"previousSummary":None,"messages":review["messages"]}
        assert "PREFERENCE_EXCLUDED" not in json.dumps(request)
        assert call("reviewSummary",session=other)["summary"] is None
        # Reviewing another chat invalidates the pending draft.
        assert not envelope("saveSummary",session=session,token=candidate["token"],text="Stale")["ok"]
        _, event = generate(31, session)
        candidate = event["summaryDraft"]
        assert not envelope("saveSummary",session=other,token=candidate["token"],text="Wrong chat")["ok"]
        assert not envelope("saveSummary",session=session,token=candidate["token"],text="api_key=secret")["ok"]
        with sqlite3.connect(fixture / "data/dolores.db") as db:
            db.execute("CREATE TRIGGER fail_summary BEFORE INSERT ON session_summaries BEGIN SELECT RAISE(ABORT,'fixture'); END;")
        assert not envelope("saveSummary",session=session,token=candidate["token"],text="Corrected summary")["ok"]
        with sqlite3.connect(fixture / "data/dolores.db") as db:
            db.execute("DROP TRIGGER fail_summary")
        saved = call("saveSummary",session=session,token=candidate["token"],text="Goal: Unicode parser. Decision: literal matching. Next: error tests.")
        assert saved["provenance"]["coveredTurns"] == 20
        assert not envelope("saveSummary",session=session,token=candidate["token"],text="Replay")["ok"]
        preview = call("context",session=session,input="Continue")
        assert preview["includedTurns"] == 2 and preview["omittedTurns"] == 0 and preview["savedTurns"] == 22
        assert preview["summary"] == saved["provenance"] and preview["sessionSummary"] == saved
        assert saved["text"] in preview["messages"][0]["content"]
        assert all("Goal 1:" not in m["content"] for m in preview["messages"])
        call("start",id=32,session=session,input="Continue")
        assert not done(32).get("error") and requests[-1]["messages"] == preview["messages"]
        call("export",session=session,path=str(fixture / "full-history.json"),format="json")
        exported = json.loads((fixture / "full-history.json").read_text(encoding="utf-8"))
        assert len(exported["messages"]) == 46 and exported["messages"][0]["content"].startswith("Goal 1:")
        context = exported["messages"][-1]["metadata"]["context"]
        assert context["summary"] == saved["provenance"] and context["tokens"] == preview["tokens"]
        assert saved["text"] not in json.dumps(context)
        review, event = generate(33,session)
        assert len(review["messages"]) == 6 and not review["hasMore"]
        assert json.loads(requests[-1]["messages"][1]["content"])["previousSummary"] == saved["text"]
        call("correctSummary",session=session,revision=1,text="Corrected older decisions.")
        assert not envelope("saveSummary",session=session,token=event["summaryDraft"]["token"],text="Stale prior")["ok"]
        review, event = generate(34,session)
        with sqlite3.connect(fixture / "data/dolores.db") as db:
            db.execute("UPDATE messages SET content=content || ' changed' WHERE id=?",(review["messages"][0]["id"],))
        assert not envelope("saveSummary",session=session,token=event["summaryDraft"]["token"],text="Stale source")["ok"]
        _, event = generate(35,session)
        saved = call("saveSummary",session=session,token=event["summaryDraft"]["token"],text="Goal: Unicode parser. Decision: literal matching. Next: error tests.")
        assert saved["provenance"]["coveredTurns"] == 23
        call("start",id=36,session=session,input="Pending work: validate errors.")
        assert not done(36).get("error")
        for number, invalid in enumerate(["credential","oversized","empty"],start=40):
            mode = invalid
            assert generate(number,session)[1].get("error")
        mode = "slow"
        review = call("reviewSummary",session=session)
        call("generateSummary",id=50,session=session,token=review["token"])
        assert not envelope("start",id=51,session=session,input="Concurrent")["ok"]
        call("cancel",id=50)
        assert done(50).get("error")
        call("setRequestSettings",settings={"maxOutputTokens":512,"timeoutSeconds":1})
        assert generate(52,session)[1].get("error")
        mode = "valid"
        review, event = generate(53,session)
        assert requests[-1]["max_tokens"] == 512
        call("discardSummaryReview",token=event["summaryDraft"]["token"])
        assert not envelope("saveSummary",session=session,token=event["summaryDraft"]["token"],text="Discarded")["ok"]
        assert call("reviewSummary",session=session)["summary"] == saved
        (fixture / "expected.json").write_text(json.dumps({"session":session,"saved":saved}),encoding="utf-8")
    finally:
        server.shutdown()
        server.server_close()
        thread.join()
else:
    expected = json.loads((fixture / "expected.json").read_text(encoding="utf-8"))
    session, saved = expected["session"], expected["saved"]
    assert call("reviewSummary",session=session)["summary"] == saved
    assert call("context",session=session,input="hello")["summary"] == saved["provenance"]
    assert not envelope("deleteSummary",session=session,revision=1)["ok"]
    call("deleteSummary",session=session,revision=saved["provenance"]["revision"])
    assert call("context",session=session,input="hello")["includedTurns"] == 24
    assert len(call("messagesPage",session=session)["items"]) == 48
    # Cascade behavior is independently covered in the SQLite unit tests.
    call("delete",session=session)
    with sqlite3.connect(fixture / "data/dolores.db") as db:
        assert db.execute("PRAGMA user_version").fetchone()[0] == 15
        assert db.execute("SELECT count(*) FROM session_summaries").fetchone()[0] == 0
call("shutdown")
print(json.dumps({"ok":True,"stage":args.stage,"scope":"summary source/wire, correction, coverage, extension, cancellation, failure, atomic save, full export and restart"}))
