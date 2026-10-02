"""Separate-process context-window/accounting diagnostic using isolated mock data.

Build the Flutter bundle and start mock-provider.mjs. Run save then restore with
the same fresh absolute --directory under output/. No personal data is read.
"""
import argparse
import ctypes
import json
import os
from pathlib import Path
import sqlite3
import time

parser = argparse.ArgumentParser()
parser.add_argument("stage", choices=["save", "restore"])
parser.add_argument("--directory", required=True, type=Path)
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


def call(command, **fields):
    payload = json.dumps({"command": command, **fields}).encode()
    buffer = ctypes.create_string_buffer(payload)
    pointer = native.dolores_call(buffer, len(payload))
    try:
        envelope = json.loads(ctypes.string_at(pointer))
    finally:
        native.dolores_free(pointer)
    assert envelope["ok"], envelope.get("error")
    return envelope.get("result")


def run(number, session, prompt):
    call("start", id=number, session=session, input=prompt)
    deadline = time.monotonic() + 15
    while time.monotonic() < deadline:
        for event in call("poll", id=number):
            if event["type"] == "done":
                return event
        time.sleep(0.01)
    call("cancel", id=number)
    raise AssertionError("Mock response deadline")


if args.stage == "save":
    assert call("bootstrap")["sessions"] == []
    call("configure", preferences={"baseUrl": "http://127.0.0.1:19421/v1", "model": "dolores-mock"}, apiKey="", rememberConnection=True,
         enabledModels=["dolores-mock", "dolores-fast"], modelContexts={"dolores-mock": 1024, "dolores-fast": None})
    call("setRequestSettings", settings={"maxOutputTokens": 128, "timeoutSeconds": 8})
    session = call("createSession", kind="side")["session"]["id"]
    with sqlite3.connect(fixture / "data/dolores.db") as db:
        db.executemany("INSERT INTO messages(session_id,role,content) VALUES(?,?,?)",
                       [(session, role, text) for n in range(6) for role, text in [("user", str(n) + "x" * 400), ("assistant", "a" * 400)]])
    preview = call("context", session=session, input="hello")
    tokens = preview["tokens"]
    assert tokens["contextWindowTokens"] == 1024 and tokens["maxInputTokens"] == 844
    assert tokens["inputTokens"] <= 844 and preview["includedTurns"] < 6
    assert preview["omittedTurns"] == 6 - preview["includedTurns"]
    assert sum(tokens[key] for key in ("systemTokens", "historyTokens", "draftTokens", "toolTokens", "framingTokens")) == tokens["inputTokens"]
    assert not run(1, session, "hello").get("error")
    messages = call("messagesPage", session=session)["items"]
    metadata = messages[-1]["metadata"]
    assert metadata["context"]["tokens"] == tokens
    assert metadata["usage"]["inputTokens"] == 64 and metadata["usage"]["totalTokens"] == 96
    assert metadata["usage"]["inputTokens"] != tokens["inputTokens"]
    assert "context budget" in run(2, session, "x" * 4000)["error"]
    assert call("messagesPage", session=session)["items"] == messages
    call("export", session=session, path=str(fixture / "chat.json"), format="json")
    call("export", session=session, path=str(fixture / "chat.md"), format="markdown")
    assert json.loads((fixture / "chat.json").read_text(encoding="utf-8"))["messages"][-1]["metadata"]["context"]["tokens"] == tokens
    call("selectModel", model="dolores-fast")
    default = call("context", input="", tools=True)
    assert default["tokens"]["contextWindowTokens"] == 131072
    assert default["tokens"]["toolTokens"] > 0 and len(default["tools"]) == 6
    (fixture / "expected.json").write_text(json.dumps({"session": session, "metadata": metadata}), encoding="utf-8")
else:
    state = call("bootstrap")
    assert state["configured"] and state["preferences"]["model"] == "dolores-fast"
    assert state["modelContexts"] == {"dolores-fast": None, "dolores-mock": 1024}
    assert call("context", input="")["tokens"]["contextWindowTokens"] == 131072
    expected = json.loads((fixture / "expected.json").read_text(encoding="utf-8"))
    assert call("messagesPage", session=expected["session"])["items"][-1]["metadata"] == expected["metadata"]
    call("selectModel", model="dolores-mock")
    assert call("context", session=expected["session"], input="")["tokens"]["contextWindowTokens"] == 1024
    with sqlite3.connect(fixture / "data/dolores.db") as db:
        assert db.execute("PRAGMA user_version").fetchone()[0] == 9
        assert db.execute("SELECT COUNT(*) FROM messages").fetchone()[0] == 14
call("shutdown")
print("TOKEN CONTEXT " + args.stage.upper() + " PASSED")
