use super::experience::*;
use dolores_core::*;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
pub(super) struct Fixture {
    pub mode: &'static str,
}
pub(super) fn doc(command: &str) -> SkillDocument {
    SkillDocument{name:"fixture-check".into(),description:"Fixture check workflow".into(),text:format!("---\nname: fixture-check\ndescription: Fixture check workflow\n---\nCheck command: `{command}`\n")}
}
#[async_trait::async_trait]
impl ModelProvider for Fixture {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "trial.fixture",
            kind: "model",
            api_version: 1,
        }
    }
    async fn stream(
        &self,
        _: Vec<Message>,
        _: mpsc::Sender<String>,
        _: CancellationToken,
    ) -> Result<(), String> {
        unreachable!()
    }
    async fn tool_turn(
        &self,
        m: &[AgentMessage],
        _: &[ToolSpec],
        _: CancellationToken,
    ) -> Result<AgentTurn, String> {
        if self.mode == "hang" {
            std::future::pending::<()>().await;
        }
        let count = m.iter().filter(|m| m.role == "tool").count();
        let old = m[0].content.contains("node obsolete.cjs");
        let mut config = m
            .iter()
            .find(|m| m.role == "tool")
            .and_then(|m| serde_json::from_str::<serde_json::Value>(&m.content).ok())
            .unwrap_or_default();
        if config.is_object() {
            config["enabled"] = json!(true);
        }
        let call = match count {
            0 if self.mode == "claim" => None,
            0 => Some(("trial_file", json!({"path":"config.json"}))),
            1 => Some((
                "trial_file",
                json!({"path":if self.mode=="tamper"{"../evaluator"}else{"config.json"},"text":config.to_string()}),
            )),
            2 => Some((
                "trial_check",
                json!({"command":if old{"node obsolete.cjs"}else{"node verify.cjs"}}),
            )),
            _ => None,
        };
        Ok(AgentTurn {
            output_limit: self.mode == "limit",
            content: if call.is_none() {
                "Task successful".into()
            } else {
                String::new()
            },
            calls: call
                .map(|(name, args)| ToolCall {
                    id: format!("step{count}"),
                    name: name.into(),
                    arguments: args.to_string(),
                })
                .into_iter()
                .collect(),
            usage: None,
        })
    }
}
pub(super) fn trial() -> ExperienceTrial {
    ExperienceTrial {
        id: "trial".into(),
        session: "task".into(),
        revision: 0,
        created_at: 1,
        suite: EXPERIENCE_SUITE.into(),
        model: "fixture".into(),
        settings: RequestSettings {
            max_output_tokens: 1024,
            timeout_seconds: 30,
            ..Default::default()
        },
        source_revision: 1,
        baseline: doc("node obsolete.cjs"),
        candidate: doc("node verify.cjs"),
        status: "running".into(),
        results: vec![],
    }
}
#[tokio::test]
async fn files_checks_and_independent_case_qualify_but_claim_tamper_limit_do_not() {
    let cancel = CancellationToken::new();
    for case in 0..2 {
        let good = one(
            &Fixture { mode: "good" },
            &doc("node verify.cjs"),
            case,
            true,
            cancel.clone(),
        )
        .await;
        assert!(good.passed);
        assert!(good.evidence.unwrap().tools.len() >= 3);
        assert!(
            !one(
                &Fixture { mode: "good" },
                &doc("node obsolete.cjs"),
                case,
                false,
                cancel.clone()
            )
            .await
            .passed
        );
    }
    for mode in ["claim", "tamper", "limit"] {
        assert!(
            !one(
                &Fixture { mode },
                &doc("node verify.cjs"),
                0,
                true,
                cancel.clone()
            )
            .await
            .passed
        );
    }
}
#[tokio::test]
async fn stopped_and_failed_save_preserve_baseline_and_no_qualification() {
    let store = Arc::new(
        dolores_store_sqlite::SqliteStore::open(std::path::Path::new(":memory:")).unwrap(),
    );
    store.create("task").unwrap();
    let run = store.create_experience_trial(&trial()).unwrap();
    let cancel = CancellationToken::new();
    cancel.cancel();
    let (tx, mut rx) = mpsc::channel(8);
    execute(
        store.clone(),
        Arc::new(Fixture { mode: "hang" }),
        run,
        cancel,
        tx,
        1,
    )
    .await;
    let saved = store.experience_trials("task").unwrap();
    assert_eq!(saved[0].status, "stopped");
    assert!(!saved[0].improved());
    assert_eq!(saved[0].baseline, doc("node obsolete.cjs"));
    while rx.recv().await.is_some() {}
}
