# Windows idle investigation — 2026-10-06

Read-only ten-second process-time samples reproduced activity without a running
task. CPU values are percentages of **one core**, not of the entire machine.
There was no model request and no desktop helper left running during these samples.

| Window/profile | CPU, one core | Interpretation |
| --- | --- | --- |
| Normal app, existing history | 5.94% | Reproduces the earlier sustained activity |
| Normal app, empty disposable profile | 3.12%, 4.06%, 4.37% | A long conversation is not required |
| Minimal diagnostic Flutter Material window | 1.87%, 1.72% | Activity also exists without the bridge, composer or custom title bar |

The minimal diagnostic did not contain Dolores logic. It was not handed off as
the normal app; its temporary source was restored and the normal release rebuilt.
Different windows and profiles are not a matched performance benchmark. These
samples localize part of the activity to the framework/host baseline, but do not
identify the remaining difference or prove a fix. A native CPU profile and a
representative low-end host are still needed before changing rendering behavior.

The actual app's idle widget check passes: no transient animation callback or
scheduled frame remains after settling. The context ring is determinate when
idle, and chat polling is guarded by an active run. This does not establish native
idle cost or exclude cursor, plugin, engine or graphics-driver activity.

To reproduce on the normal Windows build:

```text
python scripts/desktop.py build
python scripts/desktop.py launch --data-directory <absolute-profile-directory> --pid-file <absolute-output-record>
python scripts/measure-desktop.py --pid-file <same-record> --seconds 15 --result <absolute-output-result>
```

Record focus and concurrent host activity. Sampling can miss short-lived helper
processes. The optional CPU threshold is a diagnostic aid, not a newly agreed
release criterion. Build/launch verifies normal versus diagnostic entry identity;
a missing, changed or diagnostic build requires a deliberate rebuild or explicit
diagnostic opt-in. No framework dependency or numerical default was changed.
