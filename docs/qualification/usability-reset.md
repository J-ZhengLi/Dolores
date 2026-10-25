# Usability reset qualification

Scope: Windows x64 session reuse, memory defaults, Browser setup, notifications,
General settings and close/tray behavior. The implementation plan is
[usability reset](../plans/usability-reset.md).

Batch A passes isolated public C ABI save/reopen checks for session identity,
draft retention, absent-policy defaults, preserved Off choices and explicit real
adapter installation without a browser/model request. Staging tests cover setup
failure, cancellation, invalid contents and unrelated destinations. Reviewed
compact Light/Dark renders show actionable setup with optional technical details.
This does not qualify arbitrary websites, clean machines or learning usefulness.

Batch B fixture checks cover sequential toast geometry, stable underlying layout,
expiry/swipe, unread history, bounded queue, scrollable details, reduced motion,
explicit Retry, remembered close choices/reset and saved-policy failures. Store
checks cover old notices/claims, coalescing, retention, restart and preservation of
task evidence/drafts while clearing read history. Light/Dark saved renders cover
the compact toast and expanded details.

Batch B passes the normal Windows build, public C ABI save/reopen checks and a
rendered Model settings Save > toast expiry > bell > details journey connected to
the normal Rust/SQLite bridge. The form stays fixed, its model value is saved,
expiry retains the unread notice and details marks it read. Evidence:
`output/usability-b-native-final/receipt.json` and
`output/usability-b-native-ui-delivered/notifications-receipt.json`.

A real SQLite write-lock fixture also passed bounded Quit recovery while retaining
the previously saved draft and database integrity (`output/usability-b-close-recovery/`).

The normal app completed one public fixture report while hidden, retained its
draft/result, restored the same process on a second launch and quit cleanly.
Explicit Quit with background scheduling Off also released profile ownership.
These native lifecycle-message checks are recorded in `output/usability-b-tray/`
and `output/usability-b-quit/`; they do not claim physical tray-menu clicks.
The report used three loopback fixture requests and no live model.

Native desktop visual inspection was limited by the computer-use capture helper:
`FrameArrived timed out` / `window capture timed out`; a Flutter accessibility
index also expired before input. Saved Light/Dark/compact renders and the native
bridge/rendered journey are passing evidence, not physical-input or native-frame-
timing acceptance. Shell acceptance does not establish actual Windows alert
display or reading. Broader EH10, clean-machine and general accessibility gates
remain open; no private profile or live-model evidence is implied.
