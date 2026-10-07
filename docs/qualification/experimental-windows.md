# Experimental window trial — 2026-10-07

**Disposition: held backend; single-window fallback retained.** The bounded trial
has finished. Detached group transfer and multi-window layout/rejoin (19.2/19.3)
are held, not implemented or accepted.

## Frozen backend and results

Only the maintained Flutter 3.47.5 stable SDK's same-engine experimental Windows
API was tried. No channel migration, additional engine, storage host or alternative
plugin. The [trial contract](../design/experimental-windows.md) froze an additional
empty-window ceiling of 128 MiB private memory and less than 1% idle CPU of one
core, with at most two repair iterations.

The first explicit diagnostic attempt encountered the binding's cached disabled
window owner. One initialization repair recreated the owner after enabling the
diagnostic-only flag. The second release attempt created two visible native
windows and two Flutter views with one NativeBridge/AppHost. Activation was
observed programmatically. Destroying the secondary returned to one native window,
kept the primary draft and allowed another native request. There were zero owned
descendants; the only profile owner remained in that same process. Saved primary
renders after failure/close were inspected.

| Settled short sample | Private memory | CPU (% of one core) |
| --- | --- | --- |
| Primary only | 243.84 MiB | 1.221% |
| Primary plus empty secondary | 310.67 MiB | 2.443% |
| Added window | 66.83 MiB | 1.222 percentage points |

Memory passes the ceiling; idle CPU fails the frozen **less than 1%** gate.
These are approximately four-second tail samples on this host, not sustained
performance qualification or a universal backend diagnosis. The earlier failed
initialization sample measured 242.61 MiB / 0.407%, illustrating baseline
variation. No target was relaxed and no repeat-until-pass procedure was used.
The backend remains held rather than claiming supported detach from native
window presence alone. SDK source also explicitly labels the API internal and
prohibits production imports; internal imports remain in the isolated diagnostic.

Physical keyboard/input, IME, accessibility, native multi-window theme transitions,
acknowledged dirty-file/diff/live-PTY transfers, monitor removal and failed rejoin
were not qualified. No source group was disposed and no work was moved during this
backend trial. Those dependent acceptance criteria remain held.

## Evidence and supported fallback

Reproduce using the maintained [build helper](../../scripts/desktop.py) with the
explicit `windows` diagnostic, then [trial helper](../../scripts/test-window-desktop.py).
Disposable public receipts: first
`output/window-qualification/bf9d87d34bbc4c3c8c22b41dd7eb98c5/report.json`, corrected
`output/window-qualification/b808efe71a89475c8723857ff7f15c49/report.json`.
Diagnostic receipts do not establish normal-app behavior.

19.4 exposes unavailable capability separately from the saved Multiple Window
preference. Default On and deliberately saved Off choices remain intact; saving
that preference does not create a window or extra host. Home, Folders, Source
Control and terminal splits keep their existing single-window owners. No
automatic backend retry, partial detach or new window-layout migration is added.
The normal release does not enable the SDK's experimental window flag.

147 bridge library tests and 303 Flutter tests pass; Flutter analysis and strict
bridge Clippy pass. Focused held-preference tests cover default On, Off restored,
stale save refusal and zero shell activation; widget checks retain Home/drafts and
failed-save recovery. The [release fallback helper](../../scripts/test-window-fallback.py)
checks saved Off in a fresh isolated native process; it is separate from physical
UI qualification.

That release C ABI fallback check passed in two fresh processes: default On,
unavailable capability, explicit Off restored, stale save refused and zero shell
or child-host activation. Public receipt:
`output/window-fallback-qualification/1e4dcd671f244e2290163de5ec4797c4/report.json`.
The normal main-entry Windows release was built and launched visibly using the
original profile. Its schema/table set remains 42 tables; 41 table hashes and
both existing editor-recovery rows match the earlier baseline. One new terminal
recovery row is retained rather than deleted to force a hash match. Provider,
model, history and existing recovery remain intact. Final handoff receipt:
`output/m18-m19-normal-handoff.json`. Native window presence is not a physical
input or exhaustive foreground UX check.
