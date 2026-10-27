# Source Control contract — milestone 17

The selected Home conversation supplies the project. Source Control discovers
the enclosing Git worktree lazily, identifies its branch and owns retained
views per repository. There is no project dropdown or automatic repository setup.
Git is a discovered direct executable outside the project. Human Git actions do
not grant model access. Home receipts link to repository views as separate evidence.

Use the compact Classic sidebar: Source Control heading and repository actions,
then branch and changed-file count. Keep the canonical root in the branch row's
tooltip; omit the project name and path block above it. A named “Enclosing
repository” indicator appears when this root differs from the selected project
folder. Equivalent Windows path spelling is the same root. The tooltip follows
the selected conversation and retained status after a failed read; it is not a picker.

Place the message field and a compact Commit button above independent Staged
Changes and Changes groups, with History below. The button identifies the staged
count, or explicitly says Commit all when no files are staged, using the existing
commit behavior. Keep drafts and failures visible. File rows target 26 px at normal
text scale, growing for accessibility. Show the basename first, parent path when
space permits, and complete/rename paths in the tooltip. Keep stage/unstage and
overflow actions reachable. History uses compact subject/author rows and retains
nested files, loading and retry controls.

Status uses porcelain v1 NUL records, including rename pairs; filenames must be
valid UTF-8 to be actionable. Diff bases are saved working tree/index, index/HEAD
or an exact commit/parent. Binary files show Git's change summary; non-UTF-8
display bytes carry a visible replacement-character notice.
History pages pin HEAD and use exact commit IDs. Inline and side-by-side do not
edit files; file editors keep their independent unsaved buffers. Line-comment
controls save local review findings against the exact displayed comparison.

## Review findings

Use a named line comment in either layout to save a finding. The host verifies
the comparison root/repository, revision, complete patch digest, page and side,
then captures the actual UTF-8 line excerpt. Unsupported or long segmented lines
refuse with guidance; no frontend text is accepted as source evidence. Saving a
note does not start a model. Current findings appear beside their lines; Review
findings opens retained notes, selected feedback, receipts and resolution.

Choose Queue selected or Steer with selected to submit one to eight findings.
This uses ordinary task input admission, snapshots, limits, Stop and delivery
receipts. The queue entry and finding references commit together without
consuming the composer draft. Current chat ownership and checkout are checked;
notes are never shared to another chat merely because it uses the same folder.
Scheduled/computer-use runs retain their existing live-input restriction.

A changed comparison leaves the original note readable and marks its location
changed. Open/refresh the diff and explicitly select a current line if further
repair is needed; no guessed relocation is performed. Queue acknowledgement is
not resolution. After inspecting a later changed comparison, Resolve records the
user's assessment and host-observed evidence. Reopen is explicit. Remove note
only removes the local finding; it neither edits files nor cancels submitted
input. History and retained files keep their existing deletion rules.

Findings are inspectable without a live checkout. Saving, submission or resolution
shows its actual failure and preserves the note, selected feedback and draft.
A transport failure retains the job for Check status/Stop; committed results can
be reconciled through read-only finding/submission receipts without replay.
Review operations share the existing cancellable Source Control worker, checkout
pins and Git lane; no second model dispatcher or filesystem writer is added.

Bounds: 128 retained findings per chat, 32 per page, 4 KiB note, 2 KiB line
excerpt, 32 KiB encoded record, sixteen submission references per finding and
the existing 16 KiB input/eight-pending-instruction limits. Capacity errors offer
deliberate removal, smaller selection or pending-input inspection; never silently
delete findings or split a repair into unbounded automatic requests.

## Final interaction refinement — 2026-10-07

Follow the documented interface design: clicking a history commit toggles its nested
changed files; clicking one opens that exact commit's diff. Keep each commit's
loading/error/Retry and files attached to its row. Initially load history when
binding a committed repository; Refresh history and Load more remain explicit.
Retain at most eight commit file sets per repository, evicting only collapsed
sets; if all eight are expanded, ask the user to collapse one. Failed reads retain
usable cached files and the commit message, and never relabel another commit's files.
After an applied Git action changes HEAD, reload the first history page so the
new commit is visible immediately. Retain existing exact-commit diff tabs and
expanded files still present on that page. An unavailable history read preserves
the last readable list and the completed action's outcome; Refresh history retries
only the read. Actions that leave HEAD unchanged retain the current history page.

Place repository actions in the three-dot menu beside Source Control: Commit, Pull,
Push, Fetch, and grouped Changes, Branch and Stash actions. Use familiar names
at entry points; selecting an action runs it without a review dialog. Ctrl+Enter
in the message field commits directly. File and commit overflow menus
contain their contextual actions, including Revert commit. Branch/stash/remote
selection uses a temporary picker; opening the menu performs no network request.
Keep the compact rail, Home project ownership, diff bases and editor guards.
This refines milestone 17; it does not add a full branch graph or new Git commands.

## Quick staging — 2026-10-08

Show + beside Changes and each working file, and − beside Staged Changes
and each staged file. These deliberate index-only actions apply immediately;
their menu equivalents do the same. Disable them while busy or reviewing. Keep
the native single-use revision token and dirty-editor guards, but do not generate
a full patch for reversible index actions. Section actions select the complete
bounded status (up to 2,000 entries), rather than the 16-path selection limit.
Use literal NUL-delimited pathspec files so filenames and bulk actions do not
depend on shell quoting or Windows command-line length. Stage a changed renamed
file by its current index name; unstage a rename with both its old and new names.

Commit uses the existing staged selection when nonempty. Otherwise selecting
Commit stages and commits the saved working changes, including untracked files.
Internally pin the selected paths and saved state; preparation must not stage
anything or capture an unused full commit diff. Apply checks the saved revision,
author and editor state before staging, then runs the normal commit with hooks.
Failed hooks retain the draft and staged work; Refresh reconciles the state before
a fresh action. No automatic staging occurs on page entry, refresh or typing.

### Direct Git actions — 2026-10-11

The user explicitly removed Git review dialogs. Selecting Commit, a hunk action,
Discard, Stash, Revert/Abort or a remote action is the decision to execute it.
Branch names and stash/remote choices still need their compact input/picker;
there is no second approval screen. Keep preparation and single-use revision
tokens internal, including dirty-editor, saved-state, author, remote-ref and
conflict checks. Disable duplicate actions while pending. Cancel internal
preparation if its initiating page closes before apply. Failed commits keep the
message and index; stale or uncertain outcomes require Refresh and a deliberate
retry. This does not change chat/tool approval behavior.

Commit, Discard and Revert preparation retains bounded metadata rather than
generating full patches. Full patch capture had blocked large actions at
the Git output/patch limits, despite no review being displayed. Saved-state, author,
dirty-editor and hook checks remain; staged content still takes precedence over
later working edits. Diff viewing retains its independent paging and byte bounds.

On failed stash application or revert, inspect the resulting repository state.
Confirmed stash conflicts explicitly report that the stash remains and require
Refresh/resolution before another apply. Confirmed revert conflicts offer
Refresh and Abort revert. Do not substitute a generic commit/index failure for
these recovery directions or imply no saved-file effects occurred.

## Large-diff correction — 2026-10-07

The design excludes a whole-blob size gate and the unaligned full-file side panels.
Viewing now streams Git's unified changed sections without loading either blob.
The 256 KiB file gate is removed. A view page holds at most 256 display rows and
256 KiB of raw patch; long physical lines are segmented at approximately 4 KiB
UTF-8 boundaries. Previous/Next changes stays inside Dolores and replaces the
current page. The complete patch fingerprint and saved revision pin continuation;
stale/failed reads preserve the displayed page and expose Refresh/retry.

Side by side aligns deletion/addition runs within each page, keeps context lines
on the same row, colours each changed cell and shows actual old/new file line
numbers. Both columns share vertical and horizontal scrolling.
Unchanged text outside Git's three-line hunk context is omitted. Binary change
metadata remains viewable; there is no fabricated textual binary comparison.

Saved-file revision hashing streams through a 64 KiB buffer with Stop and a
30-second read deadline, replacing the 16 MiB/file and 32 MiB aggregate gates.
Git process supervision and metadata/index bounds remain. Selected hunk actions
retain their complete-patch bounds: a displayed partial page cannot become a
complete hunk patch. Whole-file actions pin state without capturing an unused
full patch. Remote operations retain their separate checks and deadlines.

### Cross-page alignment correction — 2026-10-11

Side-by-side paging uses aligned display pairs, so a replacement run spanning
many pages keeps its old/new counterparts together. Inline cursors still index
unified rows; side-by-side cursors index pairs. Switching layout starts at its
first page after a successful read, preserving the previous layout/page on
failure. Each tab retains its own layout and navigation trail. A genuinely
unmatched insertion/deletion keeps its blank counterpart; page boundaries do
not create blanks for matched replacement segments.

Refresh diff reads the current repository state before requesting the first
comparison page. It retains the exact path/basis/commit and that tab's layout.
If the state read fails, stop there and retain the prior comparison and trail;
do not submit a new diff with the stale revision. If the tab is closed or changed
while state loads, retain the user's current selection. Refresh performs no
mutation or automatic retry.

Streaming retains only the selected slice of a change run, at most 256 pairs
and 256 KiB of source text per side-by-side page. It never buffers a whole hunk
or blob. The separate existing 256 KiB raw-patch buffer is retained only for
complete hunk eligibility; a partial aligned page is not a complete hunk patch.
Fingerprint, saved revision, UTF-8 segment and Stop/deadline
guards still apply. This revises the previous permitted boundary-blank behavior
for large replacements without adding a file-size gate or Git mutations.

## Frozen initial bounds

### Stash preparation correction — 2026-10-11

Stash creation and application pin the exact selected paths or stash object,
staged/working change counts and preservation effect. They do not require copying
the entire patch into preparation: a large saved change must remain stashable.
Creation pins the full saved-file/index revision; application pins the exact stash
ID and current revision. Internal preparation identifies its file/count summary,
without presenting it as a full content diff. Saved change pages remain available
in Source Control before stashing. Untracked inclusion, rename restrictions,
dirty-editor guards, one-use tokens, expiry, conflict retention and explicit
apply/drop semantics remain. Commit, discard and revert also use bounded metadata;
hunk patch bounds remain separate. No process-output limit is raised.

| Item | Bound and recovery |
| --- | --- |
| Git jobs | Two executing globally, one per worktree; eight retained jobs; busy requests offer Retry |
| Process | Owned process tree, closed stdin, 30 seconds local / 60 seconds remote; Stop retains uncertain outcomes |
| Combined output | 512 KiB for ordinary commands; viewer stdout is streamed into bounded pages and a full fingerprint |
| Status | 2,000 changed paths within output bound; more offers external Git/filtered cleanup then Refresh |
| Revision checks | HEAD, index (16 MiB), status/config/stash refs, streamed saved-file hashes with 64 KiB buffer and 30-second read deadline |
| Diff display | 256 inline rows or aligned pairs / 256 KiB source text per page; approximately 4 KiB text segments; explicit Previous/Next changes |
| History | 30 commits per page, cursor at most 10,000; pinned HEAD, explicit Load more |
| Retained views | Eight repository owners, eight diff tabs each; inactive clean tabs can be closed |
| Mutation preparation | One host-generated, single-use token per repo, two-minute lifetime; bounded exact metadata for whole-file actions; only selected hunks capture a complete patch |
| Selected paths/hunks | At most 16 literal selected paths / 32 hunks, 64 KiB hunk patch; section index actions use all bounded status entries; stale basis refuses before apply |
| Commit draft | 8 KiB UTF-8; retained on hook failure; configured author and hooks remain enabled |

App mutations are serialized per repository and never hold the global chat lock
while Git, hooks or credential helpers execute. A busy repository refuses another
mutation rather than retrying it silently. Shutdown cancels and reaps owned jobs.
Dirty/pending editor buffers refuse conflicting operations; Save remains explicit.
Revision checks cannot prevent an unrelated writer racing the final Git operation;
Git index locks and patch checks remain additional defenses, not an OS transaction.

Stage/unstage affect the explicitly selected saved paths or section only. Commit pins all staged paths and
the configured author. Hunk actions use host-derived patch fragments, never a
caller-supplied patch. Discard affects one tracked saved file; it cannot delete an
untracked file. Stash creation selects tracked saved paths, without blanket stash
or untracked inclusion. Apply/pop identifies the exact stash; conflicts retain it.
Branch switch and non-merge revert are deliberate; conflicts expose Resolve/Abort.
Hard reset, force push, identity changes and automatic conflict choices are excluded.

Remote actions name the configured remote/branch/effect, use existing credential
helpers and never copy credentials. Fetch is deliberate; Pull is fast-forward-only.
Push checks the remote ref and local HEAD. An interrupted/uncertain push compares
the remote ref before allowing another action. Qualification uses disposable local
bare remotes, never the user's real repository or hosted publication.

Primary references: [status](https://git-scm.com/docs/git-status),
[diff](https://git-scm.com/docs/git-diff), [stash](https://git-scm.com/docs/git-stash),
[push](https://git-scm.com/docs/git-push). Implementation and actual checks are
retained in the [historical evidence](../IMPLEMENTATION_HISTORY.md#acceptance-records).
