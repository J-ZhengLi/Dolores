# Changelog

User-visible changes are recorded here. Unreleased work has no release date;
published versions use `MAJOR.MINOR.PATCH` and an ISO date.

## [Unreleased]

### Added

- Clickable answers and a custom-answer field for agent clarification questions, with automatic continuation after submission.
- Windows desktop preview with local or hosted OpenAI-compatible models.
- Reviewed file operations, commands, web tools and optional browser/computer use.
- Project editor with split tabs, Git changes and diffs, terminal and language tools.
- Automatic useful memory, chat-created schedules and optional companionship.
- Optional Windows tray scheduling with reopen and interruption recovery.
- Windows GitHub Actions packaging with reviewed changelog notes and draft releases.

### Changed

- User messages render Markdown, including headings, lists, tables and code blocks; Copy message retains the original text.
- Chat access uses an immediate Review / Auto edit / Full access selector, remembered for the chat without a duration or extra confirmation steps.
- Companionship frequency uses a Quiet–Chatty slider from 0 to 100 per day, with pacing that follows the selected frequency and hours.

### Fixed

- Side chats receive instructions for their available tools, avoiding requests for file tools they cannot use.
- Read-only delegated agents receive file-reading guidance without instructions to create files.
- Follow-up chats retain recent saved tool evidence, including command output and exit codes, without repeating earlier actions.
- Git history diff tabs show the short commit ID so revisions of the same file are easy to distinguish.
- File failures appear once beside their recovery actions, while other file errors remain visible.
- The file editor saves with Ctrl+S and supports Ctrl+Y alongside Ctrl+Shift+Z for redo.
- Latest messages reaches the end of the final reply after the history layout settles; scrolling up still keeps your reading position.
- Earlier tasks labels the configured output limit clearly; Activity points to the current Conversation view for saved exchanges.
- Chat shows recent tool outcomes and lets you inspect all earlier actions, including failed attempts and recovery results.
- Rejected file edits show their actual failure reason instead of a misleading invalid-path heading.
- The side-panel divider responds to quick drags and acquires keyboard focus for Left/Right resizing and Home reset.
- Context previews include the scheduling tools and response limits sent on ordinary chat requests, without duplicate tool guidance.
- Agent guidance explains how available commands can read cross-project evidence instead of incorrectly treating every external path as inaccessible.
- Stopped scheduled tasks point back to Scheduled instead of claiming an empty chat draft was restored.

- Find and Replace leave the document visible and selectable below the search bar.
- Long Chinese, Japanese and emoji scheduling requests keep a valid title and their full prompt; save failures report scheduling recovery instead of a folder error.
- Scheduled task changes show readable updated/paused/deleted confirmations with their saved schedule.
- Source Control shows the actual repository root and identifies an enclosing repository when the selected project is a subfolder.
- Settings search finds companionship by Frequency, Quiet or Chatty.
- Stopping a response retains already streamed text in the Not saved panel, marked partial, while restoring the draft.
- Repair and verify requests a reviewed repair and reruns the failed check, including when the original request was read-only.
- Source Control history updates after a commit or another Git action changes the current commit, while retaining readable diffs.
- Double-clicking a file in Folders keeps its tab; reopening a kept file preserves other preview tabs.
- Switching back to an earlier chat keeps newly created conversations visible, while preserving drafts, selected models and active reviews.
- Scheduled runs execute the saved task's work instead of asking to configure the schedule again, and use a pinned skill only when one was selected.
- Scheduled runs show their pinned project and model in chat and operation reviews, while preserving the current Home conversation.
- Chat shows a readable schedule as soon as a task is saved, even if the model's reply stalls. Opening Scheduled refreshes newly created tasks.
- Language edits accept equivalent Windows file paths instead of incorrectly reporting that project files are outside the selected project.
- Source Control has quick stage/unstage buttons for each file and all changes. Commit includes working changes when nothing is staged, while preserving an existing staged selection.
- Chat scheduling interprets ordinary requests across languages, uses a disclosed 09:00 local default when time is omitted, and reports scheduling errors accurately.

### Known limitations

- Windows x64 is the release target. Packages are unsigned; other platforms are unqualified.
- Experimental detached windows remain unavailable after their performance gate failed.
- Model reliability, physical input/accessibility and low-end performance need further evaluation.
