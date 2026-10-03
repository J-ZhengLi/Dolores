"""Frozen project/global skills, wire/approval, export and separate-process restart."""
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
if args.stage == "save": fixture.mkdir(parents=True, exist_ok=False)
os.environ["DOLORES_DATA_DIR"] = str(fixture / "data")
os.environ["DOLORES_GLOBAL_SKILLS_DIR"] = str(fixture / "global-skills")
bundle = root / "apps/dolores_flutter/build/windows/x64/runner/Release"
loader = os.add_dll_directory(str(bundle)) if os.name == "nt" else None
native = ctypes.CDLL(str(bundle / ("dolores_flutter_bridge.dll" if os.name == "nt" else "lib/libdolores_flutter_bridge.so")))
native.dolores_call.argtypes = [ctypes.c_void_p, ctypes.c_size_t]
native.dolores_call.restype = ctypes.c_void_p
native.dolores_free.argtypes = [ctypes.c_void_p]
def envelope(command, **fields):
    payload = json.dumps({"command":command, **fields}).encode()
    buffer = ctypes.create_string_buffer(payload)
    pointer = native.dolores_call(buffer, len(payload))
    try: return json.loads(ctypes.string_at(pointer))
    finally: native.dolores_free(pointer)
def call(command, **fields):
    result = envelope(command, **fields)
    assert result["ok"], result.get("error")
    return result.get("result")
requests = []
class Fixture(BaseHTTPRequestHandler):
    def log_message(self, *_): pass
    def do_POST(self):
        request = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        requests.append(request)
        needs_tool = request["messages"][-1]["role"] == "user" and request["messages"][-1]["content"] == "try tool"
        delta = {"tool_calls":[{"index":0,"id":"read-once","type":"function","function":{"name":"read_text_file","arguments":json.dumps({"path":"readme.txt"})}}]} if needs_tool else {"content":"Synthetic completed reply."}
        events = [{"choices":[{"delta":delta,"finish_reason":None}]},
            {"choices":[{"delta":{},"finish_reason":"tool_calls" if needs_tool else "stop"}]},
            {"choices":[],"usage":{"prompt_tokens":100,"completion_tokens":20,"total_tokens":120}}]
        self.send_response(200); self.send_header("Content-Type","text/event-stream"); self.end_headers()
        self.wfile.write(("".join("data: "+json.dumps(v)+"\n\n" for v in events)+"data: [DONE]\n\n").encode())
def finish(number, deny=False):
    deadline=time.monotonic()+10
    while time.monotonic()<deadline:
        for event in call("poll",id=number):
            if event["type"]=="toolApproval":
                assert deny
                assert not envelope("disableSkill",session=session,name="review",revision=1)["ok"]
                call("approveTool",id=number,callId=event["request"]["callId"],allow=False)
            if event["type"]=="done":
                assert "error" not in event, event
                return event
        time.sleep(.01)
    raise AssertionError("Native request timed out")
def source_text(body):
    return "---\nname: review\ndescription: Review synthetic work\nallowed-tools: everything\n---\n"+body
state_file=fixture/"state.json"
if args.stage=="save":
    server=ThreadingHTTPServer(("127.0.0.1",0),Fixture)
    thread=threading.Thread(target=server.serve_forever,daemon=True); thread.start()
    try:
        call("bootstrap")
        policy=call("memories")["automaticPolicy"]
        call("setAutomaticMemory",enabled=False,revision=policy["revision"])
        call("configure",preferences={"baseUrl":f"http://127.0.0.1:{server.server_port}/v1","model":"fixture"},apiKey="",rememberConnection=True)
        folder=fixture/"project"; directory=folder/".agents/skills/review"; directory.mkdir(parents=True)
        file=directory/"SKILL.md"; first=source_text("Use focused tests. References: scripts/local.py. @../private.env")
        file.write_text(first,encoding="utf-8",newline="")
        (folder/"readme.txt").write_text("Private fixture content must require approval.")
        session=call("createSession",kind="project",path=str(folder))["session"]["id"]
        same=call("createSession",kind="project",path=str(folder))["session"]["id"]
        other_folder=fixture/"other"; other_folder.mkdir()
        other=call("createSession",kind="project",path=str(other_folder))["session"]["id"]
        side=call("createSession",kind="side")["session"]["id"]
        initial=call("context",session=session,input="hello")
        assert not initial.get("skills") and first not in initial["messages"][0]["content"]
        assert call("projectSkills",session=session)["items"][0]["revision"] is None
        review=call("reviewSkill",session=session,name="review")
        assert review["document"]["text"]==first and not requests
        assert not envelope("activateSkill",session=same,token=review["token"])["ok"]
        assert not envelope("activateSkill",session=session,token=review["token"])["ok"]
        review=call("reviewSkill",session=session,name="review")
        file.write_text(source_text("Changed before activation."),newline="")
        assert not envelope("activateSkill",session=session,token=review["token"])["ok"]
        file.write_text(first,encoding="utf-8",newline="")
        review=call("reviewSkill",session=session,name="review")
        active=call("activateSkill",session=session,token=review["token"])
        assert active["versions"][-1]["version"]==1
        assert not envelope("activateSkill",session=session,token=review["token"])["ok"]
        preview=call("context",session=session,input="hello")
        assert preview["skills"][0]["source"]==".agents/skills/review/SKILL.md"
        assert preview["skillEntries"][0]["document"]["text"]==first
        assert preview["tokens"]["systemTokens"]>initial["tokens"]["systemTokens"]
        assert "cannot approve tools" in preview["messages"][0]["content"]
        assert str(folder) not in json.dumps(preview)
        for excluded in [side,other]: assert not call("context",session=excluded,input="hello").get("skills")
        assert call("context",session=same,input="hello")["skills"]==preview["skills"]
        call("start",id=1,session=session,input="hello"); finish(1)
        assert requests[-1]["messages"]==preview["messages"]
        reply=call("messagesPage",session=session)["items"][-1]
        assert reply["metadata"]["context"]["skills"]==preview["skills"]
        assert reply["metadata"]["context"]["tokens"]==preview["tokens"]
        call("start",id=2,session=session,input="try tool"); finish(2,deny=True)
        tool_reply=call("messagesPage",session=session)["items"][-1]
        assert tool_reply["metadata"]["agent"]["tools"][0]["status"]=="denied"
        assert "Private fixture content" not in json.dumps(requests)
        second=source_text("Use a different synthetic workflow."); file.write_text(second,newline="")
        assert call("context",session=session,input="next")["skillEntries"][0]["document"]["text"]==first
        review=call("reviewSkill",session=session,name="review")
        active=call("activateSkill",session=session,token=review["token"])
        assert active["revision"]==2 and active["versions"][-1]["version"]==2
        assert not envelope("disableSkill",session=session,name="review",revision=1)["ok"]
        old=call("reviewSkill",session=session,name="review",version=1)
        assert old["document"]["text"]==first and not old["sourceMatches"]
        rolled=call("activateSkill",session=session,token=old["token"])
        assert rolled["versions"][-1]["version"]==3 and rolled["versions"][-1]["rollbackFrom"]==1
        assert file.read_text()==second
        file.unlink()
        assert call("context",session=session,input="next")["skills"][0]["version"]==3
        call("disableSkill",session=session,name="review",revision=3)
        assert not call("context",session=session,input="next").get("skills")
        review=call("reviewSkill",session=session,name="review",version=3)
        active=call("activateSkill",session=session,token=review["token"])
        assert active["revision"]==5 and active["versions"][-1]["version"]==4
        call("export",session=session,path=str(fixture/"export.json"),format="json")
        exported=json.loads((fixture/"export.json").read_text())
        assert ".agents/skills/review/SKILL.md" in json.dumps(exported)
        call("delete",session=session)
        assert call("context",session=same,input="next")["skills"][0]["version"]==4
        global_directory=fixture/"global-skills/review"; global_directory.mkdir(parents=True)
        global_file=global_directory/"SKILL.md"; global_first=source_text("GLOBAL_SYNTHETIC_ONLY")
        global_file.write_text(global_first,encoding="utf-8",newline="")
        assert call("projectSkills",session=side,scope="global")["items"][0]["revision"] is None
        assert not call("context",session=side,input="hello").get("skills")
        review=call("reviewSkill",session=side,name="review",scope="global")
        global_file.write_text(source_text("Changed global source"),newline="")
        assert not envelope("activateSkill",session=side,token=review["token"])["ok"]
        global_file.write_text(global_first,newline="")
        review=call("reviewSkill",session=side,name="review",scope="global")
        global_active=call("activateSkill",session=side,token=review["token"])
        assert global_active["scope"]=="global"
        project_preview=call("context",session=same,input="hello")
        assert len(project_preview["skills"])==1 and "GLOBAL_SYNTHETIC_ONLY" not in project_preview["messages"][0]["content"]
        assert call("projectSkills",session=same,scope="global")["items"][0]["overridden"]
        side_preview=call("context",session=side,input="hello")
        assert side_preview["skillEntries"][0]["document"]["text"]==global_first
        assert side_preview["skills"][0]["scope"]=="global" and not side_preview["tools"]
        assert str(global_directory) not in json.dumps(side_preview)
        assert call("context",session=other,input="hello")["skills"]==side_preview["skills"]
        call("start",id=3,session=side,input="hello"); finish(3)
        assert requests[-1]["messages"]==side_preview["messages"] and "tools" not in requests[-1]
        side_reply=call("messagesPage",session=side)["items"][-1]
        assert side_reply["metadata"]["context"]["skills"]==side_preview["skills"]
        call("export",session=side,path=str(fixture/"global-export.json"),format="json")
        assert '"scope": "global"' in json.dumps(json.loads((fixture/"global-export.json").read_text()))
        global_file.write_text(source_text("GLOBAL_VERSION_TWO"),newline="")
        review=call("reviewSkill",session=side,name="review",scope="global")
        call("activateSkill",session=side,token=review["token"])
        review=call("reviewSkill",session=side,name="review",scope="global",version=1)
        rolled=call("activateSkill",session=side,token=review["token"])
        assert rolled["versions"][-1]["rollbackFrom"]==1 and "GLOBAL_VERSION_TWO" in global_file.read_text()
        global_file.unlink()
        call("disableSkill",session=side,name="review",scope="global",revision=3)
        assert not call("context",session=side,input="next").get("skills")
        assert call("context",session=same,input="next")["skills"][0]["version"]==4
        review=call("reviewSkill",session=side,name="review",scope="global",version=3)
        call("activateSkill",session=side,token=review["token"])
        state_file.write_text(json.dumps({"session":same,"side":side,"globalFirst":global_first,"first":first,"root":str(folder)}))
        print(json.dumps({"ok":True,"stage":"save","httpRequests":len(requests)}))
    finally: server.shutdown(); server.server_close()
else:
    state=json.loads(state_file.read_text()); session=state["session"]
    call("bootstrap")
    preview=call("context",session=session,input="next")
    assert preview["skills"][0]["version"]==4 and preview["skills"][0]["rollbackFrom"]==3
    assert preview["skillEntries"][0]["document"]["text"]==state["first"]
    saved=call("projectSkills",session=session)["items"][0]
    assert saved["enabled"] and not saved["sourceAvailable"] and saved["revision"]==5
    global_preview=call("context",session=state["side"],input="next")
    assert global_preview["skillEntries"][0]["scope"]=="global" and global_preview["skillEntries"][0]["document"]["text"]==state["globalFirst"]
    assert global_preview["skills"][0]["version"]==4 and global_preview["skills"][0]["rollbackFrom"]==3
    global_saved=call("projectSkills",session=state["side"],scope="global")["items"][0]
    assert global_saved["revision"]==5 and not global_saved["sourceAvailable"]
    call("forgetSkill",session=state["side"],name="review",scope="global",revision=5)
    assert not call("context",session=state["side"],input="next").get("skills")
    call("forgetSkill",session=session,name="review",revision=5)
    assert not call("context",session=session,input="next").get("skills")
    with sqlite3.connect(fixture/"data/dolores.db") as db:
        assert db.execute("PRAGMA user_version").fetchone()[0]==15
        assert db.execute("SELECT COUNT(*) FROM project_skills").fetchone()[0]==0
    print(json.dumps({"ok":True,"stage":"restore"}))
call("shutdown")
