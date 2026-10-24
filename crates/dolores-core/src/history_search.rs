//! Local history organization is independent of execution authority.
use crate::{Session, SessionWorkspace};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ArchiveFilter {
    #[default]
    Visible,
    Archived,
    All,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Filter {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub archive: ArchiveFilter,
    pub pinned: Option<bool>,
    /// project, temporary or side; no implicit filesystem permission.
    pub kind: Option<String>,
    pub root: Option<String>,
}
impl Filter {
    pub fn validate(&self) -> Result<(), String> {
        if self.text.len() > 256
            || self.text.chars().any(char::is_control)
            || self
                .kind
                .as_deref()
                .is_some_and(|k| !matches!(k, "project" | "temporary" | "side"))
            || self
                .root
                .as_ref()
                .is_some_and(|r| r.len() > 4096 || r.chars().any(char::is_control))
        {
            return Err("Keep history search within 256 bytes and choose a valid workspace filter. Your history and search text remain.".into());
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Organization {
    pub revision: u32,
    pub pinned: bool,
    pub archived: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Cursor {
    pub filter: Filter,
    pub pinned: bool,
    pub updated_at: i64,
    pub id: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Query {
    pub filter: Filter,
    pub cursor: Option<Cursor>,
}
impl Query {
    pub fn validate(&self) -> Result<(), String> {
        self.filter.validate()?;
        if let Some(cursor) = &self.cursor {
            if cursor.filter != self.filter || cursor.id.is_empty() || cursor.id.len() > 256 {
                return Err("Search filters changed. Refresh results from the beginning; saved history remains.".into());
            }
        }
        Ok(())
    }
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Excerpt {
    pub message: i64,
    pub text: String,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Hit {
    pub session: Session,
    pub workspace: SessionWorkspace,
    pub organization: Organization,
    pub excerpt: Option<Excerpt>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub items: Vec<Hit>,
    pub next_cursor: Option<Cursor>,
}
