use super::*;
use crate::{connection::testing::MemoryCredentials, Command};
use dolores_core::{SessionStore, SessionWorkspace, WorkspaceKind};
use dolores_store_sqlite::SqliteStore;
use std::sync::Arc;
fn setup(store: Arc<SqliteStore>, root: &std::path::Path) -> Engine {
    if store.workspace("A").is_err() {
        store
            .create_workspace_session(
                "A",
                &SessionWorkspace {
                    kind: WorkspaceKind::Project,
                    root: Some(crate::workspace::canonical_folder(root).unwrap()),
                },
            )
            .unwrap();
    }
    Engine::new(store, Arc::new(MemoryCredentials::default())).unwrap()
}
#[test]
fn language_workspace_preview_apply_undo_is_atomic_and_refuses_stale_outside_paths() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("a.ts"), "const old = 1;\n").unwrap();
    std::fs::write(root.path().join("b.ts"), "old();\n").unwrap();
    let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
    let e = setup(store, root.path());
    let project = call(&e, "A", json!({"action":"workspace"})).unwrap()["project"].clone();
    let source = doc(&e, &project, "a.ts");
    let uri = |name: &str| {
        url::Url::from_file_path(root.path().join(name))
            .unwrap()
            .to_string()
    };
    let edits = json!({"changes":{uri("a.ts"):[{"range":{"start":{"line":0,"character":6},"end":{"line":0,"character":9}},"newText":"newName"}],uri("b.ts"):[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":3}},"newText":"newName"}]}});
    let language = |r: Value| {
        e.call(
            serde_json::from_value(json!({"command":"languageEdits","session":"A","request":r}))
                .unwrap(),
        )
    };
    let preview = language(
        json!({"action":"preview","document":source["document"],"version":0,"edit":edits}),
    )
    .unwrap();
    assert_eq!(doc(&e, &project, "a.ts")["text"], "const old = 1;\n");
    std::fs::write(root.path().join("b.ts"), "external();\n").unwrap();
    assert!(language(json!({"action":"apply","token":preview["token"]})).is_err());
    assert_eq!(doc(&e, &project, "a.ts")["text"], "const old = 1;\n");
    std::fs::write(root.path().join("b.ts"), "old();\n").unwrap();
    let result = language(json!({"action":"apply","token":preview["token"]})).unwrap();
    assert_eq!(result["documents"][1]["text"], "newName();\n");
    assert_eq!(
        std::fs::read_to_string(root.path().join("b.ts")).unwrap(),
        "old();\n"
    );
    assert!(language(json!({"action":"apply","token":preview["token"]})).is_err());
    let undone = language(json!({"action":"undo","token":preview["token"]})).unwrap();
    assert_eq!(undone["documents"][1]["text"], "old();\n");
    assert!(language(json!({"action":"undo","token":preview["token"]})).is_err());
    let outside = tempfile::tempdir().unwrap();
    let outside_uri = url::Url::from_file_path(outside.path().join("no.ts"))
        .unwrap()
        .to_string();
    let source = doc(&e, &project, "a.ts");
    assert!(language(json!({"action":"preview","document":source["document"],"version":source["version"],"edit":{"changes":{outside_uri:[]}}})).is_err());
    assert!(language(json!({"action":"preview","document":source["document"],"version":source["version"],"edit":{"documentChanges":[{"kind":"delete","uri":uri("a.ts")}]}})).is_err());
}
fn call(e: &Engine, session: &str, request: Value) -> Result<Value, String> {
    e.call(
        serde_json::from_value::<Command>(
            json!({"command":"editor","session":session,"request":request}),
        )
        .unwrap(),
    )
}
fn edit(
    e: &Engine,
    p: &Value,
    d: &Value,
    start: usize,
    end: usize,
    text: &str,
) -> Result<Value, String> {
    call(
        e,
        "A",
        json!({"action":"edit","project":p,"document":d["document"],"version":d["version"],"edits":[{"start":start,"end":end,"text":text}]}),
    )
}
fn doc(e: &Engine, p: &Value, path: &str) -> Value {
    call(e, "A", json!({"action":"open","project":p,"path":path})).unwrap()
}
#[test]
fn shared_document_deltas_are_unicode_safe_atomic_and_project_bound() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("a"), "a😀b\n").unwrap();
    let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
    let e = setup(store.clone(), root.path());
    let p = call(&e, "A", json!({"action":"workspace"})).unwrap()["project"].clone();
    let d = doc(&e, &p, "a");
    assert_eq!(doc(&e, &p, "a")["document"], d["document"]);
    assert!(e.editor_can_restart().is_ok());
    assert!(edit(&e, &p, &d, 2, 3, "x").is_err());
    assert_eq!(doc(&e, &p, "a")["text"], "a😀b\n");
    let ack = edit(&e, &p, &d, 1, 3, "世界").unwrap();
    assert_eq!(ack["version"], 1);
    assert!(e
        .editor_can_restart()
        .unwrap_err()
        .contains("Save or close"));
    assert!(ack.get("text").is_none());
    assert!(edit(&e, &p, &d, 0, 1, "stale").is_err());
    let d = doc(&e, &p, "a");
    assert!(edit(&e, &p, &d, 0, 0, &"x".repeat(65537)).is_err());
    assert_eq!(doc(&e, &p, "a")["text"], "a世界b\n");
    let b = tempfile::tempdir().unwrap();
    store
        .create_workspace_session(
            "B",
            &SessionWorkspace {
                kind: WorkspaceKind::Project,
                root: Some(crate::workspace::canonical_folder(b.path()).unwrap()),
            },
        )
        .unwrap();
    assert!(call(
        &e,
        "B",
        json!({"action":"save","project":p,"document":d["document"],"version":d["version"]})
    )
    .unwrap_err()
    .contains("project changed"));
    assert_eq!(
        std::fs::read_to_string(root.path().join("a")).unwrap(),
        "a😀b\n"
    );
}
#[test]
fn stale_save_comparison_rebase_and_deleted_file_save_as_keep_edits() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("a"), "original").unwrap();
    let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
    let e = setup(store, root.path());
    let p = call(&e, "A", json!({"action":"workspace"})).unwrap()["project"].clone();
    let d = doc(&e, &p, "a");
    edit(&e, &p, &d, 0, 8, "mine").unwrap();
    let d = doc(&e, &p, "a");
    std::fs::write(root.path().join("a"), "external").unwrap();
    assert!(call(
        &e,
        "A",
        json!({"action":"save","project":p,"document":d["document"],"version":d["version"]})
    )
    .is_err());
    assert_eq!(doc(&e, &p, "a")["text"], "mine");
    let compare = call(
        &e,
        "A",
        json!({"action":"compare","project":p,"document":d["document"],"version":d["version"]}),
    )
    .unwrap();
    assert_eq!(compare["disk"]["text"], "external");
    let d=call(&e,"A",json!({"action":"rebase","project":p,"document":d["document"],"version":d["version"],"revision":compare["disk"]["revision"]})).unwrap();
    call(
        &e,
        "A",
        json!({"action":"save","project":p,"document":d["document"],"version":d["version"]}),
    )
    .unwrap();
    assert_eq!(
        std::fs::read_to_string(root.path().join("a")).unwrap(),
        "mine"
    );
    let d = doc(&e, &p, "a");
    edit(&e, &p, &d, 0, 4, "retained").unwrap();
    let d = doc(&e, &p, "a");
    std::fs::remove_file(root.path().join("a")).unwrap();
    assert!(call(
        &e,
        "A",
        json!({"action":"save","project":p,"document":d["document"],"version":d["version"]})
    )
    .is_err());
    let saved=call(&e,"A",json!({"action":"saveAs","project":p,"document":d["document"],"version":d["version"],"path":"recovered"})).unwrap();
    assert_eq!(saved["dirty"], false);
    assert_eq!(
        std::fs::read_to_string(root.path().join("recovered")).unwrap(),
        "retained"
    );
}
#[test]
fn private_recovery_survives_restart_and_does_not_drop_unopened_recovery() {
    let root = tempfile::tempdir().unwrap();
    for name in ["a", "b"] {
        std::fs::write(root.path().join(name), "saved").unwrap();
    }
    let state = tempfile::tempdir().unwrap();
    let store = Arc::new(SqliteStore::open(&state.path().join("test.db")).unwrap());
    let e = setup(store.clone(), root.path());
    let p = call(&e, "A", json!({"action":"workspace"})).unwrap()["project"].clone();
    for name in ["a", "b"] {
        let d = doc(&e, &p, name);
        edit(&e, &p, &d, 0, 5, "private draft").unwrap();
    }
    call(&e, "A", json!({"action":"checkpoint","project":p})).unwrap();
    drop(e);
    drop(store);
    let store = Arc::new(SqliteStore::open(&state.path().join("test.db")).unwrap());
    let e = setup(store, root.path());
    assert_eq!(
        call(&e, "A", json!({"action":"workspace"})).unwrap()["recovery"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let a = call(&e, "A", json!({"action":"recover","project":p,"path":"a"})).unwrap();
    assert_eq!(a["text"], "private draft");
    call(&e, "A", json!({"action":"checkpoint","project":p})).unwrap();
    call(&e,"A",json!({"action":"close","project":p,"document":a["document"],"version":a["version"],"discard":true})).unwrap();
    let state = call(&e, "A", json!({"action":"workspace"})).unwrap();
    assert_eq!(state["recovery"].as_array().unwrap().len(), 1);
    assert_eq!(state["recovery"][0]["path"], "b");
    assert_eq!(
        std::fs::read_to_string(root.path().join("a")).unwrap(),
        "saved"
    );
}
#[test]
fn clean_agent_disk_changes_refresh_and_dirty_changes_require_review_before_attachment() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("a"), "saved").unwrap();
    let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
    let e = setup(store.clone(), root.path());
    let p = call(&e, "A", json!({"action":"workspace"})).unwrap()["project"].clone();
    let d = doc(&e, &p, "a");
    std::fs::write(root.path().join("a"), "agent saved").unwrap();
    let refreshed = call(
        &e,
        "A",
        json!({"action":"refresh","project":p,"document":d["document"],"version":d["version"]}),
    )
    .unwrap();
    assert_eq!(refreshed["text"], "agent saved");
    edit(&e, &p, &refreshed, 0, 11, "unsaved selection").unwrap();
    let d = doc(&e, &p, "a");
    std::fs::write(root.path().join("a"), "new disk").unwrap();
    let changed = call(
        &e,
        "A",
        json!({"action":"refresh","project":p,"document":d["document"],"version":d["version"]}),
    )
    .unwrap();
    assert_eq!(changed["changed"], true);
    assert_eq!(doc(&e, &p, "a")["text"], "unsaved selection");
    assert!(store.draft_attachments("A").unwrap().is_empty());
    assert!(call(&e,"A",json!({"action":"attach","project":p,"document":d["document"],"version":0,"start":0,"end":17,"target":"A"})).is_err());
    let parts=call(&e,"A",json!({"action":"attach","project":p,"document":d["document"],"version":d["version"],"start":0,"end":17,"target":"A"})).unwrap();
    let asset = store
        .attachment_data("A", parts[0]["digest"].as_str().unwrap())
        .unwrap();
    let text = String::from_utf8(asset.data).unwrap();
    assert!(text.contains("(unsaved)"));
    assert!(text.ends_with("unsaved selection"));
    assert!(text.contains(d["snapshot"]["revision"].as_str().unwrap()));
    edit(&e, &p, &d, 0, 17, "later edits").unwrap();
    assert!(String::from_utf8(
        store
            .attachment_data("A", parts[0]["digest"].as_str().unwrap())
            .unwrap()
            .data
    )
    .unwrap()
    .ends_with("unsaved selection"));
    assert_eq!(
        std::fs::read_to_string(root.path().join("a")).unwrap(),
        "new disk"
    );
}
