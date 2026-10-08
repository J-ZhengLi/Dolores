# Changelog

User-visible changes are recorded here. Unreleased work has no release date;
published versions use `MAJOR.MINOR.PATCH` and an ISO date.

## [Unreleased]

### Added

- Windows desktop preview with local or hosted OpenAI-compatible models.
- Reviewed file operations, commands, web tools and optional browser/computer use.
- Project editor with split tabs, Git changes and diffs, terminal and language tools.
- Automatic useful memory, chat-created schedules and optional companionship.
- Optional Windows tray scheduling with reopen and interruption recovery.
- Windows GitHub Actions packaging with reviewed changelog notes and draft releases.

### Fixed

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
