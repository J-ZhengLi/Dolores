# Dolores roadmap

**Status: proposal for user review, 2026-10-04.** Brick 8.4, cross-platform build CI, is **skipped**. Future bricks below are planned scope, not authorization to implement them. Present this plan and wait for the user to choose a direction.

## Product destination

Dolores should become a lightweight desktop agent that finishes useful work in a project, explains its progress and limits, and improves from experience. Flutter remains the selected UI; Rust owns the runtime and replaceable interfaces. The system theme and Codex-inspired project/temporary/side-chat flow remain the UI foundation.

The recommended next cycle prioritizes **useful task execution**, then **effective context**, then **learning from outcomes**, then **consistent plugins and resource improvements**. Release engineering is deferred. This order is a proposal, not an approved commitment.

Success means a user can open a project, assign a bounded task, watch meaningful progress, recover from a limit without losing work, and get a result supported by actual checks. Learning means retained knowledge or workflows that measurably help later tasks; it does not mean changing model weights.

## Starting point

| Implemented foundation | Remaining limitation |
| --- | --- |
| Project/temporary/side chats, streaming, model discovery and profiles | Small run, file and command bounds constrain whole tasks. |
| Reviewed file tools, commands, journal/revert and local MCP | Every invocation needs approval; larger projects and long checks are awkward. Commands/MCP are not folder sandboxes. |
| Context inspection, estimates, manual summaries and explicit continuation | Long tasks need manual summary and recovery management. Estimates are not exact model tokenization. |
| Automatic explicit-preference learning and reviewed global/project skills | Project knowledge and experience-based learning are absent; task feedback is not learning input. |
| Local outcomes, instruction comparisons and regression runner | Literal tool-free response tests do not establish whole coding-task success. |
| Windows preview, notices, startup guidance and composer improvements | Low-end resources, real IME/screen readers and other platforms remain unverified. |

[Implementation history](IMPLEMENTATION_HISTORY.md) preserves bricks 1–8.3. [Acceptance](ACCEPTANCE.md) records actual results. Completed bricks do not certify every broader product goal.

## Planned sequence

| Milestone | User-visible outcome | Planned bricks | Dependency and exit decision |
| --- | --- | --- | --- |
| **9 — A dependable working agent** | Complete and verify a modest multi-file task with understandable permissions, budgets and recovery | 9.1 baseline; 9.2 run controls; 9.3 approval policy; 9.4 larger files/checks; 9.5 checkpoints | Start here. Diagnose measured failures before selecting defaults; assess the complete workflow before adding learning. |
| **10 — Context for real projects** | Work beyond one short conversation without repeated discovery or silent loss of context | 10.1 guidance/skill selection; 10.2 context-pressure handling | Depends on 9's task state and policies. Review automatic sharing and summarization. |
| **11 — Learning from outcomes** | Retain useful project facts and propose tested workflows from experience | 11.1 project knowledge; 11.2 experience-to-skill loop | Needs trustworthy task evidence from 9 and context selection from 10. Keep only measured useful changes. |
| **12 — A lean, extensible harness** | Consistent plugin lifecycle and predictable cost when features are unused | 12.1 plugin registry; 12.2 resource improvements | Resource checks run throughout. Optimize measured costs; review any new plugin execution model separately. |

Each sub-brick produces one reviewable change and commit. Each milestone ends with an integration check. Later scope is planned now and reviewed at the preceding milestone's exit; new work requires an explicit roadmap revision. Dates are not promised before the baseline establishes the work size.

## Milestone 9 — A dependable working agent

### 9.1 Task and resource baselines

Create five bounded tasks on versioned synthetic projects: understand an unfamiliar project, repair a failing check, create/validate a small multi-file feature, edit a file beyond the current 16-KiB bound, and recover an interrupted/limited task. Publish fixture definitions and non-private aggregate results.

Classify failures by output/context, reasoning versus visible output, model-call/tool counts, file bounds, command timeout/capture and approval overhead. Measure release startup, idle memory, streaming and long-history behavior with a declared machine/dataset. The provisional 150-MiB idle / two-second cold-start goals remain targets; prior Flutter observations exceeded the memory target.

**Exit:** every observed harness failure has a reproducible trigger and classified cause. Each task has an unchanged executable or human-review criterion. Separate deterministic results from bounded live Qwen routine cases and DeepSeek harder cases. Model failure does not justify weakening a task. No default increases here.

### 9.2 Configurable task budgets

Add coherent controls for model-call/tool-operation ceilings and total elapsed allowance, retaining per-model output/context settings. Show remaining capacity and actual pause reasons. Define continuation segment allowances and visible accumulated task usage/context. Select proposed defaults from 9.1 evidence and review them before implementation.

**Exit:** explicitly larger allowances are usable; exhaustion saves progress with a bounded next step. Test repeated exhaustion and Stop near a tool decision. No unlimited loop, hidden retry or automatic spending increase.

### 9.3 Explicit session approval policies

Keep review-every-call as the default. Add opt-in session permission for built-in reads/list/search inside the chosen working folder, with scope disclosure and visible revocation. Edits keep diff review; commands/MCP keep individual approval. Wider write/command trust is a separate decision.

**Exit:** repeated discovery works under an explicit grant. Test out-of-folder/link refusal and revocation before a queued call executes. Files, model output and receipts cannot grant permissions. Commands/MCP remain outside the folder-sandbox claim.

### 9.4 Larger files and practical verification commands

Add bounded ranged reads and snapshot-bound patches so common source files do not need complete inclusion in each request. Define Unicode, line-ending, stale-snapshot and partial-file semantics first. Make command timeout/capture configurable within explicit ceilings; oversized logs keep an inspectable bounded result. Preserve journaled file effects and identify unjournaled command effects.

**Exit:** the larger-file baseline task can be reviewed, applied and verified without truncating source or weakening conflict/uniqueness checks. A slow check finishes under an explicit allowance; oversized output has an honest useful outcome. Test changed files between read/apply and incomplete/erroring capture. Multi-file atomic transactions are excluded.

### 9.5 Durable task progress and checkpoints

Save the task goal, proposed steps, completed evidence and pause state with the conversation. A model may propose/update a short plan; its checkboxes are not execution evidence. Restore useful drafts/progress after restart. Distinguish completed, failed and uncertain effects; explicitly resume rather than replaying commands/writes.

**Exit:** restart preserves enough state to continue or inspect a multi-step task. Test interruption after a write but before reply storage, plus unavailable tools on resume. Rerun the five baseline tasks at milestone exit and compare completion, verification, approvals, usage and time under recorded settings. Separate model and harness failures.

## Milestone 10 — Context for real projects

### 10.1 Project instructions and relevant skills

Define ancestor/nested AGENTS.md discovery, precedence and review. Select a bounded set of task-relevant, already activated global/project skill snapshots; expose selections in context inspection. Discovery does not activate unreviewed files or execute referenced scripts. Avoid including every skill in every request.

**Exit:** two projects get their correct scoped guidance and relevant skills without leakage. Test same-name overrides, changed guidance and unrelated skills. Use bounded discovery/lazy loading; no resident indexer by default.

### 10.2 Context-pressure recovery

Add an opt-in managed-summary policy using existing summary/provenance machinery. Preserve the goal, unresolved work, instructions and relevant tool evidence while keeping the full transcript locally. Keep estimates separate from reported usage, configured model windows and the 128K blank default. Show summary coverage/omissions and bound the summary request itself.

**Exit:** a small configured window triggers visible recovery with provenance. Test failed summarization and a tool result that still cannot fit. Preserve work and offer manual summary/context settings; never retry indefinitely or silently delete history. Real-model summary accuracy is reported separately.

## Milestone 11 — Learning from outcomes

### 11.1 Useful project knowledge

Extend memory to scoped project facts: verified test/build commands, project structure and user-confirmed conventions. Keep sources/freshness, correction/disable/delete and a distinction between observed facts and inference. Define automatic-save eligibility first; uncertain or higher-risk facts remain suggestions. Arbitrary file/tool text is not trusted guidance.

**Exit:** a later task benefits from retained verified knowledge; changed commands/corrected conventions invalidate stale entries. Test contradictory evidence and protected manual corrections. Compare retrieval cost and task behavior with memory enabled/disabled. Preserve existing preference-learning safeguards.

### 11.2 Experience-to-skill loop

Use completed task evidence and explicit outcomes to propose workflows from successful repairs and recurring mistakes. Drafting is opt-in, bounded and traceable, using version review/rollback. Extend evaluation to isolated tool-using project fixtures so skills are assessed on task outcomes, beyond response snippets. Automatic activation is excluded.

**Exit:** a workflow helps a frozen task set and an unsupported/harmful proposal is rejected. Test false success claims and failed/truncated drafts without losing source tasks. Old skills survive rejection/rollback. Small-case improvement is not general competence.

## Milestone 12 — A lean, extensible harness

### 12.1 Consistent plugin registry and lifecycle

Unify metadata, capabilities, enable/disable state, configuration and lifecycle across provider/storage/credential/tool implementations. Keep a small kernel and optional features. Compiled Rust plugins and external MCP adapters remain distinct execution types with honest trust boundaries. Design bounded active-tool selection as catalogs grow.

**Exit:** a built-in adapter and local MCP adapter use the registry with clear version/configuration failures, lazy startup and independent failure handling. Test incompatibility and a crashing server while chat remains usable. Native dynamic loading, a marketplace and arbitrary plugin UI/code are excluded.

### 12.2 An agreed resource budget

Use 9.1 and per-brick measurements to address dominant allocations, history/render work and startup. Keep inspections/processes on demand. Test a reference low-end Windows machine with reproducible release/data scenarios; publish aggregates without machine/user identifiers. Account for child processes separately from local model hosting.

**Exit:** compare startup, idle/active memory, idle CPU and long-history responsiveness against agreed targets. Test long history and a large tool catalog. If Flutter cannot meet the desired footprint, present the measured tradeoff for a user decision; do not silently switch frameworks or infer low-end support from a development machine.

## Checks throughout

- Freeze scope and acceptance before implementation. Check the basic flow and one or two realistic failure/recovery cases per brick.
- Use bounded Qwen3.5-2B routine probes and DeepSeek V4.1 Flash harder probes with isolated data and unchanged selected settings. Fixtures and live evidence retain separate meaning.
- Preserve configuration/history, commit each completed brick in English, and build/visibly launch the normal app after each task.
- Measure changes to retained state, processes, catalogs/context or expensive UI work. Avoid default resident services, broad indexing and unbounded retries.
- Record actual acceptance. Failed cases stay failed until the same criterion passes; model prose is not verification evidence.

## Deferred scope and direction choices

Cross-platform support remains a requirement, but **8.4's platform CI is skipped**. Other-platform execution and native physical-keyboard/IME/screen-reader gates remain visible in acceptance. CI returns only through an explicit plan change, not under a renamed brick. Signing/public release, updater, cloud sync, multi-agent orchestration, local model hosting, remote/persistent MCP, full sandboxing and vector-search memory are unscheduled.

| Direction | Tradeoff |
| --- | --- |
| **Recommended: working-agent first** | Follow 9 → 10 → 11 → 12. Address execution constraints before more learning. |
| **Learning first** | Start with 9.1 and trustworthy task evidence, then prioritize 11.1. Skill outcome evaluation still needs isolated task execution. |
| **Footprint/plugin first** | Start with 9.1, then 12. Prioritize lean/modular behavior while task constraints remain visible. |

User review should choose a direction and the next milestone. At each milestone exit, review actual outcomes and explicitly revise the remaining plan. This proposal does not authorize deferred features.
