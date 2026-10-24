# Reviewed project and global skills — bricks 5.1 / 5.1.1

Dolores reviews standard [Agent Skills](https://agentskills.io/specification) files. Project skills live at a saved working folder's `.agents/skills/<name>/SKILL.md`. Global skills live at `~/.agents/skills/<name>/SKILL.md` and apply to all project, temporary and side chats using the same application data directory. Side chats still have no working-folder tools. Discovery and activation are explicit; no watcher, implicit activation or model request runs in the inspector.

The host resolves the home directory through the platform directories library. An optional absolute `DOLORES_GLOBAL_SKILLS_DIR` selects another global collection directory, mainly for portable installations and isolated tests. It refers to the skills collection itself, not its parent. Only this collection boundary may be a symlink/junction: resolve its canonical target, open that target, and bind it to file review. Activation refuses a retargeted boundary even when text matches. Project directory components and individual skill directories/files retain direct-path checks. No ancestor or other application's skill-directory discovery occurs. Missing sources do not prevent reviewing saved versions.

Review reads one UTF-8 regular file, at most 8 KiB, validates YAML `name`/`description`, and shows its complete literal text. Names match their direct directory and the specification's ASCII naming rules. Other metadata is accepted as data; `allowed-tools` cannot approve tools. References/scripts/assets remain literal and require the existing separately approved tools within the working folder. Global resources outside that folder are unavailable; global activation does not broaden filesystem access. Review never executes files.

Activate stores a reviewed snapshot in the selected scope. File changes require new review to take effect; they do not silently replace the active snapshot or prevent use of the reviewed version. Disable preserves versions. Roll back reviews a retained version and publishes a new version containing that exact text, without modifying files. Forget removes all local versions and future retrieval; it leaves source files and historical reply provenance intact.

An enabled project snapshot overrides an enabled global snapshot with the same name for that working folder. Disabling or forgetting the project entry reveals the active global entry. Disabled project entries do not shadow global entries. Both records retain independent versions and revisions, and both inspectors disclose scope. Only effective snapshots enter the next request; overridden global text is omitted from preview, token accounting and sent context.

One five-minute, single-use review token binds session, scope, configured root, canonical global target, record revision and exact text. Activation consumes it, rechecks the source (or retained rollback version), then performs an atomic compare-and-swap. Failures preserve old state and require another review. Listing/scope switching replaces review; Close cancels it. All operations use the existing active-run exclusion.

Bounds apply separately to each scope: inspect at most 128 directory entries, return up to 32 sorted eligible names with a partial notice; store at most 12 records including disabled, retain five latest versions/record; enable at most three skills with at most 8192 combined source bytes. At most six effective snapshots / 16 KiB may enter a request, subject to the existing total byte/token budget. Over-limit activation refuses rather than silently omitting skills. Revision advances on each mutation; version advances on activation/rollback; timestamps are Unix milliseconds.

SQLite schema remains 14. The existing skills table stores project records under absolute folder keys and global records under an internal non-path namespace, disjoint from project roots. Project storage ports reject that namespace. Global JSON records include `scope: global`; legacy records and provenance default to Project without rewriting history. Optional global storage ports preserve compatibility with other plugins. No new dependencies, runtime processes or background workers are added by 5.1.1.

Preview and send share the same snapshot selection. Exact effective text is mandatory system input alongside AGENTS.md, memory, summary, schemas and framing. A model window too small for mandatory input fails before transport. Context, started events, saved replies and exports retain scope/name/source/version/review-time/rollback provenance; local preview additionally exposes only current effective snapshots. Global provenance uses the logical `~/.agents/skills/...` location, even with a configured collection; the actual local path is shown only in the catalog, never automatically sent or exported. Host tool policy and direct user requests outrank skills.

Reuse the system palette and InspectorFrame. Saved-chat headers offer Skills, or Chat actions below 480 pixels. A fixed Project / Global selector switches scope; side chats start with Global and disable Project. A scrolling list shows Available/Enabled/Disabled and global overrides. Review shows exact source, active/retained versions and explicit folder/all-chat behavior. Back/Refresh/Activate/Disable/Forget stay in a wrapping fixed footer. Pending writes lock dismissal, scope switching and duplicates; the controller locks chat changes while open. The composer stays unchanged.

The existing local bridge commands `projectSkills`, `reviewSkill`, `disableSkill` and `forgetSkill` accept optional `scope: project | global`, defaulting to Project for older callers. The name of `projectSkills` is retained for compatibility. Activation uses the token's bound scope; cancellation uses its matching token. There is no new network endpoint or model tool.

Acceptance covers frozen parsing/path/scope/approval/persistence/context cases, synthetic native provider wire/restart/export and compact light/dark rendering. This foundation does not claim task improvement, generated-skill quality, automatic selection or cross-platform OS input acceptance.

## Complete ZIP packages — EH09.1

Settings → Skills → Packages manages complete packages in either This project or
All projects. All-project management is available without a chat or model. Enter
the name declared by SKILL.md, choose a ZIP, then review its literal instructions,
file changes, declared origin and dependencies. Install stores the bundle;
Activate is a separate choice for future messages. Dependencies are inspected
only. Neither action runs scripts or grants tool access. Existing source-file
skills and their SKILL.md-only export remain available in Skills.

The ZIP has SKILL.md at its root and may contain scripts, references, binary
assets and empty directories. Optional `dolores-package.json` accepts schema 1,
version, origin and a dependencies list of name/requirement objects. Origin and
requirements are publisher claims. ZIP input is limited to 4 MiB, expanded files
to 2 MiB, each resource to 512 KiB, SKILL.md and manifest to 8 KiB each, and the
archive to 64 files/128 entries. Paths use portable relative forward slashes,
up to 240 UTF-8 bytes and twelve components; escaping paths, links and aliases
are refused. Each scope retains five installed versions per name and twelve
names; the profile retains at most sixty physical bundles. Capacity errors ask
for explicit removal rather than evicting an older usable version.

Review an installed archive to activate it, inspect its saved activation
versions or restore an exact saved version. Export ZIP copies the original
archive to a new filename and refuses overwrite. Remove requires confirmation;
saved skill versions or another scope may retain shared resources. Under Saved
versions, **Forget saved vN** removes only a confirmed older activation. The
current version, its enablement, installations and all files stay intact. The
current saved version cannot be forgotten through this action. Once no saved
activation uses an old installation, Remove can release it and make room for a
sixth distinct package. A failed save keeps the review for retry. An already
published archive from a failed sixth install can be selected again without
duplicating its files. Explicit composer references retain their reviewed
identity; refresh/reselect a stale chip while keeping the draft. Retained data and
recovery can inspect the receipt and export the original archive even when
payload bytes changed. Preserve/copy changed files before restoring originals,
or activate another healthy version. Dolores does not overwrite or remove
unknown or changed files as a repair. Failed writes keep the review for retry;
expired reviews require Refresh and a new inspection.

Selected package resources use the read-only `read_skill_resource` tool. Host
context supplies the exact skill name, Project/Global scope and archive ID;
the reader binds only packages actually selected for this request. Inactive,
overridden and unrelated packages cannot be read by supplying their names.
Preview discloses package identity, content digest and the owned resource base.
List returns paged file metadata. Read accepts one exact portable relative path
and returns UTF-8 text with a `nextOffset` byte continuation. Binary assets return
verified metadata only and remain available through exact ZIP export. Requested
results are bounded to 1–16 KiB including JSON escaping and metadata, further
limited by the normal tool allowance; resource bytes never enter context merely
because a package is installed or selected.

The host verifies the retained receipt, ZIP and payload before preparation and
again on invocation. The requested file is read and hashed through the owned
directory capability. Changed, unavailable or substituted files refuse sharing
and retain the Packages recovery route. Compaction can reuse the same binding,
but cannot replace it with another package. Normal task budgets, Stop, permission
revision checks and logged tool receipts apply. Review mode asks for host approval;
Full Access follows its saved policy. Existing working-folder automatic grants
do not grant reads from package storage. Script execution and dependency
installation retain their separate command authority. Planning may inspect
resources; children do not gain this tool in this slice. Scheduled/child lifecycle
integration remains tracked by EH09.5.
