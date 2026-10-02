"""Command execution/restart diagnostic against an isolated keyless fixture.

Build the Flutter bundle and start mock-provider.mjs. Run save then restore with
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
proof = project / "command-proof.txt"
late = project / "command-late.txt"


def run(number, session, prompt="tool-command", policy="allow"):
    call("start", id=number, session=session, input=prompt)
    deadline = time.monotonic() + 20
    approval = None
    while time.monotonic() < deadline:
        for event in call("poll", id=number):
            if event["type"] == "toolApproval":
                approval = event["request"]
                assert approval["name"] == "run_command"
                assert approval["target"] == "node"
                command = approval["command"]
                assert pathlib.Path(command["executable"]).is_absolute()
                assert command["invocation"]["program"] == "node"
                assert command["invocation"]["args"][0] == "-e"
                if policy == "stop":
                    call("cancel", id=number)
                else:
                    call("approveTool", id=number, callId=approval["callId"], allow=policy != "deny")
                    call("approveTool", id=number, callId=approval["callId"], allow=True, fails=True)
            if event["type"] == "done":
                return event, approval
        # Execution can have no bridge events between approval and its result.
        if policy == "running-stop" and (project / "command-started.txt").exists():
            call("cancel", id=number)
            policy = "cancelled"
        time.sleep(0.01)
    call("cancel", id=number)
    raise AssertionError("Mock command deadline")


if args.stage == "save":
    assert call("bootstrap")["sessions"] == []
    project.mkdir()
    call("configure", preferences={"baseUrl": "http://127.0.0.1:19421/v1", "model": "dolores-mock"}, apiKey="", rememberConnection=True)
    session = call("createSession", kind="project", path=str(project))["session"]["id"]
    assert not run(1, session, policy="deny")[0].get("error")
    assert not proof.exists()
    saved = call("messagesPage", session=session)["items"]
    assert saved[-1]["metadata"]["agent"]["tools"][0]["status"] == "denied"
    assert run(2, session, policy="stop")[0].get("error")
    assert not proof.exists() and call("messagesPage", session=session)["items"] == saved
    event, approval = run(3, session)
    assert not event.get("error")
    assert proof.read_text(encoding="utf-8") == "validated 世界"
    messages = call("messagesPage", session=session)["items"]
    record = messages[-1]["metadata"]["agent"]["tools"][0]
    assert record["command"] == approval["command"]["invocation"]
    assert "executable" not in record["command"]
    result = json.loads(record["content"])
    assert result["exitCode"] == 7 and result["stdout"] == "Checked 世界" and result["stderr"] == "diagnostic"
    assert not call("changesPage", session=session)["items"]
    for fmt, ext in [("json", "json"), ("markdown", "md")]:
        destination = fixture / f"chat.{ext}"
        call("export", session=session, path=str(destination), format=fmt)
        exported = destination.read_text(encoding="utf-8")
        assert "run_command" in exported and "Checked 世界" in exported
        assert approval["command"]["executable"] not in exported
        assert str(project) not in exported
    db = sqlite3.connect(fixture / "data/dolores.db")
    db.execute("CREATE TRIGGER fail_chat BEFORE INSERT ON turn_metadata BEGIN SELECT RAISE(ABORT,'fixture'); END")
    db.commit()
    proof.unlink()  # Only this isolated diagnostic file.
    assert run(4, session)[0].get("error")
    assert proof.read_text(encoding="utf-8") == "validated 世界"
    assert call("messagesPage", session=session)["items"] == messages
    assert not call("changesPage", session=session)["items"]
    db.execute("DROP TRIGGER fail_chat")
    db.commit()
    db.close()
    assert not run(5, session, "tool-command-output")[0].get("error")
    messages = call("messagesPage", session=session)["items"]
    shortened = json.loads(messages[-1]["metadata"]["agent"]["tools"][0]["content"])
    assert shortened["reason"] == "outputLimit" and shortened["truncated"]
    assert len(messages[-1]["metadata"]["agent"]["tools"][0]["content"].encode()) <= 16384
    stopped, _ = run(6, session, "tool-command-slow", "running-stop")
    assert stopped.get("error"), stopped
    assert (project / "command-started.txt").exists() and not late.exists(), stopped
    assert call("messagesPage", session=session)["items"] == messages
    (fixture / "state.json").write_text(json.dumps({"session": session, "messages": messages}), encoding="utf-8")
    print("Command: Deny/Stop/repeated decisions refused; exit/stderr/Unicode/output cap preserved; failed-chat effects retained with atomic history; exact arguments exported without automatic local paths.")
else:
    saved = json.loads((fixture / "state.json").read_text(encoding="utf-8"))
    assert call("bootstrap")["configured"]
    assert call("messagesPage", session=saved["session"])["items"] == saved["messages"]
    assert not call("changesPage", session=saved["session"])["items"]
    assert proof.read_text(encoding="utf-8") == "validated 世界" and not late.exists()
    print("Restart: command records/output survived exactly; command effects stayed outside Changes; stopped process made no late file.")
call("shutdown")
