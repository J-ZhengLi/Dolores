# Computer use — selected-window observation and input (14.1–14.2)

## Decision before implementation

Keep Flutter/Rust. Use Windows Graphics Capture through the pinned Rust
`windows-capture` 2.0.1 adapter in a short-lived native executable. Windows' HWND
interop targets one window; no monitor/full-desktop capture or PrintWindow/BitBlt
fallback is exposed. The helper receives only a typed local observation request,
not model credentials, provider settings, scripts or a user environment. It exits
after one frame. The host owns a five-second deadline and kills/reaps it on timeout
or Stop. Other platforms report unavailable until qualified native adapters exist.

This uses the public [Windows capture API](https://learn.microsoft.com/en-us/windows/apps/develop/media-authoring-processing/screen-capture)
and [HWND interop](https://learn.microsoft.com/en-us/windows/win32/api/windows.graphics.capture.interop/nf-windows-graphics-capture-interop-igraphicscaptureiteminterop-createforwindow),
via the [adapter's documented handler](https://github.com/NiiightmareXD/windows-capture).
It does not depend on Codex's private runtime. OS capture borders stay enabled.

## Frozen scope and limits

Settings → Computer use lists at most 64 visible titled windows locally. Select
one returned window identity (HWND, PID, process creation time), then capture it.
Recheck identity/visibility before and after capture; missing, replaced, minimized
or unsupported targets refuse. Do not switch to another window or foreground it.
Images are JPEG, at most 512 KiB and 1024 pixels on the longest edge, with original
dimensions, DPI and explicit image-coordinate metadata. Refuse original surfaces
over four megapixels or 4096 pixels per side. Screen pixels are untrusted data.
Accessibility text is optional and may be unavailable; no OCR/background recorder.

The local capture cache retains at most 64 images / 32 MiB; UUID references bind
session, bytes/digest and metadata. Preview/refresh does not contact a provider.
Captures survive restart; removal is explicit and cannot retract a shared image.
Ordinary exports retain references, not binary image data. No periodic polling or
resident helper is introduced.

The cache is local unencrypted evidence, retained even after chat deletion.
Remove images before deleting their chat; UUID/session ownership prevents another
chat from browsing the orphaned image. This initial global retention policy is
bounded rather than automatic expiry. No secure-erasure claim is made.

Sharing is an explicit one-run choice of capture and configured image-capable
model/profile, independent of the ordinary selected chat model. A read-only
snapshot tool exposes only that chosen capture; no automatic fresh capture,
other-window discovery or action is available to the model. Its image result
shares the existing primary task's model/tool/time budgets and approximate
4096-token image allowance. Host context/image guards refuse exhaustion with
retained local evidence; no silent model switch, limit increase or retry.

Completion requires a successful screenshot-tool receipt with its image reference;
an answer that skipped observation is rejected. This guard proves delivery, not
semantic accuracy. Output-limited progress remains saved. Ordinary Continue or
checkpoint recovery refuses an observation run; explicitly select evidence and
model again. Automatic memory/skill reflection is excluded from this narrow run.

## Transport and evidence

Extend tool results with typed image references while preserving text-only
plugins/history. Chat Completions tool messages remain literal text. Project
their image evidence into a following host-labelled user image message using
`image_url`, as permitted by the [public message contract](https://developers.openai.com/api/reference/resources/chat/subresources/completions/methods/create).
This is an adapter projection of untrusted tool data, never fresh user authority.
The endpoint must support that format; local wire fixtures and a bounded configured
live observation qualify it independently of the existing image-input checkbox.
Reported usage and actual model/profile stay attributed to the primary run.

## Next boundaries and qualification

14.2 adds the separate input boundary below; 14.3 adds uncertain
action recovery. Observation alone grants no desktop input authority. No filesystem grant or
Full access mode implies permission to capture arbitrary windows. Model ability,
provider projection, target availability and native capture success are separate
facts. Test unsupported vision, missing/oversized captures, stale target identity,
Stop/deadline, backward-readable receipts and restart, plus one live synthetic
window observation. Record hardware/model/platform gaps honestly in acceptance.

## Scoped desktop input (14.2)

Explicit consent in Computer use creates a chat/window-bound 15-minute grant.
It is held in memory, never inherited from file permissions or Full access, and
must be enabled again after restart. Replacing or revoking a grant invalidates
pending dispatch; revocation also cancels that chat's active run. The host refuses
input to its own process, preventing UI-based self-granting. The active target
and Revoke desktop access stay visible above the composer; Stop remains available.

One narrow run advertises `desktop_control` instead of folder/browser/MCP tools.
Its closed vocabulary is observe, click, doubleClick, type, scroll, key and drag.
Text is plain, at most 512 UTF-8 bytes; scroll is at most ±1200; keys come from a
fixed navigation/editing list with no OS/task-switching shortcuts. No script,
clipboard or model-selected process/window API is accepted. Every pointer action,
submission/deletion key and explicitly consequential action needs fresh review.
An independent opt-in covers ordinary typing, scroll and navigation only.
Typing itself can trigger application/network effects; use trusted local forms.

First observe; copy the latest capture UUID for each input; observe after every
input. A capture is single-use for input and expires after 60 seconds. The broker
rechecks HWND/PID/process start, title, physical DWM bounds, DPI and focused child
before readiness and dispatch. Only client-area points visibly belonging to the
selected window are accepted; checked drag paths cannot cross another window.
Virtual-desktop coordinates include negative origins. Capture/frame geometry must
match; uncertain mapping refuses rather than guessing. Ordinary covered input
also rejects user input since capture. Manual review may foreground the selected
window from Dolores; an unrelated foreground application causes refusal.

The short-lived helper first reports readiness, then waits for a host commit.
The host rechecks cancellation, revocation and expiry before acknowledgement.
Native checks repeat before [SendInput](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput).
Locked/secure desktops and held keys/buttons refuse. Failure after acknowledgement
is conservatively uncertain; it never triggers automatic replay. Input insertion
does not establish application success; completion requires a subsequent image
receipt. This is best-effort window scoping with foreground/race checks, **not an
OS sandbox** or exactly-once guarantee; Windows input is a global queue and UIPI
can prevent delivery. Elevated/system applications remain unqualified.

Fresh images use a run-owned evidence resolver, capped at 16 receipts / 8 MiB,
sharing the existing task/model/context/elapsed limits. Identical pixel bytes may
have different capture identities; each reference remains resolvable separately.
Literal input intentions and returned receipts remain in durable run evidence.
Generic Continue/checkpoint recovery cannot silently re-enable desktop access.
No common task defaults are raised; a longer workflow needs an explicit scoped
task budget. Platform/DPI and semantic model reliability gaps belong in acceptance.

## Interruption and reconciliation (14.3)

A refused, denied or failed desktop action pauses immediately, including the
remaining actions in the same model response. Stop and the five-second broker
deadline kill and reap the owned helper. Anything after the host's dispatch
acknowledgement is conservatively uncertain; no retry or input replay follows.
An input receipt remains uncertain until a later screenshot is returned in that
run. Pixels are evidence to inspect, not an exactly-once application guarantee.

Computer use shows bounded saved receipts and uncertain effects. Recovery uses
the original goal and a new segment in its run lineage, current model/limits,
and explicit inspection. It requires a locally captured image made after the
run stopped, for the same HWND/PID/process-start/title target, plus a current
grant. Restart drops grants; an interrupted run without a finish marker needs
an image captured in the new host process. Saved in-run pictures and old approvals
cannot qualify. Older checkpoints lacking target proof remain inspectable but
need a fresh scoped task. An unrelated chat draft is preserved.

Three near-identical observations without intervening input pause for inspection;
a 32×32 quantized signature tolerates small caret changes (at most four cells).
This bounds futile refresh loops; it cannot infer slow application redraw or
whether a remote effect succeeded. Capture and input check the current Windows
session is active and explicitly unlocked via
[WTS session information](https://learn.microsoft.com/en-us/windows/win32/api/wtsapi32/ns-wtsapi32-wtsinfoex_level1_w),
as well as the input desktop. Unknown state refuses. Only availability flags are
used; account names and other session metadata are never retained. Windows 7 is
outside this backend's supported envelope.
