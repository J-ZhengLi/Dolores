# Privacy and public repository hygiene

## When using Dolores

Dolores sends selected context, enabled instructions/memories/skills and approved tool results to your configured endpoint. Provider retention belongs to that provider. The desktop app has no background telemetry or automatic model download.

Conversations, preferences, source evidence, comparisons and change snapshots are local unencrypted SQLite data. Remembered model/MCP keys use the OS vault, scoped to the data directory. Launch-only keys stay in memory. Exports can retain approved content and evidence: inspect them before sharing.

Deleting a preference does not erase earlier reply provenance/exports. Deleting a chat preserves working files and independent change records. Commands/MCP servers run with user permissions and can access data outside the folder. See [user controls](USER_GUIDE.md).

## When contributing or publishing

Raw benchmarks, screenshots, traces, databases, audits and backups stay in ignored `output/`; they may contain personal paths, hardware details and activity timestamps. Never force-add them. Keep production credentials out of code/logs/fixtures; synthetic validation strings must stay clearly synthetic.

Inspect publishable files, branches/tags and commit metadata before publishing. Git authorship is public; this repository uses the author's configured identity. Earlier cleanup removed private benchmark/history artifacts, but subsequent author configuration is not anonymized. Secret/personal-pattern scans detect selected leak classes, not every private fact.

Ignored `output/privacy/` may contain recovery copies of private history. Never upload it. Check archives separately: Git scans do not cover ignored data/build outputs. The Windows packager accepts reviewed runtime paths and user documents, refuses unexpected files/links, and excludes workspaces, databases, credentials and diagnostic outputs by construction.

Compiled code can retain source-machine paths even when Git is clean. The Windows build removes known compiler paths and keeps debug symbols private; packaging refuses current workspace/home prefixes in its payload. Review binaries and third-party assets separately before public distribution. This local identity check is not a universal personal-data or secret detector.
