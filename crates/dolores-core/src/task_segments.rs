use crate::{AgentMessage, AgentSummary, TaskArchive, ToolApproval};
use tokio_util::sync::CancellationToken;

pub async fn save(
    approval: &dyn ToolApproval,
    summary: &mut AgentSummary,
    messages: &[AgentMessage],
    cancel: CancellationToken,
) -> Result<TaskArchive, String> {
    let (run_id, through_segment) = approval
        .save_task_segment(summary, messages, cancel.clone())
        .await?;
    if cancel.is_cancelled() {
        return Err("Response stopped.".into());
    }
    let unresolved = crate::unresolved_commands(&summary.tools);
    let recent_desktop: std::collections::HashSet<_> = summary
        .tools
        .iter()
        .rev()
        .filter(|t| {
            matches!(
                t.name.as_str(),
                "computer" | "desktop_control" | "inspect_desktop_capture" | "delegate_tasks"
            )
        })
        .take(4)
        .map(|t| t.call_id.clone())
        .collect();
    let mut kept = vec![];
    let mut steps = vec![];
    // Live/failed command sessions need their start binding and latest state,
    // not every poll response. Successful finished sessions live in the archive.
    let mut sessions = std::collections::HashMap::<String, (usize, usize)>::new();
    for (i, t) in summary
        .tools
        .iter()
        .enumerate()
        .filter(|(_, t)| t.name == "command_session")
    {
        let value: serde_json::Value = serde_json::from_str(&t.content).unwrap_or_default();
        let id = value["session"]["id"]
            .as_str()
            .unwrap_or(&t.call_id)
            .to_owned();
        sessions.entry(id).and_modify(|v| v.1 = i).or_insert((i, i));
    }
    let mut session_keep = std::collections::HashSet::new();
    for (first, last) in sessions.values() {
        let t = &summary.tools[*last];
        let value: serde_json::Value = serde_json::from_str(&t.content).unwrap_or_default();
        if matches!(
            value["session"]["state"].as_str(),
            Some("starting" | "running" | "stopping")
        ) || crate::CommandOutcome::from_content(&t.content) != crate::CommandOutcome::Succeeded
        {
            session_keep.insert(*first);
            session_keep.insert(*last);
        }
    }
    for (index, tool) in summary.tools.iter().enumerate() {
        if tool
            .command
            .as_ref()
            .is_some_and(|c| unresolved.contains(c))
            || session_keep.contains(&index)
            || recent_desktop.contains(&tool.call_id)
        {
            kept.push(tool.clone());
            steps.push(summary.tool_steps.get(index).copied().unwrap_or(0));
        }
    }
    let archive = TaskArchive {
        run_id,
        through_segment,
        tool_count: summary.archive.as_ref().map_or(0, |a| a.tool_count) + summary.tools.len()
            - kept.len(),
        model_calls: summary.model_calls,
    };
    summary.tools = kept;
    summary.tool_steps = steps;
    summary.steps.clear();
    summary.thinking.clear();
    summary.usage_by_call.clear();
    summary.archive = Some(archive.clone());
    Ok(archive)
}
