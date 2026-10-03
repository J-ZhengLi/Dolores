"""Real native/SQLite skill export, import review and separate-process restart.

Uses fresh synthetic data under output/; no provider or credential is required.
Native Save dialog interaction is covered separately by manual acceptance.
"""
import argparse
import ctypes
import json
import os
from pathlib import Path
import sqlite3

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
    result = envelope(command, **fields)
    assert result["ok"], result.get("error")
    return result.get("result")


def snapshot():
    database = fixture / "data/dolores.db"
    with sqlite3.connect(database.as_uri() + "?mode=ro", uri=True) as db:
        tables = db.execute("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name").fetchall()
        return {name: sorted(repr(row) for row in db.execute('SELECT * FROM "' + name.replace('"', '""') + '"')) for (name,) in tables}


def export_checked(session, scope, name, version, destination, expected):
    destination.parent.mkdir(parents=True, exist_ok=False)
    before = snapshot()
    review = call("reviewSkill", session=session, scope=scope, name=name, version=version)
    result = call("exportSkill", session=session, token=review["token"], path=str(destination))
    assert result == {"name": name, "scope": scope, "version": version, "bytes": len(expected.encode())}
    assert destination.read_bytes() == expected.encode()
    assert snapshot() == before, "Export mutated local data"
    assert not envelope("exportSkill", session=session, token=review["token"], path=str(destination))["ok"]
    assert destination.read_bytes() == expected.encode()
    assert list(destination.parent.iterdir()) == [destination], "Unexpected resource or temporary file"
    return review


call("bootstrap")
state_file = fixture / "state.json"
if args.stage == "save":
    folder = fixture / "project"
    source = folder / ".agents/skills/review/SKILL.md"
    source.parent.mkdir(parents=True)
    first = "---\r\nname: review\r\ndescription: >-\r\n  Review synthetic\r\n  work.\r\nmetadata:\r\n  version: '1'\r\nallowed-tools: everything\r\n---\r\nCheck focused tests. 世界\r\nReferences: scripts/local.py\r\n"
    second = first.replace("Check focused tests.", "Check current tests.")
    source.write_bytes(first.encode())
    resources = source.parent / "scripts"
    resources.mkdir()
    (resources / "local.py").write_text("# Synthetic resource: must not be copied or executed\n")
    session = call("createSession", kind="project", path=str(folder))["session"]["id"]
    side = call("createSession", kind="side")["session"]["id"]
    review = call("reviewSkill", session=session, name="review")
    invalid = fixture / "unsaved/review/SKILL.md"
    assert not envelope("exportSkill", session=session, token=review["token"], path=str(invalid))["ok"]
    assert not invalid.exists()
    call("activateSkill", session=session, token=review["token"])
    source.write_bytes(second.encode())
    review = call("reviewSkill", session=session, name="review")
    call("activateSkill", session=session, token=review["token"])
    call("disableSkill", session=session, name="review", revision=2)
    source.unlink()
    destination = fixture / "portable/review/SKILL.md"
    review = export_checked(session, "project", "review", 1, destination, first)
    assert not envelope("exportSkill", session=side, token=review["token"], path=str(fixture / "wrong/review/SKILL.md"))["ok"]
    assert not call("context", session=session, input="hello").get("skills")
    # The exported standard document is discovered in another project, but waits for review.
    imported = fixture / "imported/.agents/skills/review/SKILL.md"
    imported.parent.mkdir(parents=True)
    imported.write_bytes(destination.read_bytes())
    other = call("createSession", kind="project", path=str(fixture / "imported"))["session"]["id"]
    catalog = call("projectSkills", session=other)
    assert catalog["items"][0]["revision"] is None
    assert not call("context", session=other, input="hello").get("skills")
    review = call("reviewSkill", session=other, name="review")
    assert review["document"]["text"] == first
    call("activateSkill", session=other, token=review["token"])
    assert call("context", session=other, input="hello")["skillEntries"][0]["document"]["text"] == first
    global_file = fixture / "global-skills/review/SKILL.md"
    global_file.parent.mkdir(parents=True)
    global_file.write_bytes(second.encode())
    review = call("reviewSkill", session=side, scope="global", name="review")
    call("activateSkill", session=side, token=review["token"])
    global_file.unlink()
    export_checked(side, "global", "review", 1, fixture / "global-portable/review/SKILL.md", second)
    assert call("context", session=side, input="hello")["skills"][0]["scope"] == "global"
    state_file.write_text(json.dumps({"session": session, "side": side, "first": first, "second": second}), encoding="utf-8")
else:
    state = json.loads(state_file.read_text(encoding="utf-8"))
    export_checked(state["session"], "project", "review", 1, fixture / "restart-project/review/SKILL.md", state["first"])
    export_checked(state["side"], "global", "review", 1, fixture / "restart-global/review/SKILL.md", state["second"])
    item = call("projectSkills", session=state["session"])["items"][0]
    assert not item["enabled"] and item["version"] == 2
    with sqlite3.connect(fixture / "data/dolores.db") as db:
        assert db.execute("PRAGMA user_version").fetchone()[0] == 15
        assert db.execute("SELECT COUNT(*) FROM messages").fetchone()[0] == 0
print(json.dumps({"ok": True, "stage": args.stage, "providerRequests": 0, "schema": 15}))
