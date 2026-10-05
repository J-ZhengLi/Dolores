# Executable mods: runtime gate and ABI 1

Milestone 13 decision, 2026-10-05. The first supported executable seam is a
**stateless recovery-hint hook**, plus a bounded declarative inspector card.
It improves recovery guidance without replacing execution, permissions, context,
evaluators or the composer. Wider hooks need a new ABI and qualification.

## Runtime comparison

Run `cargo run -p dolores-mod-runtime --release --example compare`, build the
`wasm_probe` and `rhai_probe` examples, then run
`powershell -File scripts/measure-mod-runtime.ps1`. The probes exercise the same
scalar classification workload; their sizes are separate executable measurements,
not the incremental size of the app. Rhai is a development-only dependency.

| Candidate | Authority and interruption | Decision |
| --- | --- | --- |
| Capability-limited Rhai | No registered file/network/process calls; operation and value limits must be configured. Native functions would need separate bounds. | Compared, not shipped. |
| Restricted Wasmi interpreter | No imports/WASI/broker; no memory/tables; fresh store; 10,000 fuel per invocation. | Selected for ABI 1. |
| Plain native worker | Can read an owned sentinel without a broker; process ownership is not OS containment. | Rejected for generated execution. |

The VM boundary is interpreter enforcement, not an OS sandbox for the host.
No generated code can make OS calls through this ABI on any OS. Host
commands/MCP still execute with account authority. Windows is exercised here;
other-platform runtime qualification remains open. No unsupported backend may
substitute native execution. Engine vulnerabilities remain outside this bounded
envelope. Wasmi's [fuel API](https://docs.rs/wasmi/0.46.0/wasmi/struct.Store.html)
and [store limits](https://docs.rs/wasmi/0.46.0/wasmi/struct.StoreLimitsBuilder.html)
provide the enforcement primitives. Rhai's [operation limits](https://rhai.rs/book/safety/max-operations.html)
explain why registered host functions require their own limits.

## ABI and fixed limits

Source is WebAssembly text, at most 8 KiB; compiled bytes also at most 8 KiB.
Export `recovery_hint(i32) -> i32`. Input and output are host-owned enums:
0 inspect, 1 output/explicit continuation, 2 context/context review,
3 tool budget/task limits review, 4 denied/permissions review,
5 interrupted/checkpoint review. Unknown outputs refuse. No arbitrary instructions
enter context. No secrets/private text/paths enter the VM. Fresh stores discard
globals after each call. No state migration or dependency loading is supported;
schema/API mismatch refuses rather than guessing.

Fuel interrupts runaway bytecode, including module start. Stop is checked before
and after the short fuel-bounded call; this is not preemption during native
compilation. Source/byte limits bound parsing; separately measure pathological
cases. Nothing stays resident between inspector/run invocations. Identity is a
SHA-256 digest of exact source plus the validated manifest's immutable fields.

## Implemented lifecycle (13.2)

Project-scoped immutable versions, host-owned fixed cases, baseline comparison,
revision checks, safe-boundary activation, append-only receipts and quarantine.
SQLite transactions combine pointer/receipt changes; uncommitted effects roll
back on restart. Runs pin source versions; activation cannot mutate an active
run. Health failure restores the retained baseline before new use. No destructive
migrations. Missing/unreadable mod state leaves ordinary chat available.

Generated source is untrusted. Fixed evaluator criteria are supplied by the host,
never candidates. Automatic activation is a separate explicit opt-in, default
off; only a strict all-pass improvement within the same ABI/capability envelope
qualifies. Kernel changes stay ordinary reviewable source diffs/build/restart.
Cards planned for 13.3 use existing Flutter surfaces, literal bounded title/body, and only
host-resolved review actions. No arbitrary Dart/HTML, URLs or injected handlers.
This narrow corpus does not establish general automatic self-improvement.
