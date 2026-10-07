# Dolores acceptance — 2026-10-07

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

The user's final reference is implemented within milestone 17: Changes owns
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

The user reports gaming during the old slow samples, directs moving on and restores
one milestone per batch. Host contention is plausible, not proven; the old reports
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

The user finalized developer-pages **B**: a top-left title-bar sidebar-layout
toggle, drag-resize the page panel to hidden, no developer-page project dropdown,
selected project derived from the chosen Home conversation, and direct Terminal
entry at that project root or OS home. Existing terminals keep their CWD on return.
Settings remains at the bottom of the compact rail; Home remains the agent view.
The prototype is intentionally unchanged at the user's request. The page-direction
review gate is closed; the amendments are recorded for implementation and are not
claimed as tested runtime behavior. Editor adoption/16.0 completion remains open.

## Workspace/editor 16.0 — compact rail prototype and shared-view trial

The new page study preserves agent Home and file-only Folders splits. Following
the user's ChatGPT desktop screenshot, its primary navigation is a compact icon
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

The user reshaped milestones 16–19 and requested future memory, scheduling and
companionship work, implemented one brick at a time. The revised
[roadmap](ROADMAP.md) and [workspace contract](design/developer-workspace.md)
preserve Home's agent UI, define the six sidebar entries with Settings at the
bottom, and keep file and terminal splits separate. Source Control includes real
Git diffs; selected-project/root and no-project/home behavior are explicit.
Experimental multi-window has a bounded investigation and single-window fallback;
Windows keep-awake describes its actual platform limits.

The [new contract](design/memory-scheduling-companionship.md) assigns 21–24 to
automatic source-backed useful memory, chat-created schedules, opt-in in-app
companionship and optional closed-UI execution. The user's answers select one
Memory switch with inspect/forget, and companion hours plus a daily cap. Manual
schedule creation UI is deferred indefinitely. [Primary memory research](research/memory-foundations.md)
distinguishes scientific findings from proposed engineering behavior; selective
cue-led recall does not imply a complete human recording of sensations.

Repository guidance and handoffs now match the user's verification preference:
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

The user requested a new prototype preserving the current agent view and limiting
split tabs to files. Three synthetic variants compare files beside the agent,
below it, and file focus with the familiar agent on the right. The browser retains
agent drafts and unsaved file edits across variant changes and Files close/reopen.
Actual file-tab move and edge split work; a drop onto the agent is cancelled with
work retained. Wide/narrow rendering was inspected. These are prototype behaviors;
the production app and existing editor qualification gaps are unchanged. No layout
has been selected, no model call was made and 16.0 remains in progress. Captured
source and precise limits are in [qualification](qualification/workspace-editor.md).

## Workspace/editor 16.0 — revised prototype; editor adoption held

The user resumed milestones 16–19, one milestone batch at a time. The revised
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

The user approved installation/restart of the existing verified DeepSeek candidate
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
after a driver parsing failure. The user selected native-first preparation;
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

Brick **8.4 is skipped** at the user's request. The earlier forward proposal (milestones 9–12) was separated from the [implementation history](IMPLEMENTATION_HISTORY.md). That proposal is now superseded by the vision-based milestones 9–13 above; historical completed bricks retain their original scope. Documentation planning does not close existing platform/input/resource gaps. Runtime behavior and provider settings are unchanged.

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

The user authorized the next milestone batch. [Scheduling contract](design/chat-scheduling.md) freezes explicit intent, named-zone/DST/missed-run policy, bounded occurrence ownership, dedicated smaller budgets and the seven-case qualification corpus. This brick changes documentation only; scheduling is not yet available. Local links and whitespace were checked.

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

The user authorized continued implementation through the current roadmap, with
one milestone per batch and separate brick commits. [Companion contract](design/companionship.md)
freezes Off, chosen model/hours/cap, presence/quiet gates, random opportunities,
single unread message, source revalidation and bounded generation before runtime
changes. Defaults never activate companionship in the user's original profile.
Document/link checks pass; subjective welcome needs later user feedback.
