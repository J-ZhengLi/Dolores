use super::*;
use dolores_core::goal::*;
use dolores_core::TaskBudget;
fn set() -> GoalEdit {
    GoalEdit::Set {
        revision: None,
        objective: "Create a public report and verify its content".into(),
        criteria: vec!["Report exists with verified content".into()],
        constraints: "Preserve existing files".into(),
    }
}
fn store() -> SqliteStore {
    let s = SqliteStore::open(Path::new(":memory:")).unwrap();
    s.create("chat").unwrap();
    s
}
fn run(s: &SqliteStore, id: &str) {
    s.begin_run(&serde_json::from_value(serde_json::json!({"id":id,"thread":"chat","model":"fixture","settings":RequestSettings::default(),"input":"Public goal","state":"prepared","sequence":0,"createdAt":1,"build":"fixture","tools":["create_text_file"]})).unwrap()).unwrap();
}
fn evidence(s: &SqliteStore, id: &str) -> GoalEvidence {
    s.append_run_event(
        id,
        0,
        Some(dolores_core::RunState::Running),
        "started",
        &serde_json::json!({}),
    )
    .unwrap();
    let seq=s.append_run_event(id,1,None,"toolResult",&serde_json::json!({"name":"create_text_file","returned":true,"target":"report.txt","content":"Saved public report"})).unwrap();
    GoalEvidence {
        run: id.into(),
        sequence: seq,
    }
}
#[test]
fn explicit_goal_criteria_receipts_and_normal_finish_are_required() {
    let s = store();
    assert!(s.goal("chat").unwrap().is_none());
    let g = s.edit_goal("chat", &set(), TaskBudget::default()).unwrap();
    run(&s, "11111111-1111-4111-8111-111111111111");
    s.claim_goal(
        "chat",
        &g.id,
        g.revision,
        "11111111-1111-4111-8111-111111111111",
    )
    .unwrap();
    let mut p = GoalProgress {
        revision: 0,
        steps: g.steps.clone(),
        complete: true,
        blocked: false,
        note: "Verified".into(),
    };
    p.steps[0].done = true;
    assert!(s
        .progress_goal("chat", &g.id, "11111111-1111-4111-8111-111111111111", &p)
        .is_err());
    p.steps[0].evidence = vec![GoalEvidence {
        run: "other".into(),
        sequence: 1,
    }];
    assert!(s
        .progress_goal("chat", &g.id, "11111111-1111-4111-8111-111111111111", &p)
        .is_err());
    let verified = evidence(&s, "11111111-1111-4111-8111-111111111111");
    p.steps[0].evidence = vec![GoalEvidence {
        run: verified.run.clone(),
        sequence: 999,
    }];
    assert!(s.progress_goal("chat", &g.id, &verified.run, &p).is_err());
    let mut last = verified.sequence;
    for data in [
        serde_json::json!({"name":"read_text_file","returned":false,"content":"Unavailable"}),
        serde_json::json!({"name":"run_command","returned":true,"content":r#"{"exitCode":1,"reason":"completed"}"#}),
        serde_json::json!({"name":"inspect_goal","returned":true,"content":"Progress narration"}),
    ] {
        last = s
            .append_run_event(&verified.run, last, None, "toolResult", &data)
            .unwrap();
        p.steps[0].evidence = vec![GoalEvidence {
            run: verified.run.clone(),
            sequence: last,
        }];
        assert!(s.progress_goal("chat", &g.id, &verified.run, &p).is_err());
    }
    p.steps[0].evidence = vec![verified];
    s.progress_goal("chat", &g.id, "11111111-1111-4111-8111-111111111111", &p)
        .unwrap();
    assert!(s
        .progress_goal("chat", &g.id, "11111111-1111-4111-8111-111111111111", &p)
        .is_err());
    let paused = s
        .finish_goal(
            "chat",
            &g.id,
            "11111111-1111-4111-8111-111111111111",
            false,
            true,
        )
        .unwrap();
    assert_eq!(paused.state, GoalState::Paused);
    assert!(!paused.completion_requested);
    let resumed = s
        .edit_goal(
            "chat",
            &GoalEdit::Resume {
                revision: paused.revision,
            },
            TaskBudget::default(),
        )
        .unwrap();
    run(&s, "22222222-2222-4222-8222-222222222222");
    s.claim_goal(
        "chat",
        &g.id,
        resumed.revision,
        "22222222-2222-4222-8222-222222222222",
    )
    .unwrap();
    p.revision = 1;
    s.progress_goal("chat", &g.id, "22222222-2222-4222-8222-222222222222", &p)
        .unwrap();
    assert_eq!(
        s.finish_goal(
            "chat",
            &g.id,
            "22222222-2222-4222-8222-222222222222",
            true,
            true
        )
        .unwrap()
        .state,
        GoalState::Completed
    );
}
#[test]
fn cumulative_attempts_and_segments_do_not_reset_and_no_progress_blocks() {
    let s = store();
    let g = s
        .edit_goal(
            "chat",
            &set(),
            TaskBudget {
                model_calls: Some(3),
                tool_calls: Some(2),
                segments: 3,
                ..Default::default()
            },
        )
        .unwrap();
    run(&s, "33333333-3333-4333-8333-333333333333");
    s.claim_goal(
        "chat",
        &g.id,
        g.revision,
        "33333333-3333-4333-8333-333333333333",
    )
    .unwrap();
    assert!(s
        .charge_goal(
            "chat",
            &g.id,
            "33333333-3333-4333-8333-333333333333",
            GoalCharge::Model
        )
        .unwrap());
    assert!(s
        .charge_goal(
            "chat",
            &g.id,
            "33333333-3333-4333-8333-333333333333",
            GoalCharge::Tools(1)
        )
        .unwrap());
    let g = s
        .finish_goal(
            "chat",
            &g.id,
            "33333333-3333-4333-8333-333333333333",
            true,
            false,
        )
        .unwrap();
    run(&s, "44444444-4444-4444-8444-444444444444");
    s.claim_goal(
        "chat",
        &g.id,
        g.revision,
        "44444444-4444-4444-8444-444444444444",
    )
    .unwrap();
    assert!(s
        .charge_goal(
            "chat",
            &g.id,
            "44444444-4444-4444-8444-444444444444",
            GoalCharge::Model
        )
        .unwrap());
    let g = s
        .finish_goal(
            "chat",
            &g.id,
            "44444444-4444-4444-8444-444444444444",
            true,
            false,
        )
        .unwrap();
    assert_eq!(g.state, GoalState::Blocked);
    assert_eq!(g.used.model_calls, 2);
    assert_eq!(g.used.segments, 2);
    let g = s
        .edit_goal(
            "chat",
            &GoalEdit::Resume {
                revision: g.revision,
            },
            TaskBudget::default(),
        )
        .unwrap();
    run(&s, "55555555-5555-4555-8555-555555555555");
    s.claim_goal(
        "chat",
        &g.id,
        g.revision,
        "55555555-5555-4555-8555-555555555555",
    )
    .unwrap();
    assert!(s
        .charge_goal(
            "chat",
            &g.id,
            "55555555-5555-4555-8555-555555555555",
            GoalCharge::Model
        )
        .unwrap());
    assert!(!s
        .charge_goal(
            "chat",
            &g.id,
            "55555555-5555-4555-8555-555555555555",
            GoalCharge::Model
        )
        .unwrap());
    let g = s
        .finish_goal(
            "chat",
            &g.id,
            "55555555-5555-4555-8555-555555555555",
            true,
            true,
        )
        .unwrap();
    assert_eq!(g.state, GoalState::BudgetLimited);
    assert_eq!(g.used.model_calls, 3);
    assert!(s
        .edit_goal(
            "chat",
            &GoalEdit::Resume {
                revision: g.revision
            },
            TaskBudget::default()
        )
        .is_err());
}
#[test]
fn failed_save_and_v40_reopen_keep_history_goal_and_uncertain_work() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("goal.db");
    let s = SqliteStore::open(&path).unwrap();
    s.create("chat").unwrap();
    s.commit_turn("chat", "Original 世界", "Retained").unwrap();
    s.save_draft("chat", "New draft").unwrap();
    s.lock()
        .unwrap()
        .execute_batch("DROP TABLE goals;PRAGMA user_version=40;")
        .unwrap();
    drop(s);
    let s = SqliteStore::open(&path).unwrap();
    s.lock().unwrap().execute_batch("CREATE TRIGGER reject_goal BEFORE INSERT ON goals BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
    assert!(s.edit_goal("chat", &set(), TaskBudget::default()).is_err());
    assert_eq!(s.saved_draft("chat").unwrap(), "New draft");
    assert!(s.goal("chat").unwrap().is_none());
    s.lock()
        .unwrap()
        .execute_batch("DROP TRIGGER reject_goal;")
        .unwrap();
    let g = s.edit_goal("chat", &set(), TaskBudget::default()).unwrap();
    run(&s, "66666666-6666-4666-8666-666666666666");
    s.claim_goal(
        "chat",
        &g.id,
        g.revision,
        "66666666-6666-4666-8666-666666666666",
    )
    .unwrap();
    s.charge_goal(
        "chat",
        &g.id,
        "66666666-6666-4666-8666-666666666666",
        GoalCharge::Model,
    )
    .unwrap();
    drop(s);
    let s = SqliteStore::open(&path).unwrap();
    s.recover_goals().unwrap();
    let g = s.goal("chat").unwrap().unwrap();
    assert_eq!(g.state, GoalState::Paused);
    assert_eq!(g.used.model_calls, 1);
    assert!(g.current_run.is_none());
    assert!(s
        .claim_goal("chat", &g.id, g.revision, "no-replay")
        .is_err());
    assert_eq!(s.messages("chat").unwrap()[0].content, "Original 世界");
    s.delete("chat").unwrap();
    assert!(s.goal("chat").unwrap().is_none());
}
#[test]
fn pending_user_input_precedes_goal_and_stale_controls_do_not_mutate() {
    let s = store();
    let g = s.edit_goal("chat", &set(), TaskBudget::default()).unwrap();
    s.edit_task_inputs(
        "chat",
        &dolores_core::harness_task::InputEdit::Enqueue {
            id: "user".into(),
            text: "Inspect this first".into(),
            mode: dolores_core::harness_task::InputMode::Queue,
        },
    )
    .unwrap();
    assert!(s
        .claim_goal(
            "chat",
            &g.id,
            g.revision,
            "66666666-6666-4666-8666-666666666666"
        )
        .is_err());
    assert!(s
        .edit_goal(
            "chat",
            &GoalEdit::Pause { revision: 99 },
            TaskBudget::default()
        )
        .is_err());
    assert_eq!(s.goal("chat").unwrap(), Some(g));
}
#[test]
fn replacing_a_cleared_goal_keeps_prior_evidence_in_exports_and_time_never_resets() {
    let s = store();
    let g = s
        .edit_goal(
            "chat",
            &set(),
            TaskBudget {
                elapsed_seconds: Some(30),
                ..Default::default()
            },
        )
        .unwrap();
    run(&s, "77777777-7777-4777-8777-777777777777");
    s.claim_goal(
        "chat",
        &g.id,
        g.revision,
        "77777777-7777-4777-8777-777777777777",
    )
    .unwrap();
    for _ in 0..30 {
        assert!(s
            .charge_goal(
                "chat",
                &g.id,
                "77777777-7777-4777-8777-777777777777",
                GoalCharge::ActiveSecond
            )
            .unwrap());
    }
    assert!(!s
        .charge_goal(
            "chat",
            &g.id,
            "77777777-7777-4777-8777-777777777777",
            GoalCharge::ActiveSecond
        )
        .unwrap());
    let g = s
        .finish_goal(
            "chat",
            &g.id,
            "77777777-7777-4777-8777-777777777777",
            false,
            false,
        )
        .unwrap();
    assert_eq!(g.state, GoalState::BudgetLimited);
    let old = s
        .edit_goal(
            "chat",
            &GoalEdit::Clear {
                revision: g.revision,
            },
            TaskBudget::default(),
        )
        .unwrap();
    let mut edit = set();
    if let GoalEdit::Set { revision, .. } = &mut edit {
        *revision = Some(old.revision);
    }
    let next = s.edit_goal("chat", &edit, TaskBudget::default()).unwrap();
    assert_ne!(next.id, old.id);
    let mut bytes = vec![];
    s.write_export("chat", dolores_core::ExportFormat::Json, &mut bytes)
        .unwrap();
    let export: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(export["goals"].as_array().unwrap().len(), 2);
    assert_eq!(export["goals"][0]["used"]["activeSeconds"], 30);
    s.delete("chat").unwrap();
    let count: u32 = s
        .lock()
        .unwrap()
        .query_row("SELECT count(*) FROM goal_history", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 0);
}
