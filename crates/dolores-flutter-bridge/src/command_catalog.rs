use super::*;

impl Engine {
    pub(super) fn composer_commands(
        &self,
        session: Option<&str>,
        busy: bool,
    ) -> Result<Value, String> {
        let saved = self.available_skills(session)?;
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
    use dolores_core::{Message, PluginDescriptor};
    #[test]
    fn saved_version_can_be_restored_after_a_live_file_update() {
        let temp = tempfile::tempdir().unwrap();
        let global = temp.path().join("global");
        std::fs::create_dir_all(global.join("format-code")).unwrap();
        let text = "---\nname: format-code\ndescription: Format output\n---\nOLD_PUBLIC";
        let path = global.join("format-code/SKILL.md");
        std::fs::write(&path, text).unwrap();
        let store = Arc::new(SqliteStore::open(&temp.path().join("state.db")).unwrap());
        let mut engine = Engine::new(
            store.clone(),
            Arc::new(connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        engine.global_skills_directory = Some(global);
        engine
            .call(Command::SaveSkillText {
                session: String::new(),
                scope: dolores_core::SkillScope::Global,
                name: "format-code".into(),
                text: text.into(),
            })
            .unwrap();
        std::fs::write(&path, text.replace("OLD_PUBLIC", "NEW_PUBLIC")).unwrap();
        assert!(engine.available_skills(None).unwrap()[0]
            .current()
            .document
            .text
            .contains("NEW_PUBLIC"));
        let retained = engine
            .review_scoped_skill("", "format-code", Some(1), dolores_core::SkillScope::Global)
            .unwrap();
        assert_eq!(retained["alreadyActive"], false);
        engine
            .activate_skill("", retained["token"].as_str().unwrap())
            .unwrap();
        let restored = engine.available_skills(None).unwrap().remove(0);
        assert_eq!(restored.current().document.text, text);
        assert_eq!(restored.current().rollback_from, Some(1));
        assert!(std::fs::read_to_string(path)
            .unwrap()
            .contains("NEW_PUBLIC"));
    }
    #[test]
    fn direct_skill_save_failure_keeps_existing_version_and_retries_without_review() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("state.db");
        let store = Arc::new(SqliteStore::open(&path).unwrap());
        let mut engine = Engine::new(
            store.clone(),
            Arc::new(connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        engine.global_skills_directory = Some(temp.path().join("global"));
        let request = |body: &str| Command::SaveSkillText {
            session: String::new(),
            scope: dolores_core::SkillScope::Global,
            name: "format-code".into(),
            text: format!("---\nname: format-code\ndescription: Format output\n---\n{body}"),
        };
        engine.call(request("OLD_PUBLIC")).unwrap();
        let original = store.global_skills().unwrap()[0].clone();
        rusqlite::Connection::open(&path).unwrap().execute_batch("CREATE TRIGGER reject_skill BEFORE INSERT ON project_skills BEGIN SELECT RAISE(ABORT,'public fixture'); END;").unwrap();
        assert!(engine.call(request("NEW_PUBLIC")).is_err());
        assert_eq!(store.global_skills().unwrap()[0], original);
        rusqlite::Connection::open(&path)
            .unwrap()
            .execute_batch("DROP TRIGGER reject_skill")
            .unwrap();
        engine.call(request("NEW_PUBLIC")).unwrap();
        assert!(engine.available_skills(None).unwrap()[0]
            .current()
            .document
            .text
            .contains("NEW_PUBLIC"));
        for name in ["other-a", "other-b", "other-c"] {
            engine
                .call(Command::SaveSkillText {
                    session: String::new(),
                    scope: dolores_core::SkillScope::Global,
                    name: name.into(),
                    text: format!(
                        "---\nname: {name}\ndescription: Format output\n---\nPublic instructions"
                    ),
                })
                .unwrap();
        }
        assert_eq!(engine.available_skills(None).unwrap().len(), 4);
    }

    struct DirectSkillProvider;
    #[async_trait::async_trait]
    impl ModelProvider for DirectSkillProvider {
        fn descriptor(&self) -> PluginDescriptor {
            PluginDescriptor {
                id: "fixture",
                kind: "provider",
                api_version: 1,
            }
        }
        async fn stream(
            &self,
            messages: Vec<Message>,
            output: mpsc::Sender<String>,
            _: CancellationToken,
        ) -> Result<(), String> {
            assert!(messages[0].content.contains("DIRECT_PUBLIC_INSTRUCTIONS"));
            output
                .send("Used the skill directly.".into())
                .await
                .map_err(|_| "Fixture channel closed.".to_string())
        }
    }
    #[tokio::test]
    async fn actual_generation_receives_unreviewed_file_snapshot() {
        for explicit in [false, true] {
            let temp = tempfile::tempdir().unwrap();
            let global = temp.path().join("global");
            std::fs::create_dir_all(global.join("format-code")).unwrap();
            std::fs::write(
            global.join("format-code/SKILL.md"),
            "---\nname: format-code\ndescription: Format output\n---\nDIRECT_PUBLIC_INSTRUCTIONS",
        )
        .unwrap();
            let store = Arc::new(SqliteStore::open(&temp.path().join("state.db")).unwrap());
            store.create("chat").unwrap();
            let available =
                skills::for_session_with_global(store.as_ref(), Some("chat"), Some(&global))
                    .unwrap();
            let selections = if explicit {
                vec![dolores_core::SkillSelection::from_skill(&available[0])]
            } else {
                vec![]
            };
            let (output, _events) = mpsc::channel(32);
            let reply = execute(
                store.clone(),
                Arc::new(DirectSkillProvider),
                TurnRequest {
                    compatibility: None,
                    available_skills: Some(available),
                    hooks: None,
                    skill_resources: None,
                    command_sessions: None,
                    plan_execution: false,
                    planning: Default::default(),
                    goal: None,
                    selected_skills: selections,
                    delegation: None,
                    log: None,
                    compaction_provider: None,
                    compacted: false,
                    resume_run: None,
                    permissions: Default::default(),
                    task: Default::default(),
                    interaction: Default::default(),
                    continuation: None,
                    id: 1,
                    session: Some("chat".into()),
                    input: if explicit { "Hello" } else { "Use format-code" }.into(),
                    model: "fixture".into(),
                    settings: None,
                    tools: vec![],
                    approval: None,
                },
                CancellationToken::new(),
                &output,
            )
            .await
            .unwrap();
            assert_eq!(reply, "Used the skill directly.");
            assert!(store.global_skills().unwrap().is_empty());
            let saved = store.messages_page("chat", None, false, 2).unwrap();
            assert_eq!(
                saved.items[1].metadata.as_ref().unwrap().context.skills[0].name,
                "format-code"
            );
        }
    }
    #[test]
    fn source_skills_are_ready_without_review_and_changed_selection_keeps_the_draft() {
        let temp = tempfile::tempdir().unwrap();
        let root = crate::workspace::canonical_folder(temp.path()).unwrap();
        let global = temp.path().join("global");
        let project = temp.path().join(".agents/skills");
        let write = |base: &std::path::Path, name: &str, body: &str| {
            std::fs::create_dir_all(base.join(name)).unwrap();
            std::fs::write(
                base.join(name).join("SKILL.md"),
                format!("---\nname: {name}\ndescription: Use {name}\n---\n{body}"),
            )
            .unwrap();
        };
        write(&global, "format-code", "GLOBAL_PUBLIC");
        write(&project, "format-code", "PROJECT_PUBLIC");
        write(&global, "other-skill", "OTHER_PUBLIC");
        write(&global, "malformed", "invalid");
        std::fs::write(global.join("malformed/SKILL.md"), "invalid YAML").unwrap();
        let store = Arc::new(SqliteStore::open(&temp.path().join("state.db")).unwrap());
        store
            .create_workspace_session(
                "project",
                &dolores_core::SessionWorkspace {
                    kind: dolores_core::WorkspaceKind::Project,
                    root: Some(root),
                },
            )
            .unwrap();
        store.create("side").unwrap();
        let mut engine = Engine::new(
            store.clone(),
            Arc::new(connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        engine.global_skills_directory = Some(global.clone());
        let catalog = engine.composer_commands(Some("project"), false).unwrap();
        assert_eq!(catalog["skills"].as_array().unwrap().len(), 2);
        assert!(store
            .project_skills(temp.path().to_str().unwrap())
            .unwrap()
            .is_empty());
        assert!(store.global_skills().unwrap().is_empty());
        let selection: dolores_core::SkillSelection = serde_json::from_value(
            catalog["skills"]
                .as_array()
                .unwrap()
                .iter()
                .find(|s| s["selection"]["name"] == "format-code")
                .unwrap()["selection"]
                .clone(),
        )
        .unwrap();
        engine
            .call(Command::SaveComposerSkills {
                session: "project".into(),
                selections: vec![selection],
            })
            .unwrap();
        let preview = engine
            .call(Command::Context {
                session: Some("project".into()),
                input: "Hello".into(),
                tools: false,
            })
            .unwrap();
        assert!(preview["messages"].to_string().contains("PROJECT_PUBLIC"));
        assert!(!preview["messages"].to_string().contains("GLOBAL_PUBLIC"));
        assert!(preview["messages"]
            .to_string()
            .contains("cannot approve tools"));
        store.save_draft("project", "Keep this draft").unwrap();
        engine
            .call(Command::EditTaskInputs {
                session: "project".into(),
                edit: dolores_core::harness_task::InputEdit::Enqueue {
                    id: "queued-file".into(),
                    text: "Use the selected skill later".into(),
                    mode: dolores_core::harness_task::InputMode::Queue,
                },
            })
            .unwrap();
        assert!(store.task_inputs("project").unwrap().entries[0].selected_skills[0].file);
        write(&project, "format-code", "UPDATED_PUBLIC");
        assert!(engine
            .call(Command::Context {
                session: Some("project".into()),
                input: "Hello".into(),
                tools: false
            })
            .unwrap_err()
            .contains("Selected skill changed"));
        assert_eq!(store.saved_draft("project").unwrap(), "Keep this draft");
        assert!(engine
            .call(Command::EditTaskInputs {
                session: "project".into(),
                edit: dolores_core::harness_task::InputEdit::Enqueue {
                    id: "stale-file".into(),
                    text: "Keep this draft".into(),
                    mode: dolores_core::harness_task::InputMode::Steer
                }
            })
            .is_err());
        assert_eq!(store.task_inputs("project").unwrap().entries.len(), 1);
        assert_eq!(store.saved_draft("project").unwrap(), "Keep this draft");
        engine
            .call(Command::SaveComposerSkills {
                session: "project".into(),
                selections: vec![],
            })
            .unwrap();
        let updated = engine
            .call(Command::Context {
                session: Some("project".into()),
                input: "Use format-code".into(),
                tools: false,
            })
            .unwrap();
        assert!(updated["messages"].to_string().contains("UPDATED_PUBLIC"));
        engine
            .set_skill_enabled(
                "project",
                "format-code",
                dolores_core::SkillScope::Project,
                false,
            )
            .unwrap();
        let fallback = engine
            .call(Command::Context {
                session: Some("project".into()),
                input: "Use format-code".into(),
                tools: false,
            })
            .unwrap();
        assert!(fallback["messages"].to_string().contains("GLOBAL_PUBLIC"));
        engine
            .set_skill_enabled(
                "project",
                "format-code",
                dolores_core::SkillScope::Project,
                true,
            )
            .unwrap();
        assert_eq!(
            engine.composer_commands(Some("side"), false).unwrap()["skills"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        let bad = engine
            .scoped_skills("", dolores_core::SkillScope::Global)
            .unwrap();
        assert!(bad["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["name"] == "malformed")
            .unwrap()["problem"]
            .is_string());
        std::fs::remove_file(global.join("other-skill/SKILL.md")).unwrap();
        assert_eq!(
            engine.composer_commands(Some("side"), false).unwrap()["skills"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
    }
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
