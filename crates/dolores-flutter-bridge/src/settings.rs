use crate::Engine;
use dolores_core::{
    EffectiveSettings, ScopedSettings, SettingsPatch, SettingsScope, WorkspaceKind,
};
use serde_json::{json, Value};
impl Engine {
    fn settings_key(&self, scope: SettingsScope, session: Option<&str>) -> Result<String, String> {
        match scope {
            SettingsScope::User => Ok("user".into()),
            SettingsScope::Thread => {
                let id = session.ok_or("Choose a saved chat first.")?;
                self.store.workspace(id)?;
                Ok(id.into())
            }
            SettingsScope::Project => {
                let w = self
                    .store
                    .workspace(session.ok_or("Choose a project chat first.")?)?;
                if w.kind != WorkspaceKind::Project {
                    return Err("Project settings need a project chat; temporary and side chats use User or This chat.".into());
                }
                w.root.ok_or("Project folder is unavailable.".into())
            }
        }
    }
    fn settings_layers(
        &self,
        session: Option<&str>,
    ) -> Result<Vec<(SettingsScope, ScopedSettings)>, String> {
        let mut layers = vec![(
            SettingsScope::User,
            self.store.scoped_settings(SettingsScope::User, "user")?,
        )];
        if let Some(id) = session {
            let w = self.store.workspace(id)?;
            if w.kind == WorkspaceKind::Project {
                layers.push((
                    SettingsScope::Project,
                    self.store.scoped_settings(
                        SettingsScope::Project,
                        w.root.as_deref().ok_or("Project folder is unavailable.")?,
                    )?,
                ));
            }
            layers.push((
                SettingsScope::Thread,
                self.store.scoped_settings(SettingsScope::Thread, id)?,
            ));
        }
        Ok(layers)
    }
    fn inspect_layers(
        &self,
        layers: &[(SettingsScope, ScopedSettings)],
    ) -> Result<EffectiveSettings, String> {
        self.inspect_model_layers(layers, None)
    }
    fn inspect_model_layers(
        &self,
        layers: &[(SettingsScope, ScopedSettings)],
        model: Option<&str>,
    ) -> Result<EffectiveSettings, String> {
        let mut prefs = self.store.preferences()?;
        if let Some(model) = model {
            prefs.model = model.into();
        }
        let profile = self
            .store
            .model_request_settings(&prefs.base_url)?
            .contains_key(&prefs.model);
        let window = self
            .store
            .model_contexts(&prefs.base_url)?
            .get(&prefs.model)
            .copied()
            .flatten();
        dolores_core::inspect_settings(
            self.store.effective_request_settings(&prefs)?,
            if profile {
                "Model profile"
            } else {
                "User generation default"
            },
            window,
            layers,
        )
    }
    pub(super) fn effective_settings(
        &self,
        session: Option<&str>,
    ) -> Result<EffectiveSettings, String> {
        let result = self.inspect_layers(&self.settings_layers(session)?)?;
        result.validate()?;
        Ok(result)
    }
    pub(super) fn observation_settings(
        &self,
        session: &str,
        model: &str,
    ) -> Result<EffectiveSettings, String> {
        let result =
            self.inspect_model_layers(&self.settings_layers(Some(session))?, Some(model))?;
        result.validate()?;
        Ok(result)
    }
    pub(super) fn settings_view(&self, session: Option<&str>) -> Result<Value, String> {
        let layers = self.settings_layers(session)?;
        let effective = self.inspect_layers(&layers)?;
        let scopes: Vec<_> = layers
            .iter()
            .map(|(scope, record)| json!({"scope":scope,"record":record}))
            .collect();
        Ok(
            json!({"validationError":effective.validate().err(),"effective":effective,"scopes":scopes,"taskAccess":"Use Task permissions to review or grant tool access. Budget/interaction controls cannot grant self-update permission.","adaptation":"Executable self-update activation is unavailable; automatic preferences retain their separate policy."}),
        )
    }
    pub(super) fn save_settings(
        &self,
        session: Option<&str>,
        scope: SettingsScope,
        revision: u32,
        patch: SettingsPatch,
    ) -> Result<Value, String> {
        patch.validate(scope)?;
        let key = self.settings_key(scope, session)?;
        let mut layers = self.settings_layers(session)?;
        let index = layers
            .iter()
            .position(|(s, _)| *s == scope)
            .ok_or("Settings scope is unavailable.")?;
        let record = &mut layers[index].1;
        if record.revision != revision {
            return Err("Settings changed. Refresh, keep your draft and review it again.".into());
        }
        if record.patch.permissions != patch.permissions {
            return Err("Change task access in Task permissions. Budget and interaction edits must preserve existing grants.".into());
        }
        record.patch = patch.clone();
        // A child override must not mask an invalid new parent default.
        self.inspect_layers(&layers[..=index])?.validate()?;
        self.store
            .save_scoped_settings(scope, &key, revision, &patch)?;
        self.settings_view(session)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::{GenerationOverride, SessionStore};
    use std::sync::Arc;
    #[test]
    fn smaller_model_windows_keep_invalid_overrides_inspectable_and_recoverable() {
        let store = Arc::new(
            dolores_store_sqlite::SqliteStore::open(std::path::Path::new(":memory:")).unwrap(),
        );
        let prefs = dolores_core::ConnectionPreferences {
            base_url: "http://localhost/v1".into(),
            model: "fixture".into(),
        };
        store
            .save_connection_models(&prefs, None, &["fixture".into()])
            .unwrap();
        for id in ["a", "b"] {
            store
                .create_workspace_session(
                    id,
                    &dolores_core::SessionWorkspace {
                        kind: WorkspaceKind::Project,
                        root: Some("synthetic-project".into()),
                    },
                )
                .unwrap();
        }
        let engine = Engine::new(
            store.clone(),
            Arc::new(crate::connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        let large = SettingsPatch {
            permissions: None,
            task: None,
            generation: Some(GenerationOverride {
                max_output_tokens: 16384,
                timeout_seconds: 60,
            }),
            interaction: None,
        };
        engine
            .save_settings(Some("a"), SettingsScope::Project, 0, large.clone())
            .unwrap();
        engine
            .save_settings(
                Some("a"),
                SettingsScope::Thread,
                0,
                SettingsPatch {
                    permissions: None,
                    task: None,
                    generation: Some(GenerationOverride {
                        max_output_tokens: 512,
                        timeout_seconds: 60,
                    }),
                    interaction: None,
                },
            )
            .unwrap();
        store
            .save_connection_model_contexts(
                &prefs,
                None,
                &["fixture".into()],
                &[("fixture".into(), Some(8192))].into(),
            )
            .unwrap();
        assert!(engine.effective_settings(Some("b")).is_err());
        let view = engine.settings_view(Some("b")).unwrap();
        assert!(view["validationError"].is_string());
        assert_eq!(view["effective"]["contextWindowTokens"], 8192);
        assert_eq!(view["effective"]["request"]["maxOutputTokens"], 16384);
        // A valid child must not mask an invalid parent save.
        assert!(engine
            .save_settings(Some("a"), SettingsScope::Project, 1, large)
            .is_err());
        assert_eq!(
            store
                .scoped_settings(SettingsScope::Project, "synthetic-project")
                .unwrap()
                .revision,
            1
        );
        engine
            .save_settings(Some("b"), SettingsScope::Project, 1, Default::default())
            .unwrap();
        assert_eq!(
            engine
                .effective_settings(Some("b"))
                .unwrap()
                .request
                .max_output_tokens,
            2048
        );
        assert_eq!(
            engine
                .effective_settings(Some("a"))
                .unwrap()
                .request
                .max_output_tokens,
            512
        );
    }
    #[test]
    fn effective_scopes_are_private_isolated_revisioned_and_cannot_grant_access() {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(
            dolores_store_sqlite::SqliteStore::open(std::path::Path::new(":memory:")).unwrap(),
        );
        for (id, root) in [("a", dir.path().join("one")), ("b", dir.path().join("two"))] {
            store
                .create_workspace_session(
                    id,
                    &dolores_core::SessionWorkspace {
                        kind: WorkspaceKind::Project,
                        root: Some(root.to_string_lossy().into_owned()),
                    },
                )
                .unwrap();
        }
        store.create("side").unwrap();
        let engine = Engine::new(
            store,
            Arc::new(crate::connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        let patch = SettingsPatch {
            permissions: None,
            task: None,
            generation: Some(GenerationOverride {
                max_output_tokens: 512,
                timeout_seconds: 60,
            }),
            interaction: None,
        };
        engine
            .save_settings(Some("a"), SettingsScope::Project, 0, patch.clone())
            .unwrap();
        assert_eq!(
            engine
                .effective_settings(Some("a"))
                .unwrap()
                .request
                .max_output_tokens,
            512
        );
        assert_ne!(
            engine.effective_settings(Some("b")).unwrap().request_origin,
            "project"
        );
        assert!(engine
            .save_settings(Some("side"), SettingsScope::Project, 0, patch)
            .is_err());
        let frozen = engine.effective_settings(Some("a")).unwrap();
        let view = engine.settings_view(Some("a")).unwrap();
        assert!(!view.to_string().contains(dir.path().to_str().unwrap()));
        assert!(view["taskAccess"]
            .as_str()
            .unwrap()
            .contains("cannot grant"));
        assert!(engine
            .save_settings(Some("a"), SettingsScope::Project, 0, Default::default())
            .is_err());
        engine
            .save_settings(Some("a"), SettingsScope::Project, 1, Default::default())
            .unwrap();
        assert_eq!(frozen.request.max_output_tokens, 512);
        assert_ne!(
            engine.effective_settings(Some("a")).unwrap().request_origin,
            "project"
        );
    }
}
