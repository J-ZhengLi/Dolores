# Project and global skills

Valid source skills are available directly, without an upfront review or
activation step. Project collections live at the saved working folder's
`.agents/skills/<name>/SKILL.md`; global collections live at
`~/.agents/skills/<name>/SKILL.md`. `DOLORES_GLOBAL_SKILLS_DIR` can select a
different global collection. Global skills also work in temporary and side chats.
Discovery reads the configured collections at catalog/context preparation; it
does not require opening Settings, write activation records or run a model.

The `/` picker lists available skills at a plain line boundary or within prose.
Inline completion inserts the skill's `/name` at the cursor without changing
surrounding text, and pins the same inspectable selection used by the chip.
Only line-start discovery includes host commands; inline discovery cannot invoke
a same-named host action. Paths, code spans, fenced code and active IME
composition do not open discovery. Failed saves preserve the draft and previous
selection; typing during a save is never replaced by its completion.

Picked skills follow the draft. Send and Queue/Steer first submit the pinned
references, then clear them from the composer after acceptance. The pending user
message lists its submitted skills, and saved messages list skills recorded in
the request context. A failed or stopped send restores the prompt and references
for retry, preserving any newer draft/selection. A restoration that cannot be
saved stays visible and blocks resending without the skill until saving succeeds.

Ordinary relevance selection includes
matching instructions without a prior Settings action. Settings shows ready,
disabled or unavailable skills, optional **View file** and direct Enable/Disable
controls. Create/import validates and saves a skill in one action. Source edits
apply to future requests; each admitted run keeps the exact instructions it
started with, including across compaction. Selected file references bind the
canonical collection and content identity. Changed or deleted selections refuse
substitution with a clear refresh/reselect action, preserving the draft. Queued
and steering delivery rechecks the selected identity before use.

An enabled project skill overrides an enabled global skill of the same name.
Explicit disabled records remain disabled; disabling a project entry reveals its
global counterpart. Saved versions, generated skills, installed packages and
rollback snapshots retain their existing identities and recovery controls.
Restoring a saved version pins that snapshot instead of replacing source files.
Forgetting saved versions leaves source files, which become available directly
again. Missing or malformed live files are omitted from discovery with a problem
shown in Settings; a retained valid snapshot remains usable for recovery.

Source reading validates UTF-8, YAML name/description, direct regular paths and
the 1 MiB file/snapshot allocation. This is independent of model capacity and
generated-draft output. Only the configured global collection boundary
may be a symlink/junction; individual skills retain existing path checks.
The bounded scan examines 128 entries and returns up to 32 eligible names per
collection, with a partial-list notice. Availability does not consume the request
instruction allowance: up to three matching skills per scope enter one request,
with explicit picker selections taking priority over automatic matches. Exact
selected text uses the configured model input allowance; it is never silently
truncated or dropped by a separate 8-KiB aggregate ceiling. File-size, NUL and
UTF-8 errors are reported separately. Viewing or discovering a skill sends no
model request; unrelated instructions stay out of the request.
Saving another skill no longer requires disabling an unrelated one first.
Saved records retain the twelve-name and five-version bounds per scope.

Skill instructions never grant tools, execute scripts, install dependencies or
expand access. Host permissions and direct user requests keep precedence.
Metadata and referenced resources remain data subject to the existing tool path.
Malformed documents and failed saves keep editable input and the previous usable
record. Compatibility review/export/rollback bridge commands remain supported
for older clients, but normal file use and create/import need no review token
from the user interface. File selections are stored as additive `file: true`
references; the host validates them against its configured collection rather
than requiring a saved activation in SQLite. Legacy saved references retain
their original JSON shape.

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
to 2 MiB, each resource to 512 KiB, SKILL.md to 1 MiB, manifest to 8 KiB, and the
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

## Lifecycle ownership

Removing a scoped registration preserves physical resources referenced by
nondeleted scheduled tasks (including paused tasks), active occurrences and
unresolved queued/steering inputs. This includes an occurrence's old snapshot
after its task changes or is deleted. Cancel the input or delete future recurrence
and finish the active occurrence before removing retained files. Terminal history
alone is not a runnable owner and does not pin bytes forever; its immutable receipt
remains inspectable after release. An unacknowledged uncertain input keeps its pin.

A safe-boundary steering input first resolves its exact reviewed skill selections.
It can add selected resources to the parent's run binding, up to three distinct
packages for the run, but cannot replace an existing skill/scope binding. A failed
or stale selection remains recoverable instead of substituting a current package.
A child freezes its own copy of selected package sources when it is created;
later parent steering cannot widen that child's resource access. Child resource
reads use normal parent permission checks and count toward its existing cumulative
four file-operation allowance. They never execute scripts or grant command/MCP
access. Compaction retains these host bindings rather than rebuilding authority
from model-authored summaries.
