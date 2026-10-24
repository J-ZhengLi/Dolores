use super::*;
use dolores_core::history_search::{Organization, Query};

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "camelCase", deny_unknown_fields)]
pub(super) enum Request {
    Search {
        query: Query,
    },
    Organization {
        session: String,
    },
    Organize {
        session: String,
        organization: Organization,
    },
    Rebuild,
}
impl Engine {
    pub(super) fn history_call(&self, request: Request) -> Result<Value, String> {
        match request {
            Request::Search { query } => Ok(json!(self.store.search_history(&query)?)),
            Request::Organization { session } => {
                Ok(json!(self.store.history_organization(&session)?))
            }
            // Organization deliberately has no cancellation, dispatch or goal edit.
            Request::Organize {
                session,
                organization,
            } => Ok(json!(self
                .store
                .organize_history(&session, &organization)?)),
            Request::Rebuild => {
                self.store.rebuild_history_search()?;
                Ok(Value::Null)
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    #[test]
    fn search_and_archive_do_not_start_or_cancel_an_owned_run() {
        let store = Arc::new(SqliteStore::open(Path::new(":memory:")).unwrap());
        store.create("task").unwrap();
        store
            .commit_turn("task", "Old searchable prompt", "Saved answer")
            .unwrap();
        store.save_draft("task", "Keep private draft").unwrap();
        let engine = Engine::new(
            store.clone(),
            Arc::new(crate::connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        engine
            .active
            .lock()
            .unwrap()
            .reserve(crate::Run {
                thread: Some("task".into()),
                id: 740,
                cancel: CancellationToken::new(),
                events: tokio::sync::mpsc::channel(1).1,
                approvals: Arc::new(Mutex::new(None)),
            })
            .unwrap();
        let call = |request| engine.call(Command::History { request });
        let result = call(Request::Search {
            query: Query {
                filter: dolores_core::history_search::Filter {
                    text: "searchable".into(),
                    ..Default::default()
                },
                cursor: None,
            },
        })
        .unwrap();
        assert_eq!(result["items"][0]["session"]["id"], "task");
        call(Request::Organize {
            session: "task".into(),
            organization: Organization {
                archived: true,
                ..Default::default()
            },
        })
        .unwrap();
        assert!(engine.active.lock().unwrap().has_session("task"));
        assert_eq!(store.saved_draft("task").unwrap(), "Keep private draft");
        assert_eq!(
            store
                .messages_page("task", None, false, 80)
                .unwrap()
                .items
                .len(),
            2
        );
        assert!(store
            .sessions_page(None, false, 50)
            .unwrap()
            .items
            .is_empty());
        assert!(call(Request::Rebuild)
            .unwrap_err()
            .contains("Finish or stop running tasks"));
        assert!(engine.active.lock().unwrap().has_session("task"));
        engine.active.lock().unwrap().remove(740);
        call(Request::Rebuild).unwrap();
    }
}
