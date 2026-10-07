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

## Availability and cost — 24.3

The final packaged build was sampled for 60 seconds visible and 60 seconds in
the tray after completing its task. Visible CPU was **0.156% of one core**;
tray CPU was **0.130%** (added **−0.026 percentage points**, below the <1-point
ceiling). Peak private memory was **241.965 MiB** visible and **244.367 MiB**
in the tray: **2.402 MiB added**, below the 16 MiB ceiling. No child process or idle model request was
observed. This measures added background cost, not a low total-memory claim or
all-day/low-end performance. Conditional scheduling uses the existing timer.

Four settings renders at 390/1040 pixels in light/dark themes were inspected with
`view_image`. Long availability explanations remain in hover help. Failed saves
retain the saved policy and allow retry; enabling/disabling is checked separately.
Lifecycle fixtures cover blocked recovery, failed tray creation and restoring a
hidden owner when disabled. Distinct public profiles ran independently.
The final UI recovery check also verifies that a failed tray hide displays its
actionable error on Home while preserving the visible window.

A configured **DeepSeek V4.1 Flash** completed one public chat-only report with the
normal UI hidden. One model call reported **5,204 input / 50 output / 5,254 total
tokens**, including 5,120 cached input tokens. Output was limited to 1,024 tokens,
with a 30-second provider timeout. Reopen retained the result/draft; Quit exited 0.
Creation and due time were fixtures. A broader earlier report request asked to
create `daily-report.txt` and stayed at approval; it did not complete. A separate
misconfigured isolated connection stopped before execution. Neither is a live
pass. The successful narrower request does not establish general scheduling or
report quality. Temporary isolated vault entries were removed; original profile
rows were unchanged by the live checks.

## Acceptance boundaries

Physical tray clicks, actual suspend/resume, sign-in startup, low-end machines and
non-Windows availability are not established. There is no startup installation,
OS wake promise or desktop notification. Powered-off/sleep availability remains
limited by milestone 22's late-skip/interruption policy, not a second hidden worker.
Final original-profile/build preservation is recorded in the latest acceptance.
