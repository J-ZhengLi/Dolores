use super::*;
fn fixtures() -> (tempfile::TempDir, Bundle) {
    let temp = tempfile::tempdir().unwrap();
    for name in [APP, LIBRARY, LAUNCHER] {
        fs::write(temp.path().join(name), name).unwrap();
    }
    let b = manifest(temp.path()).unwrap();
    (temp, b)
}
fn build(b: Bundle) -> Build {
    let mut c = b.clone();
    let dll = c.files.iter_mut().find(|f| f.path == LIBRARY).unwrap();
    dll.id = identity(b"candidate");
    dll.bytes = 9;
    c.id = identity(&serde_json::to_vec(&c.files).unwrap());
    Build {
        id: uuid::Uuid::new_v4().to_string(),
        session: uuid::Uuid::new_v4().to_string(),
        repair_id: uuid::Uuid::new_v4().to_string(),
        revision: 2,
        source_id: identity(b"source"),
        candidate_source_id: identity(b"candidate-source"),
        evaluation_id: uuid::Uuid::new_v4().to_string(),
        cargo_id: identity(b"cargo"),
        status: "ready".into(),
        note: "Ready build; separate installation review.".into(),
        previous: Some(b),
        candidate: Some(c),
        schema: 33,
    }
}
#[test]
fn copied_complete_bundle_rejects_tampering_and_unreviewed_members() {
    let (root, b) = fixtures();
    let dest = tempfile::tempdir().unwrap();
    let copied = dest.path().join("retained");
    copy_bundle(root.path(), &copied, &b).unwrap();
    verify(&copied, &b).unwrap();
    fs::write(copied.join("unreviewed.dll"), "extra").unwrap();
    assert!(verify(&copied, &b).unwrap_err().contains("changed"));
    fs::remove_file(copied.join("unreviewed.dll")).unwrap();
    fs::write(copied.join(LIBRARY), "changed").unwrap();
    assert!(verify(&copied, &b).is_err());
    verify(root.path(), &b).unwrap();
}
#[test]
fn staged_library_is_flushed_with_a_writable_handle_and_old_bundle_is_preserved() {
    let (root, original) = fixtures();
    let target = tempfile::tempdir().unwrap();
    let candidate = target.path().join("candidate");
    copy_bundle(root.path(), &candidate, &original).unwrap();
    write_library(&candidate, b"qualified candidate DLL").unwrap();
    assert_eq!(
        file_hash(&candidate.join(LIBRARY)).unwrap(),
        identity(b"qualified candidate DLL")
    );
    verify(root.path(), &original).unwrap();
    assert!(write_library(&target.path().join("missing"), b"replacement").is_err());
}
#[test]
fn ready_build_can_change_only_the_rust_dll() {
    let (_, b) = fixtures();
    let mut build = build(b);
    build.validate().unwrap();
    let c = build.candidate.as_mut().unwrap();
    c.files.iter_mut().find(|f| f.path == LAUNCHER).unwrap().id = identity(b"evil");
    c.id = identity(&serde_json::to_vec(&c.files).unwrap());
    assert!(build.validate().unwrap_err().contains("Only"));
}
#[test]
fn malformed_paths_and_duplicate_members_are_refused() {
    let (_, mut b) = fixtures();
    b.files[0].path = "../outside".into();
    b.id = identity(&serde_json::to_vec(&b.files).unwrap());
    assert!(b.validate().is_err());
    b.files[0].path = "C:/outside".into();
    b.id = identity(&serde_json::to_vec(&b.files).unwrap());
    assert!(b.validate().is_err());
}
#[test]
fn snapshot_keeps_wal_history_and_drafts_without_rewinding_later_history() {
    let p = tempfile::tempdir().unwrap();
    let db = rusqlite::Connection::open(p.path().join("dolores.db")).unwrap();
    db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA user_version=33; CREATE TABLE drafts(session TEXT, draft TEXT); INSERT INTO drafts VALUES('chat','unfinished work');").unwrap();
    let before = database_identity(&p.path().join("dolores.db")).unwrap();
    let dest = p.path().join("before.sqlite");
    let hash = snapshot_database(p.path(), &dest).unwrap();
    assert_eq!(database_identity(&dest).unwrap(), before);
    assert_eq!(file_hash(&dest).unwrap(), hash);
    assert!(snapshot_database(p.path(), &dest).is_err());
    db.execute(
        "INSERT INTO drafts VALUES('later','work after installation')",
        [],
    )
    .unwrap();
    assert_ne!(
        database_identity(&p.path().join("dolores.db")).unwrap(),
        before
    );
    assert_eq!(database_identity(&dest).unwrap(), before);
    assert_eq!(database_schema(&dest).unwrap(), 33);
}
#[test]
fn durable_receipt_replacement_and_invalid_json_preserve_evidence() {
    let p = tempfile::tempdir().unwrap();
    let path = p.path().join("receipt.json");
    save(&path, &"started").unwrap();
    save(&path, &"failed").unwrap();
    assert_eq!(json::<String>(&path).unwrap(), "failed");
    fs::write(&path, b"{").unwrap();
    assert!(json::<String>(&path).unwrap_err().contains("do not replay"));
    assert_eq!(fs::read(path).unwrap(), b"{");
}
#[cfg(windows)]
#[test]
fn creation_identity_protects_against_a_stale_pid() {
    let me = process::identity(std::process::id()).unwrap().unwrap();
    let mut stale = me.clone();
    stale.created += 1;
    assert!(process::stop_failed_startup(&stale)
        .unwrap_err()
        .contains("changed"));
    assert_eq!(process::identity(me.pid).unwrap(), Some(me));
}
#[test]
fn interrupted_owner_and_changed_intent_cannot_be_reclaimed() {
    let (bundle, b) = fixtures();
    let profile = tempfile::tempdir().unwrap();
    let build = build(b);
    let root = profile
        .path()
        .join("repairs")
        .join(format!("build-{}", build.id));
    fs::create_dir_all(&root).unwrap();
    let p = Process {
        pid: 1,
        created: 1,
        executable: bundle.path().join(APP),
    };
    let job = Handoff {
        id: uuid::Uuid::new_v4().to_string(),
        expected_source: build.candidate_source_id.clone(),
        build,
        profile: profile.path().to_path_buf(),
        build_root: root,
        old: p.clone(),
        child: None,
        helper: None,
        operation: "install".into(),
        stage: "waiting".into(),
        note: String::new(),
        database_id: None,
        backup_id: None,
    };
    let receipt = profile.path().join("handoff.json");
    save(&receipt, &job).unwrap();
    let mut changed = job.clone();
    changed.note = "changed after the first read".into();
    save(&receipt, &changed).unwrap();
    assert!(claim_handoff(&receipt, &job, p.clone()).is_err());
    assert_eq!(json::<Handoff>(&receipt).unwrap(), changed);
    save(&receipt, &job).unwrap();
    let claimed = claim_handoff(&receipt, &job, p.clone()).unwrap();
    assert_eq!(claimed.stage, "waiting");
    assert!(claim_handoff(&receipt, &claimed, p).is_err());
    assert_eq!(json::<Handoff>(&receipt).unwrap(), claimed);
}
