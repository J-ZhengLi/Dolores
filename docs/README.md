# Documentation

| You want to… | Start here |
| --- | --- |
| Connect a model, choose a project, use tools or recover work | [User guide](USER_GUIDE.md) |
| Build, test, package or contribute | [Contributor guide](../CONTRIBUTING.md) |
| Understand components, limits and trust | [Architecture](ARCHITECTURE.md) |
| Review the target self-evolving harness design | [Architecture specification](design/evolving-harness-architecture.md) |
| Understand questioning, personality and automatic adaptation policy | [Dolores behavior](design/dolores-behavior.md) |
| Change the UI consistently | [UI contract](UI.md) |
| Work on Flutter/Rust integration | [Bridge contract](flutter/flutter-api.md) |
| Understand stored/shared information | [Privacy](PRIVACY.md) |
| Inspect Windows bundled licenses and prerequisites | [Dependency notices](DEPENDENCIES.md) |
| Check what was actually verified | [Acceptance](ACCEPTANCE.md) |
| Review priorities, dependencies and acceptance gates | [Roadmap](ROADMAP.md) |
| See completed bricks and historical scope | [Implementation history](IMPLEMENTATION_HISTORY.md) |

`design/` contains implementation contracts and feature invariants. Brick labels connect decisions to the roadmap; it is not the product getting-started guide. `research/` holds dated source research and trials, including earlier alternatives. ACCEPTANCE retains historical evidence with its original boundary; newer sections describe later behavior.

## Selected design references

- [Task access and revocation](design/task-permissions.md) explains review, selected grants, full access and the real execution boundary.

- [Task budgets and fixed baselines](design/task-budgets.md) describes scoped allowances, continuation accounting and observable task criteria.

- [Evolving harness specification](design/evolving-harness-architecture.md), [behavior policy](design/dolores-behavior.md), [vision discussion](design/dolores-vision-discussion.md) and [DSH/Claude research](research/evolving-harnesses.md). These are target/design references, not current feature acceptance.
- [Working sessions](design/working-sessions.md), [change journal](design/change-journal.md) and [commands](design/approved-commands.md).
- [Automatic preferences](design/automatic-memory.md), [skills](design/project-skills.md) and [MCP](design/mcp-connection.md).
- [Recovery](design/long-task-recovery.md), [generation profiles](design/model-generation-profiles.md) and [coding repair](design/coding-validation-repair.md).
- [Run ownership](design/run-ownership.md), [extension registry](design/extension-registry.md) and [scoped settings](design/scoped-settings.md).
- [Task feedback](design/task-feedback.md), [comparisons](design/context-comparisons.md) and [regressions](design/regression-runner.md).
- [Windows portable preview](design/windows-portable.md).
