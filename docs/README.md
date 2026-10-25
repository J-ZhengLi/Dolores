# Documentation by task

Read one matching entry, then follow only the relevant section's references.
Existing code/tests are the source of current implementation detail. The presence
of a document is not a reason to load it or update it for every task.

| Task | Owner |
| --- | --- |
| Use Dolores or recover work | [User guide](USER_GUIDE.md) |
| Build, test or contribute | [Contributor guide](../CONTRIBUTING.md) |
| Package or release | [Release procedure](RELEASING.md) |
| Understand current components/trust | [Architecture](ARCHITECTURE.md) |
| Change layout, wording, controls or motion | [UI design guidelines](UI.md) |
| Choose or continue implementation work | [Current roadmap](ROADMAP.md), then the selected batch |
| Evaluate support/qualification claims | [Current acceptance limits](ACCEPTANCE.md) |
| Trace old decisions or dated results | [History lookup](IMPLEMENTATION_HISTORY.md) |
| Change Rust/Flutter commands and ownership | [Bridge contract](flutter/flutter-api.md) |
| Change data sharing/retention | [Privacy](PRIVACY.md) |

## Feature references

Read the specification for invariants, its tests/code for implementation, and a
qualification report only when a task depends on that evidence.

| Area | Relevant specification |
| --- | --- |
| Workspace, files, Git or terminal | [Workspace](design/developer-workspace.md), [Documents](design/editor-documents.md), [Git](design/source-control.md), [Terminal](design/terminal-language-services.md) |
| Conversations, planning, goals or history | Selected task via [harness batches](plans/everyday-harness/README.md) |
| Settings or model/context controls | [Scoped settings](design/scoped-settings.md), [Generation profiles](design/model-generation-profiles.md), [Context](design/token-context.md) |
| Memory, skills or connections | [Memory](design/automatic-memory.md), [Skills](design/project-skills.md), [MCP](design/mcp-connection.md) |
| Search, browser, computer or subagents | [Search](design/web-search.md), [Browser](design/browser-use.md), [Computer](design/computer-use.md), [Subagents](design/subagents.md) |
| Scheduling or companionship | [Scheduling](design/chat-scheduling.md), [Closed UI](design/closed-ui-scheduling.md), [Companionship](design/companionship.md) |
| Recovery, permissions or ownership | [Recovery](design/long-task-recovery.md), [Permissions](design/task-permissions.md), [Ownership](design/run-ownership.md) |
| Harness architecture changes | [Target architecture](design/evolving-harness-architecture.md), [Behavior](design/dolores-behavior.md), [Self-repair](design/harness-self-repair.md) |
| Reviewed project lifecycle scripts | [Lifecycle hooks](design/lifecycle-hooks.md) |

Other specifications remain under `design/`, qualification records under
`qualification/`, and dated investigations under `research/`; locate them by name
or heading when needed. A design target or historical observation is not current
feature acceptance. Keep one owner per meaning; link to it instead of repeating
its checklist, decisions or test transcript in another document.
