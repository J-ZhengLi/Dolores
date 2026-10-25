# Current acceptance limits

Dolores is an early Windows x64 alpha. Automated/loopback fixtures establish the
flows they exercise; they do not establish general model competence, physical
input/accessibility, clean-machine support or other-platform acceptance.

## Qualification by area

Read the matching record only when the task depends on its evidence or gaps.
Records are bounded observations, not a claim that an old pass qualifies new code.

| Area | Scope and remaining boundary | Owning record |
| --- | --- | --- |
| Current harness work | EH00–EH09 delivered within declared bounds; EH10 remains open. | [Current batches](plans/everyday-harness/README.md) |
| Child supervision | Frozen enabled models, bounded follow-ups/context inspection, individual interruption, parent Stop and retained writes pass Windows loopback fixtures. Bounded goal/queue/hooks integration passes; live-model competence remains separate. | [Child supervision](qualification/child-supervision.md) |
| Workflow lifecycle | Combined package/connector/hook/child, goal/queue ownership, scheduled resource pins and Stop cleanup pass public Windows fixtures with process restart. Broader migration/performance and live-model acceptance remain EH10. | [Lifecycle](qualification/harness-lifecycle.md) |
| Lifecycle hooks | Reviewed project scripts, blocking pre-events and observational post-events pass bounded Windows fixtures. Shared child hooks and frozen scheduled admission pass; other platforms and physical input remain separate. | [Hooks](qualification/lifecycle-hooks.md) |
| Remote MCP | Windows browser sign-in/vault/refresh and connected recovery pass public loopback fixtures. Bounded legacy-session cleanup passes; third-party reliability, abrupt-loss uncertainty and other platforms remain separate. | [Remote MCP](qualification/mcp-remote.md) |
| Computer use | Windows controls and guards exist; frozen live journeys still contain failures. Arbitrary-app/model reliability remains unaccepted. | [Computer use](qualification/computer-use.md) |
| Native UX/input | Rendered interactions are covered; physical keyboard/IME, assistive technology and a complete native interaction matrix remain open. | [Deferred audit cases](qualification/first-user-audit.md#method-and-status), [UX](qualification/everyday-ux.md) |
| Editor/resources | Large-file typing, sustained idle/resource targets and representative low-end hardware require matched native evidence. | [Workspace editor](qualification/workspace-editor.md), [Idle resources](qualification/idle-resources.md) |
| Tasks/recovery | Stop/continuation and retained effects have bounded qualification; provider-specific malformed/partial responses still need relevant checks. | [Task loop](qualification/task-loop.md), [Response recovery](qualification/response-recovery.md) |
| Memory/learning | Specific capture/correction and recall cases pass; general extraction, useful automatic improvement and regression restoration remain bounded/unaccepted. | [Memory](qualification/automatic-memory.md), [Learning](qualification/learning-followup.md) |
| Scheduling/companionship | Implemented Windows scopes have fixture/native records; suspend/crash, delivery and model usefulness claims must retain those boundaries. | [Scheduling](qualification/chat-scheduling.md), [Closed UI](qualification/closed-ui-scheduling.md), [Companionship](qualification/companionship.md) |
| Git/terminal | Windows operations have targeted qualification; physical terminal input and broader editor/language/platform behavior remain separate. | [Source Control](qualification/source-control.md), [Terminal](qualification/terminal-language.md) |
| Release/platforms | Windows build/installer tooling exists. Signing, clean-machine acceptance, macOS/Linux qualification and broad native self-replacement remain separate gates. | [Release](qualification/open-source-release.md), [Self-repair](qualification/harness-self-repair.md) |

## Maintenance

Change this index when a support claim, material limitation or qualification owner
changes. Update a feature's qualification record only when its acceptance boundary
changes, replacing its current summary rather than appending routine test runs.
Report ordinary checks in the task response/CI; retain raw logs under ignored
`output/`. Historical runs, counts and failed attempts remain recoverable through
the [history index](IMPLEMENTATION_HISTORY.md#acceptance-records).

The reported Close blockage is covered by focused storage-failure, pending-action,
open-review and stalled-checkpoint regressions: Close hides immediately and recovery
cannot veto it. The original profile's underlying storage failure has not been
reproduced; this is a close-policy fix, not a claim that storage is infallible.
See [UX qualification](qualification/everyday-ux.md#window-close-boundary).
Native GPU frame timing for the glass effects remains unmeasured.
