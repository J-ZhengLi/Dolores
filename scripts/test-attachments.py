"""Release native attachment flow and bounded refusal/recovery, using synthetic data."""
import base64
import ctypes
import hashlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import sqlite3
import struct
import subprocess
import sys
import tempfile
import threading
import time
import zlib

root = Path(__file__).resolve().parents[1]
if len(sys.argv) == 1:
    with tempfile.TemporaryDirectory(prefix="dolores-attachments-") as directory:
        for phase in ("exercise", "restart"):
            subprocess.run([sys.executable, __file__, directory, phase], check=True, timeout=60)
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

def png():
    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", 2, 2, 8, 2, 0, 0, 0)) + chunk(b"IDAT", zlib.compress((b"\0" + bytes([0, 200, 0]) * 2) * 2)) + chunk(b"IEND", b"")

requests = []
reject_image = False
class Handler(BaseHTTPRequestHandler):
    def log_message(self, *_): pass
    def do_POST(self):
        payload = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        requests.append(payload)
        if reject_image:
            self.send_response(400); self.send_header("Content-Type", "application/json"); self.end_headers()
            self.wfile.write(json.dumps({"error":{"param":"image_url","message":"This model does not support image input. PRIVATE"}}).encode())
            return
        self.send_response(200); self.send_header("Content-Type", "text/event-stream"); self.end_headers()
        for value in [{"choices":[{"delta":{"content":"Saved attachment read."},"finish_reason":None}]}, {"choices":[{"delta":{},"finish_reason":"stop"}]}, {"choices":[],"usage":{"prompt_tokens":140,"completion_tokens":8,"total_tokens":148}}]:
            self.wfile.write(("data: " + json.dumps(value) + "\n\n").encode())
        self.wfile.write(b"data: [DONE]\n\n")

def run(identity, session, text):
    result = envelope("start", id=identity, session=session, input=text)
    if not result["ok"]: return result["error"]
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        for event in call("poll", id=identity):
            assert event["type"] != "toolApproval"
            if event["type"] == "done": return event.get("error")
        time.sleep(.02)
    call("cancel", id=identity)
    raise AssertionError("Bounded fixture timeout")

if sys.argv[2] == "restart":
    try:
        call("bootstrap")
        record = json.loads((directory / "restart.json").read_text())
        refs = call("draftAttachments", session=record["draft"])
        assert len(refs) == 4
        assert call("savedDraft", session=record["draft"]) == "Keep draft through restart"
        history = call("messagesPage", session=record["fork"])["items"]
        assert any(m.get("parts") for m in history)
        assert call("attachmentPreview", session=record["fork"], digest=record["textDigest"])["text"] == "Immutable 世界 snapshot."
        print("Attachment restart: draft and sent/forked references retained.")
    finally: call("shutdown")
    raise SystemExit(0)

server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
threading.Thread(target=server.serve_forever, daemon=True).start()
try:
    call("bootstrap")
    policy = call("memories")["automaticPolicy"]
    call("setAutomaticMemory", enabled=False, revision=policy["revision"])
    call("configure", preferences={"baseUrl":f"http://127.0.0.1:{server.server_port}/v1", "model":"fixture"}, apiKey="", rememberConnection=False)
    call("setRequestSettings", settings={"maxOutputTokens":128,"timeoutSeconds":30})
    session = call("createSession", kind="side")["session"]["id"]
    text = directory / "notes.txt"; text.write_text("Immutable 世界 snapshot.", encoding="utf-8")
    ref = call("attachFile", session=session, path=str(text))[0]
    text.write_text("Changed source", encoding="utf-8")
    assert call("attachmentPreview", session=session, digest=ref["digest"])["text"] == "Immutable 世界 snapshot."
    assert run(1, session, "Read the attachment") is None
    assert isinstance(requests[-1]["messages"][-1]["content"], str)
    assert "Immutable 世界 snapshot." in requests[-1]["messages"][-1]["content"]
    assert "Changed source" not in requests[-1]["messages"][-1]["content"]
    assert str(directory) not in json.dumps(requests[-1])
    assert call("draftAttachments", session=session) == []
    image = directory / "green.png"; image.write_bytes(png())
    image_ref = call("attachFile", session=session, path=str(image))[0]
    assert call("harnessInventory", session=session)["modelCapabilities"]["input"] == ["text"]
    before = len(requests)
    assert "disabled" in run(2, session, "Inspect image")
    assert len(requests) == before
    assert call("savedDraft", session=session) == "Inspect image"
    assert call("draftAttachments", session=session) == [image_ref]
    preview = call("context", session=session, input="Inspect image")
    assert preview["tokens"]["imageTokens"] == 4096
    call("setImageModels", models=["fixture"])
    assert call("harnessInventory", session=session)["modelCapabilities"]["input"] == ["text", "image"]
    reject_image = True
    error = run(3, session, "Inspect image")
    assert "Choose a capable model" in error and "PRIVATE" not in error
    assert len(requests) == before + 1
    assert call("draftAttachments", session=session) == [image_ref]
    reject_image = False
    assert run(4, session, "Inspect image") is None
    image_wire = requests[-1]["messages"][-1]["content"][1]["image_url"]
    assert image_wire["detail"] == "low"
    assert base64.b64decode(image_wire["url"].split(",", 1)[1]) == image.read_bytes()
    history = call("messagesPage", session=session)["items"]
    fork = call("forkSession", session=session, through=history[-1]["id"])["session"]["id"]
    other = call("createSession", kind="side")["session"]["id"]
    assert not envelope("attachmentPreview", session=other, digest=ref["digest"])["ok"]
    exported = call("exportAttachments", session=session, directory=str(directory))
    assert exported["count"] == 2
    folder = Path(exported["folder"])
    assert (folder / (ref["digest"] + ".txt")).read_text(encoding="utf-8") == "Immutable 世界 snapshot."
    assert (folder / (image_ref["digest"] + ".png")).read_bytes() == image.read_bytes()
    limited = call("createSession", kind="side")["session"]["id"]
    for n in range(4):
        path = directory / f"bounded-{n}.txt"; path.write_text(f"snapshot {n}")
        call("attachFile", session=limited, path=str(path))
    path = directory / "fifth.txt"; path.write_text("unused")
    assert not envelope("attachFile", session=limited, path=str(path))["ok"]
    assert len(call("draftAttachments", session=limited)) == 4
    call("saveDraft", session=limited, text="Keep draft through restart")
    path.write_bytes(b"x" * 65537)
    assert "64 KiB" in envelope("attachFile", session=limited, path=str(path))["error"]
    orphan = directory / "orphan.txt"; orphan.write_text("orphan")
    orphan_ref = call("attachFile", session=other, path=str(orphan))[0]
    call("removeAttachment", session=other, digest=orphan_ref["digest"])
    assert call("cleanupAttachments")["removed"] == 1
    call("delete", session=session)
    assert call("attachmentPreview", session=fork, digest=ref["digest"])["text"] == "Immutable 世界 snapshot."
    assert call("cleanupAttachments")["removed"] == 0
    with sqlite3.connect(directory / "data/dolores.db") as db:
        db.execute("UPDATE attachment_assets SET data=? WHERE digest=?", (b"tampered", image_ref["digest"]))
    before = len(requests)
    assert "changed" in run(5, fork, "Use the saved image").lower()
    assert len(requests) == before
    assert call("savedDraft", session=fork) == "Use the saved image"
    call("attachFile", session=fork, path=str(image))
    assert run(6, fork, "Inspect the restored image") is None
    with sqlite3.connect(directory / "data/dolores.db") as db:
        assert db.execute("SELECT data FROM attachment_assets WHERE digest=?", (image_ref["digest"],)).fetchone()[0] == image.read_bytes()
        db.executemany("INSERT INTO messages(session_id,role,content) VALUES(?,?,?)", [(other, "user" if n % 2 == 0 else "assistant", "synthetic") for n in range(242)])
    folders_before = list(directory.glob("dolores-attachments-*"))
    assert "240 messages" in envelope("exportAttachments", session=other, directory=str(directory))["error"]
    assert list(directory.glob("dolores-attachments-*")) == folders_before
    (directory / "restart.json").write_text(json.dumps({"draft":limited,"fork":fork,"textDigest":ref["digest"]}))
    print("Attachment flow: immutable text, explicit image wire, rejection recovery, scope, caps, export, tamper refusal and cleanup passed.")
finally:
    call("shutdown"); server.shutdown()
