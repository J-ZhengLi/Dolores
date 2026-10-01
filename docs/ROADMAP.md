# Dolores roadmap

Build one usable brick at a time. Each new brick needs a small design decision, scoped implementation, behavior checks, resource measurements where relevant, and an updated acceptance record. Do not equate compilation with product acceptance.

| Brick | Usable result | Acceptance boundary |
| --- | --- | --- |
| 1 — Conversation foundation | Desktop chat, replaceable provider/storage ports, OpenAI-compatible text streaming, stop, SQLite history, system theme | Local build and mock endpoint proof; live provider and OS checks recorded separately. |
| 1.1 — Resource decision | Compare Tauri with a small native Rust UI using the same core; finalize the desktop shell | First Windows webview observation is 421.80 MiB summed working set / 189.34 MiB private bytes, above the provisional working-set budget. Test comparable release builds on representative hardware; record the tradeoff before advancing. |
| 2 — Reliable workspace | OS credential storage, history pagination/export, Markdown/code, model discovery, usage accounting, context budgeting, retry guidance | Restart/recovery, secret storage, long-session behavior, Windows/macOS/Linux CI and measured release baseline. |
| 3 — Tools and bounded agent loop | Tool plugin port, explicit approval policy, visible tool results, max iterations/time/cost, filesystem read tools first | Deterministic tool protocol tests and prompt-injection cases; approval cannot be bypassed by model text. |
| 4 — Memory with provenance | User preferences and session summaries; inspect/edit/delete, source references, retrieval budget | Test usefulness, correction and deletion; secrets never enter memory by default. |
| 5 — Reusable skills | Propose a skill from successful work; user reviews before activation; version and roll back | Show outcome improvement on a frozen task set; no unreviewed generated code execution. |
| 6 — Plugin ecosystem | External protocol, capability declarations, version negotiation, install/enable/disable lifecycle | Crash, timeout, compatibility and resource tests. State what is isolated and what is not. |
| 7 — Evaluate and improve | Local feedback, task traces, regression suite, compare memory/skill versions | Promote a change only when measured success improves without violating safety/resource budgets. |

Brick 1 is the scope of the initial implementation. Learning, tool execution and third-party plugin loading are planned capabilities, not current features. Later bricks are intentionally not built before their decisions and acceptance boundaries are agreed.

## Next iteration

Complete brick 1.1 before adding feature weight: compare native and webview shells, then verify the chosen shell with the user's endpoint and representative hardware. Use those results to refine brick 2. The current functional build is available for review, but resource and cross-platform acceptance are open.
