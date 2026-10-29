use super::*;
use dolores_core::ToolCall;
use serde_json::json;
use std::{fs, sync::Arc};
use tokio_util::sync::CancellationToken;

pub(super) fn fixture(root: &std::path::Path) -> (config::Snapshot, std::path::PathBuf) {
    let home = root.join("profile");
    fs::create_dir_all(&home).unwrap();
    let script = root.join("unknown-hook.cjs");
    let replacement = if cfg!(windows) {
        "echo hello & echo once>once.txt"
    } else {
        "printf hello; printf once > once.txt"
    };
    fs::write(&script,format!(r#"
const fs=require('node:fs');let body='';
process.stdin.on('data',p=>body+=p);
process.stdin.on('end',()=>{{
 const p=JSON.parse(body);
 fs.appendFileSync('events.jsonl',JSON.stringify(p)+'\n');
 const command=p.tool_input?.command||'';
 if(command.includes('DENY')){{console.log(JSON.stringify({{hookSpecificOutput:{{hookEventName:p.hook_event_name,permissionDecision:'deny',permissionDecisionReason:'fixture denial'}}}}));return;}}
 if(command.includes('HANG')){{setTimeout(()=>{{}},60000);return;}}
 if(p.hook_event_name==='PreToolUse'){{console.log(JSON.stringify({{hookSpecificOutput:{{hookEventName:'PreToolUse',permissionDecision:'allow',updatedInput:{{command:{replacement}}},additionalContext:'PRE_CONTEXT'}}}}));return;}}
 if(p.hook_event_name==='PostToolUse'){{console.log(JSON.stringify({{hookSpecificOutput:{{hookEventName:'PostToolUse',additionalContext:'POST_CONTEXT'}}}}));return;}}
 console.log(JSON.stringify({{hookSpecificOutput:{{hookEventName:p.hook_event_name,additionalContext:'SESSION_CONTEXT'}}}}));
}});
"#,replacement=serde_json::to_string(replacement).unwrap())).unwrap();
    let command = format!("node \"{}\"", script.display());
    let handler = json!({"type":"command","command":command,"timeout":5});
    fs::write(
        home.join("hooks.json"),
        json!({"hooks":{
            "PreToolUse":[{"matcher":"^Bash$","hooks":[handler.clone()]}],
            "PostToolUse":[{"matcher":"^Bash$","hooks":[handler.clone()]}],
            "SessionStart":[{"matcher":"startup","hooks":[handler.clone()]}],
            "UserPromptSubmit":[{"matcher":"IGNORED","hooks":[handler.clone()]}],
            "Stop":[{"hooks":[handler]}]
        }})
        .to_string(),
    )
    .unwrap();
    let profile = Profile {
        enabled: true,
        home: Some(home.clone()),
        project: false,
        referenced_instructions: false,
    };
    (config::load(&profile, Some(root)), home)
}
fn runtime(snapshot: &Snapshot) -> Arc<hooks::Runtime> {
    let (output, _receiver) = tokio::sync::mpsc::channel(32);
    hooks::Runtime::new(
        snapshot,
        hooks::Identity {
            session: "fixture-session".into(),
            turn: "fixture-turn".into(),
            model: "fixture-model".into(),
            permission: "default".into(),
        },
        None,
        output,
        1,
    )
    .unwrap()
    .unwrap()
}
#[tokio::test]
async fn codex_rewrite_is_applied_before_command_review_and_action_runs_once() {
    let root = tempfile::tempdir().unwrap();
    let (snapshot, home) = fixture(root.path());
    let rt = runtime(&snapshot);
    let start = rt
        .fire(
            "SessionStart",
            "startup",
            json!({"source":"startup"}),
            CancellationToken::new(),
        )
        .await
        .unwrap();
    assert!(
        start.context.contains("SESSION_CONTEXT"),
        "{}",
        start.context
    );
    let prompt = rt
        .fire(
            "UserPromptSubmit",
            "",
            json!({"prompt":"Public fixture"}),
            CancellationToken::new(),
        )
        .await
        .unwrap();
    assert!(prompt.context.contains("SESSION_CONTEXT"));
    let tools = rt.wrap(vec![Arc::new(
        dolores_tools_command::RunCommand::new(root.path()).unwrap(),
    )]);
    let reply = super::agent_tests::run(
        tools,
        vec![ToolCall {
            id: "rewrite".into(),
            name: "run_command".into(),
            arguments: r#"{"command":"echo ORIGINAL"}"#.into(),
        }],
    )
    .await;
    assert_eq!(reply.pause, None);
    assert_eq!(reply.summary.tools.len(), 1);
    assert_eq!(reply.summary.tools[0].status, "completed");
    let receipt = &reply.summary.tools[0];
    assert!(receipt.content.contains("PRE_CONTEXT"));
    assert!(receipt.content.contains("POST_CONTEXT"));
    let command = receipt.command.as_ref().unwrap();
    assert!(command.args.last().unwrap().contains("hello"));
    assert!(!command.args.last().unwrap().contains("ORIGINAL"));
    assert_eq!(
        fs::read_to_string(root.path().join("once.txt"))
            .unwrap()
            .trim(),
        "once"
    );
    rt.fire(
        "Stop",
        "",
        json!({"stop_hook_active":false,"last_assistant_message":reply.answer}),
        CancellationToken::new(),
    )
    .await
    .unwrap();
    let events = fs::read_to_string(root.path().join("events.jsonl"))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(events.len(), 5);
    assert_eq!(events[2]["tool_name"], "Bash");
    assert_eq!(events[2]["tool_input"]["command"], "echo ORIGINAL");
    assert!(events[3]["tool_input"]["command"]
        .as_str()
        .unwrap()
        .contains("hello"));
    assert_eq!(events[0]["session_id"], "fixture-session");
    assert!(home.join("hooks.json").exists());
}
#[tokio::test]
async fn codex_deny_and_stop_during_hook_never_prepare_or_run_the_command() {
    let root = tempfile::tempdir().unwrap();
    let (snapshot, _) = fixture(root.path());
    let rt = runtime(&snapshot);
    let tools = rt.wrap(vec![Arc::new(
        dolores_tools_command::RunCommand::new(root.path()).unwrap(),
    )]);
    let denied = tools[0]
        .clone()
        .prepare_for_run(
            &ToolCall {
                id: "deny".into(),
                name: "run_command".into(),
                arguments: r#"{"command":"echo DENY"}"#.into(),
            },
            CancellationToken::new(),
        )
        .await
        .unwrap_err();
    assert!(denied.contains("fixture denial"));
    assert!(!root.path().join("once.txt").exists());
    let cancel = CancellationToken::new();
    let call = ToolCall {
        id: "hang".into(),
        name: "run_command".into(),
        arguments: r#"{"command":"echo HANG"}"#.into(),
    };
    let pending = tools[0].prepare_for_run(&call, cancel.clone());
    tokio::pin!(pending);
    tokio::select! {_=&mut pending=>panic!("Hook fixture did not wait"),_=tokio::time::sleep(std::time::Duration::from_millis(100))=>{}}
    cancel.cancel();
    assert!(pending.await.is_err());
    assert!(!root.path().join("once.txt").exists());
}

struct DenyEffective(std::sync::atomic::AtomicBool);
#[async_trait::async_trait]
impl dolores_core::ToolApproval for DenyEffective {
    async fn authorize(
        &self,
        r: &dolores_core::ToolRequest,
        _: CancellationToken,
    ) -> Result<bool, String> {
        assert!(r
            .command
            .as_ref()
            .unwrap()
            .invocation
            .args
            .last()
            .unwrap()
            .contains("hello"));
        self.0.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(false)
    }
}
#[tokio::test]
async fn rewritten_command_still_requires_normal_effective_input_approval() {
    let root = tempfile::tempdir().unwrap();
    let (snapshot, _) = fixture(root.path());
    let rt = runtime(&snapshot);
    let tools = rt.wrap(vec![Arc::new(
        dolores_tools_command::RunCommand::new(root.path()).unwrap(),
    )]);
    let model = super::agent_tests::Model(std::sync::Mutex::new(
        vec![ToolCall {
            id: "review".into(),
            name: "run_command".into(),
            arguments: r#"{"command":"echo ORIGINAL"}"#.into(),
        }]
        .into(),
    ));
    let approval = DenyEffective(false.into());
    let (output, mut receiver) = tokio::sync::mpsc::channel(32);
    let drain = tokio::spawn(async move { while receiver.recv().await.is_some() {} });
    let reply = dolores_core::run_agent(
        &model,
        dolores_core::preview_context(vec![], "Public fixture").unwrap(),
        &tools,
        &approval,
        output,
        CancellationToken::new(),
    )
    .await
    .unwrap();
    drain.await.unwrap();
    assert!(approval.0.load(std::sync::atomic::Ordering::SeqCst));
    assert_eq!(reply.summary.tools[0].status, "denied");
    assert!(!root.path().join("once.txt").exists());
}
