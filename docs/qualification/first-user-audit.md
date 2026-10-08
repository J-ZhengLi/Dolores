# First-user feature audit

Started 2026-10-08. This is an ongoing audit of the normal Windows app, using
native user interaction and real configured models. Existing acceptance reports
and unit tests do not count as passes here. Order follows likely everyday use.

## Method and status

Act as an experienced developer discovering Dolores through its visible controls.
Use disposable public projects and an isolated configuration-only profile; keep
personal conversations, working projects and provider settings unchanged. Use
Qwen3.5-2B first, then DeepSeek V4.1 Flash when a task demonstrates a model limitation.
Bound each request and record reported usage where available. Do not publish keys,
private transcripts, endpoints or personal paths. Never claim an untested feature works.

`[ ]` means pending; `[x]` means the listed user flow passed. A failure remains
unchecked until repaired and retested. Record app defects separately from model
limitations, unavailable prerequisites and automation restrictions. Backend probes
can diagnose a problem but cannot replace an actual UX pass. Destructive tests use
disposable data; remote Git tests use a local bare repository. No public push,
publication or unrelated system change belongs to this audit.

Each result records: checklist ID, model/build, visible route, action, expected vs
observed result, recovery, evidence, fix/retest if needed, and remaining limitation.
Each completed fix is a scoped commit with focused regression coverage. This file
tracks the audit; product design stays in the feature specifications.

## 1. Everyday chat and navigation

- [x] C01 Discover Home, Scheduled, Folders, Source Control, Terminal and bottom Settings from the compact rail.
- [x] C02 Create a temporary chat; first send creates a working folder and a visible real-model reply.
- [x] C03 Open a project folder, create a project chat and discover its working-folder identity.
- [x] C04 Create a side chat; converse without project/file tools.
- [x] C05 Select Qwen, change to DeepSeek and back; retain history and an unsent draft.
- [ ] C06 Compose multiline text and fenced code; Enter/Shift+Enter/Ctrl+Enter behave as presented.
- [ ] C07 Read streaming progress/Thinking and final Markdown/code; copy a message and code block accurately.
- [x] C08 Stop a real response; retained progress and subsequent new request remain usable.
- [x] C09 Navigate away and back with an unsent draft; restart retains settled draft and history.
- [ ] C10 Hide/show and resize the side panel with the title-bar control, drag and keyboard alternatives.
- [x] C11 Switch project conversations; developer pages follow the selected project with no dropdown.
- [ ] C12 Browse older/newer/latest messages and conversations without losing selection/draft.
- [x] C13 Export a test conversation as Markdown and JSON; inspect actual saved contents.
- [x] C14 Branch from a completed turn; branch retains context and uses the same files with fresh permissions.
- [x] C15 Record Worked/Needs work feedback and inspect the retained note.
- [ ] C16 Delete only a disposable conversation; verify its working files are retained.

## 2. Everyday agent work and recovery

- [x] A01 Ask the model to list/read a small project and explain actual contents.
- [x] A02 Request a new file; exact creation review is understandable and the saved file matches the receipt.
- [x] A03 Request an existing-file edit; inspect/apply the diff and verify only the intended saved bytes changed.
- [ ] A04 Deny an operation; model receives the denial and explains a useful next step without claiming success.
- [x] A05 Change a file after an edit proposal; stale apply refuses without overwriting newer work, then fresh review succeeds.
- [x] A06 Run a harmless project check through its exact command review; inspect output and return code.
- [x] A07 Cause a real check failure; Repair and verify fixes it and reruns the same check with truthful evidence.
- [x] A08 Inspect Changes and revert a disposable edit with the reviewed reverse diff.
- [ ] A09 Inspect Activity/Earlier tasks, settings origins, tool receipts, checkpoints and reported usage.
- [ ] A10 Hit a small explicit task/output limit; Continue resumes useful work with fresh approval.
- [ ] A11 Interrupt a task with completed effects; recovery draft explains uncertainty and does not replay earlier operations.
- [ ] A12 Read a large text file by ranges and edit a unique snapshot-bound match without truncation corruption.
- [ ] A13 Capture a larger command log; inspect its saved file and distinguish preview truncation from capture limits.

## 3. Files and editor

- [x] F01 Discover file tree, expand folders and preview a file; no selected project offers Open folder.
- [ ] F02 Preview versus pinned tabs: click, double-click and first edit preserve the expected file.
- [ ] F03 Edit text, undo/redo and Ctrl+S; UTF-8/BOM/CRLF content survives correctly.
- [ ] F04 Ctrl+P, Ctrl+F/H and Ctrl+G work with relative paths, find/replace and line navigation.
- [ ] F05 Drag tabs to reorder, split and move between groups; keyboard/menu alternatives work.
- [ ] F06 Shared file text/undo and independent cursor/scroll survive two split views and compact layout.
- [ ] F07 Close a dirty file: Save/Discard/Keep editing preserve the chosen outcome.
- [ ] F08 Create/Save as/rename a disposable file; an existing target is not silently overwritten.
- [ ] F09 External disk edit produces Compare/Reload disk/Keep my edits; stale save preserves both versions.
- [ ] F10 Restart with settled dirty buffers; private recovery preserves edits without writing source files.
- [ ] F11 Binary/oversize/long-line files offer clear readable recovery instead of broken editing.
- [ ] F12 Attach an unsaved editor selection to a chosen chat draft; no send until requested.
- [ ] F13 Model file reads see saved bytes while unsaved editor changes remain private unless attached.

## 4. Source Control

- [x] G01 Open a selected repository; status/branch, Working Changes and Staged Changes are discoverable.
- [x] G02 Stage/unstage one file with +/-; saved Git index agrees immediately.
- [x] G03 Stage/unstage all with section +/-; include untracked, renamed and Unicode/space filenames.
- [x] G04 Open working/staged inline diffs; additions/deletions and old/new line numbers are correct.
- [x] G05 Side-by-side diff is visibly different, aligned and scrolls both sides together.
- [x] G06 Stage/unstage selected hunks; unrelated hunks remain unchanged.
- [ ] G07 Commit message/Ctrl+Enter and Changes menu produce a clear review and a real commit.
- [x] G08 Empty-index Commit reviews working changes; Cancel leaves index unchanged and confirm commits them.
- [x] G09 Existing staged selection takes priority over other working changes.
- [x] G10 Failed Git hook retains message/index and gives usable retry after the hook is corrected.
- [ ] G11 Dirty editor blocks conflicting Git action without losing its text.
- [x] G12 Click a history commit to expand files; click a file for exact commit diff and collapse again.
- [ ] G13 Browse older history and several diff tabs; selections and split views remain usable.
- [ ] G14 Large text diff is rendered/pageable inside Dolores; stale page offers Refresh without losing the readable page.
- [ ] G15 Binary diff and nonrepository folder show distinct understandable states.
- [ ] G16 Changes menu branch creation/switching works; dirty branch switch refuses safely.
- [ ] G17 Stash/apply and selected discard work on disposable changes; conflicts retain stash/work and explain recovery.
- [ ] G18 Revert a disposable commit; conflict/Abort recovery does not discard unrelated work.
- [ ] G19 Fetch/Pull/Push against a local bare remote: explicit review, tracking and fast-forward behavior are correct.
- [ ] G20 Divergent Pull, changed refs and interrupted remote action retain evidence and require reconciliation.

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
- [ ] S04 Enabled project skill is pinned to the created task; project/model/skill receipt matches the request.
- [x] S05 Scheduled page shows next occurrence/timezone/status and original conversation; no manual creation form.
- [x] S06 Run now completes a real-model occurrence and exposes result, progress, error and usage.
- [x] S07 Change time/pause/resume in original chat; existing task changes without an accidental duplicate.
- [ ] S08 Page Pause/Resume/Skip next/Stop operate correctly on a disposable schedule.
- [ ] S09 A real timed occurrence runs while open; approval-needed status is visible and recoverable.
- [ ] S10 Interrupted/failed occurrence keeps evidence; explicit retry does not replay uncertain effects.
- [ ] S11 Background close-to-tray/reopen/Quit preserves task result and draft with no duplicate owner.
- [ ] S12 Delete only a disposable schedule; original conversation/results and files remain inspectable as specified.

## 7. Settings, attachments and context

- [ ] P01 Find settings by category/search; short labels/help hints and bottom placement remain clear.
- [x] P02 Light/Dark/System apply consistently; pending form drafts survive navigation.
- [ ] P03 Model context/output/stall/reasoning settings save for the selected model and survive restart.
- [ ] P04 Scoped user/project/chat overrides and inheritance show correct effective values on new runs.
- [ ] P05 Close/Escape with unsaved settings gives Save/Discard/Keep editing; cancel preserves values.
- [ ] P06 Connection controls list/manual models, selected model and remembered restart work without exposing a key.
- [ ] P07 Attach/preview/remove/send public UTF-8 text; attachment snapshot survives later source changes.
- [ ] P08 Attach public PNG/JPEG and send to a capable model; incompatible-image error preserves draft and offers settings recovery.
- [ ] P09 Over-limit/unsupported attachments give actionable rejection; attachment export contains matching manifest/snapshots.
- [ ] P10 Context ring shows preparation/estimates vs reported usage without an unintended request.
- [ ] P11 Review/save/edit session summary; later request recalls it while full history remains accessible.
- [ ] P12 Automatic compaction with a small context preserves draft and gives useful failure recovery.
- [ ] P13 Review-every-operation and selected/full-access grants affect only intended chat; revoke/expiry retains effects/evidence.
- [ ] P14 Multiple Window availability and wakefulness experimental setting are concise and preserve preference.

## 8. Memory, instructions and skills

- [ ] M01 One Memory switch starts automatic useful fact/preference capture from real completed chats, with sources/activity.
- [ ] M02 Fresh conversation recalls a retained useful fact; no manual population is required.
- [ ] M03 Inspect/correct a memory; protected correction survives later extraction.
- [ ] M04 Disable capture/recall; existing memory remains inspectable and can be forgotten deliberately.
- [ ] M05 Project facts capture approved successful check evidence with clear declaration/execution distinction.
- [ ] M06 Changed/missing/stale sources are excluded; project facts stay in the appropriate project.
- [ ] M07 Review/enable root AGENTS.md; changed/missing file blocks reuse and offers re-review/disable.
- [ ] M08 Review/activate project skill; use it in a real task and show selected skill provenance.
- [ ] M09 Global/project precedence, changed-source re-review, disable/rollback/export operate on disposable skills.
- [ ] M10 Draft skill from completed chat, edit/test it and activate only a complete strict improvement.

## 9. Optional research and integrations

- [ ] I01 Web search returns labeled source results or a truthful empty/error outcome; refine/direct URL recovery works.
- [ ] I02 Read a public primary page with citations; partial excerpt and provenance are visible.
- [ ] I03 Browser readiness/setup status is understandable; missing optional adapter has actionable recovery.
- [ ] I04 Model opens a disposable local web page, reads it and requests reviewed click/input; local screenshot is inspectable.
- [ ] I05 Browser Stop/stale reference recovery closes owned browser and avoids blind repeated input.
- [ ] I06 Inspect/enable a harmless local MCP server; catalog discovery does not silently execute tools.
- [ ] I07 Real-model invocation of enabled external tool shows exact review/receipt; missing/stale server recovers usefully.
- [ ] I08 Attach a disposable Windows window for view-only inspection with a capable model; target/sharing are explicit.
- [ ] I09 Selected-window control requires fresh state and verification; Stop/closed target never switches silently.
- [ ] I10 Model-directed delegation shows child progress/results and consumes the shared budget correctly.

## 10. Occasional and experimental workflows

- [ ] E01 Opt-in companionship model/hours and Quiet–Chatty frequency (0–100/day) produce a bounded in-app note when eligible, quiet during active work.
- [ ] E02 Open/Not now/Dismiss/Fewer messages/Turn off behave clearly without triggering external actions.
- [ ] E03 Compare instructions retains exact cases/usage and never activates a tie or incomplete result.
- [ ] E04 Skill testing has fixed disposable cases and truthful candidate/baseline results with Stop recovery.
- [ ] E05 Project reflection/automatic activation/pause retain source evidence and only activate complete independent gain.
- [ ] E06 Restore/quarantine learning baseline; manual edits conflict instead of silently overwriting work.
- [ ] E07 Harness extensions test source, bounded model draft and strict-gain activation preserve incomplete candidate source.
- [ ] E08 Recovery mod health rollback and restart reconciliation retain baseline and never replay an interrupted action.
- [ ] E09 Dolores capabilities/local diagnostic views explain build/tools/limits without an unintended model request.
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
