# Dolores acceptance — 2026-10-08

## Portable glass and motion refinement — October 25

The bottom dock and navigation selections share a Flutter-painted glass material:
translucent tint, directional reflective rim, inset shade and localized shadow.
The dock uses one clipped active backdrop blur; its selected lens reuses that
backdrop without a second active blur. Labels/icons remain crisp. Settings uses
the same lens material. No Apple-native plugin or new dependency was added.

Selection travel uses a damped spring and carries current position and velocity
into interrupted/rapid changes. A bounded stretch follows travel and returns to
the resting shape. Press/cancel feedback compresses only the visual child; hit
areas remain stationary. Dock reveal/dismissal uses eased travel with a shorter
fade, preserving edge/tap/keyboard reveal, auto-hide, focus and draft ownership.
High contrast disables blur; reduced motion also removes travel/stretch/scaling.

Full Flutter tests pass: 611 passed, 19 opt-in cases skipped. Analyzer reports no
issues. Four added cases cover rapid reversal, resize during travel, enabling
reduced motion mid-flight, cancelled mouse presses, contrast fallback and busy
content in both themes. Existing tests cover draft retention, menus, Settings,
hidden-dock input/focus, keyboard reveal and compact layouts. The layout probe
still records zero navigation-parent layouts over 20 animation frames.

Saved synthetic renders in `output/glass-refinement/` were inspected for Light,
Dark, compact drawer, Settings and dock glass over busy content, including a
mid-travel lens. These establish fixture appearance and interactions, not native
GPU frame timing, physical assistive-technology behavior or Apple parity.

The normal Windows x64 helper build passes. Its `main` build identity and compiled
artifact freshness were checked after the final visual edits. The user profile
was not launched or modified for this verification. Documentation and tracked-file
publication checks pass.

## Shared UI guidance scope — October 25

Git history shows feature workflow paragraphs accumulating in UI.md alongside
shared presentation guidance: history search, notifications, prompt branches and
MCP connection setup. The old introduction called it a contract and linked to
inline feature contracts; the agent entry point did not distinguish design
principles from feature documentation. Link checking accepted that growth.

UI.md now contains reusable design/review guidance (144 lines, previously 299),
including portable glass and motion. Existing feature interaction requirements
were retained in `docs/design/desktop-interactions.md`, reached through the docs
index. Agent/contributor pointers name the separate owners for principles,
workflows, rationale, plans and verification; exact palette values remain in the
theme. The documentation check enforces a 180-line budget and routes feature
references and dated section headings away from UI.md. This is a structural
guardrail; reviewing whether a rule is reusable remains necessary.

Three scope regression tests and five CI-helper tests pass. The old guide
produces eight scope findings under the new check; the current guide passes.
All 125 documents and 547 local links pass. No UI behavior changed in this
documentation brick.

## Terminal startup and dock cleanup — October 23

The reported Windows PowerShell startup error was reproduced using the actual
Windows PowerShell executable. A normal executable path worked with both the
full and production-filtered environment; the canonical verbatim executable
path failed with the exact ServicePointManager initialization error. A new real
PTY regression also failed before the fix. The launcher now converts canonical
disk/UNC paths to ordinary Windows paths only when starting the shell and setting
its working folder. Ownership metadata retains canonical paths; the environment
allowlist and preferred-shell selection are unchanged.

All five terminal Rust tests passed after repair. They exercised Windows
PowerShell specifically even where PowerShell 7 is installed, Unicode working
folders and output, .NET initialization, PTY resizing, stopped-session input
refusal, retained state, closing/replacement and existing descendant cleanup.
Path tests cover UNC, Unicode, ordinary paths and preservation of other device
paths. UNC conversion is unit-tested; no network-share shell was launched.
The native shell tests require no provider credentials and change no saved
provider configuration or conversation history.

Library was removed from the navigation dock. Settings remains its single
configuration entry and retains Skills; related skill-recovery text points there.
Light/Dark wide and compact dock renders were inspected under
`output/dock-terminal-renders/`. Focused Flutter checks passed, including 20
workspace, terminal and learning tests and the wider title/dock/timeline render
cases. Flutter analysis, document/publication checks and the final normal Windows
x64 build passed. Native pointer/IME interaction was not repeated;
the real PTY regression establishes shell startup and I/O separately from UI
fixture coverage.

## Inline agent activity — October 23

Public progress and tool receipts now share the conversation timeline. New runs
save the model-call number associated with each receipt, preserving interleaving
after reopening. Compact icon rows expand to the existing screenshots, command
output, diffs and specialized receipts. The final answer follows the activity;
thinking remains separately collapsed. Live runs retain later progress/receipts
beyond the former three-text/four-tool preview caps, within the existing host
execution/output limits. Interrupted unsaved work remains labeled and inspectable
in the recovery area; it is cleared for a new independent request.

Legacy chats have no exact ordering evidence: available progress and receipts
appear before the answer under Earlier activity rather than guessed interleaving.
Malformed/incomplete ordering metadata takes the same lossless fallback.

The 38 focused Flutter tests passed, exercising chronological layout, five tool
receipts across four progress updates, six live tool rounds, failed receipt
expansion/recovery guidance, legacy/malformed metadata, Stop/new-request recovery,
literal command/diff details and manual-scroll following. Saved Light/Dark renders
at 420 and 1000 pixels were inspected under `output/inline-activity-renders/`.
Core tests passed (105); bridge and storage library tests passed (215 and 89,
with one existing ignored bridge test), including metadata round trips and legacy
history. The full Flutter suite passed 529 tests with 10 opt-in native cases
skipped. Flutter analysis and the normal Windows x64 build passed. Inherited
failed-command receipts remain visible under Earlier tool results, including
when a later run contains no tool calls or incorrectly claims success.
No live provider or native interaction is needed to establish the new rendering;
these fixtures do not establish real-model task reliability.

Follow-up: a provider may reuse a call ID in a later repair run. A focused
regression failed because the new successful receipt hid an inherited failure
with that ID. Deduplication now compares full receipts, retaining both distinct
results; activity groups also keep independent expansion identities. Nine focused
repair, continuation and timeline tests passed, including opening the retained
failure and preserving the current successful receipt. The final follow-up
Windows x64 build and Flutter analysis passed.

## Persistent Settings shortcut and exported-session follow-up — October 23

The user's later exported conversation disproved a broad reliability reading of
the successful isolated Light-mode test below. Its latest active attempt did
receive named accessibility targets and successfully opened Chat actions, then
closed that menu with Escape. It never clicked the repeatedly observed navigation
reveal control, described that control as a collapse toggle, and tried the wordmark
and sidebar instead. Light mode was not reached. The final exchange explicitly
asked the agent to wait and correctly contained no computer operations. The
preceding older attempt stopped on malformed input; its blocked operation's
arguments were not retained in this export, so the precise malformed field cannot
be determined from it. No private transcript or other window titles are copied
into repository evidence. The earlier isolated success remains one passing journey,
not evidence that this longer conversation succeeded.

A Settings gear is now permanently available immediately left of Minimize in the
custom Windows title bar. It opens the existing General/Theme window using the
visible conversation's controller and retains the dock entry. An existing modal
or review prevents another settings window from stacking. Repeated activation
while opening/visible is ignored, native window controls remain usable, failed
opening retains content with Retry, and keyboard opening restores focus to the
gear after close. The hidden dock's semantic name now says navigation dock and
lists actual destinations, including Settings, instead of the ambiguous navigation
label. This improves discoverability without hard-coding theme commands into the
computer tool or changing its input/approval limits.

Three initial tests failed on the absent title gear and ambiguous dock semantics.
After implementation, five focused tests passed, including retained chat drafts,
420-pixel layouts, duplicate activation, existing-review protection, keyboard
activation/focus restoration and failed-opening recovery. The wider title-frame,
settings and UI-polish suite passed 24 tests before the two additional edge cases.
Saved renders were inspected in Light and Dark at wide and compact widths under
`output/title-settings-renders/`; the gear is aligned immediately before Minimize.
The final four-file UI suite passed all 26 tests; analysis of the three changed
widgets and new tests reported no issues. The normal Windows x64 build passed.
The direct native gear click opened General/Theme in the normal app, independently
of the dock. This UI repair does not establish
reliable navigation for every model or retroactively complete the user's session.

An isolated live run with `deepseek-v4.1-flash-1017` and the same bounded natural
request passed without navigation hints: 13 model calls, 12 computer operations,
five target-ID clicks, no coordinate inputs or approval requests. It used the
clearer navigation-dock reveal and dock Settings target, switched to Light,
closed Settings and returned a final dock screenshot; the saved turn finished
normally. The final model-fed image was visually inspected and the isolated
database persisted Light. The source user profile remained unchanged. Public
receipt/local evidence: `output/title-settings-live-20261023-01/`. The direct
gear check followed only after that run finished; it did not contribute to the
model's success. The owned preview was stopped using its verified process record.

Remaining gap: the native Windows accessibility snapshot did not expose the
custom title-bar controls, including this gear, although Flutter widget semantics
and keyboard tests pass. The gear's native pointer behavior is qualified; direct
model selection of that title-bar gear by accessibility ID is not. The dock's
Settings target remains available. The user's full long conversation was read
for diagnosis and was neither replayed nor modified; fresh-session model success
does not establish recovery of that exact history.

## General accessibility targets — October 23

Computer observations now add selected-window UI Automation controls with names,
roles, bounds and capture-bound IDs. Click, double-click and scroll can choose
an observed target instead of estimating pixels. Before input, the broker checks
the current control at that point against the observed name, role and bounds,
plus runtime identity when available, then rechecks native window/focus/geometry. Unknown IDs, mixed
coordinates/IDs and old/consumed captures refuse; recovery requires observation.
There are no theme-specific commands or application selectors.

Discovery is bounded to 512 nodes, 16 levels, 64 targets and 8 KiB. A separate
1.2-second helper deadline retains screenshot fallback when UI Automation stalls
or is unavailable. Password subtrees and disabled/offscreen controls are excluded;
values/document text are not requested. These checks improve targeting, not
model planning or exactly-once guarantees. Existing task/image/protocol budgets
and automatic ordinary-chat authorization remain unchanged.

The new bridge test first failed because target-based input was unsupported.
It now passes for a public form button and refuses invented, conflicting and
old-capture references. Shared validation also exercises scroll and unavailable
accessibility with coordinate fallback. The full suite passed: 105 core unit
tests, seven core integration tests, four helper tests and 215 bridge tests
(one existing ignored bridge test). Clippy passed for all targets of these crates
with warnings denied, and formatting passed.

`scripts/test-computer-targets.py` passed against an owned, separate-process
Win32 form: a target click fired Save; moving the button inside the unchanged
window refused the stale target without a second click; a fresh observation
allowed recovery. Disabled and password controls were absent. This check used
no model/provider requests. Public receipt: `output/computer-targets-20261023-01/`.
The fixture qualifies general native targeting, not real-model task reliability.

The first live attempt discovered only frame chrome and ended after five model
calls when a stale/mismatched target was refused. Dark mode remained, with zero
approval prompts and the original profile unchanged. This exposed a harness
discovery gap: Flutter's native child publishes its controls through MSAA, and
its compatibility elements return empty runtime IDs. Discovery now queries
native child clients through the standard bridge, filters unsupported frame
chrome and explicitly marks property-only identity fallback. Such targets require
the full name/role/bounds and current hit-test match, but cannot distinguish an
identical replacement control. The first live result remains a failed journey:
`output/computer-target-live-20261023-01/`.

After this correction, the native public-form fixture passed again with only its
two usable client targets, including changed-control refusal and fresh-observation
recovery: `output/computer-targets-20261023-02/`. Shared validation covers empty
runtime IDs with and without the explicit property identity marker. Clippy passed
again for all three crates.

The final normal Windows x64 build passed, including the accessibility worker's
independent 1.2-second watchdog so a blocked provider cannot outlive a killed
parent indefinitely. The public form fixture passed against the bundled release
helper: `output/computer-targets-release-20261023-03/`. The ordinary-chat
loopback fixture also passed capture, typing and coordinate fallback with zero
prompts through the final rebuilt bridge (`output/computer-target-fallback-20261023-02/`).

The corrected live journey passed with `deepseek-v4.1-flash-1017`: the original
natural-language request, no navigation hint, 17 model calls / 16 computer
operations, all HTTP 200 and zero approval requests. Six clicks used observed
control IDs, one input used Escape, and no coordinate input was needed. The
model first opened and closed Chat actions, then revealed navigation, opened
Settings, chose Light, closed Settings, revealed the dock and inspected its
final fresh image. The saved response identified all seven dock labels. The
isolated database persisted `light`; visual inspection of the final model-fed
screenshot confirmed Light mode with Settings closed and the dock visible.
The run finished normally and saved its turn. Receipts and local image evidence:
`output/computer-target-live-20261023-02/`. The source user profile was unchanged,
and the owned preview was stopped using its verified process record.

This is one successful real-model journey plus a separate native form fixture,
not broad application/model reliability qualification. Custom canvases, apps
without useful accessibility, identical property-only replacements, elevated
applications and pathological blocking providers remain acceptance gaps.

## Automatic ordinary-chat computer use — October 23

Ordinary foreground `computer` operations now proceed without per-action
approval, including clicking/typing in Dolores itself. The dedicated authorization
path applies in Review, Auto edit and Full access independently of folder tools;
scheduled/delegated/planning-only exclusions are unchanged. The older manual
`desktop_control` grant/review workflow is unchanged. Stop/policy revision is
rechecked before authorization; the native identity/focus/geometry handshake,
single-use screenshots, exclusive run ownership and post-input image evidence
remain required. Recovery copy now explains automatic computer continuation.

The new authorization test first failed on a click's `toolApproval` event, then
passed for all eight computer operations. It checks no approval event/slot and
refusal after cancellation. The native fixture now treats any ordinary computer
approval as failure and exercises capture, typing and clicking with zero prompts.

The live qualification runner's disposable response allowance is now 4,096
tokens after the previous 1,200-token reasoning-only exhaustion. This changes
only that explicit test's request; product defaults and execution budgets remain
unchanged. Earlier live failures above remain failures.

Verification: 104 core and 214 bridge tests passed (one existing ignored bridge
test); Clippy passed for all targets of both crates with warnings denied. Ten
focused Flutter computer/history tests passed, and the changed page passed
analysis. The normal Windows x64 build passed. Both same-process and separate
native-window fixtures passed capture, typing and clicking with zero approval
events and fresh post-input screenshots. Receipts are under
`output/computer-auto-self-20261023-01/` and
`output/computer-auto-other-20261023-01/`.

The real-model automatic run used `deepseek-v4.1-flash-1017` with a 4,096-token
response allowance and the original natural-language request, without the dock
hint. It made 17 model calls and 16 automatic computer operations, with zero
`approvalRequested` events. Captures and native clicks worked, but the model
mis-targeted dock controls and opened Scheduled. Light mode was not reached.
The run ended with the existing 2 MiB reasoning-continuation protocol allowance;
no product limit was raised, no retry occurred, and journal/capture evidence
remained available. This qualifies the removal of per-action prompts, not
reliable model navigation. Receipts and isolated evidence are under
`output/computer-auto-live-20261023-01/`. The normal user profile remained
unchanged and the owned preview was closed after the check.

## Conversation scroll rebound — October 23

A small upward wheel movement cancelled following, but the scroll-position
listener enabled it again whenever the remaining distance was less than 80 px.
The next reply/status/render update then jumped back to the bottom. The listener
now enables following only at the actual bottom (within 1 px), while upward user
scrolls cancel it immediately. Larger programmatic moves away still cancel
following, and latest-page restoration still reaches the final rendered reply.

The regression failed before the fix: a 24 px upward wheel movement was undone
by the next update. Both idle and busy cases now retain their offsets through
repeated rebuilds and streamed paragraph growth, then resume following after a
downward wheel returns to the bottom. All 20 focused history/reply-rendering tests
passed, including late-render following and compact/wide latest-page controls.
The changed Dart files passed analysis and formatting checks.
The normal Windows x64 build passed. The rebuilt isolated app accepted a native
upward wheel movement in the retained public transcript. Repeated streaming
updates are covered by the deterministic rendered tests, not a new live-model
scrolling run. The normal user profile was not relaunched or modified.

## Live computer-use qualification — October 23 (failed journey)

The normal Windows app was exercised with the configured
`deepseek-v4.1-flash-1017` real model in an isolated profile. Only its saved
connection was read; no source chat/history was copied. A bounded local relay
kept the real key in memory and retained status/count receipts, not request or
response bodies. The original profile's logical database digest remained
unchanged after both live attempts.

The natural-language request was to switch Dolores from Dark to Light, close
Settings, reveal the bottom navigation dock and inspect it. The first attempt
made five model calls: window discovery, screenshots and a reviewed native click
worked, but the proposed overflow-menu click missed. A subsequent title-area
click was declined, and the task paused without later dispatch.

A fresh attempt with an explicit dock-navigation hint made seven model calls.
It revealed the dock and correctly named Home, Scheduled, Files, Source Control,
Terminal, Library and Settings. Its proposed Settings coordinate opened Source
Control instead. After capturing that result, the model exhausted the bounded
1,200-token response allowance during reasoning and paused with retained progress.
Light mode was not reached. This is a failed end-to-end qualification, not proof
of reliable self-navigation; coordinate grounding and response-budget suitability
remain open. No automatic retries or budget changes were made to the product.

Evidence: `output/computer-live-20261023-02/relay-result.json` and
`output/computer-live-20261023-03/relay-result.json`, plus isolated run events and
captures in those directories. The reusable opt-in runner is
`scripts/qualify-computer-live.py`; it launches the normal app, leaves all input
reviews to the human/qualification operator, and bounds requests, output and time.
The initial runner attempt held its own profile lock while launching; seeding now
runs in a separate process that exits before launch. No live model was contacted
in that failed setup attempt.

## Reviewed control of Dolores itself — October 23

The ordinary-chat `computer` tool no longer rejects input solely because the
target window belongs to the host process. Its tool and system guidance now
allow operating Dolores itself under the same exact human review used for other
applications. Full access does not automatically approve this input. The native
identity, focus, geometry, single-use capture, Stop and post-input screenshot
checks remain in force. The legacy manual-grant workflow keeps its original
self-target restriction; this change addresses direct ordinary-chat control.

The self-window regression first failed with the reported "cannot control its
own window" error. It now requires a self-target click to prepare and checks
that Full access still cannot automatically approve it. Forged image references,
stale window IDs and cancelled capture checks remain covered by the same tests.
The native fixture has a `--same-process` option that hosts its disposable window
in the process loading the release bridge, exercising the real self-PID path.

Verification: four focused bridge computer tests and the core computer-guidance
test passed; Clippy passed for all targets of both crates with warnings denied.
The normal Windows x64 build passed. Against that release, the same-process
native fixture passed observation, denied typing, reviewed typing and reviewed
clicking; both successful inputs were followed by fresh screenshot evidence.
The old bundle failed this fixture before the repair because no self-input review
could be reached. Public result: `output/computer-self-green-20261023/public-result.json`.
The fixture's state publishing tolerates a brief Windows reader lock so its
observation/recovery loop continues. Documentation and publication checks passed.

These checks use an isolated profile, disposable native form and local scripted
provider. They establish real same-process input and screenshot delivery. A live
model navigating the actual Dolores settings and completing a theme change is
not yet qualified; the existing theme control permits saves during an active
run. No saved provider configuration or private history was used or changed.

## Computer availability without a working folder — October 23

The October 22 integration incorrectly required a workspace root both when
adding `computer` and when enabling its extension registration. Ordinary side
chats therefore received only scheduling and question tools, matching the
reported refusal. Removing those conditions makes direct
computer use available in foreground side chats as well as project/temporary
chats. Scheduled runs and planning-only runs retain their existing exclusions;
file, command and delegation tools still require their separate working context.

The native regression now defaults to `--kind side` and also supports `temporary`
and `project`. Before the fix, its first provider request failed with the exact
catalog `[schedule_task, manage_scheduled_task, request_user_input]`, proving that
the tool was absent rather than ignored by a model. The earlier project-only
fixture did not cover this path. The side fixture also asserts that enabling
computer use does not expose file, shell or delegation tools.

The regular bridge backpressure/cancellation test now asserts the same side-chat
catalog. Its isolated host supplies the normal private data directory for capture
storage, independently of the conversation's absent working folder. That test
also caught the second registration restriction before it was removed.

Verification: all 213 bridge tests passed (one existing ignored test), Clippy
passed for all bridge targets with warnings denied, and formatting/document-link
checks passed. The normal Windows x64 build completed through `desktop.py build`.
The rebuilt release bridge passed all three side-chat native cases: automatic
capture with no review, denied input with no subsequent dispatch, and reviewed
typing followed by a fresh screenshot. No file/shell/delegation tool appeared.
Public receipts: `output/computer-side-green-20261023/public-result.json`.
The same three native cases also passed for project and temporary chats;
receipts are in the corresponding `computer-project-green-20261023` and
`computer-temporary-green-20261023` output directories.

These native checks use a disposable form, isolated profile and local scripted
provider. They verify the missing-tool repair and image delivery, not a live
model's natural-language tool selection. No saved provider configuration or
private conversation was changed. The first build could not replace files held
by the user's app; after it closed, the final build succeeded.

## Ordinary-chat computer use — October 22

Windows working chats now offer a native `computer` tool. The agent can discover
visible windows and send the intended window's screenshot to the current
image-enabled model without a Settings handoff. It may inspect Dolores itself;
input to its own process remains refused. Other input retains exact in-chat
review, fresh single-use captures and the existing native identity/focus checks.
Only one ordinary chat owns computer use at a time. Denial, Stop and tool failure
pause before later actions; input needs a subsequent screenshot of the same
window. The manual chooser and saved-screenshot workflow remain available.

HTTP 408/5xx failures now identify a provider failure rather than asserting that
the model cannot use tools. Recovery retains work and calls for an explicit retry.
The registered-tool capacity grows from 23 to 24 to accommodate the direct tool
alongside the manual chooser; execution budgets are unchanged.

Verification exercised:

- Rust: 104 core, 213 bridge and 48 provider tests passed; one existing bridge
  test remains ignored. Tests cover self-observation, forged image references,
  stale window/capture IDs, disabled image input, cancellation, exclusive use,
  denial stopping queued actions and HTTP 503 without silent retries. The new
  guidance and outage-recovery assertions failed before their respective fixes.
- Clippy passed for all targets in the three affected crates with warnings denied.
  Thirteen focused Flutter tests passed and the two changed Dart files passed
  analysis. Compact Light/Dark review renders were inspected under
  `output/computer-use-renders-20261022/`; denying input preserves the draft.
- The normal Windows x64 build passed through `scripts/desktop.py build`.
  `scripts/test-computer-chat.py` exercised that release bridge with a disposable
  native form: automatic list/capture with zero reviews, denied input with no
  subsequent model call or text change, and one approved typing operation followed
  by a second screenshot. The captured image visibly contains the expected text;
  the form was never submitted. Receipts are under
  `output/computer-chat-20261022-02/public-result.json`.
- The existing manual handoff fixture also passed, including retained unrelated
  draft/attachment, shared task budgets, fresh image evidence and no submission.
  Receipts are under `output/computer-legacy-20261022-02/public-result.json`.
  Both native scripts now wait for fixture startup before inspecting it; the
  original startup races failed before adding that bounded wait.

Evidence boundaries: native journeys use an isolated profile and deterministic
loopback provider, never saved credentials or private transcripts. They establish
real window capture/input and image delivery, not real-model reliability or
Codex feature parity. A normal-app self-inspection journey with a live model,
cross-application targeting, elevated windows, multi-monitor/DPI combinations and
clean-machine packaging remain unqualified. This change does not diagnose the
separate storage/Close incident recorded below. The harness umbrella remains
paused at EH06.

## Desktop interaction and layout polish — October 22

Primary navigation now uses a floating bottom dock. It overlays the workspace
without reserving height, hides after inactivity/pointer departure, and reveals
from the bottom edge, its handle or Ctrl+Alt+D. Hover and keyboard navigation keep
it open; Escape dismisses it. Feature sidebars begin below the shared title bar,
which keeps the Dolores name and outlined sidebar toggle visible on every page.

Glass navigation suppresses competing Material hover/splash fills and uses a
paint translation. Workspace and settings content now fade/slide during page
changes, respecting reduced motion. External form labels use shared edges and
spacing. Long explanations use hover/focus/tap help; validation, errors and
important sharing consequences remain visible. Run details lead with per-call
metrics and retain breakdowns, settings and sources in expandable sections.

Focused verification exercises rapid navigation, retained chat/commit/settings
drafts, hidden-dock input/focus exclusion, pointer edge reveal, keyboard reveal
after focus leaves content, compact drawer dismissal and Light/Dark renders.
The hover regression failed before the fix with a nontransparent Material layer;
the navigation layout probe fell from 20 parent layouts over 20 animation frames
to zero. This measures layout invalidation, not native GPU frame time.
Long-help tests cover mouse, keyboard and tap without changing the setting.
Missing/zero usage counts and stale/failed saves retain truthful recovery states.

Full Flutter verification: 513 passed, 10 existing opt-in tests skipped; analyzer
reports no issues. All 13 focused frame/polish tests pass after the final title
alignment and toggle-size refinement. Documentation links and tracked-file
publication checks pass.

Saved widget renders under `output/ui-polish/` were inspected for wide/compact
Light/Dark forms, run details, floating/hidden dock and Source Control. They use
synthetic public data. Four native bridge journeys pass: local instructions/Memory,
chat safeguards, Stop/recovery and queue/steering. Their logs and receipts are in
`output/native-ui/d4eedf318a0c41d39b681c501e4ec2b0/`. These fixtures ran against the
shared working checkout, including concurrent backend changes. Native pointer
timing, screen-reader/IME input and real-model reliability remain outside this
fixture evidence.

The normal Windows x64 helper build completed after the final header edits. Its
`main` build identity and compiled Flutter artifact freshness were verified.
The bundle includes concurrent backend changes from the shared checkout; the
normal user profile was not relaunched or used for qualification.

Separate reported follow-up: the user's running app refused Close with
“Could not read or save local conversation data. Keep Dolores open and retry”.
The user explicitly authorized force-closing that instance to release the build
DLL. Its PID/path were checked before stopping it; no profile was edited. The
storage/Close failure remains undiagnosed and unfixed, deferred at the user's
request. Do not treat UI fixture passes as qualification of that incident.

## Frost & Graphite desktop redesign — October 22

The normal Flutter app now uses the shared cool Light / graphite Dark palette,
one labeled workspace sidebar, moving glass selection highlights, a Home composer
with real projects and recent chats, anchored composer dropdowns and centered
settings. Existing Files, Source Control, Terminal, Scheduled, chat/history and
settings editors remain reachable. Home shortcuts use AppHost conversation
ownership. Moving the composer from Home into an active chat preserves its editor
identity, focus, draft, selection and slash-command handling.

Gentle motion is a persisted application preference shared across conversations.
OS reduced motion takes precedence. Interrupted selection travel retargets from
the current position; reduced motion uses immediate, opaque selection feedback.
Failed preference saves retain the acknowledged value and offer an explicit retry.
Legacy preference JSON without the new field remains readable without migration.

Verification exercised:

- Full Flutter suite: 505 passed, 10 skipped; analyzer: no issues. An obsolete
  toolbar-position assertion was updated, and a real Home-to-chat `/stop` focus
  regression was fixed before the passing run.
- Four focused design tests passed again after the final menu refinement:
  interrupted highlight travel, reduced motion, failed-save/retry and propagation,
  Light/Dark centered settings, draft retention, Escape and compact menu bounds.
- SQLite preference tests: 2 passed; native bridge appearance/bootstrap test:
  1 passed. These cover old records, restart, stale revisions, failed writes and
  saved motion restoration.
- Normal Windows release build passed using `scripts/desktop.py build` with the
  installed Visual Studio 18 2026 generator.
- Four rendered Flutter journeys against that normal Rust bridge passed:
  local instructions/Memory, chat reentry/summary/model safeguards, Stop recovery
  and queue editing/steering. Evidence is retained under
  `output/native-ui/215e132d7a1c4d8b97a9c02d6661efce/`.
- Light, Dark, compact, settings and model-menu widget renders were inspected
  under `output/frost-design/`. They use synthetic data and system fonts.
- Rust formatting, documentation links and publication checks passed.

Scope of evidence: native-bridge journeys use isolated profiles and a public
loopback provider fixture; no normal profile, credentials or private transcripts
were used. These checks do not qualify physical Windows IME/screen-reader input,
GPU animation frame timing, clean-machine behavior or real-model reliability.
The separate EH06 acceptance gaps below remain open.

## EH07 edited-prompt branches — October 24

Edited-prompt branches add a schema-46 archive and scoped attachment ownership.
A host preview binds the saved user prompt, complete bounded original transcript,
tool metadata, workspace, ancestry and explicit context snapshots. Atomic creation
copies only completed exchanges preceding the edited prompt into live history,
then saves the edited draft and selected original prompt attachments/context.
The old answer and later messages remain in a separate local archive. Original
messages, task, queue, goal, draft and working files are not rewritten. A branch
has no copied run, active approval, goal, queue, skill selection, accepted plan
or thread override. Host guidance distinguishes historical receipts from current
files and is counted before ordinary context/token admission.

Stable IDs match the entire saved request. Repeated creation opens the retained
branch without overwriting its newer draft, including after parent/folder deletion.
Shared-folder creation validates the current folder; a missing folder refuses
with the edit retained and a No working folder recovery choice. The default
preview requires a folder choice when the source has one. Opening runs nothing;
ordinary Send handles rerun through its own existing owner. The original snapshot
and receipts are read through bounded UTF-8 pages, with retained page/error retry.

Six SQLite cases cover creation identity collision, complete rollback
on a failed planning insert, source mutation between preview/create, original
goal/queue/draft/file preservation, optional revision-bound context, original
ancestry, later prompt/attachment omission from live context, archived input and
tool-output attachment retention after parent deletion/cleanup, wrong-chat
attachment refusal, newer draft/restart recovery, schema-45 migration, Unicode
page boundaries, over-limit history, malformed archive provenance and repair.
Two core cases cover source/input bounds and host-only guidance/context admission.
The bridge case validates unavailable-folder refusal and exact saved recovery after
parent/folder deletion. The final six SQLite cases pass, including tool-image
retention and explicit source accepted-plan preservation. The complete suites
pass 108 core, 245 bridge (one existing ignored case), and 104 SQLite tests,
plus their seven core and five SQLite integration cases.

The first broad Rust run overlapped the release build and Flutter suite. Core
passed 108 cases, but bridge finished with 230 passes, fifteen failures and one
ignored case: every failure was an existing review/worktree fixture's job-wait
deadline (25 seconds in review), not a branch assertion or schema error. The
isolated review case passed unchanged in 16.35 seconds. The complete rerun with
two test threads after the build passes, including all fifteen failed cases;
bridge takes 333.55 seconds. The overloaded run is retained as failed evidence.
Production deadlines and test assertions were not changed.

Five focused Flutter branch cases and sixteen combined branch/side-chat/AppHost
cases pass with the native case opted out. They exercise no-send opening with a
busy parent owner, exact lost reply, preserved newer branch and parent drafts,
stale source refresh retaining edited text and choices, failed preview, UTF-8
input narrowing, failed snapshot page followed by recovery, and missing-parent
controls. Compact Light/Dark renders under `output/design/eh07-branches-final/`
include top and folder views; both were inspected for visible labels, scrolling,
reachable actions and current-folder warning. Initial analysis found missing
braces in new widget/test code; those were fixed and analysis is being repeated.

The full Flutter suite passes 579 cases with fourteen native opt-ins skipped.
Final analysis and all-target warnings-denied Clippy pass, including the final
hover and accepted-plan test refinements. The
normal Windows build passes. Its isolated native branch journey passes in four
seconds at `output/native-ui/prompt-branches-20261024-hover/`: six public loopback
provider calls create a marker once, save two later parent exchanges, then read
the current file and answer the edited prompt through a separate owner. The file
is deliberately changed after preview and remains unchanged by branch creation
and rerun. The new provider context includes prior completed history and host
current-folder guidance; the old answer/later turns and newer private parent
draft are absent. Parent history/draft/goal are unchanged, and deleting the parent
leaves its archive readable with parent navigation disabled. Opening the archive
sends nothing. No credentials or user profile were used.

The first native attempt at `output/native-ui/prompt-branches-20261024-flow/`
missed the hidden hover row and stopped without creating a branch. The fixture
now settles scrolling before moving a mouse from outside into the actual message
region. A focused message-hover regression passes. The native journey then passes
without changing product hover behavior. The first broad Rust failures above
remain distinct from this fixture-input issue.

The combined native edited-branch, selected-text side-chat and queue/steering
journeys pass in fifteen seconds at
`output/native-ui/prompt-branches-20261024-final/`, retaining three receipts. The
branch still reads current files without repeating creation, side sharing applies
one selected conclusion, and the queue regression retains ordering, completed
effects and a newer unsent draft. The DLL and Dart bundle postdate the final
runtime edits. Later edits affect only fixtures and documentation. The bounded
Rust rerun, final analysis, formatting, warnings-denied Clippy and document/
publication checks pass. EH07.3 is complete within these documented bounds.

Limits: 16 KiB edited prompt; 100 completed exchanges and 1 MiB encoded original
transcript; 2 MiB total archive; existing four-prompt-attachment/four-context-
snapshot limits; 32 KiB UTF-8 archive pages. Archived attachment references retain
local bytes through ordinary cleanup without adding them to live context.
EH07.4 managed-checkout branching remains separate. Fixture/rendered evidence
does not qualify real-model or physical input reliability.

## EH07 linked side conversations — October 24

Saved user/assistant text can open a linked Side chat with a reviewed rendered
excerpt and optional question draft. Selection works in Markdown, selectable
code and the long-message plain-text fallback; named message actions offer an
editable preview as an accessible alternative. Opening creates no run. The
child receives one explicit context snapshot, no parent history, attachments,
folder, skills, goals, queue authority or approvals. Existing AppHost owners keep
the parent task and both drafts independent. No schema migration is needed.

Bring to parent previews a selected saved assistant conclusion. Queue/Steer
uses the existing input transaction and parent dispatcher, atomically recording
the return receipt without consuming either draft. Stable IDs reconcile lost
creation replies against all origin fields. A separate read-only return check
matches source, selection digest and mode; it never enqueues. A changed selection
cannot be mistaken for a prior accepted return. Deleted sources leave the frozen
excerpt readable; a deleted parent disables return without deleting the child.
Returning a conclusion does not complete or replace the parent's goal.

Four SQLite cases pass with actual transaction failures, stale source,
creation/return ID collisions, preserved newer drafts, exact retry after parent
deletion, full queue, paused queue, restart/uncertain delivery, retained original
goal, oversized UTF-8 question, encoded conclusion limit, assistant-source
validation and 32 retained returns. Failed creation/return rolls back its entire
write. The bridge case checks selected-context isolation and exact read-only
receipt reconciliation after parent deletion. The complete suites pass 106 core,
244 bridge (one existing ignored case), and 98 SQLite tests.

The full Flutter suite passes 571 tests with thirteen opt-in native cases
skipped. Eight focused side-chat cases additionally pass after the final test
refinements: actual Markdown/code word selection, no unavailable fallback action,
oversized question editing/recovery, parent owner still running on navigation,
failed save retaining the question and retry identity, full queue retaining its
selection, exact lost-reply receipt checks, compact missing-parent recovery and
clearing stale links when opening a fresh project. The final side-chat/workspace/
AppHost selection passes fifteen cases after that transition fix.
Analysis and all-target warnings-denied Clippy pass. Compact Light/Dark renders
are retained at `output/design/eh07-side-labeled/`; both were inspected for readable
content, reachable actions and layout. Render review caught labels disappearing
once fields were filled; the shared labels-above-fields component now keeps
excerpt/question labels visible. Eight focused cases pass after this refinement.
The initial UI checks caught a fixture
close-finder mismatch; the rendered Close control was used. Initial lint issues
were corrected. The first bridge fixture attempted context preparation without
a user prompt; it now follows normal preparation with an explicit prompt.

Bounds are 8 KiB excerpt, 4 KiB optional question, 1 MiB canonical source read,
32 KiB encoded origin and 32 returns per side chat. Normal 16 KiB combined input,
eight pending inputs, Stop, model/context budgets and live-input restrictions
remain. Return preview checks the encoded instruction allowance before freezing
the action; an oversized question stays editable. The first normal Windows build
passes. Its native rendered journey passes at
`output/native-ui/side-conversations-20261024-project/` with three provider fixture
requests: the original parent turn, the independent child question receiving only
the selected word, and one returned conclusion applied by the parent owner.
Opening/navigation makes no request. Parent draft/goal stay unchanged and deleting
the parent retains the child's frozen excerpt with return disabled. The first
attempt at `output/native-ui/side-conversations-20261024-flow/` used a folderless
Side parent and correctly hit the existing goal restriction; the fixture now
creates an isolated project parent. The final normal Windows build passes with
bridge/Dart artifacts newer than the final runtime edits. The combined native
side-chat and queue/steering journeys pass in eight seconds at
`output/native-ui/side-conversations-20261024-final/`, retaining both receipts.
The queue regression preserves its ordered follow-ups, safe-boundary steer,
completed effect and newer unsent draft. Final analysis, formatting and
warnings-denied Clippy pass; the final bridge test also checks encoded return
preview refusal before acceptance. EH07.2 is complete. Fixture evidence does not establish
physical Windows input, real-model competence or clean-machine behavior.
EH07.3 edited-prompt branches and EH07.4 explicit checkout choices remain open.

## EH07 diff review flow — October 23

Source Control now captures findings from a verified displayed diff line rather
than accepting a supplied source excerpt. Capture, submit and resolve use its
existing worker, Git lane and managed-checkout pins. Local list/get/note/removal
inspection is independent of checkout availability. Findings remain chat-scoped
when multiple chats share one Git view. Selected feedback reaches ordinary EH01
queue/steer admission; the AppHost refreshes that chat's retained owner and uses
its normal dispatcher. Scheduled/computer-use input restrictions remain.

Four host real-Git tests pass: left/right capture, undisplayed-line
refusal, wrong checkout and changed digest, stale file preservation, actual
SQLite save failure and retry, original index/draft preservation, local inspection
after moving the checkout, exact repeated submission after a file change without
duplicate input, explicit resolution with later host-observed evidence, split
pages, overlong lines and cancellation. The fourth case confirms that only the
owning chat can release a committed terminal review job after a lost status reply;
record inspection/release neither captures again nor submits input. The broad
suites pass 106 core, 241 bridge (one ignored) and 94 SQLite tests, followed by
the additional split-page and terminal-job cases.

The full Flutter suite passes 566 tests with twelve opt-in native cases skipped.
Focused review/Source Control/AppHost checks also pass, including failed capture
with the editor note retained, full queue with selected notes retained, committed
result reconciliation without another submit, uncertain job retention/Stop/check,
cross-chat UI state, inline notes and later resolution preserving the anchor,
plus a pending initial-read regression retaining the typed note while Save is
disabled. Analysis first found two unnecessary non-null assertions in fixture
diagnostics and a missing outer loop block; these test-only lint issues were
corrected and analysis passes.
Eight Light/Dark 420/1100 renders are saved under
`output/review-finding-renders/`; compact stale findings and wide Dark inline notes
were inspected. Render review prompted a spacing/contrast refinement. Initial
widget failures caught notification during dialog build (fixed by post-frame
loading); older Git-only fixture failures exposed missing feature-capability
separation. The ordinary AppHost enables findings while Git-only harnesses opt in.

The first two native attempts stopped at capture with no saved finding or queued
input. The narrowed diagnostic showed no host operation had started: Save was
pressed while the initial findings read was still loading. Waiting for that read
made the complete normal-bundle journey pass in sixteen seconds at
`output/native-ui/review-findings-20261023-capture/`. It captured a host line,
submitted exactly one input, received the frozen finding in the provider request,
applied a reviewed exact-text tool repair, retained the raw index and unsent draft,
and saved explicit resolution only against the later changed comparison.
The UI now disables Save during the initial read and preserves typed notes;
the pending-read regression passes with the full Flutter suite. Accepted input
receipts also appear immediately in their finding rows without requiring a list
refresh. Final build/native checks are being repeated. This
fixture failure is distinct from host capture failures, which retain explicit
terminal errors and notes. The failed evidence remains at
`output/native-ui/review-findings-20261023-flow/` and
`output/native-ui/review-findings-20261023-diagnose/`.

The final normal Windows build passes and its bridge/Dart artifacts postdate the
final runtime edits. Repeated native review and queue/steer journeys pass in
nineteen seconds at `output/native-ui/review-findings-20261023-final/` with both
receipts retained. The first journey verifies exact host line capture, one
accepted input, reviewed tool repair, unchanged raw index and retained unsent
Unicode draft, then explicit resolution tied to a changed comparison. The second
keeps queue ordering, safe-boundary steering, completed effects and a newer draft.
Warnings-denied Clippy, formatting and document/publication checks pass. The final
nine focused widget cases pass after the immediate receipt display refinement;
analysis is clean after adding a required block around that existing call. The
normal Windows build was repeated successfully after this lint-only edit.
Fixture and rendered evidence does not establish real-model competence or
physical Windows input. EH07.1 is complete; EH07 stays open for side conversations,
edited-prompt branches and explicit shared-folder/worktree branch choices.

## EH07 review persistence — October 23 (in progress)

The review contract now retains a stable finding ID/revision, original comparison
and left/right line/excerpt, submission IDs and a separate explicit resolution
assessment. Validation bounds IDs, notes, excerpts, selected counts and combined
input size. Resolution needs a changed comparison of the same file; an applied
queue receipt never resolves a finding by itself. SQLite schema 45 is additive;
applied migrations are unchanged. The older worktree migration test now asserts
the current schema version instead of stopping at 44; its initial outdated
assertion failed (89 passed, one failed), and the corrected migration passes.

Selected findings enqueue through the existing EH01 edit path in the same
transaction as their submission records. The composer draft remains. Failed
receipt publication rolls back the queue input too. Exact submission retries
return existing evidence; collisions with unrelated instruction IDs, different
selections/modes or stale finding revisions refuse. Restart keeps delivery
uncertain and suspended for explicit inspection, without enqueueing again.
Local finding removal never cancels a queued instruction or edits source files.

Complete core/SQLite suites pass (106 and 94 tests), including instruction-ID
collisions, duplicate selections, aggregate input size and encoded record size.
Tests also cover stale/cross-chat mutations, failed save, full queue,
preserved input order and draft, explicit later-revision resolution/reopen,
128-finding capacity with deliberate removal/retry, 32-row pagination, deletion
and version-44 migration retaining history. Warnings-denied Clippy exposed a large
enum variant and extra borrows after queue extraction; these were corrected.
The encoded-record guard also accounts for JSON expansion of source control
characters before a database size constraint can reject publication. Broad bridge
regression passes 239 tests (one ignored). The final normal Windows x64 build
passes with the encoded-record guard; warnings-denied all-target Clippy,
formatting and document/publication checks pass. Logs are retained locally under
`D:/Temp/dolores-eh07-review-storage-verified.log`,
`D:/Temp/dolores-eh07-storage-bridge.log` and
`D:/Temp/dolores-eh07-storage-build-final.log`.
The existing queue/steer journey also passes in five seconds against that final
normal bridge at `output/native-ui/review-storage-queue-20261023/`: reorder,
safe-boundary steering, completed-effect preservation and a newer draft remain
intact after extracting the shared transaction path. This checks the existing
queue UI, not the still-pending finding controls or physical Windows input.

This is a backend increment. Host-validated diff capture, stale comparison UX,
review controls, normal native repair flow, side conversations and edited-prompt
branches are not yet delivered. EH07 stays open. No provider credentials,
normal-profile migration or external effect was used to establish these checks.

## EH06 archive and restore — October 23

Archive has an explicit review of the owned UUID folder, original checkout kept,
retained snapshot, archive path and inventory size. Raw bytes for all direct
regular files, including ignored data, the raw index and empty-directory metadata
are flushed before removal. A Git snapshot commit with owned ref retains those
same raw objects plus the original HEAD, index tree and working tree. Removal
rechecks the complete saved inventory, original index, state revision, parent/root/
common-directory identities and Git pointers before saving Archiving and invoking
supervised Git for the exact managed path. Missing/changed roots, aliases, nested
repositories/submodules, Windows path aliases and over-limit inventory refuse
without removal. Only the app's own Git lock reason is unlocked. Source checkout,
chat history, drafts and prior transfer receipts remain.

Restore requires a separate review, retained evidence/ref validation and an absent
target. Save Restoring before reserving the directory; save its new filesystem
identity before checkout. Setup/hooks/filters stay skipped. A flushed per-file
plan records initial and retained bytes before publication. Explicit continuation
skips matching completed bytes, restores only validated initial bytes and refuses
unrelated work. Original index bytes restore the staged/unstaged split. Files,
ignored binary data, empty folders and Windows readonly/Unix mode permissions
are restored; custom ACL migration is not claimed. Restart never executes a
saved intent. Inspection can reconcile completed removal/restoration without
repeating it; incomplete work retains concrete archive paths and explicit resume.

Current checks:

- Nine focused real-Git archive cases pass: staged/unstaged plus binary/CRLF/ignored/
  readonly/empty-folder round trip; stale review, wrong owned parent, occupied
  destination and missing-root preservation; repeated cycles; failed final
  Archived save inspected without another removal; partial restore with foreign
  bytes refused, then explicit resolution and continuation skipping completed
  files (verified unchanged modification time); failed Ready save inspected
  without recopying; altered backups, ignored 65 MiB over-limit file and injected
  pre-removal storage-full failure retaining the complete checkout and staging;
  stopped-terminal/language-pin ownership; actual reviewed Node setup exiting 7
  with its output preserved through archive/restore; and saved file↔directory
  transitions while keeping the original checkout untouched.
- The bridge storage-failure fixture injects a refusal at the durable intent boundary.
  A separate storage test now sets a small SQLite page quota and exercises an actual
  SQLITE_FULL allocation refusal (checked by error code), transaction rollback,
  explicit retry after quota release and restart inspection without replay.
  Its first run exposed a generic error message; Worktrees now reports full storage
  with free-space/refresh/review guidance. Locked-publication tests exercise actual
  Windows I/O refusal and retain old bytes. The bounded quota qualifies database
  exhaustion; physical whole-volume exhaustion during Git remains unqualified.
- Nineteen focused Flutter archive/worktree tests pass: explicit reviews and
  owned-chat actions, saving source draft before publication, failed draft save
  retaining the review, lifecycle inspection without replay, explicit restore
  continuation, storage-full recovery with a fresh review, archive paths scoped
  to the selected checkout, and existing creation/
  recovery controls. Eight saved Light/Dark
  420/1100 renders are retained under ignored `output/worktree-archive-renders/`;
  narrow Light review and wide Dark recovery were inspected. Flutter analysis
  and all-target warnings-denied Clippy pass at this checkpoint.

The first normal Windows build and rendered native create/transfer/archive/restore
journey pass: archive removes only the managed checkout; restore retains the
saved file, ignored binary, empty folder and raw index; the original project and
source draft remain. Receipt/image are retained at ignored
`output/native-ui/worktrees-archive-20261023-flow/`. Final broad regression and
rebuild after path-transition support are in progress. An initial failed-setup fixture incorrectly used Git's
argument-prepending transport to launch a shell/Node; it is being corrected to
use the actual reviewed command tool; the corrected fixture passes. Its failures are not counted as product
qualification. A concurrent test relink encountered the running Windows test
executable lock; that compile attempt is also not a pass. The first broad run
passed 237 tests with the old setup fixture failing (one ignored). The new
transition test then caught the snapshot directory misclassification; that
runtime bug was fixed and its focused round trip passes before final regression.

Bounds: 64 MiB raw saved content including ignored files, 20,000 entries, 16 MiB
raw index and manifest/restore plan, two-minute bounded inventory/object retention,
128 retained snapshots/drafts per worktree and eight live lifecycle reviews.
Unsupported/oversized checkouts remain in place with narrowing/external-backup
guidance; no archive removes unsupported or uncaptured data. Git-created partial
materialization without a clean baseline or saved ownership needs manual recovery,
not a reset. Native physical input, whole-volume pressure and model competence are
not established by these fixtures. Required creation/ownership/transfer/archive/
restore journeys and bounded database-exhaustion recovery pass. EH06 is complete
within these documented Windows bounds; broader resource and real-model checks
remain separately recorded for EH10. Deferred H21 is unchanged.

## EH06 reviewed transfer — October 23 (in progress)

The Worktrees panel now offers a separate Transfer changes review. It compares
saved files against the managed source's frozen base, including committed work
and non-ignored untracked files. Choose displayed paths and a destination chat
in the same Git repository, inspect source/destination and the exact selected
binary patch, then explicitly Apply. Destination staging is never reset or
copied from the source. Conflicts refuse Apply and retain both file versions.
Unsaved editors, active repository tasks/terminals and unavailable ownership
refuse before publication; reviews and saved files remain inspectable.

Recovery evidence lives beside managed checkouts in a host-owned
`worktree-transfers/<uuid>/` directory, independently of chat deletion: raw numbered
source/destination bytes with path/digest manifests, raw index copies, the patch,
and separately flushed immutable review/intent/result files. Source/destination
Git trees and index trees have owned refs. Git filters/hooks and external diff
are disabled; a private index captures snapshots without changing either user's
index. Raw copies preserve binary bytes and original line endings independently
of Git text normalization. Apply rechecks files, commits, index digests,
configuration, ownership and patch; recovery files are pinned and validated before
publication. An intent without a saved result remains uncertain; restart and
status checks never apply it. Inspect both folders and retained versions before
resolving or reviewing a new transfer. No automatic rollback or retry is claimed.

Bounds are explicit: 256 selected paths, 2,000 compared changed paths, 20,000
snapshot files, 64 MiB of saved content per checkout/raw side, 16 MiB raw index,
512 KiB selected patch through the existing Git transport, 8 live transfer
reviews, 128 listed matching receipts and a 4,096-entry evidence inventory scan.
Over-limit/unsupported symlink or submodule snapshots retain the checkout and
offer narrowing or external inspection. Ignored files and unsaved buffers are
not transferred. These bounds do not change command or model budgets.

Current evidence:

- Eight focused real-Git snapshot/transfer tests pass: exact binary/CRLF raw retention,
  skipped filters and unchanged source index; bad paths, cancelled capture and
  occupied backup preservation; selected binary addition/deletion with both
  indexes and unrelated staged/untracked work retained; committed changes plus
  non-overlapping destination edits; conflicting versions with disabled Apply;
  stale files/patch and one-use token refusal; intent-save and result-save failure
  with explicit inspection and no restart replay. The final backup-integrity
  and snapshot-limit check refuses altered recovery bytes and a 65 MiB sparse
  fixture without removing it, then recovers after explicit fixture narrowing.
- Seventeen focused Flutter worktree/transfer tests pass: exact selection and
  destination review, no Apply before review, conflict recovery with copyable
  evidence path, uncertain status polling the same job once, preserved source
  draft and existing creation/recovery controls. Four conflict renders at
  Light/Dark 420/1100 pixels are retained under ignored
  `output/worktree-transfer-renders/`; narrow Light and wide Dark were inspected.
  Tests initially looked for receipt rows before scrolling the lazy list; their
  recovery checks now scroll to the actual receipt. Product error messages remain
  visible at the top without discarding the draft.
- Flutter analysis, warnings-denied all-target Clippy and the normal Windows
  build pass. The broad bridge run passes 229 tests with one ignored; final
  backup-pin/limit changes are additionally covered by the focused case and the
  native bridge. The extended native worktree/transfer journey passes using
  isolated public data and the fresh release bridge: conflict refuses Apply;
  both versions remain unchanged; explicit resolution of only the fixture's
  destination allows a new review and selected transfer, preserving staging and
  the source draft. Evidence is retained under ignored
  `output/native-ui/worktrees-transfer-20261023-settled/`. The first attempt failed
  because the fixture accessed a not-yet-built control with `.last` rather than
  waiting for it to exist; the corrected wait passes without a product retry.
  The final repeat also passes and retains an inspected transfer-result render
  and receipt at `output/native-ui/worktrees-transfer-20261023-final/`. The first
  image was captured while the final receipt refresh still owned the controls;
  the final fixture waits for the enabled dialog Close control and scrolls to
  the completed result before capturing it.

The conflict flow deliberately keeps the source and destination untouched rather
than writing conflict markers into either checkout. Publication can still fail
partway on I/O; intent/result state and retained originals are the recovery path.
No real-model or physical pointer/IME qualification is claimed. EH06.3's native
journey passes; EH06.4 archive/restore remains open.

## EH06 checkout ownership — October 23

Management reservations now include the common Git directory, not only the
checkout list captured during review. Read-only bounded local pointer inspection
recognizes an externally added checkout and nested projects without executing Git
under the run coordinator. Missing, malformed, aliased or unsupported ownership
metadata cannot silently establish independence during management. Already-active
Git jobs and runs in the same repository refuse creation; later Git/terminal/run
admission cannot bypass the reservation through a new checkout. A changed Git
registration list invalidates creation before its saved intent and offers a fresh
review. Existing dirty work is retained.

Chats within one managed checkout share that checkout's immutable saved root as
their admission scope, including nested project chats. File/command working
folders and settings stay bound to each selected workspace. Independent managed
checkouts still run concurrently; the nested chat queues and cancellation releases
the expected owner. Parent project scopes also overlap nested Git mutations.

Verification:

- Full bridge library suite: 222 passed, one ignored. The new real-Git cases
  refuse changed registrations before saving an intent, keep the external dirty
  file and original index, recover with a new review, cover common-directory
  reservations versus existing Git jobs/later checkouts, refuse malformed
  metadata and block an active external-checkout run until explicit release.
- Root/subfolder queue identity is checked through actual Start registration;
  the credential-free waiting provider verifies independent runs and release.
  A real Windows terminal writes a file only inside its bound managed checkout;
  editor Save and Git status use that same checkout, preserve the original file,
  and retain pins until terminal Stop/Close. Pointer unit coverage includes a
  relative commondir, nested folder, malformed marker and the 8 KiB bound.
- The first full run passed 221 tests and failed the new headless terminal
  fixture because it did not answer the console cursor-position query. The
  fixture now answers that query as the terminal frontend does; the real command
  and full suite pass. A concurrent relink attempt failed because Windows still
  owned the running test executable; it is not counted as a test pass.

The final focused worktree filter passes 13 tests; warnings-denied all-target
Clippy passes. The normal Windows build and its final rebuild after the
saved-revert guard pass. Saved-change revert now retains checkout
pins and the repository/editor lock through publication; it refuses management
conflicts and unsaved editor buffers. Three existing revert tests pass, and the
native bound-checkout fixture exercises these new refusal/recovery paths.
That fixture initially supplied its pre-save editor version after Save incremented
the version; it now reads the current workspace version before the unsaved edit,
and passes. This was a fixture failure, retained separately from product checks.
No live-model or physical input qualification is
claimed. This completes the checkout ownership increment, not EH06 as a whole:
reviewed transfer, archive/restore, destructive lifecycle failure recovery and the
integrated worktree journey remain open. Native account permissions are unchanged;
the ownership coordinator is not a filesystem/network sandbox.

## EH06 creation and inspection UI — October 23 (in progress)

The Worktrees panel is reachable from chat actions and the working-folder menu.
An explicit base and setup policy produce a review of the original folder, new
folder and exact commit. Creation uses the existing supervised Git owner and
atomic chat/binding intent. Opening the new chat is a separate action, preceded
by saving the source draft. Retained records are listed independently of recent
chats; creation recovery inspects an existing checkout without rerunning setup.
Shared/nested chats inspect their actual managed checkout. Job polling now
returns a terminal error envelope separately from status transport failure, so
uncertain polling never resubmits creation.

Exercised so far:

- Nine real-Git bridge worktree tests pass, including independent checkouts,
  preserved original index/dirty files, stale review/configuration, occupied or
  replaced roots, interrupted saves and explicit reconciliation, shared/nested
  ownership inspection, and independent/queued run pins.
- Fourteen focused Flutter worktree/workspace tests pass. They exercise reviewed
  creation, actual target navigation and return to the source draft; failed
  creation with retained intent and explicit inspection; one uncertain status
  followed by polling the same job; Stop with retained interrupted work; failed
  draft save refusing navigation; identity-recovery errors; and normal chat-menu
  access. The native opt-in case is skipped in that deterministic run.
- Eight saved Light/Dark, 420/1100-pixel renders are retained in ignored
  `output/worktrees-ui-renders/`; inspected narrow Light creation and wide Dark
  recovery. Tests caught errors scrolled out of view; failures now reveal the
  error and recovery controls without losing the draft.
- Flutter analysis passes. An earlier concurrent check hit Windows error 1455
  and Flutter out-of-memory; that run is a failure. Serial focused verification
  passed afterward. No unrelated application processes were stopped.

The normal Windows build passes. `qualify-native-ui.py --case worktrees` passes
against that fresh release bridge: stale configuration is refused before saving
an intent; a new checkout contains the reviewed committed file; editing it leaves
the original dirty file and index unchanged; Open chat selects its actual folder;
returning to the source restores its unsent Unicode draft, also verified in native
storage. Receipt and inspected render are retained in ignored
`output/native-ui/worktrees-20261023-settled/`. Two earlier native fixture runs
failed because the test clicked before a control was enabled and then waited for
animation settlement while native refresh was pending. The fixture now waits for
enabled controls and completed refresh explicitly; the earlier outputs remain.
The native case uses a public Git fixture and no provider credentials. No live
model or physical keyboard/pointer behavior is claimed. Transfer, archive/restore,
the remaining ownership/race audit, real
disk-pressure recovery and full EH06 acceptance remain open. Earlier evidence
and failures below remain historical, not silently upgraded to passes.

## EH06 managed worktrees — October 22 (in progress)

The first implementation increment adds private lifecycle contracts in
`crates/dolores-core/src/worktrees.rs` and additive SQLite schema 44 in
`crates/dolores-store-sqlite/src/worktrees.rs` / `lib.rs`. The core storage port
is additive; core now uses the existing workspace UUID dependency for identity
validation. A creation intent, new chat, and actual target-folder binding are
atomic. Immutable base/source/target/setup fields, monotonic revisions, and
phase-specific recovery prevent stale saves from redirecting a checkout or
discarding its saved snapshot. Lifecycle records survive chat deletion and are
paged independently; local paths remain excluded from conversation exports.

Verification exercised:

- `cargo test -p dolores-core -p dolores-store-sqlite`: 103 core unit tests,
  7 core integration tests, 88 SQLite unit tests, and 5 SQLite integration tests
  passed. After tightening recovery transitions, the focused `worktree` filter
  passed all 7 applicable tests again.
- Creation/binding, restart interruption without replay, stale updates, duplicate
  destination rollback, retained archive snapshot after an injected SQLite write
  failure, chat deletion, export privacy, and multi-page retained records.
- A version-43 fixture reopens as schema 44 with its original conversation,
  working-folder binding and unsent draft. The older version-3 migration test
  also passed in the full suite.
- `cargo clippy -p dolores-core -p dolores-store-sqlite --all-targets -- -D warnings`
  passed. Initial compiler findings (missing core UUID dependency and a temporary
  MutexGuard coercion) were fixed before the successful tests.

### Git creation and host ownership increment

Seven real-Git bridge tests passed with `cargo test -p dolores-flutter-bridge
managed_worktree -- --nocapture`. They create public repositories under temporary
fixture directories, with no provider credentials or normal application profile:

- Create two detached, locked worktrees at the exact reviewed commit; edit one
  independently; retain the original checkout byte-for-byte for its staged index,
  unstaged file, and untracked file. Chat binding points to each actual checkout.
- Configure an executable post-checkout hook and an invalid required Git filter;
  managed creation skips both and materializes the committed file without setup.
- Reject a different chat using the review, changed repository configuration, an
  occupied destination, a replaced checkout directory, and redirected Git metadata.
  Existing contents stay in place and recovery details remain inspectable.
- Inject failure when saving Ready after successful Git checkout, reopen the host,
  and explicitly reconcile the existing checkout. Retained user-created work and
  Git index remain unchanged, and no extra checkout or setup is executed.
- Pre-cancel creation before saving an intent; no checkout is created. Repository
  ownership reservations refuse conflicting Source Control operations.

The bridge now exposes background review/create/poll/cancel and explicit creation
reconciliation operations. Startup marks in-flight lifecycle records for recovery.
Run admission, editor and Source Control entry validate Ready state plus target,
common-directory and Git-registration filesystem identities. Creation reserves
the registered repository roots while active; source Git entry and run admission
use the existing shared ownership checks. Completed job results remain bounded.

The existing Source Control regression filter also passed all 25 tests after the
initial reservation change. Subsequent ownership changes retain directory and
Git-pointer handles through queued/active runs, Source Control jobs, terminals
and language servers. Shared and nested chats resolve managed ownership by their
actual folder. A Windows fixture verifies that these pins refuse rename/pointer
replacement and that unsaved editor buffers remain inspectable/checkpointable
when the checkout disappears. The parallel-run fixture now passes: independent
checkouts run concurrently, a shared checkout queues, and cancellation releases
ownership. It caught missing project-scope registration, now saved atomically
with creation. Its provider/settings snapshot was also corrected to keep the
fixture credential-free.

### Foundation commit verification — October 22

The latest combined library test run passed 103 core tests and finished the bridge
with 207 passed, 2 failed and 1 ignored. The bridge failures were
`source_control::diff::tests::large_saved_files_render_their_small_changes_in_dolores`
(Windows error 112, disk full) and
`repair_evaluation::tests::real_installed_cargo_executes_fixed_baseline_and_candidate_fixture`
(candidate test execution incomplete; cause not yet established). SQLite was not
reached in that combined run; its earlier results above remain the available
evidence. These failures are unresolved, not waived or counted as passes.
Formatting was checked again before the foundation commit. No normal EH06 desktop
build or native user journey has been qualified.

Still required: UI creation/details/recovery, complete file/terminal/language and
queued-run ownership audit, shared-repository races including externally added
worktrees, reviewed transfer, snapshots/removal/restore, failed manual setup and
restore plus real disk-pressure recovery, and a normal Windows build/native journey.
The injected SQLite failure does not qualify disk exhaustion during Git operations.
The user requested a foundation checkpoint commit before the full batch is done.
This commit does not complete EH06; all four task checkboxes remain open.

## EH05 command sessions and isolation — October 22 (complete)

### Command and MCP integration checkpoint

Shared launch policy now routes explicit isolated Node commands, supervised
sessions and MCP servers through the same LPAC lease. Reviews retain exact root
identities before execution; replacements refuse and a fresh review recovers.
Native MCP configurations deserialize unchanged. Full access does not bypass
isolated-root review. Unsupported runtime/host combinations fail without native
fallback. Bridge startup and explicit Commands cleanup retain unexpected files
and never replay a command.

The command crate passed 32 cases plus two parent-invoked worker entry points
before the descendant-by-name refinement. The refined offline CommonJS build
passes with inherited child stdio and retains both HTML and child artifacts.
Node-created child pipes instead stalled, including with a 2-second Node child
timeout; that failed diagnostic is not counted as successful compatibility.
Host deadlines/Stop remain the recovery boundary. The runtime copy preserves its
executable basename and provides only its owned directory on PATH.

All 12 MCP transport cases pass, including an isolated real Node stdio discovery
and tool call, an actual denied sibling write, no launch from default/unavailable
isolation hosts, stale read-only-root refusal and explicit fresh-review recovery.
The three Commands bridge cases pass, including cleanup failure retaining its
foreign file/output/draft, no retry from polling, and explicit cleanup recovery.
All 15 focused MCP/Commands widgets pass. The new compact review checks exact
writable/read-only roots and no native-permissions wording, starts nothing before
Inspect, and invalidates review when the mode changes while retaining fields.
The initial assertion tried to inspect a lazily unmounted field; scrolling it
back into view corrected the test. Changed bridge all-target compilation passes.

The normal Windows build passes through the supported helper. Its bundled and
optimized Rust bridge DLLs both hash to
`77f40d85fe214d63dc9ff7e1d156317820620a7e284436369c490043130560c8`.
The fresh combined suite passes 101 core cases, seven core integration cases,
32 command cases (plus two parent-invoked workers), and 12 MCP cases. Changed
Dart analysis reports no issues. The compact dark MCP review render was inspected
at `output/eh05-mcp-isolation-review-fonts/mcp-isolated-review-dark.png`:
grants and both actions are readable. The render-only theme supplies an explicit
system font for otherwise unspecified button fonts; it changes no product code.

The native isolated command journey passes at
`output/native-ui/5ce7b1ba646148b4a8a98a1e578398fe/commands-isolated-receipt.json`.
The normal bridge consumes two public provider responses, reviews isolation in
Flutter, denies an actual sibling write (EPERM) and TCP connect (EACCES), accepts
exact 20-byte Unicode input, interrupts the owned process and reopens durable
output. Cleanup completes with zero pending journals and the draft survives.
Its saved render shows isolation, retained denial/input output and disabled
finished-session input. Some generic labels still use test-font fallback; this
does not establish normal desktop typography or physical IME acceptance.

Retained eight-case transport evidence is at
`output/isolation-qualification/0a73dabee22e4e94b012c4a26f34d571/receipt.json`.
The separate opt-in public network receipt is at
`output/isolation-qualification/b9fd5cb03ac34da993bd2110cb033fb4/receipt.json`.
Native controls connect to an owned listener on the local adapter and to
1.1.1.1:443, and resolve example.com through the system resolver. LPAC returns
EACCES for both connections and EAI_FAIL for the same resolver operation.
EAI_FAIL is recorded as resolver failure, not mislabeled as a socket-denial
code; no timeout counts as denial. No payload or private DNS name is sent.
This is evidence for these exercised APIs on this Windows host, not a universal
claim about every OS broker or future runtime. The case is opt-in and excluded
from deterministic CI; `--public-network` retains its separate source-bound log.

Final MCP save/restart qualification now passes (see the final integration follow-up below). Node-created
child pipes remain an explicit unsupported combination. The final integration follow-up closes the declared EH05.3/.4 gates. Public fixtures only;
normal profiles, credentials, installed runtime ACLs and firewall remain unchanged.

### Final integration follow-up

The native MCP save fixture initially refused a relative script at Enable: the
inspection fingerprint used a canonical Windows root while the save recheck used
the caller spelling. Fingerprinting now canonicalizes its root consistently.
The isolated MCP regression explicitly exercises verify-before-save and passes.
Clippy also exposed the larger review payload in the existing Restore enum;
boxing that variant preserves behavior and all three existing revert tests pass.
Five-crate all-target Clippy passes with warnings denied.

The broad bridge sweep passed 199 cases and skipped the explicit live-provider
case; its new human-input fixture failed once by reading the output file before
the child had written it. Its stdout echo occurs before append. The test now
waits for the exact file effect as well as output, preserving the product
contract that pipe delivery does not prove execution. No production timeout or
retry policy was changed. All three focused command bridge cases pass afterward;
the corrected case also passes eight explicit repetitions without retries.
The final normal Windows build passes after the fingerprint correction. Bundled
and optimized bridge DLLs match SHA-256
`d2cdb892fe44c2c7f9f234c3f9cd8f857ac5f11aaabbe5b3a313b2500cfbf659`.
Native MCP save and separate-process restore both pass under
`output/eh05-native-mcp-isolated-retry/`: seven public model-fixture requests,
zero live-provider requests, no credential flow, schema 43. Inspection, enabling,
exact tool approval, metadata-drift refusal, Stop, disable/re-enable, cancelled
inspection, saved isolation restoration, export and Forget all pass. Restart
starts no server and leaves no pending cleanup. The earlier failing fixture at
`output/eh05-native-mcp-isolated/` remains retained.

EH05 is complete for its declared Windows Node/MCP boundary. Other runtimes and
native helper paths remain explicit; Node-created child pipes are unsupported.
Real-model reliability, physical Windows input and clean-machine qualification
remain separate EH10/release work. Earlier transport paragraphs below describe
historical checkpoints and do not override this final status.

### Earlier LPAC runtime transport checkpoints

The Windows LPAC transport now exists in `dolores-tools-command` behind an
unadvertised library API. It copies the reviewed executable into private runtime
storage, creates a unique package identity with registryRead and no network
capabilities, applies writable-workspace/read-only-resource grants only to that
package SID, and supplies the LPAC/handle-list process attributes before resuming
the child inside its kill-on-close job. Attribute storage and SID allocations
remain live through process creation. No setup path falls back to native launch.

A synced journal precedes profile/grant effects. Cleanup merges the current ACL
and revokes only the owned SID, retaining other trustees. A host gate serializes
its grant/cleanup writes. Root handles pin their namespace while live; recorded
volume/file identities reject replacement roots during recovery. Reparse paths,
hard links, overlapping grants and grants containing private state are refused.
Validation/cleanup explicitly stop at 65,536 entries or 64 levels; runtime copies
stop at 256 MiB. These are new transport bounds, not changes to native commands.
Cleanup failure retains its journal; completed cleanup has an idempotent marker,
and unexpected private files are preserved. Recovery never reattaches or replays.

Retained transport receipt:
`output/isolation-qualification/3f421a58405d4270b822ed74da1caf0f/receipt.json`.
All six parent cases pass; two ignored worker cases are explicitly invoked by
their parents. No cleanup journal remains. The receipt binds source SHA-256s.
The broader command crate run passes 28 cases, with those two helper workers
ignored at the top level. Changed core/command/bridge all-target Clippy passes
with warnings denied; the two bridge command-session regressions still pass.

Exercised actual Windows controls:

- Exact Unicode stdin and separate stdout/stderr; workspace read/write, additional
  read-only reads, denied read-only writes and denied sibling access.
- Matching unrestricted Node TCP IPv4/IPv6, UDP and inbound-bind controls succeed.
  LPAC returns EACCES for all four operations; no timeout is counted as denial.
- Unrestricted WRITE_DAC opens succeed for user-owned read-only/workspace/state/
  runtime objects; LPAC opens return error 5 for all four, preventing user
  ownership from silently expanding those ACL rights.
- Two package grants coexist and one package's cleanup preserves the other's
  ability to run. Setup failure after granting revokes before launch. Hard-link
  and junction workspaces refuse; fresh reviewed setup then succeeds.
- A hard-killed owner loses both child and grandchild before delayed writes. The
  grandchild retains sibling denial and actual TCP EACCES before the kill. Reopen
  refuses a replacement root, preserves its unrelated file, recovers after the
  original root is restored, and permits a fresh explicit command without replay.
- Interrupted private-file cleanup retains an unexpected file, then succeeds
  after explicit removal; normal completed workspace results remain available.

These tests use only public temporary workspaces and owned loopback listeners.
Installed runtimes, normal app profiles, firewall, exemptions and installed
features are unchanged. The latest retained run places all temporary fixtures
under its output directory. Ordinary Node-file startup still needs compatibility
qualification; richer module/build runtimes, LAN/external controls, review-bound
grant identities, cancellation during setup, product command/MCP routing,
cleanup UX and a fresh integrated normal build remain open. The product continues
to refuse requested isolation until that integration is qualified. EH05.3/.4
and the batch commit remain open.

The model-side error-classification gap also has a red/green core regression:
input-limit and stale-revision failures used to become generic folder errors.
They now retain the actual bounded reason and point to Commands for inspection
before another reviewed operation. All 100 core cases and seven integration cases
pass after this repair. A normal release containing this later change is pending.

The latest transport refinement is source-bound at
`output/isolation-qualification/6731f009e1d74c4f9e8471990400c954/receipt.json`:
all seven parent cases pass, with no retained cleanup journals. Before removing
an inheritable package ACE, cleanup now validates the full tree again. A junction
introduced after child exit causes explicit retained cleanup rather than ACL
propagation through that alias. The regression compares the outside target's
DACL bytes and contents before/after, then removes only the public fixture alias
and verifies successful recovery. Cleanup errors preserve the actual alias/root
reason and avoid claiming that no command ran after execution. Core/command
all-target Clippy passes after this final change. The earlier six-case receipt
remains historical evidence, not a hash match for the newer source.

Command-session bridge/storage/UX integration is now implemented. Additive schema
43 saves a starting receipt before native launch, append-only bounded raw output,
input delivery state and final cleanup evidence. Startup marks previously live
records lost and pending input uncertain; it never reattaches or replays them.
The bridge saves progress once per second and at final cleanup. A failed evidence
write cancels the owner, preserves prior rows and keeps cached output inspectable
with an explicit Retry saving action. Saved and cached pages are separately
bounded; a finished handle can be released without deleting its durable output.
JSON/Markdown export includes these records even without a completed reply and
preserves raw bytes alongside display text. Chat deletion cascades only its rows.

Activity and Earlier tasks expose Commands with paged output, exact human stdin,
EOF, Interrupt, Close and save recovery. Model input uses its separate exact
review, including Full access. Unsent command input survives inspector navigation
in memory and never replaces the conversation draft. It is not saved across app
restart. Refresh/poll only read evidence; no action is retried automatically.

Current checks: all 22 command-crate cases, 99 core cases plus seven integration cases, 83 store cases plus five
memory audit cases pass. Four store cases cover evidence conflicts/write failure,
restart uncertainty, migration/paging/deletion and lossless scoped exports. Two
bridge cases pass for human ownership/input, post-release output and failed-save
cache recovery. The broad bridge run passed 198 cases and ignored one explicit
live-provider case; one old catalog-count assertion failed (14 tools versus 13).
Its updated Windows-aware assertion and tool-presence check pass in a focused
rerun. The packaged registry test first failed at its old 20-entry limit and now
passes at 23 after including all shipped host tools, browser and two MCP slots;
execution/output/task allowances did not increase. The full core rerun also
exposed its older 20/21 boundary test, now checked at 20/23/24 with duplicates
still refused. All 99 core cases and seven integration cases then pass.
Four-crate all-target Clippy
passes with warnings denied.

Four focused command widgets pass: model stdin review, light/dark compact
input/EOF/interruption/closed-handle inspection, and stale-input/read/save failure
recovery with no duplicate pending Send. Twenty-one combined command/review/MCP/
Stop widget cases passed before the final added stdin-review/EOF checks. Changed
Dart analysis passes. Saved compact renders were inspected under
`output/eh05-command-ui/`: body/output and wrapping footer fit; some generic action
labels still use Flutter test font fallback, so they do not establish normal-app
typography. The normal Windows build passes. Native rendered integration results
are recorded below when complete. Production isolation, actual crash cleanup,
physical Windows input and real-model acceptance remain open. EH05 stays open
and uncommitted until its required isolation gates pass.

The first native command attempt exposed an agent-boundary gap: start receipts
with both canonical operation JSON and a reviewed executable were rejected by
the old run-command-only approval shape. The direct transport/LoggedTool tests
had bypassed that validation. A new core agent test fails on that exact rejection
and passes after limiting executable previews to run_command or session start,
requiring a known session operation and allowing its bounded 32 KiB canonical
query. Input with an executable, start without one, unknown operation and
oversized arguments all refuse before approval. Retained native failed evidence:
`output/native-ui/cffe746ec9ba4509b677eb22a3a716fe/`. A prior attempt in
`output/native-ui/136bb6343285432a857db55ae62a51eb/` passed the seven existing
journeys, then failed because the new fixture tapped Send before pumping its
enabled frame; that fixture timing was corrected. Neither failed run is counted
as command native acceptance. The normal build and native journey are being
rerun after the approval repair.

The command journey now passes against the rebuilt normal bridge at
`output/native-ui/12a1c568311a418081a8e2d548154bf7/commands-receipt.json`.
Rendered controls review and launch the Node fixture, deliver exactly 20 UTF-8
bytes (`PUBLIC_INPUT 世界\n`), observe the file/output, interrupt its owned tree
with `cleanupComplete=true`, retain the newer conversation draft and reopen saved
output after live-owner release. There are exactly two public loopback provider
requests and a durable human-input intent; no real provider or credential is used.
The bundle and optimized bridge DLL SHA-256 match. The saved native-connected
image was inspected: status, input evidence and output are visible; some action
labels/CJK still use test fallback fonts. This qualifies the native bridge plus
rendered fixture, not physical desktop input, normal typography or OS isolation.

### Node isolation compatibility probes

The static Rust A/B probe still passes after diagnostic stdio/runtime additions:
`output/eh05-isolation/rust-post-stdio-20261022-01/paired-receipt.json`.
These additions copy Node into fresh owned output; installed runtime, drive and
ancestor ACLs are untouched. Ordinary Node file entry exits 1 before a receipt.
The captured diagnostic stderr identifies `realpathSync` → `lstat 'D:\\'` →
`EPERM`; it is not counted as denied network access. An explicit
`--preserve-symlinks-main` trial and an eval control reach the public fixture.

At `output/eh05-isolation/node-stdio-paired-20261022-02/paired-receipt.json`,
unrestricted Node reaches the owned TCP IPv4 listener. Both container variants
retain granted read/write and deny sibling operations with EPERM; only LPAC
denies the ALL_APPLICATION_PACKAGES-only canary. LPAC returns EACCES/-4092 from
the actual connect. Unicode stdin and separate stdout/stderr pass with exactly
the intended bytes through inherited handle-list pipes. The first stdio attempt
exposed Python text-mode CRLF conversion in the unrestricted control; binary
input fixes that diagnostic mismatch without normalizing child receipts.

The aggregate Node A/B probe remains **failed**, because ordinary AppContainer
reports ETIMEDOUT rather than the declared successful ordinary-network control.
This timeout is never counted as denial or success. The receipt separately shows
`candidateNetworkDenialPassed=true`, `stdioPassed=true` and
`ordinaryNetworkAllowObserved=false`. Draining the owned accept backlog in a
fresh trial does not change that outcome:
`output/eh05-isolation/node-stdio-drained-20261022-01/paired-receipt.json`.
The file-entry failure is independently retained with stdout/stderr in
`output/eh05-isolation/node-file-stdio-20261022-01/paired-receipt.json`.
Every completed trial reports disposable-profile deletion and no cleanup errors.
This changes the next implementation/qualification action; it does not qualify
production isolation or require a privileged broker. Broader network, module/
toolchain, child-tree, path-alias and crash-cleanup controls remain open.

The first command-supervision transport brick is implemented but unadvertised.
`CommandSessions` owns run-scoped handles and native process trees, paged combined
output, reviewed input/EOF, interrupt and close. It reuses the existing literal
command validation and executable identity checks. Host/task admission counts
remain held until cleanup is confirmed, including across a later run for the
same task. Finish closes the owner permanently; previously prepared starts cannot
dispatch after Finish or Stop. Read-only host inspection still exposes retained
output after Stop. It does not expose human terminal tabs.

The initial 21 command-crate cases passed, including seven new
session journeys: Unicode input and stale-review rejection; flooding/deadline and
explicit recovery; blocked stdin with responsive interruption; run finish/Stop
and descendant cleanup; task/host admission and closed-handle recovery; unavailable
isolation/tampering/stale-folder refusal; escaped-output byte paging within the
tool-result bound. The five focused core permission cases pass; stdin requires
fresh exact review even under Full access. Clippy for both changed crates across
all targets passes with warnings denied. These are native Windows/Node transport
fixtures, not a rendered desktop or provider journey. No normal app profile or
provider credentials were accessed.

The initial checked-cleanup implementation incorrectly reported lost sessions:
Windows job accounting reached zero before the root process handle signaled.
Requiring both observations inside the same bounded wait fixes the regression.
Failure remains explicit: an unconfirmed tree retains its host slot until app
restart; a still-pending I/O thread holds its slot until it actually exits.
Bridge registration, durable receipts, UI and the normal build were pending at
that transport checkpoint and are covered by the newer integration evidence
above. Broader crash/reopen/isolation qualification still remains pending.

A subsequent resource-compatibility probe changes the isolation direction. LPAC
with only the documented `registryRead` capability initializes Winsock and returns
an actual connect error 10013; the unrestricted listener control succeeds.
The paired test runs ordinary AppContainer and LPAC with identical registryRead
grants and no network capabilities. An ALL_APPLICATION_PACKAGES-only public
canary is readable in ordinary AppContainer and denied with error 5 under LPAC;
ordinary AppContainer connects to loopback, while LPAC is denied. Both variants
retain granted workspace access, deny sibling read/write, preserve outside content
and clean up the profile/owned grants. The paired probe exits zero. Its receipt is
`output/eh05-isolation/paired-f7417130998c448682fe9062579e9e26/paired-receipt.json`.
The earlier registryRead trial remains at
`output/eh05-isolation/lpac-registry-5894d9879bd345548dad06764528b136/receipt.json`;
it exits nonzero because its token query was unsupported, not because connection
denial failed. Query error 87 remains recorded as unknown, never changed to true.
The new paired evidence verifies the documented opt-out behavior separately.

This selects LPAC+registryRead for further qualification before a privileged
broker. It proves TCP IPv4 loopback denial for the static fixture only. IPv6,
UDP/inbound/LAN/Internet policy, Node/Python/build-tool runtime grants, stdio,
descendants, reparse/hard-link paths and crash-safe ACL cleanup remain open.
No production isolation mode is advertised or enabled by this diagnostic pass.
The session's pretty-printed provider-argument case also passes after canonical
query serialization; literal input bytes remain unchanged. Changed-crate Clippy
and Python compilation checks pass after those final edits.

In the earlier controls, ordinary AppContainer denies sibling read/write with error 5,
while allowing a real host-loopback connection. The unrestricted control reaches
the same listener. BFE/MpsSvc and firewall profiles are enabled; this observation
does not identify why the connection is allowed. LPAC without registryRead accepts the
opt-out attribute, but its LPAC token query returns error 87. Its Rust Winsock
initializer panics with error 10107 before a connect result. The diagnostic child
now catches and records initializer panics, preserving file results and explicitly
reporting `networkAttemptCompleted=false` with `networkError=null`. A setup or
runtime failure is never counted as denied network access. Both final probe
invocations exit nonzero, as required for these rejected candidates.

Final retained diagnostic receipts:
`output/eh05-isolation/appcontainer-a752d3a3bdd146c892e6bbc96fbafe3a/receipt.json`
and `output/eh05-isolation/lpac-615c2e48e49d4f71bf7e9ef67ea210bf/receipt.json`.
Both preserve outside content, remove disposable profiles and owned ACL grants,
and report no cleanup errors. Earlier failed query/panic trials remain separate.
No firewall policy, exemptions, feature installation or user data was changed.

[Research](design/eh05-isolation-research.md) records supported API directions and
their prerequisites. A WFP alternative would need provisioned rights and a
verified rule lifetime through crashes. Windows Sandbox is absent on this Home
edition. Real command-session integration, stdin review, receipts, Stop/crash
recovery and filesystem/network isolation qualification remain open. This is
feasibility evidence, not completed EH05 acceptance or a release-isolation claim.

## Chat input, model removal and automatic compaction repairs — October 21–22

An IME composition update could be ignored when focus moved and the committed
text did not change. The canonical composer then retained a composing range and
disabled Send. Regressions reproduce that stale state both after ordinary focus
loss and while an inspector locks the composer. Forwarding the owning block's
unchanged-text commit now enables Send and sends the exact retained text. A
separate inactive block cannot clear another block's composition. Another case
completes a reply, waits three seconds and sends another message. Active
composition still blocks Send. These are synthetic Flutter input events, not
physical Windows IME qualification or proof that every intermittent input issue
has the same cause.

Settings → Models offers Remove model in each row's options menu. Removing the
active model selects the first remaining model; removing the final model returns
to connection setup. The configuration transaction prunes removed model overrides
while retaining conversations and the provider's secure credential. Restart and
re-adding a model reuse that connection. Failed storage writes retain the prior
provider, choices and credential; retry explicitly recovers. Stale or missing
model targets are refused. Compact light/dark widget journeys verify the same
failure/recovery and preserve an unsent draft.

Automatic compaction is always enabled for new chats, forks and legacy opt-out
rows. The Summary switch is removed; manual review and correction remain.
Existing bounded summary/model-call/time checks still apply. Invalid summaries
keep the last valid summary, history and draft without automatic retries. An
oversized single instruction or insufficient remaining capacity can still require
shortening input, manual summary recovery or a larger configured model window.
The historical SQLite flag is retained for schema compatibility and no longer
disables protection; the legacy disable command refuses with a specific message.

Verification: 50 focused Flutter cases, 96 core and 79 storage tests pass. All
197 bridge cases pass across the broad run and isolated follow-ups, with one
existing ignored test. Three earlier fixtures assumed opt-in compaction or
unconfigured injected providers; they now exercise the always-on contract. The
subsequent broad run hit native-child cancellation and PowerShell terminal
assertions during concurrent release compilation; both pass unchanged in isolated
follow-ups. This is not a claim of a clean full parallel bridge run. Clippy for
the three changed Rust crates across all targets and changed-file Dart analysis
pass. The normal Windows x64 build passes through the supported CMake fallback.

All seven connected normal-bridge/rendered journeys pass. The new journey taps
Send twice, including an IME commit during an inspector lock, checks the always-on
summary explanation, removes the active/final models and verifies retained history,
an unsent draft and recovery of the empty model configuration. Four saved renders
were inspected at desktop/compact widths. The new menu, summary explanation and
Send state are visible; test-font fallback still obscures some unchanged labels
and Chinese glyphs, so these images do not qualify native typography. An initial
fixture timeout came from screenshot capture outside Flutter's real-async boundary;
moving capture into that boundary restores subsequent taps/pumps. Failed trials
remain under ignored output directories.

The native compaction fixture tests an old opt-out row, one bounded summary,
recorded usage, malformed output with preserved summary/draft, and explicit resend.
Its 4K case saves a valid summary but refuses a still-overfull request without
dropping more history. Explicitly choosing an 8K window recovers using that saved
summary. The positive fixture uses 8K because current host/tool definitions leave
insufficient capacity for a full source batch at 4K; production defaults and
recovery bounds are unchanged. Summary requests remain tool-free, while ordinary
side-chat requests retain their existing question/scheduling tools.

Final native evidence: `output/native-ui/f57e089a0cbf4aa289001443b218e254/` and
`output/managed-context-kqr4vo1f/receipt.json`. No live model or normal-profile
data/credential was accessed; provider traffic stayed on loopback. Physical Windows pointer/IME behavior,
real-model summary quality and the later harness batches remain open.

## EH04 planning and context references — October 21

`/plan` opens a dedicated inspection mode and a revisioned plan/decisions form.
Planning filters both advertised and executable tools to local read/list/search
and human questions, excluding generic commands, writes and external tools.
Normal unpaused replies become draft plans under checked revisions; a failed or
stale save preserves the prior plan and saved reply. Save and Accept do not
dispatch work. Explicit accepted-revision execution uses existing Start/StartGoal
admission, normal permissions and the unchanged goal/checklist binding.

The `@` picker previews scoped files, one directory page, explicitly attached
code/terminal excerpts, messages and conversations. Four historical reference
snapshots permit 16 KiB each and 24 KiB total. Directory contents are not expanded;
cross-chat references require the same folder and latest 80 messages, excluding
attachments. Preview-to-selection checks refuse changed/deleted sources. Saved
snapshots deliberately survive later source changes/deletion, keeping exactly the
selected text and provenance. Equal text from different sources has distinct
identities. Normal/queued/steered requests retain the selected snapshots, labeled
as untrusted data rather than instructions or permissions. Context preview uses
the same adapter and existing public/model budget checks.

Additive migration 42 preserves version 41 history and goals and adds planning
state plus immutable accepted-plan versions. Run journals and input receipts
retain their planning/context snapshots. JSON/Markdown export includes current
state and accepted versions; deleting a chat cascades those records. Reopen
restores plans without dispatch. Existing queued text receipts default to no
planning selections.

Five new backend cases exercise stale/deleted preview sources, retained queued
snapshots after chip removal, folder omissions (including an empty folder), scope
and traversal refusal, every excerpt category, over-budget context without
dispatch, stale execution, goal/checklist mismatch, failed SQLite plan saves,
Unicode migration/reopen, accepted-version export and deletion cascade. An empty
directory preview exposed an existing eager cursor subtraction; lazy evaluation
now prevents that panic. The full core/storage/bridge/filesystem library run
passes **407 tests**, with one existing ignored test. The five focused backend
cases pass again after the source-identity fix; Clippy passes for all four crates
across all targets.

Six new Flutter cases cover `@` boundaries/IME, real widget keyboard selection
through preview/sharing without send, failed saves, acceptance/reopen without
execution, stale source recovery with retained draft, and compact light/dark plan
review. Together with the prior 68 focused cases, **74 tests pass**. Changed-file
analysis is clean.
Both compact plan renders were inspected with readable content and reachable,
wrapping actions. These do not qualify native pointer or IME behavior.

The first connected trial retained useful evidence but failed two checks. Its
command attempt was correctly refused and ended planning, while the fixture
incorrectly expected automatic recovery. The revised journey explicitly inspects
that refusal, resumes retained queue state and retries inspection-only planning.
A collapsed instruction-receipt expansion also lacked its own Material ancestor;
the wrapper now preserves visible ink and avoids the framework assertion.
Evidence remains under `output/harness/eh04-native-trial/`. The next fresh run,
`eh04-native-confirmation`, passes all six journeys, including context sharing,
refusal/recovery, plan acceptance bound to a goal, explicit execution, one
artifact and evidence-linked completion over two goal segments. It uses nine
public provider requests: three planning attempts and six cumulative goal calls.

Final confirmation passes: the normal Windows x64 build, all six connected
journeys against its bridge, source-identity checks, lint, document links, batch
ownership, whitespace and tracked-file publication checks. Final receipts:
`output/harness/eh04-native-final/{receipt,schedule-receipt,stop-receipt,queue-receipt,goal-receipt,plan-receipt}.json`.
EH04 is complete within its declared scope. Fixtures use isolated output profiles and public loopback responses,
without credentials or real models. Real-model plan quality, sustained resource
use, native keyboard/pointer/IME and EH05 isolation remain separate qualification
gates. H21 remote supervision remains deferred.

## EH03 goals and context recovery — October 21

Explicit `/goal` controls save human objectives, criteria and constraints without
starting work. Additive SQLite migration 41 retains revisioned checklists, run
ancestry, lifecycle and cumulative model/tool/segment/active-time accounting.
Completed and cleared goals remain exportable after saving a new goal. Model
progress cannot rewrite criteria; completion requires retained successful tool
receipts and a normally saved final response. Receipt provenance does not prove
that a model interpreted the result correctly.

Goal segments use the normal admission owner and yield to drafts, attachments,
queued instructions and decisions. Stop, failed/truncated responses and reopening
retain work for explicit recovery. Resume preserves counters. Two segments with
no new work/verification receipts block continuation. Child and compaction model
attempts share the persistent ledger; failed ledger writes refuse dispatch.
Blocked verification reports its actual cause rather than a budget error.

Storage checks exercise cumulative exhaustion across three segments, active-time
reservations, stale revisions, failed saves, no-progress blocking, pending-input
priority, interruption/reopen, missing/failed/foreign completion evidence, export
of archived goals, deletion cascade and migration from version 40 with Unicode
history retained. Parallel child reservations preserve the parent's report slot,
including after earlier segments spent part of the cumulative model allowance;
an injected SQLite write failure refuses a free model attempt. A core regression
accepts bounded goal progress through 8 KiB and rejects oversized progress while
retaining the smaller envelope for unrelated tools.

Context recovery checks cover the tighter public-byte/model-input ceiling,
literal original input and steering retained separately from a strict untrusted
summary, host goal/constraints preserved, and no executable historical calls.
Valid recovery spends one summary attempt before the continuing request;
malformed recovery spends that attempt and stops before another task request.
Calm notices distinguish estimated context, response output and task allowance
and explicitly preserve verification. Recovery has a two-attempt segment cap;
images and checkpoints without useful headroom require explicit inspection.

Five new Flutter cases cover failed saves with retained form/composer edits,
Save without execution (including later automatic-dispatch checks), explicit
Start before continuation, input priority, resume with retained counters,
and compact light/dark layouts. With existing composer/queue/Stop/Skills cases,
**68 checks pass**. Changed-file analysis passes; both saved compact goal renders
were inspected with readable actions. These are rendered widget checks, not
native keyboard/pointer/IME qualification.

The first connected trial passed the four existing journeys and retained the
created goal artifact, but failed its progress submission at the core's smaller
query envelope. The goal paused with five model attempts, two segments and its
file/receipts retained; it was not completed. Evidence remains under
`output/harness/eh03-native-trial/`. Core, registry and tool preparation now agree
on the bounded goal-progress envelope. The full-suite run also exposed a fixed
80 ms assumption in an existing child Stop test; it now waits, with a deadline,
for both requests to be in flight before testing the same interruption/evidence
assertions.

Final confirmation passes: **364 Rust library tests**, with one existing ignored
test, Clippy across all targets, the normal Windows x64 build and **all five
connected journeys**. The goal journey creates one Unicode artifact, reads it
back and completes the evidence-linked checklist across two segments and six
model attempts under one cumulative budget. Final receipts:
`output/harness/eh03-native-final/{receipt,schedule-receipt,stop-receipt,queue-receipt,goal-receipt}.json`.
Document links, batch ownership, whitespace and tracked-file publication checks
pass. EH03 is complete within its declared scope. Fixtures use fresh
isolated data, public loopback responses and no provider credentials. Real-model
reliability, premature stopping under context notices, threshold tuning, native
input behavior and the broader EH10 resource/isolation gates remain unqualified.
H21 remote supervision remains deferred.

## EH02 command and skill discovery — October 21

The composer discovers fixed host commands and effective enabled reviewed skills
at a plain-line `/` boundary. Commands and skills keep distinct identities, even
when names collide. Goal and Plan stay visibly unavailable. Selected chips show
scope/version, expose the reviewed instructions, persist per chat and can be
removed. SQLite migration 40 adds selections; old input JSON defaults to no
selections. Queue/Steer snapshots references in the enqueue transaction, with
fresh exact-identity validation before delivery. Preview and normal requests use
the same resolver; scheduled occurrences keep their existing pinned skill.

Core/storage library suites pass **166 tests**, including exact unrelated-prompt
selection, disabled/stale/overridden refusal, version 39 migration with Unicode
history retained, reopen/queue snapshot independence, failed selection saves,
stale enqueue with retained draft, explicit stale removal and deletion. The full
bridge suite passes **187 tests** with one existing ignored test. Its four steering
boundaries now assert exact selected skill text reaches the next request, original
input remains and undispatched effects are prevented. A registry/preview check
covers host/skill name collisions and changed skill recovery. Clippy passes for
core, storage and bridge across all targets.

Ten new Flutter cases cover filtering/selection/inspection/removal, failed save
and retry without sending, unavailable command collisions, Escape and simulated
IME, compact light/dark layouts, first-message project scope, explicit refresh
after catalogue failure, and `/stop` retaining interrupted work without another
request. Together with existing composer, queue, Stop and Skills cases,
**63 checks pass**. Both saved compact renders were inspected: skill chips and the
picker are readable, and Queue/Steer/Stop remain reachable. Changed-file analysis
passes. Native keyboard/pointer/IME behavior and live-model reliability remain
unqualified by these fixture tests.

The final normal Windows x64 build and all four connected journeys pass. The
extended journey opens a new project chat, chooses its reviewed skill with `/`
before the first message, checks the exact snapshot in preview and all five
normal/queued/steered provider requests, and retains the completed file and newer
unsent draft while preventing the superseded file effect. Its receipt includes
scope/version references for each submitted input. Evidence:
`output/harness/eh02-native-confirmed/{receipt,schedule-receipt,stop-receipt,queue-receipt}.json`.
The first connected trial (`eh02-native-final`) passed three journeys but pressed
Enter before the refreshed picker was ready; no selection was saved. The fixture
now waits for its rendered loading indicator to finish before keyboard selection.
The passing run uses a fresh profile and retains the failed trial evidence.

Document links, feature ownership, whitespace and tracked-file publication checks
pass. EH02 is complete within its declared scope. These are public deterministic
fixtures against the normal bridge, with no credentials or real model requests;
native pointer/IME, live-model reliability and the separate EH10 qualification
gates remain open. H21 remote supervision remains deferred.

## EH01 live instruction control — October 21

EH01 adds a text instruction queue and provider-independent steering for ordinary
foreground chats. Additive SQLite migration 39 keeps pending input, revisions,
delivery states and run references. Queued execution uses the existing primary
owner and admission controller, preserves newer draft text/attachments, and does
not create a closed-app dispatcher. Stop, failed execution, resource pauses and
reopen retain input for explicit inspection/resume. Delivering becomes Uncertain
after restart; acknowledgement preserves that receipt instead of replaying it.
JSON/Markdown export includes all input receipts.

Storage checks cover accepted-input idempotency, reorder/stale edits, failed save
with retained draft, restart uncertainty/no replay, migration from version 38,
queue capacity, explicit uncertainty acknowledgement, export and deletion cascade.
The core/storage suites passed **174 checks**. Bridge boundary checks cover steering
during generation, review, final permission recheck and tool completion: original
input remains, undispatched operations are superseded and completed effects stay.
A separate failed-suspension check proves Stop still cancels execution.

Six new Flutter checks cover failed enqueue/draft retention, Stop/pending input,
reopen without dispatch, preservation of newer text/attachment references, compact
light/dark controls, and failed/stale editing with explicit comparison. Together
with composer/history/Stop regression cases, **46 checks passed**. Changed-file
analysis passed. The retained light/dark renders were inspected with readable
system-font action labels; initial Ahem-font captures were replaced. These are
widget renders, not native keyboard or pointer qualification.

The first connected trial found an observer timing flaw: fixtures waited on the
controller's approval value before the review control was rendered. Poll updates
now notify the main UI before fetching queue receipts, and the fixtures wait for
the actual rendered approval control. A fresh trial passed four connected cases,
including the new queue journey: two edited/reordered follow-ups, safe-boundary
steering, five public local provider requests, the completed file retained,
`superseded.txt` never created and a newer draft neither sent nor consumed.
Trial evidence: `output/harness/eh01-native-trial-02/queue-receipt.json`.

Final confirmation passed: the normal Windows x64 build and all four connected
journeys passed against its bridge, with five requests in the queue journey and
the retained/superseded/draft assertions above. Evidence:
`output/harness/eh01-native-final/{receipt,schedule-receipt,stop-receipt,queue-receipt}.json`.
The final full bridge suite passed **186 tests**, with one existing ignored test;
Clippy passed for core, storage and bridge. The six queue/editor tests and changed
analysis passed again after the draft-debounce preservation adjustment. Document
links, feature ownership and publication checks passed. EH01 is complete within
the declared ordinary-text-chat scope.
No provider credentials or real models were used. Queued attachments and steering
of scheduled/computer-use runs are explicitly unavailable with draft retention.
Actual native IME/pointer behavior, live-model reliability and sustained resource
qualification remain open; the earlier audit and H17 isolation gaps are unchanged.

## EH00 harness foundations — October 21

The user authorized H01–H20 implementation in batches, with a commit after each
batch. [Eleven task lists](plans/everyday-harness/README.md) preserve feature
ownership, dependencies and failure/recovery exits. H21 remote supervision remains
deferred. EH00 establishes contracts and feasibility evidence; no queue, steering,
goal, plan or isolation control is exposed by this batch.

The [frozen contracts](design/everyday-harness-contracts.md) define records,
additive migration ownership, checked revisions, deletion/export, safe-boundary
input, Stop/reopen recovery, UI/IME rules and bounded resource ceilings. Three new
core checks cover eleven dispatch cases, crash uncertainty/no replay and stale or
terminal receipt refusal. The full core suite passed **91 tests**; three existing
run-admission and two persistent-draft checks passed. Fourteen command tests passed,
including real Unicode stdin/retained receipt/Stop and existing timeout, flooding
and descendant cleanup. Core/command Clippy and formatting passed.

Forty-nine focused Flutter checks passed for composer, history, settings,
zero idle animation and Stop recovery. Four saved compact/wide light/dark
design sketches were inspected; they do not establish implemented Flutter UI or
native IME behavior. A normal Windows x64 build passed. Three rendered native-bridge
cases then passed with fresh public fixture data, including active delivery,
Stop after a completed file effect and protected recovery draft:
`output/harness/eh00-native/{receipt,schedule-receipt,stop-receipt}.json`.
These establish the existing baseline, not EH01 integration or model reliability.

An isolated normal app opened a visible window. A 10.04-second idle sample recorded
0.078 CPU seconds, 240,005,120-byte working-set peak and 248,573,952-byte private
peak, with no desktop helpers. This is an uncontrolled-focus reference sample,
not a startup, active-resource or low-end release qualification. Receipt:
`output/harness/eh00-baseline/resources.json`. Only the recorded preview was closed.

The bounded AppContainer probe **failed the required network boundary**. A verified
container token could read/write its granted folder and was denied sibling-file
read/write (error 5), but loopback TCP succeeded with no network capabilities.
The unrestricted control succeeded as expected. The sibling file remained and
profile/owned ACL grants were removed. Initial low-integrity-label setup failed
with access denied; that failed attempt is retained separately. Receipts:
`output/harness/eh00-isolation-01/receipt.json` and
`output/harness/eh00-isolation-03/receipt.json`. This is a rejected complete backend,
not a passed isolation test. EH05 must prove network enforcement or revise its
backend; requested isolation must fail closed until then. No machine policy,
firewall, loopback exemption, installation or elevation was changed.

Document/link and batch coverage checks passed: H01–H20 owned once, H21 excluded,
eleven ordered task files. Existing October audit deferrals remain unchanged.
Native keyboard/IME, measured startup/active resources, real-model reliability,
toolchain/container compatibility and full isolation remain explicit later
qualification gates.

## October 21 priority audit closure

At the user's request, this audit ends after the two remaining priority cases:
**98/129 checked, 31 deferred**, listed in the
[checklist](qualification/first-user-audit.md). Deferred items are not passes;
native terminal/privacy/window controls and experimental feature qualification
remain explicit gaps for later changes/releases.

M01 passed actual rendered Memory/composer controls and the normal native bridge
with `qwen3.7-flash`, two public completed chats and automatic fact/preference
capture, activity and exact sources. A first trial saved acknowledgement/no-tools
instructions as part of the lasting preference. A failing regression reproduced
it. The host now rejects multi-sentence automatic quotes without truncating
potential qualifiers; the prompt clarifies exclusion of one-off instructions.
English/Chinese examples, inline qualifiers and version numbers pass. After a
normal rebuild, the live rerun saved exactly the two durable sentences. The two
extractions reported 904 total tokens; foreground usage was unavailable. Four
messages and the newer draft remained. Receipt:
`output/native-audit-20261010-01/memory-live-fixed-20261021/memory-live-receipt.json`.

A11 passed rendered Create once → completed file receipt → stalled follow-up →
Stop → Earlier tasks → Prepare resume draft. Exact UTF-8 bytes and the successful
receipt remained. The cancelled checkpoint truthfully had no uncertain effects;
its recovery draft warned against repeating applied operations. Preparing recovery
protected a newer saved draft and sent no new request. Two local requests, zero
replays. This is not scheduled-occurrence S10 or native pointer qualification.
Receipt: `output/native-audit-20261010-01/stop-ui-fixed-20261021/stop-receipt.json`.

Ten core memory and two automatic-store checks, changed-file analysis, the normal
Windows build and all three connected UI cases passed. The initial Stop fixture
needed explicit UTF-8 HTTP encoding; this harness failure preceded the passing
run and was not a product defect. Original profile: all 49 table digests unchanged.
The checklist/notes accompany code and test changes, without a documentation-only
commit. No blanket claim about all models or languages is made.

## October 21 scheduled deletion and failed-occurrence retry

S12 passed a rendered Scheduled page connected to the normal Rust bridge. The
public loopback provider created a task through `schedule_task`, failed an
occurrence with 503 and completed a new occurrence only after explicit Run now.
Both history entries remained inspectable. Keep task cancelled deletion;
confirmed Delete retained exact source/result messages and public UTF-8 project
bytes. A subsequent tick claimed nothing. Four local requests; zero external
provider requests. The regression runner now executes the local and scheduled
UI cases serially and requires both receipts. Both cases and changed-file
analysis passed. Receipt:
`output/native-audit-20261010-01/local-ui-schedule-20261021/schedule-receipt.json`.
S10 remains open for interruption after completed effects; this no-effect failure
does not prove that case. Checklist: **96/129 checked, 33 pending**.

## October 21 disposable conversation deletion

C16 passed the rendered Home/sidebar and normal Rust bridge: Keep preserved the
selected chat and exact messages; confirmed Delete removed only that chat.
Another conversation and its draft remained, and the public UTF-8 project file
was byte-identical. The opt-in native UI runner passed with fresh output data;
receipt `output/native-audit-20261010-01/local-ui-delete-20261021/receipt.json`.
No native pointer or new live-model pass is claimed. Checklist:
**95/129 checked, 34 pending**. Notes accompany the executable regression change.

## October 17 controls connected to the normal Rust bridge

M07 passed rendered Flutter instruction controls using the actual `NativeBridge`
and normal Rust DLL. A source changed between review and Enable was refused;
Refresh and a new Enable succeeded. Changed/missing enabled sources blocked
context preparation, offered review/disable, and Disable recovered without losing
the draft. The literal reference in AGENTS.md did not load its referenced file.

M04 passed the same connected UI: Memory Off retained an inspectable saved record
but excluded it from context. One completed local fixture chat made exactly one
request, without automatic capture. Explicit Forget removed only that memory;
two saved messages and the newer draft remained identical. The memory was seeded
manually for this test; automatic live capture is a separate requirement.

`python scripts/qualify-native-ui.py` passed with fresh output data; receipt:
`output/native-audit-20261010-01/local-ui-qualification-03-20261017/receipt.json`.
The runner refuses an existing or outside-output directory, preserves evidence,
uses no external provider/credentials and bounds its owned test process. Analysis
passed; the ordinary widget invocation correctly skipped the opt-in case. Initial
harness failures were isolated to test async-zone/error handling and were fixed
before qualification. This is headless rendered interaction with the real host,
not a native pointer/IME/dialog pass. Checklist: **94/129 checked, 35 pending**.

## October 17 companionship feedback and experimental preferences

E02 passed compact light/dark rendered controls for Open and all feedback actions,
including failed-save retention, no silent retry and explicit recovery. Ten
public seeded-note cases against the normal native bridge verified seen state,
24-hour snooze, mute, retained draft, unchanged consumed attempts and missing-note
refusal. “Fewer messages” raised an existing Quiet (0) limit to 1. The regression
failed in both Rust and the packaged host; the fix caps at 1 without raising 0.
Feedback state/history/drafts survived database reopen. Evidence:
`output/native-audit-20261010-01/companion-feedback-green-20261017/receipt.json`.

P14 passed rendered concise availability/help and failed-save/reopen checks.
The normal bridge saved both experimental choices in disposable data; a separate
process restart retained them, restored a real Windows process-owned power
request, refused stale writes and released the request on disable/shutdown.
This does not claim bypass of manual locks or security policy. Native receipts:
`output/native-audit-20261010-01/experimental-host-20261017/`.

Four companionship Rust tests and three experimental Rust tests passed; the
credential-dependent test stayed ignored in the deterministic run. Twenty-four
relevant Flutter checks, changed-test analysis and the normal Windows build
passed. Separately, fresh qwen3.7-flash probes with explicit thinking off passed
chat and public-memory invitations: two requests, zero tools, 256 reported total
tokens. Original profile table digests remain unchanged. Native idle eligibility
and in-app delivery remain open under E01; collector success is not delivery.
Checklist: **92/129 checked, 37 pending**.

## October 17 rendered comparison and answer-retry audit

E03 passed seven public deterministic receipts reopened byte-for-byte through the
normal Rust bridge. Rendered Flutter controls in compact light/dark views expose
exact frozen requests/checks, every retained result, elapsed time and unknown
usage without inventing usage or activating instructions. Tie, failed, truncated,
stopped and interrupted receipts remain non-improvements; viewing starts no run.
Use as draft incorrectly kept the current defaults (1,024 tokens/20 seconds)
instead of the saved 512/5. A failing-first regression reproduced this; copying
the saved settings now preserves the comparison allowances as well as its cases.

C17's remaining failed-send path passed rendered Flutter interactions in both
compact themes: retained choice/custom text, Back/Next, explicit Send answer
retry, no automatic retry, no tool grant and visible completion. This failure
injection is deterministic; the October 10 native Qwen round trip remains the
live evidence. Saved renders were inspected; the question render has test-font
fallback gaps and is not evidence of native Chinese font rendering. No new
desktop or live-model interaction is claimed. All 14 focused tests, analysis of
the three changed Dart files and the normal Windows build passed. The build
handle expired after completion; its normal main-entry identity and matching
executable size/time were revalidated. Checklist: **90/129 checked, 39 pending**.

## October 16 native audit: scope inheritance and explicit continuation

P04 passed native edits of all-chat/project/chat call limits, persistence and reset.
Saved limits were 4 for one Project A chat, 2 for its sibling and 3 for Project B.
The four-call chat completed a three-call task. Native Use inherited settings
restored 2/2/3; the sibling's new run paused at two. A10 passed native Continue and
fresh read approval, with completed evidence retained and incomplete work not run
at the limit. The initial local fixture miscounted retained versus current tool
results and caused an extra pause. Its same endpoint was corrected; the final
segment completed from retained evidence without enlarging runtime limits. Each
read had its own approval. These are deterministic native host/UI passes, not
claims of model competence. On October 13, the configured Qwen request was refused
with “Model access denied”; its exact draft was restored with Model settings
recovery. Public receipt: `output/native-audit-20261010-01/p04-a10-native-receipt.json`.

The native save of a one-call limit was correctly refused by the host but exposed
absent input guidance and a dense combined error. Numeric ranges are now shown
beside task-limit inputs. Out-of-range calls/tools/segments/time are refused
locally with a short field-specific message; edited and previously saved values
remain intact. The reproducing regression failed before the fix. Host validation
remains authoritative and unchanged. All 14 focused settings widget tests, Flutter
analysis of lib/test and the normal Windows desktop build passed. Checklist:
**88/129 checked, 41 pending**.

## October 13 installer fixture cleanup race

The screenshot reports `WinError 2` launching `unins000.exe` in the locked-data
case, after a successful retention uninstall and reinstall. The baseline
full-payload lifecycle passes locally, but its logs show the next setup starts
while the previous uninstall is still finishing. Inno 6.7.3 terminates its first
process, waits for it, then delays deletion of the original uninstaller in its
temporary second process. The fixture waited only for `dolores_flutter.exe` to
disappear and assumed the next uninstaller name was always `unins000.exe`.

An owned diagnostic small-payload fixture reproduces the exact Windows error:
rapid reinstall creates both `unins000.exe` and `unins001.exe`; actual late
cleanup removes the first, leaving the registered second executable. The old
test then raises `FileNotFoundError` in the locked uninstall call. Evidence:
`output/ci-installer-investigation-20261013/exact-before.log`. A deterministic
regression also fails when app removal precedes uninstaller cleanup.

The fixture now waits, bounded to ten seconds, for the entire owned install
directory and requested data purge to disappear. It resolves the executable
from its unique current-user uninstall registration, checks install-directory
ownership/name/paired log, and requires the exact expected exit code instead of
accepting any nonzero error as a safety refusal. No automatic install/uninstall
retry is performed. Both local release and hosted release qualification add a
rapid-reinstall fixture. Workflow logs/results are retained even on failure;
unqualified product installers are not uploaded or published. Installer code,
installation scopes, user-data options and production cleanup behavior are
unchanged.

Four lifecycle tests pass: late uninstaller cleanup blocks reinstallation;
numbered registration works; external/incomplete registrations refuse; unrelated
exit codes and incomplete cleanup refuse. Rapid native qualification passes
retention, locked-data and junction refusal, and explicit synthetic purge recovery.
The full-payload native lifecycle also passes. Three additional rapid native
repetitions on two CPUs pass all five safety/recovery cases without retries.
Results: `output/ci-installer-investigation-20261013/full-after/RESULT.json` and
`output/ci-installer-investigation-20261013/rapid-repeat-result.json`. Installer,
local-runner, CI-contract and lifecycle unit checks pass; both workflow files
pass actionlint 1.7.7. This change affects the verification harness/workflow;
unrelated Rust/Flutter suites and desktop builds were not rerun. These
fixtures use their own random app ID/shortcut, ignored installation directory
and public synthetic data. Signing, clean-machine, elevated all-users and
physical production-checkbox qualification remain separate. No release/tag
mutation is performed.

## October 13 language CI observer deadline

The reported `fixture request hung` assertion belongs to the test's ten-second
observer, not the language service's timeout. The service permits 15 seconds
for initialization, followed by 15 seconds for a feature request. The unchanged
fixture passes in isolation in 0.30 seconds. An owned fixture delayed its
initialize response by 11 seconds; the original test reproduced the exact panic
twice after 10.04 seconds, before the service allowance expired. The hosted log
does not say which of completion/diagnostics/hover was pending, so its exact
phase and scheduling delay remain unobserved.

The observer now derives its bound from the unchanged initialization/request
limits plus a five-second test scheduling margin, returning the real terminal
result first. A permanent regression delays both initialization and completion
by 11 seconds, exercises unsaved Unicode text and diagnostics arriving after
the document-symbol response, then injects malformed output. Stop clears retained
jobs/process ownership, and a fresh server explicitly recovers the updated
buffer without saving it to disk. A second edge leaves completion unanswered,
asserts the exact 15-second timeout and actionable recovery, verifies no retry,
then explicitly restarts and checks retained Unicode text. No production limit
was increased. The previous local pass and stream/MCP repeat gates did not cover
this slow LSP path; both CI entry points now repeat all four language cases.

All four focused cases pass. Six repetitions in two concurrent processes on two
CPUs pass all 24 cases in 70.99 seconds, with no retries. The complete shared
local Checks pipeline also passes under two CPUs: formatting/Clippy, the Rust
workspace (182 bridge tests), 60 stream cases, 24 MCP recovery cases, four real
MCP protocol repetitions, eight language cases, script/history-audit/browser
fixtures, Flutter analysis and all 446 Flutter tests. Evidence:
`output/local-ci/language-fix-20261013/RESULT.json`. The opt-in native-vault tests
remain ignored. The normal Windows build passes in 169.105 seconds through
`scripts/desktop.py build`; its receipt is
`output/ci-language-investigation-20261013/normal-build-result.json`.
Diagnostic logs stay under `output/ci-language-investigation-20261013`; local-host checks do not
establish the hosted Windows Server image, native UI interaction or real-model
reliability. No push, tag rewrite or publication was performed.

## October 13 MCP CI recovery and local release qualification

Both reported bridge failures occur in initial Node inspection, before the
credential/review assertions. The original isolated tests passed locally in
0.98 seconds (2.82 with software SHA-256). Restricting the harness to two CPUs,
with 16 concurrent processes running both MCP cases and software hashing,
reproduced the exact 30-second error repeatedly. Instrumented samples spent
about 4 seconds on initial hashes, reached initialization after 16–25 seconds,
and expired during final hashing. Optimizing the generic MCP digest caller
helped but did not make the extreme workload reliable; that alone was rejected
as the fix. The hosted runner's exact CPU/hash capabilities remain unverified.

The bridge tests now begin at their existing reviewed-state boundary with small
real files. They still exercise production launch fingerprint verification,
review scope/expiry/replay, credential rotation, locked Forget, failed saving,
retained review/credential state and explicit recovery. Changed-file refusal and
restored-byte recovery are added. The original two-CPU/16-process/software-hash
harness passes all 48 cases; individual processes finish within 0.48 seconds.
Real Node stdio discovery, credential transport/redaction, malformed responses,
cancellation and changed-launch refusal remain covered by the MCP crate.
Production limits, fingerprint security and application behavior are unchanged.

The previous repeat gate only exercised stream deadlines, which did not cover
this failure. Both CI workflows now repeat bridge MCP recovery and real Node
pagination/client-request protocol tests with software hashing. Workspace tests
use four threads consistently. Failures/missing cases refuse without retries.
Local runner tests verify failed-step evidence retention, no inferred tag,
refusal of reused/outside output and recovery into a new directory.

`scripts/local-ci.py` runs the shared Windows checks or full installer pipeline,
requires matching Python/Node version families and Rust/Flutter pins, and
installs the history-audit dependency in an isolated environment rather than
skipping that fixture. It
records host/tool versions, revision, dirty state and step logs. CPU affinity is
limited only for the runner and its children. The first full local run in
`output/local-ci/mcp-fix-20261013` passed the workspace Rust tests (180 bridge,
11 MCP), all three repeat gates, browser fixtures, normal build and analysis,
then refused packaging on a workspace-restoration Flutter test (445 passed,
one failed). That test used two real 120 ms sleeps while approval polling waits
200 ms. The same focused case failed locally under two CPUs and again with a
controlled clock. It now verifies navigation refusal during pending Stop,
advances the scheduled poll, checks restored draft/no approval, then verifies
folder switching and restart; the focused controlled-clock check passes.
No controller behavior or polling intervals changed. Final qualification passed
in fresh `output/local-ci/mcp-fix-20261013-r2` under two CPUs: formatting/Clippy,
the full Rust workspace, 60 stream cases, 24 MCP recovery cases and four real
Node protocol repetitions (software hashing), browser fixtures, normal release
build, Flutter analysis, all 446 Flutter tests and 16 native save/restart steps.
The final installer fixture passes current-user installation, retained data,
locked-data refusal, junction refusal and explicit synthetic purge recovery.
The runner's `RESULT.json` records all seven passed stages and zero publication.
The unsigned installer is
`pipeline/installer/Dolores-0.0.1-alpha-windows-x64-setup.exe`, 19,420,976 bytes;
its SHA-256 matches the generated sidecar:
`6496204eab44306fb5d8e9ee824b1f912195d170b8a673eb112f471a5c96e6cb`.
This is the local Windows host with compiler caches, not a recreated Windows
Server 2025 hosted image. The opt-in native-vault tests remain ignored.
No GitHub publication or tag rewrite is involved. Signing, clean-machine,
elevated all-users and physical uninstall-checkbox acceptance remain separate.

## October 12 native audit: reviewed live browser form

I04 passed with native Send and four explicit Allow once controls. DeepSeek opened
an owned local form, filled `PUBLIC-BROWSER-42`, clicked Preview with the latest
state and captured the resulting page. Five model calls completed. Exact DOM
receipts and the final answer agreed; native View local screenshot showed the
input/result value, and collapsing the capture caused no error. Read-only process
inspection found no owned child after completion. Public receipt:
`output/native-audit-20261010-01/i04-native-receipt.json`.

The browser operation receipt incorrectly called its reviewed arguments “Search”.
It now says “Operation”, retaining the full JSON and outcome. The reproducing
widget check failed before the fix; all 15 focused tool-receipt/browser checks pass.
Application/test analysis and a normal Windows build pass; the rebuilt native
receipt shows “Operation” and keeps the reviewed JSON readable.

I05 separately passed native interruption and stale-state recovery. Qwen's owned
local navigation entered a deliberately delayed HTTP handler before Stop; the
exact draft was restored and zero owned child processes remained. DeepSeek's
local page changed while Fill waited for review. The stale action dispatched
nothing; returned fresh state required a new review, then Fill/Click/Screenshot
completed. Six model calls retained refusal and recovery. Receipts:
`i05-native-stop.json` / `i05-native-stale.json` in the same evidence directory.
Original-profile verification still matches all 49 table digests. Checklist:
**86/129 checked, 43 pending**.

## October 12 native audit: skill test inputs and interruption

E04 passed with explicit evidence boundaries. A native DeepSeek draft and two
completed comparisons retained exact cases, responses and usage; ties never
enabled activation. Both live Stop attempts arrived after completion and are not
interruption passes. A local deterministic fixture delayed an entered request;
native Stop reported no saved skill, retained draft/test inputs and left activation
disabled. This edge proves native recovery, not provider competence. M10 still
needs native editing and strict-gain promotion.

Empty tests previously left Evaluate enabled and produced a dense host constraint
error. The footer now gives short input guidance and enables evaluation only when
each test has a prompt and expected text. Blank/whitespace edits invalidate prior
activation without sending a request. The new regression failed before the fix;
22 focused skill tests, analysis of application `lib`/`test` and a normal Windows
build pass. Native rebuilt verification confirmed missing-input guidance, dynamic
enablement and Stop recovery. Whole-directory analysis additionally sees an
unrelated unnecessary-import notice in an ignored old screenshot generator.
Original-profile verification matches all 49 table digests. Checklist:
**84/129 checked, 45 pending**.

## October 12 native audit resumed: Git interruption reconciliation

G20 passed natively in the isolated public local-remote fixture. Changed HEAD
while the remote picker was open refused the stale basis before transfer. A
deliberately delayed local Fetch entered its transport; native Stop reported
possible retained effects, and Refresh recovered. Read-only reconciliation found
unchanged HEAD/tracking/bare remote refs, exact public file bytes and no owned
child processes. History refresh showed the new HEAD. The temporary transport
override was removed. The prior Escape checkpoint remains historical evidence,
not a pass. Checklist: **83/129 checked, 46 pending**. Notes await the next actual
code-change commit.

## October 12 background audit: nested receipt recovery

After physical Escape stopped computer-use, the user explicitly kept desktop
interaction paused. No further native input was issued. The interrupted G20
remote-picker action has no observed outcome: the owned transport-delay marker
was absent, and the temporary local upload-pack override was removed. G20 remains
unchecked; neither a stale refusal nor a canceled transfer is inferred.

Widget interaction reproduced two further receipt failures using the real
`ToolRecords` hierarchy and saved page state. Opening native-build details raised
the same boolean/scroll-offset type error as MCP metadata. A separate text key
removed the crash but exposed shared outer/inner expansion state: details opened
without being selected. Independent keys for both text and nested expansion fix
the two symptoms. Tests cover collapse, view restoration, two reordered receipts
and a failed build retaining its actual result and reviewed commands.

A missing browser capture inside its saved receipt also raised that type error.
A separate error-text key fixes it for browser and selected-window captures.
Repeated viewing attempts issue only the matching capture-read command for the
same capture; they never repeat the original action. The recovery copy now refers
to the action generally, correctly covering both screenshot types.

Both reproductions failed before their fixes. All 37 focused Flutter checks,
all 444 Flutter tests, analysis and the normal Windows build pass. These are deterministic widget and
compilation checks; no model request or native checklist pass is added.
Checklist remains **82/129 checked**, with 47 native interaction gaps. Notes are
committed together with these code changes, as requested.

## October 12 native audit: local Git remote cycle

G19 passed native Fetch, dirty-Pull refusal, clean fast-forward Pull, file +,
Commit and Push against a local bare origin. Independent Git inspection
verified unchanged HEAD after Fetch, exact peer HEAD after Pull, the one-file
native commit and matching origin/main after Push. The public untracked file
was retained across recovery. No GitHub or user repository was modified.
The injected right-Control shortcut did not dispatch; the Commit menu did.
A stale success notice was reproduced alongside a later refusal; a failing
regression now passes, with all 42 focused tests green. Rebuilt native Fetch
then divergent Pull showed only the refusal, retained both HEADs/files and
performed no merge. G20 still needs changed-ref/interruption recovery.
P09 also passed native size/format refusals preserving the draft, and exact
manifest/snapshot export after the source changed. The attachment regression,
bridge clippy, Flutter analysis and normal Windows build passed.
M02 passed fresh native Side-chat recall of a real automatically extracted
public codename using Qwen. The native receipt showed one learned memory,
zero prior turns, source/revision and 2,741 provider-reported tokens.
Memory enablement was host-level setup; M01 switch interaction stays open.
M03 passed native correction/source/history inspection and reopen after a
later real Qwen extraction skipped the conflicting old value (host evidence).
I03 passed native absent-adapter status, setup guidance and Refresh under an
owned launch-only environment override; normal environment was restored.
S04 passed native inspection/navigation of a fresh real Qwen-created and run
pinned-skill schedule (host preparation). S11 passed native Close-to-tray,
same-owner launch reopen and ordinary Close full exit with background off,
verified before replacement. Draft, result and one occurrence survived.
Tray-menu Quit itself was not exercised. I01–I02 passed native search/page receipt and citation inspection on the normal
rebuilt desktop. The page-position label regression failed before repair;
all 29 focused tests, analysis and Windows build passed afterward. Native
inspection used the retained real Qwen run; no additional model request.
I07 passed fresh native real-Qwen exact approval/result and missing-server
reinspection recovery. Its blank-panel regression failed before repair;
26 focused checks, analysis, normal build and native retest passed. G17–G18
passed direct stash conflict retention and Revert/conflict/Abort/dirty refusal
with independently verified public bytes and refs. Native MCP activation stays
open. Checklist: 82/129 checked, 47 pending.

## October 12 native audit: truthful nonrepository state

G15 passed after fixing a misleading “No commit / detached HEAD” header for
folders without Git. The failing widget regression now passes; all 42 Source
Control tests and focused analysis pass. Repository status is required before
rendering a HEAD label; empty and detached repositories have distinct labels.
The normal Windows build passed. Native navigation to a public folder outside
every Git ancestor and Refresh both retained the correct nonrepository error
without a fabricated HEAD state. Earlier binary-diff interaction completes
G15. Checklist: 70/129 checked, 59 pending. The original profile matches all
49 table digests. No Git mutation or model request was made for this check.

## October 12 native audit: text attachment snapshot retention

P07 partially passed in a fresh isolated normal-app chat. The native picker
attached a public 108-byte UTF-8 file; preview showed its original English/code
and Chinese sentence. Changing the source afterwards did not change the preview.
Sending the physical-key prompt `Repeat` to Qwen3.7 Flash produced the original
P07-ORIGINAL-42 code and Chinese sentence in one model call. Read-only storage
verification confirmed the snapshot's exact CRLF bytes and SHA-256, distinct from
the modified source. The model paraphrased the heading; this is content retrieval
evidence, not a byte-exact model reproduction claim. The native Remove control
awaits action-time confirmation; P07 remains unchecked. Count remains 69/129.

## October 12 native audit: local capabilities and source inspection

E09 passed through Advanced → Diagnostics & storage → Capabilities & source.
The normal UI showed build version, workspace, selected model/context origin,
run limits, tool availability, review requirements and unknown model reliability.
Selecting the bundled file-tool component showed its source hash and 1–60 / 342
lines. Next source lines displayed 61–120; First lines returned to 1–60.
Compare checkout opened the native folder picker; Cancel retained the source.
Refresh retained the inventory and source. These local controls sent no model
request. Checklist: 69/129 checked, with all 60 pending IDs accounted for.

## October 12 native audit: model settings survive restart

P03 passed through the normal Settings → Models UI in the owned isolated
profile. Qwen's context/output/stall changed to 1,000,001 / 1,025 / 32; DeepSeek
retained 1,000,000 / 1,024 / 30 and separately saved Provider default reasoning
instead of DeepSeek thinking off. Save succeeded for both models. Stopping and
relaunching only the owned preview through the normal desktop launcher, then
reopening both model editors, retained their exact saved values and reasoning
choices. This exercises persistence and isolation between models.

No model request was made. Edited settings remain in the disposable profile;
the original profile matches all 49 table digests. The checklist advances to
68/129 checked, with all 61 pending IDs accounted for in its current queue.

## October 12 native audit resumed: unsaved settings recovery

The user's renewed availability prompted a fresh computer-use retry. Resetting
its JavaScript session replaced a closed cached window reference with the current
owned preview. Accessibility-index input still failed, but screenshot-based
clicks and physical key presses worked. Bulk `type_text` did not change a focused
numeric field; this limitation was not treated as successful input or a verified
product defect. No alternate Windows automation was used.

P05 passed in the normal desktop UI using the isolated public-notebook profile.
Changing Qwen's stall timeout from 30 to 31, then pressing Escape, offered Save /
Discard / Keep editing. Keep editing retained 31. Closing model details offered
the same choices. Discard retained the original saved 30 on reopening; a fresh
edit saved through the Escape prompt retained 31 on reopening. Other model
values were not changed. No model request was needed. Original-profile digests
match all 49 tables. The checklist advances to 67/129, superseding the earlier
stop boundary. P03 was still open at this checkpoint and subsequently passed
as recorded above.

## October 12 audit stop checkpoint

The current checklist is 66/129 checked. Its current queue accounts for all 63
pending IDs and the native interaction still required for each group; completed
editor items were removed from the older queue. Available background paths have
been exercised without converting them into native passes. Six focused learning
checks pass on the current source, including confirmed regression/quarantine,
incomplete monitoring without retry, Stop and manual-edit/policy conflicts.
Real-model automatic regression restoration remains unqualified; a fixture pass
does not close that gap. The original profile still matches all 49 table digests.

The latest native attempt failed on cached accessibility controls and did not
establish field focus after a fresh observation. No preference was saved, and
the owned isolated preview was stopped. Under the user's stop condition, this
audit run ends at the computer-use boundary. The feature audit remains incomplete.
Resume with native P03/P05, then the current queue, including a fresh check of the
changed direct Git actions. Native install/restore and restricted permission or
terminal interactions retain their required human involvement.

## October 12 background audit: drafted skill selection and model accuracy

A public code-review exchange was drafted, edited and compared against two
response cases frozen before generation: a negative-input regression and a
zero-count regression. Required headings/risk words and forbidden false execution
claims were identical for baseline and candidate. Qwen completed the draft and
comparison (baseline 0/2, candidate 2/2), with untested activation, changed source
quotation, stale promotion and duplicate promotion correctly refused. Reopening
retained the exact evaluation. These checks establish the literal format gate,
not correctness of every model-authored statement.

The fresh-chat follow-up failed: no skill was selected, even though it was saved
and enabled. The description used “code reviews”; the request said “Review this
function” without another matching keyword. The exact-word selector omitted the
instructions. A regression failed with zero selected snapshots before repair.
Simple English singular/plural keyword pairs now match in either direction;
exact names, unrelated substring exclusion, scope precedence and existing limits
remain. All 94 core checks, Clippy and the normal Windows build pass. The identical
fresh-chat request on the rebuilt bridge includes the exact saved skill version
and returns the requested sections; original source chat/evaluation stay intact.

Manual content inspection caught an additional model failure: Qwen's original
source review incorrectly accepted the negative-input change, and its draft
faithfully copied that bad example. The first script checked format before source
accuracy; its original receipt is retained with a separate failed quality review.
The disposable Qwen skill was disabled. DeepSeek also misjudged the original terse
request. A fresh explicit before/after Python presentation of the same change
then produced a correct DeepSeek source review without weakening the two frozen
comparison cases. Draft/edit/test/strict-gain promotion and all four refusal checks
passed. Separate-process reopening retained the exact evaluation; a new empty-list
case used the saved skill with matching provenance and correctly explained the
None-to-IndexError change without claiming execution. That reply was verbose;
this does not establish consistent brevity or general review competence.

Across these bounded probes: eight Qwen calls / 10,791 reported tokens, eight
DeepSeek calls / 10,917 tokens. Drafting allowed 4,096 output tokens/60 seconds;
comparison retained its 512-token/10-second per-response limits. Original-profile
verification matches all 49 table digests. No desktop input or original skill
collection changes were made. Receipts remain under ignored
`output/background-audit-20261011-remaining/skill-draft-{qwen,deepseek}-20261012-*`.
M10 remains pending for its native source-selection/edit/test/promotion controls;
the checklist stays at 66/129. Model failures are preserved separately from the
successful background workflow.

## October 12 background audit: real automatic skill improvement

A fresh bounded Qwen3.7 Flash run qualified the existing `config-check-v1`
workflow on the normal release bridge. The approved public task changed only
the enabled flag, preserved unrelated files and recorded the obsolete check's
failure. Reflection retained the approved package declaration and exact skill
version. Both baseline cases completed and failed the fixed check; both candidate
cases completed and passed, including the independent Unicode/nested-value case.
Dolores automatically activated the exact check-command update. The existing
criteria, 1,024-output-token trial allowance, five-call/eight-operation limits and
30-second case deadline were unchanged. This is one narrow workflow improvement,
not general skill-learning reliability.

The qualification helper initially reported failure after a successful manual
restore: it expected `restored`, while the product records `quarantined` and
appends the retained baseline as a rollback version. Its assertion now checks the
specific event, exact baseline document/enabled state, retained candidate and
rollback reference. A separate fresh-process check of the same live receipt
passes without another provider request. Wrong-document and missing-rollback
variants fail the assertion. Stale and duplicate restore requests are refused;
the actual saved state and skill remain unchanged. The original failed receipt
is retained alongside the corrected reopen evidence, rather than rewritten.

The live foreground used five calls / 34,216 reported tokens; four trial phases
used sixteen calls / 16,585 tokens. Original-profile verification matches all
49 table digests. No desktop input, installation or original-project writes
were performed. Evidence is under the ignored
`output/background-audit-20261011-remaining/learning-qwen-20261012-01/`, including
`public-result.json` and `restoration-reopen-result.json`.
E05/E06 remain pending for native controls and manual-edit conflict interaction.
Real-model automatic regression restoration and a fresh native repair-to-install
scenario remain separate gaps. No additional checklist item is checked.

The native retry recovered the isolated app's current window after an obsolete
helper window reference and inspected the model settings. The helper then rejected
a freshly observed editable-control index as unavailable in its cached state.
A refreshed screenshot/pointer retry did not establish the intended field focus;
no text was entered or settings saved. Native input stopped rather than assuming
an edit succeeded. P03/P05 remain unqualified; this does not establish a product
settings failure. No alternate native automation was used.

## October 11–12 background audit: large direct Git actions

The desktop helper failed both activation attempts with `foreground window did
not report a process id`. Selection was refreshed before retry; no subsequent
UI input or alternate automation was used. Physical Git conflict/direct-action
acceptance remains pending, and the checklist stays at 66/129.

A public 678,000-byte change failed Commit on the normal release bridge:
`Git output exceeds 512 KiB`. The internal commit preparation still captured a
full prospective diff, unused after removal of the review UI. Its output cap
blocked ordinary commits; the patch also had a separate 256 KiB limit. Both are
avoided by retaining a bounded file-count summary and the existing exact paths,
saved revision, author, dirty-editor and hook guards. No process/output bound
was increased; no automatic retries were added. The first rebuilt bridge committed
the full file, then exposed the same obsolete-capture problem in Discard at the
256 KiB patch bound. An independent large Revert regression failed at 512 KiB.
Discard and Revert now retain bounded operation metadata; only selected hunk
actions need their actual patch. Their saved-state, tracked-path, dirty-editor,
clean-repository and conflict/Abort checks remain.

The subsequent stash-conflict probe retained both the exact stash and conflict
markers, but exposed an unhelpful `Index was not unstashed` error. A strengthened
regression failed on that wording. Confirmed stash conflicts now explain retention,
Refresh and resolution before retry; confirmed revert conflicts explain Refresh
and Abort revert. These messages use the post-action repository state and do not
claim that failure left saved files unchanged.

The new large-commit regression failed before this repair and passes afterwards.
It verifies automatic commit by exact Git blob hash, staged-selection precedence,
later saved working content and unrelated untracked retention. Separate large
Discard and Revert tests also failed before their repairs. Final Git tests and
rebuilt-bridge results follow below. The preceding normal-release integration
probe passed all 13 checks:
repository ownership, bulk index actions, commit preparation without staging,
history, new files, dirty buffers, hook recovery, selected stash/discard, local
bare remote, owned cancellation, no idle process, large diffs and stale refresh.
Final verification passes all 25 Rust Git tests, bridge Clippy and the normal
Windows build. The rebuilt release bridge passes five public cases: large Commit,
Discard and Revert, conflicting pop retaining the exact stash and conflicted
file, and conflicting revert followed by Abort preserving HEAD and saved bytes.
The original 678,000-byte failure now completes without changing output limits.
Final receipt: `output/git-large-actions-1a522142d7ee488bbc0c0a7b9406cc9e/result.json`.
Documentation checks pass for 107 documents/413 local links; the tracked
publication scan reports no known-pattern findings across 791 files.
Public receipts remain under ignored `output/git-large-actions-*` and
`output/source-control-17-7a29d9c22c3f426ba1629dc5e64c7f6d/`. Original profile
verification matches all 49 table digests. No provider calls were needed for
these Git operations; no original repository data or public remotes were changed.

## October 11 direct Git actions

The user's selected Git action now executes without a review dialog. Commit,
discard, selected hunks, stash, branch, revert and remote actions use the same
internal preparation and revision validation. Pickers only collect required input.
Unsaved editors, stale changes and expired state still refuse; failed commits
retain their draft. Duplicate clicks cannot prepare a second action, and leaving
the initiating view discards its pending token without mutation. Recovery messages
now say Refresh/retry instead of directing users to a removed review screen.

The native selected-discard observation also exposed JSON-escaped patch lines.
A regression failed before replacing the JSON value with its actual patch text,
then passed with multiline content and literal Windows-path backslashes intact.
All 48 focused Flutter checks (45 Source Control and three app-owner checks),
22 Rust Git tests, bridge Clippy and product-source/test analysis pass. The normal
Windows build passes. Documentation checks pass for 107 documents/413 local links;
the tracked publication scan has no known-pattern findings across 791 files.
Computer-use was stopped by physical Escape; the new direct flow has no fresh
native acceptance claim. Original profile table digests remain unchanged.

## October 11 resumed native editor audit

F04 and F06 pass on the normal Windows build in the isolated public notebook.
Physical Ctrl+P opens a relative file, Ctrl+G puts the caret at line 20,
Ctrl+F finds repeated text and Ctrl+H replaces one match. Shared undo from the
other pane restores the exact change and clears both dirty markers. Earlier
independent cursor/scroll and compact-layout evidence completes F06. No runtime
repair was needed: an input-settle delay corrected a misleading immediate capture.
The helper still refuses accessibility-index clicks, so fresh screenshot pointer
actions are used. F07 Keep editing and Save-close pass; Discard subsequently passed.
No model calls or original-profile writes were made. Checklist: 66/129 checked.

F07 now passes Discard-close and reopen, completing Save/Keep editing/Discard.
The discarded disposable draft does not overwrite a newer saved file. G11 stage
refusal preserves unsaved text and the empty index; closing the draft and freshly
staging the saved version succeeds. Individual unstage then restores the working
change without altering its contents. No original-profile writes or model calls.

G07 passes physical Ctrl+Enter and Changes → Commit. Cancel preserves draft and
index; Apply commits only fixture `8` (`d192973`), clears them and retains other
working changes. G16 branch creation, dirty-switch refusal and clean reviewed
switch to `codex/a` pass. A large tracked change exposed G17's 256 KiB stash-review
failure. Exact file/count metadata now replaces whole-patch capture for stash
create/apply, preserving revision, token, dirty-buffer and conflict checks.
The new large staged/working round-trip regression fails before the repair and
passes after it, including stale refusal and unrelated untracked retention.
All 21 source-control tests, bridge Clippy and the normal Windows build pass.
The original native case now stashes 6,000 changed lines and restores the exact
saved-file SHA-256, retaining the stash and unrelated untracked files. G17 remains
partial for conflict recovery. Selected discard restored the saved public file;
the patch formatting repair is covered above. Original 49 table digests match.

## October 11 audit: remaining background paths

Manual memory suggestions pass extraction/correction/cancel/failure/save/export
and separate-process restore. Its fixture now explicitly enables recall before
expecting a saved suggestion in context, and isolates global skills. Skill-draft
restore also passes the retained evaluation, disabled state and reviewed rollback.
These changes correct test assumptions rather than runtime behavior.

The live learning runner now accepts an explicit reasoning mode. With thinking
Off and the bounded 4,096-output / 30-second foreground setting, Qwen and
DeepSeek both completed the public config edit while preserving check scripts
and unrelated files. Qwen's independent trial was incomplete; DeepSeek's complete
trial tied the baseline. Neither activated. This qualifies strict withholding
and retained results, not positive automatic gain or regression restoration.
The runner correctly returns failure for its unachieved positive-activation goal;
reported total live usage is unavailable, not zero. Receipts: `learning-qwen-live`
and `learning-deepseek-live` under the ignored background audit directory.

Real Qwen invoked both reviewed MCP tools once, with exact arguments, two
approvals and matching receipts/answer: three model calls, 19,750 reported
tokens. It also searched the real default provider (four results), read the
explicit Rust Book primary URL and cited that exact URL while distinguishing
search snippets from the retrieved excerpt: three calls, 22,318 tokens. These
checks qualify retrieval/receipt provenance, not exhaustive prose accuracy or
native review/display. Public receipts: `mcp-qwen-live` and `web-qwen-live`.

Qwen's real recovery-mod draft used invalid Wasm syntax. The source and failure
were retained; no activation occurred (621 reported tokens). Explicit DeepSeek
fallback generated a valid ABI 1 module, passed all six fixed checks, activated
in the disposable profile and restored the baseline (214 tokens). No imports,
capabilities, dependencies, permission changes or relaxed checks were introduced.
Public receipts: `mod-qwen-live` and `mod-deepseek-live`.

The actual app browser adapter opened a public local form, accepted reviewed
input/click with fresh state tokens and captured the changed page. Qwen then
proposed invalid close arguments and paused at the explicit six-call/five-tool
limit; its four successful receipts and screenshot survived, and owned browser
workers returned to zero (46,453 reported tokens). Explicit DeepSeek fallback
completed the same operations and truthful final report in five calls / 37,100
tokens, with no blind retry and zero remaining workers. Both local screenshots
were inspected with view_image and show PUBLIC-BROWSER-42 in the field and page.
The model accurately distinguished text verification from a local screenshot
it had not received as vision. No Windows helper or alternate native automation
was used. Receipts/screenshots: `browser-qwen-live` and `browser-deepseek-live`.
Native app review/capture controls remain pending.

Both disposable live schedules were deleted after report/restart verification.
Their six source/result conversations and public project files remain unchanged;
native Delete confirmation is still pending. All 49 original-profile table
digests match. The checklist's coverage/native-queue matrix maps every remaining
ID to its background evidence and missing interaction; the native count stays
60/129. Advanced positive learning gain and native repair-to-install remain
additional acceptance gaps. The broken Windows helper was not bypassed.

## October 11 audit: scheduled occurrence execution

Real Qwen correctly created a weekday 21:00 task from Chinese instructions, but
its occurrence repeated setup confirmation instead of producing the pinned
report. The original request said to create the task now without running it yet;
the host reused that historical request as the active execution instruction.
The occurrence finished successfully at the transport level without delivering
the requested work. DeepSeek produced the report before the fix; that fallback
does not excuse the contradictory handoff.

For the provider handoff only, the exact original request is now quoted as
historical provenance, preceded and followed by a current execution instruction.
The task, source conversation and occurrence history retain the original bytes.
Later human instructions are not replaced. Pinned skill, workspace, model,
permissions, tools and budgets remain unchanged. No language keyword parsing,
automatic retry, extra approval grant or migrated task data was introduced.

The normal-bundle wire regression failed before the fix. All seven scheduling
unit checks, all 469 workspace Rust checks (two ignored), all-target Clippy and
the normal Windows build pass. The 14-request scheduling corpus and separate-
process reopen pass, including pinned/unpinned source preservation, stale skill,
Stop, overlap, offline and claim recovery. The existing saved Qwen task now
produces the public pinned report (one execution call, 5,246 reported tokens,
no tool reviews); a fresh process retains it without replay. Its original
two-call creation used 13,495 tokens. The earlier pre-fix DeepSeek creation and
report used 18,255 tokens and retained the report on separate-process restart.
Manual Run now qualifies execution, not a physical timer or native controls.
All 49 original-profile table digests still match. Evidence is ignored under
`output/background-audit-20261011-remaining/` in `scheduling-handoff-red`,
`scheduling-handoff-fixed`, `schedule-qwen-live` and `schedule-deepseek-live`.

## October 11 audit: image caption display titles

The wider real-model probe reproduced valid caption JSON without the required
Image: title prefix. This prefix is host display metadata; the host now adds it
after validating the supplied title. Caption evidence and asset identity are
unchanged. Empty/oversized titles, malformed or truncated JSON, and existing
description/uncertainty bounds still refuse. The original failing regression,
93 core tests, all 468 workspace tests (two ignored), Clippy and normal Windows
build pass. The live runner now returns failure when its receipt fails.

The ten-turn Qwen probe passed useful fact capture, correction, fresh-chat recall,
Memory Off and project isolation, but failed image capture: 46,963 foreground
and 280 maintenance tokens. After the fix, one Qwen image capture/recall passed
(5,264 + 257 tokens); the independent second refused oversized uncertainty
(4,830 + 278 tokens), retaining the reply and shared image. The explicit DeepSeek
fallback passed (5,536 + 374 tokens). No runtime fallback/retry or relaxed bound
was introduced. These public receipts remain ignored under
`output/background-audit-20261011-remaining/` in `memory-qwen-thinking-off-full`,
`image-title-qwen-fixed`, `image-title-qwen-fixed-02` and
`image-title-deepseek-fixed`. All 49 original-profile table digests match.
This bounded result does not establish general Qwen caption reliability or
native Memory-page interaction; the corresponding UI checks remain open.

## Background audit fixture repair — compact command recovery (October 11)

The full frontend suite reproduced an offscreen test tap in compact command
recovery: the fixture scrolled toward Repair and verify but clicked before the
new layout settled. Waiting for that frame preserves the original failure,
draft-protection and explicit-repair assertions. No runtime behavior changed.
The focused two-check retest and all 420 frontend tests pass. The full log stays
ignored under `output/background-audit-20261011-remaining/flutter-tests-retest.log`.
Native recovery coverage is unchanged.

## Background audit repair — companionship model settings (October 11)

E01's earlier thinking/output failures exposed a settings mismatch: companionship
constructed fresh Provider default requests instead of using the chosen model's
saved reasoning control. The regression reproduces that mismatch. Requests now
preserve that model's reasoning and tighter output/timeout settings, retaining
the existing 256-token / 20-second ceilings. The foreground model remains selected;
no automatic mode change, fallback, retry or added budget is introduced.

All 172 bridge tests pass, with the configured-vault/live check ignored. All-target
Clippy, four credential-free qualification-runner checks and the normal Windows
build pass. Two bounded qwen3.7-flash invitations using explicitly chosen thinking
Off both pass through the same model-profile/request constructor as runtime:
230 input and 28 output tokens, 258 reported total, zero tools. Original data is
unchanged. The receipt is ignored under
`output/background-audit-20261011-remaining/companion-qwen-thinking-off-02.json`.
This proves generation with that explicit mode, not native eligible delivery;
E01 remains unchecked. Provider default failures remain recorded separately.

Desktop control failed again in the native folder chooser: the returned field
index was unavailable in cached app state. A refreshed window lookup/activation
and coordinate focus attempt still reported the search field as focused. No text
was submitted into the wrong field. The owned preview was stopped, preserving
its profile. G15's binary comparison passed; nonrepository navigation remains
pending. Background testing continues under the user's fallback instruction.

## Native audit repair — stale diff refresh (October 11)

G14's native comparison displayed the 744 KB replacement as aligned old/new
text. Next/Previous preserved the corresponding page. Appending an owned public
footer invalidated the next read, retaining the readable comparison and showing
the actual stale-state reason. Refresh diff then reproduced a defect: it reused
the cached repository revision and repeated the refusal indefinitely.

Refresh now reads current repository status before requesting the same file,
basis, commit and layout. A failed status read retains the page without retrying
with stale state; changing tabs while that read waits preserves the new selection.
Both refresh-button regressions failed before the fix. All 26 Source Control
tests, changed-source analysis, formatting and the normal Windows build pass.

The rebuilt native app recovered to 6,006 logical rows with the new digest and
no stale error, then Next displayed rows 257–512 beginning at old/new line 252.
The owned fixture was restored to its exact original bytes; all 49 original
profile table digests match. No model calls or Git mutations were required.
Native observations remain ignored in
`output/native-audit-20261010-01/observations-20261011-retry.jsonl`.
G14 is checked; the count is 60/129. Remaining Git workflows follow separately.

## Native audit recovery — Git history tabs (October 11)

The owned normal `72d98bd` desktop reopened and Windows automation recovered.
The final G13 retest shows distinct history.txt tab labels 1f5bca87 and 8f683959
with exact comparisons 060 → 061 and 059 → 060. The second tab was changed to
Inline; returning to the first retained Side by side and its exact contents.
Native drag selection visibly highlights the right-side revision. Earlier
three-page history coverage completes this item. No repository changes or model
requests were made. Native observations remain ignored under
`output/native-audit-20261010-01/observations-20261011-retry.jsonl`.
G13 is now checked; the running count is 59/129. G14's native retest follows.

## Audit background repair — aligned large-diff pages (October 11)

G14's actual 744,000-byte public replacement exposed a paging defect: the first
raw page contained 251 removals and no additions, while the tail contained only
additions. Its saved side-by-side render failed the new-text assertion and showed
a blank right side. The named source-control contract now requires aligned pairs
across page boundaries. Side paging streams only its selected slice; inline
paging and complete mutation-review bounds remain separate. Successful layout
changes reset paging; failed reads preserve the previous page/layout, and each
tab retains its own layout and trail.

The original render regression is green. The bridge suite passes 171 tests,
with one unrelated vault test ignored; all 23 Source Control tests, changed-
source analysis, formatting and all-target Clippy pass. Regressions cover a
replacement spanning pages, long UTF-8 segments and unequal changes without
loss, stale fingerprints/revisions, small-diff hunk eligibility, failed layout
changes and independent tab navigation. The normal Windows build passes.

The rebuilt packaged bridge rendered all 6,000 old/new pairs over 24 pages
(6,005 logical rows), in order without missing or duplicate changes. Peak page
source text was 63,488 bytes; the 23 continuation reads took 44.08 seconds in
aggregate. The public source stayed unchanged and no model calls were made.
A saved render of the real first page shows aligned red/green lines and readable
base labels; this is a widget render, not native pointer or selection evidence.
Public receipts, first/last pages and before/after renders remain ignored under
`output/git-large-render-20261011-01/` and `-02/`. All 49 original-profile table
digests still match. Native controls remain blocked by the documented desktop
activation failure; G14 stays unchecked and the checklist remains 58/129.

## Audit background repair — read-only delegated guidance (October 11)

A provider-boundary regression found another contradictory instruction: a
read-only child was told to propose `create_text_file`, which its catalogue
excludes. File-creation guidance now belongs only to writable children. The
original regression and all five scoped-child checks pass, including shared
limits, one failed child beside a retained sibling, scope refusal and Stop.
All-target Clippy and the normal Windows build pass. The normal-bundle synthetic
delegation/restart fixture also passes after its opaque-approval correction.

The bounded qwen3.7-flash case used two read-only children with independent
direct-file scopes. Each read its assigned public file and returned its marker.
The parent independently read both files, then reported the exact markers.
Five exact operation reviews were approved; both source files stayed byte-for-
byte unchanged. All child wire requests had only the three read tools and no
file-write/command instructions. There were eight HTTP calls (four parent, four
child), reporting 36,496 input and 780 output tokens, 37,276 total. The parent
metadata's four-call count is not the combined usage. Output was configured at
512 tokens per response, idle timeout 20 seconds and task limits 10 calls / eight
tools / 80 seconds; no DeepSeek fallback was needed.

A fresh process retained the child reports, separate child usage, parent
verification receipts and final answer with no configured provider or replay.
The original 49 table digests match. Public evidence is ignored under
`output/delegation-live-20261011-01/`. This is background qualification of I10;
native approval/progress controls remain pending and the checklist stays 58/129.

## Audit background repair — instructions follow available tools (October 11)

The Memory Off failure reproduced in two of three fresh qwen3.7-flash side
chats: the provider proposed `list_folder` although the advertised catalogue had
only scheduling and questions. A DeepSeek fallback answered without tools.
All four runs kept Memory Off, stored no automatic attempt and preserved the
original 49 table digests. Their reported total was 11,330 tokens. Public wire
observations are ignored under `output/memory-off-20261011-01/`.

Preparation incorrectly added file/coding instructions to every agent run.
The minimized regression failed on the side-chat instructions, then on inherited
write guidance after switching to a read-only catalogue. General guidance now
limits calls to advertised capabilities; file and command guidance follows the
catalogue and removes inherited guidance that no longer applies. Tool dispatch,
permissions and failed-call handling are unchanged; no automatic retry is added.
The core and bridge suites pass 259 checks, with one unrelated vault check
ignored. All-target Clippy passes with warnings denied and the normal Windows
build passes. Three fresh qwen3.7-flash retests answered without a tool call or
invented codename. Their wire observations contain no contradictory file guidance;
Memory Off produced no automatic attempt. Reported usage was 6,981 input and 521
output tokens, 7,502 total. All 49 original table digests still match. Evidence
is ignored under `output/memory-off-20261011-02/`. This bounded improvement does
not establish universal model compliance or native memory UX.

The subsequent normal-bundle delegation fixture initially failed `stopApproval`:
it expected a `child.` approval-ID prefix, but IDs are intentionally opaque. It
approved both child writes without ever sending Stop. The fixture now recognizes
only its two owned create operations and requires proof of interruption/revocation
before accepting a terminal event. The original failure is green without an app
cancellation change. Background concurrency, five exact reviews, shared-budget
pause, Stop/revocation before writes, scope/ownership refusal and fresh-process
restart all pass. No native delegation acceptance is claimed.

Native audit remains blocked after the user's retry request. Fresh selection
and one retry each returned `foreground window did not report a process id`.
No alternate native automation was used. The checklist stays at 58/129;
background evidence does not check off the pending native flows.

## Audit background repair — saved tool evidence in follow-ups (October 11)

A minimized save/restart test reproduced the A07 historical-evidence gap: the
command receipt remained in turn metadata but was absent from the next-request
history. The model had only the saved acknowledgement. Summary-aware loading,
used by both preview and sending even without a summary, now projects bounded
text receipts beside their original assistant turn. At most eight receipts and
16 KiB are added per snapshot; each result preview is at most 4 KiB, with explicit
truncation/omission. They are labeled historical and untrusted, and grant no action
or verification. Saved messages, exports, attachment assets and summary-source
provenance stay unchanged; summarized/other-chat receipts are excluded.

The original failure is green after repair. Three store regressions cover restart
without transcript changes, UTF-8/count/byte bounds and summary/chat isolation,
and malformed-metadata refusal preserving saved work. The bridge test verifies
preview equals the sent messages and saved token estimate, including the receipt
before complete-turn fitting. Store and bridge suites pass 242 checks, with one
unrelated vault test ignored. Formatting, all-target Clippy with warnings denied
and the normal Windows build pass. The separate scheduling source-order repair
below additionally passes all six focused scheduling checks.

The normal packaged bridge passed a keyless two-process check. A loopback seed
provider requested two separately reviewed public print commands, yielding actual
exit-0 and exit-7 receipts. Their stdout markers appeared only in those receipts,
not saved acknowledgement prose. After restart, both fixture follow-ups and two
qwen3.7-flash follow-ups returned the exact marker and exit code without tool use
or command replay. The live follow-ups used two calls: 12,762 reported input and
248 output tokens, 13,010 total; each response was bounded to 1,024 output tokens
and a 20-second idle timeout. All 49 original-profile table digests remain
unchanged; only connection configuration was copied and credentials stayed in
memory. Reports are ignored under `output/history-evidence-20261011-*`.

These are background storage/provider checks, not native UX or proof of arbitrary
model reliability. A07's earlier native repair flow remains checked, but its
historical conversation needs a fresh native retest. No additional checklist item
is checked: 58/129 remains. Desktop activation is still unavailable as recorded
under G13; no alternate native automation was used.

## Audit static-check repair — scheduling module order (October 11)

The all-target Clippy check failed on `items_after_test_module`: two existing
scheduling context helpers followed the test module. Their bodies were moved
unchanged above the tests. All-target Clippy for the store and bridge now passes
with warnings denied; all six scheduling tests pass. This source-order-only
cleanup changes no runtime behavior and needed no further desktop rebuild or
native interaction. Hosted CI remains separate from these local results.

## First-user audit — history paging and revision tabs (October 11)

G13 native interaction loaded all three pages of 62 public commits without
replacing the readable diff. Three tabs showed exact revisions 061, 030 and
001; returning to the first retained 060 → 061. Side-by-side comparison showed
different red/green text and native selection highlighted the changed new text.
Identical tab labels made those revisions hard to distinguish. Labels now use
the short commit ID. The new regression failed before the fix, then all 21
Source Control tests and changed-source analysis passed. The normal Windows
build passed and the owned isolated profile reopened. Native retesting of the
new labels could not proceed: desktop activation reported `foreground window
did not report a process id` twice, including a fresh returned-window lookup.
Native automation stopped; G13 stays unchecked until retested. G14's 744 KB,
6,000-line diff was prepared in the disposable repository only. It establishes
no native paging pass. All 49 original-profile table digests still match; no
model requests were needed. Raw evidence remains ignored under
`output/native-audit-20261010-01/`.

## First-user audit — settings navigation and help (October 11)

P01 completed its remaining native coverage in the compact normal desktop.
Settings remains at the bottom of the activity rail. The category chooser
listed General, Models, Personalization, Memory, Tools and Advanced; selecting
Personalization displayed concise scope and explanation-style controls.
Searching `frequency` through normal context-menu paste found Companionship;
clicking the result opened the correct page and cleared the search. The
Quiet–Chatty slider showed endpoints 0/100 and the current `2 / day` value.
The Companionship question-mark hint appeared when the pointer rested over it,
keeping its longer explanation out of the default page. Closing settings
changed no preferences. No app change or model request was needed. P01 is
checked, bringing the checklist to 58/129. Native observations remain ignored
under `output/native-audit-20261010-01/`. Direct helper text injection and
accessibility value setting remained unreliable in Flutter; those failures
are not counted as settings search defects.

## First-user audit — split-editor follow-up (October 11)

F06 added native coverage with a 120-line public fixture. Both panes displayed
the same first-line edit; the right pane scrolled to lines 21–56 while the left
stayed at the start. Editing in the right pane used its separate cursor. After
reopening, the dirty shared buffer and each pane's position remained available.
Resizing the window below 600 editor pixels exposed compact Group 1/Group 2
controls. Hiding the side panel and switching groups retained their separate
scroll/cursor positions. Native Save retained the shared draft.

Native Control undo did not produce a verifiable result in this run. A Windows
widget test now exercises the real split workspace: edits in both panes, undo
from the first pane and redo from the second. Both controllers and the document
agree throughout. All 16 selected workspace/editor checks pass; changed-file
analysis is clean. No runtime change or new build was needed. F06 remains
unchecked pending native shared undo qualification; successful text, cursor,
scroll and compact-layout coverage is recorded separately. Public observations
remain ignored under `output/native-audit-20261010-01/`.

## First-user audit — external file changes (October 11)

F09 passed in the normal desktop with the public file created during F08.
Native typing produced an unsaved `7890` draft; an explicitly labeled background
fixture write simulated another editor saving `456`. Native Save refused with
the actual changed-on-disk explanation, retained the draft and left disk bytes
untouched. Compare displayed `456` and `7890` separately.

A second external write to `4567` invalidated that comparison. Keep my edits
refused with “Disk changed again after comparison. Compare again; edits are
retained.” Both versions remained intact. A fresh comparison then adopted the
reviewed revision without writing; native Save saved exactly `7890` and cleared
dirty state. Another external change to `45678` was compared with the clean
editor, and Reload disk displayed exactly `45678` without an unsaved draft.
No app change was needed. All six baseline project files and all 49 original
profile table digests remain unchanged. F09 is checked, bringing the checklist
to 57/129. Native observations and `f09-native-recovery.json` remain ignored
under `output/native-audit-20261010-01/`. No live model request was needed;
discarding an unsaved draft via Reload disk was not exercised.

## First-user audit — file creation and naming (October 11)

F08 exercised native New file, saving typed content, Save as and rename in the
isolated public project. Save as retained the source; rename retained the exact
content and updated the tab and tree. Saving an unsaved `789` draft over an
existing `78` file was refused, preserving the draft and both saved files.
Retry through Save as saved the draft under a fresh name. The normal rebuilt
desktop also refused rename to an existing file, then successfully renamed to
a fresh target: its exact bytes were `789`, both other files remained `78`, and
the old renamed path was absent. Error state cleared after recovery.

The first refusal displayed the same error twice. The workspace now suppresses
only the duplicate of the active document's error; distinct errors and errors
on inactive documents remain visible. A widget regression failed before the
fix and passes afterward, including both preservation cases. All 15 focused
file-workspace/editor checks pass; changed-file analysis and the normal Windows
build pass. Native retest shows one error row with recovery actions. All 49
original profile table digests remain unchanged. F08 is checked, bringing the
checklist to 56/129. Public observations and exact-byte receipts remain ignored
under `output/native-audit-20261010-01/`; no live model call was needed.

## First-user audit — editor shortcuts (October 11)

F03 typed a public BOM/CRLF file through the Windows editor, changing `41` to
`42`. Clicking Save preserved the exact expected bytes, including Chinese text;
all six other project files remained unchanged. Native Control chords did not
reliably reach their intended actions, including Find and app-level Quick open.
A temporary key-event trace recorded modifier state and was removed; diagnostic
observations do not establish normal keyboard acceptance.

Windows-specific widget tests isolated two real editor defects: the inner
editor shortcut consumed Ctrl+S without invoking Dolores's save, and Ctrl+Y
had no redo binding. The original code fails both regressions. The fix handles
the editor's Save intent and adds Ctrl+Y while retaining Ctrl+Shift+Z and shared
document undo. Both Control keys now pass undo/redo/save tests; failed save
retains the draft and shows the actual error, and retry saves the same content.
Native undo failure is not treated as a confirmed code defect: Windows widget
undo also works with the original implementation. Eighteen focused editor,
file-host, split-workspace and recovery checks passed; changed-file analysis is
clean. Seven editor checks passed again after adding keyboard save-failure retry.
The final normal Windows build passed after removing the trace. Reopening its
isolated editor retained `42`, the selected file and exact saved bytes. All 49
original profile table digests remain unchanged.

The later final normal-build retest resolved the native keyboard gap. In a newly
created public file, Right-Control+Z returned `78` to the saved `7`, Ctrl+Y restored
`78` and Ctrl+S saved exactly those two bytes. In the BOM/CRLF/Chinese fixture,
typing `8` changed `42` to `428`; native undo restored `42`, redo restored `428`,
and Ctrl+S saved its exact expected bytes. Undo after saving still worked, and
Ctrl+S restored the exact expected `42` file. All six other baseline project files
and all 49 original profile table digests remain unchanged. Native Left-Control
delivery is not claimed. F03 is checked; the checklist is now 55/129. Public
observations and byte receipts are under ignored
`output/native-audit-20261010-01/`; no live model request was needed.

## First-user audit — task inspection (October 11)

A09 inspected the saved qwen3.7-flash A12 task through the normal desktop entry:
Activity → Conversation/Requests, Earlier tasks → Run details, reply usage,
Changes → applied diff and Settings → Scope overrides → Effective settings.
The saved goal, completed status, 44 execution events and bounded uncertainty
remained visible. Changes showed the retained
`41` → `42` diff and concurrent footer. Usage displayed all 13 provider-reported
calls separately, with the 1,024-token output limit distinct from actual usage.
Effective settings identified Model profile, Model configuration and Dolores
default origins. Requests correctly reported no events in this new app session;
the saved exchanges remained separately available. Inspection made no model call.

Two wording corrections label Earlier tasks' configured output allowance as
`Output limit` (or provider limit when unset) and direct Activity users to the
current Conversation view instead of the retired Trajectory label. All 14 selected
checkpoint, inspector, chat-details and continuation widget checks pass; changed
file analysis is clean. The normal Windows build and native wording retest passed:
Earlier tasks shows `Output limit: 1024 tokens`, and Requests points to Conversation.
All 49 original profile table digests remain unchanged. A09 is checked, bringing
the checklist to 54/129. Native evidence remains ignored in
`output/native-audit-20261010-01/observations-20261011.jsonl`.

## First-user audit — history paging (October 10–11)

C12 used the normal desktop's isolated public fixture: 61 conversations, 140
messages and the exact unsent `Public paging draft`. Native Older showed the
oldest message page; switching away and returning restored that page and draft.
Newer returned to the newest page. Conversation-list Older/Newer kept the current
conversation and draft even while its row was on another catalog page.

Latest loaded the newest messages but initially left the final reply below the
viewport. Two focused compact/wide regressions reproduced hidden content: 723 and
1,176.57 pixels remained below the viewport after the latest-page jump. The list's
estimated maximum changed after layout; the app's own correction stopped following.
Programmatic positioning now retains follow intent, and scroll-metric changes
finish the jump. Scrolling upward still stops following. All 42 selected history,
AppHost, inspector, Markdown and tool widget tests pass, including streaming
coalescing and scroll-away behavior, unavailable-storage recovery, draft retention
and bounded page replacement. Changed-file analysis is clean. The normal Windows
build passed. October 11's native retest showed the final answer (turn 70) after
Older → Latest, with the exact draft retained. Scrolling upward then paging the
conversation list Older/Newer kept the reading position and restored the selected
row. C12 is checked; the checklist now has 53/129 completed items.
Read-only storage checks preserve all public fixture conversations/messages and
the exact draft, with no model run for the fixture. All 49 original profile table
digests remain unchanged. Evidence stays ignored under
`output/native-audit-20261010-01/observations-20261011.jsonl` and
`c12-storage-verification.json` in that directory.

## First-user audit — large-file editing and visible recovery (October 10)

A12 passed through the normal Windows desktop with qwen3.7-flash and a disposable
public project. A ranged read covered lines 1601–1602 of a 92,848-byte BOM/CRLF
UTF-8 file. Three mismatched exact-edit proposals and two missing-snapshot
proposals were rejected without writes. A footer appended while a valid review
was pending made Apply refuse the stale edit. The model read again and prepared
a new review; applying it changed only `41` to `42`. A final ranged read verified
the change. Exact byte comparison preserved Chinese text, BOM, CRLF, every other
byte and the concurrent footer. Five other files and all 49 original profile table
digests were unchanged. The run used 13 model calls and 12 tool actions, with
112,930 reported input and 2,817 output tokens across those calls. Initial model
mistakes are retained evidence; this does not qualify efficient editing.

The audit reproduced a UI defect: only the first four action receipts appeared,
hiding the stale refusal, applied edit and verification. Chat now shows the latest
actions and offers Show all / Show recent. A failed preparation's placeholder no
longer falsely labels every error an invalid path; its actual failure reason stays
selectable. Three new regressions failed before the fix and pass afterward, covering
compact light/dark, earlier failed attempts, collapse and receipt updates while
running. All 38 selected widget tests and the two existing Rust ranged-file checks
pass. Changed-file analysis is clean; whole-app analysis also reports one existing
unnecessary import in ignored screenshot tooling. The normal Windows build passed.
Native reopen confirmed the retained successful diff and final read, stale refusal,
earlier exact-match rejection and Show all / Show recent. A12 is checked;
the checklist now has 52/129 completed items. No other item was started in this pass.
Evidence remains ignored under `output/native-audit-20261010-01/`:
`a12-run-receipt.json`, `a12-byte-verification.json`, `a12-public-events.json`,
`a12-saved-metadata.json` and `observations.jsonl`.

## First-user audit — question answers and panel focus (October 10)

The normal desktop entry point used an isolated configuration-only profile and a
public disposable project. qwen3.7-flash asked two clickable questions; no answer
was initially selected. Choice selection, Back/Next retention and a custom date
overriding the earlier choice all worked. Sending continued the same run and the
model acknowledged `2026-10-11` and English exactly. Two model calls reported
13,979 input and 371 output tokens (14,350 total). Native failed-send retry is still
open; deterministic host/widget checks already cover rejected and duplicate
answers, preserved input and retry. Evidence stays in ignored
`output/native-audit-20261010-01/observations.jsonl` and the isolated run receipts.

The audit reproduced the panel-divider keyboard failure before changing code:
after a pointer click, Right moved focus to Home instead of resizing the panel.
The new regression likewise kept the width at 252 instead of 268 before the fix.
The divider now explicitly acquires focus on click and drag start. A first normal
Windows rebuild verified Left/Right, Home reset and title-bar hide/show with the
draft retained. A quick one-motion native drag still left the width unchanged;
the matching mouse regression failed at 252 instead of 316. Counting drag movement
from pointer-down fixes that lost initial movement. All seven workspace-shell
widget checks now pass, including click/drag keyboard focus, fast dragging, retained
draft and disposal. The final normal Windows build passes. Native fast widening,
drag-to-hide, title-bar restore, Left/Right and Home now work while retaining the
conversation and exact unsent draft. C10 is checked. Changed-file analysis,
documentation/link and publication-pattern checks pass. All 49 original profile
table digests remain unchanged.

The rebuilt bundle also passed seven background regression save/reopen pairs
(feedback, comparisons, continuation, generation profiles, coding repair, exact
edits and automatic memory). The skill-draft save stage failed because its fixture
still expected exactly two side-chat tools. Foreground side chats now also expose
`request_user_input`; the fixture now checks the exact three-tool catalogue while
continuing to require tool-free drafting/evaluation. A fresh skill-draft save and
reopen pair passes, including output exhaustion, explicit larger retry, ties,
regression refusal, interruption and one-use promotion. The original failed report
is retained separately from this successful retest; all eight selected pairs now
have completed passing evidence across the two reports.

The optional companionship runner no longer hardcodes a local model or treats
collector completion as a successful live qualification. It requires an enabled
`--model`, refuses an existing receipt before credential access, closes its
read-only profile connection and returns failure for a rejected case, unsuccessful
child or changed original profile. Three credential-free tests pass. A fresh
qwen3.7-flash run attempted two tool-free invitations under the existing
256-token/20-second limits; both failed and the receipt reports `passed: false`.
All original table digests remain unchanged. This is an open real-model gap,
not a native delivery qualification. Public evidence is retained under ignored
`output/native-audit-20261010-01/companion-qwen.json`.

Public-only transport diagnostics narrow the live gap: a fresh Qwen invitation
completed in 15.274 seconds, but its next memory invitation still had no answer
when cancelled at 20 seconds. The successful call sent `max_tokens: 256` and
reported 1,202 output tokens, including 1,183 reasoning tokens; this provider's
reported total output exceeded that submitted allowance. A direct DeepSeek
fallback failed both cases. A separate image-only Qwen probe retained its correct
foreground description but cancelled image indexing at 10 seconds after 1,565
reasoning characters and no caption. Original profile digests remain unchanged.
No default was raised, no partial caption was saved, and no native item gained a
checkmark. These are retained, unresolved real-model timing/provider-contract gaps.

## CI regression — Windows language paths and catalog fixture

The reported editor panic was reproduced at the same line by running the original
test with an owned Windows temporary-folder junction. A normal temporary folder
passed, showing the path-dependent failure. Language edit URIs and project roots
now resolve to canonical filesystem paths before deriving the relative target.
The original aliased-folder reproduction passes after the fix. A new Windows
regression covers folder aliases, letter casing, encoded Unicode/spaces, duplicate
aliases, outside sibling files and an escaping junction, with atomic Apply/Undo
and unchanged disk/buffer contents after refusal.

The full normal `cargo test --workspace --locked` run exposed an additional stale
storage fixture: it expected overflow at 16 tools after scheduling raised the
shared limit to 17. The fixture now includes both scheduling tools, accepts the
current boundary and rejects one extra tool while retaining the draft and prior
run. No runtime allowance changed in this correction. The final full command
passes **442 tests**, with **two existing optional ignores** (native credential
store and configured-provider companionship). Strict bridge/store Clippy, Rust
formatting, documentation/local-link and publication-pattern checks pass.

The broader workspace-wide junction injection remained red in three unrelated
fixtures: private install/compiler storage intentionally refuses linked paths,
and the no-repository fixture was placed inside this checkout. Those guards were
retained; this diagnostic environment is not claimed as a supported private
storage setup. Logs remain under ignored `output/language-ci-*.log`.

Normal Windows main-entry build/reopen passes; all **49 original profile table
hashes** remain unchanged. This is local verification of the reported Rust CI
command, not a completed hosted GitHub workflow or other-platform qualification.

## Source Control — quick staging and empty-index commit

Section +/− controls and file +/− controls stage/unstage saved changes with one
click; their menu equivalents match. Bulk actions use the complete bounded status,
without the selected-path or full-diff staging limits. Commit keeps a nonempty
staged selection; otherwise its single review prepares working changes in a
temporary index and stages them only after confirmation. Cancel changes no index.

The missing-button regression failed before the fix. **20 Source Control Flutter
tests** pass, covering all four controls, busy disabling, failure/retry, retained
drafts, history recovery and light/dark 252/320 px layouts. Final saved widget
renders were inspected; they are appearance evidence, not physical input checks.
**Five focused native mutation tests** pass, covering bulk new/modified/deleted/
renamed/large files, initial commits, preserved partial staging, stale-review
refusal and hook-failure recovery. A discovered staged-rename old-path failure
was corrected and its test passes. Scoped Flutter analysis (`lib`/`test`), Rust
formatting and strict bridge Clippy pass. Whole-directory analysis reported an
unrelated unused import in an ignored README screenshot helper.

The normal Windows main-entry build and isolated release C ABI corpus pass all
**13 checks**. New checks exercise 22 bulk files, a literal Unicode/bracket/space
filename, unstage all, cancel-safe automatic commit and dirty-editor refusal.
Existing hook/Stop recovery, two-repository ownership, local bare remotes and
large/stale diff paging also pass; no idle Git process remains. Evidence is local
under `output/source-control-17-6045a60858864f67a263d92bb06bad65/`. The owned normal
preview reopened with window presence confirmed, and all **49 profile table
hashes** are unchanged. No live model or user's external repository was needed.
Commit/destructive review bounds (256 KiB), status/index bounds and Windows-only
qualification remain; this change does not qualify arbitrary-size commits or
physical accessibility/input behavior.

## Chat scheduling — ordinary language and automatic defaults

The recurrence allowlist and English/Chinese request/edit keyword gates are
removed. Both scheduling tools coexist with ordinary project tools on human
turns. The model interprets intent; the host validates structured metadata,
source chat, task identity/revision, enabled model/skill and duplicate calls.
An omitted creation time uses a disclosed 09:00 local default. Creation pins
the current model without asking for its ID and grants no future tool authority.
Scheduling validation errors retain their actual explanation.

Rust validation passes **235 tests**, with one intentional bridge ignore;
formatting and strict Clippy pass. The native creation corpus covers nine
requests in six languages, duplicate calls, default-time receipts, malformed
fields and preserved ordinary file tools. Save/reopen scheduling recovery also
passes, including interruption, offline behavior, stale revisions and skill
recovery. The final bounded DeepSeek corpus passes **9/9**, including English,
Spanish, Japanese and Arabic creation, disclosed default time, quote/negation
refusal and ordinary work. Confirmations are short and match saved receipts.
The normal Windows build/reopen passes and all 49 original profile table hashes
remain unchanged. Earlier model misses and limits are retained in
[scheduling creation qualification](qualification/scheduling-creation.md).

## Root tooling cleanup

The dependency-free root `package.json` command aliases and empty `pnpm-lock.yaml`
are removed. Release validation now aligns Cargo and Flutter versions directly;
its two fixtures cover promotion, mismatched versions/tags and recovery without a
root Node manifest. Documentation/local-link and publication-pattern checks pass.
The maintained build remains `python scripts/desktop.py build`; browser fixtures
and the optional adapter keep their separate npm manifest/lockfile. No desktop
runtime change or rebuild is needed.

## Open-source preparation — portable guidance, releases and Windows CI

Windows x64 is the release target. Brick 8.4 now provides credential-free GitHub
Actions checks, portable ZIP packaging and version-tag draft releases. Contributor
guidance no longer requires a private provider/model setup or local indexing tools.
Optional live helpers require explicit model selection; existing named-model results
remain historical evidence. Session handoffs are removed and ignored. Publication
history cleanup works on a separate copy and retains author attribution and commits.

The English and Chinese READMEs introduce the project and its main features. Chat
and split Files screenshots were rendered using the current widgets and synthetic
data. Existing Source Control and Memory renders were inspected and included too.
All four images have no personal paths or private conversations; widget-test
fallback glyphs use Windows fonts. They illustrate UI, not real-model behavior.

Local checks pass: Rust formatting, strict workspace Clippy, **436 Rust tests**
(two intentionally ignored), clean Flutter analysis and **341 Flutter tests**.
Publication tests pass three cases, including historical handoff removal, unchanged
source refs, retained empty commits/attribution and refusal of implicit live models.
Release tests pass two cases covering version/tag mismatch, absent/duplicate notes
and recovery. Package tests pass four cases and launcher preflight passes its
missing-file/recovery case. The two real-browser synthetic-page tests and three
browser-installer tests pass. Workflow syntax passes actionlint 1.7.12.

The normal Windows main-entry build, all **16 save/restart fixture stages**, portable
ZIP creation/verification, extracted launcher preflight and extracted native bridge
bootstrap pass in an isolated checkout. A separate empty-profile normal launch
opens a window and its owned process is stopped afterward. The ZIP includes 18
runtime files, 8 guidance/notice files, a manifest and 352 dependency components.
Missing Wasm license texts are frozen from the crates' exact upstream revisions.
Earlier fixture/package failures and recoveries are recorded in the
[release qualification](qualification/open-source-release.md).

The cleaned-history trial passes across all five branches, preserving commit count
and author/committer attribution while removing historical handoffs and known
personal text. Final-copy receipts remain local under `output/github-publication/`;
use that copy for publication, as the original retains private recovery history.
No new live-model request was made. The existing 45-table profile is unchanged.
Hosted GitHub execution, signing, clean-machine installation, physical
input/accessibility and other-platform qualification remain open. Nothing is pushed.

## Background scheduling 24.3 — qualification and roadmap batch exit

Milestones 23 and 24 are implemented in separate brick commits. Windows background
mode is optional and Off by default, retaining the same app owner in the tray.
Packaged report/reopen/duplicate-launch, provider-503, approval/crash, disable and
Quit checks pass. The frozen resource check passes with no added child process or
idle model request. Four saved light/dark narrow/wide settings renders were inspected;
failed policy saves and a failed tray hide offer visible recovery without losing
the window or draft. [Qualification](qualification/closed-ui-scheduling.md) records
exact measurements, earlier failures and boundaries.

One configured DeepSeek V4.1 Flash chat-only report completes with the UI hidden:
5,254 reported total tokens, one model call. A broader report request instead waits
for file-creation approval and remains a miss; no approval or effect was automated.
Temporary isolated credentials were removed. This is bounded live execution
evidence, not general model interpretation/report reliability.

**298 native tests pass (1 ignored), 341 Flutter checks pass**, with clean analyzer,
Clippy, Markdown links and the normal main Windows build. The maintained normal
preview uses schema **38**; **all 45 original tables retain their row values**.
The four added scheduling/companion/background tables have zero original-profile
rows; both opt-in features remain Off and no task was created in that profile.

The current roadmap has no further eligible feature implementation batch. Its
documented milestone-19 fallback remains held after the CPU gate failed; detach
transfers 19.2/19.3 are unavailable. Platform CI 8.4 remains explicitly deferred.
Physical tray/input/accessibility/suspend, subjective companionship, representative
other-host resources and general model reliability remain acceptance gaps. No
startup installation, wake guarantee, desktop notification or general autonomous
repair claim is added by this batch.

## Closed-UI scheduling 24.2 — packaged execution and recovery

The normal Windows main entry retains one owner in the tray. Public fixtures
complete one report with the UI hidden, retain the Home draft/result on reopen,
restore the same process on a second launch, and Quit with exit code 0. Separate
provider-503 and waiting-approval cases preserve recovery; an owned crash/restart
does not duplicate the occurrence or replay the unapproved read. Background Off
restores ordinary Close and releases ownership. Native lifecycle protocol checks
are distinct from physical tray clicks and real wake/suspend behavior.

The first packaged run exposed a 15-built-in catalog against the older bound of
14; the exact bound is corrected to 15 with a draft-preserving overflow message.
A separate `0xc000041d` Quit failure was fixed by explicitly destroying the derived
window and unregistering its channel before engine teardown. A Close retry race
now waits for the pending scheduler operation, bounded to two seconds. Native
libraries: **298 pass, 1 ignored**; Flutter regressions: **340 pass**; analyzer and
Clippy pass. Earlier failed checks are not acceptance evidence. Details and limits:
[closed-UI qualification](qualification/closed-ui-scheduling.md).

## Background scheduling 24.1 — same-owner tray lifecycle

Windows has an Off-by-default background setting, native tray Open/Quit actions
and per-directory launch ownership. A second launch restores the existing owner
before creating another engine; the database lock remains the final guard.
Close saves recovery, stops ordinary tasks/PTYs, and retains scheduled owners;
Quit uses the normal review/shutdown. No startup or OS wake setting is installed.

Three new lifecycle fixtures and eighteen existing host/frame/scheduler widget
checks pass. One new store test covers restart, stale revision and failed save.
The normal main Windows build compiles the native owner/channel/tray integration.
Packaged hidden execution, duplicate launch and resource checks are subsequent
qualification; physical tray clicks and other platforms remain open.

## Companionship 23.4 — qualification and preservation

Milestone 23 is implemented. [Qualification](qualification/companionship.md)
separates fixtures, packaged policy/restart checks and two configured Qwen requests:
one invitation passes, one is rejected, 287 reported total tokens, no retry/tools.
296 native and 332 Flutter library/widget checks pass; eight focused companionship
checks pass after final cleanup, plus clean Clippy/analyzer and normal main build.
All 45 original tables retain their row values; companionship remains Off with
no original-profile state row. Final source/failed-save refinements are verified
by focused checks and the final batch build before handoff. Subjective welcome,
physical presence, enabled sustained resources and non-Windows support remain open.

## Companionship 23.3 — calm delivery and preferences

Personalization now has concise Companionship controls with hover help and
collapsed source/activity details. A small Home card offers Open and a menu for
Not now, Dismiss, Fewer messages and Turn off. Opening marks the note seen;
delivery itself does not change the selected chat, draft, project or model.
The host arms one 60-second eligibility timer only while enabled.

Six new Flutter checks pass across narrow/wide light/dark settings, retained
edits after a failed save, deliberate card opening, Not now, and failed feedback
followed by fewer messages. Eleven existing shell/settings checks also pass;
Flutter analysis is clean. Saved renders show compact forms without paragraphs.
Physical presence and subjective welcome remain qualification gaps.

## Companionship 23.2 — bounded grounded generation

One no-tool request uses a separately chosen enabled model, 256 output tokens and
a 20-second deadline. Windows foreground/last-input checks guard admission and
delivery. Generic chat contains no private history; the reviewed fact links to
NASA. Recall quotes retained same-project evidence. Open-work sources are usable
only while their source is the latest user turn; later conversation suppresses
the reminder rather than guessing whether it is still unresolved.

Delivery and its paired conversation commit atomically with policy/source checks.
Five focused bridge/store tests pass: suitable/malformed/pressuring output,
normal and duplicate publication, Off/restart cancellation, failed atomic save,
and Forget/Memory Off/later completion versus a current source. Unknown usage
remains unknown. Native presence and actual model generation need batch qualification;
non-Windows presence is unavailable and stays quiet.

## Companionship 23.1 — persisted quiet eligibility

Off is the default. Eligibility uses named-zone hours, a persisted 2-attempt daily
cap, a minimum three-hour gap, randomized opportunities, one pending candidate
and one unread message. Busy/absent periods do not claim requests; overdue
opportunities reschedule rather than catch up. Idle checks do not rewrite unchanged
state. Schema 37 adds a bounded singleton without changing existing records.

Four focused core/store tests pass: basic admission/cooldown, DST/unread/quiet
hours, absence without catch-up, and restart/stale/failed-save preservation.
Generation, publication and user experience are subsequent bricks; these fixtures
do not establish actual model generation or physical presence behavior.

## Terminal 18.2 — tabs and independent split groups

Terminal has its own session list, draggable tabs, plus and overflow actions.
Plus snapshots current Home selection; right/down split creates a new PTY in
that default folder. Moving/reordering/edge-splitting an existing tab transfers
only the view, preserving its emulator/process/output. Four groups resize and
provide menu move/focus/join alternatives. Compact mode shows the focused group
and retains menu access to every other group; file layouts and Home are separate.

Four terminal Flutter checks pass, including A/B plus ownership, single-session
identity after move, invalid drop and failed plus retaining tabs, split creating
the new B shell, close cancellation and compact group navigation. The existing
workspace checks remain passing. These are deterministic widget/controller
checks; native drag, focus and physical shell input are batch qualification gaps.
Private cold-restart display/output sharing follow in 18.3.

## Terminal 18.1 — real PTY and initial folder

Terminal entry now starts a real portable-pty/ConPTY shell only when no terminal
exists. The selected Home conversation supplies its initial root; without a
project it uses OS home. Revisiting after changing selection keeps the old shell
and its original owner. Missing roots/spawn failures expose Retry / Choose folder /
Open at home while Home and editing remain usable. ANSI rendering uses pinned
xterm 4.0.0, 5,000-line scrollback and incremental Unicode decoding. Copy selection,
paste, resize and Ctrl+C use terminal input, without model control or credentials.

One real Windows PTY test passes for input, cursor-position handshake, resize,
invalid sizes/oversized input, missing root, Stop and cleanup. The first test
failed because its test consumer omitted ConPTY's required cursor reply; the
corrected consumer models that emulator response. Two new Flutter tests pass for
lazy/reused ownership, split UTF-8 reads, failed spawn/Home recovery, retained
output and explicit retry after a connection failure. Updated workspace tests
exercise the new unavailable-backend recovery rather than the old placeholder.
Native restart now refuses live terminals before consuming its review.

Normal packaging, native terminal interaction and resource/model qualification
remain batch work in 18.6. Tabs/splits, private cold-restart display and first
language services follow in 18.2–18.5. This does not establish complete milestone
18 or native physical keyboard/IME acceptance.

## Source Control correction — large files and aligned changes

The reported large-file failure was reproduced on a real saved file: opening its
small edit returned `Git blob exceeds 256 KiB; use external Git.` Whole blobs
were loaded before computing the patch. The viewer now streams Git's patch into
pages of at most 256 display rows / 256 KiB, without a whole-file size gate. Long
lines are segmented at UTF-8 boundaries. Saved-byte revision hashing streams
through a 64 KiB buffer instead of rejecting files over 16 MiB. Git work retains
its deadline, cancellation and owned-process cleanup.

Side by side now aligns old/new changed rows, with red removals, green additions,
actual file line numbers and shared scrolling. Inline uses the same changes.
Previous changes / Next changes keep viewing inside Dolores. Continuations pin
the repository revision and complete patch digest; a stale or failed page load
retains the readable page and offers Refresh diff. Binary files show Git's change
summary. Partial pages never become complete mutation approvals; the separate
256 KiB full-review bound remains explicit.

Verification: 136 native bridge tests and 292 Flutter tests passed; strict Clippy
and Flutter analysis are clean. The final layout also passed all 14 focused Source
Control tests. Native regressions exercise a file over 16 MiB with one small edit,
a patch over the old 512 KiB process-output cap, an untracked file, stale revision
and mismatched continuation digest. The collector preserves a 300 KiB Unicode
line across pages and tracks file line numbers. UI checks confirm aligned old/new
values with distinct colors, retained content after failed Next, explicit retry
and Previous recovery.

The release C ABI corpus passes all twelve checks, including large-file viewing,
large-patch paging and stale-page refresh recovery:
`output/source-control-17-eb1b747ec20c4227b52b68d6865a8293/result.json`.
Hook Stop/reap took 439 ms with no remaining owned children. Its first run failed
an incorrect fixture assertion expecting additions on a page containing only
removals; the corrected assertion verifies the fresh comparison's final page.
This was a test expectation correction, not a product recovery claim.

Public isolated native renders in `output/source-control-large-diff-renders/`
were inspected with `view_image`: light/dark aligned changes, compact layout and
the second page. Rendered widget callbacks verified Next, nested history/file
selection, review cancellation and eight retained tabs; completion reported no
error and zero owned children. These are diagnostic renders/callback checks,
separate from normal startup and physical mouse/keyboard evidence.

Remaining limits: only changed hunks and surrounding context are shown. A very
large replacement can cross page boundaries, leaving unmatched cells on a page;
this is not a complete merge editor. Binary content previews, physical input/
accessibility, full branch graphs and sustained/other-platform resource coverage
remain unqualified. No model calls or hosted publication were needed. Milestone
18 remains unstarted.

Normal handoff: the maintained release build at runtime commit `333b912` completed;
the normal `main` entry launched with a visible Windows window (PID 34372). All
42 original profile table hashes remain identical, with no new tables. Provider,
selected model, configuration and saved history are preserved. Receipt:
`output/source-control-large-diff-normal-handoff.json`. This establishes normal
build/startup and profile preservation; diagnostic renders remain separate.

## Source Control refinement — VS Code interaction

The documented interface design is implemented within milestone 17: Changes owns
the repository overflow menu; Branch, Stash and whole-file actions use grouped
entries, and Commit/Fetch/Pull/Push use familiar names. Branch/stash/remote pickers
replace permanent action lists. Commit still reviews exact staged work; Ctrl+Enter
opens the same confirmation, and Cancel retains the message.

History initially loads for a committed repository. Each commit expands its own
files; a file click opens the exact commit comparison. Loading, failed reads and
Retry files stay on the affected row. Eight cached commit file sets per repository
retain usable work; collapse one to recover from the expansion limit. A new
history refresh drops cached expansions outside its refreshed page.

All 291 Flutter tests pass, including thirteen focused Source Control checks:
nested commit/file interaction, read failure and row-specific retry, late A
replies while B is selected, the eight-set limit and recovery, menu/keyboard commit
confirmation and cancellation, configured remote selection, and stashing tracked
changes without untracked files. Existing stale-diff, draft/hook, hunk and owner
checks remain passing. Native Git mutation code and its previous qualification
are unchanged.

The isolated native release used three public commits and the production panel,
menu, host and diff widgets. Rendered widget callbacks expanded two different
commits and opened a historical file; the result verified its exact commit ID.
Stage review/Cancel, eight retained diff tabs and zero owned child processes at
completion passed. Saved light/dark history, overflow menu, eight-tab and compact
renders were inspected using `view_image`:
`output/source-control-refinement-renders/`. These are diagnostic screenshots and
callback interaction evidence, not physical mouse/keyboard or normal-profile UX.
The final tracked-only stash filter is separately exercised by the focused test.

Normal handoff: maintained build of runtime commit `abe1c31` completed and the
normal `main` entry launched with a visible Windows window (PID 34896). All 42
original profile table hashes remain identical, with no new tables: provider,
selected model, configuration and saved history are preserved. Receipt:
`output/source-control-refinement-normal-handoff.json`. This proves normal build,
startup and profile preservation; diagnostic callbacks remain separate evidence.

Remaining gaps: full branch graph, aligned side-by-side changed rows, physical
input/accessibility, large expanded histories and sustained/other-platform
resource coverage remain unqualified. No model calls or real remote publication
were needed for this human Git UI refinement. Milestone 18 is not started.

## Source Control 17.6 — batch qualification

The complete 17.1–17.6 batch is implemented. Full suites pass with 133 native
bridge tests and 287 Flutter tests; strict Clippy and Flutter analysis are clean.
The public release C ABI corpus passes nine owner/editor/history/hook/stash/remote
and cleanup checks. A real canceled hook reaps in 512 ms, with no owned children
and a successful fresh reviewed commit. The initial dirty-buffer integration
failure exposed Windows extended-prefix versus stored-path comparisons; native
editor/task guards and Flutter owner retention now use the same boundary.

Final public diagnostic light/dark and compact renders were inspected, including
eight 245,783-byte saved comparisons. Earlier tiny/mixed render measurements are
explicitly invalidated. [Qualification](qualification/source-control.md) records
actual timings, memory/CPU observations, fixture failures and remaining native,
hosted-network and model gaps. Original 42-table profile preservation and normal
main-entry build/launch passed at source commit `f19106f`: the maintained normal
build opened a visible Windows window (PID 18620), and hashes of every original
table remained unchanged, with no new tables. The original configuration, selected
settings and history were preserved. No diagnostic entry is reported as the normal
app. The final normal-bundle C ABI recheck passes all nine cases, including native
dirty-buffer refusal and hook Stop/recovery (469 ms; no surviving owned children).
The original 42 tables remain unchanged after that recheck. Milestone 18 has not
started. Receipt: `output/source-control-17-normal-handoff.json`.

## Source Control 17.5 — deliberate remote actions

Remotes/tracking and ahead/behind load locally on request; network work begins
only when reviewing a named remote action. Fetch, fast-forward-only Pull and Push
use installed Git and configured credential helpers, without credential copying
or interactive prompts. Push reviews exact HEAD/remote ref and rechecks that ref;
uncertain results reconcile before a fresh remote review. Eight uncertain results
are retained at most. Restart loses that in-memory warning, but every fresh Push
still reads the current remote ref and refuses an already-present HEAD. Remote
helper diagnostics are withheld because they may contain credential-bearing URLs.

Eleven native Source Control checks pass, including a disposable bare remote's
Push/Fetch/fast-forward Pull, changed remote-ref refusal, divergent Pull preserving
HEAD/files, and injected uncertain-push reconciliation without repetition. Eight
focused Source Control UI checks pass, including named remote selection and only
offering Pull/Push for configured tracking. Flutter analysis is clean. No hosted
remote, actual credential-provider failure or real interrupted network publication
was exercised; the latter uncertainty is deterministic fixture evidence. These
limits remain distinct from the release qualification in 17.6.

## Source Control 17.4 — selected hunks and local recovery

Text hunks use host-derived IDs and patches against the displayed revision;
selected staging/unstaging preserves other hunks and working bytes. New, renamed,
mode-changing and binary files use reviewed whole-file operations. Local actions
include selected tracked-file stash, exact stash apply/pop, branch create/switch,
tracked-file discard from Index and a new non-merge reversing commit. Conflicted
pop retains its stash; conflicted revert exposes reviewed Abort. Whole-repository
actions refuse unsaved editors, and successful changes refresh saved editor/tree
state. Rename stash is explicitly deferred to external Git rather than guessing
which names to include.

Eight native Source Control checks pass: selected/stale/reverse hunks, selective
stash/pop, unrelated untracked preservation, branch create/switch, discard, new
reversing commit, conflicted pop retention and revert Abort, plus the prior corpus.
Fourteen focused Flutter/editor/recovery checks pass, including host-ID-only hunk
selection and disabling stale hunk actions. Flutter analysis is clean. Fixtures
set their own line-ending policy; the user's Git configuration is unchanged.
Normal packaging, saved renders and process/resource qualification follow in 17.6.

## Source Control 17.3 — reviewed stage, unstage and commit

Whole-file actions use a host-owned two-minute, single-use review of exact saved
paths, HEAD/index/configuration and changed bytes. Rename pairs stay together;
unstaging an unborn repository retains source files. Commit reviews all staged
paths, the configured author and bounded staged diff, with hooks enabled. A hook
failure retains the commit message and existing index; changed HEAD after an
uncertain result is reported for inspection instead of repeating the commit.
Unsaved/pending selected editors refuse the action, mutations exclude conflicting
file edits and agent tasks, and Git work uses bounded owned threads rather than
the chat runtime's blocking pool. Open review/commit drafts protect final Close.

Five native Source Control checks pass, including selective staging, unborn
unstage, real hook refusal, fresh reviewed commit and stale-byte single-use refusal.
Thirteen focused Flutter/editor/recovery checks pass for explicit Cancel, hook
failure draft retention, fresh retry, compact comparisons and file recovery.
No fixture mutates the user's repository. Full packaging/resource and native UX
qualification remain for 17.6; hunk/local/remote operations follow in this batch.

## Source Control 17.2 — saved Git diff tabs and history

Changes/Staged open retained read-only diff tabs with explicit Index, saved working
tree, HEAD or exact commit/parent labels. Inline and side-by-side views remain
separate from unsaved editor buffers. History pins HEAD, loads 30 commits per page,
and opens non-merge commit file comparisons; root commits compare with empty text.
Home change receipts link to the current saved Git comparison without relabeling
the receipt as repository truth. Refresh failures retain the prior tab.

The native corpus verifies working/index/HEAD/root-commit distinctions, Unicode,
malformed paths, binary/long-line refusal and stale saved-byte refusal. Three
native Source Control checks and eleven focused Flutter/receipt checks pass,
including compact inline/side-by-side and failed diff Retry. Physical native input,
larger paging cost and normal release renders remain batch qualification work.

## Source Control 17.1 — lazy repository status

Source Control follows the selected Home conversation without a project picker.
It discovers the enclosing canonical worktree and Git directory, showing exact
NUL-delimited Changes/Staged/rename/conflict filenames. Repository owners retain
their own results; late A responses cannot replace B. Missing Git/repository and
failed refresh have explicit recovery while the last useful status remains.
Git runs asynchronously outside the global chat lock, with two global jobs,
per-worktree serialization, eight retained results, bounded binary output and
owned process-tree cancellation. Shutdown cancels and reaps owned jobs.

Two native checks pass for real unborn repositories, Unicode/rename records,
invalid encoding, incomplete records, missing repository and cancellation. Four
Flutter checks pass for A/B response ordering, failed Refresh/recovery, exact
paths/no picker, retained Home/rail and Experimental settings. Full release,
resource and visual integration follows 17.6. Diffs and mutation are later bricks
in this batch. Initial limits are frozen in [Source Control](design/source-control.md).

## Workspace 16.6 — integrated recovery and qualification

Home, the compact rail, project-bound Folders and file-only splits are implemented.
Quit reviews dirty files and owned tasks, checkpoints chat/file drafts and layouts,
then stops owned runs and closes the bridge. Failed Save/checkpoint keeps the app
open. A failed OS window destruction can retry without repeating shutdown. Theme
changes propagate to retained conversation owners. Recovery checkpoints run once
after settled edits, avoiding the earlier checkpoint/flush timer loop.

Native shutdown closes admission before releasing active slots, so queued work
cannot begin during exit. Queue saturation is checked before preparing a run
journal. Refresh/rebase/save enforce resident-text bounds. Completed physical
Save/Rename/Delete actions report explicit recovery warnings if the separate
private checkpoint fails; they do not misreport the filesystem action as failed.
Native installation/Restore refuses dirty editor shadows before consuming review;
the UI also checks local unsynchronized edits and other conversation owners, and
checkpoints their chat drafts before restart.

The public C ABI fixture passes UTF-8 BOM/CRLF round trips, stale Save refusal,
comparison-backed rebase, equal-name project isolation, binary/long-line previews,
checkpoint failure after Save/Rename/Delete, explicit recovery Retry and fresh-process
dirty recovery with source bytes unchanged. Configured Qwen3.5-2B reads one saved
file through one approved read and reports the saved marker while the different
unsaved marker remains private: 2.64 seconds, 65 reported output tokens, 512-token
request bound. All 40 original profile tables were unchanged by that isolated case.
This is one bounded real-model case, not general provider reliability.

The final full Flutter suite passes 278 checks and analysis reports no issues.
Rust bridge passes 121 checks, filesystem/store suites pass 93, and strict
Clippy passes. Native release captures exercise light/dark four groups and compact
group access. They use real native fonts and production modules with synthetic
files; edits are programmatic, not physical keyboard/IME/accessibility checks.

Resource evidence and normal release handoff are recorded in
[workspace qualification](qualification/workspace-editor.md). Initial diagnostic
typing probes included a fifth unmounted controller; one also overlapped Flutter
tests. Their failures remain and do not qualify the intended four-view workflow.
The per-view whole-source highlighting worker produced a large memory spike;
the named 16.6 resource revision limits highlighting to 64 KiB, with a plain-text
status above that size. File/edit/memory bounds and 32/250 ms targets are unchanged.
Save as replacement of an existing file, physical input, low-end/other-OS checks,
long-duration keep-awake and earlier idle-CPU/model gaps remain open. Later pages
stay unavailable honestly; this batch does not implement milestone 17.

Normal release handoff: the maintained build of source commit `8d7dfa2` succeeded
and the maintained launch opened the `main` entry with a Windows-visible window
(PID 15748). All 40 original profile tables retain identical contents; only
`experimental_preferences` and `workspace_editor_state` were added. The normal
bundle also passes the public editor/recovery fixture, including fresh-process
dirty recovery. Window presence does not qualify foreground or physical input.

## Workspace 16.5 — file splits and project layout recovery

File tabs drag to reorder, move between groups and split on editor edges; menus
and Ctrl+Backslash/Ctrl+W/Ctrl+Tab offer alternatives. Four groups share document
text and undo, with independent cursor/selection/scroll. Clean preview tabs can
be replaced; editing or double-clicking keeps a tab. Closing a split merges its
tabs into a remaining group. Only closing the last dirty view offers Save,
Discard or Keep editing, and failed Save retains it. Close controls sit outside
the drag recognizer; accepted drops explicitly clear overlays even when their
source widget disappears.

Versioned layouts retain paths, group structure, proportions and view state per
project, bounded to four documents and 8 KiB UTF-8 metadata. Opening requests are
serialized; dirty/pending buffers cannot be evicted. Clean dormant buffers reload
without losing retained tab identities. Invalid saved layouts are validated before
any stored path opens, with a safe single group and private recovery retained.
Failed layout writes show Retry without clearing unrelated recovery errors.
Compact layouts and narrow sub-splits expose group selectors rather than hiding
unreachable editors; widening restores the retained splits.

Eighteen focused Flutter checks pass: seven new layout/workspace checks cover
reorder, cancelled drag, edge drop, four-group bounds, independent view recovery,
dirty close cancellation/failed Save, corrupt metadata, failed persistence and
A→B→A with clean eviction. Light/dark four-group and 420×480 saved renders use
loaded Windows/Material fonts and were inspected. These are widget renders and
pointer simulations, not physical native-input qualification. Full integration,
normal release and bounded saved-file model checks follow in 16.6.

## Workspace 16.4 — editor and disk reconciliation

The pinned re_editor 0.10.0 adapter uses one document-owned, bounded undo history and
independent view controllers. It provides line numbers, bounded syntax highlighting
for common source formats, indentation/clipboard shortcuts, Find/Replace, line
navigation, relative-path Quick open and explicit Save (Autosave Off). Text edits
send one acknowledged delta at a time and coalesce later work. Failed sync retains
the local draft and requires explicit Retry sync; oversized paste restores the
previous buffer before adding history. Saving keeps undo when text is unchanged.

Private recovery checkpoints run after 500 ms of settled editing, rather than
serializing the file on every keystroke. Clean disk changes refresh on returning
to Folders, explicit Refresh and a project run's completion. Dirty changes retain
the editor buffer and show Compare; Reload disk and Keep my edits validate the
reviewed disk digest again. A stale comparison cannot overwrite a newer disk base.
File actions remain blocked during pending Save; failures keep usable text.

Attach selection explicitly chooses an idle conversation in this project and an
editor version/range, explains provider sharing and stores an immutable text
snapshot with saved-byte provenance. Editing alone never attaches or shares it.
Native tests verify clean/dirty disk changes, stale attachment refusal and unchanged
source bytes; the attachment remains unchanged after later edits.

Eleven focused Flutter checks pass, including four editor checks for shared undo,
selection independence, bounded history, coalesced transactions, sync/save failure,
oversized paste and visible Find. Four native editor checks pass. Source-level
checks do not qualify physical keyboard, IME, clipboard or native accessibility;
integrated highlighting/resource measurements follow in 16.6.

## Workspace 16.3 — scoped tree and document persistence

Folders now binds lazily to the Home conversation's root and retains open document
owners through A/B navigation. Directory pages return at most 200 entries, expand
only on demand and bound excluded-entry work to 1,000 entries per request. The
20,000-entry fixture exercises the first two pages plus independent child expansion;
it does not claim a complete recursive scan. Secret/VCS paths and aliases use the
existing directory capability checks. New file, rename and explicit Delete refuse
dirty documents and existing replacement targets. Human file operations do not grant
model access or expose unsaved text to model tools.

Native document IDs and UTF-16 versioned deltas share one host shadow per file, reject
surrogate splits/overlap/stale versions and acknowledge without whole-file JSON.
The C ABI admits bounded editor payloads while retaining the earlier 128 KiB limit
for other commands. Files preserve strict UTF-8/BOM/uniform LF or CRLF; larger,
binary, invalid-UTF-8, mixed-ending and long-line files are read-only previews.
Explicit Save stages and flushes a replacement, rechecks the disk digest and
permissions immediately before publication, and retains the document on failure.
An unrelated OS writer can still race the final check/publication interval; this
is not a filesystem compare-and-swap guarantee.

Private checkpoints use an additive profile-local SQLite table. Restart recovery
retains dirty snapshots without changing source files or dropping other unopened
recovery records. Recovery checkpoint failure keeps edits open. The next editor brick
adds idle checkpoint scheduling and comparison-backed reconciliation controls.
Six focused Rust checks pass for byte round trips, stale/deleted files, Unicode
transactions, scope isolation, no-replacement file actions, tree paging and restart
recovery; seven focused Flutter checks pass for lazy expansion, coalesced binding,
retained document identity and missing-root recovery. Physical native input and
normal-app performance remain batch qualification work.

The bridge-only suite passes all 119 checks. One concurrent run with filesystem
fixtures returned the request-timeout cause in a one-second stalled-delivery test
where the fixture expected delivery-timeout wording; its isolated rerun and the
bridge-only suite pass unchanged. This timing sensitivity remains recorded;
no production deadline or recovery default was relaxed.

## Workspace 16.2 — compact navigation and Experimental preferences

The normal entry point now uses the shared host, a 64-pixel icon rail and bottom
Settings. Home retains its agent view; the title-bar sidebar-layout button toggles
the adjacent panel. Mouse dragging can hide it; keyboard arrows, Home and semantic
resize actions are available. Conversation selection supplies developer-page context
without a project dropdown. Failed folder selection keeps the prior owner. Later
Scheduled/Source Control/Terminal pages explicitly state their availability.

Experimental preferences default to Multiple Window On and Windows keep-awake Off.
They persist with revision checks in an additive table, preserving schema version 33
for native rollback compatibility. No detached window is advertised as implemented.
The Windows option owns display/system power requests, releases them on disable or
shutdown, and retains the prior preference on storage failure. Startup activation
failure remains visible in Settings; it never disables history. These requests do
not override manual locks, security policy or Modern Standby restrictions; see
[Microsoft's power request API](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-powersetrequest).

Seventeen focused Flutter checks pass, including compact navigation, drag hiding,
retained draft/project, settings defaults and failed saves. Flutter analysis is
clean after style fixes. All 171 bridge/store tests pass, including an actual Windows
request enable/disable/shutdown check against an isolated in-memory store. This
does not establish long-duration idle locking behavior or physical accessibility.
Normal build and saved-render checks follow the complete milestone integration.

## Workspace 16.1 — shared run ownership foundation

The Rust host now owns ID-addressed primary runs with two active slots, a four-item
waiting queue and one active owner per canonical working folder. Preparation pins
run context; queued work starts its execution clock after admission. Cancelled
waiters release their reservation without executing, and Stop/poll/shutdown address
the intended owners. Maintenance/settings mutation remains exclusive. Tool decision
IDs are host-generated and unique even when providers reuse call IDs.

Flutter's AppHost retains independent conversation controllers/drafts on one bridge,
uses globally distinct client run IDs and closes the bridge only through its original
owner. Failed selection retains the visible conversation. Three host tests plus
four existing workspace checks pass; all 114 Rust bridge tests pass, including
two-root admission, same-root waiting, queue exhaustion/wakeup, wrong-ID isolation
and shutdown. Focused Flutter analysis is clean. The host is wired to the new UI
in 16.2; normal build and real-model batch checks follow integration. These fixtures
do not establish real-provider concurrency or native UX qualification.

## Workspace 16.0 — proceed decision and frozen initial limits

The implementation proceeds one milestone per batch after reviewing the old slow samples. Host contention is plausible, not proven; the old reports
remain without blocking adoption alone. The new native four-view 1 MiB editing
case passes 40 replacements at 19.558 ms typing frame p95, with shared undo/redo.
The [initial document contract](design/editor-documents.md) freezes byte/line/history,
resident-document, delta and explicit-save bounds; the former 5 MiB proposal is not
adopted. Complete milestone 16's bricks in order with separate commits before 17.
Production save, native physical input/IME/accessibility and integrated resources
remain later-brick checks. No normal UI/profile setting has changed in this step.

## Workspace/editor 16.0 — opening diagnosis follow-up

The original isolated native loop passes on rerun: four-view post-corpus opening
136.258 ms, typing frame p95 at most 4.620 ms, existing provisional memory checks
passing. A focused native sequence repeats large/long-line/ordinary opening with
four shared views 12 times per fresh process. Both runs pass all 24 ordinary opens,
maxima 144.306/136.098 ms below the unchanged 250 ms target. Separate phase timings
make the loop more specific; they do not explain the historical 389.306 ms failure.
The six controller/widget checks pass again. No performance fix is claimed.

Final B layout amendments remain recorded below; its HTML is unchanged. Public
follow-up source/reports are retained on the isolated branch in
[qualification](qualification/workspace-editor.md). No production frontend/profile,
provider/model setting or OS preference was changed; no real model call occurred.
16.0 remains in progress: intermittent historical results, physical input/IME,
encoding-safe Save and production protocol/resource qualification stay open.

## Workspace 16.0 — final B layout decision (contract only)

The finalized design uses developer-pages **B**: a top-left title-bar sidebar-layout
toggle, drag-resize the page panel to hidden, no developer-page project dropdown,
selected project derived from the chosen Home conversation, and direct Terminal
entry at that project root or OS home. Existing terminals keep their CWD on return.
Settings remains at the bottom of the compact rail; Home remains the agent view.
The prototype is intentionally unchanged in the earlier plan. The page-direction
review gate is closed; the amendments are recorded for implementation and are not
claimed as tested runtime behavior. Editor adoption/16.0 completion remains open.

## Workspace/editor 16.0 — compact rail prototype and shared-view trial

The new page study preserves agent Home and file-only Folders splits. Following
The documented interface design, its primary navigation is a compact icon
rail with hover/focus names, the wider page panel beside it, and Settings fixed at
the bottom. Browser checks pass draft retention, project-separated buffers, actual
drag/edge split, invalid-drop recovery, duplicate buffers, large/binary previews,
folder cancel and compact/minimum-height access. Terminal, Scheduled and
Experimental controls are explicitly synthetic; no real backend effects occur.
Light/dark/wide/compact saved renders were inspected with `view_image`.

Six isolated editor controller/widget checks pass, including focused Ctrl+Z/Ctrl+F,
independent views and one bounded shared undo owner. Two native original-controller
reruns pass typing at 7.210/6.749 ms without a performance fix; the earlier 94 ms
failure's cause is still unknown. Shared-adapter one/two/four-view typing passes
both native runs. On the repeat, **four-view ordinary reopening after large files
takes 389.306 ms**, failing the unchanged 250 ms target. Provisional memory ceilings
set before that repeat pass; they qualify only the scratch sizing cases, not idle
or normal-app costs. See [exact evidence and remaining gates](qualification/workspace-editor.md).

**Status:** prototype and partial feasibility evidence, not completed 16.0 or a
selected production dependency. Revised layout review, inconsistent native opening,
physical input/IME, encoding-safe Save and production protocol/resource bounds
remain open. No production app source/dependency/profile/model setting changed;
no real model call or normal desktop relaunch occurred. The isolated frontend was
built/launched through the maintained Python machinery and exits after its probes.
Documentation validation passes for 86 documents and 353 local links; whitespace
checks pass. Public prototype source and native reports are retained on the
separate branch named in the qualification report.

## Roadmap revision — separate developer pages and useful continuity (planning only)

The roadmap revision reshaped milestones 16–19 and requested future memory, scheduling and
companionship work, implemented one brick at a time. The revised
[roadmap](ROADMAP.md) and [workspace contract](design/developer-workspace.md)
preserve Home's agent UI, define the six sidebar entries with Settings at the
bottom, and keep file and terminal splits separate. Source Control includes real
Git diffs; selected-project/root and no-project/home behavior are explicit.
Experimental multi-window has a bounded investigation and single-window fallback;
Windows keep-awake describes its actual platform limits.

The [new contract](design/memory-scheduling-companionship.md) assigns 21–24 to
automatic source-backed useful memory, chat-created schedules, opt-in in-app
companionship and optional closed-UI execution. the product decisions select one
Memory switch with inspect/forget, and companion hours plus a daily cap. Manual
schedule creation UI is deferred indefinitely. [Primary memory research](research/memory-foundations.md)
distinguishes scientific findings from proposed engineering behavior; selective
cue-led recall does not imply a complete human recording of sensations.

Repository guidance and handoffs now match the documented interface design:
no desktop build/launch/visual check after every task; focused checks appropriate
to the change, saved renders with `view_image` for appearance, computer-use only
when interaction/native behavior needs it. Document/link checks and whitespace
checks pass for this revision. No runtime code, dependency, schema, real profile
or selected model changed, and no model request or desktop relaunch was made.

**Open gates:** the revised 16.0 page prototype still needs review; existing editor
typing/shortcut/resource failures remain open. Numerical defaults for new memory,
schedule and companion behavior are proposals to freeze before implementation.
The user's observed lack of memory updates is not diagnosed against their live
profile. Milestones 16–19 and 21–24 are not delivered by this documentation work.
Earlier acceptance entries below are historical evidence and retain their limits.

## Workspace/editor 16.0 — agent UI retained in a file-only split comparison

The product design calls for a new prototype preserving the current agent view and limiting
split tabs to files. Three synthetic variants compare files beside the agent,
below it, and file focus with the familiar agent on the right. The browser retains
agent drafts and unsaved file edits across variant changes and Files close/reopen.
Actual file-tab move and edge split work; a drop onto the agent is cancelled with
work retained. Wide/narrow rendering was inspected. These are prototype behaviors;
the production app and existing editor qualification gaps are unchanged. No layout
has been selected, no model call was made and 16.0 remains in progress. Captured
source and precise limits are in [qualification](qualification/workspace-editor.md).

## Workspace/editor 16.0 — revised prototype; editor adoption held

The implementation plan resumed milestones 16–19, one milestone batch at a time. The revised
synthetic prototype follows their VS Code screenshot: draggable tab strips,
close/overflow controls, breadcrumbs and no pane action bars. Settings stays at
the bottom. Actual pointer move, edge split and invalid-drop cancellation retain
the four project-bound views and drafts; stale synthetic Save offers comparison
and Keep editing. Revised user review is pending.

The separate Windows release editor trial passes three controller probes and
ordinary native character input. It opens the sampled text within the 250 ms
target, but typing after the large-file sequence has frame p95 94.340 ms; disabling
large-file highlighting still gives 94.056 ms against the unchanged 32 ms target.
Ordinary-only typing (19.252 ms) and TextField (3.626 ms) are separate controls.
Ctrl+Z/Find integration and duplicate-view ownership remain unqualified. Single
editor memory measurements do not qualify four-pane cost or sustained idle CPU.

Public prototype/trial sources are captured on `codex/prototype-workspace-16`;
production source/dependencies are unchanged. 16.0 remains in progress and no
16.1–16.5 runtime brick is complete. Detailed numerical evidence, scope and open
gates are in [workspace/editor qualification](qualification/workspace-editor.md).
These checks use no model calls. The original profile audit reports all 38
original tables unchanged; existing empty repair migrations remain accepted.

The specification now records the requested Experimental toggles: Multiple Window
defaults On, activated only after backend qualification; Windows keep-awake
defaults Off, with actual platform limitations disclosed. They are planned for
the named runtime bricks, not delivered by the prototype.

## Harness self-repair 20.6 — separately authorized desktop Restore passes

The user separately approved **Restore & restart** for the installed DeepSeek
candidate's isolated profile. The actual Native repairs review dispatched Restore,
closed the idle candidate and started the retained previous normal version. The
protected launcher verifies the previous source and usable startup; its receipt
records Restored. The candidate and lazy helper exited normally.

Read-only comparison confirms every table equals the state saved after the
post-install draft edit. The completed real DeepSeek result, read receipt and
newer unsent draft are visible in the restored normal app. No task is replayed,
no additional model call is made and the original profile remains unchanged.
Detailed identities and audit evidence are in the
[qualification report](qualification/harness-self-repair.md).

This completes the demonstrated Windows Rust repair cycle for one bounded,
file-scoped DeepSeek-authored parser candidate: independent fault reproduction,
candidate/regression trials, reviewed release build, real candidate task,
separately user-authorized desktop installation and Restore, plus idle Close.
Earlier trusted startup-failure, interruption, non-improvement and replay-refusal
checks remain separate evidence. General repair competence, busy/last-keystroke
Close, OS containment, persistent shortcut routing, low-end/other-OS hosts and
the full native visual matrix remain open. The original maintained provider code
is unchanged; the candidate is retained only in its managed repair workspace.

## Harness self-repair 20.6 — authorized desktop installation and idle Close pass

the development plan includes installation/restart of the existing verified DeepSeek candidate
in its isolated profile. The actual Native repairs screen prepared the exact
bundle/profile review and its **Install & restart** action saved drafts, shut down
the old app and started the candidate through the protected launcher. The receipt
records Applied with the expected candidate source and healthy normal startup.
The old app and lazy launcher exited without fixture force-termination.

Read-only comparison verifies every isolated table unchanged across installation,
including the completed real DeepSeek turn and retained draft. A native keystroke
then adds one public character to that unsent draft; storage acknowledges it and
only `session_drafts` changes. The candidate's normal window Close exits, and
reopening the retained installed normal bundle shows the updated draft and saved
answer. All table contents compare equal after Close/reopen. The original app's
process, all original-profile tables, provider and selected settings stay unchanged.
This verification makes zero additional model calls and changes no product code.

This closes the approved desktop installation/graceful restart and idle Close
sample. It does not qualify closing busy work or an unacknowledged last keystroke.
Direct desktop **Restore & restart** still needs its own user review; trusted
Restore/rollback evidence remains valid but does not substitute for that action.
General repair competence, containment, persistent shortcut routing and the full
native/platform visual matrix remain open. See the updated
[qualification report](qualification/harness-self-repair.md).

## Harness self-repair 20.6 — one configured DeepSeek repair case passes

Configured DeepSeek V4.1 Flash inspected the running source and authored its own
provider argument-normalization patch, without receiving replacement code. The
file-scoped prompt supplied a public example of the existing double-encoded JSON
fault. This is a model-authored candidate, unlike 20.5's fixed patch. The malformed
provider responses remain deterministic independent test inputs; this fault was
not observed spontaneously on the configured DeepSeek connection.

Separate inspected proposal, native test and release-build reviews pass. The
frozen baseline passes two cases and fails the actual wrapped-object case; the
model candidate passes all three, including malformed/non-object/recursive input
refusal and exhausted output publishing no partial call. All 354 unchanged Rust
workspace regressions pass. The resulting release bundle is ready.

The protected trusted handoff starts the actual normal candidate shell. A real
DeepSeek task then runs through the candidate bridge, reads a public JSON file
exactly once, correctly reports its status and boolean with quoted/Chinese content,
and changes no project files. Saved implementation identity matches the candidate.
Restore and subsequent startup-failure rollback preserve that completed model turn
and a newer acknowledged draft. Interrupted waiting-helper and used-intent replay
refusals also pass. These handoffs use fixed test intents and exact owned idle
process cleanup; direct user approval and graceful product Close remain manual
gates. No model controls installation and no repair is applied to the original profile.

14 model calls total: 8 diagnosis/proposal, 2 test, 2 build, 2 live task. Stage wall
times are 120.13 / 357.97 / 215.55 / 5.20 seconds, including review and native
execution waits. Output is bounded to 4096 tokens per repair-phase call and 1024
per live-task call, with 60-second model inactivity and unchanged native command
limits. Provider-reported totals are 229,991 input / 13,513 output / 243,504 total
tokens, including 16,768 cached input tokens. This cold profile retains 5359.04 MiB.

Two out-of-file inspections were denied by the test's narrow review scope; the
model recovered and retained its proposal. The live-task driver's initial status
assertion expected `completed` instead of the actual successful `read`. Its false
report is retained; an independent audit checks the completed saved turn, exact
candidate source, correct values and preservation before marking this case passed.
No additional model retry or production-default change was needed.

The original profile's 38 tables, selected model, settings and paused work remain
unchanged. This closes one bounded model-authored repair case, not general repair
competence, direct user installation/Close, containment, persistent shortcut
routing or broader platform/visual acceptance. The normal app is rebuilt and
visibly reopened after this verification task; detailed evidence and exclusions
are in the [qualification report](qualification/harness-self-repair.md).

## Harness self-repair 20.5 — Windows Rust implementation and packaged fixture qualification

Separate build, installation and Restore reviews are implemented for qualified
Rust provider/command-outcome/task-budget changes. Protected policy, credentials,
schema, tests, build scripts, evaluator, launcher and Flutter shell cannot change
through this path. Full access does not skip review. A lazy bundled launcher
retains complete versions, verifies source/schema/process/history during blocked
startup and preserves current history for a later Restore. No task is replayed.

The fixed provider-parser repair reproduces double-encoded JSON arguments: two
baseline cases pass and the actual fault fails; all three candidate cases pass,
including malformed/non-object/recursive arguments and exhausted-output refusal.
The final packaged rerun passes 354 unchanged workspace regressions. This is trusted
fixture evidence, not a model-authored repair or general model reliability.

Windows verification exposed a linker path-length failure, an omitted existing MCP
test helper, and a candidate DLL flush using a read-only handle. The respective
trial/build receipts withheld qualification or installation and retained evidence.
Compiler outputs now use fresh compact owned directories, the matching source
includes that helper, and staging flushes a writable handle. No time/output limits
were raised. A driver receipt-field error stopped before build approval and was
corrected separately. Final packaged qualification takes 320.25 seconds across
three bounded trial commands, followed by a 182.89-second reviewed release build.
Stale/declined builds execute nothing; a Python bridge host cannot install.

The actual normal shell and protected launcher pass installation, later Restore
with an acknowledged post-install draft preserved, interrupted waiting-helper
refusal without stopping the old app, and recovery to the previous normal version
after the candidate cannot publish healthy startup. Every table's contents remain
unchanged across handoffs except the deliberately acknowledged new draft. Used
and interrupted intents cannot replay. The fixture constructs trusted test intents
and stops only its recorded idle processes; direct user approval and graceful
product Close remain separate acceptance gates. A transient Windows receipt-read
sharing error was corrected in the bounded driver polling; recovery itself had
already completed. The complete rerun then passed.

Protocol checks cover complete-bundle tampering, protected launcher changes,
stale process creation identity, a changed/interrupted intent refusing replay,
consistent WAL/draft snapshots and Windows DLL staging. Compact light/dark widget
checks retain stale-review errors and unsaved Settings drafts without restart.
354 Rust library tests pass (one ignored), Clippy passes, and all 256 Flutter
tests and analysis pass. Saved build failures expose their actual reason before
review details, and condensed qualified trials do not invent missing counts.
The normal dark native empty-state page was inspected, Escape closed Settings,
and the recovered normal shell's Native repairs page shows the recovery reason,
Ready to review and separate installation/Restore actions without submitting them.
Its acknowledged post-install draft is visible after rollback,
and all 38 original profile tables remain unchanged. Remaining gates include
direct user approval and graceful close, full native visual matrix, real-model
repair, other OS/lower-end hosts, containment and persistent shortcut routing.
Maintained launch still opens the maintained bundle; the repaired normal app
runs from its retained versioned directory. The fixed test profile retains about
5.24 GiB including cold compiler outputs; each complete bundle is 62.33 MiB.
The bounded routine Qwen probe performs two model calls in 14.58 seconds and
completes inspection but does not identify the build tool correctly. It performs
no build/install and does not establish model repair reliability. See the
[qualification report](qualification/harness-self-repair.md).

## Harness self-repair 20.4 — reviewed Rust evaluator, bounded qualification

2026-10-06: Native execution has its own exact review, including candidate diff,
frozen reproduction source and bounded Cargo sequence. A complete baseline test
failure, candidate reproduction success and unchanged library regression passes
are all required; baseline passing, compilation failure or incomplete evidence
withhold improvement. Existing tests cannot be weakened through this evaluator.
Schema 33 retains completed phases and unfinished/stopped evidence without replay.
Task Full access cannot authorize this operation. No installation is delivered.

342 Rust library tests pass (one ignored), Clippy passes, and 252 Flutter tests
and analysis pass. Real installed-Cargo fixtures exercise faulty baseline → fixed
candidate → regressions and Stop after a child starts, preserving source/marker.
Source/test drift, truncated evidence and stale/cross-chat persistence are covered.
Normal packaged dispatch passes, including declined/stale refusal and a passing
baseline withholding improvement without candidate execution. Configured DeepSeek
passes this bounded live path in two calls/23.53 seconds; Qwen remains unqualified
after choosing denied source inspections instead of a trial. No actual repair or
installation is claimed. All 38 original profile tables remain unchanged; only
two empty repair tables were added. The initial foreground check failed after a
fresh-window retry and showed the Windows background.
The final normal build at `576187b`, packaged fixture rerun and original-profile
preservation check pass. Follow-up foreground inspection of the updated normal
app passes with the original paused Mario chat and selected DeepSeek model intact;
no task continuation or approval was submitted. The native trial-card matrix,
actual repair, resources and other platforms remain open; see
[evidence](qualification/harness-self-repair.md).
The earlier normal 20.2 app's foreground check now passes after desktop unlock.

## Harness self-repair 20.2 — native proposal stage implemented

2026-10-06: Ordinary working chats can retain a matching-source repair separately
from the user's project, review an exact diff and inspect its saved candidate.
Each step needs independent review even under Full access. Source/revision drift,
another chat, cancellation and changed saved files preserve prior work and give
an inspection/fresh-proposal next step. Schema 32 adds scoped repair snapshots.
No candidate build, native execution, installation or improvement is claimed.

336 Rust library tests pass (one ignored), Clippy passes, and 250 Flutter tests
and analysis pass. The normal-release fixture exercises two reviewed writes,
stale/cross-chat refusals and Stop preservation in seven requests. Bounded live
DeepSeek passes five inspected/reviewed steps; routine Qwen remains unqualified
after a driver parsing failure. The design adopts native-first preparation;
At that stage 20.4–20.6 were not implemented. The later entries above supersede
that snapshot; 20.3's broader Wasm hook remains unadopted.
Final normal build/launch and packaged fixture pass. All 38 original profile
tables are unchanged; schema 32 adds only an empty repair table. The launcher
reports the normal window present. Foreground inspection awaits desktop unlock; see
[evidence and limits](qualification/harness-self-repair.md).

## Harness self-repair 20.1 — implemented, visual/platform gates remain

2026-10-06: Build-pinned compressed source now supports bounded manifest listing,
identifier/literal search and ranged reads up to 2048 lines with identities and
continuation. Actual delivery retains the existing 16 KiB tool ceiling. Failed
tool-model streams retain host-authored numeric telemetry; older/missing evidence
stays unknown and stale source is refused without touching the user's project.

332 Rust library tests pass (one ignored), Clippy passes, and the existing 248
Flutter tests/analysis pass. The normal native fixture passes reviewed navigation
with both identities plus failed partial-call privacy/no-dispatch recovery.
Configured DeepSeek passes three live inspections with correct final identity;
two Qwen probes remain unqualified. Original profile tables are unchanged.
Normal build/launch passed; foreground visual inspection awaits desktop unlock.
General model, resource and other-platform qualification remains open. Later
bricks were not implemented by 20.1; this does not claim actual self-repair.
See [evidence and limits](qualification/harness-self-repair.md).

## Useful Mario task follow-up — command path exercised, completion unqualified

2026-10-06: A bounded configured DeepSeek V4.1 Flash native-host run used the
exact Mario instruction and actual project. Eight model calls and eight reviewed
operations completed under Automatic counts, including two literal Node
validation commands with exit 0 and untruncated captures. The first exercised
4,830 game frames without runtime errors; the heuristic bot lost at tile 70 of
196. The ten-minute driver bound stopped at a proposed bot-test edit before a
final answer. No game overwrite or pending edit occurred; original HTML is
byte-identical and both new validation scripts remain. This qualifies the useful
native command path, not complete gameplay or end-to-end task completion.
Original settings/history remain and the normal app was visibly restored.
See [details and remaining gaps](qualification/task-loop.md).

## AI follow-up — automatic task execution

2026-10-06: Fresh task settings use Automatic counts rather than four model calls
and four tools. Useful work continues to a saved resource checkpoint (64 model
calls / 128 tool attempts per segment). Explicit saved counts remain authoritative.
Repeated failed requests pause before a third identical attempt, or after six
consecutive failures; successful work clears the streak. Human approval still
has no model timeout. Declined prepared plans release their review slots.

Rust workspace library suites, Clippy, 248 Flutter tests, analysis and Python
compilation PASS. Native integration PASS: 128 completed reads, 33 model calls,
saved checkpoint and all 514 activity entries, plus missing-file recovery,
explicit caps, Stop and no incomplete-call dispatch. Explicit-limit continuation
save/restore PASS across independent processes. Live Qwen PASS: six reviewed reads
and seven model calls with correct final markers. Four DeepSeek large-command
probes remain unqualified: model size/content variation caused driver denial or
adapter refusal. Thinking updates were observed; no command execution pass is
claimed. Full native Mario validation remains a gap.

The normal release was rebuilt and visibly inspected with the original profile;
Automatic settings and blank optional fields fit the native dialog. No settings
were saved during visual inspection. Self-repair 20 and CI 8.4 remain
undelivered/deferred. See [qualification and boundaries](qualification/task-loop.md).

## AI follow-up — response recovery and approval waits

2026-10-06: The inherited whole-task timeout was reproduced in the normal app.
Model inactivity and optional active-task time are now separate; human review
has no model timeout. Blank output delegates to the provider, existing numeric
settings remain, and one useful truncated response can continue within the
current task budget without executing partial calls. Repeated/empty reasoning-only
truncation pauses with saved progress. Thinking previews retain the latest
provider text and elapsed activity remains visible through public output.

254 core/provider/bridge/store tests and 247 Flutter tests PASS; Clippy and
analysis PASS. The command suite has 12 passing tests, including long quoted
validation scripts and single/aggregate overflow recovery. Scoped-settings integration PASS. Native exact-prompt DeepSeek
runs in the old chat still exhausted their explicit output setting; the fresh
provider-default follow-up created a 14,604-byte standalone HTML game and kept
approval usable for a measured 190-second wait. Syntax, headed browser rendering,
movement, jump and restart PASS. The run paused at its four-call budget after
two validation commands were rejected by the old 1-KiB argument bound. That bound
and misleading error are fixed, but native command-validation completion remains
unqualified. See [qualification](qualification/response-recovery.md).
Do not infer task completion from a fixture pass or approval card. Milestone 20
self-repair and platform CI 8.4 remain undelivered/deferred respectively.

## AI follow-up — fragmented reasoning and visible activity

2026-10-06: The earlier event-count fix retained a cumulative 2 MiB wire guard.
A real-adapter regression reproduced its failure with small decoded reasoning and
HTML surrounded by repeated metadata. The same fixture now passes. The guard
bounds traffic without decoded progress; individual event/field/tool limits,
deadlines, Stop and incomplete-call refusal remain. A decoder scan/drain change
also removes the observed fragmented-batch overhead.

Reasoning-only requests now show phase and elapsed time, with the deadline on
hover; private reasoning and incomplete arguments remain hidden. Widget and native
delayed-provider checks cover thinking/tool preparation, stale activity, Stop and
restored draft without dispatch. Core/provider/bridge: 192 tests PASS; Flutter:
245 PASS; analysis: no issues. Oversized/no-progress traffic and malformed/truncated
calls still refuse execution. There is no automatic replay or settings increase.

DeepSeek V4.1 Flash completed the exact HTML game prompt with provider-default
reasoning and 32,768 output tokens: 13,686-byte standalone file, JavaScript syntax,
browser rendering and basic input PASS. Activity continued about once a second.
GLM-5.3-Flash default reasoning timed out at 180 seconds with no file/tool call;
activity remained visible. This failure remains an acceptance gap.
A separate GLM Low run completed at the same output allowance and produced a
9,076-byte standalone HTML file with valid JavaScript; this is not a default pass.

General native self-repair was not implemented by the recovery-hint mod milestone.
Matching-source navigation, managed patches, independent trials and installation
with rollback are planned separately as milestone 20. The 120-line per-read limit
is not expanded by this fix. See [evidence and boundaries](qualification/reasoning-streams.md).
The final normal desktop release was rebuilt through Python, launched with the
original profile, brought to the foreground and visually inspected. All 38
original settings/history tables are unchanged; the app is left visibly open.

## AI fixes — streamed generation, image chat and self-diagnosis

2026-10-06: A deterministic HTML tool call with more than 4096 small SSE events
reproduced the reported frame-limit banner. Removing that event-count guard lets
the same bounded response complete; wire/event/field/timeout limits and incomplete
call refusal remain. Protocol-only reasoning continuation and an explicit GLM Low
option support reasoning-model tool follow-up without publishing private reasoning.

Bounded live DeepSeek V4.1 Flash and GLM-5.3-Flash tasks completed the exact HTML
game prompt, created standalone artifacts and passed JavaScript syntax checks.
Both games rendered in an isolated browser and responded to keyboard input.
The passing settings were DeepSeek thinking off / 8192 output tokens and GLM Low /
16384 tokens, each with 180-second responses and 240-second task deadlines.
Earlier provider-default output-limit/timeout failures remain recorded; these are
not provider-default or general coding reliability passes. Defaults and original
provider settings were preserved, with no hidden retry or command execution.

Harness inventory now includes bounded, chat-scoped recent failure summaries and
bundled streaming/settings/recovery/attachment code. A live DeepSeek follow-up to
an injected oversized event inspected inventory and exact streaming source,
identified the 256-KiB bound and explained that no incomplete call ran. Privacy
fixtures cover other-chat isolation and private prompt/plugin-error omission.

Native Windows Paint copy → Shift+Insert/right-Control+V → draft thumbnail PASS.
The delayed local provider received one image while its thumbnail was already in
the pending user bubble; saved preview remained after completion and restart.
The helper's known injected left-Control timing issue remains separate from
physical-keyboard acceptance. Windows BMP conversion, empty-text clipboard,
malformed/oversized bitmap, delayed session reservation, failed-send restoration,
retained earlier attachments and temporary cleanup checks PASS.

Core/provider/bridge suites total 190 passed tests; all 244 Flutter tests PASS,
analysis reports no issues, qualification scripts compile and four packaging
integrity/refusal tests PASS. The Windows launcher preflight also PASSes missing
and empty clipboard-DLL refusal and recovery while retaining synthetic history.
The normal release was rebuilt, restored to the original profile, visibly
inspected and left open; all 38 original history/settings tables are unchanged.
The native clipboard plugin is included in generated
Flutter notices and the Windows runtime contract. Full separate notice collection
still refuses missing `wasmi` notice text; portable release qualification is not
claimed. Other OS clipboard behavior, physical IME and general model reliability
remain open. See [full evidence and boundaries](qualification/stream-and-image-recovery.md).

## Maintenance follow-up — approved project evidence after command review

2026-10-06: A native bridge regression reproduced lost approved manifest evidence
after a failed check paused the task. Post-chat project knowledge now retains
eligible completed observations for command-review pauses. Other pause categories,
execution approval and continuation behavior remain unchanged.

The new native fixture PASS covers the actual post-run path, failed independent
trial recovery, output-limit exclusion and denied-source exclusion. Saved replies
and baseline skills remain. All 92 bridge tests PASS. Bounded DeepSeek live tasks
preserved unrelated files; the final task learned the current declaration and
completed four independent trials. Both versions passed both cases: a tie, so no
activation. Useful automatic improvement and live regression restoration remain
unqualified; [learning evidence](qualification/learning-followup.md) records the
earlier output limit, exact allowances and reproduction. The normal release was
rebuilt and visibly inspected with all 37 original profile tables unchanged.

## Maintenance follow-up — computer-use contract qualification

2026-10-06: Clarified that an observation takes only `operation`, with no unused
defaults. Native-control tests PASS for extra capture, false and null fields,
pure observe, stale/consumed capture and revoked access. The 92-test bridge suite
and two frozen-corpus checks PASS. Runtime validation and consent were unchanged.

Frozen live cases still FAIL: Qwen first supplied invalid observe fields, then a
fresh run with clearer guidance failed the initial screenshot-reading check.
DeepSeek entered the exact draft and obtained a fresh post-input image, but hit
the output limit before Save. No repeated input, hidden retry or criterion change.
See [computer-use qualification](qualification/computer-use.md). These results
do not establish dependable general computer use. The normal release was rebuilt,
visibly launched and inspected; original provider/history tables are unchanged.

## Maintenance follow-up — exact automatic response preferences

2026-10-06: A bounded literal extractor saves simple explicit response-style
preferences without a second model request. It keeps the exact words and excludes
temporary acknowledgement requests. Unrecognized wording uses the existing
reviewed model path; this is not general semantic memory qualification. Storage
still enforces scope, source, cancellation, revisions and manual protection.

Two independent Qwen3.5-2B live checks PASS for exact preference capture and an
explicit correction. Core grammar checks PASS for exact qualifiers and ambiguous,
temporary, quoted, secret-like and permission-changing inputs. The native fixture
save/restart checks PASS for scope, duplicate/correction handling, manual protection,
Stop, timeout, low context, malformed/oversized/denied extraction and retained
conversation/provenance. The stale schema-version assertion was replaced by an
integrity check; restart checks still verify the actual saved policy and memory.
The normal release was built, launched and inspected with disposable data.

## Maintenance follow-up — Python qualification and optional browser setup

2026-10-06: Retired the remaining PowerShell qualification wrappers. Python now
runs the pinned optional adapter installer, diagnostic smoke/history/restart
entries and experimental mod resource probes. The normal resource helpers replace
old desktop-shell measurements. The Flutter diagnostic entry must match the
requested case; diagnostic checks cannot count as a normal app handoff.

Three installer tests PASS: pinned atomic publication/offline repeat, failed or
timed-out download cleanup, and retention of an unrelated existing destination.
Two diagnostic-runner tests PASS cover failed-report retention and deadline
cleanup of its own process. A real pinned npm download and offline repeat PASS.
The normal native bridge browser fixture passes using that newly installed
adapter: three fresh input reviews, stale action without replay, capture recovery,
step exhaustion, Stop during slow navigation, file-tool recovery and delegated
read/browser flow. Eleven active descendants were sampled; zero remained after
completion. Sampling is not peak memory or low-end qualification.

Python mod probes also ran: three point-in-time memory samples each for Wasm and
Rhai plus the own-file worker boundary demonstration. These research probes are
not production mod or desktop qualification. Historical connection/history/smoke
diagnostic entries were consolidated, not independently rerun in this follow-up.
The normal desktop release remains visibly available with the user's history and
provider preserved. Optional browser setup still needs Node and Edge/Chrome;
an in-app installer and general model reliability remain open.

## Maintenance follow-up — truthful build and resource checks

2026-10-06: Four Python launcher tests PASS, including refusal to hand off a
diagnostic or subsequently changed binary as normal. A failed build invalidates
its entry identity. Explicit owned replacement requires a record. The normal
Windows app was rebuilt and visibly launched; all 37 provider/history tables
remain unchanged. The actual idle app widget test PASS confirms no continuously
scheduled frame after settling. Shared process-time sampling replaces duplicated
measurement code and distinguishes normal from diagnostic builds.

[Idle investigation](qualification/idle-resources.md) retains the native CPU
results and limits: normal empty-profile samples still use 3.12–4.37% of one core,
while a minimal diagnostic Flutter window uses 1.72–1.87%. No CPU fix or low-end
qualification is claimed. Temporary diagnostic source was restored exactly.

## Maintenance follow-up — Python desktop workflow and retired shells

2026-10-06: Python `scripts/desktop.py build` and `launch` replace the former
PowerShell/Node build and launch wrappers. The build retains path remapping,
private debug symbols, temporary registrant alias/restoration, generated plugin
junctions and asset-stamp invalidation. Windows CMake fallback built the normal
release without modifying Flutter or requiring Developer Mode. Launch preserves
the selected data directory and strips diagnostic environment; owned replacement
verifies PID, executable and process creation time through the terminating handle.
An already exited recorded preview is handled without touching another process.

Removed the retired Iced and Tauri/Svelte implementations, comparison/native-shell
scripts, root web tooling and obsolete automatic platform workflow. 8.4 remains
deferred until explicit user instruction. The Rust lockfile pruned 384 package
entries with no added package/version identities; retained versions are unchanged.
Browser tooling and Flutter's maintained platform runners remain.

Exercised: three Python checks PASS (failed-build exact config restoration, stale
PID identity refusal, Unicode/space junction reuse and unexpected-directory
preservation), two regression-runner checks PASS, 306 Rust workspace tests PASS
with one native-vault check explicitly ignored. Ten logo outputs match; document
check PASS for 71 documents / 236 local links. The normal Windows release built
and was launched through Python, then visually inspected in the native dark
conversation. All 37 provider/history tables remained unchanged. macOS/Linux
execution and platform CI are not claimed. The rest of the authorized follow-up
is tracked in [remaining work](qualification/remaining-work.md).

## Brick 14.1 — selected-window observation

The normal Windows release bridge passes `scripts/test-desktop-observation.py`
save/reopen in separate disposable processes. The native helper captures only the
synthetic selected window: a 1024 × 626 JPEG, 40,806 bytes. Listing, capture and
preview issue no provider request. A local SSE fixture verifies the exact text
tool-result/image projection, retained typed image receipts, original chat model,
and old text/paired image history after restart. Unsupported vision, missing or
oversized image bytes, closed target, and a completed answer that skips the image
are refused without fallback. The draft/evidence remains. Five bounded local
requests exercise success, a skipped tool and output exhaustion; partial progress
survives and ordinary Continue/checkpoint recovery cannot escalate into file tools
or replay provider work. Unit checks cover image-context refusal before the first
request, wrong-session/digest checks and cache exhaustion/removal recovery.

A deliberately stalled helper exercises the real normal-release process owner:
the five-second deadline returned in 5.69 seconds and Stop in 0.02 seconds. Both
kill/reap their helper and preserve local evidence. No credentials or user history
are passed to the helper. No resident observation worker is added.

Bounded configured-provider probes use only the disposable synthetic application
and isolated test data. Final Qwen/Qwen3.5-2B (2.09 seconds) and DeepSeek V4.1 Flash
(4.12 seconds) each used two calls, received the image and correctly identified
the button “Save note” and status “Ready”, with a 1024-token output limit and
unchanged ordinary model selection. Earlier Qwen probes returned an incomplete
identification or skipped the image entirely. The provider reported completion
for those probes, so output-token exhaustion cannot be claimed as their cause.
The new evidence guard rejects the skipped-image case; it does not prove that a
model which reads the image describes it accurately. Provider token usage was
unavailable, not zero. This is one small passing observation per model, not
consistent general desktop-task qualification.

Checks pass 225 Rust tests across eight suites and 216 Flutter tests; final focused
Computer use tests, Rust Clippy and Flutter analysis pass. Widget checks cover
compact light/dark layouts, unsupported image models, a closed-window error
surviving refresh, retained draft and no implicit retry. Packaging fixtures pass
four tests and the payload contract includes the helper and existing window
plugins. Actual dependency-notice qualification remains blocked by missing
upstream notice text for the existing Wasmi runtime; the optional installed browser
adapter also lies outside the strict portable payload. No new distributable archive
is qualified by this brick.

The normal app was rebuilt and launched (PID 31560). Original preferences/history
remain unchanged across 37 table digests. Native visual verification succeeded
on retry in the running normal app: the dark 1127 × 813 window shows Computer use
inside unified Settings with readable controls and a reachable fixed footer.
Refreshing and selecting the disposable application, capturing locally and
scrolling expose the readable image, snapshot dimensions/DPI and separate model
and sharing controls. Closing that application and attempting another capture
shows an actionable error pinned above the scroll area while preserving the
previous preview. No Analyze action or provider request was made in this visual
check. The owned fixture and its single local test capture were removed afterward;
the app stays open and the 37-table preservation check passes. The earlier desktop
activation failure was a verification-tool limitation, resolved on retry. Native
light-theme, other-size and input/accessibility qualification remain separate gaps.

Desktop click/type, accessibility observations, other platforms, protected/locked
desktops, DPI/movement corpus, and representative low-end resource measurements
remain open under 14.2–14.4. Images survive chat deletion: remove them while their
chat exists; orphan-cache cleanup currently needs manual local maintenance.
The initial cache has no automatic expiry or secure erasure. General browser
screenshots still are not projected to the model. Nothing qualifies arbitrary
applications, implicit sharing or unattended desktop actions.

## Milestone 13.4 — integrated recovery and qualification boundary

The normal Windows release bridge passes `scripts/test-mods.py` save/reopen in
separate processes with disposable data and a local SSE provider. Exercised
fixed repair/activation, a paused output-limited task's pinned hint in live and
durable events, active-run mutation refusal, unsupported capabilities/imports/
card/schema refusal, source-drift health failure, rollback with all 32 audit slots
occupied, and ordinary chat afterward. Two bounded incomplete drafts exercise
output-token exhaustion and Stop after the first delta. Both preserve editable
source, keep the baseline, and issue no implicit retry. Restart retains messages,
preferences, partial source and quarantine, clears an interrupted activation and
does not replay provider work. The fixture made four local requests, no external
requests and used no real credentials or private transcripts.

Pressure checks found and fixed two recovery defects: Stop used to lose received
source; full audit capacity used to block restore. A UTF-8-safe prefix now survives
oversized deltas, and a separate bounded latest recovery receipt lets restore and
restart reconciliation proceed without deleting old audit entries. A failed
receipt transaction still retains the old active pointer. Eight retained
candidates exhaust the catalog explicitly; existing active guidance remains
usable and can still be restored. No limits were silently increased.
Malformed mod state in one project also no longer blocks pending activation
recovery in another project; the broken project's inspector keeps its local error.

Broader Rust checks pass 224 tests across ten suites, followed by the added
candidate-capacity and malformed-project recovery cases (two additional passes).
The final Flutter suite passes
214 tests, including three Mods cases covering stale refresh, output truncation
and compact 800 × 600 recovery with an empty stopped request. Rust Clippy and
Flutter analysis pass. The documentation check passes 65 Markdown documents and
215 local links; external links were not rechecked. The existing runtime cases
also exercise runaway bytecode/fuel and undeclared resource refusal.

Screenshots of the normal release at 1127 × 813 show the Settings → Harness mods
panel in system-dark and light themes, with no overflow and footer actions
visible. System theme was restored. Compact recovery has widget evidence only;
the native resize attempt retained the original dimensions and does not count
as compact native acceptance. All 37 noninternal data tables match the fresh
pre-verification hashes, including history, provider/model choices and policies.
The final normal app was rebuilt and visibly relaunched.

Resource observations on this Windows development device: the release bridge DLL
is 15,398,912 bytes and runner EXE 91,648 bytes, excluding Flutter assets/runtime.
During settings interaction one sample reached 340.57 MiB working set / 348.84 MiB
private bytes. Later samples were 252.55–274.03 / 258.26–282.84 MiB respectively;
one quiet one-second interval showed zero CPU-time increase at timer resolution.
There was one app process, no resident mod helper and no owned browser worker.
These are unpaired observations, not proof of a memory improvement, cold-start
latency or steady-state target compliance. The provisional 150 MiB working-set
target is not met by these samples; low-end acceptance stays open. The selected
runtime's compile/call/probe measurements are recorded under 13.1.

Milestone 13's implemented envelope is stateless recovery hints, bounded drafting,
fixed independent trials, transactional activation and reversible quarantine.
The live DeepSeek repair below passed; Qwen drafting reliability and milestone
12's useful automatic skill-improvement gate remain open. General agent/context/
UI replacement, stateful migrations, native self-replacement, sustained large
history/catalog/browser-plus-child pressure, cold-start timing, physical IME/
screen readers, representative low-end hardware and macOS/Linux execution remain
unqualified. No OS sandbox, general self-improvement or Codex-equivalent ability
is claimed. Milestone 14 remains planned and was not started in this batch.

## Milestone 13.3 — model-authored recovery mods

Settings → Harness mods exposes scoped source/tests, activation, change history,
bounded model drafting, a themed literal card and deliberate restore/quarantine.
Automatic ABI 1 activation is a separate switch, off by default. Only mod source,
not chats, facts, credentials or private roots, goes into the drafting request.
Six fixed host-owned categories compare baseline/candidate at equal fuel. Source
and manifest drift invalidates activation. Unsupported hooks/capabilities/UI
manifests refuse. Paused working tasks execute a pinned source snapshot and retain
the resulting recovery hint in run events/trajectory; hints never run actions.

Two draft fixtures pass for valid, truncated, malformed, oversized and stopped
responses. Two widget fixtures pass for stale-test recovery and partial draft
retention; the seven existing Settings tests pass. Partial generation is persisted
for reopening and remains editable. No JSON repair/retry loop or raised default
was added. Broader batch checks: 222 Rust tests across ten suites and 213 Flutter
tests pass; Clippy passes (final UI analysis recorded at batch exit).

Bounded live probes in disposable data used the configured endpoint and synthetic
mod source. First Qwen returned non-improvement (232 total tokens); DeepSeek
returned invalid WAT (1,209 total, 941 reasoning). Both were withheld. A clearer
source-inspection prompt, unchanged 1,024-token/30-second allowances, was then
tested explicitly once per model: Qwen again tied (233 total tokens, 1.11 seconds),
remaining rejected. DeepSeek repaired a synthetic conditional context-category
fault in 3.23 seconds (237 input / 368 output, 341 reasoning, 605 total). All six
candidate cases passed against a five-pass baseline; opted-in automatic activation
occurred, and explicit restore quarantined it and restored the retained source.
The synthetic baseline was injected for this test, not a real previous learned
activation. The user's selected model/configuration/history were untouched.

This demonstrates a small executable recovery-hint repair, not general autonomous
task improvement. Qwen drafting reliability remains unaccepted. Milestone 12's
broader automatic skill-improvement acceptance remains open independently.

## Milestone 13.2 — activation and durable recovery

Four focused host/store tests pass: strict improvement, safe-boundary activation,
exact source/manifest identity checks, stale revision refusal, retained snapshot
pinning, restore/quarantine, rejected non-improvement, interrupted intent
reconciliation, and failed receipt persistence. A SQLite trigger permits the
intent but refuses the pointer/receipt update; the old pointer remains and
restart recovery clears the pending intent before an explicit successful retry.
Migration 31 adds only project_mods; old history/settings remain readable.
Registry descriptors identify restricted Wasm separately from compiled/MCP code.
Mod state errors stay local and do not prevent registry/chat preparation.

ABI 1 has no persistent mod state or dependencies: schema/capability expansion
refuses; there is no migration path to execute. No credential or private task
text is passed to bytecode. An active run blocks activation through the existing
host coordinator. Wider stateful migration/OS qualification remains excluded.

## Milestone 13.1 — runtime decision

Selected Wasmi 0.46.0 for the narrow stateless recovery-hint ABI in
[the runtime contract](design/executable-mods.md). Two runtime tests pass:
fixed classification plus import/memory/table/output refusal, and runaway/Stop.
Release comparison: Wasmi compile 980 µs, 1,000 fresh-store calls 1,536 µs;
Rhai compile 1,321 µs, 1,000 calls 457 µs. These are scalar microbenchmarks,
not task reliability or representative low-end measurements. Standalone probes
are 2,536,448 and 2,960,896 bytes respectively. First three sampled working sets
were Wasmi 15.23/14.98/14.98 MiB and Rhai 15.53/15.20/15.30 MiB. Repeat sampling
varied during startup (5.92–15.23 and 5.93–15.22 MiB); no idle-memory target is
claimed from these snapshots. Each owns one process and exits without helpers.
The worker reads its owned sentinel directly, confirming lack of containment.
Rhai runaway/file-access probes pass, but it remains development-only.

The normal Windows desktop release was rebuilt and launched with existing
configuration/history. No live model call is required to select the engine.
Other OS execution, native interpreter vulnerabilities, general hooks and
representative device/resource targets remain unqualified. No OS sandbox is
claimed; generated Wasm has no imports/WASI and no native fallback.

## Milestone 12 — native visual verification follow-up

Retried on the unlocked Windows desktop with screenshots of the normal release
at 1127 × 813 in dark theme. Project knowledge, the Skills library, Tool trials
and Learning panels fit within the unified Settings dialog. The disabled trial
comparison and learning inspection actions correctly show their prerequisites.
Scrolling exposed a crowded floating command label; added 16 pixels of spacing,
rebuilt and visually confirmed the separation in the user's normal workspace.
The three focused Learning tests and Flutter analysis passed.

A disposable database in the same normal release supplied a synthetic active
learning event and retained independent provider trial receipts. Visually checked
the expanded event, wrapped receipt text, restore command preview, Cancel,
confirmed restore/quarantine and the persistent footer recovery notice. Cancel
kept the active event; restore changed it to quarantined and removed its restore
action. This verifies the native manual recovery UI, not automatic activation or
real-workflow improvement. No model request was made during this follow-up.

Stopped only the owned fixture process and visibly relaunched the user's normal
desktop app. All 36 noninternal user-data tables match the fresh before-check
baseline, including provider choices, history and the user's current policies.
Native dark-theme settings/manual recovery inspection is now complete. Native
light/compact layouts, automatic live improvement/restoration, low-end and other
platform qualification remain open; existing fixture checks do not close them.

## Brick 12.4 — learning history and regression recovery

Implemented durable reasons/snapshots across originating project chats,
pause/disable, deliberate restore preview, atomic restore/quarantine and one
bounded matching regression comparison. Quarantine prevents automatic
reactivation. A manual change produces a conflict without overriding it;
provider/output/step-limit incidents do not justify restoration. Unfinished
reflection/monitoring becomes interrupted on startup and is not replayed.
Recovery notices remain visible in the settings footer while inspecting history.

Rust core/store/bridge full suites: 177 tests passed; final six focused
adaptation tests include an added in-flight Stop test and passed. Clippy with
warnings denied passed. Fixtures exercise qualifying repair followed by
independent regression restoration, harmful-candidate refusal/quarantine,
unrelated limits, incomplete checks/no retry, stale choices, retention failure
atomicity, restart and preserved manual edits. Flutter: **211 tests passed**;
analysis clean. Restore confirmation/Cancel, conflict notices, preserved drafts
and Refresh without replay are covered.

DeepSeek V4.1 Flash used a synthetic activation/incident in a disposable DB and
real tool comparisons: **38.63 seconds**, all four phases complete and passing
for both versions. It correctly retained the active skill (no regression).
An explicit native API restore then quarantined the candidate. This verifies
the comparison/preservation/manual restore flow, not live improvement or
automatic regression restoration. Qwen's incomplete 12.3 probe is retained above.

Normal final release built and launched with a Dolores window handle. Original
34 user-data tables remain unchanged; schema 30/37 tables and new learning
tables empty in the user's DB. Windows was locked at that checkpoint, leaving
native visual verification pending; the follow-up above records the retry.
Launch/window presence alone is not screenshot acceptance.
A post-launch snapshot measured 246.6 MiB working set and 286.3 MiB private
memory; no low-end/idle performance claim follows from it. Learning adds no
resident worker, new child process or idle model requests.

Milestone 12 implementation batch is delivered. The required real-workflow
improvement/automatic restoration gate remains **unaccepted**; keep reviewed
updates as the dependable path and experimental automatic policy off by default.
Executable containment, broad skill competence, automatic live behavior,
low-end and other-platform qualification remain open. The follow-up above
records the native settings/manual recovery review.

## Brick 12.3 — targeted project skill adaptation

Implemented opt-in, deduplicated local attribution and the exact host
config-check-v1 command repair. Independent complete stored trials, current
knowledge, policy and unchanged project snapshot gate activation. Global and
unknown instruction changes remain reviewed; no permission or budget expansion.
Baseline snapshots and reasons remain local. Interrupted evidence is ineligible.

Rust core/store/bridge: 173 tests passed; Clippy with warnings denied passed.
Fixtures demonstrate one repair/activation and deduplication, harmful attempts,
false claims/output limits and stale knowledge/manual edits/paused policies.
Flutter: 11 focused learning/trial/settings tests and analysis passed. Failed
saves retain the workflow draft; Refresh does not replay it; compact themes and
Side chat prerequisites pass.

Qwen3.5-2B reflection probe used a synthetic approved-source incident in a
disposable database and real provider trials: 21.47 seconds, three retained
phases, incomplete regression and no activation. It does not demonstrate a real
workflow improvement. Automatic activation acceptance remains **unaccepted**;
the narrow experimental opt-in and broader reviewed updates remain available.
Normal release built and launched; 34 original user-data tables unchanged,
schema 30/37 tables. Visual desktop verification remains pending while the
Windows session is locked/unavailable.

## Brick 12.2 — independent tool trials

Implemented the fixed config-command-v1 suite with in-memory file tools and
simulated checks. Original and independent Unicode/nested-value regression
cases share the same model/profile and fixed 1024-token, five-model/eight-tool,
30-second allowances. Local append-only receipts retain files, calls and usage.
False claims, evaluator/outside writes, output limits, Stop, stale saves,
criterion tampering and failed storage are exercised; incomplete or unsaved
work does not qualify. Restart retains baseline and partial evidence.

Core/store/bridge: 169 tests passed before the final receipt-grade validation;
final focused three trial tests passed. Flutter: nine focused trial/settings
tests and analysis passed; failed preparation/Refresh preserve the candidate.
Final Rust Clippy passed with warnings denied.

DeepSeek V4.1 Flash completed the first pair (both passed), then reached an
output limit in regression: 32.98 seconds, three retained phases, non-qualifying.
Qwen3.5-2B retained an unsuccessful baseline and incomplete candidate in 5.75
seconds, also non-qualifying. Neither run activated anything. A small fixture
pass is not real-model reliability; these live attempts establish preservation
and refusal under pressure, not an improved real workflow.

Normal release built/launched; original user-data tables remain unchanged
(schema 29, 36 tables). Windows became locked before screenshot verification;
visual launch inspection remains pending. No OS process/network trials or
general skill qualification is claimed.

## Brick 12.1 — scoped knowledge

Rust core/store/bridge: 166 tests passed; focused observer tests and Clippy
passed after the final Windows path correction. Flutter: 206 tests passed,
analysis clean. Fixtures exercise opt-in/scoping, deduplication, missing/changed
sources and folders, expiry, protected corrections, secret/oversized rejection,
atomic stale/failed saves and restart. Compact light/dark editor fixtures retain
drafts after a failed save; Refresh does not replay the save.

A bounded Qwen3.5-2B live read learned one package script declaration and the
next request included exactly one knowledge fact without another tool call.
It named the exact command; calls took 1.72 and 0.86 seconds. Feedback sharing
stayed off. The first live attempt exposed normal versus extended Windows path
comparison; it was fixed and the probe repeated. This establishes this simple
evidence-reuse flow, not broader model reliability or tested script execution.

Normal release rebuilt and visibly launched; all 34 original user-data tables
were unchanged and the new table empty (schema 28, 35 tables). No policy or
selected model changed in the user's data. Knowledge is deterministic and
bounded; feedback eligibility does not upload notes yet. Other-platform and
physical keyboard/IME acceptance remain open.

## Brick 11.3 — on-demand browser use

Implemented the [owned browser contract](design/browser-use.md): optional pinned
Playwright-core 1.63.0 with installed Edge/Chrome and Node, one fresh visible
parent-run browser, origin-restricted networking, bounded state/refs/actions and
local JPEG previews. Settings → Browser exposes setup and capture location
without starting a service. Click/fill/press require fresh review even under
Full access; Stop/end/deadline releases owned resources without undoing remote
effects. Children do not receive the browser tool.

- Rust: **166 passed** across browser/core/store/bridge suites; Clippy with
  warnings denied passed. Proposal tampering, single use, argument/receipt
  bounds, malformed receipts, cancellation and dropped-owner descendant cleanup
  are exercised. The full tool catalog now fits SQLite run snapshots; an
  excessive catalog remains atomically rejected.
- Flutter: **204 full-suite tests passed** before the final setup wording/cache
  guidance changes; the final **three focused browser tests** and analysis
  passed. Compact Light/Dark fixtures cover missing-adapter setup, stale and
  uncertain status, local preview failure/retry and unsafe capture IDs. Failed
  preview retains receipts and repeats only the local image read.
- Real browser adapter: **two tests passed**, using a local reversible form.
  Open/fill/click and a 9,388-byte JPEG pass. Expired tokens and changed DOM do
  not dispatch actions; disabled controls time out with uncertainty/fresh state;
  closed tabs recover through explicit open; popups are closed. UTF-8 truncation
  marks partial evidence, and 128 captures refuse another save while retaining
  page inspection. Browser cleanup is exercised after the fixture.
- Normal-release host fixtures exercise fresh Full-access input review, stale
  no-replay, saved capture preview/missing-image recovery, model-step limits,
  Stop during navigation, subsequent file/browser use and a parent combining
  scoped child file work with browser input. The browser stops within the
  fixture deadline, completed evidence remains and file tools stay usable.
- Final active Windows sample: **one Node worker, 11 owned processes, 661.16 MiB
  aggregate working set**. Shared pages are counted; this is neither unique
  memory nor a peak or representative-device benchmark. After run completion,
  owned workers/processes return to zero. Static page reading remains the
  lighter option; no low-end performance claim follows from this sample.

- Live configured provider, isolated data and unchanged six-call/six-tool
  allowance: **DeepSeek V4.1 Flash passed** open → fill → click → close, using
  current tokens/refs and verifying the returned synthetic value in **13.48
  seconds / five model calls**. Reported usage was retained; workspace files
  stayed unchanged. This is one small reversible task, not general reliability.
- **Qwen3.5-2B failed the same flow twice** (21.69 and 27.81 seconds), omitting
  the required state token on input. No input action was dispatched; the run
  paused at its six-call allowance with evidence preserved. The first attempt
  exposed a misleading generic file-denial error; browser-specific argument
  guidance and field descriptions now replace it. A regression fixture verifies
  correction/review without private error leakage. Qwen still omitted tokens
  after that correction; reliable Qwen browser use remains an acceptance gap.

- Final normal Windows release build passed and was visibly launched. Its Dark
  screenshot was inspected with Settings → Browser: runtime-installed status,
  bounded setup/storage details and Refresh fit the unified panel. All **34
  local table digests** matched the pre-task snapshot, including provider,
  selected model, appearance and history. With the normal app open on Browser
  settings, a final sample found **zero owned browser workers/processes**.

Broad real-site/authentication behavior, browser adapter
packaging, physical input, macOS/Linux lifetime behavior and general model
reliability remain unaccepted. Same-origin browser networking is not the
DNS-pinned page reader or an OS sandbox. Screenshots stay local rather than
providing model vision. Cross-origin dependencies and logins can make real sites
unusable under the initial policy.

## Milestone 11 — exit review

The optional subagent, search and browser tools are implemented within existing
host authority, shared usage and cancellation contracts. A combined parent/
child/browser fixture passes, and browser failure/Stop leaves ordinary file
work usable. Search has a separate bounded live Qwen pass. Active browser
process cost and idle cleanup are measured above. These small checks do not
close broad research/delegation/website, low-end or other-platform gaps.
Milestone 12 should build scoped evidence and independent evaluations on these
contracts; it must not treat tool transport success as task success.

## Sidebar resizing and title-strip spacing

The title strip now has one full-width theme color. Sidebar content starts 16
pixels higher, and brand-to-New-chat spacing is reduced by eight pixels. A
9-pixel target around the right divider resizes the sidebar from 220–360 pixels,
with a 480-pixel conversation allowance. Its accent is limited to an active
drag; release or cancellation restores the border color, including retained
keyboard focus. Width is local to the current window, not persisted on restart.

- Full Flutter suite: **200 passed** before the final accent correction; the
  final **four focused sidebar tests passed**, including the added release/cancel
  regression. Final Flutter analysis and normal Windows release build passed.
- Basic and edge fixtures cover widening/shrinking, double-click and keyboard
  reset, extreme drags, editable-draft preservation, 760-pixel window clamping,
  compact drawer use and restoring the preferred width on expansion. Minimum
  width exposed section-label overflow; flexible ellipsis now preserves the
  chevron. Caption tests verify the drag area contains no split-color segment.
- The final normal release was visibly launched and its Dark screenshot
  inspected: unified title strip, higher single brand and normal divider.
  Clicking the divider and Right changed its width from 252 to 268 pixels;
  Home restored 252, with conversation content and the normal border color
  intact. All **34 local table digests** remained unchanged, including provider,
  selected model, appearance and history. No model request was made.

Native automation's pointer drag did not change the sidebar width. Widget
pointer behavior and native keyboard resizing are verified; physical pointer
feel, spoken screen-reader feedback and macOS/Linux execution remain acceptance
gaps. The previously recorded native window move/resize gaps also remain open.
This refinement does not advance the browser-use brick.

## Themed desktop title strip

The normal desktop entry point hides native title branding and adds a themed
32-pixel drag strip with Windows/Linux Minimize, Maximize/Restore and Close.
macOS keeps native traffic lights. The sidebar remains the only visible brand;
the OS/taskbar title and infinity icon remain Dolores.

- Flutter full suite: **197 passed**, with final focused **6 passed** after
  adding the drag dispatch assertion. Flutter analysis and normal Windows
  release build passed. No Rust/provider/tool behavior changed and no model
  request was necessary for this window-only task.
- Basic flow: the actual normal release window was inspected in Light, including
  its light title strip and a single brand. A live Dark switch also changed the
  strip/controls with Settings open; the original Light preference was restored.
  Double-click maximize and the custom Restore control passed, with bounds
  changing from a restored window to the monitor work area and back.
- Native right-click window menu passed. Its keyboard Move/Size commands changed
  window position and width without losing conversation content. Custom Minimize
  produced a confirmed minimized window, then activation restored the content.
  Custom Close worked with Settings open; relaunch retained the Light preference.
  All **34 local table digests** matched the pre-check snapshot after restoring
  the theme, including provider settings and history.
- Edge cases: failed/pending window actions preserve an editable draft, prevent
  duplicate operations and offer explicit retry. Initialization refusal restores
  the native frame. Real-app widgets retain the rich draft through theme changes,
  and caption controls remain usable with Settings open. Compact 420×480 controls,
  external maximize events, listener cleanup and macOS traffic-light spacing pass.

Pointer drag/border gestures were attempted through native UI automation but did
not change geometry; native Move/Size and widget drag dispatch are verified, while
physical pointer drag/resize and Windows snap-hover behavior remain acceptance
gaps. macOS/Linux native execution, multi-monitor DPI and screen-reader testing
also remain open. This UI follow-up does not advance the browser-use brick.

## Settings remaster — before browser use

One bottom-left Settings entry replaces the separate Memory, Model connection
and Request settings actions. The shared window contains Appearance, Models,
Personalization, Memory, Web search, External tools, Skills, Permissions and
Task limits. Models combines connection/capabilities/context, response profiles
and project/chat generation overrides. Chat actions uses an ellipsis and retains
conversation actions. System/Light/Dark theme selection is acknowledged locally,
applies immediately and persists through restart; default remains System.

- Rust storage/bridge: **109 tests passed**; warnings-denied workspace Clippy
  and Flutter analysis passed. Flutter's full suite passed **191 tests** after
  final notification/removed-model hardening; its focused 19 tests also passed.
- Basic flow: the single entry opens without creating a session or provider
  request, Models controls and Web search are reachable, and explicit Close
  retains the composer draft without saving editor drafts. Retained editors are
  created only on first visit; the app theme subscribes only to appearance changes.
- Failure cases: pending theme save blocks Close/Escape and navigation;
  injected failed save retains the prior theme and enables explicit retry.
  A stale endpoint or removed enabled model retains response edits, refuses a
  save, avoids dropdown assertion failure and offers Reload response settings.
  Category switches retain unsaved connection/web edits. Personalization reset
  preserves generation and task overrides; scoped revision checks remain intact.
- Normal release bridge in isolated data passed theme save, invalid-value refusal
  and process-restart restoration without model traffic or credentials.
  Additive schema **27** adds appearance storage; original **33 table digests**
  remained unchanged, with **34 tables** after migration.
- Wide dark Appearance, wide light Models and compact dark Models screenshots
  were rendered with desktop fonts and inspected. The normal release desktop
  was visibly launched; Settings, Models, Appearance and Web search were inspected
  in its real window without saving the user's configuration. A cached accessibility
  index initially failed; refreshed screenshot navigation recovered the check.

Native theme persistence was tested with isolated data; actual theme transitions
in the open window are covered by widgets rather than changing the user's saved
theme. No new live-model task was needed for this local settings/navigation change.
Physical keyboard/IME/screen-reader, macOS/Linux and low-end resource qualification
remain open. This detour does not implement brick 11.3 browser use.

## Brick 11.2 — default and configurable web research

Working chats now have default unauthenticated Mwmbl search and public HTTPS
page reads. Global settings can disable both or choose Brave/public SearXNG.
Side chats/children remain without web tools. Exact queries/URLs, primary source
links, partial coverage, retrieval receipts and manual recovery use existing
approvals, budgets and run history. No idle service or automatic paid fallback.

- Full Rust workspace: **257 passed, one intentionally ignored**, 22 suites;
  final five web tests also pass after adding Brave/SearXNG parsing coverage.
  Flutter: **184 passed**. Analysis and warnings-denied workspace Clippy are clean.
- Transport fixtures exercise quota refusal, redirect-to-local refusal, declared
  and chunked oversize bodies, malformed formats, Unicode excerpt continuation
  and stale offsets. Deep/excessive markup refuses with simpler-source guidance,
  preventing quadratic ancestry extraction under the byte ceiling. The ordinary
  live adapter probe passes again with this final guard. URL tests reject
  encoded/private/tunnel/authentication cases.
  Cancellation before dispatch prevents credential reads; missing Brave keys
  point to settings/default search. Hostile HTML instructions remain untrusted
  quoted data, with hidden/script content removed; this is not general semantic
  prompt-injection immunity, particularly under Full access.
- Configuration tests cover no-setup default, disabled/side catalogues, stale
  saves, locked vaults, atomic database failure preserving the existing key,
  replacement cleanup and restart. Compact light/dark widgets cover fixed
  controls, provider selection, failed-save draft retention/Refresh/manual retry,
  literal approval sharing and readable partial/error receipts. The compact test
  exposed and fixed a dropdown overflow.
- Live public adapter probe returned five sources and an 8192-byte partial official
  Rust page excerpt. This machine's resolver supplied a synthetic reserved IP;
  validated public DNS resolution plus IP pinning fixed connectivity without
  accepting local/private addresses. Mixed/private/malformed DNS answers refuse.
- Isolated normal-release **Qwen3.5-2B** research completed in **13.89 seconds**:
  approved search and official page read both completed, three ownership rules
  were summarized with the official URL, three model calls had reported usage,
  no task pause/error and no working-file changes. Credentials/settings/private
  transcripts stayed out of tracked artifacts; ordinary model selection stayed
  unchanged. This establishes one bounded task, not broad search reliability.
- On the final normal bundle, repeating the same Qwen task completed both web
  reads but attempted unrelated `search_text` and reached the four-call step
  limit without the requested answer/citation. The saved paused reply and source
  receipts survived and files remained unchanged. **Qwen consistency is not
  accepted**; increasing the default budget would not address that behavior.
  The identical bounded task with **DeepSeek V4.1 Flash** completed in **8.19
  seconds**, with both web tools, official citation, two model calls, reported
  usage and unchanged files. This is narrow passing evidence, not a guarantee.

Normal Windows release built and opened visibly (initial PID 36176); a native screenshot
confirmed the normal window. The final guarded bundle was rebuilt and relaunched
(PID 7288); its returned window is titled Dolores, but final activation/screenshot
recovery reports `failed to activate captured window`, so final native visibility
is not independently confirmed. All **32 original table digests** match the fresh
baseline; schema 26 adds one empty web configuration table (33 total). Inspector
visual/recovery acceptance is widget-based: the desktop helper's stale element
cache prevented completing native menu interaction. No native inspector visual
pass is claimed. Idle samples show one app process, 266.77 MiB working set,
268.4 MiB private bytes and 0% of one core over two intervals. These are this
development machine/current history, without a matched pre-brick comparison;
no low-end performance benefit is claimed.

The locked HTML parser introduces transitive dependency notices. Four omitted
upstream license files now have exact-version reviewed notice fallbacks; the
collector retains verified MPL corresponding source. Collection succeeds for
244 components without a network request, and four packaging integrity/recovery
tests pass. The normal portable bundle also passes archive integrity/private-path
checks (final bundle: 21 files, 17080954-byte ZIP), retained only in ignored local output.
No installer/signing/public release was made.

Remaining: Brave needs a user's service key for live quota/auth acceptance;
custom SearXNG needs a JSON-enabled public instance. Default index coverage and
availability vary. Browser/login/redirect/PDF/private networks are excluded.
Other OS/physical input/accessibility/representative-device acceptance remains
open. Credential cleanup across simultaneous vault/database failure is best
effort, as documented. Browser use is the next brick.

## User bubble color

User bubbles use the same soft background and normal text color as highlighted
sidebar sessions, replacing the overly bright primary-color fill.
The 12 focused message-frame/reply-rendering tests pass, including compact
light/dark layouts, long/multiline text, hover stability and keyboard Copy.
Flutter analysis is clean. No additional model request or new limit is involved.

The normal Windows release was rebuilt and reopened visibly. An actual dark-mode
screenshot confirmed that the user bubble matches the highlighted sidebar session
and keeps readable text. All 32 table digests match the fresh baseline, preserving
settings and history. Light mode has widget coverage; native light-mode and
other-platform visual acceptance were not repeated for this color refinement.

## Conversation layout — alignment and hover controls

User messages now use right-aligned, naturally sized bubbles; assistant content
stays left-aligned. Sender names and profile images are removed from both roles.
Hover or keyboard focus reveals Copy and a local saved timestamp in reserved
space. Original Markdown source is copied, and existing attachment, tool,
usage and recovery controls remain available. Streaming replies omit whole-message
copy. Schema 25 adds optional atomic local save timestamps without rewriting old
messages or inventing their dates; forks and JSON exports retain known times.

All 179 Flutter tests pass, with clean analysis, including compact light/dark
views, long unbroken/multiline prompts, role alignment, absent sender names/avatars,
hover enter/exit without text movement, keyboard Copy and exact source copying.
Unknown/invalid dates do not crash rendering or fabricate timestamps. All 52
core, 61 bridge and 43 storage tests pass (156 total), with clean Clippy. New
storage tests cover legacy migration, failed timestamp writes rolling back the
entire turn, restart, original timestamps on forks/export and deletion cascades.
The release-native attachment/restart regression also passes.

The normal Windows release was rebuilt. Actual dark-mode screenshots of isolated
synthetic messages confirmed right/left placement, removed names/avatars, user
hover controls and bounded code rendering. The owned fixture instance was closed,
and the normal user-data app reopened visibly (PID 21000). All 31 original table
digests match the fresh baseline; schema 25 adds one empty timestamp table for this
existing history. Provider settings, transcripts and drafts are preserved. These
are presentation/storage checks; no model request was needed. Other-platform and
spoken screen-reader acceptance remain open.

## Brick 11.1 — bounded subagents

Implemented [scoped delegation](design/subagents.md): one batch of one or two
children, inherited model/settings/permissions, non-overlapping writable scopes,
shared model/tool allowances, depth one and parent-owned cancellation/deadline.
Children have file tools only; the parent owns command verification. Child
reports, model usage and tool evidence stay linked to the owning run. Existing
default allowances remain unchanged. Failed/paused children retain completed
work and require explicit review/Continue; restart never replays them.

All 52 core, 61 bridge and 41 storage tests pass (154 total), with clean Clippy.
Flutter analysis and all 175 widget tests pass, including stale event rejection,
stopped-child/draft preservation, literal bounded reports, light/dark compact
cards and malformed-result recovery. Release-native `scripts/test-subagents.py`
verifies simultaneous children, separate batch/file approvals, shared allowance
exhaustion, Stop at queued approvals and running requests, revocation without
writes, Full access refusing scope escape, case-conflicting ownership, retained
reports/Changes and fresh-process restart without connection/replay. The existing
release-native task-budget fixture passes. Bridge tests also preserve one child's
report when another model fails and bound JSON-escaped answers. An 8,000-fragment
stream produces bounded per-step commentary instead of exhausting run events.

Live probes used isolated synthetic data with the configured providers and no
change to the normal selected model/settings. Qwen3.5-2B completed the routine
read task in 4.66 seconds but ignored explicit delegation (zero child batches),
so that is **not a delegation pass**. The first DeepSeek V4.1 Flash coding trial
used all 12 model calls and paused after command repair. Its CommonJS `.js`
fixture inherited an ancestor's module setting and prompted an extra package
file; 223 durable events also exposed excessive per-fragment child logging.
The original trial remains failed. Logging was coalesced and the fresh fixture
used explicit `.cjs` files, with unchanged 1024-output/60-second request and
12-model/12-tool/90-second task allowances.

The corrected DeepSeek case passed in 11.39 seconds: one batch, two reported
children (two model calls each), three parent calls, exactly two owned files and
two applied change records. The parent separately ran the fixed Node assertions
successfully, including strings, NaN and Infinity rejection, with complete
capture and exit 0. Provider-reported usage is available in parent metadata and
child run events; child usage is not silently folded into parent-only counts.
One small passing case does not qualify general delegation reliability.

The normal release app was rebuilt. Saved live child scopes/reports and their
inner scrolling were inspected in its actual Windows dark-mode screenshots,
using an isolated data directory. That owned test instance was closed and the
normal user-data release launched visibly (PID 29944). All 31 table digests
match the fresh baseline, schema 24; provider configuration/history are preserved.
A development-machine point measured 242.87 MiB working set, 247.87 MiB private,
49 threads and a 10,806,784-byte bridge; this is not attributable overhead or
low-end qualification. Physical approval/keyboard and live progress UAT,
other-platform/resource qualification and dependable Qwen delegation remain
open. File scope is an application tool boundary, not an OS sandbox.

## Attachment controls — visual placement and image settings

Inspected screenshots of the normal Windows release before and after this refinement. The plus action had drifted toward the middle because unused model-selector width was distributed by the footer; it is now anchored at the left and vertically aligned with Send/context, while the model selector remains right-aligned. Per-model Supports image input and context-window controls now appear at the top of Model connection. Errors involving draft/history images offer Model settings directly. Cancel preserves capabilities, draft and attachments; failed saves retain edited values and reveal the error for explicit retry.

The layout regression failed before the fix. All 171 Flutter tests and clean Flutter analysis pass, including wide/compact layouts, short/long model names, light/dark themes, visible image settings without scrolling, draft preservation on Cancel, history-image recovery, per-model isolation and failed-save retry. The release-native attachment/restart fixture passes. A fresh isolated Qwen3.5-2B probe refuses disabled image input while retaining the draft, then completes with the correct green color after explicit enabling (2.63 seconds). It ignores the single-word format requirement; that check remains failed, and reported usage is unavailable. This is a small vision probe, not broad model qualification.

The normal desktop release was rebuilt and opened visibly (PID 8992). Its composer and image setting were inspected in screenshots. All 31 current user-data table digests match the baseline captured for this refinement; provider/model/settings and draft attachments remain unchanged. Real-window visual inspection covered Windows dark mode; compact/light behavior has widget-test coverage, while other-platform and broad image-format acceptance remain open.

## Brick 10.6 — file and image attachments

Implemented [immutable local snapshots](design/attachments.md), backward-readable message references, draft preview/removal, explicit model image capability, bounded provider content parts, snapshot export and manual orphan cleanup. Text-only history/wire remain compatible. Image allowance is explicitly approximate; no OCR, unlimited upload or background sharing is added.

Verified 50 core, 56 bridge, 33 provider and 41 storage tests (180 total), clean Rust Clippy, workspace compilation, Flutter analysis and 168 Flutter tests. Release-native `scripts/test-attachments.py` verifies source changes preserving the chosen Unicode snapshot, compatible text wire, exact image data URL, default-disabled images issuing no HTTP request, provider rejection retaining draft/reference without retry or private-body leakage, scoped access, four-file/size limits, export bytes, damaged assets refusing before HTTP then recovering through verified reattachment, retained fork references and cleanup/restart. Store tests also repair missing assets; provider checks preserve the original text-only wire allowance. Export beyond 240 messages refuses before creating a folder. Compact light/dark tests exercise preview/removal after refused Send, cleanup failure retaining the draft, and failed-selection isolation. Complex saved permission grants cannot be flattened by the simple form without explicit revocation.

The configured Qwen3.5-2B live text/image probes returned the correct frozen number and dominant green color, in 0.61 / 1.42 seconds. Both ignored the requested exact terse answer format, so their original strict literal checks remain failed; the observed comprehension is recorded separately. Provider usage was unavailable for these calls. Normal connection/model/settings were preserved and synthetic transcripts/credentials stay outside tracked artifacts.

The final normal release app was rebuilt and visibly launched (PID 26864). Original 22 user-data table digests remain unchanged; schema 24 has 31 tables. A development-machine point from the attachment release measured 233.48 MiB working set / 242.50 MiB private, 43 threads and a 10,632,192-byte bridge. These are point observations, not attribution, startup timing or low-end qualification. Header checks do not establish full image validity; native picker/physical-input, other-platform and broad vision acceptance remain open. Cleanup reclaims logical bytes, not guaranteed physical shrinking/secure erasure.

## Milestone 10 — batch exit

All six planned bricks are implemented and independently committed. Final release-library regression fixtures for task budgets, Review/Auto/Full permissions, restart after edit, managed compaction/forks and attachments pass. The fixed evaluator still rejects invented completion, weakened checks and discarded preserved prefixes. Existing model/task defaults remain unchanged.

The unchanged v1 six-case corpus was rerun against isolated data using the configured Qwen3.5-2B for routine understanding and DeepSeek V4.1 Flash for coding/pressure cases:

| Case | Observable result | Approval / recovery evidence |
| --- | --- | --- |
| Understand project | Qwen **failed**; a separate DeepSeek comparison **passed** unchanged facts/files | Qwen returned an unnecessary folder question without tool use; 0 approvals. DeepSeek inspected the same fresh fixture under the same request/task limits. |
| Repair and fixed check | **Passed**: unchanged assertions execute successfully | DeepSeek, 5 approved operations, no denied operation. |
| Multi-file edit | **Passed**: both intended files change and output is exactly Hello, World! | DeepSeek, 6 approvals and 1 denied extra command; fixed check still passes. |
| Larger file | **Passed**: 1600-line prefix/check file unchanged and assertion passes | DeepSeek, 6 approved operations using a ranged read and source-bound patch. |
| Limit recovery | **Passed after explicit Continue**: greeting bytes exactly match | Initial two-model-call segment pauses after one read. A linked second segment uses the same 2-model/4-tool allowance, reads and edits once, completing in 9.47 seconds; no silent increase or replay. |
| Interruption | **Passed**: file unchanged after Stop at pending edit | One approved read, Stop at edit approval, cancelled run and no saved incomplete turn. |

Repair/multi-file/larger-file use explicit 2048-output/90-second request and 8-model/12-tool/180-second task allowances; routine understanding uses 1024/60 and 6-model/8-tool/180. Recovery uses 1024/60, 2-model/4-tool/2-segment/180. The original four-call/four-operation defaults are not changed. Reported live usage remains unavailable in saved foreground metadata; per-case wall timing was not captured by the initial corpus driver, apart from the timed recovery. This timing/usage coverage gap is retained rather than inferred from token estimates or reply length.

The initial routed corpus had five observable passes after explicit recovery. A separate fresh DeepSeek understanding case passed the same frozen criteria and limits, so each of the six cases now has passing evidence with the stronger model. The original Qwen failure remains recorded: consistent behavior across the configured models is **not accepted**. A completed response is not a task pass, and these small DeepSeek cases do not establish general coding competence. Advanced optional tools are planned next, while model grounding and broader native/platform/resource qualification remain explicit gaps. Executable self-evolution/automatic skill adaptation remain in milestones 13/12, respectively.


## Brick 10.3 — Ranged files and explicit command limits

Implemented [practical tools](design/practical-tools.md): bounded line reads with SHA-256 source snapshots, unique snapshot-bound edits/reverts up to 1 MiB, and reviewed command deadlines/capture with local large-log artifacts. Existing default command/model budgets remain unchanged. Large captures require explicit reviewed limits; literal automatic command grants cover only the default limits.

49 core, 53 bridge, 37 store, 33 file-tool and 10 command-tool tests passed; Clippy warnings denied, Flutter analysis and 163 Flutter tests passed. New file cases preserve Unicode/CRLF and all untouched lines, reject a concurrent edit and stale digest, then succeed after a fresh read. Invalid ranges and oversized single lines explain recovery. Command cases retain all 5,000 Unicode output lines in a local log, keep model output bounded, report a one-second timeout truthfully and complete under a fresh three-second allowance. Existing child cleanup/output-budget cases still pass. The native task-budget fixture also passes.

A bounded DeepSeek V4.1 Flash v1 larger-file baseline passed: all 1,600 prefix lines and check.py remained exact, the snapshot-bound edit was applied and the unchanged executable assertion passed. One extra command was denied by the synthetic test policy; the required check still completed. This is one small case, not general competence. Private credentials/transcripts and the normal selected model were preserved.

Normal release build and visible launch passed (PID 32436); original 22 table digests remain unchanged, schema 21. Local logs are explicit user-managed working-folder files; no command-effect rollback, atomic multi-file operation or universal platform/input/resource acceptance is claimed. Physical log-opening/range UI UAT remains open; host conflict/range recovery and local artifact contents are exercised.

## Brick 10.2 — Thread permissions and revocation

Review, explicit automatic grants and full task access now apply to working chats. Grants stay thread-local, are pinned to a revision and expiry, and are checked before dispatch. Commands/MCP retain their actual containment limits; task access cannot activate self-updates.

49 core, 53 bridge and 37 store tests passed; Clippy and Flutter analysis were clean, with 163 Flutter tests passing. The normal-library fixture exercised Review/Auto/Full access, uncovered-path review, stale saves, expiry before dispatch, revocation during approval and late approval refusal. Generic settings cannot replace permission grants; another chat sharing the folder remains in Review. Existing budget/settings fixtures also passed. Full access retained the configured operation cap.

A bounded DeepSeek V4.1 Flash multi-file baseline passed independently: both files changed and the executable produced Hello, World! with exit zero. Covered discovery/read operations were automatic; two edits and one command required review. Credentials and transcripts remain local, normal settings unchanged. This small result does not establish general reliability; the earlier Qwen failure remains recorded.

The final normal app was visibly launched (PID 13476), with all 22 original table digests unchanged. A development-machine point was 225.72 MiB working set, 238.62 MiB private and 46 threads; bridge 10,152,960 bytes (+122,880). Resource conditions vary. Native keyboard/IME, MCP automatic-grant integration, other-OS and low-end acceptance remain open. Revocation cannot undo already-started effects; commands are not an OS sandbox.

## Brick 10.1 — Fixed baselines and coherent task allowances

Implemented [task budget contracts](design/task-budgets.md), scoped inspector controls, effective inventory/context/run values, accumulated continuation segments and the six-case v1 corpus. Existing model/output/context defaults and four-call/four-operation defaults are retained. The new default task ceiling is four explicit segments; no unlimited continuation or automatic model/budget switch occurs.

Mechanical evidence: 47 core, 52 bridge and 37 store tests passed; Clippy warnings denied and Flutter analysis clean; all 161 Flutter tests passed. The external evaluator's two negative cases reject a false completion, altered checks and loss of the larger file's preserved prefix. A normal Windows DLL/loopback fixture passed eight model requests: an explicit five-operation cap allowed exactly five approved reads, the sixth proposed operation paused; an exhausted segment refused before another request; truncated pending calls had zero approvals; Stop at approval had no effect intent and preserved files. Invalid task limits preserved the saved revision/value. The old scoped-settings native fixture remains a separate check.

Bounded live corpus probes used isolated data and in-memory configured credentials, leaving the normal selected model/settings intact. Qwen/Qwen3.5-2B (1024 output tokens, 60-second request; explicit 6-model/8-tool/180-second segment) made no tool call and failed all three understand-project facts; files remained unchanged. This was a model/prompt behavior failure rather than observed step exhaustion, not proof of provider or universal model failure. DeepSeek V4.1 Flash (2048/90; explicit 8-model/12-tool/180-second segment) performed discovery, two reads, an edit and the fixed command check. Both unchanged assertions passed and the executable file outcome independently passed. Its five operations exceed the original four-operation limit; this supports configurable allowances, not a silent default increase or broad competence claim.

Remaining gates: the full unchanged corpus must be rerun after 10.2–10.6. Larger-file baseline is intentionally beyond the old whole-file tool limit; interruption/restart, managed compaction and multimodal UX belong to their scheduled bricks. Task/provider failure attribution remains evidence-based and may be unknown. Native input, low-end and other-OS acceptance remain open. The final normal release app was built and visibly launched (PID 13244), with all 22 original table digests unchanged, schema 21/25 tables. The native scoped-settings fixture also passed its four wire requests. A development-machine resource point was 237.55 MiB working set, 256.66 MiB private, 45 threads; bridge 10,030,080 bytes (33,280 more than 9.4). Startup/inspection conditions vary; no attributable memory benefit or low-end acceptance is claimed.

## Brick 9.4 — Scoped settings and Dolores interaction

[User/project/chat settings](design/scoped-settings.md) now affect primary request construction and context preview, retain origins/revisions in durable runs and provide discussion/assumption-checking controls. Task review, automatic preferences and future self-update activation remain separate. The shared base prompt no longer incorrectly asserts that every request has no tools or learned memories.

- **45 core / 52 bridge / 37 store tests pass**; **161 Flutter tests**, analysis and Clippy with warnings denied pass. Precedence/scope isolation, frozen values, private-root exclusion and shared preview/send accounting are exercised. Storage tests cover invalid/missing scopes, stale revisions, injected transaction failure, restart and chat deletion. UI tests cover save/reset, stale-save draft preservation and explicit retry, scope/style changes, busy read-only access, changed-window recovery and compact dark layout. The new header icon initially caused compact overflow; moving Settings into Chat actions restored the existing layout checks.
- The **normal Windows C ABI/HTTP fixture passes four actual requests**: chat override → project override → model profile, then recovery after changing the model window. A stale save makes no request; a smaller window leaves invalid values inspectable but refuses send before any new request/run. Reset restores valid execution. Historical run settings remain frozen and a disconnected start creates no chat. This is an isolated synthetic fixture, not a model evaluation.
- The retained mock-provider catalog check now accepts the comparison shells' six built-ins or Flutter's seven including inspection. Focused HTTP checks verify both catalogs and refusal of an unknown tool; all historical diagnostic smoke stages were not rerun in this brick.
- **Bounded live DeepSeek V4.1 Flash false-premise probes passed the observed criterion twice**, including the final normal bundle: a synthetic Rust project was called React; the final probe performed four approved discovery/read operations, explained observed Rust source, asked for the intended target and made no changes. It accurately exposed exhausted tool allowance. This is useful premise checking in two examples, not proof of stable personality/general competence.
- **Qwen3.5-2B routine coding acceptance remains open.** At 512 output tokens / 60 seconds, the first exact greeting-file edit made no tool call and asked for an already supplied location. A retry read the file but its test also denied a discovery operation; no edit occurred. The final normal-bundle probe allowed discovery/read/edit, yet made no call, invented an extra confirmation requirement and confused names/text. A separate short arithmetic response avoided unnecessary questions but misstated the expression; it does not establish correct reasoning. These failures are retained, not replaced with fixture passes or attributed solely to prompt length/model size. Clarifying relative paths and removing the contradictory base prompt were scoped fixes; consistent tool use and meaningful task completion need milestone 10's fixed corpus and failure attribution.
- The updated **normal desktop window is visibly open**; all 22 original table digests match the pre-batch baseline. Schema 21 adds three empty tables across this milestone; provider selection/configuration/history remain intact. Live probes used isolated data and launch-only credentials, without changing the normal model/settings or tracking private transcripts/keys.
- Final development-machine observation: **239.59 MiB working set, 250.73 MiB private bytes, 45 threads; bridge 9,996,800 bytes**. Against the preceding 9.3 observation, the bridge adds **129,024 bytes**. Process-memory observations used different startup/inspection conditions and cannot establish attributable overhead or representative idle/low-end acceptance. No new always-running service or idle model request is added; the new default interaction instructions consume request context. Resource qualification stays open.

Milestone 9's implemented contracts are complete with separate commits and basic mechanical/recovery verification. Behavioral/task acceptance is partial as described above. Auto approval/full access, checkpoints/forks, multimodal input, richer scoped knowledge, executable mods/activation and independent tool-using adaptation trials remain planned. Native input/accessibility, macOS/Linux execution and representative resource gaps remain open.

## Brick 9.3 — Extension registry and pinned contracts

Compiled components and MCP entries now share [bounded descriptors and dependency resolution](design/extension-registry.md). Primary runs pin exact registration/configuration snapshots and tool ownership. A compiled proposal hook fails closed; external servers cannot install hooks or confer authority. The capability inspector adds a collapsed registry view without new polling/services.

- **43 core / 50 bridge / 36 store tests pass**, followed by a new focused inspection-refusal test: dependency/API/cycle isolation, retirement cleanup, failed/retargeting hook refusal, external-hook rejection, server failure isolation and removal without stale new-run tools. **11 MCP tests pass** including malformed/flood/nonresponsive servers, changed manifests, independent servers, Stop/process cleanup and credential handling. Analysis/Clippy are clean.
- The first bounded Qwen probe requested an inspection that failed preparation; the generic recovery incorrectly called it a file-access refusal. Recovery now explains valid inventory/source arguments without relaxing validation. The separate deterministic case confirms zero approvals/effects for malformed inspection. A fresh Qwen/Qwen3.5-2B probe (same 512-token / 60-second allowance) completes one saved inspection receipt, six durable events and **nine pinned registrations**. This is a narrow integration pass, not general model reliability or proof the earlier model request had a particular malformed field.
- Existing **156 Flutter cases pass** before the new registry expansion case; its compact scrolling/unsupported-API/Refresh flow passes after the test explicitly scrolls into view. Normal release build succeeds, a visible normal window is reopened and all original 22 table digests remain unchanged. A point sample across 9.2→9.3 gives bridge size +111,104 bytes, working set 227.54→225.82 MiB, private bytes 235.36→238.03 MiB and 40→46 threads. Startup timing differs, so memory/thread differences are not attributable overhead or low-end qualification. No registry service/poller is added. Executable loading/activation, broader hooks, native inspector interaction and cross-platform/resource qualification remain open.

## Brick 9.2 — Run ownership and durable evidence

Primary chat runs now have frozen snapshots and ordered approval/operation/terminal records. Run history is available during execution. Startup marks unfinished runs interrupted without replay; one process owns a data directory and one execution owns cancellation/decisions. See [implemented boundaries](design/run-ownership.md), including transient deltas, auxiliary job receipts and foreground-navigation limits.

- **49 bridge / 36 store tests pass**: directory-lock refusal/release, single execution and shutdown, other-chat reads versus mutation refusal, wrong-ID Stop, scoped/stale event writes, restart uncertainty/idempotence, exhausted evidence with reserved terminal marker, invalid transition rollback and chat deletion. Existing backpressure/deadline tests pass. Migration fixtures preserve legacy history/settings; schema expectations advance to 20.
- **156 Flutter tests pass**, including compact dark inspection during execution, failed refresh/recovery and safe menu actions versus disabled exports. Analysis and Rust Clippy are clean. A bounded live Qwen/Qwen3.5-2B inspection (512 output tokens / 60 seconds) completes with one saved tool receipt and six ordered durable events, including intent/result and terminal state. Original model/settings remain untouched.
- A separate normal-library, two-process fixture refuses the second host and, after simulated effect-intent interruption, reopens with an interrupted/restart marker and unchanged fixture effect. This is deterministic failure injection, not a real interrupted command. Normal release build succeeds; a visible normal Dolores window is reopened. The original **22 table digests** are unchanged, with two empty schema-20 tables added. Physical native inspector interaction, cross-platform and representative resource acceptance remain open. Exactly-once external execution and automatic replay are excluded.

## Brick 9.1 — Capability and bundled-source introspection

Working chats now advertise a reviewed read-only `inspect_harness` tool; saved chats offer **Chat actions → Dolores capabilities**. Inventory reports actual tool registration, approval/containment, model/configured state, limits and window origin. Model function-call reliability is explicitly unknown, images remain unsupported, and self-updates remain unavailable. Five public source components are bundled with bounded line reads; a selected checkout is compared byte-for-byte but never replaces running source. No private root, provider key or user transcript is included.

- **39 core / 46 bridge tests pass**, including disconnected/Side inspection, bounded ranges, invalid names/ranges and matching versus stale checkout. The catalog maximum is nine (six original built-ins, introspection and two MCP aliases); model-call/tool-operation limits remain four each.
- **154 Flutter tests pass**, including compact dark inspection/source selection, failed local inspection then Refresh, and correct read-only/sharing approval. Analysis and Rust Clippy are clean. Native physical keyboard/IME/accessibility and broader resource/platform acceptance remain open.
- A bounded **Qwen/Qwen3.5-2B** probe uses isolated data, 512 output tokens / 60 seconds and disabled automatic memory: one inspection approval, one completed saved receipt and a completed response. No project-file/command request occurs. Original model/settings are unchanged. This verifies the narrow tool flow, not general reasoning.
- Normal release build succeeds and a normal Dolores window is visibly reopened. All **22 schema-19 table digests** match the original configuration/history snapshot. FNV source identities are diagnostic, not cryptographic attestation; checkout matching compares full bytes. Bundled coverage is limited. Executable loading, source edits and automatic activation are excluded.

## Architecture and roadmap planning

The clarified vision is captured in the [target architecture specification](design/evolving-harness-architecture.md), [behavior policy](design/dolores-behavior.md) and revised [roadmap](ROADMAP.md): 21 scoped bricks across milestones 9–13, with dependencies, basic acceptance, realistic failure/recovery checks, exclusions and milestone exit gates. All requested basic/special features have named coverage. Brick 8.4 remains skipped; no future runtime brick is claimed complete.

The fit assessment uses current core ports/Message, agent tool/approval contracts, Flutter host coordination and existing skill/comparison/feedback design boundaries. It concludes that compiled modular ports and persistent knowledge are reusable, but executable self-evolution still needs lifecycle/activation, scoped runs, independent task evaluation and enforced generated-code boundaries. This is a source/design assessment, not a live security or model-reliability test.

This task changes documentation only. All **51 Markdown documents / 153 local links** pass the local checker; external URLs are counted, not revalidated. The roadmap's 21 consecutive brick IDs and scope/basic-flow/failure-recovery/exclusion fields are checked, with whitespace checks before commit. No provider requests, app/data/config changes, dependency changes or runtime tests are needed. Per the user's documentation exception, no desktop build/relaunch is performed. Runtime, cross-platform, resource and automatic-activation acceptance remain open.

## Earlier roadmap review

Brick **8.4 is skipped** in the earlier plan. The earlier forward proposal (milestones 9–12) was separated from the [implementation history](IMPLEMENTATION_HISTORY.md). That proposal is now superseded by the vision-based milestones 9–13 above; historical completed bricks retain their original scope. Documentation planning does not close existing platform/input/resource gaps. Runtime behavior and provider settings are unchanged.

## Brick 8.3 — Composer input and accessibility

Native rich-composer fields now expose Message plus heading level, code language or paragraph position, while retaining editable values and focus. The code-language button has a named enabled state and a tooltip excluded from repeated speech. Send and language changes wait for active composition to commit rather than submitting unfinished candidates or clearing the composing range. No editor engine, dependency, worker, provider request or theme change is added. See the [verification matrix](design/native-input-verification.md).

- **151 Flutter tests pass**, with clean analysis and formatting. Four new cases cover semantic names/values/focus, Windows whole-draft select/delete/undo, active Chinese composition refusing Send then sending exactly once after commit, and composition refusing language changes then preserving text/focus after commit. The latter two exercise realistic recovery at the platform text-input seam, not a real IME candidate window.
- The normal Windows release builds. Native isolated-data checks exercise continuous heading/fence/code typing, Shift+Enter, Down into prose, right Ctrl+A across heading/code/prose, complete Backspace removal and undo, Tab into Model connection and Escape dismissal. Editable names/values/focus appear in the Windows accessibility tree. No synthetic draft was sent or connection form saved.
- An injected left-Control chord failed. A temporary key-only probe showed its letter event arriving after Control was released, unlike right-Control; this does not establish a physical-keyboard defect. The probe is removed from final source/build. Physical left-Control, real Chinese IME candidate selection, spoken Narrator traversal and enlarged native display-scale acceptance remain **pending**. Immediate native snapshots may precede settled focus/selection and were refreshed before judging behavior.
- All **22 schema-19 table digests** still match the existing configuration/history snapshot. This UI-only change requires no live-model request and makes no general model-reliability claim. Rust behavior tests and portable packaging are outside this change; prior evidence retains its original scope.
- All **46 Markdown documents / 113 local links** pass. The public working-copy snapshot covers **331 paths**, with zero known personal-prefix/email matches and a clean Gitleaks result. The final release's native language-button name appears once; code-card creation retains typing and Down exit. The normal desktop app is reopened with the original data location for user verification.

Brick 8.3's implementation and bounded checks do not close all native input/accessibility release gates. Actual clean-machine startup, macOS/Linux builds, signing and representative low-end measurements remain separate work.

## Commit message language

All **40 existing commits** now have English subjects and bodies. Every rewritten commit was checked against its original raw object: **author and committer dates (including timezone offsets), identities, tree snapshots, and parent order are preserved**. Commit IDs changed. Unrelated tree refs, the index, and working files remained unchanged during the rewrite. A verified local recovery bundle and old-to-new commit map are retained in ignored output, without publishing another history.

Project guidance now requires English commit messages. The normal Windows release rebuild passes and the app is visibly open; all **22 schema-19 table digests** still match the prior data snapshot. This task changes history metadata and documentation; it adds no runtime behavior or live-model reliability claim.

## Brick 8.2 — Dependency notices and startup recovery

The Windows preview now includes a [versioned dependency inventory and full notices](DEPENDENCIES.md), bound to runtime/locked-input hashes, and **Start-Dolores.cmd** with recovery guidance. Collection uses prepared local dependencies without downloads or model calls. The original infinity logo and **Dolores** title are retained.

- A fresh normal Windows release build passes. Offline collection records **203 components**: **165 Rust**, **32 Dart**, **4 SDK**, **1 native SQLite** and **1 artwork** entry. This is a conservative normal/build/production closure, with unused platform/build components and consolidated upstream inventories; it does not measure exactly linked/shipped code. All selected entries have full notice text. Engine revision matches the Windows SDK notice manifest; the actual bundled SQLite 3.46.0 header and ring/BoringSSL/fiat notices are retained. The single MPL-only crate (`option-ext` 0.2.0) includes corresponding source verified against its Cargo.lock-hashed cache archive and local source. SDK packages inheriting the Flutter repository license are labeled with that full text rather than silently omitted.
- **Four packaging tests and one native batch test pass.** Existing completeness/architecture/privacy/corrupt/no-overwrite cases remain green. New pressure cases refuse runtime/lock mismatch and truncated notices before destination creation; a corrected explicit retry succeeds without deleting previous evidence. A synthetic missing extraction reports fresh whole-ZIP guidance, then a missing runtime reports Microsoft's x64 install/repair link, then supplied presence fixtures pass. Empty runtime files refuse too. The native command path includes **spaces, ampersand, parentheses and Unicode**; its initial quoting failure was corrected and the recovery sequence passes. Synthetic history remains byte-identical. No Windows system DLL was changed, installed or uninstalled; fixtures test messaging/control flow, not actual runtime version/loadability.
- A fresh unsigned x64 ZIP contains **21 payload files plus its manifest**, **15,999,836 bytes**. The full notice text is **4,806,230 bytes**, alongside Flutter's existing compressed notices. Exact archive verification, all **21 extracted hashes**, and the real extracted launcher preflight pass. Launching through **Start-Dolores.cmd** opens a visible normal **Dolores** window using isolated test data; that owned test process was closed afterward. No artifact was published or signed. Prior brick 8.1 previews remain intact with their original payload contract.
- The updated normal desktop app is visibly open with the original data location. All **22 schema-19 table digests** match the pre-task snapshot, preserving provider configuration/history. No provider/model setting or transcript was used as test input. No model test is required for this packaging-only change; prior live-model failures retain their original acceptance boundary. Flutter/Rust behavior tests were not rerun for this task; the logo task's **147 Flutter tests** remain separately recorded below.
- All **45 Markdown documents / 110 local links** pass. The public source snapshot covers **329 paths**, with zero known personal-prefix/email matches and a clean Gitleaks result; the extracted ZIP also passes its secret scan and known local-prefix refusal. Package entry selection remains enabled. The inventory contains upstream attribution/source identifiers, no local cache/builder paths. These checks do not classify every possible personal detail.

Actual clean-machine startup, damaged/old C++ runtime version recovery, 32-bit command-host behavior, signing, native keyboard/IME/accessibility, macOS/Linux packaging and reference low-end resources remain open. This technical notice audit is not an exact linkage proof or a blanket legal-compliance certification; new dependencies/native flags/artwork require another review.

## Logo consistency

Flutter keeps the original `Icons.all_inclusive_rounded` infinity mark in its home/sidebar/assistant avatars. Its exact existing font outline is retained in `assets/dolores.svg` for Windows title/taskbar/executable resources, macOS assets, Linux's embedded GTK icon and retained web/Tauri assets, with Google's upstream icon license. A development-only Pillow generator produces **14 outputs** and verifies that checked-in files match; it adds no Flutter runtime image package or model request. The visible Windows title and product description are **Dolores**, without the framework suffix.

- Normal Windows release built and visibly launched; all **22 schema-19 data-table digests** match, preserving configured provider/history. **147 Flutter tests**, clean analysis/format and the retained web production build pass. Existing compact light/dark cases remain green.
- Small **16/24/32-pixel icons** were visually inspected against a light background, with antialiased transparent corners. The built EXE's actual seven icon resources (**16/24/32/48/64/128/256**) match the generated ICO payload byte for byte, covering small and high-DPI selection rather than checking the source file alone. The first custom closed-loop design was rejected by the user; the final assets reuse the original rounded glyph exactly.
- macOS/ICNS assets decode and regeneration matches; Linux embeds the PNG so the window icon does not need a source-folder path at runtime. macOS/Linux builds and desktop-shell icon display remain unverified. GTK icon display depends on the window manager; Linux launcher/package integration remains a platform release check. No icon cache or pinned user shortcut was reset.

## Brick 8.1 — Windows portable preview and documentation

README now introduces Dolores and its first-use flow. The separate [user guide](USER_GUIDE.md) covers controls/data/recovery, [contributor guide](../CONTRIBUTING.md) covers standard build/test/package commands, and [documentation index](README.md) points to internal design/evidence. Architecture and privacy describe current behavior rather than the original scaffold; the previously missing Flutter bridge reference is supplied. Documentation command examples and public build/regression helpers no longer require an output-filtering wrapper.

- The full plain-tools regression run passed **23/23 stages**: **203 Rust tests / 1 ignored**, **147 Flutter tests**, formatting/Clippy/analysis, normal release build and eight isolated native save/restart pairs. After compiler-path privacy changes, the Rust bridge was rebuilt and all **16 native fixture stages** passed again. The final generated-registrant mapping then rebuilt the normal Flutter app successfully; extracted and normal desktop windows opened visibly. These are separate scopes, not a claim that the earlier full run exercised the final compiler metadata mapping.
- **Three packaging tests pass**: complete payload/hashes/no-overwrite; corrupted content and malformed manifests refuse; missing Rust DLL, unexpected synthetic database, wrong architecture and embedded local build paths refuse before creating a destination, then an explicit corrected attempt succeeds. UTF-8 path matching across a stream-chunk boundary is checked. Existing artifacts remain intact. These synthetic minimal PE fixtures do not establish executable compatibility.
- A real unsigned Windows x64 ZIP contains **18 payload files plus its manifest**, **15,662,989 bytes (14.94 MiB)**. Exact-entry/hash verification and every extracted-file hash pass. Full runtime assets/notices, user/privacy/start guidance and MIT license are included; no workspace/database/key/debug-symbol files enter the allowlist. The normal extracted app opened visibly using separate test data. No package was published or signed.
- An extra binary scan found current source-machine paths in the first Rust/Dart bundle. Rust path remapping removed home/toolchain paths; Flutter split debug information plus a build-only stable generated-registrant package URI removed the remaining absolute URI. Original package-config bytes are restored afterward; no SDK or generated source patch is used. Packaging now refuses current home/workspace prefixes in UTF-8/UTF-16. The final extracted payload has **zero known personal-path/email matches**, and its Gitleaks scan passes. This local-prefix/secret scan is not a universal PII or foreign-builder scan.
- Bounded **Qwen/Qwen3.5-2B**, 1024 output tokens / 60 seconds, through the final extracted Rust library: request/complete-turn persistence and synthetic feedback passed in **0.31 s**, but the exact arithmetic rule **failed** (wrong single digit), correctly saving Needs work. A preliminary unremapped bundle's equivalent probe passed in 0.28 s. Neither is general model competence, and no silent corrective retry was used. Original credentials were read only; live probes used isolated data and disabled automatic preference learning.
- All **44 Markdown documents / 105 local links** pass the local link check; external URLs are counted, not automatically verified. Public working-copy scan covers **321 paths** with zero known personal-pattern/Gitleaks findings. The updated normal app was visibly launched with the existing preview data; all **22 schema-19 table digests** match the baseline, preserving provider selection/settings/history.

The [portable contract](design/windows-portable.md) labels freshness unverified when packaging an arbitrary existing bundle. This local preview is not public-release acceptance. Complete third-party license review, signing/trusted distribution, clean-machine/missing-C++-runtime behavior, native keyboard/IME/accessibility, macOS/Linux and representative low-end resources remain open. A portable executable does not make its history/vault data portable.

## Brick 7.3 — Repeatable regression checks

`scripts/run-regressions.py` runs build/lint/unit/widget checks and eight selected native save/restart pairs in fresh isolated data directories. Each stage has a deadline, retained log and incremental JSON/Markdown report. Failures, interruption and invalid receipts stop the run; a fresh retry is explicit. Source/bundle hashes distinguish working-copy evidence, and skipped/native-only scope stays labeled. See [runner prerequisites and recovery](design/regression-runner.md).

- Full Windows run: **23/23 stages passed**, including **203 Rust tests passed / 1 ignored**, **147 Flutter tests passed**, clean format/Clippy/analysis, normal release build and all **16 separate-process fixture stages**. Cases cover feedback CAS/storage recovery, frozen comparisons and failed-result preservation, repeated output/step limits/Stop, per-model settings/malformed tool calls, actual Node validation/repair, exact LF/CRLF edits/revert, automatic-memory protection/cancellation and skill-extraction truncation/31-second draft/frozen promotion/rollback. All requests in this runner were synthetic loopback fixture calls, with zero configured live requests.
- Two runner tests pass: nonzero exits preserve usable logs, invalid/mismatched/missing/nonzero-live receipts and unfinished scopes cannot pass, a deadline stops the owned child tree (no delayed file write), and an explicit fresh retry succeeds without deleting earlier evidence. The delayed-child check was strengthened after the full run and passed independently. Python isolated mode keeps assertions active. A separate native-only selected comparison pair passes with skipped/freshness scope explicitly labeled; reuse of that existing output directory refuses before execution. Paths outside the ignored output root are refused.
- The first two full attempts stopped honestly on dependency/plugin-link preparation, retaining their incomplete reports. The build helper now prepares generated junctions after dependency resolution and uses `--no-pub` for subsequent build steps; analysis/tests also avoid repeated dependency preparation. The fresh full run passed without changing the SDK/Developer Mode or weakening checks. Two older fixtures were updated from schema 15 to 19, with explicit request counts and global-skill isolation.
- Normal updated desktop app was visibly launched after the full build/checks. Original schema-19 configuration/history retain all 20 pre-migration table digests and the two new empty tables. Public privacy scan covers **313 paths**, with no known personal-path/email or Gitleaks findings. No new runtime dependency or resident process was added.

**Brick 7 is implemented in its defined bounded scope:** explicit local task outcomes, original task traces, frozen memory/skill response comparisons and repeatable selected pressure regressions. DeepSeek's synthetic comparison improvement and Qwen's failed candidate are recorded under 7.2; the runner deliberately does not turn fixtures into live-model acceptance. General task competence, representative/statistical evaluations, native keyboard/IME/accessibility, other platforms, low-end resources and release packaging/signing remain open.

## Brick 7.2 — Frozen context comparisons

Saved chats provide **Chat actions → Compare instructions**, with scoped memory/retained skill copies, labeled manual snapshots, 1–3 frozen literal tests and comparison-only limits. Receipts retain the exact requests, model/settings, outputs, reported usage and elapsed time. Strict completed improvement never activates instructions automatically. See [comparison design](design/context-comparisons.md).

- Rust workspace: **203 passed, 1 ignored**; six new tests cover immutable prompts/prefixes, scope/revision/deletion/restart, capacity/paging/export, budget and malformed evidence refusal, truncation, timeout, cancellation, excessive capture and reported usage. All-target Clippy is clean.
- Flutter: **147 passed**. Three new compact light/dark and pending-run cases cover stale-source refusal retaining editable inputs, explicit retry, truncated snippets failing evaluation, Stop retaining baseline, duplicate exclusion and Close locking. These are widget checks; native keyboard/IME/accessibility acceptance remains open.
- `scripts/test-context-comparisons.py`: save/restore passes through the normal bundle FFI, SQLite and independent HTTP fixture, **14 fixture responses during save, zero on restart**. Basic strict gain, tie, source correction/stale refusal before requests, output-limit partial preservation, timeout/Stop with baseline retained, injected persistence failure with copyable volatile receipt and a fresh explicit recovery all pass. Both exports retain comparison evidence, and chat messages/other active memory remain excluded. An unfinished stored prefix stays unfinished after restart.
- Authorized synthetic conflicting-input check, 512 output tokens/60 seconds, two responses per model: **Qwen/Qwen3.5-2B** completed in **5.29 s** but failed the candidate literal rule (0/1 versus 0/1), correctly reporting no improvement. **DeepSeek V4.1 Flash**, thinking explicitly off, completed in **2.92 s** with 0/1 baseline and 1/1 candidate. This establishes the selected response check, not general task or statistical improvement. No original provider settings or history were used as test inputs or modified.

Normal Windows release rebuilt and visibly launched with schema 19. All **20 pre-existing table digests** match the fresh baseline; two new tables are empty. Provider configuration/history remain intact. Public scan covers **310 paths**, no known personal-path/email or Gitleaks findings. Broader native/platform/low-end acceptance remains open; no price, representative benchmark or automatic promotion is claimed.

## Brick 7.1 — Local task feedback

Saved replies and trajectory rows offer Worked / Needs work with an optional note. Feedback compares the exact saved reply/metadata and its own revision, stays local, preserves command evidence and never feeds model context or learning. Clear retains a revision tombstone; deleting a chat cascades its feedback. Conversation exports retain the assessment explicitly. See [feedback design](design/task-feedback.md).

- Rust workspace: **197 passed, 1 ignored** native-vault opt-in; all-target Clippy clean. Basic correction/restart/export/deletion and unchanged context are tested. Edge cases cover wrong chat/user-message/stale reply/revision, excessive/control/credential-like notes and injected transactional failure retaining the previous assessment.
- Flutter: **144 tests passed**, analysis clean. Three new compact light/dark and pending-save cases preserve notes after stale/save failure, bind the original model/reply, retry explicitly, exclude duplicates/Close and clear using the saved revision. Compact light/dark widget captures were inspected; these are not native input UAT. No composer changes or automatic request were added.
- `scripts/test-task-feedback.py` save/restore through the normal bundled FFI, SQLite and independent HTTP fixture passes: **two fixture requests during save, zero after restart**. Wrong-scope/source/stale writes refuse; an injected SQLite failure preserves feedback and a fresh retry succeeds. JSON/Markdown exports retain the note, and the next provider request excludes feedback. Restart restores exact messages/feedback; clear rejects stale re-creation. A test-driver extension error was corrected before the fresh passing run; no product retry occurred.
- Authorized **Qwen/Qwen3.5-2B**, 1024 output tokens/60 seconds: the isolated one-digit arithmetic probe passed the exact check in **0.31 s**, and its synthetic Worked assessment persisted on the original reply. This is a small routine flow, not evidence of broad model competence.
- Normal Windows release rebuilt and visibly launched. SQLite is **schema 18** with one empty new table; all **19 pre-existing table row digests** match the fresh baseline. Provider/key, selected model and history remain intact. Public scan covers **303 paths**, with no known personal-path/email matches or Gitleaks findings. Native input UAT, macOS/Linux and low-end resource acceptance remain open.

Feedback is an assessment, not a success score or promotion permission. Brick 7.2 adds frozen comparisons; brick 7.3 makes the selected regression checks repeatable. There is no new dependency, resident worker, model retry or default increase.

## Brick 6.8 — Reliable exact coding edits

A failing regression confirmed that a multiline LF proposal could not match a CRLF source. The edit tool now adapts only proposal line breaks to a file's uniform LF/CRLF style; characters, indentation and a unique match remain exact. Review shows the actual resulting diff. Mixed/lone-CR files retain byte-exact matching and get actionable single-line/original-ending guidance when adaptation would be needed. Raw snapshots, one-use approval, size bounds, journal and revert remain intact. See [exact-edit design](design/reliable-exact-edits.md).

- Rust workspace: **195 passed, 1 ignored** native-vault opt-in; formatting and all-target Clippy pass with warnings denied. Four new filesystem tests include a 16-case LF/CRLF insertion/replacement/deletion matrix, BOM/Unicode/terminal-newline preservation, overlapping/repeated matches, indentation mismatch, no-op after adaptation, NUL, mixed/lone-CR refusal, post-adaptation overflow, Stop, single-use approval and an external newline-only snapshot conflict. The minimized multiline regression failed before the fix and passes after it.
- Flutter: **22 focused tools, changes, continuation and command-repair tests passed**. No Dart code changed; the full Flutter suite was not rerun for this brick. Windows release diagnostics pass **134 checks at each of 1120×780 and 620×700**. Existing ordinary diff/revert screenshots were inspected; these captures do not specifically establish rendered newline-adaptation behavior. Native pointer, keyboard/IME and chooser UAT remain separate.
- `scripts/test-exact-edits.py` passes save/restore in separate processes through the normal bundled FFI, SQLite, independent loopback provider and actual Node: **13 fixture model requests and nine approval requests during save**, including one cancelled edit; **zero model/approval requests after restart**. An LF multiline proposal repairs a CRLF/BOM module and passes three unchanged assertions. Stop/Deny preserve work; a newline-only external change refuses the stale edit without a journal write. Mixed-ending refusal recovers through a fresh single-line edit and approved checks. SQLite retains exact before/after bytes; restart preserves messages/files, and a fresh approved revert restores the original bytes. Cancelled/stale/single-use revert paths remain covered.
- Authorized routine **Qwen/Qwen3.5-2B**, 4096 output tokens/180 seconds: initial run (**38.95 s**) tries to edit the frozen checks, which the probe denies, then pauses at four model calls. One explicit continuation (**15.92 s**, four calls) applies a reviewed module edit that returns the array unchanged, then pauses again. An independently reviewed Node invocation outside the agent run exits **1** on the empty-array assertion. Checks remain unchanged and CRLF/comments are preserved. **The model task remains unfinished**; this independent validation is not an agent command receipt.
- Authorized harder **deepseek-v4.1-flash**, 8192 output tokens/180 seconds and explicit thinking-off adapter: one bounded run (**52.03 s**, including review wait, four model calls/four tool operations) reads both files, applies one LF multiline edit to the CRLF module, and executes the approved exact Node check. **Nine unchanged physics assertions pass, exit 0**, without continuation. CRLF, unrelated Unicode comments and the absent final newline remain intact. This brick's prompt differs from 6.7, so the result is not a controlled improvement score or general game-building acceptance.
- The normal Windows release was rebuilt and launched with a **confirmed visible window**. SQLite remains **schema 17**; all **19 table row digests** match the fresh pre-task baseline. Original provider settings/key and history were preserved. The publishable allowlist scan covers **297 paths**, with no known personal-path/email matches or Gitleaks findings. Synthetic/live transcripts, screenshots and launch evidence remain ignored.

No dependency, migration, automatic retry, fuzzy matching or default increase was added. Optimistic snapshot rechecks remain; they are not an OS-level atomic compare-and-swap. A file without line breaks has no inferred style, and newline-only conversion is not an edit-tool operation. The next brick (7.1) records explicit task feedback alongside run evidence, keeping model claims separate from verified checks. Qwen task reliability, broad coding tasks, macOS/Linux and representative low-end runtime acceptance remain open.

## Brick 6.7 — Coding validation and repair

Returned commands now distinguish failed exits, incomplete evidence and complete zero exits. Final replies with unresolved command failures save a commandReview pause with Repair and verify, even when the model claims success. The host carries prior receipts through explicit Continue and clears a failure only after complete zero-exit evidence for the same program/literal arguments. Completed files, failed output, earlier segments and fresh approvals remain. Per-call remaining-budget guidance encourages validation within the unchanged four-model-call/four-operation defaults. See [validation/repair design](design/coding-validation-repair.md).

- Rust workspace: **191 passed, 1 ignored** native-vault opt-in; formatting and all-target Clippy pass with warnings denied. Core cases distinguish nonzero/truncated, timeout, unavailable exit, capture/UTF-8 errors and malformed receipts; exact rerun reconciliation includes legacy completed-status receipts and refuses unrelated/denied successes. After moving initial budget guidance into preview/trimming/accounting, the affected core/Flutter-bridge suites pass again (**79 tests**), with final Clippy clean.
- Flutter: **141 tests passed**, analysis clean. Compact light/dark tests show inherited failures beneath a false success claim, literal expandable stderr, Repair and verify, preserved drafts, latest-source binding and duplicate exclusion. Windows release diagnostics pass **134 checks at each of 1120×780 and 620×700**. Failed/incomplete command outcomes reload with repair provenance. Both command-repair screenshots were inspected; native pointer, keyboard/IME and chooser UAT remain separate.
- `scripts/test-coding-repair.py` passes save/restore in separate processes through the final normal bundled FFI, SQLite, independent loopback provider and actual Node: **16 fixture model requests and 12 approval requests** across both stages, including one cancelled edit. The first segment creates a broken module and unchanged checks, executes a real failing assertion, then receives deliberately false model success text. Restart preserves the exact failed segment. Stale-source continuation is rejected before model access; Stop preserves history/files; a changed-file edit is refused without a journal write. An unrelated successful Node invocation leaves repair open. Fresh read/exact edit/approved rerun passes all three assertions, preserves the check file and external comment, and clears the pause. Oversized command output stays incomplete and actionable.
- Authorized routine **Qwen/Qwen3.5-2B**, 4096 output tokens/180 seconds: the seeded sum repair finishes its bounded run in **10.56 s**, but its file read is blocked and subsequent listing/search requests are denied under the probe's exact-file policy. It never edits or validates the module. **The task remains unfinished**, with the original files unchanged.
- Authorized harder **deepseek-v4.1-flash**, 8192 output tokens/180 seconds and explicit thinking-off adapter: initial run (**60.28 s**, including review wait) reads both files, executes the real failing physics check, then encounters an exact-edit mismatch and pauses at the step limit. First explicit continuation (**9.57 s**) repairs movement but encounters another multiline mismatch and pauses. Second explicit continuation (**23.17 s**, including review wait) applies two reviewed collision edits and reruns the exact check command: **nine unchanged assertions pass, exit 0**, and repair clears. Checks cover horizontal/vertical edge touching, overlap, left/right/opposing movement, zero time and landing. Test contents remain unchanged. This is a small seeded repair, not general game-building acceptance; Qwen reliability and the reported Mario/Three.js task remain open.
- The normal Windows release was rebuilt and launched with a **confirmed visible window**. SQLite remains **schema 17**; all **19 table row digests** match the fresh pre-task baseline. Provider settings/key and history were preserved. The publishable allowlist scan covers **294 paths**, with no known personal-path/email matches and no Gitleaks findings. Diagnostic screenshots, synthetic/live test data and launch evidence remain ignored.

No dependency, migration, resident process, automatic retry or default increase was added. Model planning remains fallible, exact-edit CRLF mismatches consumed real repair steps, and complete exit-zero evidence establishes only that command's result. Commands retain their existing OS permissions and non-journaled effects. The next brick (6.8) improves actionable exact-edit recovery without fuzzy matching or weaker concurrency checks. macOS/Linux and representative low-end runtime acceptance remain open.

## Brick 6.6 — Per-model generation profiles

Request settings now selects an enabled model independently of the chat. Save binds output allowance, timeout and optional reasoning adapter to that exact endpoint/model. Switching models/restart restores the right profile; Restore defaults removes only that override after Save. Existing application settings remain the fallback, with unchanged 2048/180 defaults unless already edited. Context-window configuration still defaults to 128K when blank. See [profile design](design/model-generation-profiles.md) and [official contract research](research/model-generation-controls.md).

- Rust workspace: **189 passed, 1 ignored** native-vault opt-in; formatting and all-target Clippy pass with warnings denied. New checks cover unchanged legacy JSON, unknown reasoning refusal, endpoint/model isolation, schema-16 migration preserving exact settings/history, stale/failed writes, transactional profile reset/pruning, model switch/restart without credential rotation and bounded structured consumers retaining their caps. Actual HTTP tests check all adapters across chat, streamed tools and strict tool calls. A named generation rejection, including one also mentioning stream_options, produces no retry or raw-body exposure.
- Flutter: **139 tests passed** in the full suite; the final expanded seven-test settings suite also passes, analysis clean. Compact light/dark cases cover another model's profile, draft preservation, pending/failed Save with retained edits, local reset and explicit Save, and actionable unsupported-setting/malformed-call recovery without a Retry button. The reset test exposed Form.reset changing the model selection and retaining the override; the corrected path validates fields without resetting the model dropdown. Windows release diagnostics pass **132 checks at each of 1120×780 and 620×700**. Wide/compact profile screenshots were inspected. Native pointer/keyboard/IME UAT remains separate.
- `scripts/test-generation-profiles.py` passes save/restore in separate processes through bundled FFI, SQLite and an independent loopback provider: **seven fixture model requests, one approval during save, zero model requests after restart**. It verifies actual profile-dependent cap/reasoning fields, saved reasoning usage on an output-limit pause, provider rejection preserving history/settings without retry, a malformed call after a completed creation retaining its file/journal without a partial chat turn, explicit profile reset and unchanged sibling settings. Restart retains exact history/profiles/files.
- Authorized routine **Qwen/Qwen3.5-2B**, 4096 output tokens/180 seconds/provider default: the completed diagnostic run (**98.77 s**, including manual approval wait) created two files and executed the approved validator, which failed. The generated CommonJS files contained invalid import/export usage and incorrect sums. The run paused at four model calls; work and failure receipts remain available. **This task is unfinished.** An earlier driver attempt was interrupted by a test-only decision-file read/write race before validation; it was excluded from this outcome and the decision handoff was made atomic for the fresh run. No hidden product retry occurred.
- Authorized harder **deepseek-v4.1-flash**, 8192 output tokens/180 seconds/explicit DeepSeek thinking off: the first run (**25.63 s**, four model calls) created a **7997-byte physics module and 6847-byte validator**, then paused at the step limit before executing validation. The probe allowed only its named files and exact Node validator; one folder listing was denied. Explicit Continue (**35.15 s**, including approval wait, two model calls) ran the separately reviewed validator successfully and returned a complete response without recreating files. Movement, overlap/edge-touch, zero dt, landing and 80 unique checkpoints were checked; the original paused segment is intact. This demonstrates one small coding slice, not Mario/Three.js or general coding reliability. A successful gateway response alone does not prove the reasoning control was honored; the first run's aggregate usage was unavailable.

Provider default omits reasoning overrides. DeepSeek off sends thinking.type=disabled with max_tokens; OpenAI low/medium/high send reasoning_effort and max_completion_tokens. Explicit DeepSeek thinking on remains unavailable until full reasoning-history replay is supported. Named settings errors expose Request settings, and malformed calls expose Model connection/Changes guidance. Neither settings nor budgets are silently changed or retried; unfinished calls never run. Existing step, context, file/diff and transport limits remain. Structured skill-draft output/timeout overrides retain the chosen reasoning adapter; other structured operations retain their smaller caps.

SQLite schema **17** adds one profile table without rewriting existing tables. No dependency or resident process was added. Alternative-shell profile UX, corrupt-database recovery, direct thinking-mode replay, broad gateway compatibility, macOS/Linux runtime and low-end measurements remain unaccepted. Coding validation/repair within explicit budgets is next (6.7).

The normal release was rebuilt and launched with a **confirmed visible window**. All **18 pre-existing table row digests** match the baseline; the new profile table is empty in the user's original data. Provider configuration/key, selected model, settings and history were preserved. The publishable allowlist scan covers **290 paths** with no known personal-path/email matches and no Gitleaks findings. Synthetic/live evidence and screenshots remain ignored.

## Brick 6.5 — Larger reviewed coding files

Create/edit JSON arguments now accept **64 KiB per call**; other tools retain **4 KiB**. The exact file, diff and edit snapshot remain limited to **16 KiB**. Complete arguments still require validation and a separate one-use approval before execution. Coding guidance asks for a small runnable slice, smaller files/exact edits and actual approved validation. It does not enlarge the existing 2048-output-token, 180-second, four-model-call/four-operation defaults. See [design and limits](design/larger-code-files.md).

- Rust workspace: **183 passed, 1 ignored** native-vault opt-in; formatting and all-target Clippy pass with warnings denied. Added cases cover fragmented escaped/Unicode arguments, exclusive file-tool limits, exact argument boundaries, file/diff overflow before approval, cancellation and saved receipts, and stale edit snapshots.
- A real approved relative Node script exposed a Windows working-directory defect: passing a canonical verbatim drive path to the child caused Node to fail while resolving the script. A regression in a Unicode/spaces folder failed before the fix and passes after it. Only the child directory spelling changes; canonical authorization identity, executable checks, environment filtering and process supervision remain intact. Drive/UNC spelling conversion has unit coverage; actual network UNC and very long paths remain unverified.
- Flutter: **138 tests passed**, analysis clean. Windows release diagnostics pass **129 checks at each of 1120×780 and 620×700**. The larger-file case shows the final line of the full Unicode/escaped diff before the file exists, then verifies exact saved content and a created receipt after approval. Wide/compact review captures were inspected. Native pointer, keyboard and IME UAT remain separate.
- `scripts/test-coding-files.py` passes save/restore in separate processes through bundled FFI, SQLite, an independent loopback provider and actual approved Node execution: **11 fixture model requests and 10 approval requests during save, zero model requests after restart**. The three-file slice includes a 5738-byte computation module and six executable checks. An oversized proposal is refused before approval and adapts to a smaller file; cancelling the second of two creations retains the first without saving a partial chat turn. Fresh approved reading/validation recovers the retained work without recreating it. Restart preserves six messages and five journal entries.
- Authorized routine **Qwen/Qwen3.5-2B** probe, 4096 output tokens/120 seconds: the first attempt created a 153-byte module with the wrong export and then returned invalid/incomplete tool arguments (**17.95 s**). Explicit recovery (**8.50 s**, four model calls) produced a blocked read and denied folder/command requests, without fixing or validating the files. Approval allowed only the named task files and exact validation command. **The task remains unfinished.**
- Authorized harder **deepseek-v4.1-flash** probe, 8192 output tokens/120 seconds: a two-file computation task paused at the output limit (**44.09 s**) without any file proposal. Provider usage reports **8192 reasoning tokens**, consuming the output budget. Explicit continuation at 16384 output tokens timed out (**120.02 s**), without new files or a saved continuation. The original paused response remains available. **The task remains unfinished.** These bounded probes expose model/budget limitations; they do not reproduce or establish success on the reported Mario/Three.js task.
- The normal Windows release was rebuilt and launched with a **confirmed visible window** for user verification. SQLite remains **schema 16**; all **18 table row digests** match the existing baseline. Provider settings/key and history were preserved. AGENTS.md now records the per-task visible launch requirement, and the normal launch helper checks visibility. The publishable allowlist scan covers **285 paths** with no known personal-path/email matches and no Gitleaks findings. Synthetic live/diagnostic data and screenshots remain ignored.

No dependency, schema migration or resident process was added. File/diff limits, restricted operations, missing general directory creation and model instruction-following still constrain larger coding work. Per-model output/reasoning/deadline configuration is next (6.6); unsupported provider settings must be handled explicitly. macOS/Linux and representative low-end runtime acceptance remain open.

## Brick 6.4 — Long-task recovery

The selected Flutter shell now saves a labeled paused response on explicit provider output-limit or agent step-limit completion. Partial text, usage, public model steps and completed tool receipts survive. Continue on the latest saved paused message appends a separate bounded run using current settings and fresh tool approvals. Unfinished calls never enter execution; Stop, timeout, transport and malformed responses retain their earlier failure behavior. Paused replies skip automatic preference learning. The existing 2048-output-token, 180-second, four-model-call/four-operation defaults remain unchanged. See [recovery design](design/long-task-recovery.md).

- Rust workspace: **177 passed, 1 ignored** native-vault opt-in; formatting and all-target Clippy pass with warnings denied. Checks cover bounded untrusted recovery context, incomplete streamed calls with same-frame final Unicode and trailing usage, fixed loop limits, durable pause/restart, atomic failed continuation commits and stale-source refusal. Structured skill/memory consumers still reject truncation.
- Flutter: **138 tests passed**, analysis clean. The compact light/dark recovery check additionally passed after the final missing-connection refinement; it verifies draft preservation, failed continuation with an empty composer, explicit retry, duplicate exclusion and current source binding. Windows release diagnostics pass **127 checks at 1120×780 and 620×700**, including real rendered Continue through FFI/SQLite. Both pause screenshots were inspected; native pointer, keyboard and IME UAT remain separate.
- `scripts/test-continuation.py` passes save/restore in separate processes through the bundled bridge, SQLite and its own loopback provider: **nine fixture requests during save and one after restart**. It covers repeated output limits, stale/wrong-session/modified requests rejected before model access, Stop after visible continuation text, three approved file creations followed by a step-limit pause, then fresh approved reading of saved work without recreating those files. The fourth pending creation stays absent. Restart and JSON export retain pause provenance.
- Authorized routine **Qwen/Qwen3.5-2B** probe: synthetic long inventory text paused at **128 output tokens** in **3.76 s**, retaining 530 characters; explicit continuation at **1024 tokens** paused again in **29.28 s**. Authorized harder **deepseek-v4.1-flash** probe: synthetic long game-component design text paused at **128 tokens** in **2.03 s**, retaining 80 characters; explicit continuation at **1024 tokens** paused again in **9.30 s**. Both used 90-second per-request timeouts, retained two separate segments and kept the original intact without errors. **These tasks remain unfinished.** Four explicit live operations establish recovery, not task completion or general coding competence. The reported game-building failure has not been reproduced.
- The normal Windows release was rebuilt and reopened. SQLite stays **schema 16**; all **18 table row digests** match the existing baseline. Original provider settings/key and chat history were preserved. The publishable allowlist scan covers **282 paths** with no known personal-path/email matches and no Gitleaks findings; synthetic data, live evidence and screenshots remain ignored.

Recovery is bounded by the existing context token/byte guards plus a 96-KiB prompt and 16-receipt preflight cap. Oversized accumulated progress requires a reviewed summary/new chat; receipts are not silently dropped. Continuation instructs the model to preserve completed effects but cannot guarantee every model follows it. There is no automatic retry or budget increase, no dependency or resident process, and alternative shells retain their previous strict UX. Larger file-output/task decomposition is next (6.5); macOS/Linux and representative low-end runtime acceptance remain open.

## Brick 6.3 — Multiple reviewed MCP connections

Project/Temporary folders can save four independently reviewed local servers, sharing two active external tool slots alongside the six built-ins. The inspector shows the server list and per-server status. Stable connection IDs separate aliases, credentials and one-use approvals; identical original tool names are supported. Disable/Forget affect only the selected server. Capacity refusal preserves the current review; inline Disable on another server frees a slot for an explicit retry. Servers remain on-demand and stateless.

Validation:

- Rust workspace: **174 passed, 1 ignored** native-vault opt-in; formatting and all-target Clippy pass with warnings denied. New cases verify byte-preserving schema-15 migration including credential/retirement references, independent revisions and atomic capacity/fifth-server refusal, colliding tool names, missing/misbound credentials before startup, and desktop JSON connection identity/default compatibility. The actual native test initially exposed a missing field-name mapping that replaced the first server; the fix has a JSON regression test and passed the rerun.
- Flutter: **137 tests passed**, analysis clean. Compact light/dark tests cover multiple selection, independent controls, failed Save with preserved review and inline sibling Disable followed by Enable without reinspection. Windows release diagnostics pass **125 checks at each of 1120×780 and 620×700**. The rendered flow adds two same-named tools, verifies distinct aliases, and forgets one while preserving the other. Wide/compact screenshots were inspected; native pointer, chooser, keyboard and IME UAT remain open.
- `scripts/test-mcp-multiple.py` save/restore passes in separate processes through bundled FFI, SQLite, actual stdio servers and a loopback provider (**four fixture model requests, zero live requests**). Capacity refusal followed by sibling Disable and same-token Enable succeeds. Both original `echo` tools route correctly; one server crashes during invocation and the other still completes in the same turn. Forget preserves the sibling identity/revision/alias; restart and context launch no process; exports retain both servers' provenance. The legacy credential diagnostic also passes save/restore (**seven fixture requests**), including actual OS-vault rotation/reuse/Forget and unchanged legacy aliases.
- Authorized routine **Qwen/Qwen3.5-2B** probe: the first wording returned a confirmation question without calling the tool (**3.28 s, one model call, 1744 input / 107 output**). A separate explicit tool-call wording passed (**2.51 s, two model calls, 68 combined output tokens**). This records a model instruction-following limitation; Dolores did not silently retry or switch providers.
- Authorized harder **deepseek-v4.1-flash** probe: first server intentionally crashes, then the second server succeeds and the final reply reports both outcomes (**7.53 s, three model calls, 555 combined output tokens**). Both live probes used **2048 maximum output tokens and 120-second request timeout** in isolated synthetic working chats. The configured provider/key were read only; the user's model, settings and history were preserved. These cases establish specific integration behavior, not general coding-task competence.

The existing four-model-call/four-operation limits and two active external slots are retained. Long-task progress preservation and explicit continuation are the next brick (6.4); the reported game-building failure has not been reproduced and its cause is not established by these probes. Persistent/remote servers, third-party compatibility, corrupt-configuration recovery, macOS/Linux runtime and representative low-end resource measurements remain open. New project-local AGENTS.md records the user's basic-plus-edge-case testing, model-selection and per-brick commit preferences.

- The normal Windows release was rebuilt and the existing preview reopened. Schema advanced from 15 to 16; all **18 table row digests** matched the baseline. The publishable allowlist scan covered **277 paths**, with no known personal-path/email matches and no Gitleaks findings. The machine-local CodeGraph index is excluded by its generated `.codegraph/.gitignore`; that ignore file is retained in this brick.

## Brick 6.2 — Secure MCP credential bindings

External tools now accepts explicit masked credential rows before inspection. Enable stores keys in the existing OS vault and only names/opaque references in SQLite. Values are scoped to the canonical folder, exact launch, reviewed file hashes and environment name. Inspection and approved calls forward only configured bindings; host secret variables are not inherited. Approval displays names only. Disable retains keys; rotation retires prior entries and Forget disables first, revokes keys and preserves references on a cleanup failure. Missing/locked/misbound entries require unlock or re-entry and a fresh review. See [credential design and limits](design/mcp-connection.md#explicit-credentials-brick-62).

- Rust workspace: **170 passed, 1 ignored** native-vault opt-in. All-target Clippy passes with warnings denied. New tests cover masked-value bounds/reserved names, duplicate bindings, launch-field refusal, nested metadata leaks, overlapping exact-value redaction, no vault read or process during preparation, approved forwarding, locked/missing/wrong-folder/wrong-launch refusal before startup, failed-write cleanup, explicit rotation and disabled retryable Forget. Focused MCP/bridge tests were rerun after final cleanup/redaction refinements.
- Flutter: **136 tests passed**, analysis clean. The credential test verifies obscured fields, disabled suggestions, explicit inspection values, token-only Enable, blank saved-name reuse and refresh/retry after failed Forget. Approval shows environment names without values. No composer/theme changes.
- Windows release diagnostics: **122 checks at each of 1120×780 and 620×700**, including the rendered masked credential row and actual native-vault Enable/Disable/Forget. Compact screenshots were inspected; pointer, native chooser, keyboard and IME UAT remain separate.
- Native save/restore through bundled FFI, SQLite, supervised Node stdio and a loopback model fixture passed both with and without credentials: **seven fixture requests per save**, zero live-provider requests. Checks include Deny without startup, approved credential echo redaction, context/send parity, changed metadata refusal, Stop, rotation, restart without startup, explicit saved-key reuse, changed-launch refusal and key-free database/model/history/export/fixture-log text. An independent Windows credential-existence probe confirms active entries exist and rotation/Forget remove the actual prior entries without reading their blobs. Synthetic entries from diagnostics were removed.

SQLite remains **schema 15**; legacy credential-free records retain their serialization shape. No dependency, resident server, general environment editor or background vault scan was added. All **18 existing preview tables** retained identical row counts/content digests after rebuilding and reopening the normal release. Privacy scanning covered **274 publishable paths**, with zero Gitleaks findings and zero known personal-path/email matches; diagnostic data and screenshots remain ignored.

Vault/SQLite cleanup is recoverable but cannot be a distributed atomic transaction; failed rollback cleanup or an interrupted save can leave unused vault entries, reported when detected. Exact-value redaction does not prevent a trusted external program from encoding/splitting/transmitting credentials. macOS/Linux native-vault/runtime acceptance, third-party server interoperability, multiple/persistent/remote connections and representative low-end measurements remain open.

## Brick 6.1 — Reviewed local MCP tools

One installed local stdio server can be explicitly inspected and enabled per Project/Temporary working folder, with up to two selected tools alongside the six built-ins. Chat actions exposes the review; every invocation retains a separate Run once/Deny decision with exact JSON arguments. Side chats omit MCP. Servers stop after inspection/call and never start during context, preparation, saving, restart or idle chat. This begins brick 6's plugin ecosystem; the umbrella retains broader gaps. See [design and protocol limits](design/mcp-connection.md).

- Rust workspace: **167 passed, 1 ignored** native-vault opt-in. All-target Clippy passes with warnings denied. Added protocol tests cover initialization/version/tools capability, pagination, ping and unsupported server requests, malformed/unmatched/crashed/flooded output, changed lists/metadata, exact one-use approval, file hash changes, cancellation and dropped-run descendant cleanup. A shortened deadline exercises the same timeout path; ordinary operations have a 30-second limit. Image content and text over 8 KiB are refused. `isError` stays explicit.
- Storage and bridge tests cover folder isolation, disabled tools, restart, revision conflicts, atomic failed writes, expired/wrong-session/replayed tokens and retained review after failed save. SQLite **schema 15** adds one `mcp_connections` table without rewriting existing tables.
- Flutter: **134 tests passed in the full suite**, plus the added compact light/dark approval-card test passed in the focused six-test MCP run (135 tests total across these runs). Analysis is clean. Tests exercise literal arguments, no automatic selection/enable, the two-tool limit, enable/disable/forget, edit invalidation, failed-save retry, Stop and visible one-use approval buttons with long arguments.
- Windows release: **120 checks at each of 1120×780 and 620×700** passed. New rendered flow inspects the actual stdio fixture, selects/enables one tool, and disables/forgets it; MCP selection/enabled screenshots were inspected. Pointer, native chooser, keyboard and IME UAT remain separate.
- Separate native save/restore passes through bundled FFI, SQLite, actual supervised Node stdio server and a loopback model fixture: **seven model-fixture requests**, zero live-provider requests. Checks include preview/send parity, Deny without startup, exact approved call, untrusted server instructions excluded from context, private environment excluded, changed metadata blocked before call, Stop without a partial turn, explicit re-enable, cancelled inspection, restart without startup and durable provenance/export.

The adapter adds SHA-256 through an already locked dependency and reuses the supervised command transport; no server/runtime is bundled. Discovery/calls are bounded to 30 seconds; the adapter accepts text results only and advertises no client capabilities. External programs have user-account filesystem/network permissions, can act during inspection and can leave effects after Stop. Hash/metadata checks detect changes in reviewed inputs; they do not seal dependency graphs or eliminate races between checks and execution. Calls start a fresh server and cannot preserve server-session state. MCP credentials, remote/multiple/persistent connections, third-party interoperability and macOS/Linux runtime remain open. No measured low-end performance or universal plugin compatibility is claimed.

The normal release was rebuilt and reopened after diagnostics. All **17 pre-existing preview tables** retained identical row counts/content digests across migration; schema is now 15 and the new connection table is empty. Configuration/history were preserved. Privacy scanning covered **273 publishable paths**, with zero Gitleaks findings and zero known personal-path/email matches. Local evidence, launch paths and diagnostic data remain ignored.

## Brick 5.3 — Portable reviewed skills

Skills → Saved versions → select a retained version → Export SKILL.md now opens a native Save dialog. Choose or create a matching skill-name folder and save SKILL.md. Export publishes the exact retained document with original YAML, Unicode and line endings. It works in Project/Global scope, including disabled/older snapshots and missing sources. Unsaved drafts/unretained file reviews cannot use this action. Nothing is activated, uploaded, executed or copied beyond that document; resources and historical evaluation receipts remain separate. See [export design](design/skill-export.md).

- Rust workspace: **157 passed, 1 ignored** native-vault opt-in; all-target Clippy passes with warnings denied. New checks cover exact standard-file reimport, generated document construction, existing/empty/directory targets, invalid paths/metadata, racing exports, temporary-file cleanup, selected old version, wrong-session/token, expiry/cancellation/changed saved state, both scopes and unchanged activation. A dangling-link refusal test is supplied for Unix but was not executed on Windows.
- Flutter: **129 tests passed**, analysis clean. Added compact light/dark checks cover selected version/path payload, no activation, missing source, cancellation without publication, global side-chat scope, pending dismissal/duplicate exclusion and failed export retaining review for retry. Status messages return into view after layout; the existing composer/keyboard/IME controller regressions remain passing.
- Windows release: **117 checks passed each at 1120×780 and 620×700**. The rendered export flow cancels without writes, exports exact version 1 while version 2 remains active, refuses an existing destination with no temporary files, then separately activates the older review. Wide/compact review and refusal captures were inspected. The picker callback supplies an isolated destination; native Save dialog interaction and actual OS pointer/keyboard/IME UAT remain unverified.
- Separate native save/restore: `scripts/test-skill-export.py` passes with **zero provider requests**. Export preserves all SQLite table contents, old disabled project snapshots and global side-chat snapshots with deleted sources. Copying the standard document into another project makes it discoverable but waits for explicit activation. Restart preserves exact text, disabled state and schema 14; no transcript records or resources are exported.

No dependencies, database migration, background worker or idle process is added. The 8-KiB document / five-version bounds remain. Existing files are always refused; overwrite review is outside this brick. This is document portability, not acceptance of referenced scripts, external harness execution, general skill quality or macOS/Linux behavior. The normal release is rebuilt after diagnostics and the preview's existing 17 tables, configuration, credentials and history are preserved. Publishable source/evidence remain separated and privacy-scanned before commit.

## Brick 5.2.2 — Draft JSON contract

The reported error was reproduced with the configured **deepseek-v4.1-flash**, using one synthetic selected exchange and the native Engine/provider path. Its response was complete, syntactically valid JSON with `evidence` as an object. The previous compact drafting instruction explicitly requested that object, while SkillDraft required an array; the shared error mislabeled the schema mismatch as invalid JSON. This was a Dolores prompt/parser conflict, not evidence of a DeepSeek-specific stream failure.

The prompt now requests an evidence array and includes an explicit JSON shape serialized from SkillDraft itself. Parsing distinguishes syntax from required-format failures while retaining original-text typed deserialization, duplicate/unknown-field refusal, source quote matching and all size/credential/activation checks. No normalization, provider-specific parameter, model retry, dependency, database migration or UI layout change is introduced.

- Rust workspace: **154 passed, 1 ignored** native-vault opt-in; all-target Clippy passes with warnings denied. The initially failing prompt-contract test passes through the production parser. Additional tests distinguish object/array, unknown and duplicate fields from malformed JSON, keep invalid exact evidence refused and avoid echoing untrusted field values in errors.
- Native separate-process save/restore: **21 loopback requests** verify truncated-output refusal, explicit larger retry, 31-second drafting, unchanged saved settings, frozen response checks, explicit one-use promotion, Stop, preserved transcript, restart and rollback. Flutter widget/layout code is unchanged; its previous checks are not claimed as rerun here.
- Live DeepSeek: **two authorized requests total**, with identical synthetic sources and 4096-token / 120-second allowances. Before the fix, the draft was rejected in **7.27 seconds** (297 input / 1162 output / 1459 total tokens). After the fix, a complete array-shaped draft was accepted without metadata warnings in **8.37 seconds** (366 input / 1291 output / 1657 total tokens). Both finished with `stop`; no skill was saved or activated. A loopback diagnostic proxy preserved the request payload and forwarded it only to the original credential-bound endpoint; captured evidence stays ignored and contains no request headers or key. This is one successful synthetic drafting case, not universal adherence or task-quality acceptance. Older generic JSON-error records did not distinguish syntax from schema failures.

The normal bundled Rust release is rebuilt and the existing preview is restored with its configuration, credentials and history preserved. Schema stays 14. See [draft design](design/skill-drafts.md).

## Brick 5.2.1 — Skill-draft output limits

The reported failure was reproduced through Engine → HTTP SSE → provider collection: a saved 4096-token limit became 1024 during drafting, and the simulated provider returned `finish_reason: length`. Generation now starts with saved model settings and exposes per-draft output-token/timeout overrides without changing those settings. Prompt guidance asks for a compact complete JSON proposal. Input capacity is checked with the chosen output reservation before transport; response/document bytes stay bounded to 8 KiB. A length failure names its actual allowance, discards incomplete text and retains selected sources for explicit retry. No automatic retry, budget increase or activation occurs. The shared review collector honors the provider deadline; memory/evaluation retain their existing smaller limits.

- Rust workspace: **152 passed, 1 ignored** native-vault opt-in. The initially failing real-request regression now passes for saved 4096 tokens, a deliberately insufficient 1024 tokens and an explicit 8192-token override. Invalid limits are refused before transport; saved settings and empty skill storage are checked. All-target Clippy passes with warnings denied.
- Flutter: **125 tests passed**, analysis clean. Added compact dialog checks verify model-setting defaults, limit validation without sharing sources, retained selection/token after failure and a single explicit larger retry. Errors return into view after the layout update; fields lock during requests. Existing pending/Stop and composer regressions pass.
- Windows release: **110 existing checks passed at each of 1120×780 and 620×700**; source/limit dialog screenshot inspected. These are rendered widget/controller/FFI checks, not OS pointer/keyboard acceptance.
- Separate native save/restore passes on isolated data: **21 HTTP requests**, including a truncated 2048-token proposal, no silent retry, invalid-limit refusal, explicit 4096-token retry and a successful **31-second** draft under a 90-second timeout. Saved model settings remain 2048 tokens / 180 seconds; frozen response comparisons, promotion, Stop, transcript preservation, restart and rollback still pass.
- Authorized **Qwen/Qwen3.5-2B**: one synthetic generation with an explicit 8192-token / 180-second allowance completed in **4.29 seconds** without an output-limit error, but returned invalid JSON. No skill was saved or activated. This does not establish usable live drafting; structured-output adherence remains open independently of the cap fix.

Schema 14 and dependencies remain unchanged. The normal release is rebuilt after diagnostics; configuration, credentials and all 17 existing preview tables are preserved. Publishable source scan: **260 paths**, zero Gitleaks findings and no known personal-path/email matches. Evidence and provider data remain ignored. See [draft design](design/skill-drafts.md).

## Brick 5.2 — Reviewed skill drafts and response evaluation

Skills now offers Draft from chat in Project/Global scope: explicitly select useful completed exchanges, generate an evidence-backed proposal, correct it, define frozen response tests, inspect baseline/candidate outputs and explicitly Activate tested skill. Promotion requires every candidate test to pass and strictly more passes than baseline. This is a literal response-check foundation, not general task/tool outcome evaluation. Generated versions and bounded historical receipts stay in local SQLite; files and chat history are unchanged. See [design](design/skill-drafts.md).

- Rust workspace: **151 passed, 1 ignored** native-vault opt-in; all-target Clippy passes with warnings denied. Added validation/evidence/literal-score tests, expiry/scoped source review, cancelled/replaced publication guards, pre-provider saved-entry limits with same-name replacement, frozen atomic storage/CAS, forced failure, legacy side-chat compatibility, restart and historical receipt retention on rollback. Existing parsing/approval/token-budget regressions remain passing.
- Flutter: **123 tests passed**, analysis clean. New compact light/dark flows cover explicit source selection, corrected request payloads, separate promotion, score ties, edit/test invalidation, failure retaining corrections, pending-call exclusion and Stop. A repairable-metadata test caught inherited source-list scroll position hiding the notice; generation now returns the editor to the top. Existing composer keyboard/selection/IME controller tests remain passing.
- Windows release: **110 checks passed each at 1120×780 and 620×700**, including source selection, unsaved proposals, baseline/candidate gate, edit invalidation and saved receipts entering future context with unchanged source/history. Compact editor/test/score and wide saved receipt captures were inspected. Diagnostic callbacks exercise rendered Flutter widgets, FFI, HTTP and SQLite; native OS pointer/keyboard/IME interaction is not covered.
- Separate native save/restore processes pass with **20 loopback HTTP requests** in the save stage. Exact frozen requests match provider wire; usage persists; ties/regressions refuse promotion; successful activation is one-use; Stop sends no later candidate request; a concurrent local skill write invalidates the snapshot. Generated versions have no file dependency; reviewed rollback after restart retains the exact historical receipt. Transcript and schema 14 remain unchanged.
- **Live Qwen/Qwen3.5-2B: two one-request synthetic drafting probes did not produce usable drafts**—one had invalid skill metadata, one malformed JSON. Neither saved or activated a skill. Bounded, quoted drafts with repairable metadata now open for correction, while testing still requires a valid document; malformed JSON/evidence stays refused. No successful live generation/evaluation/promotion acceptance is claimed, and no automatic repair/retry is introduced. Real-provider draft quality remains open.

No dependencies, persistent workers, watchers or idle processes are added. One generation request is bounded to 1024 output tokens / 30 seconds, and 1–3 tests use at most six 512-token / 10-second requests, or lower configured limits. All requests are tool-free and budget-checked; unchanged approval rules govern future chat tools. Receipts retain exact tested requests/responses, settings, window, usage and source evidence within the existing five-version / 12-entry limits, with optional legacy-compatible JSON. Normal preview configuration/credentials/history are preserved and the normal release is reopened after diagnostics. Evidence/provider data/screenshots remain ignored and publishable files are privacy-scanned before commit. Automatic skill selection, portable export, representative task evaluation, low-end resource acceptance and other platforms remain open.

## Brick 5.1.1 — Global skills

The Flutter host now discovers `~/.agents/skills/<name>/SKILL.md` on demand and offers Project / Global scope selection. Global reviewed snapshots apply to project, temporary and side chats in this application's data directory. Enabled same-name project entries override global entries; Disable/Forget reveals the global version. Both scopes retain independent revisions, five recent versions and explicit review/activation. Global collection junctions/symlinks are supported, with canonical-target binding during file review; individual skill links remain refused. Global activation grants no new tool or filesystem access. See [design](design/project-skills.md).

- Rust workspace: **145 passed, 1 ignored** native-vault opt-in; all-target Clippy passes with warnings denied. New checks cover legacy default/serialization, global provenance and project override/fallback, independent storage/restart/CAS/rollback/Forget, internal namespace isolation, global side-chat preview, collection junction support and retargeted-boundary refusal even with identical text. Unix symlink cases remain locally unexecuted.
- Flutter: **118 tests passed**, analysis clean. Three additional checks cover scope-isolated actions, side-chat default/disabled Project selection, and opening Skills through the actual side-chat action while preserving chat-switch exclusion. This final regression caught the inherited working-folder guard; Skills now uses the existing settings guard. Compact light/dark lifecycle and pending-write tests still pass. Longer limits/resource guidance is expandable.
- Windows release: **105 checks passed each at 1120×780 and 620×700**, including global activation, same-name project override/fallback, side-chat inheritance and global Disable/Forget with unchanged source files. Compact side-chat Global and wide Global source captures were inspected. Diagnostic callbacks scroll lazy skill rows into the widget tree when needed; these checks cover controller/renderer/FFI/HTTP/SQLite, not native OS pointer/keyboard/IME interaction.
- Separate native save/restore processes pass on isolated loopback data with **four HTTP requests**. Global exact preview equals side-chat provider input, with no tools advertised; saved/exported provenance includes scope and a logical home-relative source, without sending the actual directory. Tests cover changed global source, separate revisions, project override, reviewed version update/rollback, missing source, Disable/re-enable, restart and Forget. Existing explicit tool-denial and project persistence checks remain.

Limits apply per scope: three enabled skills / 8 KiB combined, 12 saved entries including disabled, five versions per entry. Effective context may contain up to six skills / 16 KiB, subject to existing mandatory-input token/byte limits. Schema stays 14, using a disjoint internal global namespace in the existing table; no dependencies, watchers, workers or processes are added. Normal preview data is compared before/after reopening, and the bundled bridge is compared to the release artifact. Local evidence/screenshots/provider data remain ignored; publishable source is scanned before commit. Generated skills, automatic selection, global resource tools and broader platform/resource acceptance remain open.

## Brick 5.1 — Reviewed project skills

The selected Flutter host now reviews standard `.agents/skills/<name>/SKILL.md` files and explicitly activates folder-scoped snapshots. Disable preserves versions; reviewing an old version then Activate version records a rollback as a new version without changing project files. Forget removes future retrieval/local versions while preserving source files and historical reply provenance. Changed or missing sources leave reviewed snapshots usable. Shared preview/send counts every active skill in mandatory system input. YAML metadata and resource references cannot approve tools or run/load files. SQLite schema 14 adds independent folder/name records. See [design](design/project-skills.md).

- Rust workspace: **141 passed, 1 ignored** native-vault opt-in; all-target Clippy passes with warnings denied. Checks cover YAML multiline/quoted metadata, duplicate/mismatched names, empty/binary/invalid/oversized source, direct paths, Windows junctions at each directory level, bounded/partial listing, exact source/state/session/expiry/single-use review, version retention, rollback, scope, active count/byte/saved limits, forced transaction failure, restart and chat deletion. Unix symlink cases are compiled only on Unix and remain locally unexecuted.
- Flutter: **115 tests passed**, final focused skills/request-controls/tool regressions **23 passed** and analysis clean. New compact light/dark checks exercise exact literal review, no pre-activation writes, missing-source saved-version review, Disable/Forget, rollback, pending-write exclusion, stale-source retained state, Close/cancel and context provenance. A header-overflow regression led to placing Skills in Chat actions below 480 pixels; other desktop widths retain the direct action.
- Windows release: **102 checks passed each at 1120×780 and 620×700** through controller/renderer/FFI/HTTP/SQLite, including six skill checks for explicit activation, exact/countable context, changed-file snapshot stability, version publication, reviewed rollback and Disable/Forget. Wide dark and final compact light/dark source/rollback captures were inspected. Native OS pointer/keyboard/IME interaction is not exercised.
- Separate native processes: `scripts/test-project-skills.py save` / `restore` pass on isolated loopback data with **three HTTP requests**. The exact preview equals first provider input and saved token/provenance metadata. A skill containing `allowed-tools: everything` still requires a separate file approval; Deny shares no fixture file body. Tests also cover wrong-session/replayed/stale source review, folder/side isolation, versioned update/rollback, missing source, disable/re-enable, export, chat deletion, restart and Forget. Python fixture writes explicitly preserve LF; a Windows CRLF mismatch was test setup, while the Rust reader preserves actual source bytes.

Storage stays bounded to 12 records/folder and five recent versions/record; activation refuses beyond three entries/8 KiB combined. Discovery/review is on demand, with no worker, watcher, model call or global scan. YAML parsing adds `serde_yaml_ng` and its `unsafe-libyaml` dependency; no new runtime process is introduced. The normal app is rebuilt without diagnostic mode and its existing schema 13 data is verified before/after additive migration to 14. Provider keys/configuration/history remain preserved; local evidence and screenshots stay ignored. Publishable snapshot: **258 paths**, zero Gitleaks findings; the one known-path regex match was the generic `/users/` privacy-filter literal, rather than personal data.

This is the skill foundation. Generated proposals from successful work, automatic selection, outcome-improvement measurements and reliable model adherence remain future acceptance; no live model claim is made for this brick. Low-end resource acceptance, macOS/Linux execution and OS input/accessibility checks remain open.

## Brick 4.4 — Automatic preferences

Automatic learning is enabled in the selected Flutter host, with a global Memory switch for manual-review mode. After successful reply persistence, one eligible completed user message can produce up to three exact-quote preferences using the active model. Working chats save in the current folder; side/legacy chats save in All chats. Same-topic replacement requires an explicit correction and an untouched enabled automatic entry. Manual/reviewed/edited/disabled entries are protected. Activity, separate reported usage, provenance, Edit/Disable/Delete and context selection stay inspectable. Stop/failure preserves the saved reply. SQLite schema 13 atomically compares source/policy/applicable preferences and claim; interrupted/failed/deleted evidence never replays automatically. See [design](design/automatic-memory.md).

- Rust workspace: **135 passed, 1 ignored** native-vault opt-in; all-target Clippy passes with warnings denied. Frozen core/store checks cover verbatim qualifiers, conservative English/Chinese eligibility, false-positive refusal, duplicate/conflict handling, automatic correction, manual protection, scope, changed source/policy/preferences, cancellation, replay, multi-candidate forced rollback, restart and deletion. Existing memories migrate without rewriting their data; default-false `autoUpdate` is additive.
- Flutter: **110 tests passed**, analysis clean. Compact light/dark policy tests cover persisted toggle/revision, latest activity/reported usage, failed-toggle retained state and no preference write. Existing manual/reviewed/summary/composer tests remain intact. Renderer review caught automatic provenance mislabeled “reviewed”; learned entries/context/reply details now label their source accurately, and identical quote/body is shown once.
- Windows release: **96 checks passed at 1120×780 and 620×700** through controller/renderer/FFI/HTTP/SQLite, including automatic exact-quote learning, next-context retrieval and disabling/deleting it. Wide/compact automatic Memory captures were reviewed; final compact verification also covers provenance-label refinement. Native OS pointer/keyboard/IME interaction is not exercised.
- Separate native processes: `scripts/test-automatic-memory.py save` / `restore` pass on fresh synthetic loopback data (**35 requests** including ordinary replies). Checks cover exact source-only extraction plus declared existing preference fields, no replies/tools/absolute root on learning wire, output cap 512, lower timeout, ineligible/off/mandatory-capacity refusal without an extra request, scoped learning, duplicates/conflicts/correction/protection, forged/paraphrased/malformed/empty/oversized output, raw provider denial redaction, Stop/concurrency, saved-reply retention, future context, complete export with memory provenance, restart and deletion. Extraction compatibility retains the provider's one explicit unsupported-usage resend; there is no inference/output failure retry.
- Authorized **Qwen/Qwen3.5-2B**: the first probe learned its initial preference but omitted the explicit correction. An extraction example clarifying correction semantics fixed that observed case. The revised probe saved both exact user statements; extraction reported **349 input / 43 output / 392 total** and **372 input / 79 output / 451 total** tokens. Initial learning/fresh-chat reply/correction completed in **2.52 / 3.80 / 4.23 seconds**. Corrected text appeared verbatim in another fresh chat's prepared context. A longer follow-up hit its 1024-token output limit after **28.64 seconds**, preserving completed learning; one separately requested short follow-up completed in **1.02 seconds** (**200 input / 28 output / 228 total**) and recalled detailed explanations with two examples. It paraphrased the preference with stronger “always/exactly” wording; that reply wording was not written to memory. These are specific learning/retrieval observations, not universal adherence or semantic quality. Normal connection/key/history remained read-only; evidence is ignored.

Publishable snapshot: **250 paths**, zero Gitleaks findings and no known personal-path/email matches. Provider evidence, native screenshots and isolated/normal data remain ignored. Normal-preview schema migration is checked against fingerprints of all pre-existing tables before reopening.

Eligibility/topic/sensitive-pattern filters deliberately miss unsupported phrases and languages and cannot universally classify private or inappropriate facts. Same-topic ambiguities/protected conflicts skip replacement. Preferences remain local plaintext and subordinate to the current user/workspace/host policy; no permission or execution capability is granted. There is no additional idle worker or new package; the SQLite crate now directly declares the already-shared cancellation library. Automatic summaries, broader habit inference, OS input/accessibility, macOS/Linux and low-end acceptance remain open.

## Brick 4.3 — Reviewed session summaries

Saved chats have a header Summary action. Review the exact next contiguous oldest batch (up to 20 complete user/final-assistant pairs / 144 KiB), generate a tool-free draft, correct it and explicitly Save summary. Extensions include the previous reviewed summary; editing preserves coverage and deletion restores ordinary newest context. One session-scoped summary replaces only covered raw turns in future shared preview/send, with separately counted coverage and token/byte budgeting. Full history/export stays intact. SQLite schema 12 adds one cascading summary record; frozen source/prior-summary checks and optimistic revisions make replacement atomic. No dependency or idle worker is added. Automatic memory updates are explicitly expected and deferred as requested. See [design](design/session-summaries.md).

- Rust workspace: **132 passed, 1 ignored** native-vault opt-in. Added core/store/bridge checks cover bounded Unicode/credential-like content, source ordering, context precedence/coverage, contiguous extension, stale source/prior revision, wrong-session/replay/expiry refusal, forced write failure, correction/deletion, restart and cascading chat deletion. All-target Clippy passes with warnings denied.
- Flutter: **108 tests passed**; compact light/dark summary tests exercise literal source review, explicit correction/Save/Edit/Delete, failed-save retained text, Discard/Close no-save, Stop with dismissal blocked and no retry, and disabled generation for empty history. Analysis is clean after enclosing four single-line state updates in blocks. Native screenshot review caught milliseconds being interpreted as seconds in the review date; the display now uses the storage unit and the lifecycle test checks its calendar date.
- Windows release: **93 checks passed each at 1120×780 and 620×700** using actual controller/renderer/FFI/HTTP/SQLite. New summary checks verify local locked review, no turn writes while generating, exact corrected context with covered history absent, and deletion restoring recent context without losing history. Wide/compact dark source/draft/saved captures were inspected. Native OS pointer/keyboard/IME input is not exercised.
- Separate native processes: `scripts/test-session-summary.py save` / `restore` pass on fresh synthetic loopback data. Exact wire contains the displayed pairs and prior summary only, without tools or saved preference text. Effective output/timeout limits, mandatory model-capacity refusal before transport, wrong-session/replay, forced insert failure with retained candidate, exact corrected Save, stale source/prior refusal, extension, oversized/empty/credential-like output, Stop/concurrency/deadline, discard, preview-to-wire-to-saved-accounting equality, complete exports, restart and deletion are checked. Historical metadata retains coverage/provenance without copying summary bodies.
- Authorized **Qwen/Qwen3.5-2B**: four isolated calls completed in **1.06 / 2.09 / 2.80 / 1.21 seconds** (source reply, initial draft, a second draft after a local review invalidated the first candidate, and corrected follow-up). The second draft retained the synthetic codename, literal-matching decision and pending tests; extraction reported **208 input / 92 output / 300 total tokens**. Corrected Save replaced the covered turn, and the follow-up included all three requested details (**140 input / 33 output / 173 total tokens**). Full history remained available. This is one synthetic continuity probe, not general summary accuracy or memory-learning acceptance. The normal provider/key/history were read-only; raw evidence stays ignored.

Publishable source snapshot: **245 paths**, zero Gitleaks findings and no known personal-path/email matches. Ignored provider evidence, private data and screenshots are excluded from the source snapshot.

Credential-pattern screening is limited, and completed replies can quote private file contents; sharing is disclosed before Generate. Summaries are fallible local plaintext, not executable instructions or permissions. Existing supported history is append-only; external database edits after Save are outside source-freshness guarantees. Native input/accessibility/system-theme UAT, macOS/Linux runtime/CI, exact tokenization and representative low-end resource acceptance remain open. Existing working-set measurements exceed the provisional target. Diagnostic entry points are rebuilt to normal mode before reopening the preserved preview.

## Brick 4.2 — Reviewed conversation preferences

Memory now offers Suggest from this chat. Review the latest 20 completed messages written by you, explicitly select 1–6 within 8 KiB, and generate up to three tool-free drafts using the active model. Only selected source text is shared. Each draft must cite an exact selected quote; review and correct its title/text and scope before Save preference. No automatic memory or conversation writes occur. Five-minute tokens bind source/session/folder/candidate, and SQLite checks all frozen sources and inserts the corrected preference in one transaction. Save failures preserve the candidate; successful Save is single-use. Host provenance records the original quote/model/message and survives editing, disabling, chat deletion, restart and export. Schema 11 and dependencies are unchanged. The standard provider may resend the same approved text once without a rejected usage-reporting option, under the same deadline; failed inference/output is never retried automatically. See [design](design/memory-suggestions.md).

- Rust: **129 passed, 1 ignored** native-vault opt-in; all-target Clippy clean. New core/store/bridge checks cover strict JSON/selected quotes, invalid IDs/duplicates/credential-like text/limits, complete user-only bounded history, source changes, manual-origin refusal, scope/replay/write failure, source-preserving correction and cancelled hung/oversized output. Independent review found a cancellation/publication race; a failing commit-boundary regression reproduced late review publication, and cancellation checking under the publication mutex plus cancel-before-clear shutdown ordering fixes it. The final regression preserves any newer review. These checks validate citation identity, not the semantic truth of generated preference wording.
- Flutter: **104 tests passed**, analysis clean; the final focused suggestion suite **4 passed**. Compact light/dark tests cover explicit source selection/sharing, no auto-save, source/correction review, failed-save retained draft, cancellation/no retry, empty results, closing/discarding, one successful Save and unavailable-source labels. No new palette/composer behavior or idle worker.
- Windows release: **89 checks passed each at 1120×780 and 620×700** through actual controller/renderer/FFI/HTTP/SQLite. New checks verify zero preselection, no preference/turn writes during extraction, and explicit corrected Save with an exact source quote. Wide and compact dark review/results captures were inspected; existing light-theme Memory and compact light/dark widget checks pass. These use widget callbacks, not OS keyboard/IME/pointer interaction. The enlarged regression diagnostic exceeded its former 60-second wrapper budget after reaching final settings; a 90-second run passed. The diagnostic permits up to 120 seconds while each model timeout stays unchanged. A foreground fixture ended across an interrupted turn; the rerun used a fresh owned fixture.
- Separate native processes: `scripts/test-memory-suggestions.py save` and `restore` pass against isolated synthetic loopback data. Wire input contains only extraction instructions and exactly selected user text, with no tools/replies/AGENTS.md/saved preferences/root; effective lower output/timeout settings are respected. Checks cover malformed/oversized/invented-quote/credential-like output, empty results, Stop/concurrency/deadline, wrong session, forced insert failure with retry, one-use corrected Save, stale-source refusal, source quote excluded from retrieval input, historical metadata/export, restart, disable, unavailable deleted-chat source and preference deletion.
- Authorized **Qwen/Qwen3.5-2B**: three isolated calls completed in **1.99 / 1.96 / 0.28 seconds**. Extraction returned one valid source quote, with **242 input / 62 output / 304 total reported tokens**, and saved nothing until explicit correction. Its draft broadened a greeting-only preference into an unconditional response rule, demonstrating that quote validation is insufficient for semantic accuracy. The corrected preference and origin were retrieved and persisted exactly, but the follow-up returned an ordinary greeting rather than the corrected marker (**297 input / 3 output / 300 total tokens**). This verifies transport, review, correction and provenance; it does **not** establish reliable model adherence or general learning usefulness. Normal connection, credential and history were read-only; raw evidence stays ignored.

Publishable source snapshot: **238 paths**, zero Gitleaks findings and no matches for the known personal-path/email filters. Ignored fixtures, configured-provider evidence and captures are excluded; these checks are bounded scans, not a universal privacy guarantee.

Source quotes remain local plaintext and in historical reply details/exports after source chat or preference deletion; this is disclosed before Save. Credential screening is limited and user review remains necessary. No universal model, semantic, secrecy or learning guarantee is claimed. Native input/accessibility/system-theme UAT, Linux/macOS runtime and CI, representative low-end resource checks and exact tokenization remain open; earlier Flutter working-set measurements exceed the provisional target. The normal app is rebuilt after diagnostic entry points with existing preview data preserved.

## Brick 4.1 — Manual preferences with provenance

Sidebar Memory is available in every chat mode. Explicitly save a short preference for All chats or the current saved working folder, inspect/edit it, disable/re-enable it or delete it. Entries carry Added by you, scope, revision and local timestamps. Save is the activation step; conversations, tool data and model text never create entries automatically. SQLite schema 11 adds scoped records without rewriting history. Optimistic revisions and transactions refuse stale/wrong-scope edits and deletion. Shared preview/send retrieval includes complete entries under a 4-KiB serialized-entry budget, folder first/newest/stable ID; full system text counts against existing byte/token limits. Context shows exact selected entries and omissions, while retained metadata keeps provenance without duplicate preference bodies or automatic absolute roots. See [design](design/user-preferences.md).

- Rust workspace: **123 passed, 1 ignored** native-vault opt-in. Tests cover Unicode limits, deterministic selection/whole-entry bounds, complete newest-turn trimming, stale revision/wrong scope, per-scope caps, failed SQLite update/delete rollback, migration/restart/deleted-chat retention and bridge isolation. All-target Clippy passed with warnings denied.
- Flutter: **100 tests passed**, including compact Memory Save/Cancel/Close, locked scope, literal text, correction/disable/delete, pending/failed-save draft retention, no-folder inspection and exact context disclosure. Analyzer reported no issues. No dependency or new idle worker was added.
- Windows native controller/renderer: **86 checks passed each at 1120×780 and 620×700** against the loopback HTTP fixture, including Memory exclusion while open, Cancel/Close no-save and disabled-context behavior. Inspected actual light/dark saved-preference and compact edit captures; actions remain accessible. These diagnostics use callbacks and rendered widgets; actual OS pointer/keyboard/IME/accessibility acceptance remains open.
- Isolated native frozen wire/restart: `scripts/test-memory.py` verified manual creation only, exact preview/provider messages and token/provenance snapshot equality, All chats versus folder/Side/temporary isolation, correction and disabled/deleted exclusion, historical provenance/export retention, schema 11 and separate-process restart. A saved preference claiming all tools are approved still produced a read approval; Deny shared no file contents. References remained literal. The large synthetic set verifies whole-entry selection and omission at the provider boundary.
- Authorized live **Qwen/Qwen3.5-2B**: one isolated Side-chat greeting used a manually saved preference and returned the exact synthetic marker; 194 provider-reported input tokens, 9 output, 203 total. Local estimate was 206 input tokens and remained separate. Saved context/provenance matched preview. This demonstrates one explicit saved-preference flow, not general memory usefulness or reliable adherence across prompts/models. Normal provider settings, credential and chat history were read-only.

Manual preferences are plaintext local data; entry sharing is explicit, with no automatic secret harvesting or classification. Delete stops future retrieval but does not rewrite earlier replies/provenance or promise forensic erasure. Database corruption may require local repair. Raw data, screenshots, endpoints and probe output remain ignored under `output/`. Gitleaks found no secrets in the publishable source snapshot; the targeted known personal-path/email scan had no matches. These scans do not guarantee every possible personal detail is recognized. Low-end performance, Windows/macOS/Linux CI and OS input/theme UAT remain open; earlier Flutter working-set observations exceed the provisional target. Conversation-derived suggestions and automatic learning are future work.

## Brick 3.9 — Reviewed workspace instructions

Working-folder headers now open a literal root `AGENTS.md` review with explicit Enable/Disable. A single-use five-minute token binds activation to the same session/folder/current text. Additive schema 10 saves one bounded 8-KiB UTF-8 snapshot per folder; chats in that folder and restarts reuse it. Changed/missing/unloadable enabled files block the next preview/send before a provider call, with a manual Instructions action and restored draft. Shared preparation counts guidance, retains whole newest turns, and records source/revision/time/byte provenance in saved replies/exports without automatically exposing absolute folder paths. Tool approvals remain independent. See [design](design/workspace-instructions.md).

- Rust workspace: **118 passed, 1 ignored** native-vault opt-in; all-target Clippy clean. New checks cover root-only text, literal includes, missing/binary/invalid/oversized files, whole-turn budgets, idempotent host tool guidance, wrong/expired/repeated/cancelled/stale activation, folder/Side isolation, failed-save rollback and snapshot restart/delete behavior. Unix no-follow/link checks compile conditionally; Unix runtime remains unverified.
- Flutter: **95 passed**; four instruction checks also pass after the final loading/error-state refinement, analysis clean. Compact light/dark checks cover literal review, Close/cancel without activation, save exclusion, stale review requiring Refresh, missing-file Disable, chat locking, meter invalidation and visible provenance. Native keyboard/IME/accessibility interaction remains a separate acceptance gap.
- Windows diagnostic release: **83 checks passed at each of 1120×780 and 620×700**. Existing renderer/controller/FFI/HTTP/SQLite checks plus explicit activation, counted/literal guidance, stale-file refusal and disable recovery pass. Wide/compact dark instruction review captures were inspected; the body scrolls and footer actions remain separate. These checks exercise callbacks/rendering, not OS pointer/keyboard/IME input.
- `scripts/test-workspace-instructions.py save` then `restore` pass in separate processes with isolated keyless loopback data. Actual provider messages/schemas match the inspected context; `@../private.env` stays literal and the referenced sentinel never enters input. Guidance declaring all tools approved still produces a separate read decision; Deny prevents file contents from being shared. Changed guidance sends no extra HTTP call or saved messages. Source revisions survive complete JSON/Markdown export, folder changes and restart; deleting the file blocks preview, and Disable restores sending context without losing history.
- Authorized **Qwen/Qwen3.5-2B**: the first generic greeting completed in **0.51 seconds**, with the instruction snapshot included, but returned an ordinary greeting instead of the guidance marker. A second prompt explicitly asking to use enabled workspace guidance passed in **0.47 seconds**, one model call and no tools, with **1364 estimated initial tokens / 1530 reported input / 8 output / 1538 total**. Saved context/provenance matched preview. These synthetic probes establish transport and a specific prompted response, not universal instruction-following reliability. Personal provider settings, key and history stayed unchanged; private artifacts remain ignored.
- Publishable snapshot: **226 paths**, zero Gitleaks findings. No runtime dependency, automatic discovery, network counting, watcher or idle worker is added. Resident instruction state is bounded to one pending review and one request's snapshot; no new low-end performance acceptance is claimed.

Root instructions are explicitly enabled and folder-scoped. Ancestor/nested precedence, include expansion and automatic instruction generation are outside this brick. Review text may itself contain private information; the full text precedes activation, and no automatic secret-classification guarantee is made. Existing native-input/platform/CI/reference-device acceptance remains open. The normal release is rebuilt after diagnostics with existing preview data preserved.

## Brick 2.5 — Token context and per-model capacity

The composer ring and context inspector now use tokens, keeping local estimates separate from provider-reported usage. Model connection exposes a context-window override for each enabled model; blank uses the user-requested **128K (131072 tokens)** fallback. Additive schema 9 stores endpoint-scoped overrides atomically with connection/model/key metadata. Preview and send share the Codex-style UTF-8 heuristic, schema/framing accounting, response reservation and 5% headroom. They retain newest whole turns; mandatory overflow fails before a request, and subsequent tool growth is checked without orphaning call/result pairs. Byte guards remain internal resource limits. See [design](design/token-context.md) and [pinned Codex/DSH comparison](research/token-context-accounting.md).

- Rust workspace: **114 passed, 1 ignored** native-vault opt-in; all-target Clippy clean. Coverage includes Unicode/rounding, unknown plugin capacity, complete-turn trimming, fixed schema overflow, tool-result overflow before another model call, preview/send/saved-snapshot agreement, legacy preservation, model/settings switches, restart, invalid profiles, atomic failed-save rollback and secure-entry cleanup. Recognized provider context-overflow codes produce fixed sanitized guidance without retry across plain and tool requests.
- Flutter: **91 passed**, analysis clean. Compact light/dark settings verify independent model edits, whole-number validation, blank-default persistence, Cancel/draft preservation and capacity restoration. The ring uses token capacity rather than byte limits, labels estimated versus reported readings, restores only the latest call total and invalidates mismatched model/capacity previews. Composer keyboard/editing regressions still pass. Focused settings/recovery checks cover manual model/output-settings actions for context failures.
- Windows release: **79 checks passed at each of 1120×780 and 620×700**. Actual renderer/controller/FFI/HTTP/SQLite flows verify selected capacity/framing/reservation, inspectable prepared messages, active-model settings and existing tool/history flows. Wide/compact token-inspector and scrolled model-window captures were inspected. These checks do not exercise OS pointer/keyboard/IME input.
- `scripts/test-token-context.py save` then `restore` pass in separate processes against isolated keyless mock data. They verify 1024-token projection/trimming, exact saved estimates separate from reported 64-input/32-output usage, refused oversized input without new messages, full exports, six advertised tool schemas, null-override 128K fallback and restoration of both model capacities after restart.
- Authorized configured **Qwen/Qwen3.5-2B** ordinary-chat probe completed in **0.43 seconds**, with **55 estimated initial tokens versus 54 provider-reported input / 8 output / 62 total**. The 32768-token override and initial estimate matched saved metadata. An earlier tool probe chose an unrequested listing instead of the requested read; that listing was denied and no read was accepted. No live tool-planning reliability claim is made by this brick. The real provider/key/history stayed unchanged; private probe artifacts remain ignored.
- Publishable snapshot: **218 paths**, zero Gitleaks findings and no matches for personal-path/email/identity/debug-marker filters. No tokenizer/model downloads, runtime dependencies, idle worker or remote counting request is added.

Estimates do not guarantee exact fit for every provider/tokenizer. The 128K fallback is a product default, not discovered model capacity; configure the real limit when it differs. No automatic summarization/compaction is included. Native input/accessibility, Linux/macOS runtime, CI and representative low-end resource acceptance remain open. The normal release is rebuilt after diagnostics with existing preview data preserved.

## Brick 3.8 — Reviewed command execution

Working chats now advertise `run_command` through a separate compiled plugin. The local review shows the resolved executable, saved working folder, individually quoted literal arguments and user-permission/effect disclosure before Run once/Deny. Initial programs are direct installed git/node/python/python3/cargo/rustc/dart executables; Windows batch/shell expansion is unavailable. Each invocation is limited to 30 seconds and 8 KiB combined output; serialized results stay within 16 KiB and preserve stdout/stderr, actual exit code and shortening/UTF-8/read-error flags. Commands are not sandboxed and their effects are outside Changes; stopping a reply or failing its save does not reverse effects. No schema migration or idle service is added. See [design and platform boundaries](design/approved-commands.md).

- Rust: **108 tests passed, 1 ignored** native-vault opt-in; all-target Clippy clean. Seven command-plugin checks cover literal empty/space/quote/backslash/shell-character/Unicode args, working folder, filtered environment, nonzero exit, invalid/tampered/consumed proposals, pre-cancellation, executable resolution, deadline, raw/escaped output bounds, invalid UTF-8, cancellation/dropped futures with descendants, and parent completion with inherited descendant pipes. Core denial tests bind exact arguments and public records omit automatic executable paths.
- Flutter: **87 tests passed**, analysis clean; focused approval suite **12 passed** after the final disclosure move. Compact light/dark command tests check explicit Deny/Run once, exact args/local paths, permission/effect disclosure, literal result text and output-shortening labels. Initial expansion hit a bool/double storage-state collision between the expansion control and new selectable argument text; a separate argument storage key fixes the causal regression in both themes.
- Windows desktop release: **77 checks passed at each of 1120×780 and 620×700**. Rendered/controller/FFI/HTTP/SQLite flows cover command preview before execution, Deny, Stop at approval, Unicode stdout/stderr/nonzero exit and saved args on reload, unchanged file journal, output-limit cleanup and Stop during execution with retained prior effects/restored draft/no partial history. Compact dark review capture was inspected; disclosure is visible before arguments in the bounded scroll area. An initial diagnostic pressed a control before its rendered frame enabled it, and another mock repeated 8-KiB output past an 8-second response setting; frame synchronization and a short fixture summary corrected these diagnostic failures.
- `scripts/test-command-tool.py save`/`restore` pass in separate processes using isolated keyless fixture data. Repeated decisions are refused; no execution on Deny/Stop at approval; exact saved args, exit/stderr/Unicode and capped output survive restart/export without automatic executable/root leakage. Forced chat-save failure preserves the already executed file effect while history remains atomic and Changes stays empty. Running Stop leaves its prior marker but no late file after restart. The diagnostic checks cancellation while event batches are empty, rather than waiting for a tool result.
- Authorized configured **Qwen/Qwen3.5-2B** probe passed in **2 model calls, 27 public text chunks, 3.88 seconds**. An exact independently reviewed Node invocation read one synthetic fixture marker, returned exit 0 and printed the expected marker; saved arguments/result and complete exports matched. Approval accepted only that exact invocation. Personal provider/history stayed unchanged; endpoint/key and private test artifacts are excluded from Git. This proves that flow, not universal command-planning reliability.
- Publishable source snapshot: **213 paths / 1.67 MB**, zero Gitleaks findings and no matches for the personal-path/email/identity/debug-marker filters. Local fixtures, captures, real-provider evidence and benchmarks remain ignored. Deliberately configured Git author identity is outside this source scan.

Windows is the local command/runtime acceptance target. Unix process-group code is implemented but has no local Linux/macOS execution evidence here; deliberately detached/external processes are outside its cleanup guarantee. Native pointer/keyboard/IME/folder-picker/accessibility UAT, CI execution and representative low-end measurements remain open. Command CPU/memory usage is not capped by these output/time limits. Future sandboxing must be designed separately from the current working-directory behavior.

## Brick 3.7 — Reviewed new files and recorded removal

Working chats can now create a small UTF-8 file in an existing directory after reviewing its complete addition and choosing Create once. Publication refuses an occupied path, including a competing entry during approval. Schema 8 records explicit existence, preserving empty files distinctly from absence and migrating schema-7 replacements without changing snapshots. Changes labels Created/Edited/Removed; reverting a recorded creation opens a separate deletion preview with Remove once. No directory creation, arbitrary model deletion or shell tool is added. See [design and concurrency boundaries](design/approved-creation.md).

- Rust: **100 tests passed, 1 ignored** native-vault opt-in; all-target Clippy clean. Creation coverage includes unchanged absence until approval, complete Unicode/CRLF/missing-newline text, empty files, occupied files/directories, invalid/secret/device paths, bounded arguments, tampered/consumed approval, cancellation, alias directories, intent failure, a competing writer during intent and receipt failure after publication. Removal verifies saved bytes before and after intent, failure preserving external content, empty-file absence and honest pending receipts. Store/bridge checks cover schema-7 migration, restart/deleted-chat retention, explicit existence, root/session binding, Cancel/conflict/single use and refusal to reverse a removal.
- Flutter: **85 tests passed**, analysis clean. Added compact light/dark checks verify Create once/Deny, complete literal diff, zero-byte result, Created status and a separate Cancel/Remove once flow. Existing edit, composer, sidebar, history and inspection checks pass.
- Windows desktop release: **70 checks passed at both 1120×780 and 620×700**. Actual rendered/controller/FFI/HTTP/SQLite flows cover creation preview, Deny, Stop, an occupied pending destination, exact Unicode/CRLF publication, existence-aware journal metadata, Cancel/removal and empty files. Wide/compact dark approval/removal captures were reviewed. These do not exercise OS pointer/keyboard/IME/folder-picker interaction.
- `scripts/test-file-creation.py save` and `restore` in separate native processes pass with isolated keyless fixture data. Forced intent failure prevents creation; failed final chat-save retains the applied file and independent record; failed creation receipt remains pending; empty creation survives chat deletion/restart. Stale/cancelled/conflicting removal is refused. Reviewed removal is single-use and its own receipt failure reports actual file absence with pending status. No transient staging files remain in the successful fixture.

The configured **Qwen/Qwen3.5-2B** completed a strictly diff-matched synthetic creation → final flow in two model calls, 26 public-text chunks and about 2.66 seconds. A local reviewed removal consumed the saved creation and preserved both receipts without another model call. The first probe proposed a file without its requested trailing newline; the harness preserved that proposal exactly, and the test rejected the content mismatch. The final probe uses exact content and full-diff approval. Model output still requires review; this verifies the specific flow, not universal reliability. Source connection/history stayed unchanged, credentials remained in memory and raw outputs remain ignored.

Evidence is under ignored `output/file-creation/` and `output/live-qwen/`. The 206-path publishable snapshot passed secret and targeted personal-path/email scanning. The normal production app was rebuilt after diagnostics; its bridge matches the completed release and the existing preview data was preserved. Publication was exercised on Windows NTFS; unsupported hard-link filesystems refuse creation rather than overwrite. Removal's optimistic checks are not cross-process atomic CAS, and universal ACL/xattr guarantees, native input/accessibility, macOS/Linux and low-end/resource acceptance remain open. Per-file/page memory is bounded but journal disk retention grows. No additional dependency, idle task or background worker was introduced.

## Brick 3.6 — Change journal and reviewed revert

The selected Flutter host records local before/after snapshots independently of the reply and exposes folder-scoped Changes in the header. A FULL-synchronous intent is saved before a file is published. Intent failure prevents the edit; receipt failure after publication still reports applied bytes and displays Needs check. Review revert verifies the current file, shows its own literal reverse diff and requires Revert once. It preserves a separate revert record and never infers that an empty or pending record proves execution. See [design, retention and concurrency boundaries](design/change-journal.md).

- Rust: **92 tests passed, 1 ignored** native-vault opt-in; all-target Clippy clean and the alternative Tauri host compiles. Coverage includes snapshot limits/UTF-8 byte counts, root isolation, 20-row keyset paging, restart/deleted-chat retention, conversation exports without journal snapshots/root, atomic revert receipts, failed intent/receipt, cancellation or external writes during intent, direct-path/current-byte checks, conflicting/stale/wrong-session/expired/consumed approvals and generation exclusion.
- Flutter: **81 tests passed**, analysis clean. Compact light/dark checks cover separate review/apply decisions, Cancel/Close, pending disclosure, conflict without retry, page replacement and exclusion of other chat actions during inspection. A regression diagnostic covers independently visible reply details in the compact lazy transcript.
- Final Windows release: **62 checks passed at both 1120×780 and 620×700**, using a bundled bridge verified byte-identical to the completed build. Added checks exercise actual Changes controls, reverse-diff preview without mutation, Cancel, exact restoration and original/revert receipts. Wide/compact dark captures were reviewed. These are renderer/controller/FFI/HTTP/SQLite checks, not OS pointer/keyboard/IME/folder-picker UAT.
- Separate native processes through `scripts/test-change-journal.py` verify a forced final-chat-save failure retaining an applied edit; a forced receipt failure retaining pending intent and applied bytes; an intent failure preventing publication; deletion preserving records; restart exposing records from a new chat in the same folder; stale/cancelled/conflicting/repeated approval refusal; and exact reviewed restoration without partial history. Use the local keyless fixture and a fresh absolute directory under ignored `output/`.

The configured **Qwen/Qwen3.5-2B** completed a synthetic read → separately approved edit → final flow in three calls (190 public-text chunks). The applied edit appeared in Changes; a local reverse preview and explicit revert restored the marker and both receipts without another model request. The configured source connection/history were not changed; its existing credential was used in memory with isolated launch-only test data. This verifies that specific flow, not universal provider/model reliability.

One first native attempt used a button helper that did not handle list-row taps. A compact attempt also failed an older check that assumed both reply-detail widgets were mounted simultaneously. Both saved models were correct; a focused compact regression proves lazy rendering exposes each model when scrolled into view, and the diagnostic now does that. Product rendering was unchanged. A stale library copy made before compilation completed was replaced; final diagnostics use the completed binary. Evidence remains ignored under `output/change-journal/` and `output/live-qwen/`. The 201-path publishable snapshot passed secret and targeted personal-path/email scanning.

Schema 7 is additive; pre-existing edits have no invented snapshots. Journal data is unencrypted and retained after chat deletion, with bounded individual snapshots/pages but growing disk retention. Revert covers original replacements only, not creation/deletion or reversing a revert. Optimistic filesystem checks, pending receipts after crashes, ACL/xattr limits, cross-platform packaging, accessibility/native input and low-end/resource acceptance remain explicit limits. Normal production release is rebuilt after diagnostics and the existing preview data is preserved.

## Brick 3.5 — Reviewed file edits and live Qwen verification

Working chats now offer one exact unique replacement in an existing writable UTF-8 file. The local approval card shows a bounded literal diff with Apply once/Deny; no edit is applied during preview. Files changed during approval are refused, while accepted edits stage/flush/atomically replace through the held directory capability. Optional diff metadata and status `edited` preserve old schema-6 records. Applied file effects remain after later chat failure/Stop; this is stated on the card. There is no rollback, durable failed-run edit journal, file creation/deletion or shell in this brick. See [design and concurrency/platform limits](design/approved-edits.md).

- Rust: **85 tests passed, 1 ignored** native-vault opt-in; Clippy clean and the alternative Tauri host compiles. Edit coverage includes unique/overlapping Unicode matches, CRLF/missing final newline, unsafe/binary/readonly/oversized paths/content, unchanged bytes until invocation, changed snapshots, alias redirection, tampered preview/single use, cancellation, failed atomic publication cleanup, denied proposals and safe validation feedback followed by a separately approved retry.
- Flutter: **75 tests passed**, analysis clean. New compact light/dark checks verify the literal colored diff, explicit Apply once, saved diff/byte counts, conflict and Stop without granting permission. Existing composer and sidebar behavior passes.
- Native release: **58 checks passed at both 1120×780 and 620×700**, including preview without mutation, Deny, changed-file refusal, Stop and applied bytes/diff restoration. Reviewed dark approval captures show the complete short diff and both decisions at each size. These are renderer/controller/FFI/HTTP/SQLite checks, not OS pointer/keyboard/IME UAT.
- Separate native processes verify edit-diff JSON/Markdown export, restart restoration and a forced chat-save failure: the applied file remains while the complete-pair history transaction rolls back. No partial conversation is saved.

The configured **Qwen/Qwen3.5-2B** was used only with isolated synthetic folders and launch-only test connections, using the existing key in memory. The live streaming list → search → read flow completed in four model calls with the correct marker, 191 public-text chunks, per-call usage, saved history and exports. Denied read recovery passed; Stop at a live approval saved no turn. A bundled release probe also completed read → reviewed edit → final in three calls (129 public-text chunks), applying the synthetic change and saving its diff. These are specific compatibility observations, not a success-rate or universal model guarantee. Some earlier probes ended before the expected approval or returned unapproved/blocked proposals. Their exact argument causes were not captured; no model-reliability fix is claimed. Fixed local validation feedback now distinguishes argument shape, missing/ambiguous matches and size limits so a valid retry can still require its own approval.

Diagnostics remain ignored under `output/approved-edits/` and `output/live-qwen/`; no endpoint, credential, personal path or raw live response is published here. The 192-path public-source snapshot passed secret scanning and the targeted personal-path/email scan. One initial desktop attempt was started before rebuild completion and locked library copying; it was discarded. Another reached a saved ordinary reply but checked the UI before asynchronous history refresh finished. Native completion waits now include both generation and history refresh; product controller behavior was unchanged. OS pointer/keyboard/IME/folder-picker, macOS/Linux, ACL/xattr preservation and low-end/resource acceptance remain open.

## Brick 3.4 — Streaming agent progress

Project/temporary replies now stream public text throughout tool-enabled runs. Commentary appears before approval and moves into expandable Agent progress when the next call begins. Successful metadata, chat, trajectory and exports preserve intermediate commentary separately from final text. The provider assembles indexed/fragmented function calls within byte/frame limits; only complete validated calls enter the existing approval flow. No hidden reasoning, automatic tool-free fallback, dependency, worker or migration is added.

- Rust workspace: **77 passed, 1 ignored** native-vault opt-in; all-target Clippy clean; alternative Tauri host compiles. New coverage includes fragmented Unicode/interleaved calls, incomplete/malformed/reused/oversized arguments, stream frame/wire budgets, public text before completed calls, no partial approval/invocation, cancellation/blocked delivery/deadlines, generic errors without retry, explicit unsupported-usage fallback and separate final text/usage snapshots.
- Flutter: **73 tests passed**, analysis clean. New compact checks in both themes verify commentary during approval, saved progress expansion, final-text separation, step changes, stale-text exclusion, Stop/draft recovery and clearing transient progress. Existing composer/sidebar/history/inspector tests pass.
- Native release: **53 checks passed at both 1120×780 and 620×700**. The real renderer/controller/FFI/HTTP/SQLite path verifies public text before argument completion, streaming final text, restored progress, interrupted arguments without approval, Stop while arguments stream and previous working-session/discovery behavior. Approval/progress captures were reviewed. These are not OS pointer/keyboard/IME/folder-picker checks.
- Separate native processes verify per-step deltas without a duplicate final delta, saved final text/usage, complete JSON/Markdown exports, automatic root exclusion, a forced metadata-save failure rolling back the entire turn, restart restoration, interrupted calls and argument-stream timeout without saved turns, then successful denied-read recovery.

Evidence remains in ignored `output/agent-streaming/`. The first wide diagnostic failed an existing visible reply-details assertion; instrumented wide and compact reruns passed with two enabled detail controls and correct saved models. No root cause or product fix is claimed for that intermittent observation. Live model compatibility, platforms, accessibility, input and resource targets remain unverified. Normal production release is rebuilt after diagnostics and the existing preview data is preserved.

## Brick 3.3 — Working sessions and grouped sidebar

The selected Flutter shell now offers project chats with saved folders, default temporary chats with distinct app-managed folders, and explicit side chats without file access. The sidebar follows the supplied reference with a quiet New chat action/menu, collapsible Projects containing nested project chats and per-project + actions, and Recents containing temporary/side chats. Shared history paging remains visible beneath the grouped scroll area. The header shows the working folder/mode and keeps the model selector in the composer. Legacy chats remain intact as side chats. Folder associations survive restart, are authoritative for tools, and never cause automatic working-file deletion.

- Windows Rust workspace checks: **67 passed, 1 ignored** native-vault opt-in; all-target Clippy clean; alternative Tauri host compiles. Tests cover schema-6 legacy preservation, private/atomic folder identity, unique temporary roots, registered projects, immutable scope and exports without automatic absolute paths.
- Flutter: **71 tests passed** and analysis clean. Coverage includes default tools and explicit approval, per-chat/controller-restart restoration, unavailable/pending creation without a tool-free send, grouped/collapsible Projects and Recents in both themes, new temporary/side actions, history pager visibility and the existing composer keyboard/editing regressions.
- Native release diagnostics: **48 checks passed at each of 1120×780 and 620×700** after the sidebar refinement. Actual Flutter renderer/controller callbacks, Rust FFI, HTTP fixture and SQLite verify default temporary approval, separate temporary roots, tool-enabled ordinary prompts, restored project/temporary roots and existing list/search/read/Stop/export behavior. Grouped and collapsed wide/compact dark captures were reviewed. These do not exercise OS pointer/keyboard/IME or the native folder dialog.
- Separate native processes verify saved project/temporary identity and keyless fixture recovery, automatic approved file reads with session ID only, explicit side streaming, root-redirection rejection, missing-folder failure without a fallback turn, exports without automatic folder paths, working-file preservation on chat deletion and a forced folder-association save failure rolling back both database rows and its newly empty temporary folder.

Diagnostics and synthetic folder/database/export artifacts remain in ignored `output/working-sessions/`. No new resource measurements or live-model/platform acceptance claims are made. Per-operation approvals, bounded read/search limits and read-only tools remain; streaming agent replies, actual folder-picker/input/system-theme accessibility UAT, macOS/Linux and representative low-end measurements remain open.

Flutter is the selected default. Its earlier Windows working-set measurement exceeds the provisional 150 MiB target; choosing it does not establish performance acceptance. **Release acceptance remains open** for representative hardware, sustained interaction/startup, real keyboard/IME input, screen-reader support and other platforms. This is not yet a self-learning agent.

## Brick 3.2 — Folder listing and text search

Folder-enabled runs now register listing, literal search and full-file read plugins against one directory capability. Each valid operation requires its own Allow once/Deny before listing names, scanning text or reading a full file. Search discloses its exact query, scope and scan limits; a search hit cannot authorize a later full read. Denied requests match tool, target and exact query within one run. Model/file text cannot supply approval. Root-relative result paths preserve scoped folder prefixes, so a discovered file can be read without guessing its path. No writes, regex, background index, dependency or new worker is added.

Listing inspects at most 512 entries and returns at most 100 sorted names from that bounded subset. Search visits at most four subfolder levels, inspects 256 entries, attempts 64 file reads and reads at most 256 KiB total, with a 16-KiB per-file cap. Up to 30 matching lines have 240-character Unicode snippets; both discovery results stay within 16 KiB serialized text. Counts and partial/truncation labels distinguish bounded coverage from a complete search. Binary, invalid UTF-8, oversized/unreadable files, links, common credential/VCS and generated/dependency paths are skipped. Filename filtering does not identify every sensitive file. Cancellation checks run on the existing blocking pool; an OS operation already executing cannot be forcibly interrupted.

Shared approval/result cards now distinguish each operation, keep Allow/Deny accessible below bounded scrolling text, and display readable selectable names or file/line/snippet results instead of raw JSON. Search queries have a distinct saved-state key: compact testing caught an expansion-state/scroll-position type collision, which is fixed and covered by the same test. Names, queries and snippets persist unencrypted/export with completed replies; old records lacking query remain compatible with schema 5. Four model calls/four total tools and existing run/answer/context limits remain in force. A listing → search → read → final answer fits the fixed model-call budget.

Validation: 65 Rust tests pass (one native-vault test remains opt-in), all-target Clippy is clean and the alternative Tauri host compiles. All 67 Flutter tests pass and analysis is clean. New checks cover exact query/denial scope, legacy records, literal case-sensitive Unicode matches, returned subfolder prefixes, hidden/generated/binary exclusions, actual Windows junction escape, canceled operations and depth/entry/file/byte/match/serialized-result limits. Compact widget checks exercise Allow/Deny for both new operations in both themes, query disclosure, readable partial results and expansion.

Windows release diagnostics pass 44 checks at each of 1120×780 and 620×700 logical sizes. The real HTTP/FFI/controller flow lists a folder, separately approves a search with its exact query, separately approves the discovered file read, and saves a final answer with four model-call snapshots and three records. Reload, denied search and Stop while waiting preserve expected history/draft behavior. Listing/search/result captures were inspected in both sizes. Separate processes verify complete JSON/Markdown trace export, exact Unicode/query provenance, a forced final-save rollback, restart restoration, pending-search timeout cleanup and absent folder authority on the next launch. Normal production release is rebuilt after diagnostics and the existing preview data is preserved.

Evidence stays in ignored `output/folder-discovery`, including isolated `ffi-restart-verified` data. All 175 publishable paths pass local secret/personal-path checks. Tests use a local fixture and rendered/controller callbacks, not a real language model or OS folder-picker/pointer/IME interaction. Live tool compatibility, macOS/Linux, accessibility and representative low-end/resource acceptance remain open; no new memory-performance improvement is claimed.

## Brick 3.1 — Approved text-file tools

Flutter now supports opt-in folder tools for the current launch. The core owns typed tool/provider/approval ports and an explicitly registered bounded loop. The only compiled tool reads a relative regular UTF-8 text file after a run-bound Allow once/Deny decision. It uses an opened directory capability, validates resolved names, rejects common credential/VCS/device/traversal paths and outside symlink/junction targets, and caps content at 16 KiB. The filename filter is not a general secret detector. Model/file instructions cannot grant approval. Stop, timeout and stale/repeated decisions cannot consume an approval for another run; denied targets do not cause repeated prompts within the same run. No writes, shell execution, remembered approvals or external plugin loading are implemented.

Tool mode uses standard non-streaming Chat Completions function messages; ordinary chat retains streaming. Four model calls/four tools, 4-KiB arguments, 256-KiB provider JSON, 128-KiB serialized context/model text and the smaller of configured timeout/300 seconds bound the phase including approvals. OS filesystem operations already executing cannot be forcibly interrupted; preparation/read tasks use the existing blocking pool and canceled queued work checks its token. Initial folder/history preparation and final persistence remain outside the phase deadline. The initial context preview includes local tool guidance; saved context/ring readings distinguish this initial input from later file-result requests. No price budget or model token-window claim is made.

Successful replies retain expandable literal tool results in chat and trajectory, and per-call provider usage in reply details without an inferred aggregate. Approved contents go to the selected endpoint and remain unencrypted in completed metadata and JSON/Markdown exports; folder roots are excluded. Schema 5 needs no migration for the optional agent record. Final user/reply/metadata save together; failed, stopped and exhausted runs retain only transient visible results, without publishing partial conversation pairs. The composer footer order and shared theme remain unchanged.

Validation: 60 Rust tests pass (one native-vault test remains opt-in), all-target Clippy is clean, the alternative Tauri host compiles, all 66 Flutter tests pass and analysis is clean. New checks cover unknown/replayed/malformed calls, fixed budgets, exact/idempotent initial guidance and complete-pair trimming, denied prompt suppression, injected file text, UTF-8/binary/oversized reads, actual Windows junction escape, single-use/stale decisions and Stop/timeout cleanup. Widget tests cover Allow/Deny in compact light/dark layouts, result expansion, Stop racing a decision and launch-only folder selection without background I/O.

Windows release diagnostics pass 37 checks at both wide and compact logical sizes, including the real rendered Allow control, approval before file sharing, saved/reloaded Unicode records, Deny, Stop, blocked outside paths and loop exhaustion. Captures were inspected. Separate processes using the bundled C ABI verify exact initial context, per-call usage, full JSON/Markdown trace export, a forced failed-save rollback, restored trace, approval timeout recovery/cleanup and absent folder authority after restart. Restart testing exposed missing approval-timeout recovery classification; fixed local timeout guidance is now covered by regression and native checks. One concurrent diagnostic run failed an existing slow-stream assertion without a captured cause; the isolated rerun and final serial checks passed. No root-cause fix is claimed for that intermittent observation.

Evidence stays in ignored `output/tools-final-verified` and `output/tools-native-final/ffi-restart`. These tests exercise rendered/controller/FFI/HTTP/SQLite behavior, not native folder-picker, pointer/keyboard/IME interaction or a real language model. All 172 publishable paths pass the local secret scan and personal-path checks; raw captures, test file contents and machine paths stay ignored. Live model tool compatibility, macOS/Linux, low-end/resource and release UAT remain open; no new memory-performance claim is made.

## Brick 2.4 — Request controls and recovery

Flutter's Request settings dialog saves an output-token limit (default 2048, integer range 1–32768) and whole-response timeout (default 180 seconds, range 1–900). Defaults restore locally until saved. Schema 5 persists nonsecret settings independently of connection/credentials. A successful save activates a validated replacement provider; failed validation/storage preserves the previous state. Model switching, recovery and Forget preserve the settings. Completed replies retain the actual settings used, so later changes cannot alter their history/export details; older replies remain unavailable.

The response deadline includes headers, body, the bounded usage-option compatibility attempt, and queued delivery to Flutter. History reads and final atomic persistence remain outside it. A regression reproduced a stalled window suspending the adapter deadline because event forwarding stopped polling the provider; the bridge now bounds streaming plus delivery, and the paused-consumer test passes. Cancellation remains prompt and incomplete replies are discarded. Failure cards provide fixed local guidance and explicit Retry message/Request settings/Model connection actions as appropriate. Retry sends the current edited draft; no automatic failure retry was added.

Validation: 50 Rust tests pass (one native-vault test remains opt-in), all-target Clippy is clean, and the alternative Tauri host compiles. All 63 Flutter tests pass, including six settings/recovery checks; final analysis is clean. The final Windows release diagnostic passes 29 checks at both 1120×780 and 620×700 logical pixels. These include actual outgoing max_tokens, a mid-stream timeout with no partial saved pair, preserved draft, explicit retry of edited text, saved per-request settings, and unchanged composer footer geometry. Settings/recovery captures in both sizes were inspected; both themes and 390-pixel forms/error cards are covered by widget tests. Two separate processes using the bundled production C ABI verify saved settings/provider restoration, exported metadata, timeout atomicity, original snapshots after edits, and Forget retaining settings/history. Evidence stays under ignored `output/request-controls`; diagnostic checks exercise rendered widgets/controller/FFI/HTTP/SQLite, not OS pointer/IME input. All 163 publishable paths pass the local secret scan. No new resource-performance or live-provider/platform acceptance claim is made.

## Brick 2.3.2 — Usage and context visibility

UI refinement: the keyboard-help label is removed. Inside the composer's bottom-right corner, the model picker is followed immediately by a context ring and Send/Stop. The ring names its reading's source in its tooltip and measures the app's UTF-8 byte budget; unknown readings stay static and draft edits invalidate a preview. Its on-demand inspector shows a composition bar, exact byte breakdown, turn counts and expandable selectable system/history/draft text from the actual prepared message sequence. The header trajectory action opens independently paged saved exchanges and a live Log of submitted/prepared/first-text/saved/Stop/failure observations, with original model, local timestamps and monotonic elapsed time. The Log holds at most 200 in-process events across conversations, coalesces deltas, and contains no prompts, credentials, raw errors or network payloads. It is not a durable audit log or a provider timing trace.

Refinement validation: all 57 Flutter tests pass; 10 focused Rust bridge tests pass; Flutter analysis and bridge all-target Clippy are clean. New tests exercise compact light/dark context disclosures, exact Unicode source, known/unknown ring semantics, inline footer order, independent history paging without draft/scroll changes, live inspection without generation-time history reads, original-model/session association, Stop/failure restoration, saved-reply versus history-refresh failure labels, retention bounds, deletion and no fetch on typing. The Windows release diagnostic has 24 checks at wide and compact sizes, including the actual footer geometry, exact prepared system/history/draft sequence, system-text expansion and trajectory/log rendering. Evidence stays under ignored `output/context-inspector/final-wide` and `final-compact`; these checks exercise rendered widgets/controller/HTTP/FFI/SQLite rather than OS pointer/IME input. Final grammar and history-refresh labels are additionally covered by the final Flutter tests and normal release rebuild. The 159 publishable paths pass the local secret scan.

The release captures exposed a stale tree-shaken icon subset: new trajectory/context icons were absent even though Dart code had updated. The build helper now invalidates only generated Windows release-asset stamps before building. The icon subset is rebuilt with tree shaking retained; an actual bundled-font check fails before the fix and passes afterward for timeline, subject and list-alt glyphs (3916-byte subset). No SDK or user data is modified. Resource/platform/input acceptance remains open; no new memory-performance claim is made for this refinement.

The initial usage implementation: completed Flutter replies show provider-reported input/output tokens and a details dialog for nullable totals, cached input, reasoning, the original request model and context. Missing counters remain unavailable; genuine zeros remain zero. Its original on-demand Context control previewed the current draft with latest saved history, regardless of the viewed page, without provider I/O or writes. It displays included/saved/omitted turns and exact UTF-8 text bytes, including system/current-message content. The existing 40-complete-turn/128-KiB app bounds remain unchanged and are distinguished from unknown model token windows. Older turns remain saved; no token estimates, pricing or session aggregate is claimed.

Optional provider/storage capabilities preserve existing plugin implementations. The adapter requests usage, accepts final empty-choices chunks and retains the last valid snapshot without summing duplicates. Invalid or missing counters remain null. Only an explicit, bounded HTTP 400/422 unsupported-usage-option rejection permits one pre-stream compatibility attempt; generic validation, denial, transport and mid-stream errors are not retried. Schema version 4 adds cascading per-reply metadata, saved atomically with a complete turn. Older history is preserved, legacy replies omit metadata, and JSON/Markdown exports retain the new records. Failed saves/Stop/interruption never publish a partial turn or its metadata.

Validation: 41 Rust tests pass (one native-vault test remains ignored), all 49 Flutter tests pass, analysis and all-target Clippy are clean, and the alternative Tauri host compiles. New checks cover UTF-8 byte/turn bounds, known/unknown history totals, final usage-only chunks, null/invalid/partial/zero counters, duplicate snapshots, option rejection versus generic errors, migration from version 3 without history loss, restart/export/cascade/transaction rollback, saved request model, local-only previews and action exclusion while viewing an older page. Existing streaming/history and uninterrupted rich-composer input checks remain green.

Nineteen actual Windows release/controller/HTTP/FFI/SQLite checks pass at each of 1120×780 and 620×700 logical sizes. The native usage actions retain each reply's original model after switching; omitted usage still saves a complete reply. The final context/usage dialogs, unavailable footer, composer and light/dark conversation captures were inspected. Dialog content is capped at 440 logical pixels and scrolls as needed. Raw fixtures/captures stay in ignored `output/usage-context`; the normal release is rebuilt after diagnostics and the original preview data is preserved. These checks use a local fixture, not a real vendor or OS pointer/IME interaction. Cross-platform, accessibility, low-end performance and model-specific limits remain open.

Normal release observation: one visible, unminimized window using the isolated usage fixture database, two short messages displayed with an empty draft and launch-only connection guidance, eight seconds settling and five samples: **220.84 MiB working set / 232.70 MiB private bytes**, with no CPU-time increase at the available timer resolution across four idle intervals. This is a single development-machine observation, not paired with earlier runs or a low-end/peak/active-interaction acceptance result. The 150 MiB working-set target remains unmet. Raw machine details stay in ignored `output/usage-context/runtime.json`; the original preview was reopened with its existing data.

## Brick 2.3.1 — Markdown and code presentation

Flutter assistant replies now render native GitHub-flavored Markdown: selectable headings, lists, emphasis, quotes, tables and inline code. Code blocks have a language label, exact code Copy and their own horizontal scrollbar. Whole-message Copy keeps the original Markdown; user prompts stay literal. The shared theme supplies all colors and locally installed monospace fonts, without an added font asset or syntax highlighter. The renderer is pinned to `flutter_markdown_plus` 1.0.12 and `markdown` 7.3.1.

The input box now renders headings and fenced code as actual native editable blocks. Heading markers and fences are hidden in rich mode. Code cards have syntax colors and a language selector using selectively registered grammars from pinned `highlight` 0.7.0. The Source and Add text controls are removed. Down Arrow on the last visual line moves into the following block or creates a paragraph below the final card, closing incomplete fences as needed and preserving CRLF separators. Wrapped-line navigation, Shift+arrow selection and active composition stay native. Ordinary paragraphs retain literal inline Markdown. Enter in code inserts a newline; Ctrl/Command+Enter sends. Active IME composition remains native and suppresses folding/sending. Canonical Markdown retains its markers, whitespace and line endings when sending; edits replace the relevant body ranges. Bounded document undo includes keyboard-created exits. This is a scoped heading/code editor, with local native drag selection and composer-wide Select All, rather than a complete inline rich-text editor. No links/images load or idle timer is added. Layout falls back to full source above 32,768 UTF-16 code units, 800 newlines or 64 blocks; code coloring falls back to plain code above 8,192 code units, 200 newlines or 2,048 token nodes.

Forty-four Flutter tests pass and analysis is clean. Eight reply checks cover light/dark compact layout, code whitespace/Unicode copying, inert visible HTML, all image source types, explicit destination inspection/copy and rejection of other link schemes, partial fences, coalesced updates, final flush/disposal, completed-widget reuse, complete plain-text fallback, original whole-message Copy and following streamed layout while preserving a reader's scroll position. Twenty-eight composer checks cover native editable headings/code cards in both compact themes, syntax colors, exact pasted/sent Markdown, body edits, language changes, document undo/redo, boundary arrows and Down exits into existing/new prose, wrapped-line and Shift+arrow preservation, Enter/send distinction, native composition, CRLF/nested/tilde/partial fences, unknown/large-code fallback and continuing prose after complete/incomplete code. Three regressions reproduced empty-block deletion and focused-field-only Select All before the fix. Nine added tests now cover structural removal/undo, Ctrl/Command+A from heading and code, full Markdown copy/cut, whole-draft deletion and replacement, identical-body paste and typing, IME replacement, and returning to local copying/editing. Seven further regressions cover uninterrupted text input through heading/code creation and removal, removal of a later code card, visible native paragraph newlines, Shift+Enter at paragraph boundaries and over a selection, heading-to-prose transitions and repeated blank lines, and code Enter/Shift+Enter followed by Down and continued typing. Three of these first reproduced the lost input connection and stripped newline before the fix. The earlier tests refocused fields between edits and missed this regression; these updates connect once and continue typing without refocusing. The existing history and connection checks also pass.

Fifteen real Windows release/controller/HTTP/FFI/SQLite checks pass at each of 1120×780 and 620×700 logical sizes. They include discovery/model/stop/denial/interruption checks, rich streaming and source-preserving reload, actual native code/table widgets, and native editable headings/code cards sending/storing their exact canonical source. Native dark compact and light wide composer captures show the large heading, hidden fences, language selector and colored code; the final captures without Source/Add text controls and with headings/code selected together were inspected. The native Select All action also verifies both visible selection ranges without changing canonical Markdown; it does not simulate OS keyboard input. The welcome view now scrolls when a taller composer reduces available space. Raw captures and diagnostic data stay in ignored `output/rich-composer/focus`; no machine-identifying report is committed. The normal release is rebuilt after diagnostics.

Normal Windows release observation before the input-box refinement: one visible 1120×780 window (visibility checked through the Windows API), two fixture conversations with the single rich turn displayed, eight seconds settling and five samples: **220.74 MiB working set / 233.11 MiB private bytes**, with no CPU-time increase across the four idle intervals at the available timer resolution. This is not paired with earlier measurements, a peak/streaming frame-time measurement or representative low-end acceptance. The provisional 150 MiB target remains unmet. The raw machine/timestamp details are retained only in ignored local output; the original preview was reopened with its existing data.

After the rich composer change, a normal visible release using the isolated rich smoke database and an empty draft was observed after eight seconds settling across five samples: **232.48 MiB working set / 242.69 MiB private bytes / mean 0.82% of one core**. This checks the normal release at rest, not active rich typing, syntax-coloring peaks or low-end performance, and is not a paired before/after comparison. The 150 MiB target remains unmet. Raw details stay in ignored `output/rich-composer`; the normal preview is reopened with its original data.

Active reply rendering coalesces at 80 ms, with no idle timer. Completed visible replies reuse their parsed widget. Above 32,768 UTF-16 code units or 800 newline characters, rich layout yields to labeled selectable plain text without truncating the source. This bounds rich layout work, not total plain-text shaping/memory cost. Native clipboard interaction, OS input/theme/accessibility, macOS/Linux fonts and rendering, and representative low-end performance remain unverified. Next is provider-reported usage and visible context limits; no token/cost accuracy is claimed by this rendering brick.

## Brick 2.2 — history browsing, export and shared UI contract

Implemented bounded keyset browsing beyond the old 100-session/40-turn view: one 50-session sidebar page and one 80-message transcript page, explicit Older/Newer/Latest controls, and complete Markdown/JSON export through the native Save dialog integration. Draft/page/scroll state restores for the 20 most recently visited views during the current launch. Export streams a consistent full-conversation snapshot into a new file and never replaces an existing file. Provider context remains bounded independently of browsing. `UI.md` now states the visual contract, with the shared palette, dimensions and Material theme in Flutter's `theme.dart`.

Validation: 32 Rust workspace tests and eight Flutter tests pass; analysis and workspace all-target Clippy are clean. New storage tests cover 137 tied-timestamp sessions, both pagination directions, deleted boundaries, invalid page sizes, 123-turn history after reopening, complete Unicode/Markdown/JSON export, and writer failures. Bridge export checks cover existing destinations and errors after writing begins without partial publication. Controller/widget checks cover bounded page replacement, draft/view restoration, failures, canceled export, generation exclusion, export menu, compact layout and theme reuse.

Thirteen actual release renderer/controller/FFI/SQLite checks pass at each of the wide/compact sizes. An isolated legacy database reaches all 137 conversations and all 246 saved messages, exports both endpoints of the full history, preserves view state, rejects replacement, and sends from an earlier page using the latest bounded provider context. Nine existing streaming/model-selection regression checks also pass. Both themes and the compact history controls were visually inspected. The normal release was rebuilt and the native file-selector library is bundled. The Windows build helper's generated-only junction fallback works without system/SDK changes.

Normal release observation: one visible window on the Windows development machine, 50 sidebar entries/80 short messages loaded from the isolated 137-chat database, eight seconds settling and five samples: **231.48 MiB working set / 240.61 MiB private bytes, 0% of one core across the four idle intervals**. This single observation is not a paired comparison, peak/export measurement or low-end acceptance; the provisional 150 MiB target remains unmet. No idle paging/export timer was added.

Evidence: private local benchmark report (excluded from Git). Native Save dialog pointer/keyboard interaction, OS theme/IME/accessibility, macOS/Linux sandbox export and low-end hardware remain unverified. macOS user-selected file access and outbound-network entitlements are configured but not validated here. Brick 2.3 starts next with Markdown/code rendering.

## Brick 2.1.1 — retained model discovery and selection

The Flutter connection dialog fetches a searchable model list and enables a chosen subset, with manual IDs as a fallback. The composer picker changes the active model without discarding the conversation/draft or re-entering a key. Choices and the active model persist; remembered credentials continue to restore after restart.

Validation: 29 Rust workspace tests and five Flutter widget tests pass. HTTP tests cover authenticated sorted/deduplicated discovery, denial, unsupported/malformed/empty lists and redirects; connection tests cover enabled-only switching, unchanged vault identity, restart restoration and launch-only key preservation. Listing/switching are rejected during generation. Nine real HTTP/FFI/SQLite controller checks pass at both wide and compact sizes, including a fixture that rejects requests using the wrong selected model. Four fresh processes verify model/choice restoration, authenticated discovery with the saved key, forgetting and history retention. The generated test key is deleted and absent from data files. Analysis and all-target Clippy are clean.

Evidence: private local benchmark report (excluded from Git). Models returned by a provider are not guaranteed to support text chat. Real vendor endpoints, OS input/accessibility, macOS/Linux and low-end hardware remain unverified. Earlier resource measurements below are historical; this refinement adds no idle discovery timer.

## Brick 2.1 — retained credential checks

Implemented a replaceable OS credentials port/plugin, restart restoration, Remember connection, recovery warnings/Retry, and Forget without conversation loss. SQLite schema 2 stores nonsecret metadata and opaque references, with additive migration from schema 1. API keys remain in the OS vault or process memory for a launch-only connection; no plaintext fallback exists. Endpoint binding and per-directory vault names prevent restoring a key to another configured endpoint/workspace.

| Check | Result and boundary |
| --- | --- |
| Rust workspace | 26 tests passed; native-vault test separately opt-in; all-target Clippy passed with warnings denied |
| Native Windows credential store | Generated entry written, reopened/read, deleted and absence confirmed; no unrelated entries accessed |
| Four separate Flutter processes | Save → restore and authorized real fixture stream → forget while retaining history → restart confirms connection absent and history retained |
| Secret handling | Bootstrap omits keys; generated key absent from DB/data files; no real user key used in validation |
| Recovery/endpoint isolation | Locked/missing vault, keyless restoration without a vault, launch-only mode, failed secure save preserving old settings and changed-endpoint rejection passed against an injected test vault |
| SQLite migration/rollback | Legacy DB/history retained; metadata and preferences commit atomically, including a forced transaction failure |
| Flutter | Three widget tests pass; analysis clean; recovery, hidden/omitted key and Forget behavior covered |
| Streaming/rendering | Six real HTTP/FFI/SQLite checks pass at each of 1120×780 and 620×700 logical pixels; updated settings/light/dark captures inspected |
| Default build | Root Flutter build helper successfully builds the normal Windows release with bundled Rust library |
| Normal release observation | One visible window, two complete fixture turns and reconnect guidance: 216.56 MiB working set / 228.91 MiB private bytes, five settled samples averaging 0.30% of one logical core; this is not a paired comparison or low-end acceptance |

Evidence: private local benchmark report (excluded from Git). Reproduce with the Flutter app's [instructions](../apps/dolores_flutter/README.md). The OS vault and SQLite have separate transactions: interrupted/failed cleanup may leave an unreferenced secure entry; failures are reported and never trigger plaintext storage. Locked-vault tests use an injected deterministic implementation; Windows lock/unlock UI, macOS Keychain, Linux Secret Service, real hosted providers and CI are not verified locally. History remains unencrypted. This retained record predates brick 2.2; see the current history acceptance above.

## Brick 1.2 — retained Flutter trial

A real Flutter 3.47.5 Windows release now shares the same Rust core/provider/store through a bundled C ABI. Two Rust bridge tests, two Flutter widget tests and six real HTTP/FFI/SQLite controller checks passed. Static analysis reports no issues. Light, dark, settings and narrow renderer captures were inspected at 1120×780 and 620×700 logical pixels.

| Normal-release metric, new paired observation | Iced | Tauri | Flutter |
| --- | --- | --- | --- |
| Process-tree working set, mean | 29.60 MiB | 427.47 MiB | 224.36 MiB |
| Process-tree private bytes, mean | 13.81 MiB | 197.71 MiB | 229.52 MiB |
| Runtime bundle, uncompressed | 8.64 MiB | 12.41 MiB | 31.13 MiB |

The same two-turn fixture was copied into isolated directories for each normal release. Windows were visible and unminimized, with verified 1120×780 logical client areas at 144 DPI. Five samples after eight seconds of settling, one launch each, on the Windows development machine. Flutter's working set is lower than Tauri's, but private allocation is higher. GPU/system-wide memory is not included. The Flutter bundle includes its engine and Rust DLLs, AOT library and assets; its tiny launcher is not the bundle size. All entries exclude already-installed system runtimes.

Evidence: private local benchmark report (excluded from Git), [historical resource comparison](research/flutter-shell-trial.md), [current build/run instructions](../apps/dolores_flutter/README.md). The comparison runner was retired after Flutter was selected; this is historical evidence, not a current reproduction command. Low-end hardware, real model/OS input/IME, accessibility and macOS/Linux acceptance remain open; platform CI is deferred.

## Brick 1.1 — retained comparison

Both normal release builds used the same SQLite fixture data with two complete turns, a visible unminimized 1120×780 logical window at 144 DPI (150%), no browser debugging and no smoke feature. A Windows development machine. Five process-tree samples per launch after eight seconds of settling; this is not low-end hardware. The Tauri executable retains the verified brick 1 implementation; Iced reuses the same core/provider/store crates.

| Metric | Iced + tiny-skia | Tauri + WebView2 |
| --- | --- | --- |
| Process-tree working set, mean | 29.47 MiB | 430.60 MiB |
| Process-tree private bytes, mean | 13.68 MiB | 202.58 MiB |
| Processes | 1 | 7 |
| Windows executable | 9,060,352 bytes / 8.64 MiB | 13,014,528 bytes / 12.41 MiB |
| Sampled idle CPU, one logical core | 0.00% | 0.30% |

Working set is 93.16% lower in this paired observation. Shared pages can be counted more than once; private bytes represent committed allocation, not resident physical-memory savings. Two additional native launches averaged 29.57 and 29.48 MiB working set, with 13.78 and 13.69 MiB private bytes. Their short idle intervals also showed no CPU-time increase at the available timer resolution. Do not turn a rounded zero into a claim of zero work under all conditions.

Earlier launches showed roughly one core of CPU consumption, including a native 1040×760 run. That did not recur in subsequent native launches, and the renderer trace showed five startup redraws rather than a continuous redraw loop. The cause was not established; repeat startup, foreground/background and sustained interaction checks before accepting performance. No driver, framework or app fix is claimed from that observation. The CPU script also now uses floating-point arithmetic to avoid rounding sub-second deltas to integers.

Raw machine-specific evidence is retained locally and excluded from Git. This retired-shell measurement used helpers that have since been removed. Local screenshots and detailed traces remain ignored historical evidence; use the current Python helpers for the maintained Flutter release.

| Native check | Result and limit |
| --- | --- |
| Windows release | Built with software renderer and no wgpu/webview dependency |
| Shared Rust workspace | 18 tests passed; Clippy all targets including smoke feature passed with warnings denied |
| Real loopback HTTP fixture | Configuration, Unicode streaming and two complete persisted turns passed |
| Stop, HTTP 401, truncated stream | Draft restored, failed/partial turns absent, raw provider body hidden |
| Stored history reload | Four messages reload through the controller; SQLite restart/deletion behavior also covered by existing store tests |
| Light/dark/settings/narrow screenshots | Real renderer captures inspected, including CJK text and wrapping |
| Run bounds | One reserved run; bounded event queues; newest 80 messages retained by UI/store; core context/output limits unchanged |
| Input and accessibility | Ctrl/Command+Enter, Copy, Back/Escape implemented; actual pointer/keyboard/IME UAT and screen-reader acceptance remain open |
| Native startup | Cold/warm readiness timing not measured; old webview readiness timing below is not transferable |

The Iced shell deliberately uses a settings page and Ctrl/Command+Enter. Iced has no current screen-reader integration; arbitrary transcript selection is also absent (whole-message Copy is available). Tauri remains runnable with `pnpm desktop:web`; Iced with `pnpm desktop:iced`. The selected Flutter shell uses `pnpm desktop`.

## Brick 1 — retained webview evidence

### Verified

| Check | Result |
| --- | --- |
| Frozen frontend dependency installation | Passed |
| Svelte/TypeScript + accessibility diagnostics | 0 errors, 0 warnings |
| Production frontend build | Passed; JS 53.54 kB (20.70 kB gzip), CSS 10.30 kB (2.79 kB gzip) |
| Rust workspace tests | 14 passed across core, provider, storage and host |
| Rust Clippy, all targets, warnings denied | Passed |
| Rust and frontend formatting | Passed |
| Windows release executable | Built successfully; 13,014,528 bytes (12.41 MiB), no installer |
| Native desktop against real local HTTP fixture | Connection configuration → Unicode streaming → complete turn persistence passed |
| Stop during preparation and active response | Passed; draft restored and cancelled turn absent after reload |
| Process restart | Completed history survives; connection/credential must be configured again |
| HTTP 401 and truncated stream | User-facing errors; raw fixture error body not exposed; failed turns absent after reload |
| Session delete and remaining history | Passed in native UI; cascade also covered by storage test |
| System light/dark + narrow layout | Screenshots inspected; native narrow conversation selector checked |
| Console errors | None in final native checks and production browser preview |

The fixture is `scripts/mock-provider.mjs`, bound to loopback only. It is not a language model. Native validation uses `output/native-smoke-data`, separate from default user data. Screenshots and raw normal-run metrics are in ignored `output/playwright`.

### Original measured baseline

A Windows development machine; release executable with two short stored turns. This is **not** the proposed 2-core/4-GiB reference machine.

| Metric | Observation | Boundary |
| --- | --- | --- |
| No-debug idle process-tree working set | Mean 421.80 MiB; range 421.67–421.93; 7 processes, 5 samples | Above provisional 150 MiB target. Summed working sets may count shared pages multiple times. |
| No-debug process-tree private bytes | Mean 189.34 MiB; range 189.27–189.47 | A separate allocation measure, not equivalent to working set or physical system-memory delta. |
| With CDP testing attached | Mean 445.26 MiB working set; about 210 MiB private bytes | Test instrumentation increases the observation; it does not explain the whole gap. |
| Warm process start → UI ready | 1.3914 seconds in one instrumented run | Process start timestamp to `dolores-interactive` mark (initial history loaded and rendered). Cold startup target remains unverified. |

This historical normal-run working set included the native host and all its descendants after removing the browser debugging argument. Its shell measurement helper has since been retired. It excluded the separate fixture server, browser preview and build tools. Raw evidence remains ignored; warm startup was measured before any page reload, and absolute activity timestamps remain private. It is not a current Flutter baseline.

No optimization benefit is claimed from WebView2's inactive-memory control. Its documented Low mode is best effort and can affect responsiveness; see [research](research/architecture-options.md). Do not weaken sandboxing or silently lower acceptance thresholds to label this target achieved.

## Remaining acceptance

- Complete native pointer/keyboard/copy-paste/IME/system-theme UAT, investigate any recurring CPU spike, and resolve or explicitly accept accessibility/selection gaps before release.
- Repeat release memory, startup, scrolling and cancellation checks on a 2-core/4-GiB reference device or representative equivalent.
- Expand real-model acceptance beyond the specific authorized Qwen probes recorded above; provider/model reliability is not universal.
- macOS/Linux builds and platform UI behavior remain unverified in this Windows workspace. Platform CI brick 8.4 is skipped; manual platform qualification is planned in 13.4 when suitable hosts exist, without reinstating CI.
- Local unsigned Windows packaging, versioned dependency notices and preflight recovery are verified in bricks 8.1/8.2. Public distribution, signing and actual clean-machine prerequisite/version recovery remain open. Reusable skills and bounded automatic preference updates are implemented above; broader learning and model/task reliability remain unaccepted.

## Brick 10.4 — durable drafts and linked recovery

Verified 49 core, 54 bridge, 38 storage and 165 Flutter tests; clean Flutter analysis and Rust Clippy. Native restart fixture terminated after an applied edit: the file and receipt survived, the resumed run only read it, fresh model/output settings were used, and a stale source was refused. Draft tests cover restart, isolation, invalid saves preserving previous content and conditional clearing preserving newer edits. The native budget fixture also passed on the final bundle.

A bounded Qwen3.5-2B probe resumed a synthetic interrupted checkpoint against the unchanged understand-project corpus. It returned without tool use and failed all required grounding facts; files remained unchanged. This is a model/prompt reliability gap, not a passed task or evidence of real-model interruption recovery.

The normal release app was built and visibly launched (PID 24476). Original 22 user-data digests remained unchanged; schema 22 contains 26 tables. Draft persistence applies to saved chats with a 250 ms debounce and a switch flush; a crash before persistence can lose the newest edits. Latest 20 runs, eight linked ancestors and bounded receipts are explicit recovery limits. No automatic replay, detached execution, exactly-once external effects or cross-platform qualification is claimed.

## Brick 10.5 — forks and managed context

Verified 50 core, 55 bridge, 39 storage, 34 filesystem and 165 Flutter tests; clean Rust Clippy and Flutter analysis. Fork checks cover complete boundaries, wrong source, atomic duplicate failure, shared folder, parent deletion and absence of copied drafts/runs/thread authority. Nested guidance tests exclude the outside parent and refuse oversized aggregate snapshots. Relevant-skill tests preserve a visible catalogue while excluding unrelated full instructions and allowing explicit selection.

`scripts/test-managed-threads.py` passed on the normal native library: read-only preview made no request, one compaction permitted the send, reported summary usage was recorded in run evidence, and an oversized next summary preserved the previous summary and draft. The restart-after-edit fixture also passed. Compaction tests retain literal goal, full history and a reduced model-call allowance. A valid but insufficient batch stops for manual recovery; automatic mid-tool compaction is excluded.

A bounded live Qwen3.5-2B probe with synthetic saved history and a 4096-token configured window completed in 8.51 seconds and preserved the literal goal. Summary usage: 2403 input / 266 output / 2669 total tokens, separately reported from foreground usage. This establishes this compaction probe, not general coding competence or semantic completeness of every summary.

Normal release build launched visibly (PID 30968). Original 22 user-data digests remained unchanged; schema 23 contains 27 tables. Guidance discovery has explicit depth/directory/file/byte coverage; it does not read parent folders outside the selected root. Forks share files rather than isolating them. Physical keyboard/IME, other platforms and low-end qualification remain open.

## Brick 14.2 — scoped Windows desktop input

Delivered a separate, explicit 15-minute chat/window grant and a closed input
broker (click, double-click, type, scroll, key, drag). Observation remains separate.
Covered ordinary input is optional; every click/drag/submission/deletion needs
review even under Full access. Fresh capture UUIDs, identity/geometry/DPI/focus,
client-area/occlusion checks and a readiness/commit handshake guard dispatch.
Input consumes its observation. Revocation cancels the run; restart needs consent
again. Screen sharing uses the independently selected image-capable model.

`scripts/test-desktop-input.py basic` passed on the normal release bundle: the
actual native broker typed and performed a reviewed client-coordinate Save click, fresh images reached the local
provider, Full access did not bypass click review, a move during approval refused
dispatch, and revocation while queued left the Save count unchanged. `reopen` passed:
local evidence survived, access did not. Pure checks refuse stale/consumed/wrong
capture IDs, unsupported keys, oversized text and out-of-image coordinates.
Compact light/dark widget checks verified explicit consent, failed grants retaining
the goal/preview, unsupported vision and closed-target recovery. Existing 216
Flutter tests passed before the new third desktop test; focused desktop tests
subsequently passed. Final Rust core/provider/bridge libraries passed 178 tests; focused desktop checks passed. Clippy was clean.

A bounded configured DeepSeek V4.1 Flash run entered `Verified local note`, saved
it exactly once, and observed the saved screen: 14.59 seconds, six model calls,
five tool calls, three fresh observations, two explicitly reviewed inputs, about
203 KiB retained image evidence. Its task profile was explicitly 12 calls/12 tools
and 90 seconds, output 1024 tokens; ordinary defaults and selected model stayed
unchanged. Reported token usage was unavailable, not zero. Routine Qwen3.5-2B
did not reliably invoke the required observation and was rejected; this remains
a model reliability gap. The live probe also exposed identical-byte/different-ID
image lookup failure, now fixed by retaining distinct capture references.

The normal desktop was visibly relaunched and Computer use inspected using the
computer-use plugin. A stale verification-helper binding initially failed; a
kernel reset selected the current HWND and screenshots worked. Existing 37-table
provider/history digests remained unchanged. No private captures/transcripts or
credentials are committed. Uncertain-effect reconciliation belongs to 14.3;
low-end, multi-monitor/DPI transitions, arbitrary applications and macOS/Linux
qualification remain open. Input insertion is not task success or an OS sandbox.
## Brick 14.3 — interruption and reconciliation

Implemented immediate desktop-review pauses, durable uncertain-effect receipts,
post-stop local screenshot/target proof, explicit inspection, original-goal run
lineage and current-grant recovery. Stop/failure guidance directs users to
Computer use instead of suggesting an ordinary retry. Three near-identical
observations pause; current-session availability gates both capture and input.

Exercised `scripts/test-desktop-recovery.py` against the normal release with a
test-only wrapper around the actual native helper and the disposable local form:
input applied before lost receipt, immediate pause without a later provider call,
cold restart dropping access, stale screenshot and absent inspection refusals,
fresh explicit recovery without duplicate typing, unchanged-screen pause after
three observations, injected locked target, Stop after actual input before receipt
and the five-second broker deadline. Owned helpers exit and entered text remains.
The wrapper is a debug example and is never shipped. The locked-target fixture
is failure injection, not a passed physical lock/unlock qualification.

Rust core/provider/bridge/helper checks: 183 tests passed across seven suites;
Clippy and Flutter analysis clean. Full Flutter suite: 218 passed, including
420×480 light/dark recovery controls, no automatic inspection/grant, and preserved
unrelated draft. A normal release was rebuilt and visibly inspected in Computer
use, plus saved receipts and recovery controls in an isolated normal release
instance. The 37-table user-data comparison remains unchanged.

Observed Windows locking during verification exposed why a desktop-name check
alone is insufficient; current-session active/unlocked flags now fail closed.
Actual locked-session refusal and physical recovery still need qualification.
Slow redraw, arbitrary application effects, IME/screen reader, mixed-monitor DPI,
elevated targets and non-Windows hosts remain gaps. A later image receipt is
available evidence, not proof of exactly-once effects or semantic correctness.

## Brick 14.4 — bounded workflow qualification

Delivered a frozen three-case corpus, isolated live runner, trusted renderer for a
model-generated declarative dashboard, native budget-pressure fixture, owned
helper memory sampler and normal release startup probe. Full evidence and
reproduction instructions are in [computer-use qualification](qualification/computer-use.md).

The fixed DeepSeek generated-view inspection passed all visible facts with fresh
evidence and no input (4.07 seconds, two inspection model calls plus one separate
generation request). Qwen draft failed because it skipped the required observe;
the host refused before input. DeepSeek save-once retained the exact note but
proposed a click outside Save; the operator denied it and the host paused. These
failures remain failures. Foreground token usage was unavailable; image-file bytes
are recorded separately. No defaults, criteria or user configuration were relaxed.

Native tool exhaustion after input preserved text and an uncertain-effect
checkpoint; output truncation retained a partial answer without input. Helpers
were reaped. Receipt-loss, restart, explicit recovery and Stop/deadline checks
passed in 14.3. Rust 183 / Flutter 218 checks pass; analysis and Clippy clean.
The normal release's light/dark recovery views were visually inspected; compact
420×480 coverage is widget-based, not an additional native resize pass. User
provider/history digests across 37 tables remain unchanged.

One isolated normal launch exposed a window in 1,513 ms, with 227.2 MiB working
memory after settling. The native pressure fixture sampled a largest helper of
55.5 MiB and a Python-host/owned-helper tree of 89.7 MiB; this is not a Flutter
active-capture peak. Available hardware has 24 logical processors/about 32 GiB RAM.
Sampling, first usable frame, representative low-end hardware and sustained
desktop costs remain qualification gaps. macOS/Wayland authority differences were
reviewed against primary docs; no other-platform backend was enabled.

Milestone 14's planned implementation/reporting work is complete. **Its full
real-model acceptance gate is not passed.** Observation adherence, coordinate
grounding, arbitrary-app reliability, physical lock/unlock, mixed-monitor DPI,
IME/accessibility and other-platform/low-end execution require follow-up evidence.

## Brick 15.1 — complete settings audit and UX contract

Audited all 12 Settings categories and 17 views in the normal Windows release at
`c1e4f7e`, using an isolated synthetic project and example model IDs. Read the
configuration forms/handlers, prerequisites, scope/save/error paths and related
conversation surfaces. No provider request, credentials or access/sharing/learning
policy changes were made. Findings and delivery mapping are in the
[UX contract](design/ux-simplification.md).

Defined milestone 15 with seven subsequent runtime/qualification bricks, beginning
with conversation-first computer use, then navigation/preferences, model setup,
tools/access/connections, memory/skills, context/recovery and final flow qualification.
Numerical and authority defaults remain unchanged. Target usability metrics are
explicitly goals; this audit does not claim a runtime UX improvement or close
milestone 14's live-model gate. Corrected stale milestone 14 status in the roadmap.

Native audit covered wide dark screens. Compact/light behavior and populated,
unavailable/stale cases were reviewed in source/existing tests; fresh native
qualification remains scheduled per runtime brick. Physical assistive technology,
other platforms and a user study were not exercised. Documentation/link/diff checks
apply here; no new app build or live model request is required for this audit.
All 74 local documentation links passed. Audit writes used only the isolated
profile; the existing user-data digest baseline predates this task and no longer
matches conversation/run tables, so it does not establish a fresh whole-history
preservation pass for this audit. A fresh before/after baseline is required for
the upcoming runtime bricks.

### Conversation UX follow-up — prototype, not runtime delivery

Extended milestone 15's design review to Chat actions, Session summary/Context,
trajectory/run evidence, Changes, instructions, branch/export, feedback and task
recovery. Source findings and retained-control mapping are recorded in the UX
contract; production changes belong to 15.7 after the prototype is reviewed.

Three structurally different layouts are captured on
`codex/prototype-everyday-ux`: temporary details drawer, docked workspace panel and
conversation-local actions/details. Browser review exercised all three desktop
layouts and compact light Settings at 420×480. Simulated edge flows retained
summary edits after a failed Save and succeeded on an explicit second Save;
Continue refused an occupied draft, retained it and worked after clearing; unknown
context showed unavailable rather than zero and offered Retry. Full secondary
forms, modal focus/accessibility and native Flutter behavior are not qualified by
this sketch. Actions are local stubs; no provider requests, user-data changes or
desktop input/capture occur. No real-model or performance pass is implied.

The prototype is outside the release source on a separate branch and is visibly
open in the browser for review. The current desktop UI is unchanged, so no normal
desktop rebuild/relaunch is applicable. This preserves the requested prototype
before implementation boundary. The recommendation is provisional; there is no
user-selected layout or demonstrated usability gain yet.

The subsequent user review selected **A**, with existing icons, icon-only header
actions with hover hints, retained theme previews and no colored left-edge status
cards. The refined branch embeds the exact Material glyph outlines already used
by Flutter; no font or production dependency was added. Browser inspection checked
the plain paused card, System/Light/Dark preview tiles and light theme switching.
At 420×480 the icon header remained usable, and Enter on Activity opened the
expected details. The simulated failed summary save still retained edits and
succeeded on explicit retry. These checks qualify the updated prototype only;
native hover/keyboard/accessibility and runtime acceptance remain pending.

### 15.2 — Conversation window access implementation; qualification pending

Working-chat tools now advertise a host-only window-access request. It saves a
pause before later queued tools dispatch; discovery grants no access or pixels.
The composer plus menu and saved pause reach a local window picker, with View
only / Control this window and one sharing confirmation. An unconfigured image
capability receives a bounded generated-image check (64 output tokens, 15 seconds);
success enables image transport without changing the selected chat model. This
checks transport and a simple color observation, not general visual competence.

The desktop segment records its parent and original goal, subtracts prior shared
model/tool attempts and elapsed work, and cannot replenish limits through a model
switch or larger settings. Image checks for a handoff use a separately journaled
child with a reserved call/time allowance; interrupted checks never grant input.
Handoffs do not consume or upload unrelated draft attachments. Failed starts
retain drafts and revoke any newly created control grant. Existing input effect
reviews, expiry, cancellation and reconciliation checks remain unchanged.

Checks so far: 200 core/bridge/store tests and 221 Flutter tests passed, plus the
new atomic draft/attachment preservation test after restart and an injected save
failure. The three picker tests cover closing without sharing, failed image support
at 420×480, and a stale handoff retaining an unrelated draft. Rust Clippy and
Flutter analysis passed. The normal Windows release built and launched with an
app window handle; a fresh 37-table user-data digest comparison was unchanged.

The deterministic full native handoff reached its saved access pause, then the
helper correctly refused window listing because Windows was locked. A screenshot
confirmed the lock screen, so no native input was attempted. Native visual
inspection, successful fixture handoff, first-use image check, bounded Qwen/
DeepSeek computer-use tests and ready-path measurements remain pending until
Windows is unlocked. This is implementation progress, not a delivered milestone
or a new model-reliability pass. Later 15.7 work also replaces the legacy desktop
reconciliation screen with unified conversation recovery.

## Milestone 15.3 — Settings structure (implementation; native review pending)

Six sections replace the twelve main destinations. Search and typed legacy links
retain the old editors, including model responses/overrides and desktop recovery.
General is the ordinary entry. Personalization defaults to All chats and exposes
style directly; switching a dirty scope requires a decision. Connection, response,
web and scoped settings register live drafts, with Save/Discard/Keep editing on
Close. Remaining authoring editors receive the same protocol in 15.5/15.6.

Focused checks passed: 21 navigation/style/web/connection tests followed by 20
affected settings/model/attachment/context tests. The added 420×480 case confirms
six sections and a stale Close→Save failure retaining the actual switch value;
search navigation retains endpoint drafts and allows cancellation/discard. Theme
failure, pending dismissal, override reset and stale model-response recovery remain
covered. The System tile now uses one continuously laid-out miniature scene.

The normal release built and launched with a native window. Windows remains locked
and its screenshot is the lock screen, so light/dark native visual acceptance and
physical keyboard/assistive technology checks remain pending. No native app input
was dispatched. Model reliability and the milestone 14.4 gates are unchanged.

## Milestone 15.4 — Model setup (implementation; native review pending)

Ordinary Models presents an enabled-model library, guided Connect/discovery/manual
fallback and a single details editor for context, image support and expandable
responses. Explicit image-error recovery opens those details for the current model.
Advanced/search retains response defaults and scoped generation overrides.

The SQLite transaction compares the loaded model snapshot and atomically writes
context, image flags and response settings. An injected failure in its final write
rolled back all three; stale or disabled-model writes were refused. The result
survived restart and blank/reset context restored inheritance. Selected model and
other models' settings were preserved. Clippy passed. 23 affected Flutter tests
and 18 model/settings/response tests passed, including 420×480 light/dark, stale
save/refresh/Close, and a refresh failure after an acknowledged save.

A native C ABI fixture and a bounded configured-provider Qwen/Qwen3.5-2B live check
both discovered models, saved details, completed the READY chat (512 output-token
allowance, 30-second request), and restored the isolated profile. No credentials
or transcripts were saved as public artifacts; the user's 37-table digest remained
unchanged. These establish the bounded setup/chat path, not general task reliability.
The normal desktop release built; native visual/image-readiness qualification
remains pending while Windows is locked. Milestone 14.4 live failures remain open.


### 15.5 Tools, access and guided connections — implementation

Tools opens a readiness overview for built-in search/browser/computer use. Default
search has no key/setup step; optional connections and technical details are closed.
Browser setup appears only under Setup when unavailable. No package is installed
silently. Chat access is visible beside the composer plus button; Review/Custom/
Full access comes from saved policy, expired grants display Review, unavailable
policy displays Access rather than inventing authority. Custom grants, expiry,
containment and concrete consent remain reachable. Authoring participates in
Save/Discard/Keep editing; conflicts preserve values for explicit review.

Connections starts with a list and Add connection / Import JSON. Import validates
one to four local MCP configurations, preserves literal arguments and masks imported
credentials; remote/unknown/malformed/oversized configurations are rejected without
launch. A separate concrete launch review names the executable, arguments and
credential bindings before inspection. Only subsequently selected tools can be
enabled. Changing a reviewed launch invalidates its token; switching connections
protects unsaved edits. Saving uninspected fields cannot start a program implicitly.
Approval cards lead with effect, target and sharing; operation details retain
the full bounds. Existing host checks, defaults and approval rules are unchanged.

Exercised: compact light/dark MCP inspection, explicit tool selection and enable,
shared-slot refusal/retry, stale/failed saves, masked credential reuse, Stop and
review invalidation; malformed and oversized imports plus retained-text correction/
cancel. Web stale save and endpoint recovery remain covered. The acknowledged
response-default update preserves a selected model override in controller state.
Normal Windows release build/launch is required for this brick; detailed native
screen qualification is recorded in 15.8 rather than inferred from widget tests.

15.2 follow-up after Windows unlock: owned native window handoff fixture PASS
(original goal/lineage, remaining model/tool allowance, fresh image, unrelated
draft/attachment retained and not uploaded, zero submission). Qwen live handoff
hit the bounded response deadline; DeepSeek live handoff paused at outputLimit
before sharing any window. These are failures, not model-reliability passes.
Native General settings/theme previews visually inspected in the normal app;
System uses a single continuous scene. No user configuration was saved during it.

### 15.6 Memory and skills — implemented

Memory uses My preferences / Project facts, short sharing controls and expandable learning details. Remember this opens an editable preference from your own message; nothing is saved implicitly. Drafts survive failed saves and stale revisions; Refresh keeps edits and refreshes the expected revision. Closing an edited preference offers Save/Discard/Keep editing. Global skills work without a saved chat. Import and Create produce exact host-validated reviews before separate activation, with single-use tokens and stale-state checks. Trials and learning experiments are under Advanced; their policies and evaluation criteria are unchanged.

Exercised: 41 focused Flutter tests passed, including malformed/stale skill drafts retaining text and discard cancellation; Rust no-chat global import, malformed/oversized input, explicit activation and concurrent revision refusal passed, with Clippy clean. Normal Windows release built and launched visibly. Provider/history preservation check: all 37 tables unchanged. Bounded configured DeepSeek V4.1 Flash draft PASS (4096 tokens / 90 seconds, no activation). Qwen3.5-2B memory FAIL: ordinary reply saved, extraction used 61 output tokens but produced no verified preference; no retry or memory write. This remains a model acceptance gap, not an output-limit diagnosis.

### 15.7 Conversation details and recovery — implemented

Header keeps icon-only Changes / Activity / Chat actions with hints. The composer ring and Chat details open a right panel with Context, Activity and Changes. Context combines estimated tokens, usage details, summary corrections/source review and automatic compaction. Panels retain draft state across tabs; failed summary saves return to Context with the correction and error visible. Earlier tasks retains ordered expandable evidence and explicit checkpoint preparation. Project instructions remains a contextual action. Chat actions offers Branch chat / Export / Chat details; export format and attachment choices are deliberate. Comparison, capability/source inspection and unused-attachment cleanup moved to searchable Advanced settings. Cleanup retains a separate concrete confirmation.

Actual output/step pauses expose View progress and the relevant Adjust limits link alongside bounded continuation. Uncertain desktop effects still require inspection without replay. Access labels refresh after Settings changes; no access/budget/privacy defaults changed. Updated README/user guide and old-to-new control map reflect implemented navigation, including Windows input rather than its obsolete planned label.

Exercised: all 233 Flutter tests PASS; analyzer clean. Includes 420x480 light/dark panel paths, no eager context request, failed context plus unsaved correction across tabs/Close, failed attachment cleanup with draft retained, source/approval expansion, Stop and existing continuation/uncertainty tests. Normal Windows release built and launched visibly; 37 user tables unchanged. Native panel matrix and model qualification remain tracked in 15.8.

### 15.8 Everyday-flow qualification and Advanced refinement — implemented, acceptance gaps

Advanced now starts with a short chooser. Skill testing/learning and diagnostics/
storage are collapsed groups; technical policy, limits and source explanations
appear on demand inside the editors. Sharing consequences and experimental status
remain visible. Tools includes a direct Skills entry. The System theme preview
uses one continuous scene. No numerical, privacy or approval defaults changed.

Native verification exposed Escape failing on protected modal routes. A real-route
widget test reproduced the failure before the fix; Escape now takes the existing
Close path with pending-operation and Save/Discard/Keep editing protections.
Cancellation retains edited model text and saves nothing. An error inserted during
workflow creation also exposed a collapsing editor; its stable identity now keeps
the open form and values. All 238 Flutter tests PASS, analyzer clean, preview Python
scripts compile. Compact 420x480 light/dark coverage includes the new chooser.

The final normal Windows release built and was visually inspected: dark General/
Advanced/Harness extensions and light General/Advanced/Context/Activity/Changes.
Native Escape closed Settings and Chat details. Earlier wide review covered all
six sections, model details and populated memory. These are sampled native checks;
the full light/dark/compact nested-screen matrix remains open. The user's normal
provider/history profile is restored for the visible final launch, with preservation
checked against all 37 tables.

Frozen computer-use reruns remain FAIL: Qwen3.5-2B stopped at screenshot receipt
validation before input; DeepSeek V4.1 Flash typed the exact text and observed it,
but the proposed Save target was refused by the unchanged review. Zero saves,
no blind retry or repeated submission. These do not close milestone 14 reliability.
Bounded learning results remain separate: DeepSeek draft parsed without activation;
Qwen ordinary chat survived but extraction saved no verified preference.

The [qualification report](qualification/everyday-ux.md) records navigation counts,
reproduction and resource observations. Two settled final-release samples used
6.81–6.92% of one CPU core; the final endpoint was 322 MiB working set / 326 MiB
private bytes. Sustained idle cost, a matched performance baseline, exhaustive
native layouts, physical IME/accessibility, other OS and lower-end hosts remain
acceptance gaps. Optional browser installation still uses setup instructions.

## Brick 18.3 — Terminal supervision and recovery (2026-10-07)

Implemented bounded PTY queues and emulator scrollback, owned Windows process jobs, busy-close/final-quit review, and private stopped-tab display recovery. Selected output can be attached to a chosen chat draft. Native checkpoint tests cover invalid/oversized data retaining the prior record, restart with zero live PTYs, and selection sharing scoped to the recipient. Flutter terminal and workspace recovery tests pass (8); cold split restoration, corrupt recovery, canceled close and failed save preserve owners/output. Analyzer passes. Native flood, descendant-cleanup, normal packaging and the harder live model case remain for 18.6; other operating systems are unqualified.

## Brick 18.4 — Lazy language features (2026-10-07)

Implemented lazy TypeScript/JavaScript and Rust server discovery, bounded background JSON-RPC, request cancellation/deadlines and process-tree ownership. Editor actions offer current-version diagnostics, plain completion, hover, definition and references with keyboard alternatives. Unsaved buffers are synchronized as UTF-16 documents. Native protocol tests pass (2), including an actual Node stdio fixture receiving unsaved Unicode and rejecting malformed server output. Flutter language/editor/recovery tests pass (9); stale typed/project results are refused and buffers remain. Analyzer passes. These are protocol fixtures, not real language-server qualification; actual server cases, diagnostics timing and resource measurement remain in 18.6. Snippet/additional-edit completion is explicitly refused, and locations outside the project are not opened.

## Brick 18.5 — Managed language setup and reviewed edits (2026-10-07)

Implemented explicit pinned/hash-verified downloads into immutable private folders, cancellable progress, verified startup members and no package scripts. Added exact Before/After previews, scoped atomic multi-file buffer Apply and Undo language edit. Native language tests pass (5): stdio/malformed output, UTF-16 framing, canceled full install plus refused offline connection, checksum/archive failure, and a two-file rename with stale-disk/outside/resource-operation refusal before partial mutation. Flutter language/editor/recovery tests pass (10), including no download on setup open and cancellation leaving the dialog usable. Analyzer and strict Clippy pass. Real upstream downloads/server behavior and normal packaging remain qualification work in 18.6; managed Rust downloads are Windows x64 only.

## Brick 18.6 — Real terminal and language qualification (2026-10-07)

146 bridge library tests and 302 Flutter tests PASS; analysis and strict Clippy PASS.
Actual PowerShell project A/B/HOME, Unicode, resize, Ctrl+C, flood/backpressure,
six-process descendant cleanup and stopped recovery PASS. Real pinned TypeScript
diagnostics/completion/navigation/two-file rename/Undo and Rust hover/isolation PASS.
Qualification corrected PATHEXT, optional Rust debug archive membership and
TypeScript unversioned diagnostics/Windows URI matching, with regression coverage.
Unversioned diagnostics show a lag notice rather than a current-version guarantee.

Production Flutter programmatic two-PTY split/move/compact/light/dark/Stop corpus
PASS; saved renders inspected. Configured DeepSeek V4.1 Flash completed one saved
fixture read and one approved command (6.53 seconds, 633 output tokens), returning
the correct marker/42 while unsaved editor text remained private. Original 42-table
profile hashes unchanged. Normal main-entry Windows release built and launched
visibly with that same profile. See [qualification](qualification/terminal-language.md)
for measured incremental samples and reproduction scripts.

Sustained idle cost remains OPEN: absolute Flutter samples were high and variable
even where incremental terminal envelopes passed. Physical keyboard/IME,
accessibility, exhaustive native focus and other hosts remain unqualified.

## Brick 19.1 — Bounded Windows backend trial (2026-10-07)

One maintained Flutter 3.47.5 same-engine backend was tested in an isolated release
diagnostic. Initial disabled-owner initialization retained primary work. One repair
then created two visible native windows/two views under one NativeBridge/AppHost,
observed activation, destroyed the secondary and retained the primary draft/native
request path. Zero owned descendants remained; saved recovery renders inspected.
Added memory 66.83 MiB PASS (128 MiB ceiling); added idle CPU 1.222% of one core FAIL
(less-than-1% gate). Short samples are recorded, not sustained acceptance.

Disposition: backend HELD. No channel migration or second backend was attempted.
19.2 acknowledged transfers and 19.3 window-layout/rejoin are HELD, not implemented.
Physical input/IME, multi-window theme, transfer/monitor recovery and accessibility
remain unqualified. See [qualification](qualification/experimental-windows.md).

## Brick 19.4 — Explicit single-window fallback (2026-10-07)

The host now returns held window capability separately from saved Experimental
preferences. Settings explains the failed performance check and points to existing
split views. Default On remains a preference; Off remains Off across reopen.
Stale/failed saves retain the acknowledged choice. No window creation, automatic
trial retry, profile migration or source-group disposal is added. 19.2/19.3 stay held.

147 bridge library tests and 303 Flutter tests PASS; analysis and strict bridge
Clippy PASS. Native preference restart and stale-save tests leave zero terminal
owners. Three focused page/settings tests cover existing Home/navigation recovery,
held reason, failed-save recovery and Off after reopening.

Final release C ABI fallback corpus PASS in two fresh processes: held capability,
default On, saved Off restored, stale-write refusal and zero shell/child-host
activation. Normal Windows main-entry build and visible launch PASS. All 42 tables
remain; 41 hashes and the two pre-existing editor recovery rows match the earlier
baseline. One new terminal recovery row is preserved. Configuration/model/history
are unchanged; no recovery was erased to force a matching hash. Receipt:
`output/m18-m19-normal-handoff.json`.

## Terminal keyboard repair (2026-10-07)

Reproduced the exact Windows failure in a release, non-executing diagnostic using
the production terminal page: focused `a` key event yielded no input, while Enter
yielded `\r`. xterm 4.0.0 omitted the Flutter view ID from its text-input client;
Flutter 3.47.5 Windows rejects that client. A failing view-ID regression now catches
the attachment error. A pinned local xterm snapshot adds the containing view ID,
with its MIT license and only one upstream source-line change.

Native event injection after the repair received/rendered ordinary and shifted
letters, with Enter and Backspace unchanged. Nine terminal tests and all 305
Flutter tests PASS; analyzer PASS. Tests cover Home return, split focus/owner
isolation, composition committing once, Unicode, stopped-shell input refusal and
retained output followed by usable live-pane recovery. The fixture executed no
commands and made no model requests. See [keyboard qualification](qualification/terminal-keyboard.md)
for reproduction and before/after receipts. Physical IME, accessibility, exhaustive
native focus and other platforms remain open; programmatic Unicode tests are not
physical-IME acceptance. The normal Windows main release built and launched
visibly with the original profile; all 42 table hashes remained unchanged across
that launch. Only the recorded owned diagnostic processes were stopped.


## Milestone 21.0 — Memory audit and frozen corpus (2026-10-07)

Isolated SQLite baseline test passes: completed project decision remains in history but is ineligible for narrow explicit-preference capture; disabled policy prevents claiming and malformed extraction is separately refused. Current trigger/publication/source/scope and master-switch gaps are traced in [memory qualification](qualification/automatic-memory.md). Frozen public corpus and exact-source/scope/Forget/Off targets are recorded before runtime changes. This does not diagnose the private user profile or qualify future automatic memory.


## Milestone 21.1 — Memory switch and source index (2026-10-07)

Schema 34 adds transactional source indexing; fresh Memory is Off, existing saved choices are retained. Off stops both capture and recall while keeping inspectable entries. Source-linked confidence labels and setup-free empty states are exposed. 128 automatic records per scope are separate from the existing 12 manual preference allowance; bounded payload costs are in [qualification](qualification/automatic-memory.md). Focused native memory tests and 11 compact light/dark Flutter memory tests pass, including stale policy recovery. Maintained normal Windows release build passes; packaging required stopping only the owned preview after its open DLL blocked replacement. Capture expansion and cue-led recall remain the next bricks.


## Milestone 21.2 — Useful automatic capture (2026-10-07)

Facts, decisions, reported outcomes and open work accept exact standalone user evidence without a remember phrase. One invalid candidate refuses the entire batch. Background maintenance admits promptly, runs one request with a 16-chat coalescing queue, preserves completed replies and separate usage, and cancels on Off/Shutdown without startup replay. Frozen useful/negative/partial-quote tests and hung-provider queue/full/Off recovery pass. Legacy migration replay fixtures are preserved through idempotent indexing. [Qualification](qualification/automatic-memory.md) records bounds and remaining live-model checks.

21.2 verification: all 277 core/store/bridge native tests pass. Maintained normal Windows build passes. Packaged C ABI save/restore corpus passes in separate processes (30 local fixture requests, zero live requests), including scoped capture, correction/manual protection, malformed/invented/oversized output, provider denial, lower context/deadline, background stop and retained replies. Live-model reliability remains for 21.6.


## Milestone 21.3 — Cue-led recall and sources (2026-10-07)

Six frozen English/Chinese cues rank the expected fact/decision/outcome/open work/preference first. Recall inserts at most eight compact entries and three exact source excerpts within the remaining provider context allowance; very small memory allowance preserves the question unchanged. Cross-chat same-project recall, equal entities in separate projects, local scoped source opening and deleted-source exclusion pass native fixtures. Twelve compact Flutter memory/source/recovery tests pass. [Qualification](qualification/automatic-memory.md) separates this fixture coverage from upcoming live model checks.

21.3 packaged qualification passes: 21 local fixture requests, six automatic captures and six correct first-ranked cue matches, zero negative captures or scope leaks, and all six deleted sources excluded. The maintained normal Windows build passes. This remains synthetic evidence; live checks follow in 21.6.


## Milestone 21.4 — Correction and Forget (2026-10-07)

Current facts supersede bounded historical versions; reported completion resolves open work and manual corrections take priority. Forget removes derived records/index/history, stops pending maintenance and prevents replay from older retained source messages, while original conversations remain separately controlled. Five store audit cases, all 281 native tests and 12 Flutter memory tests pass. See [qualification](qualification/automatic-memory.md) for retention bounds and replay semantics.

21.4 maintained normal Windows release build PASS.


## Milestone 21.5 — Shared image memory (2026-10-07)

Selected image-capable models can index one explicitly shared image with source/asset IDs and an uncertainty label. Later relevant recall reopens one retained scoped asset within context bounds; missing/unsupported/malformed image handling preserves text memory. All 282 native tests and the normal Windows release build pass. Packaged save/restore uses six local fixture requests, zero live requests, and exercises actual image input, cross-chat recall, malformed caption with valid text capture, restart, missing source and Forget. [Qualification](qualification/automatic-memory.md) records costs and limits; live evaluation follows in 21.6.


## Milestone 21.6 — Batch reliability qualification (2026-10-07)

Milestone 21.0–21.6 is delivered with separate brick commits: one switch, useful
background capture, scoped source-backed recall, correction/history/Forget and
explicit shared-image memory. 286 native / 310 Flutter tests, analyzer, Clippy
and maintained normal Windows build PASS. Packaged capture/recall/image fixtures
PASS (30/19/9 local requests; zero live; restart zero). Six frozen cue hits, zero
negative captures/scope leaks and six deleted sources excluded. Recovery covers
Off/interruption/provider failure, stale publication/manual priority/Forget,
malformed captions preserving text/usage, mixed text/image capture and request-image
limits. Four compact/wide light/dark history/Forget UI cases PASS; renders are
diagnostic, not physical native interaction.

Final configured live: Qwen fact/recall PASS, DeepSeek scoped correction
PASS, image/recall PASS. Qwen fact capture is local; model-based
Qwen extraction previously missed and remains a gap. Caption metadata/global
conflict failures prompted parser/scope fixes; refusals count as misses.
[Qualification](qualification/automatic-memory.md) records earlier runs, costs,
ceilings and remaining limits. Original main-entry launch at schema 35
preserves all 42 original table hashes and adds three memory
tables only. Provider configuration/history/policy are preserved. Scheduling,
companionship and optional older-history Catch up are not delivered.

## Settings simplicity refinement (2026-10-07)

Memory and Experimental now show short names and brief supporting text. Circled
question-mark icons beside setting names expose longer explanations on hover only,
without moving controls or changing settings on click/hold. Memory's empty state,
Forget notice and recent activity are concise. Experimental shows plain availability
and actionable errors; internal performance and qualification messages stay in docs.
The explicit simplicity and settings-copy rules are recorded in [UI.md](UI.md).

All 318 Flutter tests and analyzer PASS, including eight compact/wide light/dark
hover cases. Existing recovery tests preserve failed-save choices, Off after reopen,
memory correction and Forget behavior. Saved diagnostic renders were inspected for
readable text, compact layout and wrapped hints; they do not establish physical
native hover or screen-reader behavior. Those interaction/accessibility checks
remain open. No provider requests or memory/runtime-policy changes were made.

Normal Windows main-entry release build and visible-window launch PASS. The original
profile retains all 45 table hashes, including provider settings, history and memory.
Private hash receipt: `output/settings-simplicity-normal-handoff.json`.

## UI guide cleanup (2026-10-07)

Reduced UI.md from 596 to 98 lines of shared design guidance, with on-demand
links to feature specifications and an explicit boundary against implementation
history. Removed obsolete and duplicated feature details. Documentation links
and whitespace checks PASS; no runtime change or desktop build was needed.

## Milestone 22.0 — Frozen scheduling contract (2026-10-07)

the development plan includes the next milestone batch. [Scheduling contract](design/chat-scheduling.md) freezes explicit intent, named-zone/DST/missed-run policy, bounded occurrence ownership, dedicated smaller budgets and the seven-case qualification corpus. This brick changes documentation only; scheduling is not yet available. Local links and whitespace were checked.

## Milestone 22.1 — Chat creation and durable receipt (2026-10-07)

Source-bound typed schedule_task resolves named-zone rules and enabled saved skills, pins model/project/budgets and returns a Scheduled receipt without a redundant approval. Schema 36 adds two scheduler tables. Five focused native tests pass: direct creation/duplicate calls, absent skill, quoted intent, cancellation/single-use plans, stale or failed writes/reopen, plus named-zone gap/overlap/weekdays and invented time refusal. Future tool effects remain reviewed. Clock dispatch/page/live-model acceptance follow in later bricks; normal build is reserved for the completed runtime batch.

## Milestone 22.2 — Durable clock claims (2026-10-07)

Seven focused native tests pass. Atomic occurrence claims advance next due and task revision together, refuse overlap/duplicate ticks and retain one missed skip after a long suspension. Restart interrupts unfinished claims without replay; backward clock changes cannot redispatch. Failed claim writes roll back next due. Named-zone DST gap/overlap and Friday-to-Monday boundaries pass. Actual app-open ticking/execution follows in 22.3.

## Milestone 22.3 — App-open execution owners (2026-10-07)

Scheduled occurrences use the existing host queue and chat execution, pinned endpoint/model/budgets/skill, reviewed tool effects, result conversations and Stop. App-open clock checks run only when schedules or active occurrences need them. Eight focused native scheduling tests and six targeted Flutter owner tests pass; analyzer is clean. Missing provider fails visibly and pauses recurrence with its result thread retained. Background execution preserves Home model/project/draft; owner-allocation failure is recorded once; paused tasks own no idle clock and failed refresh retains the list. Normal-packaged/live execution qualification remains for 22.6.

## Milestone 22.4 — Scheduled management page (2026-10-07)

Scheduled now lists/filters chat-created tasks and shows next run, actual run state, approval/progress, history and result/source links. A single actions menu offers Pause/Resume/Run now/Skip next/Stop/Delete; no creation form. Nine focused native scheduling tests and eight targeted Flutter tests pass, including stale-action recovery, background preservation, compact/light/dark and failed refresh. Initial transparent-background renders and a Material ancestor warning were corrected; final saved renders were inspected with view_image (diagnostic appearance evidence only). Delete cancels recurrence and retains results; unused deleted task metadata can age out at the 32-task retention boundary, without deleting result conversations.

## Milestone 22.5 — Conversational task changes (2026-10-07)

A source-bound manage_scheduled_task tool changes time/skill/model or pauses/resumes/skips/cancels an identified task. Pronouns require one source-chat candidate; otherwise an explicit task ID is needed. Task IDs are inspectable in Details. Twelve focused native scheduling/edit tests pass, including repeated time edits, immutable in-flight snapshots, ambiguous/other-chat/quoted/wrong-action refusal and pause/cancel. Creation deduplication is scoped to one exchange so a later deliberate identical request is possible. Full qualification follows in 22.6; unsupported phrasing remains a model/intent acceptance concern, not a claimed universal parser.

## Milestone 22.6 — Scheduling batch qualification (2026-10-07)

22.0–22.6 implementation and qualification tooling are delivered in separate brick
commits. All 288 core/store/bridge library tests and 326 Flutter tests PASS; analyzer,
strict Clippy and normal Windows main-entry build PASS. Packaged public corpus uses
11 local requests and zero live requests: creation/duplicate calls, due clock/report,
reviewed read, Stop while waiting, overlap refusal, offline recovery, Pause/Skip/
Run now, stale writes and disabled-skill conversational recovery PASS. Fresh-process
interrupted claims do not replay and completed results remain. Clock injection and
saved renders are diagnostic evidence, not physical 21:00 timer or native input
acceptance.

Bounded configured live: DeepSeek creation and actual pinned skill report PASS;
quoted examples create no tasks. Qwen direct creation/report MISS, and DeepSeek
time-change probes MISS on adapter argument refusal. Failed changes preserve the
task; no silent model fallback occurs. General natural-language scheduling/editing
reliability remains **unaccepted**. Four isolated runs use 13 public chat/run turns;
token totals are unavailable, not zero. Original configuration/history are never
copied or changed by live qualification. [Full evidence](qualification/chat-scheduling.md)
records prior misses, exact bounds and remaining gaps.

Qualification fixes cover idempotent migration, metadata-only creation catalogs
without raising the 14-tool limit, future skill guidance loaded only at execution,
single-version skill pinning, explicit model profiles, same-second history ordering
and active-owner retention after 60 skips. Clock queries read pending occurrences;
management omits full snapshots. At 32 tasks/1,600 maximum-size synthetic occurrences,
stored JSON is 22,791,996 bytes, list payload 238,232 bytes; ten-sample tick/list
maxima are 76.319/27.123 ms. Sustained idle/RSS, physical timing/sleep/wake,
accessibility, exhaustive rules/languages and other platforms remain unqualified.

Only the verified owned preview was stopped after its DLL blocked packaging; saved
runs were inactive and its database was backed up first. Normal main-entry launch
reports a visible window after restoration. All 45 original table hashes remain
unchanged at schema 36, with only two scheduling tables added; no original task was
created. Milestone 23 is next; closed-UI execution stays deferred to 24.

## Milestone 23.0 — Companion contract (2026-10-07)

the development plan includes continued implementation through the current roadmap, with
one milestone per batch and separate brick commits. [Companion contract](design/companionship.md)
freezes Off, chosen model/hours/cap, presence/quiet gates, random opportunities,
single unread message, source revalidation and bounded generation before runtime
changes. Defaults never activate companionship in the user's original profile.
Document/link checks pass; subjective welcome needs later user feedback.

## First-user audit — scheduling feedback (2026-10-08)

The [ranked feature checklist](qualification/first-user-audit.md) is an ongoing
normal-app audit, not acceptance of all features. An isolated profile and public
disposable project preserve original configuration/history. Qwen failed a clear
weekday 21:00 request after malformed tool calls; its unrelated command was denied.
DeepSeek saved the correct task automatically, then its confirmation stalled after
60 seconds. The completed task survived, but chat displayed an internal plan ID
and Scheduled needed manual Refresh to show it.

The scoped UI fix displays the saved schedule immediately and refreshes the task
listing on page entry, without waiting for the model's final reply. Regression
tests failed on the old behavior, then all 20 focused checks passed, including
malformed/failed receipts, default-time disclosure and failed-refresh recovery
without losing the draft or retained listing. Source/test analysis is clean;
saved compact/wide light/dark receipt renders were inspected. Normal Windows
main-entry build/launch passed. Native reopen showed the retained readable receipt
and its exact schedule/model/project details; Scheduled displayed the saved paused
task immediately. Page entry during a newly saved busy chat is covered by the
regression test; that exact live timing still needs retesting. Model scheduling
reliability remains unaccepted. Run now exposed a separate misleading “Temporary
workspace” review label for a pinned project; the audit stopped before approval
and tracks that defect separately.

## First-user audit — scheduled run identity (2026-10-08)

Run now creates its result session in the pinned project, but the start response
omitted workspace metadata and the UI retained a new controller's default folder
and current model. The bridge now returns the pinned workspace, and the controller
loads that workspace and model before presenting operation reviews. Home's chosen
project, model and draft remain separate. The AppHost regression failed on the old
project identity; the native corpus failed on its absent workspace field. After
the fix, 17 focused Flutter and five Rust scheduling tests passed, including a
temporary task's retained working folder; changed-source analysis is clean.
Normal Windows main-entry build/launch passed. The packaged C ABI corpus passed
its new identity checks and 11 local-provider requests, then a separate reopen
preserved completed results and interrupted claims without replay. Live DeepSeek
Run now displays the correct project in its running strip and every reviewed
file operation; report completion remains under test. All 49 original-profile
table hashes remain unchanged.

## First-user audit — scheduled execution handoff (2026-10-08)

The completed DeepSeek occurrence incorrectly said it could not create the schedule
and asked which scheduler to use, despite running an already-saved task. It also
called an unselected skill pinned. The host reused the original creation request
and unconditionally instructed execution with a pinned skill. Execution now
explicitly establishes completed setup, preserves the request as provenance, and
uses the actual nullable pin. Ordinary guidance applies when no skill is selected.

The real packaged host-to-provider regression failed on the old handoff. Five
focused Rust scheduling checks, formatting and the normal Windows build passed.
The rebuilt native save/reopen corpus passed with 14 local-provider requests,
including pinned and unpinned contexts, enabled-skill noninheritance, Stop/offline
recovery, stale revisions and restart without replay. These are deterministic
checks, not model reliability evidence.

Native Run now reran the same paused DeepSeek task. It completed a report from
actual project files, recognized the saved schedule and made no pinned-skill
claim. Scheduled progress/approval/history, Open result and per-call usage worked.
Four calls reported 5,131, 5,298, 5,422 and 5,856 total tokens; no total is inferred
by the UI. No file edits or commands were performed. The report was still longer
than desirable for a short report. Creation reliability, physical recurrence and
live pinned-skill execution remain separate gaps in the ongoing checklist.

## First-user audit — retained conversation catalog (2026-10-08)

Native navigation from a newly created Side chat back to a retained project chat
made the Side chat disappear from Recents. Each controller retained its own old
conversation catalog; the stored chat and its messages remained intact. Two
AppHost regressions failed before the fix, including unavailable-storage recovery.
Returning to a retained controller now refreshes only sessions, projects and
catalog paging flags. Drafts, model settings, context, partial responses and
pending reviews remain owned by their original conversation.

All 16 focused conversation/workspace/scheduling checks passed, including retry
after an unavailable catalog and retained busy-chat state. Changed-source analysis,
document/link checks, publication pattern checks and diff checks passed. Normal
Windows main-entry build/launch passed. Native retest created another Qwen Side
chat after loading the earlier project report. Returning to the cached report
kept both side chats visible, restored the exact public draft and retained its
DeepSeek selection despite the other chat's Qwen selection. The new bounded reply
used one call with 2,353 reported total tokens. All 49 original-profile table
hashes remain unchanged. Older/newer paging remains a separate pending audit case.

## First-user audit — file preview retention (2026-10-08)

Native double-click in Folders did not keep a file, contrary to the user guide.
Tree rows handled only single-click. The widget regression failed on the original
preview state. A separate host regression found that reopening a kept file removed
another preview and converted the kept tab back into a preview. The tree now has
double-click opening with an explicit kept-tab option. Reopening a tab already in
the group selects it without replacing or reclassifying another preview.

Failed opens leave the previous tabs intact; retry keeps only the requested file.
Accessible rows retain their button, focus and tap actions. Fourteen focused
editor/layout/host checks passed. Native normal-build retest kept notes beside the
README preview and retained both on reopening notes. The final normal build passed;
native rows expose named buttons and reopening notes retained both tabs. First-edit pinning, physical editing and
save shortcuts remain pending; helper input alone has not established an app defect.

## First-user audit — Git commit recovery and history (2026-10-08)

Native Source Control passed initial-commit file expansion, exact historical diff
and collapse. Empty-index commit review included tracked and untracked saved
changes; Cancel left the index untouched, and a fresh confirmation created the
exact commit. Existing staged selection took priority over other working changes.
A deliberately failing disposable hook retained the message/index and explained
Refresh; after correction, fresh review committed only the staged file. Section
stage/unstage passed with spaces, Chinese filenames and a detected rename, with
independent saved Git checks. Physical Ctrl+Enter remains unqualified: helper input
inserted a newline, while the deterministic shortcut test passes.

Successful commits did not refresh the displayed History. Two host regressions
failed before the fix. Applied actions now reload history only when the loaded
head differs from saved HEAD; unchanged-head staging retains paged history.
Existing exact-commit tabs and expanded files still on the first page remain.
An unavailable history read retains the completed commit outcome and readable
list; explicit retry reads history without repeating the commit. All 24 focused
Git widget/host checks and changed-source analysis passed. The normal Windows
build/launch passed. Native confirmation created a new rename/addition commit:
its history row appeared without Refresh, while the older expanded commit's files
and exact diff remained readable. Broader Git qualification remains on the checklist.

## First-user audit — reviewed file work and check repair (2026-10-08)

Native Qwen creation proposed a file without its requested final newline. The
review exposed the mismatch; Deny left it absent. Qwen wrongly explained denial
as directory restrictions. DeepSeek's fresh creation review saved the exact
29-byte LF text, and its receipt/final answer agreed. Only that new file changed.
A subsequent edit became stale after an external disposable-file change. Apply
refused without overwriting it; DeepSeek read the newer content, explained the
conflict, and after explicit chat confirmation proposed a fresh diff. Applying
that diff preserved the external change and both LF newlines. The separately
reviewed Python check completed with actual exit 0 and CHECK OK evidence.

After deliberately changing the fixture units, the same literal check failed
with exit 1. Repair and verify sent generic Continue, and DeepSeek obeyed the
original no-edit request instead of repairing. Widget and core prompt regressions
failed before the fix. Repair now conveys a new request to diagnose, propose
reviewed changes and rerun the same check, including after a read-only request.
Ordinary Continue remains unchanged. The bridge binds the repair input to a
latest command-review pause, rejects altered inputs and accepts legacy Continue.
Two core, one bridge and three Flutter checks passed, with clean source analysis.
Normal build and native repair retest passed in a clean conversation stating the
Celsius requirement up front. The same check failed with actual AssertionError,
then Repair and verify performed fresh reviewed reads, the single-line repair
and the unchanged literal check without further chat instructions. Expanded
evidence showed exit 0, CHECK OK and empty stderr, matching the final answer.
Independent comparison confirmed only notes changed, preserving its exact CRLF
bytes and all other lines; the check and other seven public files stayed intact.
The earlier conversation still failed after clarification: the model disowned
real reads, requested approval in prose, then promised an edit without submitting
one. That case remains open rather than being counted as a successful recovery.

DeepSeek also incorrectly called an earlier real successful check fabricated.
That historical-evidence/model-quality issue remains separate and unresolved;
a later failing check does not disprove an earlier successful invocation. Details
and exact live usage are in the first-user checklist. Original-profile checks
confirmed all 49 table hashes unchanged; no real project files/hooks were modified.

### First-user audit — reverse edit and feedback (2026-10-08)

On the same normal rebuilt app, Changes displayed the saved forecast diff and a
reviewed reverse diff retaining the externally changed city. Cancel preserved the
file; a fresh Revert once restored the exact earlier LF text, marked the old edit
Reverted and saved a separate record. The repaired CRLF notes and all other files
stayed intact. Task feedback saved and reopened both Worked and Needs work with
the same public note, then restored Worked for the verified repair. No new model
request was sent. Activity, Earlier tasks, expanded receipts and per-call usage
were inspected; settings origins and remaining checkpoint cases are still open
in the checklist. These existing flows required no runtime change.

## Companionship frequency refinement (2026-10-08)

The daily limit is now an integer Quiet–Chatty slider from 0 to 100, showing the
selected count with longer guidance behind hover help. Default 2 and existing
opt-in/model/hours remain unchanged. Pacing scales with the allowed window and
cap, retaining the former three-hour gap at the default; high settings no longer
remain bound to that gap. Attempts, including failures, count before generation.
Lowering the cap preserves consumed attempts rather than refilling the budget.

Five core, four storage and one bridge companionship checks passed (the optional
live bridge check remained ignored). Coverage includes zero silence, rejection
of 101, 100 spaced attempts and the cap, next-day reset, overnight hours, restart
with 100 consumed attempts, stale/failed saves, absence/busy/unread suppression
and no catch-up. Six Flutter checks passed, including real slider gestures at
both ends, failed-save retention/retry, refresh and narrow/wide light/dark layouts.
Saved renders were inspected; changed-source analysis and the normal Windows
build passed. In the isolated native profile, the previous 2 loaded correctly;
100 and 0 saved through the visible controls, and independent stored policy reads
agreed while model/hours/Off were retained. This cap is an upper bound, not a
promise of 100 daily notes. Native eligible real-model delivery remains pending
under E01 in the first-user checklist.

### First-user audit — aligned Git diffs and selected hunks (2026-10-08)

Historical and five-hunk working comparisons displayed distinct aligned old/new
contents; scrolling from either side retained shared vertical position through
the final hunk. A selected-hunk stage review included only line 10 of a disposable
200-line file. Apply changed only that hunk in the index, leaving all five edits
saved and a separate file untracked. The staged comparison and selected-hunk
unstage review then returned the index to empty while retaining those exact saved
edits and unrelated file. Native UI and independent Git/file checks agreed.
G05/G06 are checked in the audit; remaining Git interactions retain their own gaps.

Native chat export passed for Markdown and JSON with run details, saving fresh
ignored files through Save As and showing the correct four-message count.
Every exported JSON message ID/role/content matched storage exactly; Markdown
contained all complete messages. Actual earlier failed and later successful
command receipts, literal arguments, separate call counts and saved feedback
were retained. The selected chat/model stayed usable, with no new model call.
C13 is checked; attachment export remains separate and pending.

Native branching from the completed repair retained the exact four original
messages and shared working folder in a distinct fork, leaving the original chat
unchanged. Qwen answered from stale history without reading. A DeepSeek fallback
with an explicit fresh-read request required a new one-use file review; the actual
receipt and Celsius answer matched exact current bytes. C14 is checked for branch
context and fresh permissions; Qwen's unsupported stale answer remains a live
reliability failure in the checklist.

### First-user audit — Stop retains partial output (2026-10-08)

Real Qwen streaming exposed loss of already delivered public text at Stop: only
the draft survived, with no text in Earlier tasks. Two regression cases failed
before the fix, covering a side reply and the latest agent reply with three
earlier steps. The terminal UI handler now transfers incomplete text into the
existing Not saved panel, labels it Partial and retains the latest step within
the existing three-step/public-text bounds. It creates no saved partial turn and
does not include unsaved text in the next request.

Twenty-two focused Flutter checks and source analysis passed, including compact
light/dark display, restored draft, independent new request, approval/Stop and
run ownership. The normal Windows build passed. Native real-Qwen retest showed
actual streaming, a restored draft and selectable unfinished text labeled Partial;
a separate short request then completed accurately. Independent storage inspection
confirmed only the new completed exchange was saved. C08 is checked. This recovery
is transient until another chat/run; persistence of interrupted text across restart
remains unqualified. Reported live usage and advice-quality limits are recorded in
the checklist. The original profile's 49 table hashes remained unchanged.

### First-user audit — settings search and retained drafts (2026-10-08)

Native Settings search for Frequency produced No matching settings even though
Companionship showed that control. Two wide/compact tests reproduced the exact
failure before the fix. The search catalog now includes Frequency and Chatty;
all 16 focused settings/companionship checks and changed-source analysis passed.
Tests cover case-insensitive cross-category discovery, unknown-search recovery,
retained unsaved slider value, no implicit save and failed-save recovery. The
compact companionship fixture now scrolls to a lazily built toggle rather than
assuming it remains mounted after scrolling down; it passes without render fonts.
The normal Windows build and native Frequency search/open retest passed.

Native Light/Dark/System selection, immediate application across Settings/Home,
reopening, Dark restart persistence and unsaved form retention across category
and theme changes passed (P02). Close and Escape both offered the unsaved review;
Keep editing retained the draft and Save retained its value after restart. P05
remains partial until Discard is exercised; native hover-help remains separate
under P01. Original appearance/style were restored in the isolated audit profile.
No live model request was needed for these settings checks. The original profile's
49 table hashes remained unchanged before rebuilding.

### First-user audit — visible Git repository identity (2026-10-08)

Native navigation to a second public project showed its own Files hierarchy but
the enclosing Git worktree in Source Control. Enclosing discovery is intentional;
the defect was a branch-only header with the actual action target hidden in a
hint. No Git mutation was executed against that enclosing repository. The header
now names the repository, shows its selectable canonical path and identifies an
enclosing repository. The name stays readable when a long path fills the narrow
panel; the complete path remains available in its hint.

Two light/dark regression cases failed before the root-display fix. All 26
focused Git checks and changed-source analysis passed, including failed reads
retaining the identity, changed selection clearing the previous error/root,
equivalent Windows paths and a long canonical path. Saved narrow light/dark
renders were inspected and the normal Windows build passed. Native retest named
both the enclosing repository and the original project correctly, retaining
their distinct branch/status and readable names with long paths.

Files restored the selected project's hierarchy and retained tabs. Terminal Plus
opened a shell at each selected public project; switching conversations retained
the existing shell's own CWD. No terminal commands or text were entered. The
compact rail exposed all six pages with Settings at the bottom. C01/C11/T03 are
checked; Terminal input, HOME fallback and splits remain pending. This change
preserves enclosing-worktree discovery and the existing operation reviews. The
original profile's 49 table hashes remained unchanged.

### First-user audit — live scheduling creation and page discovery (2026-10-08)

Qwen failed the explicit public-project weekday 9pm request after eight bounded
calls, retaining actual read receipts and repeated blocked-call errors without
creating a task. Its progress misstated the date/time requirements. A clean
DeepSeek fallback corrected one rejected call and created the exact weekdays
21:00 Asia/Shanghai task, with no further setup instructions. Three calls,
reported totals 6,166 / 6,305 / 6,525 tokens. The visible final confirmation and
Task scheduled receipt matched the independently saved project/model/rule.

Scheduled immediately listed the new task without Refresh, with the correct
next occurrence, timezone, model, public folder, null skill and No runs yet.
Open source chat returned to its exact origin. S01/S05 are checked for this
fallback; Qwen reliability, Chinese/default-time cases and timed execution
remain pending. Task management is being qualified separately.

The original source chat changed the same task to weekday 20:30 and paused it,
then resumed it without creating a duplicate. Visible confirmations and stored
rule/paused/revision values agreed (S07). Update/pause used three DeepSeek calls,
reported totals 6,415 / 6,712 / 6,900; resume used two, 6,474 / 6,664. Native page
Pause/Resume/Skip next also passed; the skip recorded today's 20:30 occurrence
and advanced to tomorrow's 20:30. S08 remains partial until a fresh active Stop
case. Both disposable tasks were left paused.

Successful change cards displayed opaque plan IDs. Six compact light/dark
regressions failed before the fix. Cards now name the saved updated/paused/deleted
outcome with its schedule, retaining the exact ID and metadata in expanded
details. Paused/deleted receipts do not show an active Next time. Failed or
malformed results retain their error/evidence and never claim a saved change.
All 28 focused Flutter checks and changed-source analysis passed. The normal
Windows build and native restart retest passed: saved update/pause/resume cards
kept their own historical outcomes, with the exact ID/model/project readable
and no active Next time in the paused receipt. Deletion labeling passed only
fixtures; native deletion remains pending under S12. The original profile's 49
table hashes stayed unchanged.

### First-user audit — multibyte scheduling title failure (2026-10-08)

The native Chinese workday/default-time request failed to save with DeepSeek,
after an exact approved public listing. Its 247-byte prompt generated an
80-character, 208-byte title, exceeding the existing 160-byte storage ceiling.
The actual receipt mislabeled that metadata failure as a folder-tool access
failure. No new task or file change occurred; DeepSeek truthfully reported the
failed creation and intended 09:00 default (three calls, reported totals 6,134 /
6,302 / 6,479). S02/S03 initially remained pending until native recovery.

The exact prompt failed a deterministic save regression before the fix.
Title generation now retains a complete character prefix within the unchanged
80-character/160-byte bounds; the full source prompt and rule remain intact.
Chinese/Japanese/emoji/English cases verify default-time disclosure and duplicate
reconciliation. A separate real agent-flow fixture reproduced the misleading
error. Scheduling invocation errors now preserve known limit/stale/expired
messages, with a bounded inspect-before-retry fallback for unknown failures;
private backend diagnostics stay excluded.

All six scheduling bridge tests and 87 core unit/integration checks passed;
the new failure fixtures require no provider credentials.

The normal Windows rebuild and native retry passed. A fresh public-project chat
sent the same Chinese request to DeepSeek. It corrected one invalid scheduling
call and saved the weekday 09:00 Asia/Shanghai task, with a Default time receipt
and an accurate Chinese final confirmation (three calls, totals 6,200 / 6,336 /
6,538). Scheduled immediately displayed its actual model, project, source,
weekday rule and next due time. The stored title is a complete 160-byte prefix;
the full 247-byte prompt is unchanged. No file operation was requested in this
retry. The disposable task was paused after inspection.

Separate native Chinese negated-example and quoted-document interpretation
requests produced explanations with no tool calls (one call each, 6,095 and
6,112 tokens). All three complete stored task records stayed byte-identical
across both controls. S02/S03 are checked for these DeepSeek flows; arbitrary
languages, Qwen competence and execution of the new task are not inferred.

### First-user audit — editor search panel covers the document (2026-10-08)

Native Editor actions → Find/Replace hid the entire disposable catalog document.
The panel's Column expanded to the available editor height inside the editor's
overlay Stack; PreferredSize only advertised padding and did not constrain it.
Four wide/compact light/dark regressions reproduced an actual 608-pixel overlay
advertised as 56 pixels. The panel now shrinks to its controls, with enough
advertised space to keep the first document line below both search rows.

All 18 focused editor, layout, file-host and workspace tests passed, along with
changed-source analysis. The new regressions select document text below each
panel and apply a Unicode replacement, checking dirty retention and Close Find.
Real search-isolate work runs in the test's bounded real async scope; fake-clock
completion alone does not establish replacement.

The normal Windows rebuild and native retest passed. The visible Garden match
remained readable below Replace; replacement with 海风 pinned the preview and
marked it dirty. Opening the README preview retained that tab (F02). The source
stayed byte-identical until an explicit save. A settled owned-process
interruption/relaunch restored both tabs and the exact unsaved Unicode buffer
without source writes (F10). Dirty Close → Keep editing retained the buffer;
fresh Dirty Close → Save saved and closed it. Actual saved bytes were exactly
`Shelf: 海风\r\nBooks: 3\r\n`; the README stayed unchanged. Reopen showed the
clean saved content. Pointer dragging reordered tabs, split at the right edge,
and moved between groups; Move tab to next group and Split down also worked
through the menu with both contents preserved (F05).

F02/F05/F10 are checked. F03/F04/F07 remain partial for their additional physical
shortcut, undo/redo, BOM, quick-open/line-navigation and Discard cases. No model
call or Git mutation was needed for this file-editing check. The original
profile's 49 table hashes stayed unchanged.

### First-user audit — file boundaries and disk conflict recovery (2026-10-08)

The normal `8abca4b` build passed F11/F12/F13 through native interaction.
Three disposable edge cases (3-byte binary, 100 KiB line, 5 MiB text) showed
read-only previews, explained their applicable bounds and disabled Save. Their
original bytes remained identical, and normal editable tabs stayed usable.

An unsaved `catalog-copy.txt` contained `Books: 7`, with saved source still 4.
Qwen declined without requesting a read (one call, 6,306 tokens). DeepSeek's
fallback requested and completed the exact single-file review, then reported 4
(two calls, 6,258/6,295 tokens). Native selection attachment disclosed its unsaved
version/range, offered the chosen idle project chat, and populated only its
draft. Preview contained just the selected line and provenance. No generation
started until explicit Send; DeepSeek then reported 7 without a tool call
(one call, 6,446 tokens). Saved bytes stayed at 4. This qualifies the sharing
boundary through fallback, not Qwen competence.

F06/F08/F09 remain partial. Both split views shared replacements and dirty/clean
state. Stale Save retained editor 4 and disk 5. Compare showed both; changing
disk to 6 while the review was open invalidated Keep my edits with a fresh-
comparison recovery message. Fresh Compare → Keep my edits left disk at 6;
explicit Save then wrote editor 4 with exact Unicode/CRLF preservation.
Create produced an empty file. Save as copied exact bytes and retargeted both
views while retaining the original. Create at the existing copy path opened
the retained file without overwriting it. Shared undo, independent long
scrolling/compact selection, distinct rename, Save-as collision and Reload disk
are pending. No destructive native UI action or terminal command was used.

Native DeepSeek denial recovery also passed A04. Deny on the README read left
a denied receipt; the final response acknowledged the denial, claimed no file
contents and offered a user-requested retry or alternate sharing. No tool
executed and no automatic retry occurred (two calls, 6,728/6,804 tokens). The
earlier Qwen limitation remains recorded separately.

### Windows CI — MCP inspection timeouts under CPU contention (2026-10-09)

Both reported bridge MCP tests passed in isolation (3 tests, 7.62 seconds),
but twelve copies restricted to the same single CPU reproduced both exact
30-second inspection failures in every copy. Inspection fingerprints the real
91,694,408-byte Node executable before and after contacting the fixture server;
unoptimized SHA-256 made this CPU work sensitive to concurrent tests.

The test profile now optimizes only the `sha2` dependency. Application code
retains debug checks, and the production 30-second deadline, cancellation,
fingerprint comparisons and fixture subprocesses are unchanged. The same twelve
copies passed after the change (36 tests; 24.79 seconds overall, with another
build running concurrently). The isolated group passed in 0.76 seconds.
This reproduces and resolves the failure locally; a hosted Actions rerun is
still needed to establish the result on GitHub's runner.
`cargo test --workspace --locked` passed, including all 165 enabled bridge tests
and 11 MCP adapter tests (cancellation/deadline and malformed-server recovery
included). The optional live companionship bridge test remains ignored.

### Background first-user audit — scheduled Stop recovery (2026-10-09)

The scheduled owner restored its prior empty draft correctly, but used ordinary
chat recovery text claiming a draft had been restored. The frontend now reports
“Scheduled task stopped” and directs an explicit rerun through Scheduled. A
subsequent ordinary request on the same controller still restores its own draft;
no automatic retry or scheduler prompt is inserted into the composer.

The new regression failed on the original wording. The repaired flow passes
33 focused scheduling/owner/recovery widget tests, analysis of `lib` and `test`,
and the normal desktop build performed before the interrupted native retest.
Six Stop tests cover 390/1127 px, both themes, and ordinary agent/side-chat
recovery. Saved fixture renders were inspected with readable Windows fonts;
the affected message wraps without clipping and the empty composer remains
usable. These are widget renders, not normal-window screenshots. S08's final
native message inspection remains pending until the user makes the computer
available. No computer-use input was issued during this background audit.

### Background first-user audit — regression fixtures and live memory (2026-10-09)

The existing normal Windows bundle passed all eight regression save/restore
pairs (16 stages, no provider credentials or live requests). This does not claim
a new build, native input qualification or completion of pending checklist items.
Four stale fixtures were repaired: manual continuation now sets an explicit
output cap and separately verifies two-call default recovery with retained
partial text; generation profiles read `agent.usageByCall` and preserve an
unreported total; tiny-context memory testing checks foreground refusal with no
HTTP request/turn, a retained draft and successful explicit retry after capacity
is increased; skill tests allow exactly the two scheduling tools on the human
turn while drafting/evaluation requests must remain tool-free. Existing timeout,
denial, cancellation, malformed evidence and frozen-promotion checks still pass.

Live qualification scripts now read current per-call usage when legacy turn
usage is absent, without double-counting or inventing missing fields.
Eight offline checks of both scripts' actual reporting functions passed for
legacy, per-call, mixed and absent usage (including unknown totals).
Ten earlier public memory turns reported 41,284 input and 1,326 output tokens (42,610 total);
their saved metadata supplied the missing initial report totals without further
requests. Image maintenance reported 421 tokens separately. Automatic capture,
fresh-chat recall, project correction/isolation and synthetic image recall passed
in those bounded cases. DeepSeek sometimes requested manual memory changes
despite successful automatic capture, and some replies reached the deliberately
small output cap. These model limitations remain recorded, not product passes.

At the user's request, the audit now selects configured `qwen3.7-flash` first,
with DeepSeek available when a demonstrated limitation warrants fallback. A
three-turn background check on a configuration-only isolated profile passed:
Memory Off did not invent the codename, enabling memory captured the supplied
fact without an extraction model call, and a new chat returned it exactly.
All three replies completed within the 512-token/20-second request bounds;
reported totals were 7,478 tokens. Only public synthetic prompts were sent.
No native UI action, app restart, original-profile write or credential-vault
change was performed. Native settings/inspection/forgetting checks remain pending.

### Cross-project task blocker and immediate access modes (2026-10-09)

The supplied exported conversation shows local discovery/read receipts followed
by a claimed outside-folder restriction, with no attempted command and no
access-denied receipt. The actual command adapter already permits literal
absolute arguments with account-level access. New command-specific context
describes that route only when `run_command` is advertised, and skill guidance
no longer declares all outside resources unavailable. Missing locations, genuine
OS denials, host reviews and ambiguous task intervals remain distinct concerns.
The private transcript was inspected locally and is not included in fixtures.

The composer now offers an immediate Review / Auto edit / Full access menu;
Settings uses the same presets. Choices persist for the current chat with no
expiry. Existing custom grants remain unchanged until an explicit replacement,
and are identified as custom. Review remains available during work to revoke
automatic access; expanding access is disabled while busy. Stale saves retain
the saved mode and require explicit refresh/reselection. No global grant
inheritance or change to OS/file/tool boundaries was added.

The new direct-selection widget and command-guidance tests failed before the
fix. Seven permission widgets pass, including reopening, compact/light/dark,
custom-grant preservation, busy revocation and stale-save recovery. Eight shared
settings-help checks and four saved-render cases also pass; inspected renders
show concise mode rows with hover hints and no duration/acknowledgement/Save
controls. Static analysis is clean. All 81 core unit tests, seven core memory
integration tests and 13 command tests pass, including an actual sibling-file
read through Node with Unicode paths and unchanged source bytes.

An isolated normal-entry Windows build passed through the standard desktop
build entry point. Its runtime source matches the working tree; the running
app and original profile were left intact. The candidate's native-host fixture
passes per-operation Review, automatic covered access, budget preservation,
revocation before dispatch, stale-save refusal and other-chat isolation. That
fixture now explicitly sets its three-tool budget rather than assuming an old
product default; the previous implicit-default check requested 63 reads.

Two bounded public cross-project report cases used configuration-only isolated
profiles, with Memory Off and an eight-model-call limit. `qwen3.7-flash` attempted
commands but repeatedly supplied invalid arguments; it retained a helper script
and paused without creating a report (eight calls, 53,284 reported tokens).
Blocked arguments were not retained in run events, so their precise malformed
shape is not established. This is a failed model case, not a successful report.
DeepSeek V4.1 Flash completed the same task in five calls (36,118 tokens): a
successful command read sibling evidence, a file creation wrote all three
expected identifiers, and a fresh read verified the report. Evidence bytes
remained unchanged and Full access needed no approval prompts. Its final prose
loosely described host review despite automatic access; the actual receipts
establish what ran. Both profiles retained Full access without expiry across a
separate process reopen, with no extra model request.

Candidate handoff and the original task's native retry remain pending while UI
work is paused at the user's request. Background model/widget checks do not
establish that the currently running binary has the fix or that Qwen reliably
completes cross-project work.

### Background audit — developer workflows and context preview (2026-10-09)

The next batch uses disposable projects and an isolated normal-entry Windows
candidate without desktop input. Editor integration passed BOM/CRLF preservation,
project isolation, stale save/compare/rebase, binary/long-line recovery, successful
file operations despite checkpoint-write failures, and fresh-process dirty-buffer
recovery without writing the recovered source. Git integration passed staging,
hunk/index ownership, commit/hook recovery, selected stash/discard and local
bare-remote push/fetch,
owned-job cancellation and in-app large-diff paging with stale-page refresh.
The viewer cases include a 17 MiB file with a small change and over 60,000 patch
rows. Attachment fixtures passed snapshot/export/restart, image-wire handling,
scope, rejection recovery and tamper refusal. Scoped-setting precedence, stale
saves, frozen run values and changed model-capacity recovery also passed.
Thirty-five related frontend tests passed. These results qualify the exercised
background seams; physical input, native dialogs and pointer behavior remain open.

The instruction/summary fixtures exposed an actual context-preview mismatch:
human requests advertised scheduling tools and response-limit guidance that the
preview omitted. Some external guidance was appended again at agent entry, so
saved initial accounting also differed from the first sent request. Preview now
includes the ordinary human tool catalog, including available project integrations,
and shares scheduling/response preparation with sending. Response and external
guidance are included before memory selection and token accounting and are
idempotent at agent entry. The new no-request side-chat preview regression failed
before repair. History trimming and its saved initial token snapshot are checked
against a captured first request. All 255 core/bridge checks passed (one optional
live test ignored); two focused recall checks passed after the final ordering
adjustment.

The skill fixture now explicitly requests its named skill and separately verifies
that an irrelevant greeting includes the catalogue without its full instructions.
Side chats allow only the two scheduling tools, preserving the absence of project
tools. Old fixed schema-number assertions in these lifecycle fixtures are replaced
by database integrity checks; retained data/source and mutation checks remain.

A public text attachment also passed a configured `qwen3.7-flash` check: after
attaching the note, its disk source was changed. The model returned the exact
captured identifier rather than the newer disk value (one call, 2,711 reported
tokens; 2,471 input and 240 output including 229 reasoning). A separate process
reopened the original snapshot and reply without another request. The first
probe's preflight compared LF text with a Windows CRLF snapshot and stopped before
any model call; a fresh fixture fixes its own newline setup. This was not an app
snapshot failure. No credentials, private transcripts or original-profile writes
were involved. This does not establish native attachment-picker behavior.

The final isolated normal-entry Windows build passed; runtime source matches the
working tree (bridge SHA-256
`fbb27ec627a310ce8ea4b79716c6a4c87736173e4c2f50e4c81ac13d7e7c85f9`).
All eight fresh save/reopen stages passed for summary, workspace instructions,
project/global skills and automatic memory. Exact captured message/tool definitions
and saved initial token accounting remain checked, rather than relaxing the
preview equality assertions. Recovery covers changed/missing sources, stale
review tokens, failed saves, Stop/deadline, tiny-context refusal with no request,
and explicit retry. Failed first-batch logs remain alongside the successful fresh
rerun under ignored `output/background-audit-20261009-01/`.

The audit stops before the next native keyboard/IME, copy, permission-selector and
Scheduled Stop checks. No computer-use input, normal-app replacement/restart or
original-profile write was performed. Native checklist items remain unchecked.

### Native first-user audit — input, context and scheduling recovery (2026-10-09)

After the user made the computer available, the isolated normal-entry candidate
at `32fdbe6` passed five more checklist flows: C06, C07, P10, P11 and S08.
The checklist now has 50/128 checked; the other items remain open or partial.
No new runtime change or rebuild was needed for this batch.

Native input verified multiline prose, fenced code, Shift+Enter newline, Enter
newline inside code, Right-Control+Enter submission from code and Enter submission
from prose. Chinese IME selection committed `啊`; IME confirmation did not send
prematurely. Literal text injection and Left-Control shortcuts were inconsistent
in Flutter, including a regular summary text field. Menu paste and Right-Control
worked after explicitly clicking to refocus the editing surface. Physical
Left-Control behavior remains unqualified; this automation discrepancy does not
establish a product defect. Copy code and Copy message were pasted into a scratch
editor and matched the public response, including its Markdown fences.

The context inspector distinguished reported usage from its 2,561 / 131,072
next-message estimate. Opening it left existing messages/runs unchanged. Summary
review showed the exact complete turn; Qwen generated a draft that was edited
before saving. A later Qwen reply returned the saved reference `AUDIT-93`, while
the original history remained visible. This newly saved native summary has not
been reopened in a fresh process; the earlier background reopen evidence is
separate. Inspector dismissal with Escape remains unqualified.

Scheduled Run now retained the existing task's pinned DeepSeek model. Stop at the
public README read approval cancelled without authorizing the read, showed
Stopped and retained the correct result message: “Scheduled task stopped. Open
Scheduled to run it again.” The composer stayed empty; the occurrence is saved
as cancelled with no error. Earlier native Pause/Resume/Skip-next coverage plus
this Stop retest completes S08 and closes its false draft-restored message gap.

Four bounded Qwen foreground calls reported 9,586 input and 473 output tokens
(10,059 total, reasoning included in output). One additional Qwen summary request
and the cancelled pinned DeepSeek run have unavailable usage in retained metadata;
their usage is not treated as zero. Two submission probes returned `OK.` plus
their supplied text fence, so exact `OK.`-only compliance was not established.
Request bounds were 512 output tokens and 30 seconds. Automatic memory remained
Off in the disposable profile. No original-profile write, public push or private
transcript export was performed.

Sidebar toggle/drag resize/drag hide passed, but keyboard resize remains pending
(C10). The immediate access menu was inspected; native mode changes were excluded
by the computer-use skill's security-setting restriction (P13). Host/widget
permission checks remain separate evidence. Saved native screenshots, accessibility
text, observations and usage receipt are in ignored
`output/native-audit-20261009-02/`. Pending physical editor, terminal and attachment
flows are the next audit batch.

### User-message Markdown and native audit pause (2026-10-10)

User messages now use the same selectable Markdown renderer as assistant replies.
Headings, emphasis, lists, tables and fenced code render inside the existing
right-aligned bubble. Copy message retains the complete original Markdown source;
Copy code preserves its code bytes. Attachments and provider/history content are
unchanged. The existing long-content fallback now says “Long message” for either
role and keeps all source text selectable. Image sources remain inert and links
retain the existing explicit destination inspection.

The actual chat-path test failed before repair because the user prompt remained
literal. Eighteen focused renderer/message-frame/idle checks pass after repair,
including both-role source copying, incomplete user fences, oversized user text,
unchanged assistant streaming/scroll behavior and horizontal code scrolling.
Four saved user-bubble renders cover 390 px and 1200 px widths in light/dark.
They are widget renders rather than normal-window/native-input evidence; the
render fonts are loaded only when explicitly requesting local screenshots.
Some test-font fallback glyphs remain placeholders, so these renders establish
layout and contrast rather than native font/Chinese glyph qualification. Chinese
source preservation is asserted separately. Static analysis and the normal
Windows build passed. No provider requests or original-profile writes were needed.

At the user's request, computer-use auditing is paused until further notice.
No native input or app launch was performed for this change. The feature audit
remains 50/128 checked; physical Left-Control, panel keyboard resizing and other
pending native checks are not promoted by these widget results.

### Clickable questions and custom answers (2026-10-10)

Foreground chats now advertise `request_user_input`. The model supplies short
questions and choices; the chat shows an Answer question card with a custom text
field. Back/Next retain answers and Send answer resumes the same run. No answer
is preselected. Text replies are not parsed into buttons. Background scheduled
work and delegated children do not receive the interactive tool.

Three bridge tests cover multilingual/malformed requests, invalid and stale
submissions retaining the pending card, single-use replies, separation from tool
approval, and Stop while waiting beyond an explicit task deadline. Thirty-two
focused Flutter tests pass, including the actual chat path, custom answers,
Back/Next, failed-send retry, duplicate-submit prevention, Stop/draft recovery
and existing tool/message rendering. Static analysis passes. Four saved widget
renders cover 390/1200 px widths and light/dark themes; the question card is
readable with explicitly loaded test fonts. Some unrelated sidebar glyphs remain
test-font placeholders. These are layout checks, not native input evidence.

The first packaged-bridge loopback round trip passed in English and Chinese,
including stale/replayed replies and refusal to treat an answer as tool approval.
An initial live Qwen probe opened cards and delivered the exact custom marker
in completed tool receipts, but the model ignored that marker and asked again
or gave a vague acknowledgement. This is retained as a failed probe. Tool and
system guidance now explicitly describe `answers[].answer` as the submitted
human answer, including custom text, while retaining host permission checks.
The system guidance is included only when the question tool is advertised;
other catalogs retain their previous context cost.

After the final normal Windows build, both packaged-bridge loopback cases pass.
Qwen3.7 Flash opens a card, receives a public custom date and acknowledges it in
English and Chinese on the resumed model call. The exact answer remains in the
completed tool receipt. An intermediate Chinese acknowledgement added spaces
within the date; the probe initially classified that formatting difference as
a failure. The final check allows whitespace in acknowledgement prose while
still requiring the receipt to match byte-for-byte. Both cases then passed;
this qualifies the public date/clarification flow, not arbitrary task completion
or consistent model formatting. No DeepSeek fallback was needed for those cases.

The final Rust suites pass 82 core tests and 169 bridge tests, with one unrelated
vault test intentionally ignored. Clippy, Flutter static analysis, document/link
and tracked-file publication checks pass. Three bounded live probes used twelve
model calls, reporting 34,640 input and 2,695 output tokens (37,335 total;
reasoning is included in output). Each response was bounded to 1,024 output
tokens and a 20-second idle timeout. Each probe verified the original database
unchanged. No native input, personal history copy or desktop launch was performed.
Saved reports and renders stay in ignored `output/questions-*` directories.

Computer-use remains paused. Checklist C17 stays unchecked until native UX
testing resumes; the audit is now 50/129 checked.

## October 11 audit: thinking-off image memory

A real Qwen image caption was valid JSON prefixed by a stray closing thinking marker. The reproduced failing adapter test passes after one exact leading marker is normalized only for explicit thinking off. All 47 provider checks, 465 workspace checks (two ignored), Clippy and the normal Windows build pass. Fragmentation, Unicode, literal embedded/standalone/incomplete markers, default reasoning and usage remain covered; tool arguments are unchanged. A fresh normal-bundle Qwen check captured the public blue-square image and recalled it in a new chat (5,231 foreground plus 265 maintenance tokens, within existing bounds). Original profile table digests match. Public receipt: output/background-audit-20261011-remaining/image-qwen-thinking-off-fixed/. Native Memory-page interaction remains pending; this adds no native checklist pass.

## October 11 audit: full host catalog with two MCP tools

The normal-bundle two-server fixture reproduced a start refusal at the old 18-registration bound. The supported Windows catalog is 17 host tools, one optional browser and two existing external slots; capacity now matches that bounded total of 20. A failing core regression now passes at 20 and refuses 21 and duplicate aliases before approval. All 91 core checks, Clippy and the normal Windows build pass. Single-server deny/approve/drift/Stop and separate-process restore pass; two-server shared-slot refusal, independent invocation, one-server crash with successful sibling, forget/export/reopen pass. Fixtures assert exact tool names and unique aliases instead of obsolete counts/last-item positions, and schema 38. No external capacity, per-turn/task budget or process lifetime changed. Native MCP settings/review interaction remains pending (I06-I07).

## October 11 audit: maintained background fixtures

Normal-bundle terminal/LSP qualification passes three independent project/HOME shells, Unicode, resize, actual Ctrl+C return to the shell prompt, 64-KiB polling/backpressure, owned descendant cleanup, stopped-output restoration, scoped sharing, TypeScript diagnostics/completion/navigation/rename/atomic undo/stale refusal, Rust hover/project isolation and language-process cleanup. The initial fixture raced the child exit message and submitted before PowerShell returned its prompt; it now waits for that observable prompt. Physical Flutter keyboard/IME remains pending.

Project/global skills, workspace instructions and skill export pass save and separate-process restore. Fixture assertions now include interactive questions, compare exact tool specifications without relying on registration order, and request the named skill instead of expecting an irrelevant hello to inline it. Model-details qualification checks the actual saved assistant reply rather than a legacy plain-chat event type. These fixture-only repairs change no runtime or UI behavior. Scoped settings, permissions, public attachments, clickable question continuation, summaries, comparison cancellation/reopen, task/output limits, journal checkpoints, source navigation and read-only learning-evidence retention also pass. Native dialogs/review controls remain separate checklist gaps.

## October 12: Flutter SDK version recovery in Windows CI

The failed hosted build reported `0.0.0-unknown` while its SDK revision passed
the existing guard. The workflow checked out the pinned Flutter commit with the
default single-commit/no-tag fetch. Flutter 3.47.5 derives its framework version
from Git release tags; matching HEAD alone does not establish a usable SDK
version. The SDK checkout now fetches history/tags while retaining the exact
commit pin. CI checks both cached framework version and revision before Rust
compilation. Unknown/stale version metadata refuses with release-tag and fresh
SDK/cache recovery guidance; it does not rewrite the SDK version or relax app
dependency constraints.

Three deterministic regression tests failed before the fix and pass afterward:
unknown metadata with the correct revision is refused before compilation and
recovers with the correct version, a different revision is refused early, and
the pinned workflow checkout retains release tags. These tests exercise the CI
entry point using synthetic SDK metadata and mocked child commands, not a fresh
hosted Flutter bootstrap. Release, package, launcher and regression-runner script
tests pass, along with document/link and tracked-file publication checks.

A fresh normal Windows x64 build resolved dependencies with the existing local
Flutter 3.47.5 SDK and passed using the installed Visual Studio 2026 generator
and the documented CMake path. CI retains its Visual Studio 2022 generator.
Local packaging initially refused a leftover root `native_assets.json` dated
October 11; its native-assets map was empty and matched the maintained asset
manifest. That extra file was retained outside the runtime under ignored output,
without broadening the package allowlist. The retry produced an unsigned preview
ZIP under `output/releases/ci-flutter-fix-20261012/package/` with 18 runtime files,
dependency notices, changelog, launcher and per-file hashes. Archive verification,
ZIP SHA-256 comparison and the fully extracted `Start-Dolores.cmd --check` pass
on this machine. No app launch, provider request or user-profile change was made.
Hosted CI rerun, clean-machine launch and signing remain unverified.

## October 13: first-alpha installers and release flow

Windows installer assembly reuses the verified portable runtime internally and
publishes an EXE/checksum instead of a ZIP. The install-mode dialog supports
current-user and all-users scopes, with scoped shortcuts and uninstall entries.
Uninstall retains local data by default and offers an unchecked local-data
removal checkbox. Its exact target is the current uninstaller account's normal
RoamingAppData `dev.dolores.desktop` directory. Custom data locations, external
projects, shared skills, other accounts and OS-vault credentials are retained.
The dialog discloses those boundaries. Missing C++ runtime files explain the
prerequisite before installation; nothing is downloaded by the application installer.

Four deterministic installer checks cover verified staging, existing-output
retention, missing compiler, tampered archive refusal before compilation,
scope/default-retention contracts and wrong-host/foreign-notice refusal.
The real Inno 6.7.3 compiler was downloaded from its official release and matched
the pinned WinGet SHA-256. A diagnostic installer used its own random app ID,
shortcut, ignored install directory and synthetic data; current-user install,
silent data retention, locked-data refusal, junction refusal and explicit
synthetic purge recovery pass. The first recovery assertion raced asynchronous
uninstaller cleanup; waiting for the actual data-removal boundary resolved it.
The external synthetic project survived removal. Production compilation contains
no diagnostic data-directory/purge override. This fixture does not qualify
interactive checkbox selection, all-users UAC or clean-machine installation.

Five CI regressions confirm usable pinned SDK metadata, early SDK refusal,
release-tag fetches, ordinary checks skipping native build/fixtures and release
workflow installer/artifact routing. Both workflows pass actionlint 1.7.7.
Commit/PR runs retain source/fixture checks and use dependency/compiler caches;
only tag/manual release runs build/package and upload the installer. Tag releases
validate dated reviewed notes, attach installer/checksum to a draft release and
mark prerelease tags without making them Latest. Existing release assets are
never overwritten. Four release-preparation checks cover alpha alignment,
duplicate notes, invalid versions and rollback after a synthetic partial write.
The `0.0.1-alpha` source, Flutter build number 2 and all 13 workspace lock entries
are aligned; third-party lock versions are unchanged and locked offline Cargo
metadata succeeds. Reviewed notes are dated 2026-10-13 with a new Unreleased
section. Tag CI extracts those notes without modifying the tagged commit.

macOS PKG/Linux DEB packaging and native notice-collection commands are implemented
but excluded from tag releases. Native build/install/launch, dependency
qualification, signing/notarization and uninstall remain unverified on those hosts.
Windows release qualification passes Rust formatting, Clippy/workspace tests,
browser/script fixtures, Flutter analysis, all 446 widget tests and all 16 native
save/restart steps. The initial composite run stopped because analysis included
an ignored local screenshot helper; excluding generated `output/**` preserves
that helper and makes the maintained-source analysis pass. The remaining checks
were then run separately, without repeating the completed Rust build/tests.
One optional history-audit fixture remains skipped locally because its external
history tool is absent; hosted CI installs that pinned tool. A separate broad
history audit reports historical path/session-guidance patterns already reachable
from master; no history rewrite was attempted. Current tracked-file audits pass.

The fresh normal alpha bundle assembled an internal ZIP with matching dependency
notices and release notes. The production Windows installer is 19,407,654 bytes
under `output/releases/alpha-001-20261013/installer/`; its SHA-256 and local-path
scan pass. The actual alpha payload also passes the isolated Inno install/uninstall
fixture, including retention, busy/linked refusal and purge recovery. No provider
request, normal-profile write, app launch or physical native interaction was
performed. Hosted workflows, global/UAC installation, production checkbox visual
acceptance, clean-machine prerequisite recovery and signing remain unverified.
The tag/release is prepared locally; public publication is a separate step.

## October 13: deterministic streaming deadline regression

Both hosted workflows failed the same bridge assertion after the earlier local
workspace test pass. The original fixture inherited the legacy provider contract
(`manages_stream_inactivity` false), enabling a one-second host request fallback
while each UI delivery also had a one-second timeout starting later. It assumed
the later delivery error would win and checked completion after a real 1300-ms
sleep. Timer rounding and scheduler wake-up timing could make both ready in one
poll, masking the competing request deadline locally. One successful full local
run did not establish timing stability; no repeated timing gate existed.

The exact hosted failure was reproduced locally by delaying fixture stream
startup 50 ms: expected delivery-stalled recovery, received model-request timeout.
Three tests now use a test-only paused Tokio clock, explicitly establish a full
UI queue and distinguish modern inactivity-managed delivery expiry from legacy
provider expiry. They check the boundary before expiry, retain queued public
partial text, keep failed turns out of history, verify prompt Stop and complete
one explicitly requested recovery without replaying the failed turn. Production
timeouts, provider behavior, retry policy, recovery messages and user data are
unchanged. The 50-ms synthetic startup separation remains part of the regressions.

The three cases pass 100 repetitions with eight concurrent processes (300 cases,
6.092 seconds, zero retries). Both shared CI paths now run 20 repetitions with
two concurrent processes after workspace tests; any failure or missing cases
fails the gate. Two synthetic gate tests verify that later passes cannot hide
an earlier failure and that an empty filter refuses then recovers. CI dispatch
tests confirm the timing gate runs for ordinary and full release checks. This
guards the reproduced race; it does not qualify every timing-sensitive test.
The full locked workspace suite passes, including 180 bridge tests with the
existing native-vault test intentionally ignored. Formatting, workspace/all-target
Clippy, CI/gate tests, document checks and tracked-file publication checks pass.
The normal Windows build passes with the installed Visual Studio 2026/CMake
path; hosted CI continues to use its existing Visual Studio 2022 path. These
are local results; neither corrected hosted workflow has been rerun here.
No model credentials, provider request, native interaction or profile mutation
is needed. Existing published tags are retained: rerunning the old alpha tag
still checks its old code; release the corrected commit under a new version tag.

## EH09 retained package backend - October 25

The backend separates installation from activation. Additive SQLite schema 49
retains scoped package records; legacy skill versions omit package metadata.
Complete bundles live under the locked profile, with exact source ZIP, host
receipt and payload files/directories. Publication uses held directory
capabilities, refuses links/replacement and precedes catalogue saving. A failed
save leaves the same complete bundle for explicit retry without enabling it.
Scope, session, canonical cache and saved catalogue/skill revision bind reviews.

Thirteen focused package tests pass: two core archive-contract tests, eight
bridge archive/Windows-host cases and three SQLite cases. They exercise exact
binary/Unicode resources and empty directories, same-bundle retry after failed
registration, version activation/rollback and reopen, changed-resource refusal
through both package and legacy Skills activation, export without overwrite,
expired/wrong-session reviews, live-owner refusal without cancellation, native
junction/existing-destination refusal, five-version/sixty-bundle limits, and v48
migration retaining skills and drafts. Partial removal preserves the source ZIP
and receipt. An interrupted staging fixture needs explicit original-ZIP reimport,
then removes only its verified complete/prefix bytes without unregistering the
recovered installation. Changed or unknown bytes remain for manual inspection.

Package archive identity is also part of composer selection and submission
deduplication. A forgotten/re-added name with identical text/revision/version
cannot substitute another package. The core regression and seventeen composer/
instruction widgets pass; stale chip inspection retains the draft and shows the
selection as unavailable. Full suites pass 111 core, 256 bridge (one existing
ignored test), 122 SQLite and seven/five core/SQLite integration tests. Analysis,
warnings-denied Clippy, formatting and the normal Windows build pass.
Queue/steering and live/stale Attention native UI regressions pass together in
eight seconds at output/native-ui/eh09-packages-backend-20261025/ against that
normal build. These regressions preserve their drafts, ownership and saved work;
they do not establish the still-pending package management UI journey.

This is backend and native filesystem qualification. Package management controls,
rendered Project/Global journeys and a model's bounded resource-reading tool are
still required for EH09.1. No source collection, user profile, provider credential,
dependency installation, external model or physical desktop was used. Existing
command/MCP and child permissions are unchanged.

## EH09 skill package archive admission - October 24

Core skill-package contracts and the bridge ZIP
inspector preserve exact SKILL.md, script/reference bytes and binary assets,
compute archive and deterministic content/resource digests, and inspect bounded
declarative provenance/version/dependency metadata. The inspector has no file
publication, activation, network, dependency installation or script execution.
Its limits and next integration slices are recorded in the EH09 task file.

Six focused tests pass: portable paths and metadata, complete scripts/references/
binary resources and hashes, escaping/case/file-directory collisions, truncated
ZIPs, expanded-byte limits, malformed manifests, symlinks and excessive file
catalogues. Five existing bridge skill review/export tests pass, including stale
activation, scoped global/project behavior and retargeted directory refusal.
Warnings-denied Clippy, formatting and the normal Windows build pass. No
rendered/native user-flow acceptance is claimed for this internal slice.

This slice does not yet add a package UI or a usable installed-resource flow.
Reviewed publication, failed-update recovery, retained rollback/export and
context resource integration remain required before EH09.1 can be checked.

## EH08 task attention and notifications - October 24

Seven focused attention storage tests pass within the full 108 core
and 119 SQLite suites (plus seven/five integration tests): opt-out/dedup,
stale decisions, delayed-question suppression, failed save rollback independent
of completed task results, claimed/uncertain delivery, real reopen/migration,
preference revision checks and bounded retention/paging. Existing runs are
baselined on migration rather than notifying about old history. Source cursors
are committed with derived notices; failed saves retain the cursor for retry.
Claims are one-use; restart marks unfinished handoffs uncertain without resend.

The bridge question fixture passes: opening resolves only the currently waiting
question and cannot answer it, approve a tool, restart work or revive an older
decision after a later request. Attention/AppHost/workspace widget checks pass.
Full Flutter passes 592 tests with seventeen opt-in native skips. Seven focused
Attention/run-history tests include direct navigation to a retained run outside
the recent twenty and disposal during pending native availability. The latter
first reproduced a notification sent after disposal; the fix prevents later
claims/timers while preserving a handoff receipt already in flight.
Opening a retained entry also updates the unread badge only after the Seen save
succeeds; its focused regression first reproduced the stale badge and now passes.

The final normal Windows build passes with the compiled notification adapter.
The owned native diagnostic at output/native-attention/20261024-final/
compiles the actual runner/channel code with injected Shell outcomes and checks
quiet-time suppression, failed add/modify, immutable identities, distinct
question/review notices, exact click routing, duplicate callback refusal and
timeout cleanup. Actual OS notification policy query and owned icon registration/
removal also pass. No balloon is sent by this diagnostic; it does not establish
visible display or a physical click. Windows acceptance is reported separately
from API handoff, and in-app entries remain available on suppressed/failed or
uncertain delivery. The first rendered/native live-question run reached completed
evidence but timed out on a fixture wait outside real asynchronous execution;
its database retains one question and completion, with no duplicate effects.
That diagnostic is not a passed native journey. The corrected journey passes at
output/native-ui/attention-20261024-cleanup/: a live question opens without an
answer, its explicit answer completes with exactly two provider requests, an old
question notice opens completed evidence, and another chat's Unicode draft
remains. The fixture pumps asynchronous dialog completion and disposes its owner
before Flutter's timer invariant; earlier wait/cleanup failures remain diagnostic.
The combined rerun exposed the same older fixture cleanup omission in branch and
side-chat tests, and a history fixture that sent its next message before the
previous refresh released `changing`. Their assertions now wait for admission
and dispose their owned hosts before Flutter checks timers; no task deadline or
production retry policy was expanded.

Final compact Light/Dark captures at output/design/eh08-attention-qualified/
were inspected; failed polling retains its entries and a reachable Refresh action.
Formatting, Flutter analysis and warnings-denied Clippy pass. This is local
Windows/native-boundary qualification, not a clean-machine release, a physical
notification display/click check, or general real-model reliability.
Complete Rust suites pass 108 core/248 bridge/119 SQLite tests, one existing
ignored bridge test, and seven/five core/SQLite integration tests.
The final normal build including the unread-badge fix passes. Five combined
native UI journeys pass in 21 seconds at output/native-ui/eh08-20261024-admission/:
attention, history search, edited-prompt branches, linked side chats and queue/
steering. The history fixture now waits for `changing` to release before Send;
branch/side owners are disposed explicitly. No user profile or external model
was used. EH08.2/EH08.3 pass the declared local gates with the display/physical
click and release qualification limits above retained explicitly.

## EH08 history search and organization - October 24

EH08.1 passes its declared gates. Six focused SQLite cases cover literal
English/Chinese/punctuation and short Unicode queries, tied 111-result paging,
query-bound cursors, workspace filters, pin/archive/restore, stale/failed saves,
atomic failed-turn indexing, older-profile backfill and interrupted search/index
rebuild. Draft, goal and queued input remain unchanged. Full suites pass 108
core/112 SQLite tests plus seven/five integration tests. The focused host case
confirms active ownership survives archive and refuses rebuild until idle.

Five search widgets pass, including Ctrl+K/modal preservation, page replacement,
exact navigation, failed rebuild/stale edits and compact Light/Dark layouts.
Full Flutter passes 587 tests with sixteen opt-in native skips. Formatting,
analysis and warnings-denied Clippy pass. Light search and Dark recovery captures
under output/design/eh08-search-final/ were inspected. Fixture render fonts
lack the Chinese sample glyphs; these captures qualify layout, not Windows CJK.

The final normal Windows build passes. Four combined native journeys pass in
18 seconds at output/native-ui/history-search-20261024-admission/: history,
edited branches, linked side chats and queue/steer. History uses three public
loopback provider requests, finds Chinese text in a saved answer, pins and
archives/restores the exact chat, preserves drafts/input, and archives/restores
an independently running owner without cancelling it. Rebuild refuses while
that response is owned, then succeeds after completion; the same saved answer
remains searchable. No user profile, external model or physical desktop used.
Earlier diagnostics corrected the fixture server's UTF-8 charset and ensured
an active task had saved searchable history; those failures are not acceptance.

The 1,000-turn public corpus finishes three indexed/short Unicode searches in
about 2 ms locally. The host command worker uses a one-second SQLite search
budget; rebuild/backfill has 30 seconds. Rebuild is idle-only and holds admission
until complete, preserving prior index/history on interruption. These fixtures
do not qualify arbitrary huge profiles or disk pressure. Windows notifications
and stale-click routing were subsequently delivered in EH08.2/EH08.3 above.

## EH07 managed-worktree prompt branches - October 24

Eight focused SQLite prompt-branch cases pass, including stale revision,
occupied/wrong-source targets, failed transaction rollback and same-target retry.
The real-Git host case passes with source dirty bytes and staged index unchanged,
target committed bytes, active-task refusal, exact retry preserving a newer target
draft, and subsequent occupied-target refusal. The picker recovery case retains
edited text and one created checkout across selection and final-save failures
without sending a model request. Seventeen focused branch/worktree widget tests
pass; eight updated branch tests include compact Light/Dark renders under
output/design/eh07-managed/. All four managed-choice/picker renders were
inspected: controls fit and retry/creation remain explicit.

The normal Windows build passes. Full Flutter passes 582 tests with fifteen
opt-in native cases skipped. Warnings-denied Clippy, Rust/Dart formatting and
final Flutter analysis pass. Combined rendered/native managed/shared branch,
side-conversation and queue journeys pass in the normal bridge under
output/native-ui/prompt-managed-branches-20261024-final/, retaining four receipts.
Each branch uses six public loopback provider requests. Managed creation reviews
its exact base commit, retains source staging/dirty bytes, attaches an unsent
edited draft, then Send reads committed target bytes rather than the newer
source file. Historical tool receipts enter context as evidence without replay;
old selected/later answers stay in the local archive. Parent draft, goal and
history survive. Deleting the parent leaves that archive readable.

Earlier diagnostic attempts are retained: the first combined in-file variant
attempted to reopen a process-lifetime bridge after shutdown, then separate-file
checks revealed fixture taps before polling settled and below a lazy panel's
viewport. A subsequent check found Windows Git autocrlf changed fixture checkout
bytes. Separate native files, settled enabled controls, bounded scrolling and
repository-local explicit line endings correct the fixture without changing
product lifecycle, test assertions or Git/user configuration. These diagnostic
runs are not successful acceptance evidence. No physical desktop, external model
or whole-volume resource-pressure reliability claim is made. The complete
two-thread Rust regression gate passes 108 core, 246 bridge and 106 SQLite
tests, plus seven core and five SQLite integration tests (one existing bridge
case ignored). EH07.4 is complete within these declared Windows bounds.
# EH09 package controls - October 25

Scope: the Settings → Skills → Packages inspector, using the existing reviewed
package commands. This slice establishes rendered/widget interaction and a normal
Windows compilation gate; it does not establish the connected native bridge or
model/resource journey. EH09.1 remains open.

- **Basic flows:** deterministic fixtures exercise both Project and Global import,
  inspection, installation without activation, explicit activation, a second
  version, identity-bound saved-version rollback and original-ZIP export. Global
  management opens without a chat or provider. Repeated ordinary activation of
  the already active package is disabled. Scope changes cancel the old review,
  keep import name/path and never activate a package in the new scope.
- **Recovery:** a failed catalogue save retains the same Install token for retry
  and leaves active versions alone. A stale activation retains the review and
  export; Refresh cancels it and reloads the catalogue. Export overwrite failure
  leaves an explicit retry. A damaged-payload fixture refuses ordinary inspection,
  then opens retained-original inspection and exports the original ZIP. Removal
  requires confirmation; Keep retains the installation. Pending installation
  disables duplicate publication and navigation. The backend's previously
  qualified byte/receipt checks remain the authority for these commands.
- **Checks:** package, existing Skills and Settings suites pass **28 tests**:
  `D:/Temp/dolores-eh09-package-controls-widgets2.log`. The initial combined command
  named a nonexistent `settings_window_test.dart`; it was corrected to
  `settings_test.dart` and the combined suite rerun successfully. Focused Flutter
  analysis reports no issues. Document/link checks pass. The normal Windows x64
  build through `python scripts/desktop.py build` passes:
  `D:/Temp/dolores-eh09-package-controls-build.log`. The helper used the configured
  Visual Studio 18 fallback after Flutter's initial toolchain detection refusal.
  Publication and staged-whitespace checks also pass before commit.
- **Rendered UX:** inspected six saved **390 × 740** Light/Dark import, update and
  stale-activation recovery images under `output/eh09-package-controls/`.
  Instructions/files/origin/dependencies/identity use progressive disclosure;
  errors and wrapping recovery actions remain in a fixed footer. Reproduce with
  `flutter test test/skill_packages_test.dart --dart-define=DESIGN_RENDER_DIR=<absolute-output>`.
- **Remaining gates:** native file dialogs, embedded Settings interaction against
  the real bridge, Project/Global persisted rollback/export, verified bounded
  package-resource tool reads and normal provider requests remain subsequent
  EH09.1 acceptance. No live provider, user's history/settings or physical desktop
  was used, and no model-reliability claim is made.
# EH09 verified package resources - October 25

Scope: per-run `read_skill_resource`, package identity in prepared context and
preview, registry/planning/logged-approval integration, and the connected normal
Windows package/composer/tool fixture. H18 remains open for the capacity recovery
gap below; EH09.2–EH09.5 and EH10 are not implied complete.

- **Authority and reads:** bind only the exact prepared `SkillSource` packages,
  including scope and archive identity. Compaction can rebind the same snapshot,
  not substitute another one. Project overrides, inactive/unrelated packages,
  wrong identities and portable-path escapes are refused. List returns paged
  metadata; UTF-8 reads provide byte-offset continuation. Binary resources return
  metadata only. Requests accept at most 4 KiB and return requested 1–16 KiB JSON
  pages including escaping/metadata, capped by the normal tool allowance. Complete
  receipt/ZIP/payload checks precede preparation and invocation; requested bytes
  are re-read and hashed through the owned capability before sharing. No script,
  dependency, command or MCP execution is added. Existing folder grants do not
  grant package reads. Normal Review/Full Access, permission revision checks,
  budgets, cancellation and retained tool receipts apply. Children gain no tool.
- **Deterministic failures/recovery:** four reader tests cover selected scope and
  package substitution, missing/escaping paths and unknown/oversized arguments,
  UTF-8 plus heavily escaped text pagination, list continuation, binary metadata,
  changed bytes between preparation and invocation, cancelled dispatch and a
  tampered target. Changed data shares no bytes; original ZIP recovery remains
  usable. Restoring exact fixture bytes allows an explicit read retry. Registry
  pinning and read-only planning classification are also checked.
- **Checks:** full core **111**, its integration **7**, and bridge **260 passed,
  one existing ignored** (`D:/Temp/dolores-eh09-resource-rust.log`). Four focused
  reader tests pass again after the final target-label adjustment. One attempted
  parallel relink was refused while the earlier Windows test executable was
  running; waiting for that exact suite to finish and retrying passed without
  stopping it. Clippy with warnings denied passes
  (`D:/Temp/dolores-eh09-resource-clippy.log`). Package/Skills/context suites pass
  **29 tests**, with the opt-in native case correctly skipped in ordinary runs
  (`D:/Temp/dolores-eh09-resource-widgets.log`). Focused Flutter analysis is clean.
  The normal Windows x64 build passes
  (`D:/Temp/dolores-eh09-resource-build.log`), using the configured VS18 fallback.
- **Connected native bridge/rendered controls:**
  `output/native-ui/eh09-package-resources-20261025-final/` retains the successful
  test log and `skill-packages-receipt.json`. Both Project and Global flows import
  complete public ZIPs with text/script/binary/empty-directory resources, install
  without activation, explicitly activate/update, restore saved v1 as activation
  v3, and export byte-identical original ZIPs. Damaging v2's payload refuses normal
  inspection; retained-original inspection/export stays usable and v1 remains the
  selected version. `/` chooses the exact reviewed skill; preview and persisted
  reply metadata retain its package ID. A public loopback provider receives the
  list, v1 reference text and binary metadata through **three approved read calls
  and four model requests per scope** (six reads/eight requests total). All saved
  tool receipts have status `read`; changed bytes and v2 reference content never
  reach the provider. Only fixture activations are disabled after checking.
- **Fixture repairs:** the first native attempt selected Plan mode because
  `/review` also matches its description; the fixture now selects the exact skill
  row. The second inspected cleared transient tool records after save; the final
  assertion instead verifies the three persisted successful tool receipts through
  the normal messages API. Earlier failed evidence remains alongside the final
  profile. Neither failure required a product-code workaround.
- **Rendered UX:** compact 390 × 740 Light/Dark package-resource context disclosure
  images under `output/eh09-package-resources/` were inspected. Archive/content
  identity and resource base wrap inside the scrolling body; Close stays fixed.
  Existing package import/update/recovery renders and checks continue to pass.
- **Remaining gates:** source audit found that five installed versions can all be
  protected by the five saved activations. A sixth distinct install is refused,
  and direct removal offers forgetting the whole skill. EH09.1
  stays open until reviewed recovery preserves the current active version and
  qualifies a sixth update. EH09.5 must audit pinned scheduled-package retention
  and child/lifecycle ownership. Chooser paths in this fixture are injected;
  physical file dialogs, real-model reliability and other platforms remain
  unqualified. No user provider configuration/history or private transcript was
  used. Fixture evidence does not establish general model reliability.
# EH09 package capacity recovery - October 25

Scope: explicitly forget an older package activation without forgetting the
skill, its current version or its resources. The existing five installed
versions/name, twelve names/scope and sixty physical bundles/profile limits are
unchanged. No migration, silent eviction, tool grant or script execution is added.

- **Reproduction:** `five_activated_installations_refuse_a_sixth_without_clearing_the_current_version`
  first passed against the earlier behavior: five distinct installations and
  activations protected all five packages; a sixth install and direct removal
  refused while the current v5 stayed intact. This established the capacity path
  before adding recovery. The earlier UI's direct removal guidance offered
  forgetting the whole skill; the new route releases a reviewed older activation.
- **Recovery contract:** `ForgetVersion` binds the existing review token's
  session, scope/root, saved revision and exact package/version. One SQLite
  transaction removes only a noncurrent matching activation and advances revision.
  Current version, enablement, surviving version numbers/rollback provenance,
  installations, files and drafts stay intact. Current, missing/mismatched and
  stale versions refuse. An active owner refuses without cancellation. A failed
  save retains the old state/review for explicit retry. Forgotten activations do
  not themselves remove package registrations or files; Remove remains a separate
  confirmed action with the existing reference and verified-byte guards.
- **Store/bridge evidence:** both scopes preserve current v5 through injected
  save failure, old-activation retirement, installation removal and retry of the
  sixth already-published archive, then activate v6 without renumbering survivors.
  Reopen retains both scope records. The bridge case additionally checks current
  and wrong-package refusal, active-owner preservation, stale review after another
  retention change, and a damaged older payload. Retained-original review can
  retire its activation; removal refuses changed bytes and keeps them inspectable
  while the freed registration permits the sixth install. Explicit restoration of
  exact fixture bytes then allows verified cleanup. The current package/draft
  stays unchanged until explicit activation of the sixth package.
- **Checks:** full core **111**, bridge **261 passed / one existing ignored**, and
  SQLite **124**, plus their **7/5** integration checks pass:
  `D:/Temp/dolores-eh09-capacity-rust.log`. Clippy with warnings denied and focused
  Flutter analysis pass. Package/Skills/Settings widgets pass **30 tests**:
  `D:/Temp/dolores-eh09-capacity-widgets.log`. The normal Windows x64 build passes
  (`D:/Temp/dolores-eh09-capacity-build.log`), using the configured VS18 fallback.
- **Rendered UX:** four compact **390 × 740** Light/Dark confirmation and failed
  save renders under `output/eh09-package-capacity/` were inspected. The dialog
  states exactly which old activation is forgotten and which current version is
  kept. Keep version cancels without mutation; the current version has no Forget
  action. Failure keeps the version/review and recovery controls reachable.
- **Connected normal bridge:**
  `output/native-ui/eh09-package-capacity-20261025/skill-packages-receipt.json`
  and its passing log cover both Project and Global controls. In addition to the
  prior package import/export/rollback/resource-provider journey, each scope
  installs five distinct packages, observes the sixth-install refusal, confirms
  forgetting the older v3 activation of package 1, removes that unused
  installation, and retries/activates package 6. Because the earlier journey had
  rolled back once, package 5 is saved activation v6 and package 6 becomes v7.
  Package 5's current identity and reference bytes remain intact throughout
  recovery. The retained skill is never forgotten. All six reviewed resource
  calls/eight loopback provider requests still pass; only fixture activations
  are disabled afterward.
- **Bounds and remaining work:** EH09.1 is complete under the archive/backend,
  controls, resource and capacity acceptance records. EH09.2–EH09.5 and EH10 stay
  open, including pinned scheduled-package and child/lifecycle retention audits.
  Native chooser paths are injected, physical chooser dialogs and general
  real-model reliability remain unqualified, and other platforms need separate
  qualification. No user configuration/history, credentials or private transcript
  was used. These fixture passes do not establish real-model competence.

# EH09 remote MCP transport foundation - October 25

Scope: an internal `dolores-tools-mcp::remote` client, not a new connection type
in Settings. EH09.2/H19 stays open for reviewed persistence, secure credentials,
OAuth, health/search controls and the connected normal-bridge journey. Existing
stdio records, process isolation, permissions and normal app controls are unchanged.

- **Protocol:** current `2026-07-28` stateless `server/discover`, per-request
  metadata and mirrored HTTP method/name/annotated primitive headers; pinned
  handshake/session compatibility for `2025-11-25`, `2025-06-18`, `2025-03-26`.
  Inspection probes can fall back after an unrecognized HTTP 400 or legacy
  method-not-found/invalid-request response; recognized modern version/header
  errors, auth refusal, redirects and server errors never cause downgrade.
  Business calls never negotiate or replay. No 2024 two-endpoint transport,
  sampling/roots authority, MRTR continuation or subscription worker is added.
- **Bounds:** one 30-second deadline across connection/discovery/call/close;
  64 KiB per JSON/SSE message, 256 KiB cumulative received body bytes, 64 stream
  notifications/requests and 128 client RPC requests. Catalogs retain the local
  profile's 32 tools, four pages and 32 KiB of serialized admitted definitions;
  arguments are objects within 4 KiB. Unsupported required task execution is
  excluded. Invalid modern header annotations exclude only that tool and report
  a bounded reason. No limits are raised or silently reset per request.
- **Credentials and authority:** HTTPS endpoints, with HTTP only for explicit
  loopback; URL user info/fragments are rejected. Bearer values are separate,
  sensitive request headers; redirects and HTTP-client automatic retries are
  disabled. Known credential echoes in metadata refuse admission; tool text is
  scrubbed and credential-bearing object keys refuse. Error Debug/Display omits
  raw bodies, endpoint strings and authentication challenges. The bounded raw
  challenge accessor is untrusted input reserved for the later OAuth layer.
  Calls require the exact freshly admitted definition. Failed refresh or remote
  call clears the callable catalog; explicit inspection is required to recover.
- **Failure/recovery evidence:** public loopback HTTP fixtures exercise modern
  discovery/header mapping and Unicode/base64 values; legacy probe, initialized
  notification, SSE and explicit DELETE/405 cleanup; 401/403, redirect and 503
  refusal; changed protocol pin and expired session; stale catalog invalidation;
  malformed/mismatched/missing result, truncated SSE, frame/wire/notification and
  catalog count/page/byte limits; ambiguous annotation paths and duplicate names.
  Credential metadata refuses without granting a tool, while echoed result text
  is scrubbed. An explicit successful inspection recovers after auth refusal and
  sends only one business call. Stop after a confirmed dispatch returns uncertain
  effects without replay; legacy Stop notifies the exact request ID. Cancellation
  while initialization's notification is blocked also terminates. The in-flight
  deadline test confirms dispatch before pausing/advancing the Tokio clock; the
  later-call test confirms earlier work does not reset the operation deadline.
- **Checks:** `cargo test -p dolores-tools-mcp` passes **29 tests**: **17** remote
  cases plus **12** existing stdio/credentials/isolation cases; doc tests pass.
  Log: `D:/Temp/dolores-eh09-remote-tests.log`. Clippy for this crate and all
  targets with warnings denied passes: `D:/Temp/dolores-eh09-remote-clippy.log`.
  The final-source normal Windows x64 build passes using the configured VS18
  fallback: `D:/Temp/dolores-eh09-remote-build-final.log`. Documentation links and
  the staged publication/diff checks pass. No desktop launch was needed for this
  internal library slice; it introduces no rendered control.
- **Remaining gates:** no Settings connection, vault binding, OAuth/refresh,
  searchable health controls, persisted catalog/revision or normal permission
  receipt is claimed by this library. No real provider or external MCP service,
  user configuration/history, credential or private transcript was used. Native
  interaction, rendered integration and real-model reliability remain open.
  Explicit close is bounded; dropping/cancelling a client does not guarantee
  remote legacy-session deletion. No SSE resumption or automatic business retry
  occurs. Later lifecycle integration must qualify cleanup and uncertain-effect
  recovery through the host. H21 remains deferred.

# EH09 remote MCP targets and approved invocation - October 25

Scope: core target/preview contracts, the existing SQLite MCP catalogue and
shared tool-plugin factory, bound bearer payloads and the approved HTTP driver.
This is a backend slice of EH09.2, which remains open for foreground connection
controls, OAuth, health/search and the connected normal-bridge/model journey.

- **Saved targets:** each connection has exactly one local launch or remote
  endpoint. Remote records have a bounded label, endpoint and explicit None or
  Bearer authentication; no dummy executable, file fingerprints or command
  isolation is accepted. Local JSON retains its old launch object and omits the
  absent remote field; legacy identities, aliases, credential payloads and SQL
  rows are preserved. The same table, revision CAS, four connections and two
  active tools per folder apply across both transports. No migration or limit
  increase is needed. URL policy is shared by core validation and HTTP transport.
- **Bound credentials:** remote vault payloads bind a canonical working folder,
  connection identity, exact target and bearer purpose. Saved configuration and
  previews contain opaque references/names only. Invalid/missing/locked keys,
  changed root/ID/endpoint/purpose and a local-format payload refuse. Known keys
  in a connection label or URL refuse before encoding. No key is read during
  factory construction, specification or preparation. The approved invocation
  checks Stop before lazily starting a vault worker, and the operation deadline
  includes that worker and subsequent HTTP work. An already running native vault
  read itself cannot be forcibly interrupted; its result is discarded on Stop
  and no later network action follows it.
  Bearer values are at most 4 KiB of visible ASCII; canonical root bindings are
  at most 4 KiB of UTF-8 and encoded vault payloads at most 16 KiB.
- **Approval and metadata:** the shared factory routes remote targets through
  the ordinary ToolPlugin boundary. One-use previews bind alias, connection
  revision, endpoint and exact arguments, with no process-isolation claim.
  Tampering, discard, replay and pre-dispatch Stop refuse without a vault read
  or HTTP request. Invocation pins the saved protocol, rediscovers tools and
  compares every selected definition plus server name/version before one call.
  Changed schema/description/server identity or unavailable credentials never
  sends `tools/call`. This driver adds no automatic negotiation, reconnect or
  business retry. Existing outer ownership/permission/effect logging remains
  the integration route; a GUI approval/receipt journey is not claimed here.
- **Output and recovery:** both transports share the existing bounded text-only
  decoder. Known bearer echoes are scrubbed by the HTTP transport. Nontext output
  refuses. Stop after a confirmed dispatch reports possible remaining effects
  and sends no replay. A successful text result remains usable if legacy-session
  DELETE fails; its `cleanupWarning` explains that repeating the tool is not a
  cleanup action. Results including warnings retain the 16 KiB encoded limit;
  oversized/unsupported post-dispatch output explains the actual limit and asks
  for inspection instead of repeating the operation to retrieve output. Tests
  cover nontext, more than 8 KiB of text and JSON escaping beyond 16 KiB.
  This does not prove remote session deletion after client drop,
  failed connection setup or cancellation; EH09.5 retains that lifecycle gate.
- **Deterministic evidence:** full core **113** plus **7** integration tests,
  bridge **261 passed / one existing ignored**, SQLite **125** plus **5**
  integration tests, MCP **35**, and doc tests pass:
  `D:/Temp/dolores-eh09-remote-target-tests.log`. Six new MCP cases exercise real
  public loopback HTTP through the shared prepared/invoked plugin factory and
  bound vault fixtures. The new store case qualifies mixed capacity, stale
  revision, injected save failure, preserved legacy bytes, invalid mixed target,
  reopen, disable and explicit forgetting. Two core cases cover the legacy
  shape and exclusive target/authentication contracts. Clippy across these four
  crates/all targets with warnings denied passes:
  `D:/Temp/dolores-eh09-remote-target-clippy.log`.
- **Receipt-race diagnosis:** a later focused run exposed the existing stdio
  client-response receipt assertion failing after process cleanup. Ordinary
  sequential **30** and concurrent **48** repetitions passed, so the regression
  fixture now deliberately delays callback consumption by 250 ms. The old
  fixture's immediate catalog completion fails that controlled case:
  `D:/Temp/dolores-mcp-receipt-race-pinned-red.log`. The fixture now records the
  exact ping response and sampling refusal before releasing its catalog; missing
  or malformed replies still fail. No production stdio behavior was changed.
  The delayed case passes, the whole MCP suite passes **35**, and **48**
  repetitions with eight concurrent processes pass:
  `D:/Temp/dolores-mcp-receipt-race-fixed.log`. This replaces a scheduling
  assumption with protocol evidence, rather than adding a sleep to assertions.
- **Final checks:** final-source core target checks and MCP **35** pass:
  `D:/Temp/dolores-eh09-remote-target-final-tests.log`. Workspace/all-target
  compilation passes: `D:/Temp/dolores-eh09-remote-target-workspace-check.log`;
  final MCP Clippy with warnings denied passes:
  `D:/Temp/dolores-eh09-remote-target-clippy-final.log`. The normal Windows x64
  build passes with the configured VS18 fallback:
  `D:/Temp/dolores-eh09-remote-target-build-final.log`. Documentation/link and
  staged diff/publication checks pass. No new control or desktop
  interaction is introduced in this slice, so no rendered/native journey is
  claimed. Foreground persistence/vault rollback, OAuth/refresh, searchable
  health controls and normal-model effect/permission receipts still require
  connected qualification. No real service/provider, user settings/history,
  credential or private transcript was used. H21 remains deferred.

# EH09 remote MCP foreground controls - October 25

Scope: remote endpoint setup, optional bearer authentication, inspection/health,
bounded catalog search, secure enablement and a normal-bridge/model tool journey.
EH09.2 remains open for OAuth and its remaining connected recovery gates; this
does not complete H19 or the EH09 umbrella.

- **Controls:** Settings Connections supports Local program or Remote endpoint
  for a new entry, with immutable saved type, a separate masked bearer field and
  the existing shared connection/tool capacities. Local executable/import and
  isolated-launch behavior remains available. Loading makes no network request
  and shows Not checked. Inspect/Recheck reports only the latest inspection in
  this window; it does not claim ongoing health. A changed target requires the
  key again. Search filters reviewed names/descriptions locally, retains hidden
  selections, offers no-match recovery and keeps the selected count in the fixed
  Enable action. Remote approval shows the exact selectable endpoint/arguments
  and explains external effects without claiming a local program launch.
- **Host ownership and saving:** the existing foreground coordinator owns remote
  inspection, poll and Stop. Its five-minute review binds session/root, exact
  target/authentication, previous record and admitted metadata. Inspection never
  saves a key or enables a tool. Enable sends only the token and selected names;
  both transports use the same vault-write/configuration-CAS helper. Failed vault
  writes/saves preserve the old connection and review, delete newly prepared
  keys when possible, and retain the existing cleanup-failure recovery message.
  Successful rotation retires the old key only after the new record is saved.
  Locked Forget retains a disabled record/references for explicit retry.
- **Backend failure evidence:** two new normal-command/poll HTTP cases exercise
  remote inspection without saving, wrong-session refusal, locked-vault and
  injected insert/update failure, same-review retry, saved-key reuse, actual 401
  after expiry, successful replacement and locked Forget/retry. The other case
  covers expiry, changed previous snapshot, cancellation after a confirmed
  discovery dispatch and refusal to publish/save a stopped review. The local
  three review/credential cases still pass after helper extraction.
- **Loopback routing:** the explicit proxy probe first recorded the synthetic
  bearer at the proxy instead of its chosen loopback peer:
  `D:/Temp/dolores-eh09-remote-loopback-proxy-red.log`. The shared HTTP builder now
  bypasses proxies for explicit loopback hosts, including localhost/IP forms;
  other HTTPS targets retain the client's proxy configuration. The passing
  regression verifies zero proxy requests, one target request and the expected
  synthetic Authorization header at the selected peer, without changing process
  environment or contacting an external service.
- **Deadline admission:** a regression exposed that a timer-only async select
  could begin a request whose operation deadline was already expired. Immediate
  cancellation/deadline checks now precede RPC, response/notification dispatch
  and legacy close. The existing in-flight controlled-clock/cancellation cases
  remain intact. The expired-at-entry case passes **200** repetitions with eight
  concurrent processes and verifies no new tool request:
  `D:/Temp/dolores-eh09-remote-deadline-repeat.log`. Inspection shares its original
  30-second deadline with credential resolution and cannot reset it when HTTP
  starts. A running native vault read cannot be forcibly interrupted; its result
  is discarded on Stop and cannot cause subsequent network work.
- **Checks:** full bridge **263 passed / one existing ignored**, plus doc tests:
  `D:/Temp/dolores-eh09-remote-controls-bridge-tests.log`. MCP **36** and doc tests
  pass: `D:/Temp/dolores-eh09-remote-controls-mcp-tests.log`. Final focused bridge
  **5** and both crates' all-target Clippy with warnings denied pass:
  `D:/Temp/dolores-eh09-remote-controls-final-rust.log`. Flutter MCP widgets pass
  **13**, including Light/Dark review, retained search/selection, failed save and
  retry, blank saved-key reuse, changed endpoint, auth failure, Stop and endpoint
  approval: `D:/Temp/dolores-eh09-remote-controls-widgets.log`. Focused Flutter
  analysis passes.
- **Rendered UX:** four **390 × 700** Light/Dark form/error renders under
  `output/eh09-remote-controls/` were inspected. Keys remain masked; the failure
  stays visible at the top and fixed actions remain reachable with Enable's
  selected count. Local review/isolation and existing one-use approval widgets
  also pass. Test navigation now reveals offscreen controls from the list's top
  instead of assuming every new field lies below the current position.
- **Connected normal bridge:**
  `output/native-ui/eh09-remote-controls-20261025-3/mcp-remote-receipt.json` and its
  passing process log qualify the public loopback journey through rendered
  Connections, composer and approval controls. Loading/Enable make no extra HTTP
  requests; inspection performs no tool effect. It preserves a selected tool
  through a no-match filter, saves a native-vault bearer, reopens/reuses it, retains
  the old record on 401 and refuses to forward it to a changed target. Fresh-key
  rotation succeeds; Stop preserves the rotated record. One exact endpoint
  approval produces **one** real MCP tool call, **two** loopback model requests,
  the expected UTF-8 marker and a saved completed tool receipt. Neither public
  credential appears in provider messages. Only its fixture connection is
  disabled afterward. The first run's functional assertions passed but the
  fixture failed teardown on a pending attention timer; explicit AppHost disposal
  and a pump now finish before the receipt/test completes.
- **Final qualification:** the final-source normal Windows x64 build passes
  using the configured VS18 fallback:
  `D:/Temp/dolores-eh09-remote-controls-build-verified.log`. The repeated native
  journey passes against that DLL after the loopback/deadline fixes:
  `D:/Temp/dolores-eh09-remote-controls-native-final.log`. Documentation/link and
  staged diff/publication checks pass.
- **Remaining scope:** OAuth sign-in/refresh and connected changed-catalog /
  malformed-SSE recovery still require qualification before EH09.2 closes.
  EH09.5 retains cleanup after failed setup/drop/Stop and the scheduled/child
  lifecycle gates. Physical pointer/keyboard behavior, third-party services,
  real-model reliability and other platforms are unqualified. No user settings,
  history, real credential or private transcript was used. H21 stays deferred.

# EH09 remote MCP OAuth protocol foundation - October 25

- **Scope:** internal OAuth protocol API in `dolores-tools-mcp/src/remote/oauth/`.
  This does not add an OAuth choice to Settings or close EH09.2. Browser opening,
  loopback callback ownership, secure token storage/rotation, run admission and
  foreground recovery still need their own connected Windows qualification.
- **Basic flow:** public TCP fixtures exercise protected-resource metadata,
  issuer metadata, native dynamic registration, a random state/verifier and S256
  challenge, exact callback validation, code exchange and refresh. The fixture
  independently hashes the verifier received at the token endpoint, checks the
  authorization challenge, resource/client/redirect fields, native application
  type and requested authorization/refresh grants. Rotation uses the new refresh
  token; omission of a replacement retains the previous refresh token.
- **Discovery/binding:** challenged metadata takes precedence; a challenged scope
  can differ from the resource's advertised scopes. Scoped/root resource URLs and
  all three path-bearing OAuth/OIDC metadata locations are exercised in order.
  Resource mismatch, exact issuer mismatch, missing S256 and unsupported public
  client authentication refuse before code exchange. Pre-registration cannot be
  reused across issuers. Hosted client metadata binds exact document identity,
  response/grant type and loopback address/path while permitting port changes;
  only its validator is fixture-qualified, not a real hosted HTTPS document.
- **Callback/failure recovery:** missing/changed state, duplicate parameters,
  malformed percent encoding/UTF-8, missing required issuer and differently
  cased/default-port/slash issuer strings refuse without token HTTP. Error
  callbacks validate issuer first and never expose remote descriptions. Optional
  issuer omission works only when not advertised as required. A controlled clock
  expires the attempt after five minutes. Each exchange consumes the attempt.
- **Tokens and Stop:** revoked refresh reports a fresh-sign-in action; malformed
  tokens, invalid expiry, unsupported token types, oversized refresh values,
  unexpected explicit scopes and token redirects preserve the old token object.
  When resource metadata omits scopes, sign-in omits the scope parameter and
  accepts the server's returned default scopes; this is distinct from silently
  adding authority to an explicit scope request.
  Cancellation before dispatch produces zero requests. Stop after a confirmed
  refresh dispatch returns without replay and retains prior local tokens. The
  server may already have rotated them: preservation is not a guarantee that the
  old refresh remains usable, so the host must offer fresh sign-in.
- **Network/privacy bounds:** one 30-second phase deadline; twelve requests,
  32 KiB per JSON response and 128 KiB cumulative bytes. Controlled in-flight
  timeout, cumulative byte/request exhaustion, malformed/oversized bodies and
  redirect refusal pass without silent fallback/retry. Mixed-case JSON media
  types are accepted. HTTP authorization is restricted to an HTTP loopback MCP
  resource; external resources require HTTPS. Shared transport still bypasses
  proxies for explicit loopback. All authentication header fields are retained
  within a bound, including a Basic challenge preceding Bearer. Invalid,
  oversized or known-secret-bearing fields fail parsing through a nonsecret
  sentinel instead of selecting another field. OAuth failures contain static
  recovery text; token/attempt containers implement neither Debug nor Serialize.
- **Checks:** final MCP **48 passed**, including eleven OAuth cases and the new
  combined-challenge case, plus doc tests. MCP/bridge all-target Clippy passes
  with warnings denied: `D:/Temp/dolores-eh09-oauth-protocol-verified.log`.
  The five existing bridge MCP review/save/reuse/rotation/Stop cases pass:
  `D:/Temp/dolores-eh09-oauth-protocol-final.log`. The final-source normal Windows
  x64 build passes using the configured VS18 fallback:
  `D:/Temp/dolores-eh09-oauth-protocol-build.log`. No launch/render is required
  for this internal layer; no available foreground authentication choice changes.
  Documentation/link, staged diff and publication checks pass.
- **Remaining acceptance:** native browser/callback lifetime, real vault binding,
  atomic rotation/refresh ownership and failed-save recovery, rendered Sign in /
  Stop / expired-login controls and their normal model/tool journey are required
  next. Connected changed-catalog/malformed-SSE recovery and EH09.5 lifecycle
  cleanup remain open. No real credentials, user history or third-party login
  were used. Fixtures do not establish real-service/model reliability. H21 stays
  deferred.
