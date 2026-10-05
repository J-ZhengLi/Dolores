use super::{adaptation::reflect, experience_tests::Fixture, knowledge};
use dolores_core::*;
use dolores_store_sqlite::SqliteStore;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

pub(super) fn fixture() -> (tempfile::TempDir, Arc<SqliteStore>, String) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir
        .path()
        .canonicalize()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let store = Arc::new(SqliteStore::open(&dir.path().join("data.db")).unwrap());
    store
        .create_workspace_session(
            "task",
            &SessionWorkspace {
                kind: WorkspaceKind::Project,
                root: Some(root.clone()),
            },
        )
        .unwrap();
    let manifest = r#"{"scripts":{"check":"node verify.cjs"}}"#;
    std::fs::write(dir.path().join("package.json"), manifest).unwrap();
    let skill = store
        .activate_project_skill(
            &root,
            &check_workflow("node obsolete.cjs").unwrap(),
            None,
            None,
        )
        .unwrap();
    let mut context = ContextSummary::from_messages(&[], Some(0));
    context.skills.push(SkillSource {
        scope: SkillScope::Project,
        name: skill.name.clone(),
        source: "fixture".into(),
        version: 1,
        reviewed_at: 0,
        text_bytes: skill.current().document.text.len(),
        rollback_from: None,
    });
    let rec = |name: &str, target: &str, content: &str, command: Option<CommandSpec>| ToolRecord {
        call_id: name.into(),
        name: name.into(),
        target: target.into(),
        status: if command.is_some() { "failed" } else { "read" }.into(),
        content: content.into(),
        command,
        query: None,
        diff: None,
        mcp: None,
    };
    let metadata = TurnMetadata {
        paused: None,
        model: "fixture".into(),
        usage: None,
        context,
        request_settings: None,
        agent: Some(AgentSummary {
            model_calls: 2,
            usage_by_call: vec![None, None],
            tools: vec![
                rec("read_text_file", "package.json", manifest, None),
                rec(
                    "run_command",
                    "node",
                    r#"{"reason":"completed","exitCode":1,"truncated":false}"#,
                    Some(CommandSpec {
                        program: "node".into(),
                        args: vec!["obsolete.cjs".into()],
                    }),
                ),
            ],
            steps: vec![],
        }),
    };
    store
        .commit_turn_metadata(
            "task",
            "Enable config.json using project-check",
            "Old check failed",
            &metadata,
        )
        .unwrap();
    let mut k = store.knowledge(&root).unwrap();
    k.learning = true;
    store.save_knowledge(&root, 0, &k).unwrap();
    knowledge::learn(store.as_ref(), "task").unwrap();
    let mut s = store.adaptation(&root).unwrap();
    s.enabled = true;
    s.automatic = true;
    s.policy_revision = 1;
    store.save_adaptation(&root, 0, &s).unwrap();
    (dir, store, root)
}

#[tokio::test]
async fn targeted_repair_is_activated_once_with_baseline_retained() {
    let (_dir, store, root) = fixture();
    let (tx, _rx) = mpsc::channel(8);
    let provider = Arc::new(Fixture { mode: "good" });
    assert!(reflect(
        store.clone(),
        Some(provider.clone()),
        "task",
        "fixture",
        CancellationToken::new(),
        &tx,
        1
    )
    .await
    .unwrap()
    .is_some());
    let state = store.adaptation(&root).unwrap();
    assert_eq!(state.events[0].status, "active");
    let skill = store.project_skills(&root).unwrap().remove(0);
    assert_eq!(
        workflow_command(&skill.current().document).as_deref(),
        Some("node verify.cjs")
    );
    assert_eq!(
        workflow_command(
            &state.events[0]
                .baseline
                .as_ref()
                .unwrap()
                .current()
                .document
        )
        .as_deref(),
        Some("node obsolete.cjs")
    );
    assert!(store.global_skills().unwrap().is_empty());
    assert!(reflect(
        store.clone(),
        Some(provider),
        "task",
        "fixture",
        CancellationToken::new(),
        &tx,
        2
    )
    .await
    .unwrap()
    .is_none());
    assert_eq!(store.adaptation(&root).unwrap(), state);
}
#[tokio::test]
async fn tamper_limits_and_false_claims_cannot_activate() {
    for mode in ["tamper", "limit", "claim"] {
        let (_dir, store, root) = fixture();
        let (tx, _rx) = mpsc::channel(8);
        reflect(
            store.clone(),
            Some(Arc::new(Fixture { mode })),
            "task",
            "fixture",
            CancellationToken::new(),
            &tx,
            1,
        )
        .await
        .unwrap();
        let state = store.adaptation(&root).unwrap();
        assert_eq!(state.events[0].status, "inconclusive");
        assert_eq!(store.project_skills(&root).unwrap()[0].revision, 1);
        assert!(store
            .activate_adaptation(&root, state.revision, &state.events[0].id)
            .is_err());
    }
}

#[tokio::test]
async fn stale_knowledge_manual_edits_and_paused_policy_keep_baseline() {
    for mode in ["knowledge", "manual", "paused"] {
        let (_dir, store, root) = fixture();
        let mut state = store.adaptation(&root).unwrap();
        state.automatic = false;
        store
            .save_adaptation(&root, state.revision, &state)
            .unwrap();
        let (tx, _rx) = mpsc::channel(8);
        reflect(
            store.clone(),
            Some(Arc::new(Fixture { mode: "good" })),
            "task",
            "fixture",
            CancellationToken::new(),
            &tx,
            1,
        )
        .await
        .unwrap();
        state = store.adaptation(&root).unwrap();
        assert_eq!(state.events[0].status, "review");
        match mode {
            "knowledge" => {
                let k = store.knowledge(&root).unwrap();
                store.save_knowledge(&root, k.revision, &k).unwrap();
            }
            "manual" => {
                store
                    .activate_project_skill(
                        &root,
                        &check_workflow("node manual.cjs").unwrap(),
                        Some(1),
                        None,
                    )
                    .unwrap();
            }
            _ => {
                state.paused = true;
                state = store
                    .save_adaptation(&root, state.revision, &state)
                    .unwrap();
            }
        }
        let before = store.project_skills(&root).unwrap();
        assert!(store
            .activate_adaptation(&root, state.revision, &state.events[0].id)
            .is_err());
        assert_eq!(store.project_skills(&root).unwrap(), before);
    }
}
