use super::*;
use dolores_core::history_search::*;
fn store() -> SqliteStore {
    SqliteStore::open(Path::new(":memory:")).unwrap()
}
fn query(text: &str) -> Query {
    Query {
        filter: Filter {
            text: text.into(),
            ..Default::default()
        },
        cursor: None,
    }
}
#[test]
fn literal_multilingual_message_search_indexes_atomic_turns_and_deletion() {
    let s = store();
    s.create("one").unwrap();
    s.create("other").unwrap();
    s.commit_turn(
        "one",
        "Find special punctuation \" OR * near ()",
        "检查中文上下文预算 世界",
    )
    .unwrap();
    s.commit_turn("other", "Unrelated saved prompt", "Other answer")
        .unwrap();
    for text in [
        "中文上下文",
        "预算",
        "界",
        "punctuation \" OR *",
        "near ()",
        "上下文预算",
    ] {
        let found = s.search_history(&query(text)).unwrap();
        assert_eq!(found.items.len(), 1, "{text}");
        assert_eq!(found.items[0].session.id, "one");
        assert!(found.items[0].excerpt.as_ref().unwrap().text.contains(text));
    }
    assert!(s
        .search_history(&query("nothere OR other"))
        .unwrap()
        .items
        .is_empty());
    s.lock()
        .unwrap()
        .execute(
            "UPDATE sessions SET title='New saved title' WHERE id='other'",
            [],
        )
        .unwrap();
    assert_eq!(
        s.search_history(&query("saved title")).unwrap().items[0]
            .session
            .id,
        "other"
    );
    s.delete("one").unwrap();
    assert!(s
        .search_history(&query("中文上下文"))
        .unwrap()
        .items
        .is_empty());
    s.rebuild_history_search().unwrap();
    assert_eq!(
        s.search_history(&query("saved title")).unwrap().items.len(),
        1
    );
}
#[test]
fn archive_pin_stale_revision_and_storage_failure_preserve_task_data() {
    let s = store();
    s.create("one").unwrap();
    s.commit_turn("one", "Original question", "Original answer")
        .unwrap();
    s.save_draft("one", "Keep my draft 世界").unwrap();
    let goal = s
        .edit_goal(
            "one",
            &dolores_core::goal::GoalEdit::Set {
                revision: None,
                objective: "Keep this goal".into(),
                criteria: vec!["Verify the saved task".into()],
                constraints: "Preserve input".into(),
            },
            dolores_core::TaskBudget::default(),
        )
        .unwrap();
    s.edit_task_inputs(
        "one",
        &dolores_core::harness_task::InputEdit::Enqueue {
            id: "pending-followup".into(),
            text: "Keep queued instruction".into(),
            mode: dolores_core::harness_task::InputMode::Queue,
        },
    )
    .unwrap();
    let inputs = s.task_inputs("one").unwrap();
    let org = Organization {
        revision: 0,
        pinned: true,
        archived: true,
    };
    s.lock().unwrap().execute_batch("CREATE TRIGGER fail_organization BEFORE INSERT ON history_organization BEGIN SELECT RAISE(ABORT,'fixture disk unavailable'); END;").unwrap();
    assert!(s.organize_history("one", &org).is_err());
    assert_eq!(
        s.history_organization("one").unwrap(),
        Organization::default()
    );
    s.lock()
        .unwrap()
        .execute_batch("DROP TRIGGER fail_organization")
        .unwrap();
    let saved = s.organize_history("one", &org).unwrap();
    assert_eq!(saved.revision, 1);
    assert!(s.sessions_page(None, false, 50).unwrap().items.is_empty());
    assert!(s
        .search_history(&query("Original"))
        .unwrap()
        .items
        .is_empty());
    let mut archived = query("Original");
    archived.filter.archive = ArchiveFilter::Archived;
    archived.filter.pinned = Some(true);
    assert_eq!(
        s.search_history(&archived).unwrap().items[0].session.id,
        "one"
    );
    assert!(s
        .organize_history("one", &org)
        .unwrap_err()
        .contains("changed"));
    assert_eq!(s.saved_draft("one").unwrap(), "Keep my draft 世界");
    assert_eq!(s.goal("one").unwrap(), Some(goal));
    assert_eq!(s.task_inputs("one").unwrap(), inputs);
    assert_eq!(
        s.messages_page("one", None, false, 80).unwrap().items.len(),
        2
    );
    s.organize_history(
        "one",
        &Organization {
            revision: 1,
            pinned: true,
            archived: false,
        },
    )
    .unwrap();
    assert_eq!(s.sessions_page(None, false, 50).unwrap().items.len(), 1);
}
#[test]
fn pinned_tied_pages_and_filter_bound_cursors_cover_all_history() {
    let s = store();
    for n in 0..111 {
        let id = format!("task-{n:03}");
        s.create(&id).unwrap();
        s.lock()
            .unwrap()
            .execute(
                "UPDATE sessions SET title='Common needle',updated_at=7 WHERE id=?1",
                [&id],
            )
            .unwrap();
        if n % 3 == 0 {
            s.organize_history(
                &id,
                &Organization {
                    pinned: true,
                    ..Default::default()
                },
            )
            .unwrap();
        }
    }
    let mut q = query("needle");
    let mut ids = vec![];
    loop {
        let page = s.search_history(&q).unwrap();
        if ids.is_empty() {
            assert!(page.items[0].organization.pinned);
        }
        ids.extend(page.items.into_iter().map(|v| v.session.id));
        q.cursor = page.next_cursor;
        if q.cursor.is_none() {
            break;
        }
    }
    let unique: std::collections::BTreeSet<_> = ids.iter().collect();
    assert_eq!(unique.len(), 111);
    assert_eq!(ids.len(), 111);
    let mut q = query("needle");
    q.cursor = s.search_history(&q).unwrap().next_cursor;
    q.filter.archive = ArchiveFilter::All;
    assert!(s
        .search_history(&q)
        .unwrap_err()
        .contains("filters changed"));
    let mut bad = query("世".repeat(100).as_str());
    assert!(s.search_history(&bad).is_err());
    bad.filter.text = "Valid".into();
    assert!(s.search_history(&bad).is_ok());
}
#[test]
fn old_profile_backfill_and_failed_rebuild_keep_canonical_history() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("old.db");
    let s = SqliteStore::open(&path).unwrap();
    s.create("one").unwrap();
    s.commit_turn("one", "Old searchable prompt", "Retained answer 世界")
        .unwrap();
    drop(s);
    let c = Connection::open(&path).unwrap();
    c.execute_batch("DROP TRIGGER history_title_insert;DROP TRIGGER history_title_delete;DROP TRIGGER history_title_update;DROP TRIGGER history_message_insert;DROP TRIGGER history_message_delete;DROP TRIGGER history_message_update;DROP TABLE history_titles;DROP TABLE history_messages;DROP TABLE history_organization;PRAGMA user_version=46;").unwrap();
    drop(c);
    let s = SqliteStore::open(&path).unwrap();
    assert_eq!(
        s.search_history(&query("searchable")).unwrap().items.len(),
        1
    );
    // A competing SQLite writer models an interrupted/unavailable rebuild before
    // publication. Neither canonical history nor the previous index is modified.
    s.lock()
        .unwrap()
        .busy_timeout(std::time::Duration::ZERO)
        .unwrap();
    let other = Connection::open(&path).unwrap();
    other.execute_batch("BEGIN IMMEDIATE").unwrap();
    assert!(s.rebuild_history_search().is_err());
    other.execute_batch("ROLLBACK").unwrap();
    assert!(history_search::rebuild_limited(&s, std::time::Duration::ZERO).is_err());
    assert!(
        history_search::search_limited(&s, &query("searchable"), std::time::Duration::ZERO)
            .unwrap_err()
            .contains("one-second allowance")
    );
    assert_eq!(
        s.search_history(&query("searchable")).unwrap().items.len(),
        1
    );
    s.rebuild_history_search().unwrap();
    assert_eq!(
        s.messages_page("one", None, false, 80).unwrap().items.len(),
        2
    );
}

#[test]
fn indexed_corpus_search_and_short_unicode_scan_stay_bounded() {
    let s = store();
    s.create("corpus").unwrap();
    for n in 0..1000 {
        s.commit_turn(
            "corpus",
            &format!("Public prompt {n} {}", "context ".repeat(64)),
            &format!("Saved answer {n} 中文预算 {}", "evidence ".repeat(64)),
        )
        .unwrap();
    }
    let start = std::time::Instant::now();
    for text in ["Saved answer 997", "预算", "文"] {
        assert_eq!(s.search_history(&query(text)).unwrap().items.len(), 1);
    }
    println!(
        "1000 public turns indexed/short Unicode searches: {:?}",
        start.elapsed()
    );
    let before = s
        .messages_page("corpus", None, false, 80)
        .unwrap()
        .items
        .len();
    assert!(history_search::rebuild_limited(&s, std::time::Duration::ZERO).is_err());
    assert_eq!(
        s.messages_page("corpus", None, false, 80)
            .unwrap()
            .items
            .len(),
        before
    );
    assert_eq!(
        s.search_history(&query("Saved answer 997"))
            .unwrap()
            .items
            .len(),
        1
    );
}

#[test]
fn workspace_filters_and_failed_turn_save_leave_index_and_other_chats_consistent() {
    use dolores_core::{ContextSummary, SessionWorkspace, TurnMetadata, WorkspaceKind};
    let s = store();
    for (id, kind, root) in [
        ("project", WorkspaceKind::Project, Some("C:/public-project")),
        ("side", WorkspaceKind::Side, None),
    ] {
        s.create_workspace_session(
            id,
            &SessionWorkspace {
                kind,
                root: root.map(str::to_owned),
            },
        )
        .unwrap();
        s.commit_turn(id, "Common phrase", "Saved answer").unwrap();
    }
    let mut q = query("Common phrase");
    q.filter.kind = Some("project".into());
    q.filter.root = Some("C:/public-project".into());
    assert_eq!(s.search_history(&q).unwrap().items[0].session.id, "project");
    q.filter.root = Some("C:/unrelated".into());
    assert!(s.search_history(&q).unwrap().items.is_empty());
    s.lock().unwrap().execute_batch("CREATE TRIGGER failed_indexed_turn BEFORE INSERT ON turn_metadata BEGIN SELECT RAISE(ABORT,'fixture storage failure'); END;").unwrap();
    let metadata = TurnMetadata {
        model: "fixture".into(),
        usage: None,
        context: ContextSummary::from_messages(&[], None),
        paused: None,
        request_settings: None,
        agent: None,
    };
    assert!(s
        .commit_turn_metadata(
            "project",
            "Never published needle",
            "Uncommitted answer",
            &metadata
        )
        .is_err());
    assert!(s
        .search_history(&query("Never published needle"))
        .unwrap()
        .items
        .is_empty());
    assert_eq!(
        s.search_history(&query("Common phrase"))
            .unwrap()
            .items
            .len(),
        2
    );
    assert_eq!(
        s.messages_page("project", None, false, 80)
            .unwrap()
            .items
            .len(),
        2
    );
}
