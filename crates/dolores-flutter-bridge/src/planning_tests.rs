use super::*;
use dolores_core::{goal::*, planning::*};
fn engine() -> (Engine, Arc<SqliteStore>, tempfile::TempDir, String) {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(SqliteStore::open(&dir.path().join("data.db")).unwrap());
    let engine = Engine::new(
        store.clone(),
        Arc::new(connection::testing::MemoryCredentials::default()),
    )
    .unwrap();
    let project = dir.path().join("project");
    std::fs::create_dir(&project).unwrap();
    std::fs::write(project.join("public.txt"), "SELECTED_PUBLIC_CONTEXT 世界").unwrap();
    let s = engine
        .create_working_session(dolores_core::WorkspaceKind::Project, Some(project))
        .unwrap()["session"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    (engine, store, dir, s)
}
#[test]
fn reference_preview_select_race_snapshot_queue_export_and_scope_are_explicit() {
    let (engine, store, dir, s) = engine();
    let source = planning::Source::File {
        path: "public.txt".into(),
    };
    let preview = engine.reference_preview(&s, &source).unwrap();
    std::fs::write(dir.path().join("project/public.txt"), "CHANGED").unwrap();
    assert!(engine
        .edit_planning(
            &s,
            0,
            planning::Edit::Add {
                source: source.clone(),
                expected: preview.revision
            }
        )
        .err()
        .unwrap()
        .contains("changed"));
    let preview = engine.reference_preview(&s, &source).unwrap();
    engine
        .edit_planning(
            &s,
            0,
            planning::Edit::Add {
                source: source.clone(),
                expected: preview.revision,
            },
        )
        .unwrap();
    std::fs::remove_file(dir.path().join("project/public.txt")).unwrap();
    assert!(engine.reference_preview(&s, &source).is_err());
    store.save_draft(&s, "Preserved instruction").unwrap();
    store
        .edit_task_inputs(
            &s,
            &dolores_core::harness_task::InputEdit::Enqueue {
                id: "queued-ref".into(),
                text: "Preserved instruction".into(),
                mode: dolores_core::harness_task::InputMode::Queue,
            },
        )
        .unwrap();
    engine
        .edit_planning(
            &s,
            1,
            planning::Edit::Remove {
                digest: store.planning(&s).unwrap().references[0].revision.clone(),
            },
        )
        .unwrap();
    let queued = store.task_inputs(&s).unwrap().entries.remove(0);
    assert_eq!(queued.planning.references[0].text, "CHANGED");
    let context = prepare(
        preview_context(vec![], "Original instruction").unwrap(),
        &queued.planning,
    )
    .unwrap();
    assert!(context.last().unwrap().content.contains("CHANGED"));
    assert!(context
        .last()
        .unwrap()
        .content
        .contains("untrusted source data"));
    assert!(engine
        .reference_preview(
            &s,
            &planning::Source::File {
                path: "../data.db".into()
            }
        )
        .is_err());
    store.create("unrelated").unwrap();
    assert!(engine
        .reference_preview(
            &s,
            &planning::Source::Conversation {
                session: "unrelated".into()
            }
        )
        .is_err());
    let folder = engine
        .reference_preview(&s, &planning::Source::Folder { path: ".".into() })
        .unwrap();
    assert!(folder.omissions.contains("subfolders omitted"));
    let mut state = store.planning(&s).unwrap();
    state.references = vec![
        ContextReference {
            kind: "file".into(),
            source: "public".into(),
            revision: "f".repeat(64),
            text: "x".repeat(16384),
            omissions: String::new()
        };
        2
    ];
    assert!(store.save_planning(&s, state.revision, &state).is_err());
    let mut export = vec![];
    store
        .export_conversation(&s, dolores_core::ExportFormat::Json, &mut export)
        .unwrap();
    let export: Value = serde_json::from_slice(&export).unwrap();
    assert!(export["instructions"].to_string().contains("CHANGED"));
    assert!(export.get("planning").is_some());
}
#[test]
fn accepted_revision_binds_goal_and_reopen_has_no_execution() {
    let (engine, store, dir, s) = engine();
    store
        .edit_goal(
            &s,
            &GoalEdit::Set {
                revision: None,
                objective: "Create report".into(),
                constraints: "Preserve files".into(),
                criteria: vec!["Read back report".into()],
            },
            Default::default(),
        )
        .unwrap();
    engine
        .edit_planning(
            &s,
            0,
            planning::Edit::Draft {
                text: "Inspect then create and read back report".into(),
                decisions: "Preserve unrelated files".into(),
            },
        )
        .unwrap();
    engine.edit_planning(&s, 1, planning::Edit::Accept).unwrap();
    let state = store.planning(&s).unwrap();
    assert!(state.planning);
    assert_eq!(
        state.accepted.as_ref().unwrap().criteria,
        vec!["Read back report"]
    );
    assert!(planning::check_goal(store.as_ref(), &s, &state).is_ok());
    assert!(engine
        .call(Command::StartPlan {
            session: s.clone(),
            revision: 1,
            id: 99
        })
        .err()
        .unwrap()
        .contains("changed"));
    assert!(!engine.active.lock().unwrap().is_some());
    let g = store.goal(&s).unwrap().unwrap();
    store
        .edit_goal(
            &s,
            &GoalEdit::Clear {
                revision: g.revision,
            },
            Default::default(),
        )
        .unwrap();
    store
        .edit_goal(
            &s,
            &GoalEdit::Set {
                revision: Some(g.revision + 1),
                objective: "Different".into(),
                constraints: String::new(),
                criteria: vec!["Different criterion".into()],
            },
            Default::default(),
        )
        .unwrap();
    assert!(planning::check_goal(store.as_ref(), &s, &state).is_err());
    drop(engine);
    drop(store);
    let reopened = SqliteStore::open(&dir.path().join("data.db")).unwrap();
    assert_eq!(reopened.planning(&s).unwrap(), state);
    assert!(reopened.runs(&s).unwrap().is_empty());
    let mut json = vec![];
    reopened
        .export_conversation(&s, dolores_core::ExportFormat::Json, &mut json)
        .unwrap();
    assert!(String::from_utf8(json)
        .unwrap()
        .contains("Preserve unrelated files"));
    reopened.delete(&s).unwrap();
    assert_eq!(reopened.planning(&s).unwrap(), PlanningState::default());
}
#[test]
fn planning_policy_has_no_commands_delegation_or_external_tools_and_failed_save_retains_plan() {
    let (engine, store, dir, s) = engine();
    engine
        .edit_planning(&s, 0, planning::Edit::Mode { enabled: true })
        .unwrap();
    let view = engine
        .call(Command::Context {
            session: Some(s.clone()),
            input: "Inspect only".into(),
            tools: true,
        })
        .unwrap();
    let encoded = serde_json::to_string(&view).unwrap();
    assert!(encoded.contains("Planning mode"));
    assert!(view["tools"]
        .as_array()
        .unwrap()
        .iter()
        .all(|t| inspection_tool(t["name"].as_str().unwrap())));
    for name in [
        "run_command",
        "create_text_file",
        "edit_text_file",
        "delegate_tasks",
        "schedule_task",
        "browser",
        "mcp_tool_fixture",
    ] {
        assert!(!inspection_tool(name));
    }
    assert!(inspection_tool("read_text_file"));
    let state = store.planning(&s).unwrap();
    rusqlite::Connection::open(dir.path().join("data.db")).unwrap().execute_batch("CREATE TRIGGER fail_plan BEFORE UPDATE ON planning BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
    assert!(planning::save_result(store.as_ref(), &s, &state, "Useful draft plan").is_err());
    assert_eq!(store.planning(&s).unwrap(), state);
    assert!(store.runs(&s).unwrap().is_empty());
}

#[test]
fn all_excerpt_sources_preserve_exact_text_and_context_exhaustion_refuses_before_dispatch() {
    use sha2::{Digest, Sha256};
    let (engine, store, _dir, s) = engine();
    for (kind, name) in [
        ("code", "Selection v2 - source.rs"),
        ("terminal", "Terminal output.txt"),
    ] {
        let text = format!("{kind} PUBLIC_SELECTED 世界");
        let digest = format!("{:x}", Sha256::digest(text.as_bytes()));
        store
            .add_attachment(
                &s,
                &dolores_core::AttachmentData {
                    reference: dolores_core::AttachmentRef {
                        digest: digest.clone(),
                        name: name.into(),
                        mime: "text/plain".into(),
                        bytes: text.len(),
                    },
                    data: text.as_bytes().to_vec(),
                },
            )
            .unwrap();
        let source = if kind == "code" {
            planning::Source::Code { digest }
        } else {
            planning::Source::Terminal { digest }
        };
        assert_eq!(engine.reference_preview(&s, &source).unwrap().text, text);
    }
    let root = store.workspace(&s).unwrap().root.unwrap();
    std::fs::write(
        std::path::Path::new(&root).join("same.txt"),
        "SELECTED_PUBLIC_CONTEXT 世界",
    )
    .unwrap();
    let first = engine
        .reference_preview(
            &s,
            &planning::Source::File {
                path: "public.txt".into(),
            },
        )
        .unwrap();
    let second = engine
        .reference_preview(
            &s,
            &planning::Source::File {
                path: "same.txt".into(),
            },
        )
        .unwrap();
    assert_eq!(first.text, second.text);
    assert_ne!(
        first.revision, second.revision,
        "Same text from different sources must remain independently removable"
    );
    let other = engine
        .create_working_session(dolores_core::WorkspaceKind::Project, Some(root.into()))
        .unwrap()["session"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    store
        .commit_turn(&other, "Original shared input 世界", "Public evidence")
        .unwrap();
    let id = store.messages_page(&other, None, false, 80).unwrap().items[0].id;
    assert!(engine
        .reference_preview(
            &s,
            &planning::Source::Message {
                session: other.clone(),
                message: id
            }
        )
        .unwrap()
        .text
        .contains("Original shared input 世界"));
    let conversation = engine
        .reference_preview(&s, &planning::Source::Conversation { session: other })
        .unwrap();
    assert!(conversation.omissions.contains("attachments"));
    let mut state = PlanningState {
        references: vec![conversation],
        ..Default::default()
    };
    state.references[0].text = "x".repeat(16000);
    let mut context = preview_context(vec![], "Preserved original input").unwrap();
    context[0].content.push_str(&"s".repeat(120000));
    let prepared = prepare(context, &state).unwrap();
    assert!(dolores_core::prepare_token_context(prepared,&[],Some(32768),Default::default()).is_err(),"Configured model capacity, not a fixed public-text ceiling, must reject oversized fixed instructions");
    assert_eq!(state.references[0].text.len(), 16000);
    assert!(store.runs(&s).unwrap().is_empty());
}
#[test]
fn v41_migration_preserves_unicode_history_and_accepted_versions_survive_new_drafts() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("migration.db");
    {
        let s = SqliteStore::open(&path).unwrap();
        s.create("chat").unwrap();
        s.commit_turn("chat", "保留原始输入", "Saved reply")
            .unwrap();
    }
    rusqlite::Connection::open(&path)
        .unwrap()
        .execute_batch("DROP TABLE plan_versions; DROP TABLE planning; PRAGMA user_version=41;")
        .unwrap();
    let store = Arc::new(SqliteStore::open(&path).unwrap());
    let engine = Engine::new(
        store.clone(),
        Arc::new(connection::testing::MemoryCredentials::default()),
    )
    .unwrap();
    engine
        .edit_planning(
            "chat",
            0,
            planning::Edit::Draft {
                text: "First concrete plan".into(),
                decisions: "Human choice".into(),
            },
        )
        .unwrap();
    engine
        .edit_planning("chat", 1, planning::Edit::Accept)
        .unwrap();
    engine
        .edit_planning(
            "chat",
            2,
            planning::Edit::Draft {
                text: "Revised plan".into(),
                decisions: "Human choice retained".into(),
            },
        )
        .unwrap();
    assert!(store.planning("chat").unwrap().accepted.is_none());
    let mut export = vec![];
    store
        .export_conversation("chat", dolores_core::ExportFormat::Json, &mut export)
        .unwrap();
    let value: Value = serde_json::from_slice(&export).unwrap();
    assert_eq!(value["acceptedPlans"][0]["text"], "First concrete plan");
    assert_eq!(store.messages("chat").unwrap()[0].content, "保留原始输入");
    store.delete("chat").unwrap();
    let c = rusqlite::Connection::open(&path).unwrap();
    let n: i64 = c
        .query_row("SELECT count(*) FROM plan_versions", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 0);
}
