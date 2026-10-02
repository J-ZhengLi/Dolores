"""Separate-process native FFI diagnostic, using only the local keyless fixture.

Build the Flutter bundle first and run the mock provider on port 19421.
Run save, then restore with the same fresh, absolute --directory under output/.
No real provider settings or files are read by this diagnostic.
"""
import argparse
import ctypes
import json
import os
import pathlib
import sqlite3
import time

parser = argparse.ArgumentParser()
parser.add_argument("stage", choices=["save", "restore"])
parser.add_argument("--directory", required=True, type=pathlib.Path)
args = parser.parse_args()
root = pathlib.Path(__file__).resolve().parents[1]
fixture = args.directory
if not fixture.is_absolute() or not fixture.resolve().is_relative_to(root / "output"):
    raise SystemExit("Use an absolute isolated directory under output/.")
if args.stage == "save":
    fixture.mkdir(parents=True, exist_ok=False)
os.environ["DOLORES_DATA_DIR"] = str(fixture / "data")
bundle = root / "apps/dolores_flutter/build/windows/x64/runner/Release"
loader = os.add_dll_directory(str(bundle)) if os.name == "nt" else None
library = "dolores_flutter_bridge.dll" if os.name == "nt" else "lib/libdolores_flutter_bridge.so"
native = ctypes.CDLL(str(bundle / library))
native.dolores_call.argtypes = [ctypes.c_void_p, ctypes.c_size_t]
native.dolores_call.restype = ctypes.c_void_p
native.dolores_free.argtypes = [ctypes.c_void_p]


def call(command, *, fails=False, **fields):
    payload = json.dumps({"command": command, **fields}).encode()
    buffer = ctypes.create_string_buffer(payload)
    pointer = native.dolores_call(buffer, len(payload))
    try:
        envelope = json.loads(ctypes.string_at(pointer))
    finally:
        native.dolores_free(pointer)
    if fails:
        assert not envelope["ok"], "Expected local refusal"
        return
    assert envelope["ok"], envelope.get("error")
    return envelope.get("result")


project = fixture / "project"
file = project / "readme.txt"
before = "Hello from an approved workspace file. 世界."
after = "Updated with an approved edit. 世界."


def write(text):
    file.write_text(text, encoding="utf-8")


def run(run_id, session):
    call("start", id=run_id, session=session, input="tool-edit")
    deadline = time.monotonic() + 15
    while time.monotonic() < deadline:
        for event in call("poll", id=run_id):
            if event["type"] == "toolApproval":
                assert file.read_text(encoding="utf-8") == before
                call("approveTool", id=run_id, callId=event["request"]["callId"], allow=True)
            if event["type"] == "done":
                return event
        time.sleep(0.01)
    raise AssertionError("Mock edit deadline")


if args.stage == "save":
    assert call("bootstrap")["sessions"] == []
    project.mkdir()
    write(before)
    call("configure", preferences={"baseUrl": "http://127.0.0.1:19421/v1", "model": "dolores-mock"}, apiKey="", rememberConnection=True)
    state = call("createSession", kind="project", path=str(project))
    session = state["session"]["id"]
    assert not run(1, session).get("error")
    saved = call("messagesPage", session=session)["items"]
    db = sqlite3.connect(fixture / "data/dolores.db")
    db.execute("CREATE TRIGGER fail_chat BEFORE INSERT ON turn_metadata BEGIN SELECT RAISE(ABORT,'fixture'); END")
    db.commit()
    write(before)
    assert run(2, session).get("error")
    assert file.read_text(encoding="utf-8") == after
    assert call("messagesPage", session=session)["items"] == saved
    assert len(call("changesPage", session=session)["items"]) == 2
    db.execute("DROP TRIGGER fail_chat")
    db.execute("CREATE TRIGGER fail_receipt BEFORE UPDATE ON file_changes WHEN NEW.status='applied' BEGIN SELECT RAISE(ABORT,'fixture'); END")
    db.commit()
    write(before)
    assert not run(3, session).get("error")
    assert file.read_text(encoding="utf-8") == after
    latest = call("changesPage", session=session)["items"][0]
    assert latest["status"] == "pending"
    result = json.loads(call("messagesPage", session=session)["items"][-1]["metadata"]["agent"]["tools"][0]["content"])
    assert result["applied"] and result["journalStatus"] == "pending"
    db.execute("DROP TRIGGER fail_receipt")
    db.execute("CREATE TRIGGER fail_intent BEFORE INSERT ON file_changes BEGIN SELECT RAISE(ABORT,'fixture'); END")
    db.commit()
    write(before)
    assert not run(4, session).get("error")  # Model reports the refused edit.
    assert file.read_text(encoding="utf-8") == before
    assert len(call("changesPage", session=session)["items"]) == 3
    db.execute("DROP TRIGGER fail_intent")
    db.commit()
    db.close()
    write(after)
    preview = call("previewRevert", session=session, changeId=latest["id"])
    call("cancelRevert", token=preview["token"])
    call("applyRevert", session=session, token=preview["token"], fails=True)
    old_token = call("previewRevert", session=session, changeId=latest["id"])["token"]
    call("delete", session=session)
    state = call("createSession", kind="project", path=str(project))
    assert len(call("changesPage", session=state["session"]["id"])["items"]) == 3
    (fixture / "state.json").write_text(json.dumps({"state": state, "changeId": latest["id"], "oldToken": old_token}), encoding="utf-8")
    print("Native journal: failed-chat edit retained; receipt failure pending; intent failure prevented write; deletion and cancellation preserved files.")
else:
    saved = json.loads((fixture / "state.json").read_text(encoding="utf-8"))
    session = saved["state"]["session"]["id"]
    assert call("bootstrap")["configured"]
    assert len(call("changesPage", session=session)["items"]) == 3
    assert call("messagesPage", session=session)["items"] == []
    call("applyRevert", session=session, token=saved["oldToken"], fails=True)
    detail = call("changeDetails", session=session, changeId=saved["changeId"])
    assert "root" not in detail and "+Updated with" in detail["diff"]
    preview = call("previewRevert", session=session, changeId=saved["changeId"])
    assert "+Hello from" in preview["diff"]
    write("external change")
    call("applyRevert", session=session, token=preview["token"], fails=True)
    assert file.read_text(encoding="utf-8") == "external change"
    write(after)
    preview = call("previewRevert", session=session, changeId=saved["changeId"])
    result = call("applyRevert", session=session, token=preview["token"])
    assert result["applied"] and result["journalStatus"] == "applied"
    assert file.read_text(encoding="utf-8") == before
    call("applyRevert", session=session, token=preview["token"], fails=True)
    page = call("changesPage", session=session)["items"]
    assert len(page) == 4 and page[0]["reverts"] == saved["changeId"] and page[1]["status"] == "reverted"
    call("previewRevert", session=session, changeId=page[2]["id"], fails=True)
    print("Native restart: snapshots and pending receipt restored; stale/conflicting/repeated approval refused; reviewed revert restored exact text and both receipts.")
call("shutdown")
