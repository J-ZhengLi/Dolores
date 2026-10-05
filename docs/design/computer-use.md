# Computer use — selected-window observation (14.1)

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

14.2 adds target-scoped input grants and dispatch checks; 14.3 adds uncertain
action recovery. 14.1 grants no desktop input authority. No filesystem grant or
Full access mode implies permission to capture arbitrary windows. Model ability,
provider projection, target availability and native capture success are separate
facts. Test unsupported vision, missing/oversized captures, stale target identity,
Stop/deadline, backward-readable receipts and restart, plus one live synthetic
window observation. Record hardware/model/platform gaps honestly in acceptance.
