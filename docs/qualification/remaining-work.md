# Reliability follow-up — 2026-10-06

The user authorized maintaining Python build/launch commands, removing retired
desktop experiments, and continuing the recorded reliability/qualification work
before proposing another roadmap. This is a work ledger, not a new feature plan.
Platform CI (8.4) remains deferred until explicit user instruction.

| Work | Status |
| --- | --- |
| Python build/launch and retired-shell cleanup | Delivered; normal Windows build/native launch verified |
| Sustained idle CPU and resource baseline | Reproduced and narrowed; native profile/low-end qualification still open, see idle-resources.md |
| Computer-use observation/targeting reliability | Pending diagnosis; frozen criteria retained |
| Automatic preference extraction | Pending diagnosis; preserve verified-only storage |
| Useful skill improvement and regression restoration | Pending live qualification |
| Browser installation convenience and bounded input | Pending |
| Native theme/layout/input verification | Pending available-host checks |
| Current documentation paths/status | In progress |

Each fix receives a separate English commit, basic and realistic failure/recovery
checks, and a normal build/visible launch where runtime changes. Provider settings,
history and selected model are preserved. Live cases use bounded isolated data;
private transcripts, captures and keys remain ignored. Failures remain failures.

Physical IME/screen-reader, representative low-end hardware and macOS/Linux native
execution require suitable hosts or human checks. They cannot be closed by Windows
fixtures. General native self-replacement and other deliberately deferred features
remain outside this follow-up; see the existing roadmap exclusions.
