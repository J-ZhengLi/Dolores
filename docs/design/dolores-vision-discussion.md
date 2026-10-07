# Dolores vision discussion

Recorded 2026-10-04 as a historical discussion proposal. The [architecture specification](evolving-harness-architecture.md), [behavior policy](dolores-behavior.md) and [current roadmap](../ROADMAP.md) supersede its planning sequence. This document preserves the design rationale; current implementation and release status belong in the roadmap and acceptance records.

## Product direction

Dolores should be a calm, kind, caring coding agent that gets work right, questions questionable assumptions, remembers useful answers, and improves its skills and environment from experience. Its Westworld inspiration is continuity and growth across iterations. The harness should support that behavior without claiming that software extensions establish consciousness or AGI.

Basic features requested: thread/context management with compaction; auto approval and full-access execution; subagents, browser use and web search; file/image attachments; and settings. Distinctive features requested: accurate self-inspection, controlled modification of harness components, self-evolving skills, meaningful questions, and a consistent humane personality.

## Feasibility and actual boundaries

| Requested behavior | Assessment | Required boundary or structural work |
| --- | --- | --- |
| Durable threads, resume/fork/compaction | Feasible | Persist task/event state separately from model-visible compacted context. Forking conversation history does not isolate working files. |
| Auto approval and full access | Feasible | Separate approval policy, OS execution containment, and spending/iteration budgets. Do not describe skipped prompts as sandboxing. |
| Subagents | Feasible; host refactor needed | Current host has one active-run slot. Introduce owned per-thread child runs, bounded concurrency, inherited permissions, cancellation, usage and explicit file ownership/conflict handling. |
| Browser use and web search | Feasible as optional tools | Lazy adapters and declared permissions. Browser processes have material resource cost; avoid an always-running bundled browser. Search/retrieved pages are data, not authorization. |
| File/image attachments | Feasible; message refactor needed | Current core Message stores text in a String. Add content parts, bounded local attachment storage/preview, provider modality declarations and explicit sharing. Text-only models need an explicitly selected vision/OCR adapter rather than invented image comprehension. |
| Settings | Feasible | Separate user defaults, project overrides and thread snapshots; expose their effective source. Include tools, budgets, permissions, models, attachments, personality, learning and update policy. |
| Read its own source and capabilities | Feasible | Version-matched source/catalog access, effective permissions/limits and enabled plugin inventory; load source on demand. Source text does not prove that a capability works or that a model understands it. |
| Modify and hot-reload itself | Feasible for designed extension surfaces | Needs versioned lifecycle/API, restricted trials, activation boundaries, health checks and last-working version. Arbitrary native kernel/release UI replacement is a separate rebuild/restart path. |
| Diagnose and improve skills | Feasible | Attribute failures before editing instructions; compare versioned candidates on unchanged real task criteria and retain rollback. One failure is not proof of a broken skill. |
| Ask and remember useful questions | Feasible | Verify cheap facts first, challenge consequential contradictions, ask targeted questions, preserve source/scope and distinguish a one-task answer from a durable preference. |
| Stable humane behavior | Feasible to influence and evaluate | Personality instructions, interaction policies, memory and UX can support warmth/judgment. Consistency depends on the selected model; neither feelings nor universally correct judgment can be guaranteed. |

## What the references contribute

[DSH and Claude Mods research](../research/evolving-harnesses.md) documents primary sources and limitations. DSH puts capabilities behind replaceable services/events and managed registration lifecycles. Claude Mods expose event middleware and session-approved turn-boundary reload. Both make narrow behavior changes practical without a complete harness fork. Neither makes generated code trustworthy or improvement automatic.

Codex documents durable thread start/resume/fork and asynchronous compaction. This supports the proposed separation of stored thread history from the model's current context. Borrow the lifecycle semantics while keeping Dolores's own provider/storage abstractions; adopting this design does not require routing Dolores through Codex. [Official App Server documentation](https://learn.chatgpt.com/docs/app-server).

Flutter's own hot reload is a debug-mode development mechanism. A distributed release needs its own extension interpreter/host or restartable module boundary; it cannot rely on arbitrary live edits to release Dart. [Flutter hot reload](https://docs.flutter.dev/tools/hot-reload).

## Candidate architecture

Keep a small host-controlled kernel for effective permission grants, cancellation, resource accounting, durable event integrity, extension loading and recovery. Make providers, tools, context policies, skill selection, learning strategies and permitted presentation contributions versioned extensions. Keep user identity/preferences and evaluator authority outside a candidate's power to silently redefine them.

The kernel is readable and diagnosable. Dolores may propose a kernel patch, but applying a live kernel update is a separate reviewed rebuild/restart, with migration/recovery checks. Protected means protected from automatic self-activation, not hidden from inspection.

Give Dolores an introspection tool exposing the running build/version, active extensions, actual available tools, effective permission policy, model capabilities, current limits and source references. Distinguish installed, enabled, unavailable, permitted and empirically tested. A source bundle must match the running build; reading the latest repository checkout alone can diagnose the wrong binary. Credentials/private user data are not part of source introspection.

Use ordered typed events for configurable stages such as context preparation, tool proposal/result, task completion, learning and UI contribution. Specify ownership/disposal, cancellation, event ordering, state schemas and failure behavior. Pin extension versions for an in-flight run. Activate a candidate at a safe boundary; make migrations and dependency changes explicit. Rollback restores code/config/state where designed, not files, messages or other external effects already produced.

For initial executable mods, investigate a capability-limited script/Wasm runtime or a restricted worker boundary. Benchmark footprint and startup before choosing an engine. A separate process alone is not a sandbox. Agent-authored code should start with mocked external actions, no direct credential access and bounded CPU/memory/time; real integration trials need explicit host-granted capabilities. Declarative Flutter contributions can add supported cards/actions/panels; arbitrary native UI changes follow the build path.

**Full access and a protected kernel create a real tradeoff.** If a shell/process has the same OS identity and unrestricted write access to all harness files, an application-level plugin rule cannot make those files untouchable. Strong separation requires OS permissions/containment or a separate privileged broker. An initial policy-only boundary must be honestly labeled. An agent may propose increased permissions, but cannot grant them to itself. Ordinary task permission and self-update authority are independent settings.

## Experience and skill improvement loop

Observe a meaningful failure or user correction, then classify the likely source: request premise, skill, missing context, model response, unavailable tool, provider fault or harness limit. Reproduce where possible. A runtime-imposed limit cannot be fixed by rewriting a skill to ignore it.

For a likely skill fault, preserve the exact version and relevant evidence; propose a targeted edit with a reason; run the original case plus a regression/held-out case using recorded model/settings/budgets; compare task outcomes, costs and regressions; activate under the user's update policy; retain the previous version and monitor later results. Candidate authors must not silently change the acceptance criteria or grant their own permissions.

Improve the existing skill path before broad self-modification. It already has retained versions and outcome-comparison concepts, but current response-snippet checks are insufficient to judge whole coding workflows. Experience-based learning needs trustworthy command/file/task receipts and correction when earlier conclusions become stale.

Prefer event-driven, bounded reflection after meaningful tasks over an always-running process that changes something every day. Sometimes retaining the current behavior is the right conclusion. Fixed model weights and noisy evaluation mean no guarantee of improvement on every day or task.

## Questioning and personality

Questioning is part of execution judgment, not just tone. The product design calls for discussing the reasoning before most implementation work. Before implementation, verify available factual premises and briefly explain the proposed approach, evidence, tradeoffs and uncertainties. For a consequential mismatch, give the observation, implication and a focused question. Discussion does not require a separate approval for every routine edit: agreed work can proceed, while unresolved decisions that would materially change the outcome need an answer. Do not ask about facts a cheap local check can establish, or turn every task into an interview.

Example: if asked to improve a Rust app while the repository is Flutter with a Rust backend, explain that distinction and ask which performance surface matters before refactoring. If a skill mandates a command that repeatedly fails because the project changed, question the skill instead of exhausting the same retry pattern. Correction should identify what changed and move work forward without repeated apology loops.

Proposed stable traits: calm language under pressure; honest uncertainty and evidence; kind, respectful disagreement; care for the user's time, budget and saved work; curiosity about consequential missing information; and continuity through corrected preferences and project knowledge. Warmth should not become false agreement, unsupported claims of emotions or pressure to form a relationship. Self-generated changes may adapt workflows and user-approved tone, but should not silently redefine these values.

## Selected direction and decisions still open

The design adopts automatic activation of low-risk changes after tests, with rollback, and discussion of reasoning before most implementation work. These are product decisions, not authorization to start implementing this proposal. Full-access execution is requested as a feature, not granted to every future self-update trial.

Define low risk by the change's capabilities and impact, not by whether it is called a skill or plugin. Proposed automatic activation applies only to narrowly scoped, versioned changes within an existing host-approved capability envelope, with fixed acceptance criteria, a recoverable state transition and no new external effects. Permission policy, credential handling, the kernel, evaluator criteria, new dependencies/capabilities, destructive migrations and changes with irreversible effects require review. A wording-only skill change can still alter execution safety and must be evaluated accordingly. The host enforces this classification; the candidate cannot label itself low risk. This precise boundary remains to be discussed.

Other decisions: initial executable extension runtime; supported UI contribution surface; session/project/global update scope; permission-policy versus strong OS isolation for the first version; triggers/budget for reflection; and what evidence counts as an improved coding skill.

## Proposed planning consequence

The previous roadmap put plugin unification late. This vision needs extension/authority contracts, thread state and personality/question behavior designed early. Discuss a revised sequence: architecture contracts and self-inspection; everyday thread/permission/settings/attachment flows; optional advanced tools; evidence-based skill adaptation; then executable self-modification. Questioning/personality should be present from the first stage, not reserved for a final polish pass. These stages are discussion candidates and do not approve implementation.
