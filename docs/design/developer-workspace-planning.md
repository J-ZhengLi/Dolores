# Historical developer workspace planning

This planning note is superseded by the [roadmap](../ROADMAP.md),
[workspace specification](developer-workspace.md) and
[qualification](../qualification/workspace-editor.md). It is retained as a design
reference, not a session handoff or instruction to start a particular milestone.

The adopted layout keeps chat and conversation selection on Home. A compact icon
rail opens Scheduled, Folders, Source Control and Terminal, with Settings at the
bottom. The title-bar button toggles a resizable side panel. The selected Home
conversation supplies the project; developer pages have no project dropdown.
Folders uses file-only draggable tabs and splits. New terminals start at the
selected project root or OS home and retain their existing working directories.

Shared run, document, repository and terminal ownership prevents navigation from
retargeting work or losing unsaved content. Experimental detached windows require
their own performance gate and retain a usable single-window fallback.

The continuity features are automatic useful memory with inspect/forget,
chat-created scheduled tasks with a management page, optional in-app companionship,
and optional Windows tray scheduling. Their implemented scopes and remaining
limits belong in the roadmap and qualification reports.

Contributor verification follows [AGENTS.md](../../AGENTS.md) and
[CONTRIBUTING.md](../../CONTRIBUTING.md). Local profiles, captures and session notes
stay untracked. Optional live tests explicitly select an available model; fixture
checks require no provider credentials.
