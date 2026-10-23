use super::*;
use dolores_core::{
    harness_task::InputMode,
    side_conversations::{Create, Return},
    SessionWorkspace, WorkspaceKind,
};
use dolores_store_sqlite::SqliteStore;

#[test]
fn selected_context_and_exact_return_receipts_survive_lost_replies_without_requeueing() {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(SqliteStore::open(&dir.path().join("state.db")).unwrap());
    store
        .create_workspace_session(
            "parent",
            &SessionWorkspace {
                kind: WorkspaceKind::Side,
                root: None,
            },
        )
        .unwrap();
    store
        .commit_turn(
            "parent",
            "Original task",
            "Visible **excerpt** and unshared parent tail",
        )
        .unwrap();
    let parent_message = store
        .messages_page("parent", None, false, 20)
        .unwrap()
        .items[1]
        .id;
    let engine = Engine::new(
        store.clone(),
        Arc::new(crate::connection::testing::MemoryCredentials::default()),
    )
    .unwrap();
    let call = |session: &str, request| {
        engine.call(Command::SideConversation {
            session: session.into(),
            request,
        })
    };
    let quote = store
        .side_quote("parent", parent_message, "excerpt")
        .unwrap();
    call(
        "parent",
        Request::Create {
            request: Create {
                id: "child".into(),
                quote,
                question: "Explain it".into(),
            },
        },
    )
    .unwrap();
    let input = vec![dolores_core::Message {
        role: dolores_core::Role::User,
        content: "Explain it".into(),
        parts: vec![],
    }];
    let prepared =
        dolores_core::planning::prepare(input, &store.planning("child").unwrap()).unwrap();
    let context = serde_json::to_string(&prepared).unwrap();
    assert!(context.contains("excerpt"));
    assert!(!context.contains("unshared parent tail"));
    assert!(store.runs("child").unwrap().is_empty());
    store
        .commit_turn(
            "child",
            "Question",
            "Selected conclusion and unshared child tail",
        )
        .unwrap();
    let message = store.messages_page("child", None, false, 20).unwrap().items[1].id;
    let request = Return {
        id: "returned-once".into(),
        quote: store
            .side_quote("child", message, "Selected conclusion")
            .unwrap(),
        mode: InputMode::Queue,
    };
    assert!(call(
        "child",
        Request::PreviewReturn {
            message,
            excerpt: "\u{0001}".repeat(4096)
        }
    )
    .unwrap_err()
    .contains("16 KiB"));
    assert!(call(
        "child",
        Request::InspectReturn {
            request: request.clone()
        }
    )
    .unwrap()
    .is_null());
    call(
        "child",
        Request::BringBack {
            request: request.clone(),
        },
    )
    .unwrap();
    assert!(call(
        "child",
        Request::InspectReturn {
            request: request.clone()
        }
    )
    .unwrap()
    .is_object());
    let mut different = request.clone();
    different.quote.excerpt = "Different selected text".into();
    assert!(call(
        "child",
        Request::InspectReturn {
            request: different.clone()
        }
    )
    .unwrap()
    .is_null());
    assert!(call("child", Request::BringBack { request: different }).is_err());
    store.delete("parent").unwrap();
    let saved = call("child", Request::InspectReturn { request }).unwrap();
    assert_eq!(saved["parentAvailable"], false);
    assert_eq!(saved["origin"]["returns"].as_array().unwrap().len(), 1);
    assert_eq!(store.saved_draft("child").unwrap(), "Explain it");
}
