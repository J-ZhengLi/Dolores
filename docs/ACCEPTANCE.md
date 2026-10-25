# Current acceptance limits

Dolores is an early Windows x64 alpha. Automated/loopback fixtures establish the
flows they exercise; they do not establish general model competence, physical
input/accessibility, clean-machine support or other-platform acceptance.

## Qualification by area

Read the matching record only when the task depends on its evidence or gaps.
Records are bounded observations, not a claim that an old pass qualifies new code.

| Area | Scope and remaining boundary | Owning record |
| --- | --- | --- |
| Current harness work | EH00–EH08 delivered within declared bounds; EH09 incomplete and EH10 pending. Remote MCP OAuth has protocol fixtures, while connected sign-in/vault/recovery remains open. | [Current batches](plans/everyday-harness/README.md) |
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

One reported conversation-storage/Close failure was deferred during UI work. It
remains unqualified unless a later focused investigation establishes its outcome;
passing UI renders do not close it. Native GPU frame timing for the glass effects
also remains unmeasured.
