# Next-brick handoff — developer workspace and useful continuity

**Current revision — 2026-10-07. Planning only.** Read the [current handoff](../HANDOFF.md)
for delivered work and the [roadmap](../ROADMAP.md) for order. The user reshaped
16–19 and added 21–24. Implement one brick at a time; the earlier milestone-batch
instruction and mixed chat/file four-pane arrangement are superseded.

## Product intent

Keep Dolores's current agent UI on Home, including the session/project side panel
and main chat. The compact primary icon rail follows the user's ChatGPT desktop
screenshot: Home, Scheduled, Folders, Source Control and Terminal; Settings remains
at the bottom. Use hover/focus names and semantic labels, with the wider page panel
beside the rail and one brand there. Each page has suitable
secondary content. Folders has a project file tree and VS Code-style draggable
file tabs/splits; chats never become editor tabs. Source Control shows selected
project changes/history/diffs. Terminal has separate tabs/splits: plus starts at
the current project root or OS user home if none is selected, without retargeting
existing processes. Keep familiar icons, the infinity brand, palette and compact
behavior. Follow the [workspace contract](developer-workspace.md).

Experimental Multiple Window is default On when supported, with a bounded backend
trial and usable single-window fallback. Detached developer views first; do not
spend an indefinite milestone on experimental Flutter windowing. Windows keep-awake
is separately default Off, with actual screen-saver/security-policy limits shown.
First language families remain TypeScript/JavaScript and Rust.

The [memory/scheduling contract](memory-scheduling-companionship.md) plans:

- **21:** one-switch automatic useful facts/decisions and source-backed episode
  recall, with inspect/forget. No manual population/setup. Audit existing narrow
  explicit-preference capture before attributing the user's observed lack of updates.
- **22:** explicit natural-language task creation with a receipt and management-only
  Scheduled page. No manual Create task form; deferred indefinitely. Existing tool
  grants still apply. Initial availability requires the scheduler host to be alive.
- **23:** opt-in occasional in-app companionship during chosen hours, with a daily
  cap and separately configured weaker model. No focus stealing, fabricated memories
  or automated tool execution from a greeting.
- **24:** optional local scheduling worker while the UI is closed. Qualify ownership,
  power/network availability and cleanup separately; no promise to run while asleep.

Human-memory findings support selective, reconstructive, cue-driven recall rather
than recording every sensation. See [research](../research/memory-foundations.md).
Proposed numerical defaults require pre-implementation freezing and measurement.

## Technical starting point and open gates

The implemented stack is Flutter plus bundled Rust/typed plugins, SQLite and native
credentials. Current UI selects one ChatController session and the Rust Engine has
single-active-run assumptions. Shared run/document/Git/PTY/LSP ownership is a real
16.1 prerequisite; windows own views/subscriptions, never independent profile owners.
Navigation must not retarget a run, discard a dirty buffer or kill an existing PTY.

Candidate editor/terminal/window libraries are researched options, not installed
production dependencies. Fresh isolated typing now passes without a performance
fix or explanation of the earlier 94 ms failure. Six controller/widget checks
cover shortcuts/shared-view undo, but four-view reopening after large files fails
at 389.306 ms versus the unchanged 250 ms target. Physical input, byte-safe Save
and production protocol/resource gates remain open. See [qualification](../qualification/workspace-editor.md). Older
prototypes are evidence, not approval of the revised separate-page UI.

[Remaining work](../qualification/remaining-work.md) retains broad model, learning,
computer-use, idle-resource, platform/accessibility gates. Milestone 20's reviewed
Windows repair cycle is bounded evidence, not broad autonomous competence. Do not
replay unrelated paused work or silently raise limits to disguise failures.

## Work and verification rules

- Read applicable AGENTS.md/RTK guidance; use RTK commands and CodeGraph first for
  indexed source. Follow [UI](../UI.md).
- Start with **16.0: revised page prototype and editor feasibility**. Review this
  concrete UI direction before production changes; do not promote a failing trial.
- Keep work to the authorized brick; commit completed bricks in English with hooks.
  Do not push without explicit instruction. Platform CI 8.4 stays deferred.
- Verify basic flow and one or two realistic failures/recovery when applicable;
  record actual results/gaps in [acceptance](../ACCEPTANCE.md).
- No build/launch/visual check after every task. Documents need link/document checks;
  runtime work needs focused tests and a normal build when compilation/packaging is
  affected. Prefer saved UI renders with `view_image`; use computer-use only for
  interactions/native behavior that need it. Launch for relevant integration checks
  or on request, through Python desktop helpers, preserving configuration/history.
- Routine model probes use configured Qwen3.5-2B; harder cases use DeepSeek V4.1 Flash.
  Bound usage, preserve selected settings and keep private data in ignored output.

The original preview profile is `output/model-picker/preview/data`, with ownership
record `output/maintained-preview.json`; verify current process identity before
replacing anything. Do not change this profile for planning. No fixed dates or
full VS Code/Meta Muse parity have been promised.
