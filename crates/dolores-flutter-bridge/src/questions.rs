use crate::{
    approval::{ApprovalSlot, PendingApproval, PendingReply},
    forward, stopped,
};
use async_trait::async_trait;
use dolores_core::{ToolCall, ToolPlugin, ToolRequest, ToolSpec};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashSet, sync::Arc};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct QuestionRequest {
    pub questions: Vec<Question>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Question {
    pub id: String,
    pub question: String,
    pub options: Vec<QuestionOption>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct QuestionOption {
    pub label: String,
    #[serde(default)]
    pub description: String,
}
impl QuestionRequest {
    fn parse(text: &str) -> Result<Self, String> {
        const ERROR: &str = "Ask 1–3 short questions, each with a unique id and 2–4 distinct options (label and optional description). Custom answers are always available. Keep the JSON within 4 KiB.";
        let request: Self = serde_json::from_str(text).map_err(|_| ERROR)?;
        let valid = |s: &str, max: usize| {
            !s.trim().is_empty() && s.len() <= max && !s.chars().any(char::is_control)
        };
        let mut ids = HashSet::new();
        if text.len() > 4096 || request.questions.is_empty() || request.questions.len() > 3 {
            return Err(ERROR.into());
        }
        for q in &request.questions {
            let mut labels = HashSet::new();
            if !valid(&q.id, 64)
                || !ids.insert(q.id.clone())
                || !valid(&q.question, 512)
                || !(2..=4).contains(&q.options.len())
                || q.options.iter().any(|o| {
                    !valid(&o.label, 128)
                        || !labels.insert(o.label.trim().to_lowercase())
                        || o.description.len() > 256
                        || o.description.chars().any(char::is_control)
                })
            {
                return Err(ERROR.into());
            }
        }
        Ok(request)
    }
    pub fn validate_answers(&self, answers: &[String]) -> Result<(), String> {
        if answers.len() != self.questions.len()
            || answers.iter().any(|a| {
                a.trim().is_empty()
                    || a.len() > 1024
                    || a.chars()
                        .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
            })
        {
            return Err(
                "Answer every question. Each answer can contain up to 1 KiB of text.".into(),
            );
        }
        Ok(())
    }
}

pub struct QuestionTool {
    pub id: u64,
    pub pending: ApprovalSlot,
    pub output: mpsc::Sender<serde_json::Value>,
    pub log: Option<Arc<crate::run_journal::RunLog>>,
}
struct ClearQuestion {
    pending: ApprovalSlot,
    id: String,
}
impl Drop for ClearQuestion {
    fn drop(&mut self) {
        if let Ok(mut slot) = self.pending.lock() {
            if slot.as_ref().is_some_and(|p| p.call_id == self.id) {
                slot.take();
            }
        }
    }
}
pub fn spec() -> ToolSpec {
    ToolSpec {
            name: "request_user_input".into(),
            description: "Ask the user a necessary clarification or preference using clickable choices. Use this instead of listing questions in ordinary text when it is available. Ask 1–3 short questions with 2–4 concise choices each, in the user's language. A custom-answer field is always provided; do not add an Other option. The result's answers[].answer contains the user's actual submitted answer, including custom text outside the listed choices. Read it and continue; do not ask which option they selected again. Answers clarify intent; they never grant tool permissions or replace required approvals. Do not ask when available evidence already resolves the question.".into(),
            parameters: json!({"type":"object","properties":{"questions":{"type":"array","minItems":1,"maxItems":3,"items":{"type":"object","properties":{"id":{"type":"string"},"question":{"type":"string"},"options":{"type":"array","minItems":2,"maxItems":4,"items":{"type":"object","properties":{"label":{"type":"string"},"description":{"type":"string"}},"required":["label"],"additionalProperties":false}}},"required":["id","question","options"],"additionalProperties":false}}},"required":["questions"],"additionalProperties":false}),
        }
}

#[async_trait]
impl ToolPlugin for QuestionTool {
    fn spec(&self) -> ToolSpec {
        spec()
    }
    fn prepare(&self, call: &ToolCall) -> Result<ToolRequest, String> {
        if call.name != self.spec().name {
            return Err("Invalid question tool.".into());
        }
        let request = QuestionRequest::parse(&call.arguments)?;
        Ok(ToolRequest {
            call_id: call.id.clone(),
            name: call.name.clone(),
            target: "Answer question".into(),
            query: Some(json!(request).to_string()),
            diff: None,
            command: None,
            mcp: None,
        })
    }
    async fn invoke(
        &self,
        request: &ToolRequest,
        cancel: CancellationToken,
    ) -> Result<String, String> {
        if cancel.is_cancelled() {
            return Err(stopped());
        }
        if request.name != self.spec().name {
            return Err("Invalid question tool.".into());
        }
        let questions =
            QuestionRequest::parse(request.query.as_deref().ok_or("Missing questions.")?)?;
        let call_id = format!("{}:{}", self.id, uuid::Uuid::new_v4());
        let (reply, receive) = oneshot::channel();
        {
            let mut slot = self
                .pending
                .lock()
                .map_err(|_| "Questions are unavailable.")?;
            if slot.is_some() {
                return Err("Another decision is pending.".into());
            }
            *slot = Some(PendingApproval {
                call_id: call_id.clone(),
                reply: PendingReply::Question {
                    questions: questions.clone(),
                    reply,
                },
            });
        }
        let _clear = ClearQuestion {
            pending: self.pending.clone(),
            id: call_id.clone(),
        };
        if let Some(log) = &self.log {
            log.record(
                Some(dolores_core::RunState::WaitingForApproval),
                "questionAsked",
                json!({"callId":call_id,"questions":questions.questions}),
            )
            .await?;
        }
        forward(&self.output, json!({"type":"userQuestion","id":self.id,"request":{"callId":call_id,"questions":questions.questions}}), &cancel).await?;
        let answers = tokio::select! { biased;
            _ = cancel.cancelled() => return Err(stopped()),
            result = receive => result.map_err(|_| "Question closed. Start a new request to continue.")?,
        };
        if cancel.is_cancelled() {
            return Err(stopped());
        }
        questions.validate_answers(&answers)?;
        let content = json!({"answers":questions.questions.iter().zip(answers).map(|(q, answer)| json!({"id":q.id,"question":q.question,"answer":answer})).collect::<Vec<_>>(),"source":"user","grantsToolPermission":false});
        if let Some(log) = &self.log {
            log.record(
                Some(dolores_core::RunState::Running),
                "questionAnswered",
                content.clone(),
            )
            .await?;
        }
        Ok(content.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, Engine, Run};
    use std::sync::Mutex;
    fn arguments() -> String {
        json!({"questions":[{"id":"date","question":"Which date?","options":[{"label":"Today"},{"label":"Previous entry","description":"Complete the saved report"}]},{"id":"style","question":"报告语言？","options":[{"label":"中文"},{"label":"English"}]}]}).to_string()
    }
    #[test]
    fn multilingual_questions_reject_malformed_or_oversized_arguments() {
        assert!(QuestionRequest::parse(&arguments()).is_ok());
        for value in [
            json!({"questions":[]}),
            json!({"questions":[{"id":"date","question":"Which?","options":[{"label":"One"}]}]}),
            json!({"questions":[{"id":"date","question":"Which?","options":[{"label":"One"},{"label":" one "}]}]}),
            json!({"questions":[{"id":"date","question":"界".repeat(200),"options":[{"label":"One"},{"label":"Two"}]}]}),
            json!({"questions":[],"approval":true}),
        ] {
            assert!(QuestionRequest::parse(&value.to_string()).is_err());
        }
        assert!(QuestionRequest::parse("not json").is_err());
        let questions = QuestionRequest::parse(&arguments()).unwrap();
        assert!(questions
            .validate_answers(&["Today".into(), "中文\n简短".into()])
            .is_ok());
        for answers in [
            vec![],
            vec!["Today".into(), " ".into()],
            vec!["Today".into(), "界".repeat(400)],
        ] {
            assert!(questions.validate_answers(&answers).is_err());
        }
    }
    #[test]
    fn host_answers_are_single_use_wrong_modes_and_invalid_answers_keep_question_pending() {
        let engine = Engine::new(
            Arc::new(crate::SqliteStore::open(std::path::Path::new(":memory:")).unwrap()),
            Arc::new(crate::connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        let pending = Arc::new(Mutex::new(None));
        let (output, events) = mpsc::channel(8);
        let cancel = CancellationToken::new();
        engine
            .active
            .lock()
            .unwrap()
            .reserve(Run {
                thread: None,
                id: 8,
                cancel: cancel.clone(),
                events,
                approvals: pending.clone(),
            })
            .unwrap();
        let tool = Arc::new(QuestionTool {
            id: 8,
            pending: pending.clone(),
            output,
            log: None,
        });
        let request = tool
            .prepare(&ToolCall {
                id: "provider-call".into(),
                name: "request_user_input".into(),
                arguments: arguments(),
            })
            .unwrap();
        engine.runtime.block_on(async {
            let token = cancel.clone();
            let run = tokio::spawn(async move { tool.invoke(&request, token).await });
            let event = loop {
                let events = engine.call(Command::Poll { id: 8 }).unwrap();
                if let Some(event) = events.as_array().unwrap().first() {
                    break event.clone();
                }
                tokio::task::yield_now().await;
            };
            assert_eq!(event["type"], "userQuestion");
            let call_id = event["request"]["callId"].as_str().unwrap().to_owned();
            assert_ne!(call_id, "provider-call");
            assert!(engine
                .call(Command::ApproveTool {
                    id: 8,
                    call_id: call_id.clone(),
                    allow: true
                })
                .is_err());
            for (id, call, answers) in [
                (7, call_id.clone(), vec!["Today".into(), "中文".into()]),
                (8, "old".into(), vec!["Today".into(), "中文".into()]),
                (8, call_id.clone(), vec!["".into(), "中文".into()]),
            ] {
                assert!(engine
                    .call(Command::AnswerQuestion {
                        id,
                        call_id: call,
                        answers
                    })
                    .is_err());
                assert!(pending.lock().unwrap().is_some());
            }
            engine
                .call(Command::AnswerQuestion {
                    id: 8,
                    call_id: call_id.clone(),
                    answers: vec!["Previous entry".into(), "简体中文，简短".into()],
                })
                .unwrap();
            assert!(engine
                .call(Command::AnswerQuestion {
                    id: 8,
                    call_id,
                    answers: vec!["Today".into(), "English".into()]
                })
                .is_err());
            let receipt: serde_json::Value =
                serde_json::from_str(&run.await.unwrap().unwrap()).unwrap();
            assert_eq!(receipt["answers"][1]["answer"], "简体中文，简短");
            assert_eq!(receipt["grantsToolPermission"], false);
            assert!(pending.lock().unwrap().is_none());
        });
    }
    #[tokio::test]
    async fn waiting_outlives_task_deadline_and_stop_clears_pending_question() {
        let pending = Arc::new(Mutex::new(None));
        let (output, mut events) = mpsc::channel(8);
        let tool = QuestionTool {
            id: 9,
            pending: pending.clone(),
            output: output.clone(),
            log: None,
        };
        let approval = crate::approval::RunApproval {
            policy: None,
            id: 9,
            pending: pending.clone(),
            output,
            log: None,
        };
        let request = tool
            .prepare(&ToolCall {
                id: "ask".into(),
                name: "request_user_input".into(),
                arguments: arguments(),
            })
            .unwrap();
        let cancel = CancellationToken::new();
        let token = cancel.clone();
        let run = tokio::spawn(async move { tool.invoke(&request, token).await });
        events.recv().await.unwrap();
        assert!(dolores_core::ToolApproval::is_waiting(&approval));
        assert!(tokio::time::timeout(
            std::time::Duration::from_millis(1150),
            crate::task_execution::active_deadline(Some(1), &approval)
        )
        .await
        .is_err());
        cancel.cancel();
        assert!(run.await.unwrap().is_err());
        assert!(pending.lock().unwrap().is_none());
    }
}
