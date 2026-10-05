# Windows computer-use qualification

Qualification date: 2026-10-05. Bricks 14.2–14.4 deliver scoped input, explicit
interruption recovery and reproducible qualification tooling. **The complete
real-model acceptance gate is still open.** The current fixed corpus has a passing
visual inspection and two failing input workflows. Host refusal/recovery passes
do not turn those model failures into successful tasks.

## Frozen workflow corpus

[Corpus version 1](../../scripts/desktop-corpus-v1.json) was fixed before these
attempts. Each case uses an isolated data directory and a disposable local window.
No user project, provider profile, selected model or history is changed. The test
operator permits only the stated synthetic text and form controls; a rejected
coordinate counts as a failed workflow, even when the host pauses correctly.

| Case / model | Required outcome | Observed outcome | Task time | Model / tool calls | Fresh observations / reviews | Saved image bytes |
| --- | --- | --- | --- | --- | --- | --- |
| Native draft / Qwen3.5-2B | Exact text, no save, observe after input | Failed: input attempted before the required observation; refused with no input | 2.67 s | 1 / 1 | 0 / 0 | 46,646 |
| Save once / DeepSeek V4.1 Flash | Exact note, one save, fresh Saved status | Failed: exact note retained; proposed click was outside Save and was denied; zero saves | 12.70 s | 4 / 4 | 2 / 2 | 146,862 |
| Generated view / DeepSeek V4.1 Flash | PulseBoard, Ready, Completed 42, In progress 7, Blocked 3; no input | Passed all visible facts with fresh image evidence | 4.07 s | 2 / 1 | 1 / 0 | 38,511 |

Tool counts include blocked/denied proposals. Task time covers the agent run;
total preparation/run time was 12.25, 22.39 and 7.30 seconds respectively. Image
bytes measure retained capture files, not HTTP transport bytes or image tokens.
Foreground token usage was unavailable for these runs and is recorded as unknown.

The generated-view case also made **one generation request**, capped at 256 output
tokens and 30 seconds, before its two inspection calls. Its generation token usage
was not retained in this qualification run. It generated a bounded declarative
JSON dashboard rendered by a trusted fixture, not arbitrary executable code or a
full generated website. Both stages keep the original criteria; malformed or
unexpected view specifications are refused before launching the renderer.

Task profiles: output 1,024 tokens, request timeout 30 seconds, total run 90
seconds, one segment. Model/tool allowances were 6/6 for draft, 12/12 for save,
4/4 for visual inspection. No ordinary defaults were increased, no automatic
fallback model was used, and no failed criterion was relaxed. These are individual
bounded attempts, not a reliability percentage. The separate successful 14.2
DeepSeek save-once probe remains recorded in [acceptance](../ACCEPTANCE.md#brick-142--scoped-windows-desktop-input);
the current failure shows that success is not yet consistent.

## Failure and recovery evidence

The normal native release and deterministic providers exercised:

| Pressure | Observable recovery |
| --- | --- |
| Native input takes effect before its receipt is lost | Immediate pause; text retained; uncertain effect persisted; no later queued action or provider call |
| Cold restart | Previous authority dropped; old screenshot/absent inspection refused; new consent and fresh capture allow explicit recovery with the original goal |
| Explicit reconciliation | Previously entered text remains; resumed segment observes instead of duplicating input; lineage retained |
| Stop after effect, before receipt | Applied text retained and marked uncertain; owned helper stopped |
| Five-second broker deadline | Owned stalled helper reaped; usable text retained; no blind retry |
| Three near-identical observations | Bounded pause with an inspection next step |
| Injected locked target | Refusal and pause without further provider requests; physical lock/unlock remains unqualified |
| Two-tool allowance exhausted after typing | Three model calls, two tool executions; retained text and uncertain-effect checkpoint; no save |
| Output length limit before input | Partial answer retained; explicit output-limit pause; existing text unaffected |
| Window moved during review / grant revoked while queued | 14.2 native tests refused stale dispatch; Save count unchanged |

Recovery/crash checks use a test-only wrapper around the actual input helper.
The wrapper is never shipped. Budget checks use a deterministic local provider,
not an assertion of real-model failure handling. Changing limits did not form
part of recovery. Saved progress is available through Settings → Computer use;
the user inspects a fresh screenshot and explicitly resumes the saved original
goal with current access. Prior inputs and approvals are never replayed.

Rust checks: 183 passed across core/provider/bridge/helper suites; Clippy clean.
Flutter: 218 passed; analysis clean. Desktop widgets exercise recovery, retained
draft and default-off access at 420×480 in light/dark. The normal release was built,
launched visibly and inspected in both themes, including saved receipts and
recovery controls. Native resizing did not establish an additional compact-layout
pass; compact evidence is the widget coverage. Original 37-table provider/history
digests remained unchanged.

Two focused corpus checks also passed: numbers attached to the wrong card labels
do not satisfy visual criteria, and unexpected/oversized declarative views fail
before launching a window. These validate the evaluator/fixture, not model quality.

## Resources and startup

Measurements are from the available Windows host, with 24 logical processors and
about 32 GiB RAM. This is **not** representative lower-end qualification. No device
identifier, account information, private transcript or screenshot is committed.

One isolated normal release launch exposed a window in **1,513 ms**. After three
seconds, working memory was **238,239,744 bytes (227.2 MiB)** and private committed
memory **251,731,968 bytes (240.1 MiB)**. Window presence is not the first usable
frame; this is one launch, not a startup distribution. The existing normal user
instance separately used about 269 MiB working memory. Neither measurement
includes an active desktop capture in Flutter.

Read-only 25 ms sampling during the native budget fixture observed:

| Sampled component | Peak working memory | Peak private committed memory |
| --- | --- | --- |
| Python C ABI test host | 34.4 MiB | 17.3 MiB |
| Largest individual owned capture/input helper | 55.5 MiB | 53.3 MiB |
| Concurrent test host plus owned helper tree | 89.7 MiB | 70.4 MiB |

Five helper instances were observed and zero remained after completion. Each live
corpus run likewise ended with zero owned helpers. The Python host is not the
Flutter desktop process; its memory cannot establish an end-to-end desktop peak.
Sampling can miss short-lived processes, so these are observed lower bounds.
Private committed memory is not resident memory. CPU, battery use, representative
4 GiB/two-core hardware and sustained end-to-end active capture remain unmeasured.

## Portability review

| Platform | Backend / authority boundary | Qualification status |
| --- | --- | --- |
| Windows | Selected-window Graphics Capture and checked SendInput; separate runtime grant; session availability and identity/focus/geometry checked | Native synthetic input/recovery verified; arbitrary applications, secure/elevated targets, physical lock recovery and mixed-monitor DPI unqualified |
| macOS | Candidate: ScreenCaptureKit for capture, Accessibility APIs for target inspection, explicit OS permissions and checked input dispatch | Design review only; backend remains disabled until native implementation and failure tests |
| Linux / Wayland | Candidate: ScreenCast plus RemoteDesktop portals, user-selected session/devices, PipeWire and EIS; reject scope broader than the selected target | Design review only; compositor/backend differences need native qualification; no silent global X11 input fallback |

Windows input uses the OS global input queue, with host checks rather than an OS
sandbox. UIPI can block elevated input. Current-session checks require active and
unlocked state; the Windows 7 reversed-flag behavior is outside the supported
envelope. See [Microsoft's session flags](https://learn.microsoft.com/en-us/windows/win32/api/wtsapi32/ns-wtsapi32-wtsinfoex_level1_w)
and the [input contract](../design/computer-use.md).

macOS screen sharing and Accessibility trust are separate concerns. Native
permission-denial/revocation and target identity need dedicated tests before
enabling this candidate backend. Primary references: [ScreenCaptureKit](https://developer.apple.com/documentation/screencapturekit)
and [Accessibility trust](https://developer.apple.com/documentation/applicationservices/1459186-axisprocesstrustedwithoptions).

Wayland's RemoteDesktop portal selects devices and starts a user-mediated session;
ScreenCast can supply PipeWire streams, and EIS is the recommended input route.
Use nonpersistent sessions for Dolores's current grant contract and verify the
actual returned capture/input scope per compositor. This is a proposed mapping,
not an implemented backend. See the [portal specification](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.RemoteDesktop.html).

## Reproduction

### 2026-10-06 follow-up

The frozen corpus and its 1024-output-token, 30-second request and 90-second task
allowances were retained. Qwen's native draft failed on extra fields in an
observation request, with no input or save. After making the exact observe
arguments explicit in the tool description, a fresh Qwen run instead failed at
the initial screenshot-reading check, again before input. This clarification is
not a demonstrated reliability fix. Rust checks preserve rejection of observe
requests containing a capture, false default or null input field.

A DeepSeek native save run obtained two fresh observations and correctly entered
the requested text, but paused at the output limit before Save: one reviewed input
and zero saves. No input was replayed, no criteria were relaxed and no automatic
model switch or budget increase occurred. General model adherence, coordinate
grounding and completion remain unqualified.

Build the normal desktop bundle first. Use a fresh absolute directory under the
repository's ignored `output/` for each command. Test scripts refuse existing
directories. The local form must be in the foreground before input; the bounded
focus wait refuses to take over an unrelated application.

```text
python scripts/test-desktop-pressure.py --directory <absolute-output-directory>
python scripts/test-desktop-corpus.py
python scripts/qualify-desktop.py --case native-draft --directory <absolute-output-directory>
python scripts/qualify-desktop.py --case native-save-once --directory <absolute-output-directory>
python scripts/qualify-desktop.py --case generated-app-visual --directory <absolute-output-directory>
python scripts/desktop.py launch --data-directory <absolute-profile-directory> --pid-file <absolute-output-record>
python scripts/measure-desktop.py --pid-file <same-record> --seconds 15
```

Live cases require `DOLORES_TEST_BASE_URL` and `DOLORES_TEST_API_KEY` supplied by
the operator; the public runner never reads personal provider configuration.
Failed criteria produce a nonzero exit and retain the local result. The startup
probe leaves its owned normal release visible for inspection and records its PID
only in ignored output. Close that instance after inspection.

Keep local test databases/captures/transcripts private. Public results contain
only synthetic outcome counters and resource measures. Follow-up acceptance
should improve observation adherence and visual coordinate grounding, then repeat
the unchanged corpus before expanding to broader apps or asserting dependable
computer use. Other-platform and lower-end acceptance stays open.
