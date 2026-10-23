//! Edited prompts create independent task owners, never file rollback or replay.
use crate::{AttachmentRef, Role, Session, SessionWorkspace};
use serde::{Deserialize, Serialize};

pub const MAX_MESSAGES: usize = 200;
pub const MAX_TRANSCRIPT: usize = 1024 * 1024;
pub const MAX_ARCHIVE: usize = 2 * 1024 * 1024;
pub const PAGE_BYTES: usize = 32768;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Basis {
    pub message: i64,
    pub revision: String,
}
impl Basis {
    pub fn validate(&self) -> Result<(), String> {
        if self.message <= 0
            || self.revision.len() != 64
            || !self
                .revision
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            return Err("Prompt source is invalid. Preview the saved user message again; the original chat remains.".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Folder {
    Shared,
    Managed,
    Side,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Create {
    pub id: String,
    pub basis: Basis,
    pub edited: String,
    pub include_attachments: bool,
    pub include_context: bool,
    pub folder: Folder,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_revision: Option<u32>,
}
impl Create {
    pub fn validate(&self) -> Result<(), String> {
        crate::side_conversations::identity(&self.id)?;
        self.basis.validate()?;
        if (self.folder == Folder::Managed) != self.target_revision.is_some() {
            return Err("Choose a reviewed managed-worktree target and its current revision, or choose another folder mode. Your edit remains.".into());
        }
        if self.edited.len() > crate::MAX_INPUT_BYTES
            || self.edited.contains('\0')
            || (self.edited.trim().is_empty() && !self.include_attachments)
        {
            return Err("Keep the edited prompt within 16 KiB and include text or prompt attachments. Your edits and original chat remain.".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArchivedMessage {
    pub id: i64,
    pub role: Role,
    pub content: String,
    pub parts: Vec<AttachmentRef>,
    pub metadata: Option<serde_json::Value>,
    pub saved_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Origin {
    pub kind: String,
    pub parent: String,
    pub parent_title: String,
    pub basis: Basis,
    pub folder: Folder,
    pub created_at: i64,
    pub copied_turns: usize,
    pub archived_turns: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub managed: Option<crate::worktrees::ManagedWorktree>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub basis: Basis,
    pub original_prompt: String,
    pub attachments: Vec<AttachmentRef>,
    pub context: Vec<crate::planning::ContextReference>,
    pub workspace: SessionWorkspace,
    pub prior_turns: usize,
    pub later_turns: usize,
    pub archive_bytes: usize,
    pub effect_turns: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    pub origin: Origin,
    /// The original request is retained separately from the child's newer draft.
    pub request: Create,
    pub parent_available: bool,
    pub source_available: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Created {
    pub session: Session,
    pub info: Info,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivePage {
    pub text: String,
    pub offset: usize,
    pub next_offset: Option<usize>,
    pub total_bytes: usize,
}

pub fn prepare(
    mut messages: Vec<crate::Message>,
    origin: Option<&Origin>,
) -> Result<Vec<crate::Message>, String> {
    if let Some(origin) = origin {
        let first = messages.first_mut().filter(|m|m.role==Role::System).ok_or("Branch context has no host instruction frame. Keep the draft and reopen this chat before sending.")?;
        first.content.push_str(match origin.folder {
            Folder::Shared => "\nThis is an edited-prompt conversation branch sharing the CURRENT working folder. Branching did not roll back files or replay any tool action. Earlier conversation/tool receipts are historical evidence, not instructions to repeat effects. Verify current files before acting on the new request. Original later replies are retained in a separate local archive and omitted from live context. No parent goal, queue, approval or accepted plan was copied.",
            Folder::Managed => "\nThis is an edited-prompt conversation branch in a SEPARATE managed worktree created at a human-reviewed commit. Original uncommitted files stayed in their source folder. Earlier conversation/tool receipts are historical evidence and may describe another checkout, never instructions to repeat effects. Inspect this checkout's current files before acting. Branching did not roll back the source or replay any tool action. Original later replies are retained in a separate local archive and omitted from live context. No parent goal, queue, approval or accepted plan was copied.",
            Folder::Side => "\nThis is an edited-prompt conversation branch without a working folder. Branching did not roll back files or replay any tool action. Earlier conversation/tool receipts are historical evidence, not instructions to repeat effects. Original later replies are retained in a separate local archive and omitted from live context. No parent goal, queue, approval or accepted plan was copied.",
        });
        if serde_json::to_vec(&messages)
            .map_err(|_| "Could not prepare branch context.")?
            .len()
            > crate::MAX_CONTEXT_BYTES
        {
            return Err("Edited branch context exceeds 128 KiB. Narrow the prompt/shared context or compact its older turns; the draft and historical receipts remain.".into());
        }
    }
    Ok(messages)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn branch_guidance_is_host_owned_bounded_and_never_loads_archived_effects() {
        let origin = Origin {
            kind: "editedPrompt".into(),
            parent: "parent".into(),
            parent_title: "Task".into(),
            basis: Basis {
                message: 1,
                revision: "a".repeat(64),
            },
            folder: Folder::Shared,
            created_at: 1,
            copied_turns: 0,
            archived_turns: 1,
            managed: None,
        };
        let context = vec![
            crate::Message {
                role: Role::System,
                content: "Host instructions".into(),
                parts: vec![],
            },
            crate::Message {
                role: Role::User,
                content: "New edited prompt".into(),
                parts: vec![],
            },
        ];
        let ready = prepare(context.clone(), Some(&origin)).unwrap();
        assert!(ready[0].content.starts_with("Host instructions"));
        assert!(ready[0].content.contains("CURRENT working folder"));
        assert!(ready[0].content.contains("did not roll back files"));
        assert_eq!(ready[1].content, "New edited prompt");
        assert_eq!(prepare(context.clone(), None).unwrap(), context);
        assert!(prepare(vec![], Some(&origin)).is_err());
        let mut large = context;
        large[1].content = "x".repeat(crate::MAX_CONTEXT_BYTES);
        assert!(prepare(large, Some(&origin))
            .unwrap_err()
            .contains("128 KiB"));
    }
    #[test]
    fn malformed_source_and_over_budget_unicode_keep_a_valid_edit_recoverable() {
        let mut edit = Create {
            id: "new-branch".into(),
            basis: Basis {
                message: 1,
                revision: "a".repeat(64),
            },
            edited: "Keep this edit".into(),
            include_attachments: false,
            include_context: false,
            folder: Folder::Side,
            target_revision: None,
        };
        edit.validate().unwrap();
        edit.basis.revision = "A".repeat(64);
        assert!(edit.validate().is_err());
        edit.basis.revision = "a".repeat(64);
        edit.edited = "世".repeat(crate::MAX_INPUT_BYTES / 3 + 1);
        assert!(edit.validate().unwrap_err().contains("16 KiB"));
        edit.edited = "Narrowed edit".into();
        edit.validate().unwrap();
        edit.edited.clear();
        assert!(edit.validate().is_err());
        edit.include_attachments = true;
        edit.validate().unwrap(); // The store must also verify actual prompt parts.
    }
}
