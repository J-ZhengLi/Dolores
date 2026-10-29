use dolores_core::{SessionStore, ToolCall};
use serde_json::{json, Value};
use std::{
    fs,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

fn request(engine: &crate::Engine, value: Value) -> Value {
    engine.call(serde_json::from_value(value).unwrap()).unwrap()
}
#[test]
fn one_human_start_runs_codex_setup_through_the_normal_engine_and_saves_evidence() {
    let folder = tempfile::tempdir().unwrap();
    let root = crate::workspace::canonical_folder(folder.path()).unwrap();
    let (_, home) = super::hook_tests::fixture(folder.path());
    fs::write(home.join("AGENTS.md"), "GENERIC_SETUP_GUIDANCE").unwrap();
    let script = folder.path().join("another-package.cjs");
    fs::write(&script,r#"
const fs=require('node:fs'),readline=require('node:readline');
fs.appendFileSync('server-starts.txt','start\n');
readline.createInterface({input:process.stdin}).on('line',line=>{
const p=JSON.parse(line);if(!p.id)return;
let result=p.method==='initialize'?{protocolVersion:'2025-11-25',capabilities:{tools:{}},serverInfo:{name:'another-package',version:'1'}}:
p.method==='tools/list'?{tools:[{name:'echo',description:'Generic fixture',inputSchema:{type:'object',properties:{text:{type:'string'}}}}]}:
{content:[{type:'text',text:'hello from another package'}]};
console.log(JSON.stringify({jsonrpc:'2.0',id:p.id,result}));
});
"#).unwrap();
    fs::write(
        home.join("config.toml"),
        format!(
            "[mcp_servers.another]\ncommand='node'\nargs=[{}]",
            serde_json::to_string(&script.to_string_lossy()).unwrap()
        ),
    )
    .unwrap();
    let original = fs::read(home.join("config.toml")).unwrap();
    let store =
        Arc::new(dolores_store_sqlite::SqliteStore::open(&folder.path().join("state.db")).unwrap());
    store
        .create_workspace_session(
            "chat",
            &dolores_core::SessionWorkspace {
                kind: dolores_core::WorkspaceKind::Project,
                root: Some(root),
            },
        )
        .unwrap();
    store
        .save_draft("chat", "Verify the generic integration")
        .unwrap();
    let mut engine = crate::Engine::new(
        store.clone(),
        Arc::new(crate::connection::testing::MemoryCredentials::default()),
    )
    .unwrap();
    engine.workspace_directory = Some(folder.path().join("workspaces"));
    request(
        &engine,
        json!({"command":"configure","preferences":{"baseUrl":"http://127.0.0.1:9/v1","model":"fixture"},"apiKey":"","rememberConnection":false}),
    );
    let policy = store.automatic_memory_policy().unwrap();
    request(
        &engine,
        json!({"command":"setAutomaticMemory","enabled":false,"revision":policy.revision}),
    );
    request(
        &engine,
        json!({"command":"saveCodexSettings","session":"chat","profile":{"enabled":true,"home":home,"project":true,"referencedInstructions":true}}),
    );
    let calls = vec![
        ToolCall {
            id: "catalog".into(),
            name: "codex_mcp".into(),
            arguments: r#"{"action":"list"}"#.into(),
        },
        ToolCall {
            id: "rewrite".into(),
            name: "run_command".into(),
            arguments: r#"{"command":"echo ORIGINAL"}"#.into(),
        },
        ToolCall {
            id: "external".into(),
            name: "codex_mcp".into(),
            arguments:
                r#"{"action":"call","tool":"mcp__another__echo","arguments":{"text":"hello"}}"#
                    .into(),
        },
    ];
    engine.connection.lock().unwrap().provider = Some(Arc::new(super::agent_tests::Model(
        Mutex::new(calls.into()),
    )));
    request(
        &engine,
        json!({"command":"start","id":1,"session":"chat","input":"Verify the generic integration"}),
    );
    // A newer composer draft belongs to the user while the task is running.
    store
        .save_draft("chat", "Unrelated composer draft")
        .unwrap();
    let end = Instant::now() + Duration::from_secs(20);
    let mut done = false;
    while Instant::now() < end {
        for event in request(&engine, json!({"command":"poll","id":1}))
            .as_array()
            .unwrap()
        {
            if event["type"] == "toolApproval" {
                request(
                    &engine,
                    json!({"command":"approveTool","id":1,"callId":event["request"]["callId"],"allow":true}),
                );
            }
            if event["type"] == "done" {
                assert!(event.get("error").is_none(), "{event}");
                done = true;
            }
        }
        if done {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(done, "Native engine fixture timed out.");
    let messages = store.messages_page("chat", None, false, 10).unwrap().items;
    let agent = messages
        .last()
        .unwrap()
        .metadata
        .as_ref()
        .unwrap()
        .agent
        .as_ref()
        .unwrap();
    assert_eq!(agent.tools.len(), 3);
    assert!(
        agent.tools.iter().all(|t| t.status == "completed"),
        "{:?}",
        agent.tools
    );
    assert!(agent.tools[1].content.contains("PRE_CONTEXT"));
    assert!(agent.tools[2].content.contains("hello"));
    assert_eq!(
        fs::read_to_string(folder.path().join("server-starts.txt"))
            .unwrap()
            .lines()
            .count(),
        1
    );
    assert_eq!(fs::read(home.join("config.toml")).unwrap(), original);
    assert_eq!(
        store.saved_draft("chat").unwrap(),
        "Unrelated composer draft"
    );
    let run = store.runs("chat").unwrap().remove(0);
    let events = store.run_events("chat", &run.id).unwrap();
    assert_eq!(events.iter().filter(|e| e.kind == "toolResult").count(), 3);
    assert!(events.iter().any(|e| e.kind == "codexHookInput"));
    request(&engine, json!({"command":"shutdown"}));
}
