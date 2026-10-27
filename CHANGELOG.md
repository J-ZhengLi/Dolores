# Changelog

User-visible changes are recorded here. Unreleased work has no release date;
published versions use SemVer and an ISO date.

## [Unreleased]

### Changed

- The command and skill picker uses compact, padded rows with a clear keyboard selection. Skills can also be completed within a sentence while preserving surrounding text and their pinned reference.
- Project and global skill files are ready to use in chat and the `/` picker without prior review. File updates apply to future messages, with optional viewing, direct enable/disable and one-step create/import. Existing snapshots and tool permissions are preserved.

### Fixed

- Command tools can launch installed CLIs by PATH name or executable path, including project-local tools. The fixed program-name restriction is removed from both one-shot commands and command sessions; existing access modes and process controls still apply.
- Unfinished provider streams now report elapsed time and the last activity, with provider-specific recovery instead of a generic tool-response error. Drafts, selected skills and completed earlier work remain available.
- Foreground tasks report status inline without creating notification toasts or inbox entries. Other conversations and tasks running while Dolores is unfocused still notify; skipped foreground events are not replayed later.
- Submitted skill references now move from the composer to the message instead of remaining selected for later prompts. Failed sends restore both prompt and skills while preserving newer drafts and selections.

## [0.1.0-alpha] - 2026-10-26

First public alpha of Dolores. The earlier `0.0.1-alpha` entry was an
unsuccessful CI trial, never a published release; its changes are included here.

### Added

#### Windows desktop and installation

- Windows x64 desktop app with local or hosted OpenAI-compatible models, light/dark themes, a floating navigation dock, project shortcuts and searchable Settings.
- Windows installer with current-user/all-users modes, optional shortcuts, upgrades and an uninstaller that retains local data unless removal is explicitly selected.
- Tag-driven GitHub releases with a verified installer, SHA-256 checksum and the matching reviewed changelog section as release notes.
- Experimental macOS PKG and Linux DEB packaging commands for future native qualification.

#### Conversations and task control

- Markdown messages, streamed replies and ordered expandable tool receipts, including retained failures and recovery results.
- Search saved conversation titles and messages; pin, archive and restore chats while preserving drafts, queued input and active tasks.
- Queue and edit follow-ups during a task, or steer the task at its next safe boundary. Stop pauses queued work for explicit inspection and resume.
- Edit a saved prompt in a separate branch with a chosen folder and shared context, retaining the original transcript and tool receipts.
- Ask about selected text in an independent side chat and queue or steer a conclusion back to its parent.
- Clickable clarification answers and a custom-answer field, with automatic continuation after submission.
- Inspection-only plans, explicit execution of accepted revisions, and saved goals with verifiable criteria and cumulative task limits.
- Previewed context snapshots through `@` for files, folders, code, terminal excerpts and messages; discover commands and reviewed skills with `/`.
- Automatic context compaction, useful memory with saved preferences, and explicit recovery that preserves drafts, streamed partial replies and operation evidence.

#### Project workspace and developer tools

- Project folders, split editor tabs, Find/Replace, keyboard save/redo, language tools and an integrated terminal.
- Git changes, staging, commits, history, side-by-side diffs, stash and revert with conflict recovery that preserves existing work.
- Diff-line comments with queued or steered feedback and retained delivery/resolution evidence.
- Create task worktrees from a reviewed branch, tag or commit; compare and transfer selected changes while retaining both versions on conflict.
- Archive managed worktrees with recoverable Git snapshots, retained staging and ignored files; restore explicitly and inspect interrupted operations.
- Inspect task-owned command output, send reviewed input, interrupt or close commands, and retain receipts across failures and restarts without replaying commands.

#### Tools, skills and connections

- File, command and web tools, plus optional browser and Windows computer use. In-app Browser setup reports prerequisites, progress and cancellation.
- Windows computer control through fresh screenshots and capture-bound accessibility IDs, including control of Dolores itself; ordinary-chat actions run within the selected access mode and task limits.
- Review / Auto edit / Full access choices remembered per chat, with file, shell and computer permissions kept separate.
- Remote MCP endpoints with bearer keys or Windows browser sign-in, explicit inspection, searchable tool selection and exact endpoint approval. Credentials use the native vault; interrupted login has explicit recovery.
- Complete skill ZIP packages with reviewed instructions, resources, dependencies and versions; separate installation/activation, version restoration and original-archive export.
- Bounded access to selected package references, including Global packages; binary assets return metadata and scripts retain separate execution permissions.
- Forget reviewed older skill activations and remove unused installations without clearing the current skill or resources.
- Reviewed project lifecycle hooks for required checks and observations, retained failure output, fresh review after script changes and a Disable all recovery control.
- Optional Windows isolation for Node commands and MCP servers with reviewed folder access and denied network access; failed setup refuses native fallback.

#### Schedules and notifications

- Chat-created schedules, optional companionship with a Quiet–Chatty frequency control, and Windows tray scheduling with reopen and interruption recovery.
- Compact animated toasts and retained Notifications history for completion, failure, review and question events, with optional Windows alerts and links to tasks or evidence.
- Remembered Ask / Quit / Tray close choices; minimizing preserves current work independently of scheduled background settings.

### Fixed

#### Conversation continuity and recovery

- Repeated New chat and project actions reuse empty conversations; switching chats preserves newly created conversations, drafts, selected models and reviews.
- Scrolling up preserves the reading position during replies; Latest messages reaches the final reply after layout settles.
- Sending recovers after input-method focus changes. Attachment and file errors explain the actual failure and keep drafts and recovery actions available.
- Earlier failed-command receipts remain available when a repair reuses a tool-call ID. Follow-up chats retain saved tool evidence without replaying earlier actions.
- Models can be removed while retaining conversations and provider connections. Saved task limits and instruction comparisons preserve their configured ranges, output and time limits.
- Memory captions, titles and thinking-off responses retain valid content; saved memory and companionship preferences remain respected.

#### Workspace and tool reliability

- Git views distinguish non-repositories, empty repositories and detached HEAD; subfolders show the enclosing repository root.
- Git actions clear stale success notices, refresh history and comparisons after changes, and handle large diffs, commits, stashes and restores without losing retained comparisons or work.
- Kept editor tabs survive reopening and double-clicks; Find/Replace preserves document selection, and the panel divider supports quick drags and keyboard resizing.
- Language edits recognize equivalent Windows paths. PowerShell startup and Unicode output work with legacy console code pages.
- Managed-worktree queues include subfolder chats and refuse conflicting repository operations; changed registrations require fresh review.
- Computer tools are available in ordinary working chats and side chats without a folder; provider failures report useful errors. Browser, external-tool and web receipts show correct arguments and excerpt locations.
- Skill matching handles simple singular/plural keywords, incomplete evaluations explain missing inputs before model requests, and selected MCP tools can run alongside built-in tools.
- Repair-and-verify requests retain the reviewed repair and rerun the failed check; failed screenshots and native-build details keep readable recovery information.

#### Scheduling and desktop behavior

- Scheduled runs execute saved work rather than repeating setup, retain their chosen project/model and use a pinned skill only when selected.
- Scheduling handles multilingual requests and long Unicode prompts, displays readable saved/updated/paused/deleted status, and reports save failures accurately.
- Quiet companionship and model reasoning preferences remain respected; omitted schedule times use a disclosed 09:00 local default.
- Settings search finds Theme and companionship frequency controls; hovering a toast close button keeps it visible.
- Quit and Tray hide promptly and retain bounded recovery during failed saves, busy actions and open reviews.
- Installer verification waits for complete uninstall cleanup before reinstalling and retains diagnostic logs on failure.

### Known limitations

- This is an alpha release. Windows x64 is the supported release target; installers are unsigned and require the Microsoft C++ x64 runtime. macOS/Linux packages remain experimental and unqualified.
- Model reliability, physical input/accessibility and low-end performance need further evaluation. Experimental detached windows remain unavailable.
- Remote MCP sign-in and recovery are covered by public Windows fixtures; third-party login/model reliability and broader lifecycle cleanup remain separate. Cross-call remote sessions and background subscriptions are unavailable.
- Isolated Node child processes must inherit stdio; child pipes can stall until the deadline or Stop. Other runtimes, human terminals, browser helpers and native repair builds retain native execution.
