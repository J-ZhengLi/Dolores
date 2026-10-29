use super::*;
use config::load;
use std::fs;

fn profile(home: &std::path::Path) -> Profile {
    Profile {
        enabled: true,
        home: Some(home.into()),
        project: true,
        referenced_instructions: true,
    }
}
#[test]
fn layers_replace_servers_without_importing_model_auth_or_environment_values_into_reports() {
    let data = tempfile::tempdir().unwrap();
    let home = data.path().join("home");
    let root = data.path().join("repo");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(root.join(".codex")).unwrap();
    fs::create_dir(root.join(".git")).unwrap();
    fs::write(home.join("config.toml"),"model='do-not-import'\n[mcp_servers.fixture]\ncommand='old'\nargs=['old']\n[mcp_servers.other]\ncommand='other'\n[mcp_servers.other.env]\nTOKEN='PRIVATE_FIXTURE_VALUE'").unwrap();
    fs::write(
        root.join(".codex/config.toml"),
        "[mcp_servers.fixture]\ncommand='new'\nenabled=false",
    )
    .unwrap();
    let loaded = load(&profile(&home), Some(&root));
    assert_eq!(loaded.servers["fixture"].command.as_deref(), Some("new"));
    assert!(!loaded.servers["fixture"].enabled);
    assert!(loaded.servers["fixture"].args.is_empty());
    fs::write(
        root.join(".codex/config.toml"),
        "[mcp_servers.fixture]\nenabled=false",
    )
    .unwrap();
    let disabled = load(&profile(&home), Some(&root));
    assert!(!disabled.servers["fixture"].enabled);
    assert!(disabled.diagnostics.is_empty());
    assert!(!loaded
        .report()
        .to_string()
        .contains("PRIVATE_FIXTURE_VALUE"));
    assert!(!loaded.report().to_string().contains("do-not-import"));
    fs::write(
        root.join(".codex/config.toml"),
        "[mcp_servers.fixture]\ncommand='new'\nunknown='unsafe'",
    )
    .unwrap();
    let loaded = load(&profile(&home), Some(&root));
    assert!(!loaded.servers.contains_key("fixture"));
    assert!(loaded.diagnostics.iter().any(|v| v.contains("unknown")));
    fs::write(root.join(".codex/config.toml"), "broken = [").unwrap();
    assert!(load(&profile(&home), Some(&root))
        .diagnostics
        .iter()
        .any(|v| v.contains("Malformed TOML")));
    fs::remove_file(home.join("config.toml")).unwrap();
    fs::remove_file(root.join(".codex/config.toml")).unwrap();
    assert!(load(&profile(&home), Some(&root)).servers.is_empty());
    assert!(load(&Profile::default(), Some(&root)).home.is_none());
}
#[test]
fn instruction_scope_override_references_cycles_and_live_skill_identity() {
    let data = tempfile::tempdir().unwrap();
    let home = data.path().join("home");
    let root = data.path().join("repo");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(root.join(".agents/skills/unknown-fixture")).unwrap();
    fs::create_dir(root.join(".git")).unwrap();
    fs::write(home.join("AGENTS.md"), "GLOBAL\n@extra.md").unwrap();
    fs::write(home.join("extra.md"), "IMPORTED\n@AGENTS.md").unwrap();
    fs::write(root.join("AGENTS.md"), "IGNORED").unwrap();
    fs::write(root.join("AGENTS.override.md"), "PROJECT").unwrap();
    let skill = root.join(".agents/skills/unknown-fixture/SKILL.md");
    fs::write(
        &skill,
        "---\nname: unknown-fixture\ndescription: fixture guidance\n---\nFIRST",
    )
    .unwrap();
    let before = fs::read(home.join("AGENTS.md")).unwrap();
    let snapshot = load(&profile(&home), Some(&root));
    assert!(snapshot.instructions.contains("GLOBAL"));
    assert!(snapshot.instructions.contains("IMPORTED"));
    assert!(snapshot.instructions.contains("PROJECT"));
    assert!(!snapshot.instructions.contains("IGNORED"));
    let mut first = vec![];
    add_skills(&snapshot, &mut first);
    assert_eq!(first.len(), 1);
    fs::write(
        &skill,
        "---\nname: unknown-fixture\ndescription: fixture guidance\n---\nSECOND",
    )
    .unwrap();
    let mut second = vec![];
    add_skills(&load(&profile(&home), Some(&root)), &mut second);
    assert_ne!(first[0].revision, second[0].revision);
    fs::remove_file(skill).unwrap();
    let mut removed = vec![];
    add_skills(&load(&profile(&home), Some(&root)), &mut removed);
    assert!(removed.is_empty());
    assert_eq!(fs::read(home.join("AGENTS.md")).unwrap(), before);
    let mut p = profile(&home);
    p.project = false;
    p.referenced_instructions = false;
    let snapshot = load(&p, Some(&root));
    assert!(!snapshot.instructions.contains("IMPORTED"));
    assert!(!snapshot.instructions.contains("PROJECT"));
    assert!(snapshot.skill_collections.is_empty());
}
#[test]
fn inline_and_json_hooks_accept_regex_and_report_unsupported_handlers() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("config.toml"),"[[hooks.PreToolUse]]\nmatcher='^Bash$'\n[[hooks.PreToolUse.hooks]]\ntype='command'\ncommand='fixture'\ntimeout=12").unwrap();
    fs::write(dir.path().join("hooks.json"),r#"{"hooks":{"Stop":[{"hooks":[{"type":"command","command":"fixture"}]}],"Unsupported":[{"hooks":[]}],"PostToolUse":[{"hooks":[{"type":"prompt","command":"fixture"}]}]}}"#).unwrap();
    let loaded = load(&profile(dir.path()), None);
    assert_eq!(loaded.hooks.len(), 2);
    assert_eq!(loaded.hooks[0].timeout, 12);
    assert!(loaded.hooks[0].matcher.is_match("Bash"));
    assert_eq!(loaded.diagnostics.len(), 2);
}
#[test]
fn persisted_profile_reuse_reload_and_corrupt_recovery_leave_sources_untouched() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("source");
    let data = root.path().join("dolores");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&data).unwrap();
    fs::write(home.join("AGENTS.md"), "ORIGINAL_SOURCE").unwrap();
    let store = std::sync::Arc::new(
        dolores_store_sqlite::SqliteStore::open(&data.join("state.db")).unwrap(),
    );
    let mut engine = crate::Engine::new(
        store,
        std::sync::Arc::new(crate::connection::testing::MemoryCredentials::default()),
    )
    .unwrap();
    engine.codex_compatibility = Settings::open(&data).unwrap();
    engine.save_codex_settings(None, profile(&home)).unwrap();
    let restored = Settings::open(&data).unwrap();
    assert!(restored
        .snapshot(None)
        .unwrap()
        .instructions
        .contains("ORIGINAL_SOURCE"));
    fs::write(home.join("AGENTS.md"), "UPDATED_SOURCE").unwrap();
    assert!(restored
        .snapshot(None)
        .unwrap()
        .instructions
        .contains("UPDATED_SOURCE"));
    fs::remove_file(home.join("AGENTS.md")).unwrap();
    assert!(restored.snapshot(None).unwrap().instructions.is_empty());
    fs::write(data.join("codex-compatibility.json"), "{broken").unwrap();
    let broken = Settings::open(&data).unwrap();
    let snapshot = broken.snapshot(None).unwrap();
    assert!(snapshot.home.is_none());
    assert!(!snapshot.diagnostics.is_empty());
    assert_eq!(
        fs::read_to_string(data.join("codex-compatibility.json")).unwrap(),
        "{broken"
    );
    engine.codex_compatibility = broken;
    engine
        .save_codex_settings(None, Profile::default())
        .unwrap();
    assert!(engine.codex_snapshot(None).unwrap().diagnostics.is_empty());
}

#[tokio::test]
async fn generic_mcp_is_discovered_persists_for_the_task_and_respects_stop() {
    use dolores_core::ToolCall;
    let data = tempfile::tempdir().unwrap();
    let home = data.path().join("profile");
    let root = data.path().join("project");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&root).unwrap();
    let script = root.join("unknown-server.cjs");
    fs::write(&script,r#"
const readline=require('node:readline'),fs=require('node:fs');
fs.appendFileSync('starts.txt','start\n');
const tools=['echo','unused'].map(name=>({name,description:'Unrelated integration fixture',inputSchema:{type:'object',properties:{text:{type:'string'}},required:['text']}}));
tools.push({name:'large_metadata',description:'Metadata only '+ 'D'.repeat(14000),inputSchema:{type:'object'}});
let count=0;
readline.createInterface({input:process.stdin}).on('line',line=>{
 const p=JSON.parse(line); if(!p.id)return;
 if(p.method==='tools/call' && p.params.arguments.text==='hang')return;
 if(p.method==='tools/call')fs.appendFileSync('calls.txt','call\n');
 let result=p.method==='initialize'?{protocolVersion:'2025-11-25',capabilities:{tools:{}},serverInfo:{name:'unrelated-fixture',version:'1'},instructions:'Use this unrelated fixture to verify configured sessions.'}
 :p.method==='tools/list'?{tools}
 :{content:[{type:'text',text:JSON.stringify({text:p.params.arguments.text==='large'?'L'.repeat(24000):p.params.arguments.text,count:++count,marker:process.env.FIXTURE_MARKER,home:process.env.CODEX_HOME})}]};
 console.log(JSON.stringify({jsonrpc:'2.0',id:p.id,result}));
});
"#).unwrap();
    let source=format!("[mcp_servers.unrelated]\ncommand='node'\nargs=[{}]\ndisabled_tools=['unused']\n[mcp_servers.unrelated.env]\nFIXTURE_MARKER='PRIVATE_FIXTURE_MARKER'\n[mcp_servers.off]\ncommand='not-installed-disabled-fixture'\nenabled=false\n[mcp_servers.missing]\ncommand='not-installed-fixture'\nstartup_timeout_sec=1",serde_json::to_string(&script.to_string_lossy()).unwrap());
    fs::write(home.join("config.toml"), source).unwrap();
    let cancel = tokio_util::sync::CancellationToken::new();
    let snapshot = load(&profile(&home), Some(&root));
    let activated = super::mcp::activate(&snapshot, cancel.clone())
        .await
        .unwrap();
    assert_eq!(activated.tools.len(), 1);
    assert_eq!(activated.diagnostics.len(), 1);
    assert!(activated.guidance.contains("unrelated fixture"));
    assert!(!activated.guidance.contains("PRIVATE_FIXTURE_MARKER"));
    let gateway = activated.tools[0].clone();
    let list = gateway
        .prepare(&ToolCall {
            id: "discover".into(),
            name: "codex_mcp".into(),
            arguments: r#"{"action":"list","query":"Unrelated integration fixture"}"#.into(),
        })
        .unwrap();
    let catalog: serde_json::Value =
        serde_json::from_str(&gateway.invoke(&list, cancel.clone()).await.unwrap()).unwrap();
    assert_eq!(catalog["tools"].as_array().unwrap().len(), 1);
    let alias = catalog["tools"][0]["name"].as_str().unwrap();
    // A single larger schema must make cursor progress and remain readable,
    // rather than failing legacy inline-catalog limits or returning no tools.
    let discover = gateway
        .prepare(&ToolCall {
            id: "large-catalog".into(),
            name: "codex_mcp".into(),
            arguments: r#"{"action":"list","query":"Metadata only"}"#.into(),
        })
        .unwrap();
    let mut metadata: serde_json::Value =
        serde_json::from_str(&gateway.invoke(&discover, cancel.clone()).await.unwrap()).unwrap();
    assert_eq!(metadata["saved"], true);
    let metadata_id = metadata["resultId"].as_str().unwrap().to_owned();
    let mut text = metadata["text"].as_str().unwrap().to_owned();
    while let Some(offset) = metadata["nextOffset"].as_u64() {
        let read = gateway
            .prepare(&ToolCall {
                id: format!("schema-{offset}"),
                name: "codex_mcp".into(),
                arguments:
                    serde_json::json!({"action":"read","resultId":metadata_id,"offset":offset})
                        .to_string(),
            })
            .unwrap();
        metadata =
            serde_json::from_str(&gateway.invoke(&read, cancel.clone()).await.unwrap()).unwrap();
        text.push_str(metadata["text"].as_str().unwrap());
    }
    let metadata: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(metadata["tools"].as_array().unwrap().len(), 1);
    assert_eq!(metadata["nextCursor"], serde_json::Value::Null);
    assert_eq!(
        metadata["tools"][0]["description"].as_str().unwrap().len(),
        14014
    );
    assert!(!root.join("calls.txt").exists());
    for i in 1..=2 {
        let request = gateway
            .prepare(&ToolCall {
                id: format!("call-{i}"),
                name: "codex_mcp".into(),
                arguments:
                    serde_json::json!({"action":"call","tool":alias,"arguments":{"text":"hello"}})
                        .to_string(),
            })
            .unwrap();
        let result = gateway.invoke(&request, cancel.clone()).await.unwrap();
        assert!(result.contains(&format!(r#"\"count\":{i}"#)));
        assert!(!result.contains("PRIVATE_FIXTURE_MARKER"));
        assert!(result.contains("[redacted]"));
    }
    let large = gateway
        .prepare(&ToolCall {
            id: "large".into(),
            name: "codex_mcp".into(),
            arguments:
                serde_json::json!({"action":"call","tool":alias,"arguments":{"text":"large"}})
                    .to_string(),
        })
        .unwrap();
    let mut result: serde_json::Value =
        serde_json::from_str(&gateway.invoke(&large, cancel.clone()).await.unwrap()).unwrap();
    assert_eq!(result["saved"], true);
    assert_eq!(result["isError"], false);
    let result_id = result["resultId"].as_str().unwrap().to_owned();
    let mut combined = result["text"].as_str().unwrap().to_owned();
    while let Some(offset) = result["nextOffset"].as_u64() {
        let read = gateway
            .prepare(&ToolCall {
                id: format!("page-{offset}"),
                name: "codex_mcp".into(),
                arguments:
                    serde_json::json!({"action":"read","resultId":result_id,"offset":offset})
                        .to_string(),
            })
            .unwrap();
        result =
            serde_json::from_str(&gateway.invoke(&read, cancel.clone()).await.unwrap()).unwrap();
        combined.push_str(result["text"].as_str().unwrap());
    }
    assert!(combined.contains(&"L".repeat(24000)));
    assert!(!combined.contains("PRIVATE_FIXTURE_MARKER"));
    assert_eq!(
        fs::read_to_string(root.join("calls.txt"))
            .unwrap()
            .lines()
            .count(),
        3
    );
    assert_eq!(
        fs::read_to_string(root.join("starts.txt"))
            .unwrap()
            .lines()
            .count(),
        1
    );
    let reply = super::agent_tests::run(
        vec![gateway.clone()],
        vec![ToolCall {
            id: "agent-call".into(),
            name: "codex_mcp".into(),
            arguments:
                serde_json::json!({"action":"call","tool":alias,"arguments":{"text":"hello"}})
                    .to_string(),
        }],
    )
    .await;
    assert_eq!(reply.summary.tools[0].status, "completed");
    assert!(reply.summary.tools[0].content.contains("hello"));
    let request = gateway
        .prepare(&ToolCall {
            id: "stop".into(),
            name: "codex_mcp".into(),
            arguments:
                serde_json::json!({"action":"call","tool":alias,"arguments":{"text":"hang"}})
                    .to_string(),
        })
        .unwrap();
    let future = gateway.invoke(&request, cancel.clone());
    tokio::pin!(future);
    tokio::select! {_=&mut future=>panic!("Hanging MCP fixture ended early"),_=tokio::time::sleep(std::time::Duration::from_millis(50))=>{}}
    cancel.cancel();
    assert!(future.await.is_err());
    drop(activated);
    assert_eq!(
        fs::read_to_string(root.join("starts.txt"))
            .unwrap()
            .lines()
            .count(),
        1
    );
    fs::remove_file(home.join("config.toml")).unwrap();
    assert!(super::mcp::activate(
        &load(&profile(&home), Some(&root)),
        tokio_util::sync::CancellationToken::new()
    )
    .await
    .unwrap()
    .tools
    .is_empty());
}
