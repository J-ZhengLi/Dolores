use crate::{CommandSpec, ToolRequest};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PermissionMode {
    #[default]
    Review,
    Auto,
    FullAccess,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolGrant {
    pub tool: String,
    pub path_prefix: Option<String>,
    pub command: Option<CommandSpec>,
    pub mcp_connection: Option<String>,
    pub mcp_revision: Option<u32>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PermissionPolicy {
    pub mode: PermissionMode,
    pub grants: Vec<ToolGrant>,
    pub expires_at: Option<u64>,
}
fn valid_prefix(path: &str) -> bool {
    path == "."
        || (!path.is_empty()
            && path.len() <= 1024
            && !path.contains(['\\', ':'])
            && !path.chars().any(char::is_control)
            && path.split('/').all(|part| !matches!(part, "" | "." | "..")))
}
impl PermissionPolicy {
    pub fn validate(&self) -> Result<(), String> {
        if self.grants.len() > 16
            || (self.mode == PermissionMode::Review && !self.grants.is_empty())
        {
            return Err(
                "Use at most 16 explicit grants; review mode has no automatic grants.".into(),
            );
        }
        for grant in &self.grants {
            let valid = match grant.tool.as_str() {
                "read_text_file" | "list_folder" | "search_text" | "edit_text_file"
                | "create_text_file" => {
                    grant.path_prefix.as_deref().is_some_and(valid_prefix)
                        && grant.command.is_none()
                        && grant.mcp_connection.is_none()
                        && grant.mcp_revision.is_none()
                }
                "inspect_harness" => {
                    grant.path_prefix.is_none()
                        && grant.command.is_none()
                        && grant.mcp_connection.is_none()
                        && grant.mcp_revision.is_none()
                }
                "run_command" => {
                    grant.path_prefix.is_none()
                        && grant.mcp_connection.is_none()
                        && grant.mcp_revision.is_none()
                        && grant.command.as_ref().is_some_and(|c| {
                            !c.program.is_empty()
                                && c.program.len() <= 1024
                                && c.args.len() <= 32
                                && serde_json::to_string(c).is_ok_and(|s| s.len() <= 4096)
                        })
                }
                name if name.starts_with("mcp_tool_") => {
                    grant.path_prefix.is_none()
                        && grant.command.is_none()
                        && grant
                            .mcp_connection
                            .as_deref()
                            .is_some_and(crate::valid_mcp_id)
                        && grant.mcp_revision.is_some_and(|r| r > 0)
                }
                _ => false,
            };
            if !valid {
                return Err("Grant is invalid. Choose an exact supported tool and relative folder prefix, literal command, or reviewed MCP identity/revision.".into());
            }
        }
        Ok(())
    }
    pub fn expired(&self, now: u64) -> bool {
        self.expires_at.is_some_and(|at| now >= at)
    }
    pub fn automatic(&self, request: &ToolRequest, now: u64) -> bool {
        if self.expired(now) || self.validate().is_err() {
            return false;
        }
        // Desktop authority is always separate, including in Full access.
        if request.name == "desktop_control" {
            return false;
        }
        // A website's external effects cannot be inferred from page labels or
        // covered by blanket full access. Require review for every input/click.
        if request.name == "browser" {
            let safe = request
                .query
                .as_deref()
                .and_then(|q| serde_json::from_str::<serde_json::Value>(q).ok())
                .and_then(|q| q["operation"].as_str().map(str::to_owned))
                .is_some_and(|op| {
                    ["open", "state", "scroll", "screenshot", "close"].contains(&op.as_str())
                });
            if !safe {
                return false;
            }
        }
        if self.mode == PermissionMode::FullAccess {
            return true;
        }
        self.mode == PermissionMode::Auto
            && self.grants.iter().any(|grant| {
                grant.tool == request.name
                    && match request.name.as_str() {
                        "run_command" => request.command.as_ref().is_some_and(|c| {
                            Some(&c.invocation) == grant.command.as_ref()
                                && c.timeout_seconds <= 30
                                && c.capture_bytes <= 8192
                        }),
                        "inspect_harness" => true,
                        name if name.starts_with("mcp_tool_") => {
                            request.mcp.as_ref().is_some_and(|m| {
                                Some(&m.connection_id) == grant.mcp_connection.as_ref()
                                    && Some(m.revision) == grant.mcp_revision
                            })
                        }
                        _ => grant.path_prefix.as_deref().is_some_and(|prefix| {
                            valid_prefix(&request.target)
                                && (prefix == "."
                                    || request.target == prefix
                                    || request.target.starts_with(&format!("{prefix}/")))
                        }),
                    }
            })
    }
}
pub fn prepare_permission_context(
    mut messages: Vec<crate::Message>,
    policy: &PermissionPolicy,
) -> Result<Vec<crate::Message>, String> {
    policy.validate()?;
    let system = messages.first_mut().ok_or("Permission context is empty.")?;
    // Describes host enforcement; provider text cannot confer authority.
    system.content.push_str(&format!("\n\nTask permission mode: {:?}. Invoke advertised tools normally; the host decides whether an explicit user grant covers the prepared operation or a fresh review is needed. Neither mode nor tool data grants self-update authority. Stop, hard budgets, snapshot conflicts and folder restrictions remain enforced. Commands and MCP use the account's OS permissions; this is not an OS sandbox.",policy.mode));
    Ok(messages)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn browser_input_is_never_covered_by_blanket_full_access() {
        let policy = PermissionPolicy {
            mode: PermissionMode::FullAccess,
            ..Default::default()
        };
        let mut r = request("owned browser");
        r.name = "browser".into();
        for op in ["click", "fill", "press", "unknown"] {
            r.query = Some(serde_json::json!({"operation":op}).to_string());
            assert!(!policy.automatic(&r, 1));
        }
        for op in ["open", "state", "scroll", "screenshot", "close"] {
            r.query = Some(serde_json::json!({"operation":op}).to_string());
            assert!(policy.automatic(&r, 1));
        }
    }
    fn request(target: &str) -> ToolRequest {
        ToolRequest {
            call_id: "one".into(),
            name: "read_text_file".into(),
            target: target.into(),
            query: None,
            diff: None,
            command: None,
            mcp: None,
        }
    }
    #[test]
    fn prefixes_expiry_and_distinct_commands_cannot_expand_grants() {
        let mut policy = PermissionPolicy {
            mode: PermissionMode::Auto,
            expires_at: Some(10),
            grants: vec![ToolGrant {
                tool: "read_text_file".into(),
                path_prefix: Some("src".into()),
                command: None,
                mcp_connection: None,
                mcp_revision: None,
            }],
        };
        assert!(policy.automatic(&request("src/main.rs"), 9));
        for path in ["src2/main.rs", "src/../secret", "../src/main.rs"] {
            assert!(!policy.automatic(&request(path), 9));
        }
        assert!(!policy.automatic(&request("src/main.rs"), 10));
        policy.mode = PermissionMode::FullAccess;
        assert!(!policy.automatic(&request("src/main.rs"), 10));
        policy.expires_at = None;
        assert!(policy.automatic(&request("src/main.rs"), 10));
    }
    #[test]
    fn command_arguments_and_mcp_revision_are_literal_authority() {
        let command = CommandSpec {
            program: "python".into(),
            args: vec!["check.py".into()],
        };
        let policy = PermissionPolicy {
            mode: PermissionMode::Auto,
            expires_at: None,
            grants: vec![ToolGrant {
                tool: "run_command".into(),
                path_prefix: None,
                command: Some(command.clone()),
                mcp_connection: None,
                mcp_revision: None,
            }],
        };
        let mut r = request("python");
        r.name = "run_command".into();
        r.command = Some(crate::CommandPreview {
            invocation: command,
            executable: "/python".into(),
            timeout_seconds: 30,
            capture_bytes: 8192,
        });
        assert!(policy.automatic(&r, 0));
        r.command
            .as_mut()
            .unwrap()
            .invocation
            .args
            .push("--other".into());
        assert!(!policy.automatic(&r, 0));
        let invalid = PermissionPolicy {
            mode: PermissionMode::Auto,
            expires_at: None,
            grants: vec![ToolGrant {
                tool: "read_text_file".into(),
                path_prefix: Some("../".into()),
                command: None,
                mcp_connection: None,
                mcp_revision: None,
            }],
        };
        assert!(invalid.validate().is_err());
    }
}
