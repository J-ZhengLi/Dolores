use crate::{Message, RequestSettings, Role};
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SettingsScope {
    User,
    Project,
    Thread,
}
impl SettingsScope {
    pub fn key(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Project => "project",
            Self::Thread => "thread",
        }
    }
}
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DiscussionStyle {
    #[default]
    Discuss,
    Brief,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InteractionPolicy {
    pub discussion: DiscussionStyle,
    pub question_assumptions: bool,
}
impl Default for InteractionPolicy {
    fn default() -> Self {
        Self {
            discussion: DiscussionStyle::Discuss,
            question_assumptions: true,
        }
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GenerationOverride {
    pub max_output_tokens: Option<u32>,
    pub timeout_seconds: u32,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SettingsPatch {
    #[serde(default)]
    pub permissions: Option<crate::PermissionPolicy>,
    #[serde(default)]
    pub task: Option<crate::TaskBudget>,
    pub generation: Option<GenerationOverride>,
    pub interaction: Option<InteractionPolicy>,
}
impl SettingsPatch {
    pub fn validate(&self, scope: SettingsScope) -> Result<(), String> {
        if let Some(p) = &self.permissions {
            if scope != SettingsScope::Thread {
                return Err("Permissions are explicitly scoped to a saved working chat.".into());
            }
            p.validate()?;
        }
        if let Some(t) = self.task {
            t.validate()?;
        }
        if let Some(g) = self.generation {
            if scope == SettingsScope::User {
                return Err("User generation defaults are edited in Request settings.".into());
            }
            RequestSettings {
                max_output_tokens: g.max_output_tokens,
                timeout_seconds: g.timeout_seconds,
                ..Default::default()
            }
            .validate()?;
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScopedSettings {
    pub revision: u32,
    pub patch: SettingsPatch,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SettingsSource {
    pub scope: SettingsScope,
    pub revision: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EffectiveSettings {
    #[serde(default)]
    pub permissions: crate::PermissionPolicy,
    #[serde(default)]
    pub task: crate::TaskBudget,
    #[serde(default)]
    pub task_origin: String,
    pub request: RequestSettings,
    pub request_origin: String,
    pub reasoning_origin: String,
    pub interaction: InteractionPolicy,
    pub interaction_origin: String,
    pub sources: Vec<SettingsSource>,
    pub context_window_tokens: u32,
    pub context_origin: String,
}
pub fn resolve_settings(
    request: RequestSettings,
    base_origin: &str,
    window: Option<u32>,
    scopes: &[(SettingsScope, ScopedSettings)],
) -> Result<EffectiveSettings, String> {
    let result = inspect_settings(request, base_origin, window, scopes)?;
    result.validate()?;
    Ok(result)
}
impl EffectiveSettings {
    pub fn validate(&self) -> Result<(), String> {
        self.task.validate()?;
        self.request.validate()?;
        crate::input_token_allowance(Some(self.context_window_tokens), self.request)?;
        Ok(())
    }
}
/// Preserve inspectable values when a changed model window makes them unusable.
/// Execution must still call validate or resolve_settings before dispatch.
pub fn inspect_settings(
    request: RequestSettings,
    base_origin: &str,
    window: Option<u32>,
    scopes: &[(SettingsScope, ScopedSettings)],
) -> Result<EffectiveSettings, String> {
    request.validate()?;
    let mut result = EffectiveSettings {
        permissions: Default::default(),
        task: Default::default(),
        task_origin: "Dolores default".into(),
        request,
        request_origin: base_origin.into(),
        reasoning_origin: base_origin.into(),
        interaction: Default::default(),
        interaction_origin: "Dolores default".into(),
        sources: vec![],
        context_window_tokens: window.unwrap_or(crate::DEFAULT_CONTEXT_WINDOW_TOKENS),
        context_origin: if window.is_some() {
            "Model configuration"
        } else {
            "128K default"
        }
        .into(),
    };
    for (scope, record) in scopes {
        record.patch.validate(*scope)?;
        if let Some(p) = &record.patch.permissions {
            result.permissions = p.clone();
        }
        result.sources.push(SettingsSource {
            scope: *scope,
            revision: record.revision,
        });
        if let Some(t) = record.patch.task {
            result.task = t;
            result.task_origin = scope.key().into();
        }
        if let Some(g) = record.patch.generation {
            result.request.max_output_tokens = g.max_output_tokens;
            result.request.timeout_seconds = g.timeout_seconds;
            result.request_origin = scope.key().into();
        }
        if let Some(i) = record.patch.interaction {
            result.interaction = i;
            result.interaction_origin = scope.key().into();
        }
    }
    result.request.validate()?;
    Ok(result)
}
pub fn prepare_behavior_context(
    mut messages: Vec<Message>,
    policy: InteractionPolicy,
) -> Result<Vec<Message>, String> {
    let system = messages
        .first_mut()
        .filter(|m| m.role == Role::System)
        .ok_or("Interaction policy needs system context.")?;
    const CHARACTER:&str="\nDolores interaction policy: Be calm, kind, candid and curious. Respect time, work and cost. Separate evidence, assumptions and verified results. No flattery or invented feelings. On failure preserve work and explain one actionable recovery. Follow scoped memory rules.";
    if !system.content.contains(CHARACTER) {
        system.content.push_str(CHARACTER);
        system.content.push_str(match policy.discussion {DiscussionStyle::Discuss=>" Explain the intended result and material tradeoffs before implementation; then continue authorized work without repeated confirmation.",DiscussionStyle::Brief=>" Keep routine explanations brief and proceed under existing authorization; explain material tradeoffs before consequential changes."});
        if policy.question_assumptions {
            system.content.push_str(" Check consequential premises. Correct a contradicted premise respectfully. Ask only when an unresolved answer materially changes the result; inspect available facts first.");
        }
    }
    if system.content.len() > crate::MAX_CONTEXT_BYTES {
        return Err("Interaction context exceeds its limit.".into());
    }
    Ok(messages)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scoped_precedence_preserves_reasoning_windows_and_frozen_previous_values() {
        let user = ScopedSettings {
            revision: 1,
            patch: SettingsPatch {
                permissions: None,
                task: None,
                interaction: Some(InteractionPolicy {
                    discussion: DiscussionStyle::Brief,
                    question_assumptions: true,
                }),
                generation: None,
            },
        };
        let project = ScopedSettings {
            revision: 2,
            patch: SettingsPatch {
                permissions: None,
                task: None,
                generation: Some(GenerationOverride {
                    max_output_tokens: Some(512),
                    timeout_seconds: 60,
                }),
                interaction: None,
            },
        };
        let thread = ScopedSettings {
            revision: 3,
            patch: SettingsPatch {
                permissions: None,
                task: None,
                generation: Some(GenerationOverride {
                    max_output_tokens: Some(256),
                    timeout_seconds: 30,
                }),
                interaction: Some(InteractionPolicy::default()),
            },
        };
        let before = resolve_settings(
            Default::default(),
            "user default",
            None,
            &[
                (SettingsScope::User, user),
                (SettingsScope::Project, project.clone()),
                (SettingsScope::Thread, thread),
            ],
        )
        .unwrap();
        assert_eq!(before.request.max_output_tokens, Some(256));
        assert_eq!(before.interaction_origin, "thread");
        assert_eq!(before.context_window_tokens, 131072);
        let after = resolve_settings(
            Default::default(),
            "model profile",
            Some(8192),
            &[(SettingsScope::Project, project)],
        )
        .unwrap();
        assert_eq!(after.request.max_output_tokens, Some(512));
        assert_eq!(before.request.max_output_tokens, Some(256));
        assert_eq!(after.reasoning_origin, "model profile");
        let context = prepare_behavior_context(
            crate::prepare_context(vec![], "small edit").unwrap(),
            before.interaction,
        )
        .unwrap();
        assert!(context[0].content.contains("without repeated confirmation"));
        assert!(context[0]
            .content
            .contains("Correct a contradicted premise"));
    }
}
