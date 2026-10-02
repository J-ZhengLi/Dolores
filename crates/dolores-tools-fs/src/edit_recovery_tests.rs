use super::*;

struct RecoveringModel;
#[async_trait]
impl ModelProvider for RecoveringModel {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "test",
            kind: "provider",
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
        messages: &[AgentMessage],
        _: &[ToolSpec],
        _: CancellationToken,
    ) -> Result<AgentTurn, String> {
        let count = messages.iter().filter(|m| m.role == "tool").count();
        let calls = if count < 2 {
            let mut next = call(
                "note",
                if count == 0 { "wrong match" } else { "before" },
                "after",
            );
            next.id = format!("recover-{count}");
            if count == 1 {
                assert!(messages
                    .last()
                    .unwrap()
                    .content
                    .starts_with("Exact old_text was not found."));
            }
            vec![next]
        } else {
            vec![]
        };
        Ok(AgentTurn {
            content: if calls.is_empty() {
                "Finished".into()
            } else {
                String::new()
            },
            calls,
            usage: None,
        })
    }
}
struct AllowEdit;
#[async_trait]
impl ToolApproval for AllowEdit {
    async fn authorize(&self, request: &ToolRequest, _: CancellationToken) -> Result<bool, String> {
        assert!(request.diff.as_ref().unwrap().contains("-before\n"));
        Ok(true)
    }
}
#[tokio::test]
async fn blocked_edit_returns_specific_safe_feedback_then_only_a_valid_reviewed_retry_can_write() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("note"), "before").unwrap();
    let context = vec![
        Message {
            role: Role::System,
            content: "Local rules".into(),
        },
        Message {
            role: Role::User,
            content: "Change the note".into(),
        },
    ];
    let (events, _receiver) = mpsc::channel(32);
    let result = dolores_core::run_agent(
        &RecoveringModel,
        context,
        &super::super::super::folder_tools(root.path()).unwrap(),
        &AllowEdit,
        events,
        CancellationToken::new(),
    )
    .await
    .unwrap();
    assert_eq!(result.summary.model_calls, 3);
    assert_eq!(result.summary.tools[0].status, "blocked");
    assert!(result.summary.tools[0].diff.is_none());
    assert_eq!(result.summary.tools[1].status, "edited");
    assert!(result.summary.tools[1].diff.is_some());
    assert_eq!(
        std::fs::read_to_string(root.path().join("note")).unwrap(),
        "after"
    );
}
