use super::*;
use dolores_core::side_conversations::{Create, Return};
use sha2::{Digest, Sha256};

#[cfg(test)]
#[path = "side_conversation_tests.rs"]
mod tests;

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "camelCase", deny_unknown_fields)]
pub(super) enum Request {
    Info,
    Preview { message: i64, excerpt: String },
    PreviewReturn { message: i64, excerpt: String },
    Create { request: Create },
    BringBack { request: Return },
    InspectReturn { request: Return },
}

impl Engine {
    pub(super) fn side_conversation_call(
        &self,
        s: &str,
        request: Request,
        active: &run_journal::RunCoordinator,
    ) -> Result<Value, String> {
        match request {
            Request::Info => Ok(json!(self.store.side_conversation(s)?)),
            Request::Preview { message, excerpt } => {
                Ok(json!(self.store.side_quote(s, message, &excerpt)?))
            }
            Request::PreviewReturn { message, excerpt } => {
                let quote = self.store.side_quote(s, message, &excerpt)?;
                dolores_core::side_conversations::conclusion(s, &quote)?;
                Ok(json!(quote))
            }
            Request::Create { request } => {
                Ok(json!(self.store.create_side_conversation(s, &request)?))
            }
            Request::InspectReturn { request } => {
                dolores_core::side_conversations::identity(&request.id)?;
                request.quote.validate()?;
                let saved = self.store.side_conversation(s)?;
                let digest = format!("{:x}", Sha256::digest(request.quote.excerpt.as_bytes()));
                Ok(match saved {
                    Some(info)
                        if info.origin.returns.iter().any(|r| {
                            r.id == request.id
                                && r.message == request.quote.message
                                && r.revision == request.quote.revision
                                && r.mode == request.mode
                                && r.excerpt_digest == digest
                        }) =>
                    {
                        json!(info)
                    }
                    _ => Value::Null,
                })
            }
            Request::BringBack { request } => {
                let origin = self
                    .store
                    .side_conversation(s)?
                    .ok_or("Open a linked side discussion before bringing a conclusion back.")?;
                let parent = &origin.origin.parent;
                if let Some(id) = active.client_id(parent) {
                    if self.store.scheduled_for_run(parent, id)?.is_some()
                        || self.store.runs(parent)?.first().is_some_and(|run| {
                            run.tools.iter().any(|name| {
                                matches!(
                                    name.as_str(),
                                    "desktop_control" | "inspect_desktop_capture"
                                )
                            })
                        })
                    {
                        return Err("Finish the parent's scheduled or computer-use run before sharing this conclusion. Its draft and this selection remain.".into());
                    }
                }
                Ok(json!(self.store.return_side_conclusion(s, &request)?))
            }
        }
    }
}
