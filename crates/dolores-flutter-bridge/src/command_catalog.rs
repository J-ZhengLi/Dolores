use super::*;

impl Engine {
    pub(super) fn composer_commands(
        &self,
        session: Option<&str>,
        busy: bool,
    ) -> Result<Value, String> {
        let saved = skills::for_session(self.store.as_ref(), session)?;
        let selected = session
            .map(|s| self.store.composer_skills(s))
            .transpose()?
            .unwrap_or_default();
        let paused = session
            .map(|s| self.store.task_inputs(s))
            .transpose()?
            .is_some_and(|s| s.suspended || s.recovery_required);
        Ok(json!({
            "selected": selected,
            "commands": [
                {"id":"stop","title":"Stop","description":"Stop this chat's current response","available":busy,"reason":"No response is running in this chat"},
                {"id":"resume-queue","title":"Resume queue","description":"Resume saved instructions after inspecting them","available":paused && !busy,"reason":"Available when this chat's queue is paused and idle"},
                {"id":"goal","title":"Goal mode","description":"Set or inspect a saved objective and criteria","available":!busy,"reason":"Finish or stop this response before editing its goal"},
                {"id":"plan","title":"Plan mode","description":"Review a plan before implementation","available":!busy,"reason":"Finish or stop this response before changing its plan"}
            ],
            "skills": dolores_core::effective_skills(&saved).into_iter().map(|s|json!({
                "selection":dolores_core::SkillSelection::from_skill(s),
                "description":s.current().document.description,
                "text":s.current().document.text
            })).collect::<Vec<_>>()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn registry_is_host_owned_and_preview_rejects_stale_selection() {
        let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
        store.create("chat").unwrap();
        let doc = dolores_core::SkillDocument {
            name: "goal".into(),
            description: "Structure exports".into(),
            text: "EXACT_SELECTED_V1".into(),
        };
        let skill = store.activate_global_skill(&doc, None, None).unwrap();
        let engine = Engine::new(
            store.clone(),
            Arc::new(connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        let catalogue = engine
            .call(Command::ComposerCommands {
                session: Some("chat".into()),
            })
            .unwrap();
        assert_eq!(catalogue["commands"][2]["id"], "goal");
        assert_eq!(catalogue["commands"][2]["available"], true);
        assert_eq!(catalogue["commands"][3]["available"], true);
        assert_eq!(catalogue["skills"][0]["selection"]["name"], "goal");
        engine
            .call(Command::SaveComposerSkills {
                session: "chat".into(),
                selections: vec![dolores_core::SkillSelection::from_skill(&skill)],
            })
            .unwrap();
        let preview = engine
            .call(Command::Context {
                session: Some("chat".into()),
                input: "Hello".into(),
                tools: false,
            })
            .unwrap();
        assert!(preview["messages"]
            .to_string()
            .contains("EXACT_SELECTED_V1"));
        assert_eq!(preview["skills"][0]["version"], 1);
        store.save_draft("chat", "Retained thought").unwrap();
        store
            .activate_global_skill(
                &dolores_core::SkillDocument {
                    text: "EXACT_V2".into(),
                    ..doc
                },
                Some(1),
                None,
            )
            .unwrap();
        assert!(engine
            .call(Command::Context {
                session: Some("chat".into()),
                input: "Hello".into(),
                tools: false
            })
            .unwrap_err()
            .contains("Selected skill changed"));
        assert_eq!(store.saved_draft("chat").unwrap(), "Retained thought");
    }
}
