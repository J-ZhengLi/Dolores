# Changelog

User-visible changes are recorded here. Unreleased work has no release date;
published versions use `MAJOR.MINOR.PATCH` and an ISO date.

## [Unreleased]

## [0.0.1-alpha] - 2026-10-13

### Added

- Windows installer with current-user/all-users modes, shortcuts, upgrades and an uninstaller with optional local-data removal.
- Experimental macOS PKG and Linux DEB packaging commands for later native qualification.
- Clickable answers and a custom-answer field for agent clarification questions, with automatic continuation after submission.
- Windows desktop preview with local or hosted OpenAI-compatible models.
- Reviewed file operations, commands, web tools and optional browser/computer use.
- Project editor with split tabs, Git changes and diffs, terminal and language tools.
- Automatic useful memory, chat-created schedules and optional companionship.
- Optional Windows tray scheduling with reopen and interruption recovery.
- Windows GitHub Actions packaging with reviewed changelog notes and draft releases.

### Changed

- Commit and pull-request CI runs checks without packaging; tag/manual release runs retain installers and checksums, with reviewed changelog notes on tag releases.
- One release preparation command aligns Cargo/Flutter/lock versions and dated notes, including alpha prereleases.
- Git actions run directly from their controls without an extra review dialog; stale-state, unsaved-editor and conflict safeguards remain.
- User messages render Markdown, including headings, lists, tables and code blocks; Copy message retains the original text.
- Chat access uses an immediate Review / Auto edit / Full access selector, remembered for the chat without a duration or extra confirmation steps.
- Companionship frequency uses a Quiet–Chatty slider from 0 to 100 per day, with pacing that follows the selected frequency and hours.

### Fixed

- Browser receipts label reviewed operations correctly instead of calling them searches.
- Skill drafts explain missing test inputs before evaluation, keeping incomplete tests from sending a model request.
- Windows CI retains Flutter release tags so the pinned SDK resolves dependencies with its correct version.
- Native-build details keep their own expanded state, and missing screenshot errors stay readable without repeating the original action.
- External-tool receipts render their reviewed arguments correctly when expanded or reopened.
- Web page receipts distinguish excerpt positions from search queries and describe partial-page continuation in plain language.
- Starting a new Git action clears the previous action's success notice, so a refusal cannot appear to have succeeded.
- Attachment errors distinguish oversized text from unsupported files and offer the appropriate next step without losing the draft.
- Source Control no longer labels folders without Git as detached HEAD; empty and detached repositories show distinct states.
- Activated skills match simple singular/plural keywords, so a request to review code can use a skill described for code reviews.
- Large changes can be committed, discarded or reverted without an unused review diff blocking the action.
- Stash and revert conflicts explain how to recover while preserving the stash and existing work.
- Large saved changes can be stashed and restored without fitting their entire diff into a review.
- Scheduled occurrences execute the saved work instead of repeating setup-only instructions from the original request.

- Image memories supply their display title prefix without discarding otherwise valid captions.

- Two reviewed MCP tools can run together alongside the current built-in tools without exceeding an obsolete catalog limit.

- Explicit thinking-off responses handle a stray leading closing marker, allowing valid image memories to be saved and recalled.

- Companionship respects its chosen model's reasoning setting and tighter response limits.
- Refresh diff reloads the current Git state after an external edit, while failed reads keep the previous comparison.
- Large side-by-side Git diffs keep old and new lines together across pages; tabs retain their layout and failed reads preserve the current comparison.
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
