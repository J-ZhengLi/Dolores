use super::*;
use crate::approval::PendingReply;
use dolores_core::attention::{Kind, Request};
impl Engine {
    pub(super) fn attention_call(
        &self,
        request: Request,
        active: &run_journal::RunCoordinator,
    ) -> Result<Value, String> {
        let mut value = self.store.attention(&request)?;
        if let Request::Resolve { .. } = request {
            let session = value["notice"]["session"]
                .as_str()
                .ok_or("Retained attention target is invalid.")?;
            let kind: Kind = serde_json::from_value(value["notice"]["kind"].clone())
                .map_err(|_| "Attention kind is invalid.")?;
            let live = if value["sameDecision"] == true {
                active
                    .client_id(session)
                    .and_then(|id| active.get(id))
                    .is_some_and(|run| {
                        run.approvals.lock().ok().is_some_and(|p| {
                            p.as_ref().is_some_and(|p| {
                                matches!(
                                    (&p.reply, kind),
                                    (PendingReply::Tool(_), Kind::Approval)
                                        | (PendingReply::Question { .. }, Kind::Question)
                                )
                            })
                        })
                    })
            } else {
                false
            };
            value["liveDecision"] = json!(live);
            value["navigationOnly"] = json!(true);
        }
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::RunState;
    #[test]
    fn click_resolution_targets_only_live_question_and_cannot_answer_or_restart_it() {
        let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
        store.create("task").unwrap();
        store.save_draft("task", "Keep draft").unwrap();
        let run = "11111111-1111-4111-8111-111111111111";
        store.begin_run(&serde_json::from_value(json!({"id":run,"thread":"task","model":"fixture","settings":RequestSettings::default(),"input":"Private question","state":"prepared","sequence":0,"createdAt":1,"build":"fixture","tools":[]})).unwrap()).unwrap();
        store
            .append_run_event(run, 0, Some(RunState::Running), "started", &json!({}))
            .unwrap();
        store
            .append_run_event(
                run,
                1,
                Some(RunState::WaitingForApproval),
                "questionAsked",
                &json!({"callId":"original"}),
            )
            .unwrap();
        let engine = Engine::new(
            store.clone(),
            Arc::new(crate::connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        let (send, mut receive) = tokio::sync::oneshot::channel();
        let pending = Arc::new(Mutex::new(Some(crate::approval::PendingApproval {
            call_id: "current-question".into(),
            reply: PendingReply::Question {
                questions: crate::questions::QuestionRequest { questions: vec![] },
                reply: send,
            },
        })));
        engine
            .active
            .lock()
            .unwrap()
            .reserve(Run {
                thread: Some("task".into()),
                id: 720,
                cancel: CancellationToken::new(),
                events: tokio::sync::mpsc::channel(1).1,
                approvals: pending.clone(),
            })
            .unwrap();
        let call = |request| engine.call(Command::Attention { request });
        let page = call(Request::Poll { cursor: None }).unwrap();
        let id = page["items"][0]["id"].as_str().unwrap().to_owned();
        let current = call(Request::Resolve { id: id.clone() }).unwrap();
        assert_eq!(current["liveDecision"], true);
        assert_eq!(current["navigationOnly"], true);
        assert!(receive.try_recv().is_err());
        assert!(pending.lock().unwrap().is_some());
        store
            .append_run_event(
                run,
                2,
                Some(RunState::Running),
                "questionAnswered",
                &json!({}),
            )
            .unwrap();
        assert_eq!(
            call(Request::Resolve { id: id.clone() }).unwrap()["liveDecision"],
            false
        );
        // A newer pending slot cannot revive an older decision notice.
        store
            .append_run_event(
                run,
                3,
                Some(RunState::WaitingForApproval),
                "approvalRequested",
                &json!({"callId":"new"}),
            )
            .unwrap();
        assert_eq!(
            call(Request::Resolve { id }).unwrap()["liveDecision"],
            false
        );
        assert_eq!(store.saved_draft("task").unwrap(), "Keep draft");
        assert!(receive.try_recv().is_err());
        engine.active.lock().unwrap().remove(720);
    }
}
