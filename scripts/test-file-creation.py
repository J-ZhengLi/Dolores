"""Creation/removal restart diagnostic. Uses only an isolated local keyless fixture.

Build the Flutter bundle, start mock-provider.mjs, then run save and restore with
the same fresh absolute --directory under output/. No personal data is read.
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
native = ctypes.CDLL(str(bundle / ("dolores_flutter_bridge.dll" if os.name == "nt" else "lib/libdolores_flutter_bridge.so")))
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
file = project / "created.txt"
empty = project / "empty.txt"
content = "# Created with approval\r\nHello 世界.\r\n".encode()


def run(run_id, session, prompt="tool-create"):
    call("start", id=run_id, session=session, input=prompt)
    deadline = time.monotonic() + 15
    while time.monotonic() < deadline:
        for event in call("poll", id=run_id):
            if event["type"] == "toolApproval":
                request = event["request"]
                assert request["name"] == "create_text_file"
                assert request["diff"].startswith("--- /dev/null\n")
                assert not (project / request["target"]).exists()
                call("approveTool", id=run_id, callId=request["callId"], allow=True)
            if event["type"] == "done":
                return event
        time.sleep(0.01)
    raise AssertionError("Mock creation deadline")


if args.stage == "save":
    assert call("bootstrap")["sessions"] == []
    project.mkdir()
    call("configure", preferences={"baseUrl": "http://127.0.0.1:19421/v1", "model": "dolores-mock"}, apiKey="", rememberConnection=True)
    session = call("createSession", kind="project", path=str(project))["session"]["id"]
    db = sqlite3.connect(fixture / "data/dolores.db")
    db.execute("CREATE TRIGGER fail_intent BEFORE INSERT ON file_changes BEGIN SELECT RAISE(ABORT,'fixture'); END")
    db.commit()
    assert not run(1, session).get("error")
    assert not file.exists() and not call("changesPage", session=session)["items"]
    db.execute("DROP TRIGGER fail_intent")
    db.execute("CREATE TRIGGER fail_chat BEFORE INSERT ON turn_metadata BEGIN SELECT RAISE(ABORT,'fixture'); END")
    db.commit()
    saved = call("messagesPage", session=session)["items"]
    assert run(2, session).get("error")
    assert file.read_bytes() == content
    assert call("messagesPage", session=session)["items"] == saved
    assert call("changesPage", session=session)["items"][0]["status"] == "applied"
    db.execute("DROP TRIGGER fail_chat")
    db.execute("CREATE TRIGGER fail_receipt BEFORE UPDATE ON file_changes WHEN NEW.status='applied' BEGIN SELECT RAISE(ABORT,'fixture'); END")
    db.commit()
    file.unlink()  # Remove isolated fixture to exercise a second fresh creation.
    assert not run(3, session).get("error")
    assert file.read_bytes() == content
    created = call("changesPage", session=session)["items"][0]
    assert created["status"] == "pending" and not created["beforeExists"] and created["afterExists"]
    result = json.loads(call("messagesPage", session=session)["items"][-1]["metadata"]["agent"]["tools"][0]["content"])
    assert result["created"] and result["journalStatus"] == "pending"
    db.execute("DROP TRIGGER fail_receipt")
    db.commit()
    assert not run(4, session, "tool-create-empty").get("error")
    assert empty.exists() and empty.read_bytes() == b""
    empty_id = call("changesPage", session=session)["items"][0]["id"]
    preview = call("previewRevert", session=session, changeId=created["id"])
    assert preview["operation"] == "remove" and "+++ /dev/null" in preview["diff"]
    old_token = preview["token"]
    call("delete", session=session)
    session = call("createSession", kind="project", path=str(project))["session"]["id"]
    assert len(call("changesPage", session=session)["items"]) == 3
    assert sorted(p.name for p in project.iterdir()) == ["created.txt", "empty.txt"]
    (fixture / "state.json").write_text(json.dumps({"session": session, "created": created["id"], "empty": empty_id, "oldToken": old_token}), encoding="utf-8")
    db.close()
    print("Creation: intent failure refused; failed-chat creation retained; receipt failure stayed pending; empty creation and snapshots survived chat deletion.")
else:
    saved = json.loads((fixture / "state.json").read_text(encoding="utf-8"))
    session = saved["session"]
    assert call("bootstrap")["configured"]
    assert call("messagesPage", session=session)["items"] == []
    call("applyRevert", session=session, token=saved["oldToken"], fails=True)
    detail = call("changeDetails", session=session, changeId=saved["created"])
    assert "root" not in detail and not detail["change"]["beforeExists"]
    assert "--- /dev/null" in detail["diff"]
    preview = call("previewRevert", session=session, changeId=saved["created"])
    call("cancelRevert", token=preview["token"])
    call("applyRevert", session=session, token=preview["token"], fails=True)
    assert file.read_bytes() == content
    preview = call("previewRevert", session=session, changeId=saved["created"])
    file.write_bytes(b"external change")
    call("applyRevert", session=session, token=preview["token"], fails=True)
    assert file.read_bytes() == b"external change"
    file.write_bytes(content)
    preview = call("previewRevert", session=session, changeId=saved["created"])
    result = call("applyRevert", session=session, token=preview["token"])
    assert result["removed"] and result["journalStatus"] == "applied" and not file.exists()
    call("applyRevert", session=session, token=preview["token"], fails=True)
    call("previewRevert", session=session, changeId=result["changeId"], fails=True)
    db = sqlite3.connect(fixture / "data/dolores.db")
    db.execute("CREATE TRIGGER fail_removal_receipt BEFORE UPDATE ON file_changes WHEN NEW.status='applied' BEGIN SELECT RAISE(ABORT,'fixture'); END")
    db.commit()
    preview = call("previewRevert", session=session, changeId=saved["empty"])
    assert "+++ /dev/null" in preview["diff"] and empty.exists()
    result = call("applyRevert", session=session, token=preview["token"])
    assert result["removed"] and result["journalStatus"] == "pending" and not empty.exists()
    call("previewRevert", session=session, changeId=saved["empty"], fails=True)
    db.execute("DROP TRIGGER fail_removal_receipt")
    db.commit()
    db.close()
    page = call("changesPage", session=session)["items"]
    assert len(page) == 5 and page[0]["afterExists"] is False and page[0]["status"] == "pending"
    assert page[1]["reverts"] == saved["created"]
    assert not list(project.iterdir())
    print("Restart: empty/pending creations restored; stale/canceled/conflicting removal refused; single-use removal and failed receipt correctly recorded actual file absence.")
call("shutdown")
