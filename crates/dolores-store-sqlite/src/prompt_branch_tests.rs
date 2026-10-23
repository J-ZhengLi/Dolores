use super::*;
use dolores_core::{
    goal::GoalEdit,
    harness_task::{InputEdit, InputMode},
    AttachmentData, AttachmentRef, ContextSummary, SessionStore,
};

struct Fixture {
    dir: tempfile::TempDir,
    store: SqliteStore,
    selected: i64,
}
impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let store = SqliteStore::open(&dir.path().join("state.db")).unwrap();
        let root = dir.path().join("public-project");
        std::fs::create_dir(&root).unwrap();
        std::fs::write(
            root.join("a.txt"),
            "Current files after historical effects\n",
        )
        .unwrap();
        store
            .create_workspace_session(
                "parent",
                &SessionWorkspace {
                    kind: WorkspaceKind::Project,
                    root: Some(root.to_string_lossy().into()),
                },
            )
            .unwrap();
        for (user, answer, target) in [
            ("First prompt", "First saved reply", "prior.txt"),
            (
                "Original prompt to edit",
                "Old answer must not become live context",
                "a.txt",
            ),
            (
                "Later original prompt",
                "Later answer stays historical",
                "later.txt",
            ),
        ] {
            let metadata = TurnMetadata {model:"fixture".into(),usage:None,context:ContextSummary::from_messages(&[],None),paused:None,request_settings:None,agent:Some(serde_json::from_value(serde_json::json!({"modelCalls":1,"usageByCall":[],"tools":[{"callId":target,"name":"edit_text_file","target":target,"status":"edited","content":"Historical saved effect receipt"}]})).unwrap())};
            store
                .commit_turn_metadata("parent", user, answer, &metadata)
                .unwrap();
        }
        store
            .save_draft("parent", "Keep newer unsent parent draft 世界")
            .unwrap();
        let selected = store
            .messages_page("parent", None, false, 80)
            .unwrap()
            .items[2]
            .id;
        Self {
            dir,
            store,
            selected,
        }
    }
    fn request(&self) -> Create {
        Create {
            id: "edited-branch".into(),
            basis: self
                .store
                .preview_prompt_branch("parent", self.selected)
                .unwrap()
                .basis,
            edited: "Edited prompt 世界".into(),
            include_attachments: false,
            include_context: false,
            folder: Folder::Shared,
            target_revision: None,
        }
    }
}

#[test]
fn editing_an_old_prompt_preserves_source_files_receipts_goal_and_draft_without_copying_task_authority(
) {
    let f = Fixture::new();
    let original = f
        .store
        .messages_page("parent", None, false, 80)
        .unwrap()
        .items;
    let goal = f
        .store
        .edit_goal(
            "parent",
            &GoalEdit::Set {
                revision: None,
                objective: "Original task".into(),
                criteria: vec!["Verify artifact".into()],
                constraints: "Keep files".into(),
            },
            Default::default(),
        )
        .unwrap();
    f.store
        .edit_task_inputs(
            "parent",
            &InputEdit::Enqueue {
                id: "parent-input".into(),
                text: "Parent followup".into(),
                mode: InputMode::Queue,
            },
        )
        .unwrap();
    let queue = f.store.task_inputs("parent").unwrap();
    let preview = f.store.preview_prompt_branch("parent", f.selected).unwrap();
    assert_eq!(preview.prior_turns, 1);
    assert_eq!(preview.later_turns, 2);
    assert_eq!(preview.effect_turns, 3);
    let request = f.request();
    let made = f.store.create_prompt_branch("parent", &request).unwrap();
    assert_eq!(made.info.origin.copied_turns, 1);
    assert_eq!(made.info.origin.archived_turns, 3);
    let child = f
        .store
        .messages_page("edited-branch", None, false, 80)
        .unwrap()
        .items;
    assert_eq!(child.len(), 2);
    assert_eq!(child[0].content, "First prompt");
    assert_eq!(
        child[1]
            .metadata
            .as_ref()
            .unwrap()
            .agent
            .as_ref()
            .unwrap()
            .tools[0]
            .target,
        "prior.txt"
    );
    assert!(child[1].metadata.as_ref().unwrap().paused.is_none());
    assert_eq!(child[0].saved_at, original[0].saved_at);
    assert_eq!(
        f.store.saved_draft("edited-branch").unwrap(),
        "Edited prompt 世界"
    );
    assert_eq!(
        f.store.workspace("edited-branch").unwrap(),
        f.store.workspace("parent").unwrap()
    );
    assert!(f.store.goal("edited-branch").unwrap().is_none());
    assert!(f.store.runs("edited-branch").unwrap().is_empty());
    assert!(f
        .store
        .task_inputs("edited-branch")
        .unwrap()
        .entries
        .is_empty());
    let planning = f.store.planning("edited-branch").unwrap();
    assert!(!planning.planning);
    assert!(planning.accepted.is_none());
    assert!(planning.references.is_empty());
    assert!(f.store.composer_skills("edited-branch").unwrap().is_empty());
    assert_eq!(
        f.store
            .scoped_settings(dolores_core::SettingsScope::Thread, "edited-branch")
            .unwrap()
            .revision,
        0
    );
    let snapshot = f.store.prompt_branch_archive("edited-branch", 0).unwrap();
    assert!(snapshot
        .text
        .contains("Old answer must not become live context"));
    assert!(snapshot.text.contains("Later answer stays historical"));
    assert!(snapshot.text.contains("Historical saved effect receipt"));
    assert_eq!(
        std::fs::read_to_string(f.dir.path().join("public-project/a.txt")).unwrap(),
        "Current files after historical effects\n"
    );
    assert_eq!(f.store.messages("parent").unwrap().len(), 6);
    assert_eq!(
        f.store.saved_draft("parent").unwrap(),
        "Keep newer unsent parent draft 世界"
    );
    assert_eq!(f.store.task_inputs("parent").unwrap(), queue);
    assert_eq!(f.store.goal("parent").unwrap().unwrap(), goal);
    f.store
        .save_draft("edited-branch", "A newer branch draft")
        .unwrap();
    f.store.create_prompt_branch("parent", &request).unwrap();
    assert_eq!(
        f.store.saved_draft("edited-branch").unwrap(),
        "A newer branch draft"
    );
    let mut collision = request;
    collision.edited = "Different request".into();
    assert!(f
        .store
        .create_prompt_branch("parent", &collision)
        .unwrap_err()
        .contains("identity already"));
    assert_eq!(f.store.messages("edited-branch").unwrap().len(), 2);
}

#[test]
fn failed_creation_and_changed_source_are_atomic_and_repreview_keeps_the_edited_text() {
    let f = Fixture::new();
    let mut request = f.request();
    f.store.lock().unwrap().execute_batch("CREATE TRIGGER fail_branch_context BEFORE INSERT ON planning BEGIN SELECT RAISE(ABORT,'fixture branch context failure'); END;").unwrap();
    assert!(f.store.create_prompt_branch("parent", &request).is_err());
    assert!(!exists(&f.store.lock().unwrap(), "edited-branch").unwrap());
    assert!(f.store.prompt_branch("edited-branch").unwrap().is_none());
    f.store
        .lock()
        .unwrap()
        .execute_batch("DROP TRIGGER fail_branch_context;")
        .unwrap();
    f.store
        .commit_turn_preserving_draft(
            "parent",
            "Additional original turn",
            "Another original reply",
            &TurnMetadata {
                model: "fixture".into(),
                usage: None,
                context: ContextSummary::from_messages(&[], None),
                paused: None,
                request_settings: None,
                agent: None,
            },
        )
        .unwrap();
    assert!(f
        .store
        .create_prompt_branch("parent", &request)
        .unwrap_err()
        .contains("changed after preview"));
    assert_eq!(request.edited, "Edited prompt 世界");
    assert!(!exists(&f.store.lock().unwrap(), "edited-branch").unwrap());
    request.basis = f
        .store
        .preview_prompt_branch("parent", f.selected)
        .unwrap()
        .basis;
    request.folder = Folder::Side;
    f.store.create_prompt_branch("parent", &request).unwrap();
    assert_eq!(
        f.store.workspace("edited-branch").unwrap(),
        SessionWorkspace {
            kind: WorkspaceKind::Side,
            root: None
        }
    );
    assert_eq!(
        f.store.saved_draft("parent").unwrap(),
        "Keep newer unsent parent draft 世界"
    );
    assert!(f
        .store
        .preview_prompt_branch("edited-branch", f.selected)
        .is_err());
    let wrong = original_assistant(&f);
    assert!(f
        .store
        .preview_prompt_branch("parent", wrong)
        .unwrap_err()
        .contains("saved user"));
}
fn original_assistant(f: &Fixture) -> i64 {
    f.store
        .messages_page("parent", None, false, 80)
        .unwrap()
        .items[1]
        .id
}

#[test]
fn context_snapshots_are_preview_bound_optional_and_do_not_copy_accepted_plans() {
    let f = Fixture::new();
    let state = PlanningState {
        accepted: Some(dolores_core::planning::AcceptedPlan {
            revision: 1,
            text: "Original accepted plan stays with parent".into(),
            decisions: "Parent decisions".into(),
            goal: None,
            criteria: vec!["Parent plan criterion".into()],
        }),
        references: vec![ContextReference {
            kind: "sideExcerpt".into(),
            source: "Original parent selection".into(),
            revision: "a".repeat(64),
            text: "A retained selected excerpt".into(),
            omissions: "Other messages omitted".into(),
        }],
        ..Default::default()
    };
    f.store.save_planning("parent", 0, &state).unwrap();
    f.store.lock().unwrap().execute("INSERT INTO thread_context(session_id,origin) VALUES('parent',?1)",[serde_json::json!({"kind":"sideSelection","parent":"ancestor","quote":{"excerpt":"Original frozen selection"}}).to_string()]).unwrap();
    let preview = f.store.preview_prompt_branch("parent", f.selected).unwrap();
    assert_eq!(preview.context.len(), 1);
    let mut request = f.request();
    request.include_context = true;
    let mut changed = f.store.planning("parent").unwrap();
    changed.references[0].text = "A changed selection".into();
    f.store
        .save_planning("parent", changed.revision, &changed)
        .unwrap();
    assert!(f
        .store
        .create_prompt_branch("parent", &request)
        .unwrap_err()
        .contains("changed after preview"));
    request.basis = f
        .store
        .preview_prompt_branch("parent", f.selected)
        .unwrap()
        .basis;
    f.store.create_prompt_branch("parent", &request).unwrap();
    assert_eq!(
        f.store.planning("edited-branch").unwrap().references[0].text,
        "A changed selection"
    );
    let archive = f.store.prompt_branch_archive("edited-branch", 0).unwrap();
    assert!(archive.text.contains("Original frozen selection"));
    assert!(archive.text.contains("A changed selection"));
    let mut without = request;
    without.id = "branch-without-context".into();
    without.include_context = false;
    f.store.create_prompt_branch("parent", &without).unwrap();
    assert!(f
        .store
        .planning("branch-without-context")
        .unwrap()
        .references
        .is_empty());
    assert!(f
        .store
        .planning("edited-branch")
        .unwrap()
        .accepted
        .is_none());
}

#[test]
fn archived_attachment_ownership_and_snapshot_survive_parent_deletion_cleanup_and_restart() {
    let f = Fixture::new();
    let asset = AttachmentData {
        reference: AttachmentRef {
            digest: format!("{:x}", Sha256::digest(b"Retained historical attachment")),
            name: "receipt.txt".into(),
            mime: "text/plain".into(),
            bytes: 30,
        },
        data: b"Retained historical attachment".to_vec(),
    };
    assert_eq!(asset.data.len(), asset.reference.bytes);
    f.store.add_attachment("parent", &asset).unwrap();
    f.store
        .commit_turn(
            "parent",
            "Later attachment prompt",
            "Saved attachment response",
        )
        .unwrap();
    let selected = f
        .store
        .messages_page("parent", None, false, 80)
        .unwrap()
        .items[6]
        .id;
    let later = AttachmentData {
        reference: AttachmentRef {
            digest: format!("{:x}", Sha256::digest(b"later")),
            name: "later.txt".into(),
            mime: "text/plain".into(),
            bytes: 5,
        },
        data: b"later".to_vec(),
    };
    f.store.add_attachment("parent", &later).unwrap();
    f.store
        .commit_turn("parent", "Later source attachment", "Later source reply")
        .unwrap();
    let mut request = f.request();
    request.basis = f
        .store
        .preview_prompt_branch("parent", selected)
        .unwrap()
        .basis;
    request.include_attachments = true;
    f.store.create_prompt_branch("parent", &request).unwrap();
    assert_eq!(
        f.store.draft_attachments("edited-branch").unwrap(),
        vec![asset.reference.clone()]
    );
    f.store
        .remove_attachment("edited-branch", &asset.reference.digest)
        .unwrap();
    assert!(f
        .store
        .messages("edited-branch")
        .unwrap()
        .iter()
        .all(|m| m.parts.is_empty()));
    f.store.delete("parent").unwrap();
    assert_eq!(f.store.cleanup_attachments().unwrap(), 0);
    assert_eq!(
        f.store
            .attachment_data("edited-branch", &asset.reference.digest)
            .unwrap()
            .data,
        asset.data
    );
    assert_eq!(
        f.store
            .attachment_data("edited-branch", &later.reference.digest)
            .unwrap()
            .data,
        later.data
    );
    f.store.create("unrelated").unwrap();
    assert!(f
        .store
        .attachment_data("unrelated", &asset.reference.digest)
        .is_err());
    let info = f.store.prompt_branch("edited-branch").unwrap().unwrap();
    assert!(!info.parent_available);
    assert!(!info.source_available);
    assert!(f
        .store
        .prompt_branch_archive("edited-branch", 0)
        .unwrap()
        .text
        .contains("receipt.txt"));
    let path = f.dir.path().join("state.db");
    drop(f.store);
    let reopened = SqliteStore::open(&path).unwrap();
    reopened.create_prompt_branch("parent", &request).unwrap();
    assert_eq!(
        reopened.saved_draft("edited-branch").unwrap(),
        "Edited prompt 世界"
    );
    assert_eq!(reopened.cleanup_attachments().unwrap(), 0);
    reopened.delete("edited-branch").unwrap();
    assert_eq!(reopened.cleanup_attachments().unwrap(), 2);
}

#[test]
fn version_45_migrates_without_losing_history_and_unicode_snapshot_pages_and_capacity_failures_recover(
) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("legacy.db");
    let store = SqliteStore::open(&path).unwrap();
    store
        .create_workspace_session(
            "legacy",
            &SessionWorkspace {
                kind: WorkspaceKind::Side,
                root: None,
            },
        )
        .unwrap();
    store
        .commit_turn("legacy", "Earlier original prompt", &"世".repeat(20000))
        .unwrap();
    store
        .commit_turn("legacy", "Original edited prompt", "Old selected answer")
        .unwrap();
    store.save_draft("legacy", "Retain legacy draft").unwrap();
    let selected = store
        .messages_page("legacy", None, false, 80)
        .unwrap()
        .items[2]
        .id;
    store.lock().unwrap().execute_batch("DROP TABLE prompt_branch_attachments; DROP TABLE prompt_branches; PRAGMA user_version=45;").unwrap();
    drop(store);
    let store = SqliteStore::open(&path).unwrap();
    let version: i64 = store
        .lock()
        .unwrap()
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 46);
    assert_eq!(store.saved_draft("legacy").unwrap(), "Retain legacy draft");
    assert_eq!(store.messages("legacy").unwrap().len(), 4);
    let request = Create {
        id: "edited-legacy".into(),
        basis: store
            .preview_prompt_branch("legacy", selected)
            .unwrap()
            .basis,
        edited: "Edited legacy prompt".into(),
        include_attachments: false,
        include_context: false,
        folder: Folder::Side,
        target_revision: None,
    };
    store.create_prompt_branch("legacy", &request).unwrap();
    let mut offset = 0;
    let mut text = String::new();
    let mut pages = 0;
    loop {
        let page = store
            .prompt_branch_archive("edited-legacy", offset)
            .unwrap();
        assert_eq!(page.offset, offset);
        assert!(page.text.len() <= PAGE_BYTES);
        text.push_str(&page.text);
        pages += 1;
        if let Some(next) = page.next_offset {
            assert!(next > offset);
            offset = next;
        } else {
            assert_eq!(text.len(), page.total_bytes);
            break;
        }
    }
    assert!(pages > 1);
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    let archived: Vec<ArchivedMessage> = serde_json::from_value(value["messages"].clone()).unwrap();
    assert_eq!(archived.len(), 4);
    assert_eq!(archived[1].content, "世".repeat(20000));
    let invalid_offset = text.find('世').unwrap() + 1;
    assert!(store
        .prompt_branch_archive("edited-legacy", invalid_offset)
        .unwrap_err()
        .contains("Start at the beginning"));
    for _ in 0..99 {
        store
            .commit_turn("legacy", "More original work", "Saved original result")
            .unwrap();
    }
    store
        .save_draft("legacy", "Keep over-limit source draft")
        .unwrap();
    assert!(store
        .preview_prompt_branch("legacy", selected)
        .unwrap_err()
        .contains("100 completed"));
    store.create_prompt_branch("legacy", &request).unwrap(); // Exact saved identity still opens.
    let info = store.prompt_branch("edited-legacy").unwrap().unwrap();
    assert!(!info.source_available);
    assert_eq!(
        store.saved_draft("legacy").unwrap(),
        "Keep over-limit source draft"
    );
    assert_eq!(
        store.saved_draft("edited-legacy").unwrap(),
        "Edited legacy prompt"
    );
    let raw: String = store
        .lock()
        .unwrap()
        .query_row(
            "SELECT data FROM prompt_branches WHERE session='edited-legacy'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let mut corrupted: serde_json::Value = serde_json::from_str(&raw).unwrap();
    corrupted["origin"]["copiedTurns"] = serde_json::json!(usize::MAX);
    store
        .lock()
        .unwrap()
        .execute(
            "UPDATE prompt_branches SET data=?1 WHERE session='edited-legacy'",
            [corrupted.to_string()],
        )
        .unwrap();
    assert!(store
        .prompt_branch("edited-legacy")
        .unwrap_err()
        .contains("provenance"));
    assert_eq!(
        store.saved_draft("edited-legacy").unwrap(),
        "Edited legacy prompt"
    );
    store
        .lock()
        .unwrap()
        .execute(
            "UPDATE prompt_branches SET data=?1 WHERE session='edited-legacy'",
            [raw],
        )
        .unwrap();
    assert!(store.prompt_branch("edited-legacy").unwrap().is_some());
}

#[test]
fn archived_tool_output_attachment_remains_scoped_when_its_source_is_deleted() {
    let f = Fixture::new();
    let bytes = b"Public retained tool snapshot".to_vec();
    let asset = AttachmentData {
        reference: AttachmentRef {
            digest: format!("{:x}", Sha256::digest(&bytes)),
            name: "capture.png".into(),
            mime: "image/png".into(),
            bytes: bytes.len(),
        },
        data: bytes,
    };
    f.store.add_attachment("parent", &asset).unwrap();
    let metadata = TurnMetadata {model:"fixture".into(),usage:None,context:ContextSummary::from_messages(&[],None),paused:None,request_settings:None,agent:Some(serde_json::from_value(serde_json::json!({"modelCalls":1,"usageByCall":[],"tools":[{"parts":[asset.reference],"callId":"tool-image","name":"inspect_desktop_capture","target":"public-window","status":"completed","content":"Retained historical capture"}]})).unwrap())};
    f.store
        .commit_turn_metadata(
            "parent",
            "Inspect window",
            "Historical tool output",
            &metadata,
        )
        .unwrap();
    let source_user = f
        .store
        .messages_page("parent", None, false, 80)
        .unwrap()
        .items[6]
        .id;
    // Tool outputs live in metadata, not an input attachment for the user message.
    f.store
        .lock()
        .unwrap()
        .execute(
            "DELETE FROM message_parts WHERE message_id=?1",
            [source_user],
        )
        .unwrap();
    let request = f.request();
    f.store.create_prompt_branch("parent", &request).unwrap();
    f.store.delete("parent").unwrap();
    assert_eq!(f.store.cleanup_attachments().unwrap(), 0);
    assert_eq!(
        f.store
            .attachment_data("edited-branch", &asset.reference.digest)
            .unwrap()
            .data,
        asset.data
    );
    assert!(f
        .store
        .draft_attachments("edited-branch")
        .unwrap()
        .is_empty());
    f.store.delete("edited-branch").unwrap();
    assert_eq!(f.store.cleanup_attachments().unwrap(), 1);
}

fn managed_target(f: &Fixture) -> dolores_core::worktrees::ManagedWorktree {
    use dolores_core::worktrees::*;
    let id = uuid::Uuid::new_v4().to_string();
    let root = f.dir.path().join(&id);
    let source_root = f.store.workspace("parent").unwrap().root.unwrap();
    let mut record = ManagedWorktree {
        id: id.clone(),
        session: id.clone(),
        revision: 0,
        source_root: source_root.clone(),
        root: root.to_string_lossy().into(),
        common_directory: format!("{source_root}/.git"),
        common_identity: "volume:repository".into(),
        base_ref: "main".into(),
        base_commit: "a".repeat(40),
        setup: WorktreeSetup::Skip,
        state: WorktreeState::Creating,
        identity: None,
        snapshot: None,
        recovery_from: None,
        notice: String::new(),
    };
    f.store.create_worktree(&record).unwrap();
    record.revision = 1;
    record.state = WorktreeState::Ready;
    record.identity = Some(WorktreeIdentity {
        filesystem: "volume:checkout".into(),
        git_directory: format!("{}/worktrees/{id}", record.common_directory),
        git_identity: "volume:registration".into(),
    });
    f.store.save_worktree(&record).unwrap();
    record
}

#[test]
fn managed_branch_adopts_only_its_empty_ready_target_and_failed_attachment_keeps_checkout_and_both_drafts(
) {
    let f = Fixture::new();
    let record = managed_target(&f);
    let mut request = f.request();
    request.id = record.session.clone();
    request.folder = Folder::Managed;
    request.target_revision = Some(record.revision);
    assert_eq!(
        f.store
            .prompt_branch_target("parent", &record.session, record.revision)
            .unwrap()
            .root,
        record.root
    );
    let mut stale = request.clone();
    stale.target_revision = Some(0);
    assert!(f
        .store
        .create_prompt_branch("parent", &stale)
        .unwrap_err()
        .contains("changed"));
    f.store
        .save_draft(&record.session, "A newer target draft")
        .unwrap();
    assert!(f
        .store
        .create_prompt_branch("parent", &request)
        .unwrap_err()
        .contains("already has work"));
    assert_eq!(
        f.store.saved_draft(&record.session).unwrap(),
        "A newer target draft"
    );
    f.store.save_draft(&record.session, "").unwrap();
    f.store
        .lock()
        .unwrap()
        .execute(
            "INSERT INTO draft_attachments VALUES(?1,'[]')",
            [&record.session],
        )
        .unwrap();
    f.store.lock().unwrap().execute_batch("CREATE TRIGGER fail_managed_branch BEFORE INSERT ON planning BEGIN SELECT RAISE(ABORT,'fixture failed branch attachment'); END;").unwrap();
    assert!(f.store.create_prompt_branch("parent", &request).is_err());
    assert!(f.store.prompt_branch(&record.session).unwrap().is_none());
    assert!(f.store.messages(&record.session).unwrap().is_empty());
    assert_eq!(f.store.saved_draft(&record.session).unwrap(), "");
    assert_eq!(
        f.store.worktree(&record.session).unwrap().unwrap().revision,
        record.revision
    );
    assert_eq!(
        f.store.workspace(&record.session).unwrap().root.as_deref(),
        Some(record.root.as_str())
    );
    f.store
        .lock()
        .unwrap()
        .execute_batch("DROP TRIGGER fail_managed_branch;")
        .unwrap();
    let made = f.store.create_prompt_branch("parent", &request).unwrap();
    assert_eq!(
        made.info.origin.managed.as_ref().unwrap().base_commit,
        record.base_commit
    );
    assert_eq!(f.store.messages(&record.session).unwrap().len(), 2);
    assert_eq!(
        f.store.saved_draft(&record.session).unwrap(),
        "Edited prompt 世界"
    );
    assert!(f
        .store
        .prompt_branch_archive(&record.session, 0)
        .unwrap()
        .text
        .contains(&record.base_commit));
    f.store
        .save_draft(&record.session, "Keep newer branch work")
        .unwrap();
    f.store.create_prompt_branch("parent", &request).unwrap();
    assert_eq!(
        f.store.saved_draft(&record.session).unwrap(),
        "Keep newer branch work"
    );
    assert_eq!(
        f.store.saved_draft("parent").unwrap(),
        "Keep newer unsent parent draft 世界"
    );
    assert_eq!(
        f.store.worktree(&record.session).unwrap().unwrap().revision,
        record.revision
    );
}

#[test]
fn managed_targets_refuse_other_sources_and_existing_history_while_shared_side_archives_stay_compatible(
) {
    let f = Fixture::new();
    let record = managed_target(&f);
    f.store
        .create_workspace_session(
            "other",
            &SessionWorkspace {
                kind: WorkspaceKind::Project,
                root: Some("C:/another-source".into()),
            },
        )
        .unwrap();
    assert!(f
        .store
        .prompt_branch_target("other", &record.session, record.revision)
        .unwrap_err()
        .contains("source folder"));
    f.store
        .commit_turn(
            &record.session,
            "Existing target task",
            "Retained target answer",
        )
        .unwrap();
    assert!(f
        .store
        .prompt_branch_target("parent", &record.session, record.revision)
        .unwrap_err()
        .contains("already has work"));
    let request = f.request();
    f.store.create_prompt_branch("parent", &request).unwrap();
    let raw: String = f
        .store
        .lock()
        .unwrap()
        .query_row(
            "SELECT data FROM prompt_branches WHERE session='edited-branch'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let mut old: serde_json::Value = serde_json::from_str(&raw).unwrap();
    old["request"]
        .as_object_mut()
        .unwrap()
        .remove("targetRevision");
    old["origin"].as_object_mut().unwrap().remove("managed");
    f.store
        .lock()
        .unwrap()
        .execute(
            "UPDATE prompt_branches SET data=?1 WHERE session='edited-branch'",
            [old.to_string()],
        )
        .unwrap();
    let saved = f.store.prompt_branch("edited-branch").unwrap().unwrap();
    assert_eq!(saved.request, request);
    assert!(saved.origin.managed.is_none());
    f.store.create_prompt_branch("parent", &request).unwrap();
    assert_eq!(f.store.messages(&record.session).unwrap().len(), 2);
}
