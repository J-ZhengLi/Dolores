# First-user feature audit

Started 2026-10-08. This is an ongoing audit using native user interaction,
background integration with real configured models, and generated UI renders.
Order follows likely everyday use. Existing reports are references, not fresh
passes. Record the verification mode; background evidence does not establish
physical keyboard, IME, native dialogs, tray or pointer behavior.

## Method and status

**Current checkpoint: 86/129 checked, 43 pending.**

### Native audit resumed — October 12

The user resumed UI auditing. G20 now passes all three cases: the earlier
divergent Pull refusal, a changed HEAD while the remote picker was open, and Stop
after an owned local Fetch transport actually started. The stale basis was refused
before transfer. Stop explained that effects might remain; native Refresh recovered
the controls and History refresh showed the new HEAD. Exact public file bytes,
local/tracking/remote refs were unchanged after Stop, and no owned child process
remained. The temporary local transport delay was removed. Evidence is retained
in `output/native-audit-20261010-01/g20-native-resumed.json`.
**83 checked / 46 pending**. These notes will accompany the next code-change commit.

E04 subsequently passed native response testing. DeepSeek generated a draft from
a completed public code review. One- and two-case arithmetic controls retained
exact prompts/required snippets, baseline/candidate replies and reported usage.
Both comparisons tied, so activation remained disabled. Stop arrived after these
fast live runs completed; neither is counted as an interruption. A separately
labeled local fixture delayed an entered evaluation request for 20 seconds. Native
Stop reported “Skill evaluation stopped. Nothing was saved.” Draft and test inputs
remained editable, and activation stayed disabled. This proves interruption UI,
not live-model reliability. Earlier substantive real-model evaluation remains
separate evidence; M10 still needs native editing and strict-gain promotion.

The empty-test path exposed a usability defect: Evaluate was enabled before test
inputs existed and showed a dense host validation error. The footer now explains
the missing prompt/expected text and disables Evaluate until every case has both.
Native verification passed after the normal build. A failing-first regression
covers empty and whitespace-only inputs, no accidental evaluation and activation
invalidation. All 22 focused skill tests and analysis of `lib`/`test` pass.
**84 checked / 45 pending**. Original-profile verification matches all 49 tables.

I04 then passed end to end through native Send and four Allow once controls with
DeepSeek: open the owned local form, fill Name with `PUBLIC-BROWSER-42`, click
Preview using the current state, and take a local screenshot. Five reported model
calls completed. The DOM receipt and final response retained the exact value;
native View local screenshot showed it in both the input and page result. The
capture could be collapsed without an error, and no app-owned child process
remained after completion. Public evidence is retained in
`output/native-audit-20261010-01/i04-native-receipt.json`. I05's native Stop and
stale-state recovery remain separate. **85 checked / 44 pending**.

This check also found a browser operation mislabeled as “Search” in its receipt.
The label is now “Operation”; the exact reviewed JSON and result remain visible.

I05 also passed natively. Qwen requested the owned slow page, native Allow once
started navigation, and the server's entry marker proved it was in flight before
Stop. Dolores restored the exact draft and no owned child remained. In a separate
DeepSeek run, the local page changed while Fill was waiting for review. The old
state was refused with `actionDispatched: false`; the returned fresh state required
a new Allow once review, then Fill, Click and Screenshot completed. Six model
calls retained both the refusal and recovery. The rebuilt receipt used “Operation”
and explained that no stale action ran. Public receipts are
`i05-native-stop.json` and `i05-native-stale.json` under the same ignored evidence
directory. Original-profile verification still matches all 49 tables.
**86 checked / 43 pending**.

### Previous desktop pause — October 12

Physical Escape stopped computer-use during G20's remote picker. The user then
explicitly chose to keep desktop interaction paused. Native testing will resume
only after the user makes the computer available again. The unobserved action
adds no pass: its local delay marker was absent and the owned temporary transport
override was removed. Original-profile verification still matches all 49 table
digests. **82 checked / 47 pending** remains the current count.

Background widget checks found and repaired two related receipt recovery defects:
native-build details had both a scroll/expansion type collision and shared nested
expansion state; missing browser/selected-window capture errors had the same type
collision. Independent storage keys preserve readable evidence, separate expansion
choices and view-only retries. Reproductions failed before the fixes; 37 focused
checks, all 444 Flutter tests, analysis and a normal Windows build pass. No native item is checked from
this background work. Audit notes accompany the code-change commit.

Native audit resumed October 11 with the user's authorization to continue
through the checklist while the computer is free. A12, C12 and A09 are complete.
User-message Markdown was verified with deterministic widget tests and saved
renders; it adds no native checklist pass. The October 12 resumed settings check
completes P05, P03, P09, M02–M03, S04, S11, I01–I03, E09, G15, G17–G19 and I07: **82/129 checked**.
The isolated public-notebook profile exercises the normal desktop entry point
without copying private chats or changing the original provider configuration.

### October 12 resumed settings interaction

Resetting the computer-use JavaScript session recovered the current Dolores
window instead of the closed reference returned by the old session. A freshly
returned accessibility index still failed. Screenshot-based clicks and physical
key presses worked in the owned public profile; bulk `type_text` did not change
the selected numeric field, so no success was inferred from its return value.

P05 passes end to end in Settings → Models → qwen3.7-flash. A physical edit
changed the stall timeout from 30 to 31. Escape offered Keep editing / Discard /
Save; Keep editing retained 31. The model-detail close button offered the same
choices. Discard closed the editor, and reopening showed 30. A second edit to 31
was saved through the Escape prompt; reopening showed 31. Context/output/reasoning
values remained as before. No model request was made and the original profile
was not changed.

P03 subsequently passed in the same native UI. Qwen's context/output/stall
values changed from 1,000,000 / 1,024 / 31 to 1,000,001 / 1,025 / 32 and were
saved. DeepSeek retained its independent 1,000,000 / 1,024 / 30 values; its
reasoning choice changed from DeepSeek thinking off to Provider default. After
stopping and relaunching only the owned preview through the normal desktop
launcher, both models retained their exact saved values and reasoning choices.
No model request was made. These edited values remain in the disposable audit
profile; the original profile still matches all 49 table digests.

The earlier stop checkpoint is superseded by this recovered native route.
**82 checked / 47 pending**; background passes still do not count as native ones.

E09 passed through Advanced → Diagnostics & storage → Capabilities & source.
The native view showed Dolores 0.1.0, the project workspace, Qwen model/context
origin, run limits, review requirements, tool availability and unknown model
reliability. Matching bundled file-tool source displayed its SHA-256 and lines
1–60 / 342; Next source lines showed 61–120 and First lines restored 1–60.
Cancelling Compare checkout retained the bundled source. Refresh retained the
inventory and source. All inspection stayed local and made no model request.

P07's attach/preview/send and snapshot-retention checks passed natively with a
108-byte public UTF-8 fixture. After the source changed to a different code, the
preview and Qwen's one-call reply retained P07-ORIGINAL-42 and the original Chinese
sentence. Read-only storage inspection verified the exact CRLF bytes and digest.
Bulk text input worked in the native file picker but not the Flutter composer;
physical keys entered the bounded prompt. P07 stays unchecked: its native Remove
control awaits the computer-use skill's action-time deletion confirmation.

G15 now passes its remaining nonrepository navigation. A folder inside an
enclosing Git repository correctly identified that enclosing root. A separate
public folder outside every Git ancestor showed the actionable nonrepository
error but incorrectly labeled it “No commit / detached HEAD.” A regression failed
before the fix. The UI now renders HEAD states only when repository status exists,
and distinguishes an empty repository from detached HEAD. All 42 Source Control
tests and focused analysis passed. The normal Windows build and native retest
passed, including Refresh with no invented HEAD state. Its earlier native binary
comparison completes the item. No Git mutation or model request was needed.

G19 passes against a disposable local bare origin and two public clones.
Native Fetch advanced origin/main while retaining local HEAD and the untracked
file. Pull correctly refused the dirty tree; moving that public file aside and
Refresh allowed a fast-forward to the exact peer commit, with History updating.
Returning the file, native + and Commit created a one-file commit; native Push
advanced only the local bare origin to that exact commit. Tracking remained
main → origin/main. No network service or original repository was involved.
The injected right-Control shortcut did not dispatch; the visible Commit menu
worked. An old Fetch success notice remained beside the dirty-Pull refusal; its
widget regression failed before a fix and all 42 Source Control tests pass after
clearing notices at the start of a new action. The rebuilt native retest passed: successful Fetch followed by divergent Pull
showed only the truthful refusal, with both HEADs and public files retained.
G20 remains open for changed-ref and interrupted-remote reconciliation.

P09 passes native oversized and unsupported attachment rejection plus snapshot
export. A 65,537-byte text file offers smaller-file/split recovery; a four-byte
binary file offers UTF-8 text, PNG or JPEG instead of the old misleading size/OCR
advice. Both preserve the existing draft and sent attachment. Native export
produced one manifest entry and its exact original 108-byte snapshot, despite
later source-file changes. The attachment regression failed before the fix and
passes after it; bridge clippy, focused Flutter analysis, all 42 Source Control
tests and the normal Windows build passed. No new model request was needed.

M02 passes in a fresh native Side chat with qwen3.7-flash. A preceding real
completed public chat automatically saved the fictional Juniper-583 codename;
no memory entry was manually populated. Asking only “Codename?” returned the
exact value in one model call. Reply usage showed zero recent/saved chat turns,
one automatically learned memory, source message 149 and revision 1. Provider
usage was 2,540 input / 201 output / 2,741 total tokens (2,176 cached and 185
reasoning are breakdowns). The Memory switch was enabled through the host only
in this disposable profile; its native switch interaction remains open in M01.

M03 passes native inspection/edit/save of the public automatic fact. Its
exact source was visible; correcting Juniper-583 to Juniper-584 saved revision 2,
marked the earlier source historical and retained revision 1 outside recall.
A later host-level real Qwen extraction from the conflicting original wording
completed with zero saved / one skipped. Native reopening after a normal launch
retained the corrected value and revision, with that attempt visible in activity.
The extraction step is host evidence; editing and reopened presentation are native.

I03 passes native Browser readiness and missing-adapter recovery. An owned
launch-only environment override selected an intentionally absent adapter,
without removing installed software or changing saved settings. Tools showed
Needs setup; Browser showed the actual reason, expandable Node/Edge/Chrome and
pinned adapter guidance, and a truthful Refresh. The normal launcher restored
the original environment afterward. No browser install or model call occurred.

S04 passes with a fresh real Qwen-created weekday report and an enabled
public daily-report skill. Host-level preparation created and ran the task;
the native Scheduled page and in-chat receipt matched task ID, project, model,
skill, timezone and weekday time. Open source chat showed the actual request and
acknowledgement; Open result showed PUBLIC-SCHEDULE-PINNED-42, matching the pinned
skill. The occurrence used one call / 5,269 reported tokens. The task was paused
after completion; no further automatic run is intended in the disposable profile.

S11 passes native Close-to-tray and normal launch reopening with the same owner
PID, retained unsent `z` draft and completed report. Turning background tasks off
and native Close exercised full exit. Independent process inspection verified the
owner had exited before any replacement; a fresh normal launch retained the draft,
result and exactly one succeeded occurrence. The tray context-menu Quit was not
exercised; the ordinary Close exit path was used. No model request or task replay
was made by these lifecycle checks. Background preference returned to off.

I01–I02 pass native inspection of the retained real Qwen web run. Search
showed its exact query, provider, four labeled URLs/snippets and the snippet
limitation. Qwen explained that none was the requested official page, then used
the supplied Rust Book URL; its answer rendered the exact clickable source link.
The expanded primary-page receipt showed title/URL, a partial excerpt ending at
character 8,132 and the untrusted/no-script/login/redirect limitation. The old
“Search: 0” label was incorrect; its regression failed before the repair. The
normal rebuilt native view now says “Read from page start” and “Continue from
character 8132.” All 29 focused tests, including empty-search recovery and later
page offsets, pass. Tool approvals and the three real Qwen calls (22,318 reported
tokens) were host-level preparation; this native check sent no additional request.

I07 passes a fresh native qwen3.7-flash echo request: visible Send, exact
server/tool/revision/JSON Run once review, one real execution and a matching final
answer. Two calls / 14,217 reported tokens; the draft and harmless connection were
host-prepared. A temporarily missing owned script then produced a retained error
receipt with reinspection recovery, without calling the external tool. Two calls
were used for that exact request; a preceding underspecified request correctly
asked for the text to echo in one call. Native inspection failed truthfully while
the script was missing, then recovered its two-tool catalog after exact restoration.
The server log proves only the original echo executed across these checks.
Expanding MCP metadata initially crashed into a blank panel: its selectable text
shared the card's boolean expansion storage with a numeric scroll offset. A
regression failed with the exact type error; a separate metadata storage key fixes
it. All 26 focused tests, analysis, normal Windows build and both saved/fresh native
receipt checks passed. Native connection activation remains open in I06.

G17–G18 pass their remaining native conflict/recovery routes on two separate public
single-file repositories. Direct Apply stash retained the exact stash object and
both conflicting texts; the message instructed Refresh/resolution and warned against
reapplying. Refresh exposed the unmerged file. Direct Revert of an older commit
retained conflict evidence and HEAD; its in-app diff showed both versions and an
actionable resolution hint. Abort restored the committed bytes; Refresh diff then
showed no changes. A clean head Revert created the expected reverse commit. A later
dirty Revert refused before changing files, retaining the unrelated CRLF file and
HEAD exactly. Earlier successful large-stash/apply and selected-discard evidence
completes G17. No model call or external remote was involved.

### October 11 resumed editor interaction

Fresh native pointer selection works despite the helper's stale accessibility
indexes. F04 passes physical Ctrl+P/F/H/G: relative file `8` opened, line 20
received the caret, Find selected matching text and Replace changed one match.
Undo from the other split pane restored the replacement and both clean markers.
F06 now passes shared undo, complementing its earlier shared editing, independent
cursor/scroll and compact-layout checks. A short input-settle delay was necessary
before capture; the immediate screenshot had falsely suggested failed undo.
F07 Keep editing and Save-close pass; Discard-close subsequently passed. These operations
used only the isolated public notebook, no model requests or original-profile edits.

G07 now passes physical Ctrl+Enter and Changes → Commit. Cancel retained the
message and staged file; a fresh review committed only tracked fixture `8`
(`d192973`, message `audit`), cleared the draft/index and preserved six other
working changes. G16 branch creation, dirty-switch refusal and a reviewed clean
switch to `codex/a` pass. G17 exposed a large-stash review refusal at 256 KiB. Its repair
uses exact pinned file/count metadata, with full stash content retained. The new
regression failed before the repair and passes afterwards. The normal build and
native original-case retest pass: review shows 6,000 added/removed lines; stash
preserves unrelated untracked files, and Apply restores the exact SHA-256 while
retaining the stash. Selected discard subsequently restored the saved fixture;
its escaped patch display exposed a formatting bug. Conflict recovery remains pending.

### October 11 direct Git actions

At the user's request, Git actions now run when selected, without a second review
dialog. Branch, stash and remote pickers still collect required values. Internal
revision validation and unsaved-editor guards remain. Focused UI tests cover direct
commit/discard/hunks/stash/branch/revert/remote actions, stale refusal and retry,
retained commit drafts, duplicate clicks and leaving the initiating view.
The native observations above exercised the earlier review UI; the changed direct
flow has widget verification and still needs a fresh native interaction check.
Computer-use stopped on physical Escape. No additional native items are checked
from background tests. Checklist remains **66/129 checked**.

### October 11 background coverage and native queue

October 12 drafted-skill continuation: real Qwen drafting/comparison/promotion
passed the fixed format gate, but a fresh chat omitted the saved instructions
because “review” did not match the description's “reviews.” The failing core
regression now passes with simple singular/plural keyword matching; unrelated
substrings remain excluded. All 94 core checks, Clippy, normal build and the same
rebuilt-bridge fresh-chat request pass, including exact skill provenance.
Content inspection separately found an incorrect Qwen source review; its format
receipt was not semantic qualification and that disposable skill was disabled.
DeepSeek also misread the terse source request, then completed the same workflow
with explicit before/after Python source and unchanged comparison criteria.
Separate-process reopen and a new empty-list review retained evaluation/provenance
and correctly explained the change, without claiming tests ran. Brevity remained
inconsistent. Eight Qwen calls / 10,791 tokens and eight DeepSeek calls / 10,917
tokens; all original table digests match. Receipts: `skill-draft-qwen-20261012-01`,
`skill-draft-deepseek-20261012-01` and `skill-draft-deepseek-20261012-02`.
Native draft/edit/test/promotion controls remain pending (M10); count stays 66/129.

October 12: a fresh real Qwen3.7 Flash comparison completed all four fixed
`config-check-v1` cases: both baselines failed, both candidates passed, and
automatic activation succeeded. Manual baseline restoration also succeeded;
the audit helper's obsolete `restored` assertion incorrectly reported failure
because the product records `quarantined`. The corrected helper checks the exact
baseline, retained candidate and rollback reference. Fresh-process reopen,
stale/duplicate restore refusal and unchanged saved state passed without another
model request. Foreground: five calls / 34,216 tokens; trials: sixteen calls /
16,585 tokens. Evidence: `learning-qwen-20261012-01` under the background audit
directory. Native controls/manual-edit conflicts and real-model automatic
regression restoration remain pending. Count stays **66/129 checked**.

The October 12 native retry recovered the current isolated app window after the
helper first returned a closed window reference. Settings → Models → Qwen
details and response settings were readable. Editing then failed with
`element 13476 is not available in cached app state for dolores_flutter.exe`.
After refreshing the screenshot, a pointer retry did not establish focus in the
intended output field. No text was entered or preference saved. Native input
stopped; P03/P05 remain unchecked. This is a helper/observation failure, not a
verified settings defect. No alternate Windows automation was used.

The next resume again failed twice during desktop activation with `foreground
window did not report a process id`. No UI input followed. A background probe
found that large Commit still generated an unused full review patch and hit the
512 KiB process-output cap. The regression failed before the repair; pinning
paths/state with a bounded summary removes this blocker without increasing
limits. The rebuilt bridge then exposed the same Discard limit; a separate large
Revert regression also failed. Both unused captures are removed while retaining
their state and conflict checks. Normal-release integration and focused test
results and remaining native checks are tracked in ACCEPTANCE. This adds
background coverage, not native checkmarks.
The normal-bridge conflict probe also exposed a missing recovery explanation:
stash/conflict bytes were retained, but the error only said the index was not
unstashed. Confirmed stash/revert conflicts now explain retention and
Refresh/resolution or Abort; the strengthened regression failed before the change.
Final verification passes all 25 Git tests, bridge Clippy, the normal Windows
build and five rebuilt-bridge cases: large Commit/Discard/Revert, retained
conflicting stash and revert conflict/Abort preserving HEAD and saved bytes.
Receipt: `output/git-large-actions-1a522142d7ee488bbc0c0a7b9406cc9e/result.json`.
Native count remains 66 checked / 63 pending.

The updated October 12 queue has **82 checked / 47 pending**. Every pending
item below retains a native interaction requirement; available background paths
have been exercised, with model reliability gaps recorded separately. The
earlier helper failure was recovered as described above. No alternate Windows
automation was used. Continue with the pending native controls in the isolated profile; terminal
input and restricted permission/install steps may require the user's direct
interaction. Recheck the changed direct Git flow as well as the pending conflict
and remote cases. Background evidence adds no native checkmarks.
All evidence directories in this checkpoint are ignored under
`output/background-audit-20261011-remaining/`. Original-profile verification
still matches all 49 table digests. No public Git remote was changed or pushed.

| Open IDs | Background coverage completed | Remaining interaction |
| --- | --- | --- |
| C16–C17, A10–A11, A13 (5) | Disposable deletion retention, question/custom-answer continuation, retained receipts, output/task limits, explicit continuation and restart checkpoints | Delete confirmation; failed-send Retry; Continue/interrupt/log-capture controls |
| T01–T02, T04–T08 (7) | Three project/HOME shells, Unicode, resize, real interrupt/prompt recovery, bounded polling/flood cleanup, stopped snapshots and scoped output sharing | Physical typing/selection, tabs/splits, copy-vs-interrupt, close/reopen and attachment controls |
| S10, S12 (2) | Save/reopen scheduling corpus: pinned/unpinned skills, failed/Stop/overlap/stale/offline recovery; real Chinese creation, Qwen/DeepSeek pinned reports and deletion retaining chats/results/files | Skill receipt/result navigation, explicit retry/delete, tray/Quit/owner interactions |
| P04, P06–P08, P12–P14 (7) | Scoped settings, permission isolation/revoke, snapshot attachments, image capability refusal, export/caps and compaction fixtures | Scoped settings and attachment dialogs; native/manual privacy/access/experimental preference controls |
| M01, M04–M10 (8) | Real fact/correction/project isolation and image recall; useful/source-only/image memory, suggestions, instructions and skill save/restore fixtures; real draft/edit/test/promotion/reopen/provenance with format and content accuracy distinguished | Memory sources/activity/switch/correction/forget, instruction/skill reviews, draft test/promotion and rollback controls |
| I06, I10 (2) | Exact reviewed two-tool MCP invocation; real scoped delegation with retained reports | MCP activation controls and child progress |
| I08–I09 (2) | Deterministic selected-window scope/state/Stop/revoke tests | Actual window sharing/inspection/control requires a working native helper |
| E01–E03, E05–E08, E10–E12 (10) | Real companion invitations and cooldown/cap fixtures; strict comparisons/drafts; real automatic skill activation/manual restore/reopen; real mod generation, six checks, activation/restore; health/restart/source/capability refusal; isolated native-test decline/stale/non-improvement and diagnostic/export fixtures | Message actions, learning/mod/diagnostic reviews and native repair/install/restore controls |

Live safety outcomes remain distinct from positive reliability. The earlier Qwen
learning trial was incomplete; DeepSeek tied the baseline. Neither activated.
The October 12 Qwen run above adds narrow automatic strict-gain activation and
manual restore evidence; automatic regression restore remains unqualified.
Qwen's malformed Wasm draft was retained without activation (621 reported
tokens); explicit DeepSeek fallback passed six checks and activation/restore
(214 tokens). A fresh real native repair-to-install scenario remains an additional
qualification gap. Do not force weaker baselines or relax gates
to obtain checkmarks. Native installation/restoration needs its concrete review.

Real Qwen MCP used 3 calls / 19,750 reported tokens and two exact approvals;
both receipts and the final answer matched. Real web search plus primary-page
retrieval used 3 calls / 22,318 tokens: four search results, then the supplied
Rust Book URL, a bounded excerpt and an exact citation. Search snippets were
distinguished from the retrieved primary page. Evidence: `mcp-qwen-live`,
`web-qwen-live`, `mod-qwen-live`, `mod-deepseek-live`, `learning-qwen-live`
and `learning-deepseek-live`. Native review/display remains pending.

The real scheduling probe exposed a reused historical setup instruction. The host
now explicitly executes the saved work in the provider handoff, while retaining
the exact source request in task/conversation/history. The wire regression failed
before repair; seven scheduling unit checks, all 469 workspace checks (two
ignored), Clippy, normal build and the 14-request save/reopen corpus pass. The
same existing Qwen task now returns its pinned report (5,246 execution tokens)
without a new setup exchange or tool review. Separate-process restart retains it
without replay. Both disposable live schedules were then deleted, retaining all
six source/result conversations and project files. Evidence: `schedule-qwen-live`,
`schedule-deepseek-live`, `scheduling-handoff-red` and `scheduling-handoff-fixed`.
These are background checks; S04/S10–S12 still need their native routes.

The app browser adapter also ran the real public local form without the Windows
helper. Qwen filled/clicked with fresh states and captured the correct value, then
an invalid close proposal reached its explicit step limit (46,453 tokens).
Receipts/screenshot were retained and owned workers closed. DeepSeek fallback
completed the form, capture and truthful answer within the same limits (five
calls, 37,100 tokens), leaving zero workers. Both saved screenshots were inspected
with view_image. Evidence: `browser-qwen-live` and `browser-deepseek-live`.
Native app approval/capture controls remain pending; no checkmarks were added.

Latest background continuation: desktop control failed in the folder chooser
(`element 15888 is not available in cached app state`); refreshed selection and
focus recovery still targeted Search instead of Folder. Native input stopped.
G15's public binary comparison passed; nonrepository navigation is pending.

October 11 background MCP retest: two reviewed tools exposed a real start
refusal at the old 18-registration limit. Capacity now matches the supported
20-tool catalog (17 host, one browser, two external); execution budgets and
external slots stay unchanged. All 91 core checks, Clippy, normal build and
both MCP save/restore fixtures pass, including crash/Stop and sibling recovery.
Exact catalog and duplicate checks remain. Native I06-I07 are still pending.

October 11 wider memory probe: useful facts, corrections, Memory Off and project
isolation passed with real Qwen. A caption missing the Image: display prefix
exposed a second host formatting defect; the host now owns that prefix, retaining
all evidence bounds. All 468 workspace Rust checks (two ignored), Clippy and
normal build pass. One rebuilt Qwen image capture/recall passed; the independent
second failed oversized uncertainty and retained chat/image. Explicit DeepSeek
fallback captured and recalled successfully. Original data stayed unchanged.
The runner now exits nonzero for a failed receipt. Native Memory controls remain
pending; the count stays 60/129. Detailed usage and receipt paths are in ACCEPTANCE.

October 11 background follow-up: explicit thinking off still produced a stray
leading `</think>`, breaking otherwise valid image-caption JSON. A failing
provider regression now passes for all three response paths, fragmented markers,
Unicode and preserved literal markers. All 465 workspace Rust checks (two ignored),
Clippy and the normal build pass. Qwen then described the public image, saved one
image memory and recalled it in a fresh chat within the existing bounds: 5,231
foreground and 265 maintenance tokens. Original profile digests match. Native
Memory-page interaction remains pending; no checkmark is added. Evidence:
`output/background-audit-20261011-remaining/image-qwen-thinking-off-fixed/`.

Earlier background continuation: desktop control failed in the folder chooser
(`element 15888 is not available in cached app state`); refreshed selection and
focus recovery still targeted Search instead of Folder. Native input stopped.
G15's public binary comparison passed; nonrepository navigation is pending.
All eight normal-bundle regression pairs, Source Control and scheduled-task
save/reopen fixtures pass. Companionship now preserves its chosen model's saved
reasoning and tighter limits. The failing regression, all 172 bridge checks,
Clippy, four runner checks and normal build pass. Two live Qwen thinking-Off
invitations pass with 258 reported tokens and no tools; no user setting changed.
Native eligibility/delivery remains pending, so E01 gains no checkmark. Receipts
are ignored under `output/background-audit-20261011-remaining/`.

Latest October 11: G14 passes large-file Inline/Side by side, Next/Previous and
stale-page recovery. A controlled public footer first invalidated the page while
leaving its paired text readable. Refresh diff incorrectly retried the cached
revision; it now reads current status before replacing the comparison. The native
retest cleared the error, displayed 6,006 logical rows and continued to paired
row 252. The public fixture was restored byte-for-byte; all 49 original-profile
table digests match. All 26 Source Control checks, analysis and the normal Windows
build pass. Failed status reads and a selection change during refresh preserve
the previous tab. No model request was needed.

Latest October 11 retry: the owned normal build reopened and native selection
recovered. G13's pending retest now passes: history.txt tabs show distinct
1f5bca87 and 8f683959 labels with exact 060 → 061 and 059 → 060 contents.
Changing the second to Inline leaves the first Side by side; returning retains
its exact revision and native drag selection highlights the new text. Combined
with the earlier three-page history coverage, this completes G13. No model call
or repository mutation was needed. New native observations are ignored under
`output/native-audit-20261010-01/observations-20261011-retry.jsonl`.

October 11 retry: fresh desktop selection and the permitted retry both failed
with `foreground window did not report a process id`. Native input remains
pending. A background Memory Off probe reproduced Qwen's unavailable
`list_folder` call in two of three fresh chats; DeepSeek answered without tools.
Unconditional file/coding instructions caused a catalogue mismatch. Capability-
specific preparation passes the core/bridge regressions (259 checks), all-target
Clippy and the normal Windows build. Three fresh Qwen retests answered without
tools or invented facts; wire guidance, Memory Off and all 49 original table
digests agree (7,502 reported tokens). No additional native item is checked.
Public evidence: `output/memory-off-20261011-01/` and `-02/`. The next normal-bundle
delegation fixture initially missed child approval because it relied on an
obsolete ID prefix. Recognizing the two owned create operations and requiring an
actual Stop/revocation corrected the fixture. All normal-bundle concurrency,
shared-budget, Stop/revocation, scope refusal and restart cases pass without a
cancellation change. I10 remains unchecked pending native coverage.

I10's read-only provider-boundary check separately reproduced creation guidance
without a create tool. The conditional guidance fix passes all five child checks,
all-target Clippy and the normal Windows build. Qwen then completed two scoped
read-only delegates; the parent independently read both public files and confirmed
their exact markers. Five reviews, unchanged bytes and a fresh-process reopen
agree. Eight provider calls including children reported 37,276 tokens, with no
DeepSeek fallback or original-profile change. Public evidence:
`output/delegation-live-20261011-01/`. This adds background qualification only;
native progress/approval controls remain pending.

G14's background render found that a large replacement put removals and additions
on separate raw pages, leaving one comparison side blank. The original real-page
render failed, then passed after aligned side paging. All 171 bridge tests (one
vault test ignored), 23 Source Control checks, analysis, Clippy and the normal
Windows build pass. The packaged bridge returned all 6,000 replacement pairs in
order over 24 pages, with at most 63,488 source-text bytes per page. Long UTF-8,
stale reads, failed layout changes, per-tab navigation and complete-review guards
also pass. The real-page render shows aligned red/green text; no native input or
model request was used. Original data remains unchanged. Public evidence is in
`output/git-large-render-20261011-01/` and `-02/`. G14 remains unchecked until its
native navigation/recovery test; the count remains 58/129.

Background follow-up repair (October 11): A07's missing historical receipt was
reproduced at save/restart and corrected in next-request context loading. Recent
text receipts are bounded to eight / 16 KiB, labeled historical and untrusted,
counted in preview/send budgeting, and never replayed as commands or permissions.
Transcript and summary provenance stay unchanged. Store/bridge suites pass 242
checks with one vault check ignored; normal build and all-target Clippy pass.
The normal bridge's two-process keyless fixture and qwen3.7-flash both recalled
actual public exit-0/exit-7 receipts without repeating commands. Two live
follow-up calls reported 13,010 tokens; the original 49 table digests match.
The seed provider was a loopback fixture; the optional live checks exercised
only the follow-up. Reports remain in ignored `output/history-evidence-20261011-*`.
This repairs the missing-context finding but adds no native pass. The earlier
historical conversation still needs a native retest once desktop automation
recovers. A separate source-order cleanup corrected the pre-existing all-target
Clippy failure in scheduling; all six scheduling checks pass.

G13 native paging reached the final page of 62 public commits. Exact diffs for
revisions 061, 030 and 001 opened as three tabs; returning to the first retained
060 → 061. Side by side showed distinct red/green revisions and native drag
selected the right-side text. The tabs initially all read `history.txt · commit`.
They now show the short commit ID. A regression failed on the ambiguous labels,
then all 21 Source Control tests, changed-file analysis and the normal Windows
build passed. Native retesting of the new labels remains pending: after restart,
desktop activation twice failed with `foreground window did not report a process
id`, including a fresh window lookup. No alternate native automation was used.
G13 remains unchecked until that retest. Background preparation of G14's 744 KB,
6,000-line public diff adds no native pass. Original-profile verification still
matches all 49 table digests; no model requests were needed. Evidence remains in
the ignored native observation log and `g13-fixture.json` / `g14-fixture.json`.

C12 passed native message and conversation paging with 61 public conversations,
140 messages and an exact retained draft. A Latest jump initially hid the final
reply; compact/wide regressions reproduced the changing scroll-height estimate.
The correction preserves follow intent during app positioning and still lets
readers scroll away. All 42 selected widget checks and the normal Windows build
pass. The rebuilt native Latest view reaches turn 70; catalog Older/Newer retains
the scrolled reading position and selected conversation. Public storage and all
49 original profile table digests remain unchanged. Evidence:
`observations-20261011.jsonl` and `c12-storage-verification.json` under the same
ignored audit directory.

A09 passed saved Activity/Earlier tasks inspection, per-call provider usage,
applied change diff and effective settings origins. Two labels now distinguish
the configured output limit and use the current Conversation page name. All 14
selected widget checks, changed-file analysis, normal Windows build and native
wording retest passed; inspection sent no model request. Evidence stays in
`observations-20261011.jsonl`. The original 49 table digests remain unchanged.

The new question card passed a real qwen3.7-flash desktop round trip: no initial
selection, choice selection, Back/Next retention, custom date overriding a choice,
and automatic continuation acknowledging the exact date and language. Failed-send
recovery currently has widget/host coverage; that native failure remains open.
The audit reproduced C10's keyboard failure and a dropped first drag movement.
Both fixes passed focused regressions and the final normal Windows native retest:
fast widening, drag-to-hide, title-bar restore, Left/Right and Home retained the
conversation and unsent draft. Evidence stays under ignored
`output/native-audit-20261010-01/`.

A12 passed a real qwen3.7-flash native ranged-read/edit/verify cycle on a public
92,848-byte file. The host rejected three incorrect exact matches and two missing
snapshots without changing the file. Appending a public footer while the reviewed
edit waited caused Apply to refuse the stale change. The model read again,
prepared a fresh diff and completed the edit after approval. Byte checks proved
only `audit_answer = 41` became `audit_answer = 42`, preserving the UTF-8 BOM,
CRLF, Chinese text, concurrent footer and five other project files. The final
ranged read verified the result. The run needed 13 model calls and 12 tool actions;
reported usage sums to 112,930 input and 2,817 output tokens. This qualifies safe
completion and recovery, not efficient first-attempt editing.

The same audit found that chat silently displayed only its first four tool
receipts, hiding recovery and success. It now shows the recent four with a control
to reveal all actions. Failed preparation uses a neutral tool heading while
retaining the actual reason. Three new widget checks cover compact light/dark,
earlier failures, collapse and growing live lists; all 38 selected widget checks
and changed-file analysis pass. The normal Windows build and native reopen retest
passed: stale refusal, successful diff and verification remained inspectable, and
Show all / Show recent revealed earlier rejected attempts without losing results.
The two existing ranged-file Rust checks also pass. All 49 original profile table
digests remain unchanged. Public receipts, byte proofs and native observations
stay in the same ignored directory (`a12-run-receipt.json`,
`a12-byte-verification.json`, `a12-public-events.json`, `observations.jsonl`).

While computer-use is paused, the rebuilt bundle passed all eight selected
background regression pairs across the retained first run and a fresh skill-draft
retest. The skill fixture's obsolete two-tool side-chat assertion was corrected to
the exact three-tool catalogue; drafting/evaluation must still expose no tools.
This adds no native checkmarks. Reports: `regressions/report.json` and
`skill-drafts-retest/report.json` beneath the same ignored evidence directory.

The fresh qwen3.7-flash memory probe saved and recalled a useful public fact,
applied its correction and kept two projects' facts separate. Its Memory Off
baseline instead failed with an unavailable-tool proposal; the supplied blue-square
image was described correctly in the foreground but image indexing failed within
its limits, so later recall failed. Ten foreground turns reported 44,260 tokens;
maintenance usage was unavailable. The original 49 table digests stayed unchanged.
These failures remain open in `memory-qwen/receipt.json`; they do not invalidate
the separately retained earlier successful baseline.

The companionship live helper now requires an explicit enabled model and exits
unsuccessfully when either public case, child execution or profile preservation
fails. Three credential-free runner tests cover those outcomes and refusal before
credential access for an invalid model or existing receipt. Its fresh qwen3.7-flash
run failed both bounded, tool-free invitation cases; `companion-qwen.json` now
truthfully records that failure. Generation failure versus provider/output/time
limits was investigated with a public-only forwarding diagnostic. One Qwen
invitation then completed in 15.274 seconds, while the memory invitation was still
reasoning at its 20-second cancellation with no answer. The successful request
sent `max_tokens: 256` but reported 1,202 output tokens, including 1,183 reasoning
tokens; that allowance did not bound this provider's total output. The direct
DeepSeek fallback also failed both cases. Image-only tracing reproduced the
10-second image-caption cancellation after 1,565 reasoning characters and no
caption, with the foreground blue-square description retained. The app's limits
were not changed. These timing/provider-contract gaps remain open; partial or
diagnostic success does not qualify native delivery. Evidence:
`companion-deepseek.json`, `companion-qwen-traced.json`,
`companion-transport.json` and `image-qwen-caption-only/receipt.json`.
No native companionship item is checked.

Latest checkpoint (2026-10-09): native testing resumed after the user made the
computer available. **50/128 checked at that checkpoint**. C06, C07, P10, P11 and S08 gained passes
on the isolated normal-entry candidate at `32fdbe6`. Keyboard coverage uses
Right-Control; Left-Control injection remains inconsistent and unqualified.
C10's keyboard resize and P13's native access changes remain partial.

S09's earlier clock-triggered execution remains qualified separately: October 8
at 21:15 Asia/Shanghai, two public-file approvals and a correct retained README
summary (DeepSeek, two calls, 5,183/5,385 tokens). S08's earlier false draft-restored
message is now repaired and natively retested: Stop cancels the pending read,
shows the correct Scheduled recovery route and leaves an empty composer.

Act as an experienced developer discovering Dolores through its visible controls.
Use disposable public projects and an isolated configuration-only profile; keep
personal conversations, working projects and provider settings unchanged.
Use the configured `qwen3.7-flash` first, then DeepSeek V4.1 Flash when a task
demonstrates a model limitation (updated October 9 at the user's request).
Earlier results retain the model actually used; this change does not retest them.
Bound each request and record reported usage where available. Do not publish keys,
private transcripts, endpoints or personal paths. Never claim an untested feature works.

`[ ]` means pending; `[x]` means the listed user flow passed. A failure remains
unchecked until repaired and retested. Record app defects separately from model
limitations, unavailable prerequisites and automation restrictions. Backend probes
can diagnose a problem but cannot establish a native UX pass. Destructive tests use
disposable data; remote Git tests use a local bare repository. No public push,
publication or unrelated system change belongs to this audit.

Each result records: checklist ID, model/build, visible route, action, expected vs
observed result, recovery, evidence, fix/retest if needed, and remaining limitation.
Each completed fix is a scoped commit with focused regression coverage. This file
tracks the audit; product design stays in the feature specifications.

### Background checkpoint — October 9

At the background stop point, **45/128 were checked**. These results remain
background qualification; the later native checkpoint below records fresh
interaction coverage.

| IDs | New background evidence | Native check still needed |
| --- | --- | --- |
| F03/F07–F10 | BOM/CRLF save, project isolation, stale save/compare/rebase, checkpoint failure warnings and fresh-process dirty-buffer recovery passed | Physical editing, dirty-close choices, Save as/rename and conflict controls |
| G11/G13/G14/G17/G19 | Dirty-buffer guard, history paging, 17 MiB/small-edit and 60,000-row paged diff, stale-page refresh, selected stash/discard and local bare-remote push/fetch passed | Visible controls, aligned navigation, conflict/remote recovery; Pull remains separate |
| P07–P09 | Text/image snapshot, image wire/rejection, tamper refusal, export and restart passed; Qwen returned the captured text identifier after its disk source changed (one call, 2,711 tokens), then reopened unchanged | Native attachment chooser, preview/remove/send and image-model case |
| P04/P10/P11/M07–M09 | Corrected preview omission of scheduling tools/response limits and duplicate guidance; exact wire and token snapshot, summary/instruction/skill review, stale sources, rollback and reopen passed | Context inspector, summary controls and native skill/instruction interactions |

The new side-chat regression failed before the context fix. All 255 core/bridge
checks, two final recall checks and 35 frontend checks pass. The final isolated
normal-entry build and eight fresh summary/instruction/skill/automatic-memory
save/reopen stages passed. Skill fixtures now request a relevant skill explicitly
and verify that irrelevant greetings include only its catalogue entry. Initial
failed logs remain; no equality assertion was relaxed to hide the context bug.
The public attachment probe's first preflight failed on its own LF/CRLF expectation
before any request; the fresh fixture uses explicit newlines and passed with Qwen.

The planned keyboard/IME, copy, context/summary and Scheduled Stop batch has now
run on the isolated candidate. The original profile remains untouched. Native
permission selection and the original task retry remain separate open checks.
Evidence: `developer-batch-01/report.json`, `context-retest-01/report.json`,
`context-candidate-build.json` and `attachment-live-02/receipt.json` under
`output/background-audit-20261009-01/`. [Acceptance](../ACCEPTANCE.md) records limits.

The urgent cross-project task blocker is repaired in an isolated normal-entry
candidate: agent/skill guidance explains the existing command route for outside
evidence. Chat access now saves Review / Auto edit / Full access immediately,
without a duration or acknowledgement form. Seven widget checks and the native
host permission fixture pass, including stale-save recovery, revocation and
other-chat isolation; saved renders were inspected. No desktop input was used.
A public report task failed with `qwen3.7-flash` after repeated invalid command
arguments (eight calls, 53,284 tokens). DeepSeek completed the same case and
verified its output (five calls, 36,118 tokens); source evidence was unchanged.
Full access survived process reopen without expiry on both isolated profiles.
The current running app remains untouched. Native mode selection and the
original task retry are pending; these results do not add native checkmarks.
Details and limitations are recorded in [Acceptance](../ACCEPTANCE.md).

No native input or window switching was used. The existing normal Windows bundle
passed all eight regression pairs (16 save/restore stages). Four fixtures needed
updates for current behavior: explicit versus automatic continuation, per-call
usage metadata, foreground context refusal before memory extraction, and
scheduling tools on human turns while skill maintenance remains tool-free.
The context refusal retains the draft, makes no HTTP request and saves no turn;
an explicit retry after increasing capacity saves the reply and its memory.

| Area | Background result | Remaining native check |
| --- | --- | --- |
| Scheduling recovery, S08/S10 | Synthetic save/reopen passed: denied/offline execution, explicit retry, overlap refusal, changed/disabled skill, interruption without automatic replay; 14 fixture requests | Final Stop message and offline/retry controls |
| Memory, M02/M06 | Earlier Qwen3.5-2B/DeepSeek probe passed fresh-chat recall, explicit correction and project isolation; shared synthetic image captured and recalled | Settings, source inspection, correction/forget controls and stale-source UX |
| New routine model | `qwen3.7-flash`: Memory Off reports no known codename; automatic capture saves one supplied fact; fresh chat recalls it exactly; three settled replies | Native model-picker/input interaction |

The new-model check used 7,478 provider-reported tokens across three calls
(7,163 input, 315 output; reported reasoning is included in output). The earlier
ten-turn memory probe used 42,610 foreground tokens, plus 421 image-maintenance
tokens. Its initial report missed agent usage; totals were recovered from the
retained public fixture metadata without new requests. Some earlier DeepSeek
answers hit the probe's 512-token cap or misleadingly asked for manual memory
updates even though automatic capture succeeded. These are bounded case results,
not general reliability claims. The existing checklist boxes stay unchanged
where a native UX check remains pending.

Local evidence: `output/background-audit-20261009-01/`, with the completed runner
report in `regressions-complete/report.json`, live model receipt in
`qwen37-live-02/receipt.json`, earlier memory receipt and usage correction in
`memory-live/`, and scheduling save/reopen fixtures in `scheduling/`.

### Native checkpoint — October 9

The candidate uses the existing disposable audit profile and public fixtures.
The profile's default was changed to configured `qwen3.7-flash` with 512 output
tokens and a 30-second request limit; automatic memory stayed Off. The original
profile, credentials and conversations were not copied into audit artifacts.

| IDs | Fresh native result | Limitation |
| --- | --- | --- |
| C06 | Shift+Enter inserted a line; Enter inside fenced code inserted a line; Right-Control+Enter sent from code; Enter in prose sent. Chinese IME selection committed `啊`, and IME confirmation did not prematurely send. Clipboard editing retained Chinese/English and fences. | Left-Control injection and literal `type_text` did not reliably reach Flutter. Physical Left-Control is unqualified; menu paste and Right-Control worked after explicitly refocusing the editor. |
| C07 | Progress, retained Thinking and final Markdown/code rendered. Expanding Thinking retained its contents. Copy code yielded `print("AUDIT-93")`; Copy message yielded the complete sentence and fenced Markdown, both verified by pasting into a scratch editor. | This bounded reply does not establish long-stream or interruption behavior. |
| P10 | Context ring distinguished provider-reported usage from the next-message estimate. Inspector showed 2,561 / 131,072 estimated tokens and retained-history wording. Inspection left the existing 84 messages and one current-chat run unchanged. | Inspector Escape dismissal was not established; its close control worked. |
| P11 | Reviewed the exact user/assistant turn, generated a Qwen draft, corrected its text and saved a one-turn summary. A later Qwen request returned the saved reference `AUDIT-93`; the original messages remained visible. | Summary generation usage was not retained in the inspected metadata. This native summary has not received a fresh-process reopen check; earlier background summary reopen passed separately. |
| S08 | Run now retained the schedule's pinned DeepSeek model, reached approval for reading public README, then Stop cancelled before approval. Scheduled showed Stopped; its result said “Scheduled task stopped. Open Scheduled to run it again.” with an empty composer. The saved occurrence is cancelled with no error. October 8 native Pause/Resume/Skip coverage completes this item. | No read approval or deletion was performed. Cancelled-run usage is unavailable in retained turn metadata. |
| C10, partial | Title-bar toggle, drag widening and drag-to-hide retained the conversation. | Keyboard resize remains pending; clicking the collapsed boundary did not establish keyboard focus. |
| P13, partial | Inspected Review / Auto edit / Full access menu without duration, acknowledgement or Save controls. | The computer-use skill excludes native security-setting changes, so no mode was selected through automation. Prior host/widget persistence and revocation checks remain separate evidence. |

Four Qwen foreground calls reported **10,059 tokens**: 9,586 input and 473 output
(reasoning included in output). One additional Qwen summary-generation request
and the stopped pinned DeepSeek run have unavailable usage; they are not counted
as zero. Two keyboard-submit prompts included a text fence, and Qwen returned
`OK.` plus that fence; no exact `OK.`-only claim is made.

No confirmed new application defect was found in this batch; no runtime patch
or rebuild was needed. Saved screenshots, accessibility text, observations and
usage receipt are under ignored `output/native-audit-20261009-02/`.
Next coverage: pending editor physical input and dirty-buffer recovery, terminal
input/tab controls, and attachment chooser flows on disposable fixtures.

## 1. Everyday chat and navigation

- [x] C01 Discover Home, Scheduled, Folders, Source Control, Terminal and bottom Settings from the compact rail.
- [x] C02 Create a temporary chat; first send creates a working folder and a visible real-model reply.
- [x] C03 Open a project folder, create a project chat and discover its working-folder identity.
- [x] C04 Create a side chat; converse without project/file tools.
- [x] C05 Select Qwen, change to DeepSeek and back; retain history and an unsent draft.
- [x] C06 Compose multiline text and fenced code; Enter/Shift+Enter/Ctrl+Enter behave as presented.
- [x] C07 Read streaming progress/Thinking and final Markdown/code; copy a message and code block accurately.
- [x] C08 Stop a real response; retained progress and subsequent new request remain usable.
- [x] C09 Navigate away and back with an unsent draft; restart retains settled draft and history.
- [x] C10 Hide/show and resize the side panel with the title-bar control, drag and keyboard alternatives.
- [x] C11 Switch project conversations; developer pages follow the selected project with no dropdown.
- [x] C12 Browse older/newer/latest messages and conversations without losing selection/draft. October 11 native retest preserves the exact draft and reading position; Latest reaches the final reply after the scroll correction.
- [x] C13 Export a test conversation as Markdown and JSON; inspect actual saved contents.
- [x] C14 Branch from a completed turn; branch retains context and uses the same files with fresh permissions.
- [x] C15 Record Worked/Needs work feedback and inspect the retained note.
- [ ] C16 Delete only a disposable conversation; verify its working files are retained.
- [ ] C17 Answer agent questions through clickable choices or custom text; Back retains answers, failed send allows retry, and the agent continues after submission. Native Qwen choice/custom/Back/Next/continuation passed October 10; native failed-send recovery remains open.

## 2. Everyday agent work and recovery

- [x] A01 Ask the model to list/read a small project and explain actual contents.
- [x] A02 Request a new file; exact creation review is understandable and the saved file matches the receipt.
- [x] A03 Request an existing-file edit; inspect/apply the diff and verify only the intended saved bytes changed.
- [x] A04 Deny an operation; model receives the denial and explains a useful next step without claiming success.
- [x] A05 Change a file after an edit proposal; stale apply refuses without overwriting newer work, then fresh review succeeds.
- [x] A06 Run a harmless project check through its exact command review; inspect output and return code.
- [x] A07 Cause a real check failure; Repair and verify fixes it and reruns the same check with truthful evidence.
- [x] A08 Inspect Changes and revert a disposable edit with the reviewed reverse diff.
- [x] A09 Inspect Activity/Earlier tasks, settings origins, tool receipts, checkpoints and reported usage. October 11 native inspection/reopen and corrected labels passed on the retained real Qwen task.
- [ ] A10 Hit a small explicit task/output limit; Continue resumes useful work with fresh approval.
- [ ] A11 Interrupt a task with completed effects; recovery draft explains uncertainty and does not replay earlier operations.
- [x] A12 Read a large text file by ranges and edit a unique snapshot-bound match without truncation corruption. Native qwen3.7-flash edit, stale-review refusal, fresh-read recovery and byte preservation verified; all saved action receipts remain accessible.
- [ ] A13 Capture a larger command log; inspect its saved file and distinguish preview truncation from capture limits.

## 3. Files and editor

- [x] F01 Discover file tree, expand folders and preview a file; no selected project offers Open folder.
- [x] F02 Preview versus pinned tabs: click, double-click and first edit preserve the expected file.
- [x] F03 Edit text, undo/redo and Ctrl+S; UTF-8/BOM/CRLF content survives correctly. October 11 normal-build retest passes Right-Control undo, Ctrl+Y redo and Ctrl+S, including undo after saving. Exact BOM/CRLF/Chinese bytes are preserved. Both Control keys pass Windows widget regressions, including failed-save retry; native Left-Control remains separately unqualified.
- [x] F04 Ctrl+P, Ctrl+F/H and Ctrl+G work with relative paths, find/replace and line navigation. October 11 native keyboard navigation, single-match replacement and undo passed in the public notebook.
- [x] F05 Drag tabs to reorder, split and move between groups; keyboard/menu alternatives work.
- [x] F06 Shared file text/undo and independent cursor/scroll survive two split views and compact layout. October 11 native shared edits, independent cursor/scroll, reopened dirty draft and compact group switching passed. Resumed native undo restored an edit and a replacement from the other pane, clearing both dirty markers; Windows split keyboard undo/redo regression also passes.
- [x] F07 Close a dirty file: Save/Discard/Keep editing preserve the chosen outcome. Native Keep/Save and Discard/reopen pass; discarding the draft preserves a newer saved file.
- [x] F08 Create/Save as/rename a disposable file; an existing target is not silently overwritten. Native create/save, Save as, rename, both existing-target refusals and fresh-name recovery passed October 11; exact byte receipts confirm original and draft preservation. Duplicate error presentation was fixed and retested in the normal Windows build.
- [x] F09 External disk edit produces Compare/Reload disk/Keep my edits; stale save preserves both versions. Native stale save and stale comparison refused with both versions retained; fresh comparison adopted without writing, save recovered the draft, and clean Reload disk showed the exact newer bytes (October 11).
- [x] F10 Restart with settled dirty buffers; private recovery preserves edits without writing source files.
- [x] F11 Binary/oversize/long-line files offer clear readable recovery instead of broken editing.
- [x] F12 Attach an unsaved editor selection to a chosen chat draft; no send until requested.
- [x] F13 Model file reads see saved bytes while unsaved editor changes remain private unless attached.

## 4. Source Control

- [x] G01 Open a selected repository; status/branch, Working Changes and Staged Changes are discoverable.
- [x] G02 Stage/unstage one file with +/-; saved Git index agrees immediately.
- [x] G03 Stage/unstage all with section +/-; include untracked, renamed and Unicode/space filenames.
- [x] G04 Open working/staged inline diffs; additions/deletions and old/new line numbers are correct.
- [x] G05 Side-by-side diff is visibly different, aligned and scrolls both sides together.
- [x] G06 Stage/unstage selected hunks; unrelated hunks remain unchanged.
- [x] G07 Commit message/Ctrl+Enter and Changes menu create a real commit. Native commit and independent Git inspection pass October 11; removing the review dialog is widget-verified, with native retest pending.
- [x] G08 Empty-index Commit automatically stages saved working changes. Native staging/commit passed; the new direct action has focused widget coverage and native retest pending.
- [x] G09 Existing staged selection takes priority over other working changes.
- [x] G10 Failed Git hook retains message/index and gives usable retry after the hook is corrected.
- [x] G11 Dirty editor blocks conflicting Git action without losing its text. Native stage refusal leaves index empty and draft intact; close/reopen and fresh staging recover.
- [x] G12 Click a history commit to expand files; click a file for exact commit diff and collapse again.
- [x] G13 Browse older history and several diff tabs; selections and split views remain usable. October 11 final normal-build retest passes distinct commit labels, independent layouts, exact revision retention and text selection after earlier three-page coverage.
- [x] G14 Large text diff is rendered/pageable inside Dolores; stale page offers Refresh without losing the readable page. October 11 native paired paging, stale refusal and fresh-state recovery passed after repairing Refresh diff; all 6,000 pairs separately verified across 24 real pages.
- [x] G15 Binary diff and nonrepository folder show distinct understandable states.
- [x] G16 Changes menu branch creation/switching works; dirty branch switch refuses safely. Native create/refusal/switch and independent branch inspection pass October 11; direct action retest pending.
- [x] G17 Stash/apply and selected discard work on disposable changes; conflicts retain stash/work and explain recovery.
- [x] G18 Revert a disposable commit; conflict/Abort recovery does not discard unrelated work.
- [x] G19 Fetch/Pull/Push against a local bare remote: selected actions, tracking and fast-forward behavior are correct.
- [x] G20 Divergent Pull, changed refs and interrupted remote action retain evidence and require reconciliation. Native October 12: stale picker refused; entered local Fetch stopped; Refresh reconciled unchanged refs/bytes and zero owned children.

## 5. Terminal

- [ ] T01 Terminal opens a real shell immediately at selected project CWD or HOME when no project is selected.
- [ ] T02 Physical keyboard letters/punctuation, Enter, arrows, Backspace and Unicode/IME reach the shell correctly.
- [x] T03 Plus opens another shell at current project; existing shell retains its own CWD after navigation.
- [ ] T04 Tabs reorder/move/split; menu alternatives and compact group selection work.
- [ ] T05 Select/copy text; Ctrl+C copies selection or interrupts foreground work when no selection exists.
- [ ] T06 Stop/close live shell gives the expected review and keeps readable output.
- [ ] T07 Restart retains stopped output; Start creates a fresh shell without replaying commands.
- [ ] T08 Attach selected output to a chosen chat draft without automatically sharing full scrollback or sending.

## 6. Scheduled tasks

- [x] S01 Natural English weekday request creates a task without manual setup and visibly confirms exact schedule/model.
- [x] S02 Natural Chinese weekday request does the same; paraphrases such as each workday are supported by interpretation.
- [x] S03 Missing time uses disclosed 09:00 local default; quoted examples/negations create no task.
- [x] S04 Enabled project skill is pinned to the created task; project/model/skill receipt matches the request.
- [x] S05 Scheduled page shows next occurrence/timezone/status and original conversation; no manual creation form.
- [x] S06 Run now completes a real-model occurrence and exposes result, progress, error and usage.
- [x] S07 Change time/pause/resume in original chat; existing task changes without an accidental duplicate.
- [x] S08 Page Pause/Resume/Skip next/Stop operate correctly on a disposable schedule.
- [x] S09 A real timed occurrence runs while open; approval-needed status is visible and recoverable.
- [ ] S10 Interrupted/failed occurrence keeps evidence; explicit retry does not replay uncertain effects.
- [x] S11 Background close-to-tray/reopen/Quit preserves task result and draft with no duplicate owner.
- [ ] S12 Delete only a disposable schedule; original conversation/results and files remain inspectable as specified.

## 7. Settings, attachments and context

- [x] P01 Find settings by category/search; short labels/help hints and bottom placement remain clear. October 11 native category selection, frequency search/result navigation, compact slider labels and hover-only help passed; settings closed without changing preferences.
- [x] P02 Light/Dark/System apply consistently; pending form drafts survive navigation.
- [x] P03 Model context/output/stall/reasoning settings save for the selected model and survive restart.
- [ ] P04 Scoped user/project/chat overrides and inheritance show correct effective values on new runs.
- [x] P05 Close/Escape with unsaved settings gives Save/Discard/Keep editing; cancel preserves values. Native October 12: timeout 30→31; Keep editing retained 31, Discard restored 30 on reopen, Save retained 31 on reopen.
- [ ] P06 Connection controls list/manual models, selected model and remembered restart work without exposing a key.
- [ ] P07 Attach/preview/remove/send public UTF-8 text; attachment snapshot survives later source changes.
- [ ] P08 Attach public PNG/JPEG and send to a capable model; incompatible-image error preserves draft and offers settings recovery.
- [x] P09 Over-limit/unsupported attachments give actionable rejection; attachment export contains matching manifest/snapshots.
- [x] P10 Context ring shows preparation/estimates vs reported usage without an unintended request.
- [x] P11 Review/save/edit session summary; later request recalls it while full history remains accessible.
- [ ] P12 Automatic compaction with a small context preserves draft and gives useful failure recovery.
- [ ] P13 Review-every-operation and selected/full-access grants affect only intended chat; revoke/expiry retains effects/evidence.
- [ ] P14 Multiple Window availability and wakefulness experimental setting are concise and preserve preference.

## 8. Memory, instructions and skills

- [ ] M01 One Memory switch starts automatic useful fact/preference capture from real completed chats, with sources/activity.
- [x] M02 Fresh conversation recalls a retained useful fact; no manual population is required.
- [x] M03 Inspect/correct a memory; protected correction survives later extraction.
- [ ] M04 Disable capture/recall; existing memory remains inspectable and can be forgotten deliberately.
- [ ] M05 Project facts capture approved successful check evidence with clear declaration/execution distinction.
- [ ] M06 Changed/missing/stale sources are excluded; project facts stay in the appropriate project.
- [ ] M07 Review/enable root AGENTS.md; changed/missing file blocks reuse and offers re-review/disable.
- [ ] M08 Review/activate project skill; use it in a real task and show selected skill provenance.
- [ ] M09 Global/project precedence, changed-source re-review, disable/rollback/export operate on disposable skills.
- [ ] M10 Draft skill from completed chat, edit/test it and activate only a complete strict improvement.

## 9. Optional research and integrations

- [x] I01 Web search returns labeled source results or a truthful empty/error outcome; refine/direct URL recovery works.
- [x] I02 Read a public primary page with citations; partial excerpt and provenance are visible.
- [x] I03 Browser readiness/setup status is understandable; missing optional adapter has actionable recovery.
- [x] I04 Model opens a disposable local web page, reads it and requests reviewed click/input; local screenshot is inspectable. Native October 12: DeepSeek, four Allow once actions, five calls, exact public form result and readable local screenshot; zero owned children after completion.
- [x] I05 Browser Stop/stale reference recovery closes owned browser and avoids blind repeated input. Native October 12: entered Qwen navigation stopped/restored draft; DeepSeek's stale Fill dispatched nothing and recovered only with new state/review; zero owned children after each run.
- [ ] I06 Inspect/enable a harmless local MCP server; catalog discovery does not silently execute tools.
- [x] I07 Real-model invocation of enabled external tool shows exact review/receipt; missing/stale server recovers usefully.
- [ ] I08 Attach a disposable Windows window for view-only inspection with a capable model; target/sharing are explicit.
- [ ] I09 Selected-window control requires fresh state and verification; Stop/closed target never switches silently.
- [ ] I10 Model-directed delegation shows child progress/results and consumes the shared budget correctly.

## 10. Occasional and experimental workflows

- [ ] E01 Opt-in companionship model/hours and Quiet–Chatty frequency (0–100/day) produce a bounded in-app note when eligible, quiet during active work.
- [ ] E02 Open/Not now/Dismiss/Fewer messages/Turn off behave clearly without triggering external actions.
- [ ] E03 Compare instructions retains exact cases/usage and never activates a tie or incomplete result.
- [x] E04 Skill testing has fixed disposable cases and truthful candidate/baseline results with Stop recovery. Native October 12: DeepSeek comparisons retained exact cases/replies/usage and refused ties; a labeled delayed local fixture verified entered-request Stop and retained editable inputs.
- [ ] E05 Project reflection/automatic activation/pause retain source evidence and only activate complete independent gain.
- [ ] E06 Restore/quarantine learning baseline; manual edits conflict instead of silently overwriting work.
- [ ] E07 Harness extensions test source, bounded model draft and strict-gain activation preserve incomplete candidate source.
- [ ] E08 Recovery mod health rollback and restart reconciliation retain baseline and never replay an interrupted action.
- [x] E09 Dolores capabilities/local diagnostic views explain build/tools/limits without an unintended model request.
- [ ] E10 Native repair proposal/reproduction/regressions/build reviews demonstrate a real failure-to-fix result in isolation.
- [ ] E11 Isolated Install & restart/Restore & restart preserve newer draft/history and reject stale plans.
- [ ] E12 Diagnostics/storage exports and cleanup operate only on selected disposable evidence and report real outcomes.

## Results and issues

Initial build: normal Windows main entry at `a235b7d`; subsequent fixes and normal
rebuilds are recorded below. Isolated configuration-only profile;
no original chats/projects imported. Local evidence is retained under the ignored
`output/first-user-audit-20261008-03/` directory, including native accessibility
observations and original-profile preservation receipts. Live calls use Qwen with
a 2,048-token output bound and 60-second stall timeout unless stated otherwise.

| IDs | Result and evidence | Remaining coverage |
| --- | --- | --- |
| C02 | Home first send created a Temporary conversation and working folder. Qwen answered the coding introduction in two sentences; Run details showed one model call and the context ring reported 6,239 tokens. Native UI observation log records prompt, response and controls. | Streaming/copy/Stop are separate checks. |
| C03 | Home Open project opened the native folder picker. Selecting the disposable weather notebook created a project conversation, sidebar group and matching working-folder title. | Project switching is a separate check. |
| A01 | Qwen requested folder listing and `notes.txt` read; both Allow once controls worked. Final answer correctly reported Harbor, celsius, ready; two completed tool receipts and three model calls were visible. | Search is pending separately; no edits requested. |
| G01, G02 | Source Control showed main, two working changes and empty staged section. File + staged only `notes.txt`; file minus restored an empty index. Read-only Git inspection agreed with the visible state. | Other mutation flows pending. |
| G04 | Working and staged tabs showed Index → Saved working tree and HEAD → Index respectively. Inline diff correctly showed line 3 draft removed / ready added, retaining lines 1–2. | Longer diff and large-file paging pending. |
| G03, G05 (partial) | Section + staged both notes and untracked `todo 文件.txt`; section minus unstaged both. Index inspection agreed. Side-by-side rendered labeled old/new columns with aligned draft/ready row. | Renames pending for G03; long synchronized scrolling pending for G05. |
| F01, T01 (partial) | Files displayed expected hierarchy and opened notes as a CRLF/UTF-8 preview. Terminal opened PowerShell immediately at selected project CWD. | Folder expansion/no-project state and HOME fallback still pending. |
| S01 (failure), A04 (partial) | Qwen failed a clear weekday 21:00 creation request: four malformed scheduling calls, one unavailable command, then an unrelated Python date command. The command was denied and never executed; the run paused after six model calls without creating a task. | Qwen scheduling competence and a useful denial reply remain unaccepted. Generic invalid-fields feedback does not identify the actual malformed fields. |
| S01, S05 (partial) | DeepSeek's first call saved the correct weekdays/21:00/Asia-Shanghai task with the selected project and model, without another setup step. The confirmation call stalled after 60 seconds; the task survived. Initial chat showed a plan ID and Scheduled appeared empty until manual Refresh. | Visible feedback and page freshness need native retest after the fix; task execution and management remain pending. First DeepSeek call reported 6,328 tokens; stalled call usage unavailable. |
| S01, S05 regression checks | A scoped fix renders a completed task receipt as “Task scheduled” plus its rule, discloses default time and retains optional details. Opening Scheduled refreshes its listing even while chat is busy. Tests reproduced both old failures before the fix; all 20 focused checks passed, including malformed/failed receipts and unavailable-storage recovery with draft retention. Normal Windows build/launch and native reopen passed: the retained receipt shows correct schedule/model/project and Scheduled immediately shows the paused task. | Fixture results establish the busy-chat timing regression, not model competence. That exact live creation timing remains pending. |
| S06 (failure) | Run now started a separate DeepSeek occurrence and showed a folder-listing review in Scheduled. The review and running strip mislabeled the pinned project “Temporary workspace.” The stored occurrence still pins the correct public project. Stopped before approving the listing. | Diagnose actual folder binding versus displayed identity, fix and retest before accepting scheduled execution. |
| S06 identity regression | The actual result session uses the pinned project, but `scheduledStart` omitted workspace metadata and its chat controller retained default identity/model. The bridge now returns the authoritative workspace; the controller loads it and the pinned model. The AppHost test failed on the original project identity, then 17 focused Flutter and five scheduling Rust checks passed, including temporary-folder binding and retained Home draft/model. The packaged C ABI check failed on the old build's missing workspace field; rebuilt save/reopen corpus passed with 11 local requests and no live requests. Native DeepSeek Run now correctly labels the project in the running strip and operation reviews. | Report completion remains under test. All 49 original-profile table hashes remain unchanged. |
| S06 result (failure) | Scheduled history reported Succeeded and Open result opened the separate completed conversation. DeepSeek used actual Harbor/celsius/ready notes and produced report content, but also claimed it could not create the schedule and asked which scheduler to use. The original creation request was being reused as its execution goal. It additionally called a manually read, unpinned skill “pinned.” | Scheduled execution handoff must distinguish already-completed setup from today's work. Do not accept semantic task success from the Succeeded transport state alone. |
| S06 handoff regression and live pass | The host now explicitly marks setup already saved and dispatches today's work, with the actual nullable skill pin. The old packaged provider handoff failed the regression; rebuilt save/reopen passed with 14 local requests, including an unpinned occurrence in a project with an enabled skill. Native Run now of the same paused DeepSeek task completed a report grounded in README, notes, to-do and check source, accurately recognized the saved schedule and made no pinned-skill claim. Scheduled showed approval progress and Succeeded; Open result and Run details worked. Four calls reported 5,131 / 5,298 / 5,422 / 5,856 total tokens. No files were edited and the check was not executed. | Report remains longer than desirable for “short”; this is a model-quality observation. Physical recurring dispatch, pinned-skill live use and model creation reliability remain separate pending checks. |
| C05, C09; F01 partial | On the rebuilt normal app for `e4eaffb`, Qwen → DeepSeek → Qwen retained the completed report and a public unsent draft. Files navigation retained that draft; `.agents` expanded to its child folder and the notes preview remained readable. Idle title-bar Close followed by normal launch restored the same project, report history, Qwen selection and exact settled draft. | IME composition and abrupt-interruption recovery remain distinct pending cases. Files with no selected project is still pending. |
| C04, F01; model limitation | New chat options → Side chat created a visibly file-free chat. Qwen answered the small-commit question but falsely claimed project read/write access (one call, 2,392 reported total tokens). DeepSeek accurately corrected the claim, listed only the two actual scheduling tools and declined the project-file read (one call, 2,612 reported total tokens). A diagnostic packaged-provider check agreed that no file/command tools were supplied; it is not a real-model pass. Folders with this side chat selected showed Open folder and the no-project explanation. | Qwen capability-answer reliability remains a failed model case. No file read or grant occurred, and stronger-model fallback was effective. |
| C12 (failure, fixed navigation) | Returning from the new Side chat to a retained project chat removed the new chat from Recents. The cached controller kept an old conversation catalog; the stored chat remained intact. Two AppHost checks reproduced the missing row and failure-to-refresh recovery. Navigation now refreshes only the shared catalog. All 16 focused checks passed, including unavailable-storage retry and preservation of a retained model, draft, partial response and pending review. Normal build/launch passed. Native retest loaded the earlier report, created another Qwen Side chat, then returned to the cached report: both side chats remained visible, the exact project draft returned, and the retained DeepSeek selection was preserved. | Older/newer paging remains pending. New Qwen code reply used one call, 2,353 reported total tokens. |
| C07, C10 (partial) | Native Qwen response rendered the exact Python code block and sentence. Copy code produced `print(42)` with its trailing newline; Copy message produced the complete fenced Markdown and sentence. Title-bar panel hide/show, widening the divider and dragging to hide retained the project draft and conversation. | Thinking/long streaming and keyboard panel alternatives remain pending. |
| F02 (failure and retest) | Double-clicking notes in the tree did not keep its tab; opening README replaced it. Tree rows had no double-click handler. A second regression found reopening a kept file removed another preview and made the kept file a preview again. Both tests failed before the fix. The tree now opens kept tabs on double-click, and reopening a retained tab preserves its pin and the other preview. Fourteen focused checks passed, including failed-open retry and accessible button/focus actions. Native rebuild retest kept notes beside italic README and retained both when notes was clicked again. The final normal build passed; native file rows expose named buttons and reopening notes retained both tabs. | First-edit pinning remains pending. Helper keyboard/clipboard injection did not produce a verifiable edit in the code surface, so F03/F04 remain unqualified; no saved project bytes were changed. |

| G12 (pass) | Native History click expanded the initial commit's five files; notes opened an Empty tree → exact commit comparison containing `Status: draft`, matching the saved commit rather than working `ready` contents. Clicking the commit again collapsed its files while retaining the readable diff. | Older history paging and multiple-tab navigation remain under G13. |
| G07 partial; G08 (pass) | Native Changes → Commit with an empty index showed the two exact saved working changes, including the Chinese-named untracked file, author and stage-and-commit notice. Cancel retained the message and empty index. Fresh review → Apply created the exact two-file commit; the app cleared the message and showed No changes. Independent Git inspection confirmed the commit and clean tree. | Helper Ctrl+Enter inserted a newline; the existing deterministic shortcut test passes. Physical shortcut acceptance remains unqualified, consistent with the broader modifier-input calibration gap. |
| G09, G10 (pass) | Staged only notes, leaving a separate to-do edit working-only. Review listed only notes. A deliberate disposable pre-commit refusal displayed the actual hook message and Refresh recovery instruction; no commit occurred, and message/index/working changes remained. After correcting the hook and pressing Refresh, fresh review succeeded with only notes committed; the to-do edit remained working-only. | No hooks in the real project or normal user repositories were changed. |
| G03 (pass) | Native section + staged an untracked space/Chinese filename and a renamed Chinese file. Git independently reported the new addition and 81% rename; the UI showed its old filename. Section − restored an empty index and the working deletion/untracked paths, preserving saved file contents. | Bulk actions performed no commit or remote operation. |
| G13 (failure, fixed refresh) | After two successful native commits, History still showed only the initial commit even after Changes Refresh; saved Git HEAD had advanced. The host updated status without reloading history. Two focused regressions failed before the fix. Applied actions now reload the first history page only when the current commit differs from the loaded history head, preserving exact-commit diff tabs and retained expanded files. All 24 focused checks and changed-source analysis passed, including unavailable-history retry without replaying commit and unchanged-HEAD staging retaining loaded pages. Normal build/launch passed. Native confirmation created a new rename/addition commit: its row appeared without Refresh, while the older expanded commit's files and exact diff stayed readable. | Older-page and multiple-tab navigation remain pending. |

### Open observations

- A04 passed through the native DeepSeek fallback. Denying its requested
  `README.md` read produced a denied receipt and a truthful final response:
  it had no README contents, could not summarize them, and offered an explicit
  user-requested retry or another way to share the text. No read/edit/command
  ran and no automatic retry occurred (two calls, 6,728/6,804 tokens).
  The earlier Qwen denial explanation remains a model limitation.

- F11/F12/F13 passed on the normal `8abca4b` Windows build. Binary (3 bytes),
  long-line (100 KiB) and oversized (5 MiB) disposable files each opened as a
  labeled read-only preview with Save disabled. The text previews disclosed
  their editing and preview limits. All three original files stayed byte-identical;
  existing editable tabs remained usable.
  An unsaved catalog-copy buffer contained `Books: 7`, while its saved CRLF
  source still contained `Books: 4`. Qwen declined without calling a tool,
  incorrectly asking for permission to attempt the already-requested read
  (one call, 6,306 tokens). DeepSeek requested the ordinary single-file review
  and correctly read 4 (two calls, 6,258/6,295 tokens). Native text selection →
  Editor actions → Attach selection to conversation showed the unsaved version
  and selected range, then attached only that snapshot to the chosen chat draft.
  No send or new generation occurred on attachment. Preview showed `Books: 7`;
  after explicit Send, DeepSeek answered 7 without any tool call (one call,
  6,446 tokens). The saved source stayed at 4 throughout. Qwen competence for
  this flow remains unaccepted; the boundary passed through DeepSeek fallback.

- F06/F08/F09 gained partial native coverage on the same build. Replace updated
  both split catalog panes and both dirty indicators. Saving after an external
  disk change refused and retained editor value 4 and disk value 5. Compare
  displayed both values. A further disk change to 6 invalidated Keep my edits
  from that comparison, with an actionable Compare again message and both
  versions preserved. Fresh Compare → Keep my edits adopted the reviewed disk
  revision without writing; a subsequent explicit Save wrote the retained 4
  with exact Unicode/CRLF bytes and cleared both dirty indicators.
  Create produced an empty disposable file; Save as created an exact copy,
  retained the original and updated both shared views to the new path. Create
  at that existing copy path opened it without overwriting any bytes.
  Shared undo/independent long scrolling/compact selection, a distinct file
  rename, Save-as collision and Reload disk remain pending. Reload disk and
  Discard require action-time confirmation under the computer-use skill; no
  destructive UI action was performed or counted as a pass.

- F04 failed through the visible Editor actions → Find/Replace route: opening
  either panel covered the entire catalog document, hiding its search matches.
  Four wide/compact light/dark regressions reproduced the full-height overlay.
  The panel now sizes to its rows and reserves space above the first line; tests
  also select the document below the panel and replace a match with Unicode text.
  All 18 focused file/editor checks and changed-source analysis passed. The normal
  Windows rebuild and native retest passed: Replace kept Garden highlighted in
  the visible catalog, then replaced only that word with 海风. The tab changed
  from italic preview to a kept dirty tab. Opening a README preview retained the
  exact unsaved catalog (F02), and later double-click kept both clean tabs.
  The source file stayed at its original 25 CRLF bytes. After a settled owned-
  process interruption/relaunch, Files restored both tabs and the exact unsaved
  Unicode buffer without writing the source (F10). Dirty Close → Keep editing
  retained it; a fresh Dirty Close → Save saved and closed it. Independent bytes
  matched `Shelf: 海风\r\nBooks: 3\r\n`, and the README remained unchanged.
  Reopen showed that clean content. F03/F07 remain partial for undo/redo/BOM,
  keyboard Save and Discard; F04's pointer search/replacement recovery passed,
  but quick-open, line navigation and physical shortcuts remain unqualified.
  Native dragging reordered tabs, split at the right edge, and moved the catalog
  between existing groups. Move tab to next group and Split down worked through
  the overflow menu, retaining both file contents (F05). Keyboard entry
  remains unqualified: helper paste/chords did not enter text in the code view;
  native context-menu paste works in the separate Find/Replace fields.

- S02/S03 initially failed on the Chinese workday/default-time case. DeepSeek correctly
  interpreted the request but the actual save failed after a reviewed public
  folder listing. It reported no task was created and disclosed the intended
  09:00 default; independent storage agreed. Three calls reported 6,134 / 6,302 /
  6,479 tokens. The invocation receipt misleadingly said Folder tool could not
  complete within its text and access limits. The 247-byte prompt generated an
  80-character title containing 208 UTF-8 bytes, exceeding the frozen 160-byte
  storage limit. A regression with the exact prompt reproduced Invalid scheduled
  task; the title now respects both the existing character/byte ceilings without
  splitting a character or changing the full prompt/rule. A separate agent-flow
  regression reproduced the unrelated folder error. Scheduling invocation errors
  now retain known limit/stale/expired recovery or give a bounded scheduling
  message requiring inspection before retry, without backend diagnostics.
  Chinese/Japanese/emoji/English fixtures cover default-time receipts and duplicate
  reconciliation. The normal Windows rebuild passed. A fresh native project chat
  repeated the exact Chinese request; DeepSeek corrected its first invalid call
  and saved the weekday 09:00 Asia/Shanghai task without further setup or file
  access. Its visible Task scheduled receipt explicitly said Default time, and
  the final Chinese answer agreed. Three calls reported 6,200 / 6,336 / 6,538
  tokens. Scheduled immediately showed the task, exact model/project/source,
  weekdays and next day at 09:00. Independent storage confirmed a complete
  160-byte title prefix and the unchanged full 247-byte prompt. The task was
  paused after inspection. Two fresh Chinese controls asked for an explanation
  of an explicitly negated example and, separately, a quoted sentence from a
  document without an explicit prohibition. Neither dispatched any tool or
  altered any saved task; before/after hashes of all three complete task records
  matched. They used one call each, 6,095 and 6,112 tokens. The quoted answer
  exceeded the requested one sentence but accurately distinguished explanation
  from creation. S02/S03 are checked for these real DeepSeek cases; this does not
  establish Qwen reliability or arbitrary-language competence. Timed execution
  of the new task remains outside this creation check.

- S01/S05 passed with the DeepSeek fallback on the normal repository-identity
  build. A fresh project chat requested a short README summary every weekday at
  9pm. DeepSeek corrected one rejected schedule call and saved the weekdays
  21:00 Asia/Shanghai rule without additional user setup. The final confirmation
  and Task scheduled receipt matched the saved project/model; three calls
  reported 6,166 / 6,305 / 6,525 tokens. Scheduled listed the new task immediately
  without Refresh, with today's 21:00 next occurrence, correct timezone, model,
  project, no pinned skill and No runs yet. Open source chat returned to the
  exact originating conversation. The isolated database confirmed one new task
  with the matching rule and source. Execution remains under its separate checks.
  Qwen's first attempt failed: two approved public reads, repeated malformed
  scheduling calls, an invalid browser call, then a truthful bounded pause after
  eight calls. No task was created by Qwen; per-call reported totals were 6,283 /
  6,393 / 6,529 / 6,704 / 6,849 / 7,013 / 7,114 / 7,384 tokens. Its progress falsely
  claimed recurring tasks require a date and ignored the supplied time. This is
  a fallback pass, not Qwen scheduling reliability. All native observations
  remain in the ignored audit directory.

- S07 passed with DeepSeek in the original source chat. A single request changed
  the existing task to weekday 20:30 and paused it; a later request resumed it
  with that exact rule. Each confirmation matched the independent saved state,
  keeping the same task ID, advancing revisions 1 → 3 → 4 and retaining exactly
  two total tasks (the prior paused weather task and this new summary task).
  Update/pause used three calls (6,415 / 6,712 / 6,900 reported tokens); resume
  used two (6,474 / 6,664). No extra setup, file edit or command execution occurred.
  S08 is partial: native page Pause/Resume immediately changed status and next
  date; Skip next advanced October 8 to October 9 at 20:30 and saved a Skipped by
  you occurrence. Stop current run was disabled with no active occurrence.
  The task was paused again after the case; fresh active Stop remains pending.
  Native change receipts exposed opaque plan IDs. Six narrow light/dark
  regressions reproduced this before the fix. Successful changes now show their
  saved updated/paused/deleted outcome and schedule; paused/deleted details omit
  an active Next time. Failed/malformed receipts retain their actual evidence.
  All 28 focused Flutter checks and changed-source analysis passed. The normal
  Windows build and native restart retest passed: saved updated/paused/resumed
  cards retained their distinct outcomes; the expanded paused card showed the
  exact task ID/model/project and no active Next time. Deletion labeling is
  fixture-qualified only; native deletion remains pending under S12.

- C01/C11/T03 passed. The compact rail exposed all six pages, with Settings at
  the bottom. Home opened a second disposable public
  folder and created its project chat. Files showed that folder's two files;
  returning to the original repair conversation restored its own hierarchy and
  retained notes/forecast tabs. Source Control correctly discovered the second
  folder's enclosing Git worktree, as specified, but hid its actual root in a
  branch hint. No Git action was performed against that enclosing repository.
  The panel now names the repository and shows its selectable canonical path,
  with an Enclosing repository label when different from the selected project.
  Two light/dark regressions failed before the root-display fix; all 26 focused
  Git checks and source analysis passed, including failed reads, project changes,
  equivalent Windows paths and long canonical paths. Native root switching
  passed on the normal rebuilt app, including readable names for the long
  canonical paths. Terminal's existing stopped session retained the original
  folder; Plus opened PowerShell immediately at the second project. Returning
  to the original repair chat kept that live terminal at its own CWD, while
  another Plus opened a separate shell at the now-selected original project.
  No terminal commands or keyboard text were entered. Terminal input, HOME
  fallback and split interactions remain separate pending checks. Screenshots
  and native observations remain in the ignored audit directory.

- P01 search failure repaired: the native search term `frequency` returned No
  matching settings despite the visible slider. The search catalog omitted the
  new Frequency/Chatty labels. Two wide/compact regressions failed first; the
  corrected keywords passed with unknown-search recovery, cached unsaved slider
  retention and no implicit save. All 16 focused settings/companionship checks,
  changed-source analysis and the normal Windows build passed. Native retest found
  Companionship from General and opened the actual 0–100 slider. Saved before/after
  screenshots and native observations remain in the ignored audit directory.
  P01 remains partial: native hover-help behavior is not yet qualified.
- P02 passed: native Light applied across Settings and Home, reopening retained
  Light, and Dark applied while an unsaved explanation-style draft remained
  cached across category navigation. Dark and the explicitly saved preference
  survived the normal rebuild/restart. Selecting System restored the device's
  dark appearance. P05 is partial: Close and Escape each offered Save/Discard/Keep
  editing; Keep editing retained the draft, and Save closed Settings and retained
  the value after restart. Discard is still pending. The audit profile's original
  System appearance and explanation style were restored through the controls.

- C08 initially failed on the normal frequency build: a bounded real Qwen side
  reply streamed text, but Stop removed that text from chat and Earlier tasks,
  restoring only the input draft. Two deterministic regressions reproduced the
  side-chat loss and loss of the latest agent text when three earlier steps filled
  the display bound. The terminal UI handler discarded the current public text.
  A scoped fix retains it in the existing Not saved panel with a Partial label,
  replacing the oldest transient step if needed; saved history stays unchanged.
  Twenty-two focused Flutter checks and source analysis passed, with light/dark
  compact coverage, latest-step retention, fresh-request isolation and existing
  Stop/approval behavior. The normal Windows rebuild passed. Native Qwen retest
  streamed the same bounded 80-item request; Stop restored its draft and expanding
  Not saved displayed the actual unfinished text and Step 1 · Partial. Replacing
  the draft with a separate short request returned exactly Ready for the next
  task (one call; 2,320 reported tokens). Storage contained only that new exchange,
  with no stopped text saved or replayed. The cancelled call had no final reported
  usage; its initial input estimate was 2,455 tokens with a 2,048 output allowance.
  Some generated review tips were incoherent; this check qualifies interruption
  recovery, not advice quality. Retained partial text is transient until another
  chat/run and does not survive restart; durable interrupted-output recovery
  remains a separate gap. Original profile hashes remained unchanged across all
  49 tables.

- C14 passed with a DeepSeek fallback on the normal rebuilt frequency app.
  Chat actions → Branch chat offered completed turns and disclosed the shared
  working folder and separate permissions/drafts. Selecting the completed repair
  created a distinct fork with the exact four original messages and same project
  root; the original chat retained those messages. Qwen was asked to read notes
  and report its Units line, but answered Fahrenheit without a tool call despite
  the saved file containing Celsius (one call; 6,568 reported tokens). An explicit
  fresh-read request with DeepSeek opened a new Allow a file read review. Allow
  once returned the actual current four lines; its expanded receipt and final
  Celsius answer agreed (two calls; last reported 6,558 tokens). Independent
  storage/root and exact file-byte checks agreed, with no edits. This qualifies
  branch context and fresh review, not Qwen's stale-history answer reliability.

- C13 passed on the normal rebuilt frequency app. Chat actions → Export offered
  Markdown, JSON with run details and Attachments. Markdown and JSON each used
  the native Save As dialog to save a fresh ignored file, then showed Exported
  4 messages while retaining the selected chat/model. JSON parsed successfully;
  all four IDs/roles/content matched stored messages exactly. Markdown contained
  every complete message. The exported actual command receipts retained the
  earlier exit 1 and later exit 0 for the same literal Python check, separate
  four-call runs, and the saved Worked feedback/public note. No model request,
  project-file change or remote transmission was involved. Attachment export
  remains under its separate pending check.

- G05/G06 passed in native Source Control. A historical notes comparison displayed
  distinct old/new contents with the added row aligned. A disposable 200-line
  public file had five separated changes; side-by-side showed red old/green new
  rows and remained aligned when scrolling from either side through the last hunk.
  Choose hunks selected only line 10; the exact stage review omitted the other four.
  After Apply, independent Git inspection confirmed only line 10 in the index,
  with all five edits still saved and the separate forecast file untracked.
  On the normal rebuilt frequency app, the staged comparison retained that exact
  hunk. Review unstage selected hunks and Apply returned the index to empty while
  preserving all five saved edits, all 200 lines and the exact untracked forecast.
  These existing flows needed no model call or Git runtime change. Observations
  remain in the ignored native audit log; further Git flows remain unchecked.

- E01 remains partial after the requested frequency refinement. Native Settings
  loaded the previous 2; the Quiet–Chatty slider saved both 100 and 0, with the
  stored policy agreeing and the model, hours and Off state retained. Focused
  clocks/storage/UI checks cover high-cap pacing, zero suppression, restart and
  failed saves. Eligible native real-model delivery still needs a separate case;
  this UI save check does not establish proactive delivery reliability.
  A retained Qwen note in the isolated profile opened into its existing side
  chat and cleared the unread banner without starting another generation or
  exposing project tools (E02 partial). Its stored generation usage was 122
  tokens. Fresh native eligibility/delivery, quiet-during-work and the remaining
  Not now/Dismiss/Fewer messages/Turn off actions are still pending.

- A08 and C15 passed on the normal rebuilt repair build. Changes listed saved
  edits across chats in the selected working folder. The forecast edit displayed
  its exact forward diff; Review revert showed only cloudy-to-clear with Bay
  retained. Cancel preserved all bytes. Fresh review and Revert once restored
  exactly `City: Bay\nForecast: clear\n`, marked the earlier edit Reverted and
  saved a separate reverse-change record. Notes retained the verified Celsius
  CRLF text; every other public file remained unchanged. No model call was needed.
  Task feedback saved Worked with a public note and reopened with both retained.
  Switching to Needs work, saving and reopening kept the same note; Worked was
  restored to accurately describe the completed repair. No new request was sent.
  Activity listed the saved exchanges, Earlier tasks retained separate paused
  and completed runs, and expanded command receipts/per-call usage were readable.
  A09 remains partial: settings-origin and additional checkpoint inspection are
  still pending. All observations are in the ignored native audit log.

- A07 failed in native UX after deliberately changing the public notes' units
  to fahrenheit. The identical reviewed `python check.py` failed with exit 1 and
  retained AssertionError output. Repair and verify submitted the generic
  continuation, and DeepSeek refused a repair because the original task said not
  to edit files. It also falsely disowned the previous real exit-0 receipt;
  that separate historical-evidence issue remains unresolved. Inspection found
  both normal and summary-aware history select conversation text and attachment
  parts without the saved agent tool receipts, leaving the next request without
  the earlier command's actual receipt. Missing evidence does not justify the
  model's claim that the earlier result was fabricated. Qualify a bounded retained
  evidence projection in a separate repair before closing this gap.
  The failure and generic continuation used two and one calls respectively;
  last calls reported 6,839 and 7,434 tokens. Regression tests reproduced the
  missing repair intent at the visible button and saved recovery prompt.
  A scoped fix explicitly conveys the selected repair request while retaining
  fresh operation reviews, exact-check evidence and ordinary Continue behavior.
  The rebuilt native retry now submitted the explicit repair request and made
  fresh reviewed reads of the check and notes. DeepSeek correctly identified the
  mismatch and asked which units were required (two calls; last reported 8,135
  tokens). After a Celsius clarification, it again falsely disowned the actual
  prior reads and asked for approval only in prose (one call; 7,634 tokens).
  An explicit instruction to use the review UI produced a real fresh read, but
  the final answer promised an edit without calling the edit tool (two calls;
  7,889 tokens). No file changed. This retry does not qualify A07.
  A separate clean conversation stated the Celsius requirement up front and
  requested the same check without edits. Fresh reviewed reads and command
  execution retained the real exit-1 AssertionError (four calls; last reported
  6,699 tokens). Repair and verify then required no further chat instructions:
  reviewed fresh reads, the one-line Fahrenheit-to-Celsius diff, and the same
  literal command all completed. The expanded receipt showed exit 0, CHECK OK
  and empty stderr, agreeing with the final answer (four calls; 7,760 tokens).
  Independent comparison of all eight public files confirmed only notes changed;
  its exact CRLF bytes, other lines, check.py and forecast stayed intact.
  This rebuilt normal-app flow qualifies A07; the earlier conversation's missing
  historical evidence and prose-only approval behavior remain unresolved.

- A06 passed with DeepSeek on the same normal build: native review showed the
  actual Python executable, the project CWD and one literal `check.py` argument.
  Run once completed with exit 0. The expanded receipt retained `CHECK OK`, empty
  stderr and the exact argument; the brief final answer agreed. Two calls; last
  call reported 6,579 tokens. The fixture check only reads notes; no file edits.

- A03/A05 passed on the normal `a8a2ab4` build with DeepSeek. While the
  clear-to-cloudy edit review waited, the disposable file was externally changed
  from Harbor to Bay. Apply once refused with “File changed since preview. No
  edit was applied.” Independent byte inspection confirmed the newer content
  remained. A separately reviewed fresh read exposed Bay; DeepSeek accurately
  explained the stale edit and asked whether to retain the external change
  (three calls; last call reported 6,498 tokens). After explicit chat confirmation,
  a fresh diff retained Bay and changed only the forecast. Apply once saved exactly
  `City: Bay\nForecast: cloudy\n`; tracked files remained unchanged. The final
  answer and edited receipt agreed (two calls; last call reported 6,527 tokens).
  Native observations remain in the ignored audit directory. No app fix was needed.

- A02 passed with DeepSeek after Qwen proposed content without the requested final
  newline. Native review exposed the missing newline before any write; Deny
  left the file absent. Qwen's final response incorrectly called this a directory
  access restriction (two calls; last call reported 6,360 tokens), so A04 remains
  open. DeepSeek's fresh review contained both LF newlines; Create once saved
  exactly `City: Harbor\nForecast: clear\n` (29 bytes), matching the concise
  created receipt. Independent Git inspection showed only the new file. Two
  calls; last call reported 6,308 tokens. Normal build at `a8a2ab4`; evidence is
  in the same ignored native observation log. This is a successful fallback,
  not evidence that Qwen's formatting or denial explanation is reliable.

- Input calibration: helper text injection works in a blank native Notepad but
  does not enter text in the Flutter composer. Physical letter injection entered
  an IME composition; subsequent paste rendered text but Send remained disabled,
  and navigating away lost that uncommitted composition. A clean context-menu
  paste after returning to Home enabled Send and completed a real response.
  This is unresolved native input/IME coverage, not an established draft-loss
  defect. Do not mark C06/C09 passed or patch the app without a focused reproduction.
- The owned original preview was backed up and closed after confirming no active
  task, to isolate native input. Its profile must be restored when this audit
  relinquishes the desktop; existing user data and provider configuration remain
  preserved.
