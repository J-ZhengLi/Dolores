# Workspace/editor 16.0 — partial feasibility evidence

## Latest prototype — preserve agent UI, split files only

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

Finish revised prototype review, resolve the performance/input failures, qualify
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
