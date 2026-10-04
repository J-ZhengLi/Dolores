# Dolores

Dolores is a desktop agent harness for working with a local or hosted language model. Choose a project folder, ask for help, and review proposed file edits and commands before they run. Its name comes from Dolores in *Westworld*: the aim is an assistant that improves through useful memories, reusable skills and measured feedback.

Dolores is an early preview. Windows has been exercised locally; macOS/Linux builds, native accessibility and low-end performance still need verification. See [current acceptance](docs/ACCEPTANCE.md) for the tested scope.

## Get started

For a Windows portable preview, extract the **entire** ZIP to a folder and open `Start-Dolores.cmd` inside it. It checks for missing app files and explains how to install or repair the [Microsoft Visual C++ x64 runtime](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist?view=msvc-170) if needed. Keep the DLLs and `data/` folder together. Portable describes the application files; conversations and settings use your account's application data directory.

The repository includes a [build and packaging guide](CONTRIBUTING.md); it does not yet promise signed installers or an automatic updater.

1. Open **Settings → Models → Connection & models**, enter your provider's OpenAI-compatible API base URL and key, then **Fetch models**. Select the models you want available and save. Manual model IDs are available when listing is unsupported.
2. Choose **Open project…** for an existing folder, or start a **Temporary workspace** and Dolores creates one. Choose **Side chat** for conversation without file tools.
3. Select a model inside the input box and send a message. Review each proposed tool operation before allowing it. Working chats need a model that supports Chat Completions function calls.

Dolores connects to models you provide; it does not install or start a local model server. **Remember connection** stores a key in the OS credential vault. Without it, the connection lasts for the current launch.

## What you can do

- Read, find, edit and create text files in a working folder, with reviewed diffs and a local change journal.
- Attach text files or PNG/JPEG images, preview what will be shared, and use images with an explicitly configured capable model.
- Run reviewed commands and connect installed local MCP tool servers.
- Delegate scoped file work to up to two subagents, with shared limits and parent verification.
- Search the web without setup, read public source pages, or configure Brave/SearXNG for search.
- Browse and export conversations, inspect context and reported token usage, and explicitly continue paused tasks.
- Inspect learned preferences, review reusable skills, and compare instruction snapshots on bounded response tests.
- Record local **Worked / Needs work** feedback against a reply's original evidence.
- Inspect running capabilities and run evidence, and configure project/chat request and interaction overrides.
- Choose task permissions, restore saved drafts, fork completed turns and opt into bounded automatic context compaction.

Follow the [user guide](docs/USER_GUIDE.md) for controls, recovery and data handling. File tools stay inside the working folder; commands and MCP servers run with your account permissions. Chat history, preferences and file-change snapshots are local plaintext, and approved content is sent to your configured model provider.

## Develop Dolores

The selected desktop UI is Flutter, backed by a Rust core and replaceable provider, storage, credential and tool interfaces. No model or Node sidecar is bundled.

- [Contributor guide](CONTRIBUTING.md) — prerequisites, build, tests and packaging.
- [Architecture](docs/ARCHITECTURE.md) — components, trust boundaries and limits.
- [UI contract](docs/UI.md) — shared theme and interaction rules.
- [Documentation index](docs/README.md) — user, developer, design and verification references.
- [Roadmap](docs/ROADMAP.md) — planned milestones, dependencies and acceptance gates.

Dolores is licensed under [MIT](LICENSE).
