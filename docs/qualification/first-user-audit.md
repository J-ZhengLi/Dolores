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

- [ ] C01 Discover Home, Scheduled, Folders, Source Control, Terminal and bottom Settings from the compact rail.
- [x] C02 Create a temporary chat; first send creates a working folder and a visible real-model reply.
- [x] C03 Open a project folder, create a project chat and discover its working-folder identity.
- [ ] C04 Create a side chat; converse without project/file tools.
- [ ] C05 Select Qwen, change to DeepSeek and back; retain history and an unsent draft.
- [ ] C06 Compose multiline text and fenced code; Enter/Shift+Enter/Ctrl+Enter behave as presented.
- [ ] C07 Read streaming progress/Thinking and final Markdown/code; copy a message and code block accurately.
- [ ] C08 Stop a real response; retained progress and subsequent new request remain usable.
- [ ] C09 Navigate away and back with an unsent draft; restart retains settled draft and history.
- [ ] C10 Hide/show and resize the side panel with the title-bar control, drag and keyboard alternatives.
- [ ] C11 Switch project conversations; developer pages follow the selected project with no dropdown.
- [ ] C12 Browse older/newer/latest messages and conversations without losing selection/draft.
- [ ] C13 Export a test conversation as Markdown and JSON; inspect actual saved contents.
- [ ] C14 Branch from a completed turn; branch retains context and uses the same files with fresh permissions.
- [ ] C15 Record Worked/Needs work feedback and inspect the retained note.
- [ ] C16 Delete only a disposable conversation; verify its working files are retained.

## 2. Everyday agent work and recovery

- [x] A01 Ask the model to list/read a small project and explain actual contents.
- [ ] A02 Request a new file; exact creation review is understandable and the saved file matches the receipt.
- [ ] A03 Request an existing-file edit; inspect/apply the diff and verify only the intended saved bytes changed.
- [ ] A04 Deny an operation; model receives the denial and explains a useful next step without claiming success.
- [ ] A05 Change a file after an edit proposal; stale apply refuses without overwriting newer work, then fresh review succeeds.
- [ ] A06 Run a harmless project check through its exact command review; inspect output and return code.
- [ ] A07 Cause a real check failure; Repair and verify fixes it and reruns the same check with truthful evidence.
- [ ] A08 Inspect Changes and revert a disposable edit with the reviewed reverse diff.
- [ ] A09 Inspect Activity/Earlier tasks, settings origins, tool receipts, checkpoints and reported usage.
- [ ] A10 Hit a small explicit task/output limit; Continue resumes useful work with fresh approval.
- [ ] A11 Interrupt a task with completed effects; recovery draft explains uncertainty and does not replay earlier operations.
- [ ] A12 Read a large text file by ranges and edit a unique snapshot-bound match without truncation corruption.
- [ ] A13 Capture a larger command log; inspect its saved file and distinguish preview truncation from capture limits.

## 3. Files and editor

- [ ] F01 Discover file tree, expand folders and preview a file; no selected project offers Open folder.
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
- [ ] G03 Stage/unstage all with section +/-; include untracked, renamed and Unicode/space filenames.
- [x] G04 Open working/staged inline diffs; additions/deletions and old/new line numbers are correct.
- [ ] G05 Side-by-side diff is visibly different, aligned and scrolls both sides together.
- [ ] G06 Stage/unstage selected hunks; unrelated hunks remain unchanged.
- [ ] G07 Commit message/Ctrl+Enter and Changes menu produce a clear review and a real commit.
- [ ] G08 Empty-index Commit reviews working changes; Cancel leaves index unchanged and confirm commits them.
- [ ] G09 Existing staged selection takes priority over other working changes.
- [ ] G10 Failed Git hook retains message/index and gives usable retry after the hook is corrected.
- [ ] G11 Dirty editor blocks conflicting Git action without losing its text.
- [ ] G12 Click a history commit to expand files; click a file for exact commit diff and collapse again.
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
- [ ] T03 Plus opens another shell at current project; existing shell retains its own CWD after navigation.
- [ ] T04 Tabs reorder/move/split; menu alternatives and compact group selection work.
- [ ] T05 Select/copy text; Ctrl+C copies selection or interrupts foreground work when no selection exists.
- [ ] T06 Stop/close live shell gives the expected review and keeps readable output.
- [ ] T07 Restart retains stopped output; Start creates a fresh shell without replaying commands.
- [ ] T08 Attach selected output to a chosen chat draft without automatically sharing full scrollback or sending.

## 6. Scheduled tasks

- [ ] S01 Natural English weekday request creates a task without manual setup and visibly confirms exact schedule/model.
- [ ] S02 Natural Chinese weekday request does the same; paraphrases such as each workday are supported by interpretation.
- [ ] S03 Missing time uses disclosed 09:00 local default; quoted examples/negations create no task.
- [ ] S04 Enabled project skill is pinned to the created task; project/model/skill receipt matches the request.
- [ ] S05 Scheduled page shows next occurrence/timezone/status and original conversation; no manual creation form.
- [ ] S06 Run now completes a real-model occurrence and exposes result, progress, error and usage.
- [ ] S07 Change time/pause/resume in original chat; existing task changes without an accidental duplicate.
- [ ] S08 Page Pause/Resume/Skip next/Stop operate correctly on a disposable schedule.
- [ ] S09 A real timed occurrence runs while open; approval-needed status is visible and recoverable.
- [ ] S10 Interrupted/failed occurrence keeps evidence; explicit retry does not replay uncertain effects.
- [ ] S11 Background close-to-tray/reopen/Quit preserves task result and draft with no duplicate owner.
- [ ] S12 Delete only a disposable schedule; original conversation/results and files remain inspectable as specified.

## 7. Settings, attachments and context

- [ ] P01 Find settings by category/search; short labels/help hints and bottom placement remain clear.
- [ ] P02 Light/Dark/System apply consistently; pending form drafts survive navigation.
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

- [ ] E01 Opt-in companionship model/hours/cap produce a bounded in-app note when eligible, quiet during active work.
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

Build: normal Windows main entry at `a235b7d`. Isolated configuration-only profile;
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

### Open observations

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
