use super::*;
use dolores_core::{prompt_branches::Create, SessionStore, SessionWorkspace, WorkspaceKind};
use dolores_store_sqlite::SqliteStore;

#[test]
fn branch_host_checks_current_folder_but_exact_saved_retry_survives_missing_parent_and_folder() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("project");
    std::fs::create_dir(&root).unwrap();
    std::fs::write(root.join("a.txt"), "Current changed files").unwrap();
    let store = Arc::new(SqliteStore::open(&dir.path().join("state.db")).unwrap());
    store
        .create_workspace_session(
            "parent",
            &SessionWorkspace {
                kind: WorkspaceKind::Project,
                root: Some(root.to_string_lossy().into()),
            },
        )
        .unwrap();
    store
        .commit_turn("parent", "Original prompt", "Historical answer")
        .unwrap();
    store.save_draft("parent", "Keep newer draft").unwrap();
    let message = store
        .messages_page("parent", None, false, 80)
        .unwrap()
        .items[0]
        .id;
    let basis = store
        .preview_prompt_branch("parent", message)
        .unwrap()
        .basis;
    let engine = Engine::new(
        store.clone(),
        Arc::new(crate::connection::testing::MemoryCredentials::default()),
    )
    .unwrap();
    let call = |owner: &str, request| {
        engine.call(Command::PromptBranch {
            session: owner.into(),
            request,
        })
    };
    let request = Create {
        id: "branch".into(),
        basis,
        edited: "Edited request".into(),
        include_attachments: false,
        include_context: false,
        folder: Folder::Shared,
        target_revision: None,
    };
    let parked = dir.path().join("parked-project");
    std::fs::rename(&root, &parked).unwrap();
    assert!(call(
        "parent",
        Request::Create {
            request: request.clone()
        }
    )
    .unwrap_err()
    .contains("No working folder"));
    assert!(store.prompt_branch("branch").unwrap().is_none());
    assert_eq!(store.saved_draft("parent").unwrap(), "Keep newer draft");
    std::fs::rename(&parked, &root).unwrap();
    call(
        "parent",
        Request::Create {
            request: request.clone(),
        },
    )
    .unwrap();
    assert!(store.messages("branch").unwrap().is_empty());
    assert!(store.runs("branch").unwrap().is_empty());
    assert_eq!(
        std::fs::read_to_string(root.join("a.txt")).unwrap(),
        "Current changed files"
    );
    store.save_draft("branch", "Newer branch draft").unwrap();
    store.delete("parent").unwrap();
    std::fs::rename(&root, &parked).unwrap();
    let result = call(
        "parent",
        Request::Create {
            request: request.clone(),
        },
    )
    .unwrap();
    assert_eq!(result["info"]["parentAvailable"], false);
    assert_eq!(store.saved_draft("branch").unwrap(), "Newer branch draft");
    assert!(
        call("branch", Request::Archive { offset: 0 }).unwrap()["text"]
            .as_str()
            .unwrap()
            .contains("Historical answer")
    );
    let changed = Create {
        edited: "Different requested edit".into(),
        ..request
    };
    assert!(call("parent", Request::Create { request: changed })
        .unwrap_err()
        .contains("identity already"));
}
