# Desktop interaction requirements

Use this specification when changing workspace navigation, settings placement,
conversation actions or retained evidence. Shared visual and wording guidance
lives in [UI.md](../UI.md); this file owns feature-specific interaction rules.

## Workspace and settings

- Keep **Attention** reachable across workspace pages. Windows notifications are
  opt-in and carry generic status plus an opaque local target; show task titles
  only inside Dolores. Distinguish unread/seen from accepted, suppressed, failed
  or uncertain delivery. Opening a notice navigates to the exact current task or
  its retained run evidence and never answers or approves. Defer a native click
  while a modal has unsaved work. Keep errors and Refresh reachable in the fixed
  Attention footer, with preferences and event identifiers disclosed on demand.

- Offer **Search conversations** in the chat sidebar and Ctrl+K when no modal is
  open. Search saved text locally, show bounded excerpts and exact conversation
  actions, and keep additional workspace/pin filters collapsed initially. Pinning
  orders search results first. Archive/unarchive changes organization only;
  active owners, drafts, goals and queued input remain available. Keep search
  errors and recovery actions in the fixed footer, retain previous results on
  failure, and replace result pages rather than accumulating all history.

- Put Home, Scheduled, Files, Source Control, Terminal and Settings in
  one centered floating bottom dock. Overlay it without reserving workspace
  height or resizing content. Hide after inactivity/pointer departure; reveal at
  the bottom edge, by tapping its handle or with Ctrl+Alt+D. Keep it visible while
  hovered or navigating it by keyboard. Escape dismisses it. Hidden controls must
  not intercept input. Use icon-and-label items at wide widths and icons with
  hover/focus names and semantic labels at compact widths.
- Keep the infinity mark and Dolores name in the shared title bar on every page.
  Place the sidebar toggle immediately right of the title. Share one centered
  header layout across native and fallback frames: 18 px logo, compact title
  line height, 16 px toggle glyph inside a 32 px click target. Use a thin rounded
  window outline with a narrow left pane for its icon;
  preserve its tooltip, semantic name, native dragging and window controls.
- Keep a named Settings gear immediately left of Minimize in the custom Windows
  title bar, available while the navigation dock is hidden. Open the existing
  General/Theme settings window, avoid stacking it over an open review and restore
  keyboard focus when it closes. The dock reveal's accessible name identifies the
  navigation dock and its destinations so it cannot be mistaken for a sidebar toggle.
- Reserve the sidebar for the current feature, starting near the top: project
  chats and recents on Home, files on Files, repository controls on Source Control,
  and relevant task/terminal controls on their pages. Home centers a short
  invitation and composer, followed by useful projects and recent work.
- Group related controls near the content they affect. Prefer one contextual
  overflow menu to scattered secondary actions or permanent action bars.
- Saved message selections offer **Ask about this** and, in linked side chats,
  **Bring to parent task**. Review an editable excerpt in a bounded dialog before
  opening or sharing. Opening a side chat or returning to its parent sends
  nothing. Keep parent navigation and the frozen excerpt visible in the side
  chat; disclose return receipts and source details on demand. An unavailable
  parent retains the excerpt and explains why return is disabled.
- Saved user messages offer **Edit in a new branch**. Preview the edited text,
  optional prompt attachments/context and explicit folder choice before opening
  an unsent independent draft. Keep original-chat navigation and a paged receipt
  archive available in that branch. Explain that files remain current and earlier
  actions are historical. Stale previews retain edits for refresh; failed saves
  retain their identity for retry without overwriting a newer branch draft.
  The managed-worktree choice reuses explicit base review and creation, then
  returns an empty Ready target to the editor without sending or navigating.
  Show the selected folder and reviewed commit. Explain that source staging and
  dirty files stay in the source checkout and that a failed attachment retains
  the worktree for inspection and same-target retry.
- Choose the smallest surface that comfortably fits the task:

  | Task | Surface |
  | --- | --- |
  | Composer attachment, model or access choice | Compact dropdown anchored to its trigger; current choice and brief decision context |
  | General, Models, Memory, Access, Connections or Advanced settings | One centered floating window with category navigation and one content area |
  | Task review, activity or substantial workspace tool | Contextual side panel with resize/collapse controls and keyboard alternatives |

- Route broader configuration through an explicit dropdown link to the relevant
  settings category. Keep short choices in dropdowns rather than side panels.
- Keep Personalization and Companionship reachable under General, permission
  controls under Access, and tool connections under Connections. Open skill
  management through Settings. Retain named navigation to Files, Source Control,
  Terminal and Scheduled tasks alongside Home.
- Connections keeps Local program and Remote endpoint setup in the same bounded
  inspector, with fixed actions and reachable Stop. Keep bearer values masked and
  separate from names/URLs. Loading saved data does not contact a server or claim
  current health; Inspect/Recheck displays the latest inspection result. Filter
  reviewed tools locally, retain selections across filtering and keep the selected
  count in Enable. Failed inspection/saving retains the usable saved connection
  and an actionable error. Remote approvals display the exact endpoint and
  arguments without claiming a local program launch. See the
  [MCP contract](mcp-connection.md).
- Dropdowns stay within the viewport and flip above their trigger near the bottom
  edge. They dismiss on outside interaction or Escape, support arrow keys and
  Enter, and return focus to the trigger after selection or Escape. Keep the
  surrounding workspace usable.
- The settings window has a clear close action, bounded scrolling, keyboard focus
  containment and focus restoration. Category changes update its content in place.
- Preserve panel contents when hiding or resizing. At compact widths, wrap
  controls and use drawers or section selectors for substantial workspace tools.
- At compact widths, let settings occupy the available viewport and adapt its
  category navigation. Keep short dropdowns anchored and bounded. Use a scrolling
  body with a reachable, wrapping action footer for long forms and reviews.
  Keep primary actions reachable without clipping or layout jumps. Reuse the
  settings window or relevant inspector for deeper forms.


## Conversation and evidence

- Run details lead with model and per-call input/output/total metrics. Put
  token breakdowns, request settings and context sources in expandable sections.
  Keep local context estimates separate from provider usage; never infer missing
  counts or a run total from partial per-call data.
- Interleave public agent progress and compact tool rows in the conversation,
  followed by the final answer. Expand a row for receipts, screenshots or diffs;
  keep provider thinking separate. Preserve saved ordering when available and
  identify older activity whose exact interleaving was not recorded.
- In conversations, the first deliberate upward scroll stops following new
  output, including small wheel/trackpad movements near the bottom. Streaming,
  late rendering and status updates preserve that reading position. Returning
  to the bottom or choosing the latest page resumes following.

## Feature contracts

- **Managed worktrees:** expose Worktrees from chat actions and the working-folder
  menu. Require an explicit base and a review of the source, destination and exact
  commit before creation. Show setup choices without running repository scripts.
  Keep the source draft and use a separate Open chat action. Retained records and
  recovery paths remain inspectable even when their chats are absent from recent
  history. Status checks never repeat creation; failures bring their recovery
  message into view. Transfer controls compare saved files, require selection and
  an explicit destination/patch review, and keep Apply disabled on conflict.
  Expose retained versions and a copyable recovery folder; status polling or
  reopening an uncertain receipt never repeats publication. See
  [EH06](../plans/everyday-harness/eh06.md) for lifecycle gates.
  Archive and Restore use separate reviews showing the owned folder, original
  folder kept, snapshot identity and retained archive path. Save the source draft
  before applying either action. Interrupted lifecycle work exposes inspection
  and explicit continuation; it never automatically removes or restores files.

Read the matching specification when changing:

- **Workspace navigation, file tabs or splits:** [Developer workspace](developer-workspace.md).
- **Source Control or Git diffs:** [Source Control](source-control.md).
  Provide accessible line comments and retained findings with separate Queue,
  Steer and verified-resolution actions. Keep original excerpts readable when
  locations change; status recovery must never duplicate a repair instruction.
- **Terminal or language services:** [Terminal and language services](terminal-language-services.md).
- **Conversation or settings organization:** [UX simplification](ux-simplification.md).
- **Automatic memory:** [Automatic memory](automatic-memory.md).

For other feature behavior and limits, read the corresponding contract under
[design specifications](./).
