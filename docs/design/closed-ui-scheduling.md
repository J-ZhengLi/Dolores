# Closed-UI scheduling — milestone 24

## Frozen ownership and availability

The Windows implementation retains the existing supervised AppHost/Rust runtime
when its window closes to the tray. It does not start a second native host or
duplicate profile/vault owner. The tray is the chosen local worker's visible
ownership indicator. This is a same-owner availability implementation of milestone
24, with no claim that its Flutter engine has been removed from memory.

- Off by default, one `Run tasks in background` setting. Close hides the window
  only after a tray icon exists and local recovery is saved. Failed preparation
  leaves the UI open. Existing file/terminal recovery and Git review gates apply.
- Ordinary foreground runs/PTYs are stopped on Close after the normal review;
  scheduled owners continue with their immutable snapshots. Companionship stays
  quiet while hidden. No desktop notification or automatic effect approval.
- Tray Open and another launch for the same canonical profile restore that same
  window/owner. Distinct profiles retain independent owners. Windows mutex/window
  identity precedes engine initialization; the existing database lock remains the
  final single-owner guard. Crash releases OS ownership; the next deliberate
  launch reconciles saved interrupted claims without replaying uncertain effects.
- Tray Quit restores the UI for its normal saved-work/Stop review, then shuts
  down the scheduler and owned processes. Turning background mode Off prevents
  subsequent hide-to-tray; normal app-open scheduling remains available. Disabling
  from a restored UI preserves results and stops background ownership.
- No sign-in/startup installation is performed. The user must open Dolores once;
  Quit, sleep, power-off and offline availability remain explicit limits. No OS
  wake/lock bypass, scheduled Windows task, cloud service or security-policy edit.
- Existing 15-second conditional scheduler timer and bounded run queue are reused.
  No additional idle process or model request. Resource qualification targets
  less than one added CPU percentage point and at most 16 MiB added private memory
  versus the same app's visible idle state; total process cost is reported too.
  Resource failures hold the opt-in capability rather than relaxing its target.
- Non-Windows background availability stays unavailable until a native lifecycle
  implementation is qualified. Source/time/DST/late-skip/approval contracts from
  milestone 22 apply unchanged.

## Verification boundaries

### Catalog correction during qualification

The packaged Windows app has 15 built-ins when browser and desktop-access adapters
are available. Its former 14-registration bound rejected both scheduled and ordinary
project runs before model execution with a misleading snapshot error. The bound is
corrected to exactly 15, with an actionable external-connection overflow error.
Model, output, context, execution and approval allowances remain unchanged. A
15-entry regression and a 16-entry refusal retain the saved draft and run evidence.

Fixture clocks prove dispatch, late skips and owner recovery. A normal packaged
awake-host case must hide the UI, execute one public scheduled result, restore the
same process and retain that result/draft. Duplicate launch, stale owner/crash,
blocked hide/save, offline request and waiting approval are separate cases. No
automatic retry or wake promise. Native protocol checks are labeled distinctly
from physical tray clicks and human feedback; neither compilation nor a hidden
diagnostic window alone satisfies the normal-app execution criterion.
