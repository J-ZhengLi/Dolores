# Long-task harness patterns for Dolores

Research date: 2026-10-28. Scope: context, durable working state, continuation,
tool output, stopping and recovery for multi-hour coding tasks. This is source
research and an implementation proposal; it does not qualify Dolores with a
live model or establish multi-hour completion reliability.

## Source baseline

Fresh upstream HEADs were resolved with `git ls-remote`, shallow-cloned under
ignored `output/research-upstream-20261028/`, and verified with `git rev-parse`:

- OpenAI Codex: [`603a7baff0f6f37ee9a8a9375a272eaab03d3ece`](https://github.com/openai/codex/commit/603a7baff0f6f37ee9a8a9375a272eaab03d3ece), committed 2026-10-27 15:53:50 UTC.
- DeepSeek Harness: [`22ddee3792402c3e70ec5f86a63c3e3e895a2f60`](https://github.com/deepseek-ai/deepseek-harness/commit/22ddee3792402c3e70ec5f86a63c3e3e895a2f60), committed 2026-10-23 19:48:59 +08:00.

These replace the older upstream baselines for this note only. The existing
[token/context note](token-context-accounting.md) owns token estimation and the
selected Dolores capacity default; this note does not propose changing either.

Official OpenAI documentation was searched and opened before source inspection.
The [Codex prompting guide](https://developers.openai.com/cookbook/examples/gpt-5/codex_prompting_guide)
recommends its open-source harness as a reference. The
[compaction guide](https://developers.openai.com/api/docs/guides/compaction)
distinguishes two contracts: server-side compaction may allow removing input
before the latest compaction item in stateless chaining; standalone compact
returns a complete canonical window that must be reused as-is. Neither contract
means an arbitrary Chat Completions provider supports OpenAI compaction.

## Verified implementation patterns

| Concern | Observed source behavior | Useful implication for Dolores |
| --- | --- | --- |
| Separate context and execution limits | Codex independently checks active context against the full usable model window and its configured auto-compaction scope. Its turn loop compacts while a continuation is pending, then continues that turn. [Context policy](https://github.com/openai/codex/blob/603a7baff0f6f37ee9a8a9375a272eaab03d3ece/codex-rs/core/src/session/context_window.rs#L83), [turn loop](https://github.com/openai/codex/blob/603a7baff0f6f37ee9a8a9375a272eaab03d3ece/codex-rs/core/src/session/turn.rs#L679) | More model calls cannot repair lost context. Keep request fit, task budget and progress watchdog as separate policies. |
| Cheap reduction before a summary call | DSH first prunes large tool-result text, remeasures, and summarizes only if still above threshold. Range selection protects the system head and moves the retained-tail boundary to avoid splitting tool-call/result steps. [Pressure loop](https://github.com/deepseek-ai/deepseek-harness/blob/22ddee3792402c3e70ec5f86a63c3e3e895a2f60/packages/compaction/compaction-basic/src/index.ts#L326), [range selection](https://github.com/deepseek-ai/deepseek-harness/blob/22ddee3792402c3e70ec5f86a63c3e3e895a2f60/packages/compaction/compaction-basic/src/region.ts#L107) | Reduce output bodies before discarding whole turns; preserve a recent complete working tail. |
| Observable, bounded overflow recovery | DSH retries provider context overflow only after the durable surface replacement generation increased; cancellation wins. A failed summary can still permit a retry if pruning already committed useful reduction. Default overflow retries are one. [Recovery](https://github.com/deepseek-ai/deepseek-harness/blob/22ddee3792402c3e70ec5f86a63c3e3e895a2f60/packages/compaction/compaction-basic/src/index.ts#L214), [defaults](https://github.com/deepseek-ai/deepseek-harness/blob/22ddee3792402c3e70ec5f86a63c3e3e895a2f60/packages/compaction/compaction-basic/src/config.ts#L105) | Retry a changed request, with explicit reason and bound. Do not repeatedly resubmit the same overflowing envelope. |
| Compaction checkpoint, not transcript destruction | Codex installs replacement history plus context metadata, then recomputes usage. Resume reconstruction can start from a persisted replacement-history checkpoint and replay its suffix. [Installation](https://github.com/openai/codex/blob/603a7baff0f6f37ee9a8a9375a272eaab03d3ece/codex-rs/core/src/compact.rs#L406), [replay](https://github.com/openai/codex/blob/603a7baff0f6f37ee9a8a9375a272eaab03d3ece/codex-rs/core/src/session/rollout_reconstruction.rs#L493) | Persist the complete conversation separately from the compact request projection; a summary needs source boundaries and a reproducible suffix. |
| Recoverable output retention | DSH spill policy saves the complete text before replacing model-facing content with bounded head/tail plus retrieval instructions. Failure to store keeps the original content. Its model-free pruner appends replacement events with original-event references instead of rewriting earlier events. [Spill](https://github.com/deepseek-ai/deepseek-harness/blob/22ddee3792402c3e70ec5f86a63c3e3e895a2f60/packages/spill/spill-policy/src/index.ts#L102), [pruner](https://github.com/deepseek-ai/deepseek-harness/blob/22ddee3792402c3e70ec5f86a63c3e3e895a2f60/packages/compaction/compaction-tool-result-pruner/src/index.ts#L130) | Store full output once; project concise previews with an actual retrieval handle, byte/line positions and omission metadata. |
| Request retries differ from successful steps | DSH persists failed assistant attempts separately from successful messages; interrupted visible blocks are marked interrupted. Pending tool results are settled on step failure. Retry events are appended before cancellable backoff, with provider policy and counters. [Agent step](https://github.com/deepseek-ai/deepseek-harness/blob/22ddee3792402c3e70ec5f86a63c3e3e895a2f60/packages/core/agent-loop/src/agent.ts#L356), [stream settlement](https://github.com/deepseek-ai/deepseek-harness/blob/22ddee3792402c3e70ec5f86a63c3e3e895a2f60/packages/core/agent-loop/src/agent.ts#L484), [retry scheduling](https://github.com/deepseek-ai/deepseek-harness/blob/22ddee3792402c3e70ec5f86a63c3e3e895a2f60/packages/llm/llm-retry/src/index.ts#L145) | Track attempts, completed calls, actual tool execution and durable outcomes independently; reconnect must not execute a write twice. |
| Durable objective with bounded continuation | DSH stores goal phase, identity, revision and admitted round count separately from process-local armed/disarmed state. The driver flushes before continuation, checks idle/no competing prompt, waits for children where configured, and blocks at its round cap. [Goal types](https://github.com/deepseek-ai/deepseek-harness/blob/22ddee3792402c3e70ec5f86a63c3e3e895a2f60/packages/goal/goal/src/types.ts#L43), [driver](https://github.com/deepseek-ai/deepseek-harness/blob/22ddee3792402c3e70ec5f86a63c3e3e895a2f60/packages/goal/goal-round-driver/src/index.ts#L103) | A finished response can be one work round; continue only for an active authorized task, with a finite shared budget and current objective revision. |
| Restart is an explicit boundary | DSH goal activation starts disarmed, while jobs-local explicitly keeps handles, rings and cursors in memory. Codex daemon recovery captures only recorded regular-turn input, an uncancelled task and a supported single local environment. [Activation](https://github.com/deepseek-ai/deepseek-harness/blob/22ddee3792402c3e70ec5f86a63c3e3e895a2f60/packages/goal/goal/src/index.ts#L484), [jobs](https://github.com/deepseek-ai/deepseek-harness/blob/22ddee3792402c3e70ec5f86a63c3e3e895a2f60/packages/jobs/jobs-local/src/index.ts#L1), [Codex recovery](https://github.com/openai/codex/blob/603a7baff0f6f37ee9a8a9375a272eaab03d3ece/codex-rs/core/src/session/daemon_recovery.rs#L20) | Rehydrating a transcript does not prove a command survived. Restore a resumable task and reconcile uncertain work before further mutation. |

Codex also has explicitly configurable retry exceptions: ordinary retry limits,
server timing and transport fallback coexist with a feature-gated unbounded
connection-retry path. Do not describe its retries as universally bounded or
copy that exception into Dolores. [Retry implementation](https://github.com/openai/codex/blob/603a7baff0f6f37ee9a8a9375a272eaab03d3ece/codex-rs/core/src/responses_retry.rs#L92)

## Recommended implementation order

These are recommendations inferred from the source, not claims about current
Dolores implementation or guarantees furnished by either upstream harness.

1. **Carry bounded working state before increasing autonomy.** Persist a compact
   task record with objective and user constraints, current step, completed work
   with evidence/artifact references, unresolved failures, next concrete action,
   remaining shared budgets, and task revision. Keep user-authored instructions
   authoritative. Rebuild the next request from that record, a recent complete
   tail, and retrieved evidence. Do not preserve an ever-growing prose diary in
   every request. DSH's continuation prompt explicitly asks the agent to inspect
   workspace/tool/durable state instead of trusting earlier narration.
   [Prompt](https://github.com/deepseek-ai/deepseek-harness/blob/22ddee3792402c3e70ec5f86a63c3e3e895a2f60/packages/goal/goal-round-driver/src/prompt.ts#L12)
2. **Add recoverable output limits.** Write full tool output to owned session
   artifacts; expose a bounded preview plus `artifactId`, size, truncation flag
   and retrieval instructions. A read tool should support offset/limit or line
   ranges and an optional text search. Count retrieved slices in the same
   request estimator. Prefer explicit missing-artifact errors over fabricated
   continuity. This is stronger than a lossy head/tail buffer alone.
3. **Compact safely at request boundaries.** Prune oversized bodies, remeasure,
   and preserve mandatory task state plus complete call/result groups. If a
   summary backend is added, validate nonempty output, original boundary/revision
   and smaller resulting estimate before atomically installing it. Preserve the
   old checkpoint on failure. Reserve capacity for the compaction request itself;
   a summarizer cannot rescue a request that also exceeds its own model limit.
4. **Continue work rounds with one shared budget.** A round may end because the
   model returned text while a named task remains active. After a durable
   checkpoint, allow one next round if no user input/Stop superseded it, budget
   remains and no terminal outcome occurred. Count compaction, retries and
   continuations in actual resource accounting. Do not reset budgets or the
   repeated-failure watchdog when a new round begins. Keep completion tied to
   observed acceptance evidence; exhaustion produces a resumable incomplete
   result. Codex's separate cumulative rollout budget demonstrates that task
   spend is distinct from current context occupancy.
   [Budget](https://github.com/openai/codex/blob/603a7baff0f6f37ee9a8a9375a272eaab03d3ece/codex-rs/core/src/rollout_budget.rs#L18)
5. **Make pending commands and progress inspectable.** Return job handles after
   a bounded wait; poll only new output with an explicit cursor and cancellation.
   Persist command metadata and artifact output separately from volatile process
   handles. Keep lifecycle outcome and exit success distinct. Codex caps buffers
   and returns interactive process handles; DSH jobs keep a consuming model cursor
   and separate nonconsuming reads. A healthy build waiting for output is not
   automatically a failed/no-progress model loop.
   [Codex exec](https://github.com/openai/codex/blob/603a7baff0f6f37ee9a8a9375a272eaab03d3ece/codex-rs/core/src/unified_exec/mod.rs#L1),
   [DSH cursor](https://github.com/deepseek-ai/deepseek-harness/blob/22ddee3792402c3e70ec5f86a63c3e3e895a2f60/packages/jobs/jobs-local/src/index.ts#L88)
6. **Recover without replaying uncertain mutations.** Record a tool attempt
   before dispatch and its result after execution; flush durable checkpoints
   before announcing recoverability. On restart, identify pending operations and
   reconcile workspace/job evidence. Resume only the missing next action.
   DSH's persistence contract distinguishes acknowledged append from a crash
   durability barrier; its JSONL backend fsyncs batches, rolls back partial writes
   and repairs torn tails. Equivalent invariants matter more than adopting JSONL.
   [Storage contract](https://github.com/deepseek-ai/deepseek-harness/blob/22ddee3792402c3e70ec5f86a63c3e3e895a2f60/packages/session/session-persistence/src/handle.ts#L91),
   [backend](https://github.com/deepseek-ai/deepseek-harness/blob/22ddee3792402c3e70ec5f86a63c3e3e895a2f60/packages/session/session-persistence-jsonl/src/index.ts#L1409)

## Verification and remaining unknowns

Fixture checks should span multiple context windows and work rounds, with a late
step depending on an early constraint/result. Include: artifact paging without
repeated prefixes; overflow recovery that changes the request exactly once;
failed/empty/nonshrinking summary rollback; preserved call/result pairing;
checkpoint-write failure; Stop during summary/backoff/polling; user steering
invalidating a queued continuation; and restart between a write's dispatch and
result persistence, proving that the write is not automatically replayed.

Native command tests should separately verify output cursors, process exit,
timeout, cancellation and app restart. Only a bounded live-model task can establish
the selected provider's practical adherence, summary quality and multi-hour
completion behavior. Neither source inspection nor passing fixtures supplies
that evidence. No upstream runtime, test suite or live model was run for this note.
