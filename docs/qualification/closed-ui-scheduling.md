# Closed-UI scheduling qualification — milestone 24

Windows uses the [frozen same-owner tray contract](../design/closed-ui-scheduling.md).
Background mode is Off by default. These are normal main-entry packaged checks in
public disposable profiles; native lifecycle messages exercise production handlers,
not physical tray-menu clicks. Due timestamps are fixtures, not real-time schedule
interpretation or OS wake evidence.

## Execution and recovery — 24.2

- One public report completed while its window was hidden; reopening retained its
  result and Home draft. A second executable launch restored the original PID/window
  and exited without opening a second runtime/store. Tray Quit returned exit code 0.
- A provider 503 retained one failed occurrence and the Home draft. Reopening,
  forced owned exit and deliberate relaunch preserved that occurrence without a
  duplicate. No automatic retry was added.
- An unapproved file read stayed `waitingForApproval` while hidden. Reopen retained
  review; forced exit/relaunch marked the occurrence interrupted without invoking
  the tool or replaying it. The public file remained unchanged.
- Background Off followed by ordinary Close exited cleanly and released profile
  ownership. This preserves completed results; disabling does not erase tasks or
  prevent normal app-open scheduling.
- An in-flight list refresh now settles before Close. A bounded two-second wait
  prevents dispatch/close overlap; a blocked operation leaves the UI open for retry.
  File recovery failure/retry and scheduler suspension regressions pass.

Two real failures were repaired before acceptance: the full packaged catalog has
15 built-ins, exceeding its older 14-entry bound; and default derived-window
destruction left a native channel callback alive, producing exit `0xc000041d`.
The catalog bound is exactly 15 with actionable overflow recovery. Explicit window
teardown/unregistration resolves the Quit failure. Earlier failed runs are not
passing lifecycle or resource evidence.

Reproduce with `scripts/qualify-background-scheduling.py` (normal, offline and
approval modes) and `scripts/qualify-background-disable.py`, using fresh directories
under ignored `output/`. Receipts retain public results/states and no credentials
or original transcripts.

## Acceptance boundaries

Physical tray clicks, actual suspend/resume, sign-in startup, low-end machines and
non-Windows availability are not established. There is no startup installation,
OS wake promise or desktop notification. Powered-off/sleep availability remains
limited by milestone 22's late-skip/interruption policy, not a second hidden worker.
Original profile preservation and final build/resource/live results are recorded
separately as the remaining qualification brick completes.
