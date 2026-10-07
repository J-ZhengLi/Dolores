# Reliability follow-up — 2026-10-08

The development plan includes maintaining Python build/launch commands, removing retired
desktop experiments, and continuing the recorded reliability/qualification work
before proposing another roadmap. This is a work ledger, not a new feature plan.
Windows release CI (8.4) is now part of release preparation; non-Windows CI remains deferred.

Milestones 16–18 and 21–24 now have implemented scopes and bounded qualification.
Milestone 19 reached its documented held-backend fallback; 19.2/19.3 detach
transfers remain unavailable after the frozen CPU gate failed. This ledger's
broader model/platform/physical/human acceptance gaps remain open; they are not
unstarted feature batches. See [companionship](companionship.md) and
[closed-UI scheduling](closed-ui-scheduling.md) for the final AFK batch evidence.

| Work | Status |
| --- | --- |
| Python build/launch and retired-shell cleanup | Delivered; normal Windows build/native launch verified |
| Sustained idle CPU and resource baseline | Reproduced and narrowed; native profile/low-end qualification still open, see idle-resources.md |
| Computer-use observation/targeting reliability | Contract clarified and guards tested; frozen live cases still fail, see computer-use.md |
| Automatic preference extraction | Exact simple response preferences/corrections fixed and live-qualified; general-language model extraction remains open |
| Useful skill improvement and regression restoration | Missing approved evidence fixed; DeepSeek trials tied, so no activation; useful improvement/live restoration remain open |
| Browser installation convenience and bounded input | Python pinned installer and native fixture qualified; in-app installer/live model reliability remain open |
| Native theme/layout/input verification | Windows normal release reviewed in wide dark/light and narrow light; exhaustive nested matrix, physical IME/accessibility remain open |
| Current documentation paths/status | Current commands use Python; historical retired-shell evidence labeled; local links checked |

Each fix receives a separate English commit, basic and realistic failure/recovery
checks matching the change. Runtime compilation/packaging changes need a normal
build; launch or native UX checks are chosen when relevant. Provider settings,
history and selected model are preserved. Live cases use bounded isolated data;
private transcripts, captures and keys remain ignored. Failures remain failures.

Physical IME/screen-reader, representative low-end hardware and macOS/Linux native
execution require suitable hosts or human checks. They cannot be closed by Windows
fixtures. General native self-replacement and other deliberately deferred features
remain outside this follow-up; see the existing roadmap exclusions.

The authorized maintenance batch is concluded. The normal app is visibly available
with the original profile, and changes are committed separately. Removed retired
root web build/dependency caches as well as tracked experiments; the maintained
browser adapter's own dependencies remain. That maintenance batch introduced no new roadmap or platform CI. Windows CI was
subsequently added in release preparation. Open qualification gates above are not represented as passes and are
available for the user's next roadmap. See [learning follow-up](learning-followup.md)
for the reproduced defect, regression checks, live tie and extraction limits.
