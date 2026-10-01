# Dolores roadmap

Build one usable brick at a time. Each new brick needs a small design decision, scoped implementation, behavior checks, resource measurements where relevant, and an updated acceptance record. Do not equate compilation with product acceptance.

| Brick | Usable result | Acceptance boundary |
| --- | --- | --- |
| 1 — Conversation foundation | Desktop chat, replaceable provider/storage ports, OpenAI-compatible text streaming, stop, SQLite history, system theme | Local build and mock endpoint proof; live provider and OS checks recorded separately. |
| 1.1 — Resource decision | Working Iced/software UI reuses core/plugins; default development shell selected, Tauri retained | Windows release/controller/renderer comparison completed. Settled native idle near 30 MiB versus over 400 MiB webview working set. Low-end, input/accessibility and transient CPU acceptance remain gaps. |
| 1.2 — Flutter selection | Styled Flutter desktop chat sharing the Rust core/plugins through FFI; selected default | Windows build and visual/resource comparison completed; user selected Flutter. Cross-platform and low-end acceptance remain open. |
| 2.1 — Remembered connections | OS credential plugin, restart restoration, recovery retry, forget without history loss | Windows native-vault and separate-process authorization/restart checks; locked/missing vault and endpoint-binding tests. No plaintext fallback. |
| 2.1.1 — Model discovery and picker | Fetch models, enable a subset, switch beside the composer; manual fallback | Bounded authenticated listing, subset/active-model persistence, real selected-model request, restart and exclusion during generation. |
| 2.2 — History browsing and export | Browse beyond the current 100-session/40-turn view and export a chosen conversation | Stable ordering/pagination, complete export, bounded UI memory; history must not be silently truncated. |
| 2.3.1 — Markdown and code | Selectable formatted replies, code Copy, bounded rich rendering; editable headings and syntax-colored code cards in the composer | Literal HTML, no image I/O or automatic navigation, compact layout, streaming, editing/composition and canonical Markdown preservation. |
| 2.3.2 — Usage and context | Provider-reported per-reply counters, persistent request details, on-demand context inspector, context ring and trajectory/log views | Missing versus zero, complete-turn atomicity, restart/export, bounded newest history, independent inspection and actual server contract; no inferred tokens/pricing or fabricated historical timings. |
| 2.3 — Rich conversation | Markdown/code presentation, usage/context visibility | Rendering safety, server compatibility and accounting accuracy. Scope each feature separately. |
| 2 — Reliable workspace | OS credential storage, history pagination/export, Markdown/code, model discovery, usage accounting, context budgeting, retry guidance | Restart/recovery, secret storage, long-session behavior, Windows/macOS/Linux CI and measured release baseline. |
| 3 — Tools and bounded agent loop | Tool plugin port, explicit approval policy, visible tool results, max iterations/time/cost, filesystem read tools first | Deterministic tool protocol tests and prompt-injection cases; approval cannot be bypassed by model text. |
| 4 — Memory with provenance | User preferences and session summaries; inspect/edit/delete, source references, retrieval budget | Test usefulness, correction and deletion; secrets never enter memory by default. |
| 5 — Reusable skills | Propose a skill from successful work; user reviews before activation; version and roll back | Show outcome improvement on a frozen task set; no unreviewed generated code execution. |
| 6 — Plugin ecosystem | External protocol, capability declarations, version negotiation, install/enable/disable lifecycle | Crash, timeout, compatibility and resource tests. State what is isolated and what is not. |
| 7 — Evaluate and improve | Local feedback, task traces, regression suite, compare memory/skill versions | Promote a change only when measured success improves without violating safety/resource budgets. |

Bricks 1, 1.1, 1.2, 2.1, 2.1.1, 2.2, 2.3.1 and 2.3.2 are implemented. Brick 2 remains in progress. Learning, tools and third-party plugin loading are planned capabilities. Flutter is selected; unverified platforms and hardware are not labeled accepted.

## Next iteration

Next: provider request controls and recovery guidance within brick 2: configurable output limits and timeouts, explicit retry states and preserved unsent drafts. Usage/context visibility is implemented; model-specific token estimates and pricing remain outside this brick. Model discovery and Markdown/code rendering are implemented. Carry forward keyboard/IME/system-theme UAT, live-model verification, cross-platform CI and representative 2-core/4-GiB measurements. Flutter's earlier working-set observations exceed the provisional target; resource acceptance remains open. Keep each iteration small and update acceptance from evidence; the supplied CI matrix has not been executed.
