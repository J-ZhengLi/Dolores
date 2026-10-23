use super::*;
use dolores_core::{
    goal::GoalEdit,
    harness_task::{InputMode, InputState},
    SessionStore, SessionWorkspace, WorkspaceKind,
};

struct Fixture {
    dir: tempfile::TempDir,
    store: SqliteStore,
    parent_message: i64,
}

#[test]
fn oversized_questions_and_encoded_conclusions_keep_drafts_and_retained_receipts_have_a_recovery_limit(
) {
    let f = Fixture::new();
    let mut request = f.create_request();
    request.question = "世".repeat(1400);
    assert!(f
        .store
        .create_side_conversation("parent", &request)
        .unwrap_err()
        .contains("4 KiB"));
    assert!(!exists(&f.store.lock().unwrap(), "side-child").unwrap());
    request.question = "Narrow question".into();
    f.store
        .create_side_conversation("parent", &request)
        .unwrap();
    let quote = f.child_turn();
    let mut encoded = quote.clone();
    encoded.excerpt = "\u{0001}".repeat(4096);
    let invalid = Return {
        id: "encoded-limit".into(),
        quote: encoded,
        mode: InputMode::Queue,
    };
    assert!(f
        .store
        .return_side_conclusion("side-child", &invalid)
        .unwrap_err()
        .contains("16 KiB"));
    assert!(f.store.task_inputs("parent").unwrap().entries.is_empty());
    let mut wrong_role = quote.clone();
    wrong_role.message = f
        .store
        .messages_page("side-child", None, false, 80)
        .unwrap()
        .items[0]
        .id;
    assert!(f
        .store
        .return_side_conclusion(
            "side-child",
            &Return {
                id: "not-an-assistant".into(),
                quote: wrong_role,
                mode: InputMode::Queue
            }
        )
        .unwrap_err()
        .contains("assistant"));
    for n in 0..MAX_RETURNS {
        let id = format!("conclusion-{n}");
        f.store
            .return_side_conclusion(
                "side-child",
                &Return {
                    id: id.clone(),
                    quote: quote.clone(),
                    mode: InputMode::Queue,
                },
            )
            .unwrap();
        f.store
            .edit_task_inputs("parent", &InputEdit::Cancel { id, revision: 1 })
            .unwrap();
    }
    assert!(f
        .store
        .return_side_conclusion(
            "side-child",
            &Return {
                id: "over-limit".into(),
                quote,
                mode: InputMode::Queue
            }
        )
        .unwrap_err()
        .contains("32 returned"));
    assert_eq!(
        f.store
            .side_conversation("side-child")
            .unwrap()
            .unwrap()
            .origin
            .returns
            .len(),
        MAX_RETURNS
    );
    assert_eq!(
        f.store.saved_draft("parent").unwrap(),
        "Keep parent draft 世界"
    );
    assert_eq!(
        f.store.saved_draft("side-child").unwrap(),
        "Narrow question"
    );
}
impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let store = SqliteStore::open(&dir.path().join("state.db")).unwrap();
        store
            .create_workspace_session(
                "parent",
                &SessionWorkspace {
                    kind: WorkspaceKind::Project,
                    root: Some("C:/public-project".into()),
                },
            )
            .unwrap();
        store
            .commit_turn(
                "parent",
                "Original task",
                "A **public selection** and unrelated private tail.",
            )
            .unwrap();
        store
            .save_draft("parent", "Keep parent draft 世界")
            .unwrap();
        let parent_message = store
            .messages_page("parent", None, false, 80)
            .unwrap()
            .items[1]
            .id;
        Self {
            dir,
            store,
            parent_message,
        }
    }
    fn create_request(&self) -> Create {
        Create {
            id: "side-child".into(),
            quote: self
                .store
                .side_quote("parent", self.parent_message, "public selection")
                .unwrap(),
            question: "Explain this phrase".into(),
        }
    }
    fn child_turn(&self) -> Quote {
        self.store
            .commit_turn(
                "side-child",
                "A side question",
                "Selected conclusion: handle empty input. Unshared tail.",
            )
            .unwrap();
        let id = self
            .store
            .messages_page("side-child", None, false, 80)
            .unwrap()
            .items[1]
            .id;
        self.store
            .side_quote("side-child", id, "handle empty input")
            .unwrap()
    }
}

#[test]
fn creation_keeps_only_selected_context_and_retains_parent_state_and_idempotent_child_draft() {
    let f = Fixture::new();
    let before = f.store.messages("parent").unwrap();
    let parent_draft = f.store.saved_draft("parent").unwrap();
    f.store
        .edit_task_inputs(
            "parent",
            &InputEdit::Enqueue {
                id: "old-queue".into(),
                text: "Then verify the existing task".into(),
                mode: InputMode::Queue,
            },
        )
        .unwrap();
    let inputs = f.store.task_inputs("parent").unwrap();
    let request = f.create_request();
    let made = f
        .store
        .create_side_conversation("parent", &request)
        .unwrap();
    assert_eq!(made.session.id, "side-child");
    assert_eq!(
        f.store.workspace("side-child").unwrap(),
        SessionWorkspace::default()
    );
    assert!(f.store.messages("side-child").unwrap().is_empty());
    assert!(f.store.goal("side-child").unwrap().is_none());
    assert!(f
        .store
        .task_inputs("side-child")
        .unwrap()
        .entries
        .is_empty());
    assert!(f.store.runs("side-child").unwrap().is_empty());
    assert_eq!(
        f.store.saved_draft("side-child").unwrap(),
        "Explain this phrase"
    );
    let context = f.store.planning("side-child").unwrap();
    assert_eq!(context.references.len(), 1);
    assert_eq!(context.references[0].text, "public selection");
    assert!(!serde_json::to_string(&context)
        .unwrap()
        .contains("private tail"));
    assert!(!context.planning);
    assert!(context.accepted.is_none());
    f.store
        .save_draft("side-child", "Newer side draft")
        .unwrap();
    f.store
        .create_side_conversation("parent", &request)
        .unwrap();
    assert_eq!(
        f.store.saved_draft("side-child").unwrap(),
        "Newer side draft"
    );
    let mut changed = request.clone();
    changed.question = "different question".into();
    assert!(f
        .store
        .create_side_conversation("parent", &changed)
        .is_err());
    assert_eq!(f.store.messages("parent").unwrap(), before);
    assert_eq!(f.store.saved_draft("parent").unwrap(), parent_draft);
    assert_eq!(f.store.task_inputs("parent").unwrap(), inputs);
    let quote = f.child_turn();
    let bring = Return {
        id: "returned-conclusion".into(),
        quote,
        mode: InputMode::Steer,
    };
    f.store
        .return_side_conclusion("side-child", &bring)
        .unwrap();
    let queued = f.store.task_inputs("parent").unwrap();
    assert_eq!(queued.entries.len(), 2);
    assert!(queued.entries[1].text.contains("handle empty input"));
    assert!(!queued.entries[1].text.contains("Unshared tail"));
    assert_eq!(queued.entries[1].mode, InputMode::Steer);
    assert_eq!(f.store.saved_draft("parent").unwrap(), parent_draft);
    assert_eq!(
        f.store
            .return_side_conclusion("side-child", &bring)
            .unwrap()
            .origin
            .returns
            .len(),
        1
    );
    assert_eq!(f.store.task_inputs("parent").unwrap(), queued);
    assert!(f.store.side_conversation("parent").unwrap().is_none());
}

#[test]
fn failed_creation_and_return_are_atomic_and_stale_or_deleted_sources_keep_frozen_context() {
    let f = Fixture::new();
    let request = f.create_request();
    f.store.lock().unwrap().execute_batch("CREATE TRIGGER fail_side_context BEFORE INSERT ON planning BEGIN SELECT RAISE(ABORT,'fixture context storage failure'); END;").unwrap();
    assert!(f
        .store
        .create_side_conversation("parent", &request)
        .is_err());
    assert!(!exists(&f.store.lock().unwrap(), "side-child").unwrap());
    f.store
        .lock()
        .unwrap()
        .execute_batch("DROP TRIGGER fail_side_context;")
        .unwrap();
    let mut stale = request.clone();
    stale.quote.revision = "0".repeat(64);
    assert!(f
        .store
        .create_side_conversation("parent", &stale)
        .unwrap_err()
        .contains("changed"));
    f.store
        .create_side_conversation("parent", &request)
        .unwrap();
    let quote = f.child_turn();
    let bring = Return {
        id: "return-once".into(),
        quote: quote.clone(),
        mode: InputMode::Queue,
    };
    f.store.lock().unwrap().execute_batch("CREATE TRIGGER fail_return_receipt BEFORE UPDATE ON thread_context BEGIN SELECT RAISE(ABORT,'fixture return receipt failure'); END;").unwrap();
    assert!(f
        .store
        .return_side_conclusion("side-child", &bring)
        .is_err());
    assert!(f.store.task_inputs("parent").unwrap().entries.is_empty());
    assert!(f
        .store
        .side_conversation("side-child")
        .unwrap()
        .unwrap()
        .origin
        .returns
        .is_empty());
    f.store
        .lock()
        .unwrap()
        .execute_batch("DROP TRIGGER fail_return_receipt;")
        .unwrap();
    let mut wrong = bring.clone();
    wrong.quote.revision = "f".repeat(64);
    assert!(f
        .store
        .return_side_conclusion("side-child", &wrong)
        .unwrap_err()
        .contains("changed"));
    f.store
        .return_side_conclusion("side-child", &bring)
        .unwrap();
    f.store.delete("parent").unwrap();
    let info = f.store.side_conversation("side-child").unwrap().unwrap();
    assert!(!info.parent_available);
    assert!(!info.source_available);
    assert_eq!(info.origin.quote.excerpt, "public selection");
    assert_eq!(
        f.store.planning("side-child").unwrap().references[0].text,
        "public selection"
    );
    assert!(
        !f.store
            .return_side_conclusion("side-child", &bring)
            .unwrap()
            .parent_available
    );
    let new = Return {
        id: "after-parent-deleted".into(),
        ..bring
    };
    assert!(f
        .store
        .return_side_conclusion("side-child", &new)
        .unwrap_err()
        .contains("unavailable"));
    f.store
        .create_side_conversation("parent", &request)
        .unwrap();
    assert_eq!(f.store.messages("side-child").unwrap().len(), 2);
}

#[test]
fn full_or_paused_parent_queue_and_restart_never_replay_a_return_or_replace_a_goal() {
    let f = Fixture::new();
    let goal = f
        .store
        .edit_goal(
            "parent",
            &GoalEdit::Set {
                revision: None,
                objective: "Finish the original task".into(),
                criteria: vec!["Verify the saved artifact".into()],
                constraints: "Preserve unrelated work".into(),
            },
            Default::default(),
        )
        .unwrap();
    let request = f.create_request();
    f.store
        .create_side_conversation("parent", &request)
        .unwrap();
    let quote = f.child_turn();
    for n in 0..8 {
        f.store
            .edit_task_inputs(
                "parent",
                &InputEdit::Enqueue {
                    id: format!("input-{n}"),
                    text: format!("Existing work {n}"),
                    mode: InputMode::Queue,
                },
            )
            .unwrap();
    }
    let before = f.store.task_inputs("parent").unwrap();
    let bring = Return {
        id: "conclusion".into(),
        quote,
        mode: InputMode::Queue,
    };
    assert!(f
        .store
        .return_side_conclusion("side-child", &bring)
        .unwrap_err()
        .contains("queue is full"));
    assert_eq!(f.store.task_inputs("parent").unwrap(), before);
    f.store
        .edit_task_inputs(
            "parent",
            &InputEdit::Cancel {
                id: "input-0".into(),
                revision: 1,
            },
        )
        .unwrap();
    f.store.suspend_task_inputs("parent").unwrap();
    f.store
        .return_side_conclusion("side-child", &bring)
        .unwrap();
    assert!(f.store.task_inputs("parent").unwrap().suspended);
    assert_eq!(f.store.goal("parent").unwrap(), Some(goal.clone()));
    assert!(f
        .store
        .claim_task_input("parent", "conclusion", 1, "run")
        .is_err());
    let path = f.dir.path().join("state.db");
    drop(f.store);
    let store = SqliteStore::open(&path).unwrap();
    store.recover_task_inputs().unwrap();
    assert_eq!(
        store
            .side_conversation("side-child")
            .unwrap()
            .unwrap()
            .origin
            .returns
            .len(),
        1
    );
    let state = store.task_inputs("parent").unwrap();
    store
        .edit_task_inputs(
            "parent",
            &InputEdit::Resume {
                revision: state.revision,
            },
        )
        .unwrap();
    store
        .claim_task_input("parent", "conclusion", 1, "run")
        .unwrap();
    store.recover_task_inputs().unwrap();
    let inputs = store.task_inputs("parent").unwrap();
    let retained = inputs
        .entries
        .iter()
        .find(|e| e.id == "conclusion")
        .unwrap();
    assert_eq!(retained.state, InputState::Uncertain);
    let count = inputs.entries.len();
    store.return_side_conclusion("side-child", &bring).unwrap();
    assert_eq!(store.task_inputs("parent").unwrap().entries.len(), count);
    assert_eq!(store.goal("parent").unwrap(), Some(goal));
}
