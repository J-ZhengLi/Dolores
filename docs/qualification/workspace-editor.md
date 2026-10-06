# Workspace/editor 16.0 — partial feasibility evidence

## Proceed decision and initial envelope — 2026-10-07

The user reports playing a video game around the old slow samples and explicitly
directs moving on. Host contention is plausible, not established. Those reports
remain below; they no longer block adoption alone. Performance targets are unchanged.
A new native 1 MiB four-view editing case passes 40 replacements at **19.558 ms**
typing frame p95, with shared buffer/undo/redo and bounded history. Short matched
incremental peaks are 63.957 MiB working / 59.996 MiB private. The initial editable
limit is 1 MiB, with very long lines and larger/unsupported files safely previewed.
See [frozen document protocol and bounds](../design/editor-documents.md).

The user restores **one milestone per batch**. Continue milestone 16's ownership,
navigation, safe files and editor integration with separate brick commits; do not
stop after each brick or begin 17 in this batch. Native physical input/IME and
normal-app save/resource qualification remain explicit integration gaps, not claims
made by the scratch test. `20261007-editor-limits.json` and its memory samples live
in the ignored trial output alongside the retained earlier reports.

## Final layout decision — 2026-10-07

The user selects **developer-pages B**, amended as follows: top-left title-bar
sidebar-layout toggle (not Close/folder), resize the adjacent panel down to hidden,
no project dropdowns, and project selection derived from the chosen Home conversation.
Terminal enters its view directly, with initial cwd at project root or OS home and
existing shells reused. No further prototype change is requested. The page-direction
review gate is closed; the unmodified prototype does not verify these amendments.
Editor qualification remains separate and open. See the amended [contract](../design/developer-workspace.md).

### Reopening diagnosis follow-up

The original complete native loop was rerun before changing the diagnostic
instrumentation: one/two/four-view post-corpus ordinary opens were
31.553 / 76.192 / 136.258 ms; maximum typing frame p95 was 4.620 ms. Its existing
250 ms opening, 32 ms typing and provisional memory checks pass. This is a
non-reproduction of the earlier 389.306 ms failure, not a performance fix.

A focused native loop then repeats the four-view 1 MiB → 100 KiB line → ordinary
sequence twelve times, separating document/controller preparation and the original
two awaited frames from prior-view disposal. Two fresh-process runs pass all
24 ordinary openings; maxima are **144.306 / 136.098 ms** against the unchanged
250 ms target. The first run's largest sample has 0.007 ms document preparation,
0.003 ms controller preparation, 74.623 / 69.673 ms awaited-frame phases and
0.006 ms prior-view disposal. These timings describe that passing sample only;
they do not identify the historical failure's cause.

Reproduction commands use `rtk proxy python
output/workspace-16-prototype/prepare_editor_spike.py benchmark --mode multiview
--report followup.json` for the original complete case, and `--mode reopen` for
the focused sequence. The focused mode returns failure if any ordinary open
reaches 250 ms. It is agent-runnable and red-capable; a repeatable red case has
not yet been obtained, so no causal fix or dependency adoption is claimed.
The unchanged six controller/widget checks pass again after instrumentation.

Public follow-up source/reports are on `codex/prototype-developer-pages-16`, commit
`1eb912dd1f83c04c07abf68e5b88b047ea2ac8fd`, under the same prototype directory.
Reports are `20261007-reopen-reproduction.json`, its phase-separated memory file,
`20261007-reopen-focused.json` and `20261007-reopen-focused-repeat.json` in `reports/`.
The capture verifies the approved HTML blob is unchanged. The native scratch build
uses the maintained Python machinery; no normal frontend/profile was replaced.
The historical typing/opening discrepancies, physical input/IME/accessibility,
byte-safe Save and production protocol/resource gates remain open. 16.0 remains
in progress; 16.1 has not started.

## Current page prototype — compact icon rail, 2026-10-07

The user requested separate Home/Scheduled/Folders/Source Control/Terminal pages,
then supplied the ChatGPT desktop screenshot to select a compact icon rail rather
than the initial full-width navigation. The revised synthetic study uses a
64-pixel rail, 48-pixel icon targets, hover/focus names and semantic labels. Settings
stays at the bottom even at 420×480. The wider adjacent panel shows Home sessions
or the selected project's tree; brand appears once there. Home keeps the familiar
conversation/composer; only Folders has file editor tabs/splits.

Current review URL:
`http://127.0.0.1:8767/developer-pages-prototype.html?variant=A`.
A keeps the page panel visible, B folds it inline, C uses a drawer; all now use the
compact rail. This replaces the earlier full-width A/B/C navigation choices, and
the older mixed-pane and agent/file arrangements remain historical evidence.
Local primary source: `output/workspace-16-prototype/developer-pages-prototype.html`.
Primary source and public native trial reports are captured separately on
`codex/prototype-developer-pages-16` at commit
`adecd123ea708090de8e5533fcab3f7882291ed3`, under
`apps/dolores_flutter/prototypes/developer-pages-16/`. The normal branch contains
only contracts/evidence; the throwaway editor adapter is not promoted into runtime.
Screenshots: `compact-rail-home.png`, `compact-rail-files.png`, and the revised
`pages-*-home/files.png` in the same directory. The server is loopback-only.

Playwright browser walkthroughs pass navigation/draft retention, A/B buffer
separation, invalid-drop preservation, actual cross-group drag and edge split,
shared duplicate buffers, large/binary read-only previews, folder-picker cancel,
simulated terminal root/home defaults with existing CWD retained, no manual
schedule creation, Experimental defaults, light/dark, compact group access and
minimum-height Settings. Keyboard focus exposes a readable page name. Saved
renders were inspected using `view_image`; no native computer-use automation ran.
All content is synthetic/in-memory; no real filesystem, Git, PTY, scheduler, OS
keep-awake, model or profile action occurs. Reload resets the study.

### Additional isolated editor evidence

Six controller/widget checks pass on the unchanged pinned candidate: Unicode
insert/undo/redo, CRLF/BOM representation, 100 KiB-line undo, focused Ctrl+Z/Ctrl+F,
independent mounted views with shared shortcut undo, and bounded document history
after sibling closure. The throwaway adapter owns one document/history and one
controller per view, clearing widget histories; the same controller is not mounted
twice. It retains at most 64 history records and 4 Mi UTF-16 units. This proves
that seam for the tested cases, not production encoding/save/composition safety.

The unattended native release loop repeats the prior corpus without manual input.
Two fresh original-controller runs give typing frame p95 **7.210/6.749 ms**, passing
the unchanged 32 ms target without a performance fix. This does not explain the
historical 94 ms failure; its source and timing reports below are retained.

| Shared-document adapter | First typing frame p95 | Repeat typing frame p95 | Repeat ordinary reopen after large files |
| --- | ---: | ---: | ---: |
| One view | 3.515 ms | 4.783 ms | 77.651 ms |
| Two views | 4.432 ms | 4.820 ms | 141.305 ms |
| Four views | 4.344 ms | 6.267 ms | **389.306 ms — fails 250 ms target** |

All cases use one document, 40 programmatic edits, timestamp-filtered engine frames
and the same 1 MiB/100 KiB-line corpus. These are not physical typing/IME latency
or disk I/O. Four distinct documents, highlighting/Find under load and normal-app
bridge work are not measured. The first four-view reopen was 208.019 ms; the repeat
fails. Do not label this consistent opening performance or silently raise 250 ms.

The same repeat process has a shell baseline median 201.746 MiB working / 214.199
MiB private. Five-second ordinary phases are separate from subsequent corpus phases:

| Views of one document | Ordinary median working / private | Peak incremental working / private during corpus | Next-trial ceiling, each memory metric |
| --- | --- | --- | --- |
| One | 249.492 / 265.094 MiB | +86.875 / +92.855 MiB | +128 MiB |
| Two | 262.332 / 278.766 MiB | +119.871 / +133.793 MiB | +192 MiB |
| Four | 289.664 / 309.250 MiB | +188.992 / +209.426 MiB | +224 MiB |

Those provisional ceilings were set after the first sizing run and before the
repeat; all memory checks pass on the repeat. They apply to one shared document
on this scratch host, not a normal-app heap guarantee, sustained idle CPU check or
acceptance for multiple distinct documents. The full follow-up loop returns failure
because ordinary reopening exceeds its unchanged target.

**Held editor gates:** the historical
typing discrepancy and new four-view opening failure, native keyboard/selection/
clipboard/IME/accessibility, real byte-safe Save and production protocol/resource
bounds. The 5 MiB editable proposal remains unqualified; it was not silently adopted.
Freeze production snapshot/delta/file/resident budgets before adopting a dependency.
16.0 remains in progress; 16.1–16.6 have not started. Neither production source,
dependencies nor the real profile changed. Diagnostic entry `main` identifies the
scratch frontend only, not normal-app qualification. No live model request ran.

## Earlier prototype — preserve agent UI, split files only

The user is undecided about the earlier workspace layout and requested a prototype
that retains Dolores's current agent view and limits split tabs to files. The new
self-contained HTML uses the current palette, sidebar, conversation controls,
message/composer layout and bottom Settings with synthetic content. Three variants
share one route: A files beside the agent, B files below it, C file focus with the
familiar agent on the right. No variant is selected or implemented in production.

Review URL: `http://127.0.0.1:8766/agent-files-prototype.html?variant=B`. Local
source/screenshots: `output/workspace-16-prototype/agent-files-prototype.html` and
`agent-files-A.jpg`, `agent-files-B.jpg`, `agent-files-C.jpg` in the same directory.
Captured public source: `codex/prototype-agent-files`, commit
`04de922e31878eb1c86801fa84ffcbdfd3ef91bb`, path
`apps/dolores_flutter/prototypes/agent-files/`.

Browser checks exercised all three renderings, preserved a synthetic agent draft
and unsaved file edit while switching layouts and closing/reopening Files, moved
a file tab between groups, and created a third file-only group with an edge drop.
Dropping a file onto the agent refused the move and retained all work. Narrow
620-pixel rendering was inspected; file groups stack and the sidebar hides. This
does not qualify a production compact navigation drawer or native accessibility.
These are in-memory UI checks, with no disk writes or model calls. Reload/reset
clears the demo; it is not crash recovery. No formal tests are added for throwaway
UI. The earlier mixed chat/file four-pane layout is not approved. Native editor
performance/input/resource gates below remain unchanged and open.

2026-10-06. The user resumed milestones 16–19, one milestone batch at a time,
starting with 16. **16.0 is in progress; 16.1–16.5 have not started.** Neither the
revised layout nor the editor dependency is accepted for production rollout yet.

## Prototype and captured primary source

The first prototype was rejected for its pane action bars. The user's VS Code
screenshot prompted slim tab strips with close/overflow controls, breadcrumbs,
pointer reorder/move/edge-split gestures and contextual actions. Settings remains
at the bottom of the rail. Revised user review is pending.

The self-contained, synthetic HTML is retained locally at
`output/workspace-16-prototype/workspace-prototype.html`, served for review at
`http://127.0.0.1:8766/workspace-prototype.html`. A screenshot is in the same
ignored directory as `revised-layout.jpg`. The public sources and numerical
trial reports are captured outside the main branch:

- Branch: `codex/prototype-workspace-16`
- Commit: `7798985d871bbf35c3f8b18576d2a8e67ffd1027`
- Path: `apps/dolores_flutter/prototypes/workspace-16/`

Actual browser pointer input moved A's file into the chat group, then dropped it
at an edge to create a fifth pane. All four views retained their project identity.
An actual invalid drop outside the panes reported cancellation and retained all
four tabs, file buffers and chat drafts. The external-change walkthrough retained
an unsaved buffer and a different synthetic disk version; Save opened comparison
and Keep editing retained the buffer. Compact focus and restored wide layout were
exercised earlier. These establish prototype interaction only: no real runs,
filesystem writes, crash recovery or concurrent project isolation are implemented.
Within-strip reorder and every keyboard alternative remain separate checks.

## Native candidate and scope

Trial candidate: `re_editor 0.10.0`, with `re_highlight 0.0.3`, in an isolated
Flutter release frontend using the repository's Windows runner, palette and
Python build machinery. The maintained pubspec, app source and profile are
unchanged. The trial never opens the bridge/database, launches a terminal/LSP or
sends a model request. Dependencies are trial-only; production adoption is held.

Recorded host: Windows 11 build 26200, Intel Core i7-14650HX, 24 logical CPUs;
maintained Flutter 3.47.5 stable. Public corpus: 80-byte ordinary TypeScript with
Chinese/emoji, exactly 1 MiB multiline text, a 102,417-byte single-line case and
a 47-byte CRLF/BOM case. Opening measurements include assignment and two frame
completions; they do not establish disk I/O or completed asynchronous analysis.

Three controller probes pass: Unicode insertion/selection with undo/redo, BOM
representation and normalized line-ending comparison, and insertion/undo in a
100 KiB line. These are not widget, encoding-safe save or physical IME tests.
The visible native trial rendered line numbers and syntax highlighting. A native
ordinary character arrived at the focused editor and the app key counter advanced.
Ctrl+Z and Ctrl+F advanced the app key counter but did not visibly undo/open Find;
their integration is unqualified. Native selection, redo/find recovery, physical
IME, clipboard and screen-reader behavior remain open.

## Performance result and held adoption

Existing targets remain warm ordinary opening below 250 ms and typing frame p95
below 32 ms. No target was relaxed. The trial uses 40 programmatic inserts about
35 ms apart; the figures are engine frame durations, not physical input latency.
Frame batches are filtered by engine build-start time within the insertion
interval. The initial unfiltered report remains captured but is superseded as an
acceptance measurement because it included delayed frames from earlier work.

| Trial | Frames | Total frame p95 | Build p95 | Raster p95 | Result |
| --- | ---: | ---: | ---: | ---: | --- |
| Ordinary editor, typing only | 44 | 19.252 ms | 0.861 ms | 17.577 ms | Pass for this isolated control |
| Flutter TextField control | 40 | 3.626 ms | 0.930 ms | 1.451 ms | Control only |
| Editor after full corpus sequence | 36 | 94.340 ms | 1.811 ms | 47.833 ms | Fails typing target |
| Full sequence with large-file highlighting disabled and no code folding | 36 | 94.056 ms | 1.572 ms | 48.093 ms | Still fails typing target |

In the last full sequence, ordinary openings were 40.526/71.691/153.509 ms;
1 MiB opened in 105.965 ms, the long line in 82.640 ms and CRLF/BOM in 93.203 ms.
The scratch large-file policy did not resolve the typing failure. It is not a
production policy or evidence that full 5 MiB editing is safe. The root cause is
not established; retain the failure and investigate before adopting the candidate.

## Resource sample and ownership gap

Two 60-second samples used the same owned trial process, with no child processes:

| State | Median working / private memory | Peak working / private | CPU, one-core basis |
| --- | --- | --- | ---: |
| Baseline shell | 214.660 / 223.410 MiB | 219.004 / 228.906 MiB | 26.421% |
| One ordinary editor | 268.777 / 269.701 MiB | 285.090 / 290.496 MiB | 4.685% |

Median differences are +54.117 MiB working and +46.291 MiB private. Focus/background
and startup settling differed between samples, so the CPU comparison does not
show an editor CPU improvement or close the existing idle-CPU gap. This is one
scratch editor, not a matched normal-app one/two/four-pane qualification. A numeric
memory budget remains unfrozen; dependencies must not be adopted until it is
frozen and the relevant measurements pass.

The candidate controller binds one editor key. Sharing that controller between
duplicate widgets cannot establish independent cursor/selection state and shared
document undo safety. The production adapter must separate document buffer/history
ownership from focused-view binding and selection. Message bounds, actual
multi-view cost and save/revision/encoding recovery remain 16.0/16.3–16.4 gates.

## Next work and experimental settings

The amended B page direction is final. Resolve the performance/input failures, qualify
the document adapter and freeze ownership/bounds/resource decisions before the
next runtime brick. The two-project run registry in 16.1 remains required; tabs
alone cannot remove the current single-run restriction. Milestone 17 starts only
after 16's batch exit; platform CI 8.4 stays deferred.

The updated specification records bottom-rail Settings and an Experimental page
in 16.2. Multiple Window defaults On, with detach unavailable until the bounded
19 backend trial qualifies it. The requested Windows keep-awake option defaults
Off and must disclose that display/system requests do not prevent secure screen
savers, manual locking or enforced locks. Neither toggle is implemented here.
No OS security or power preference was changed by this feasibility work.
