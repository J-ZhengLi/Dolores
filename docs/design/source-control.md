# Source Control contract — milestone 17

The selected Home conversation supplies the project. Source Control discovers
the enclosing Git worktree lazily, displays its canonical root and owns retained
views per repository. There is no project dropdown or automatic repository setup.
Git is a discovered direct executable outside the project. Human Git actions do
not grant model access. Home receipts link to repository views as separate evidence.

Status uses porcelain v1 NUL records, including rename pairs; filenames must be
valid UTF-8 to be actionable. Diff bases are saved working tree/index, index/HEAD
or an exact commit/parent. Unsupported binary/encoding/oversized views are labeled.
History pages pin HEAD and use exact commit IDs. Inline and side-by-side are
read-only; file editors keep their independent unsaved buffers.

## Final interaction refinement — 2026-10-07

Follow the user's VS Code reference: clicking a history commit toggles its nested
changed files; clicking one opens that exact commit's diff. Keep each commit's
loading/error/Retry and files attached to its row. Initially load history when
binding a committed repository; Refresh history and Load more remain explicit.
Retain at most eight commit file sets per repository, evicting only collapsed
sets; if all eight are expanded, ask the user to collapse one. Failed reads retain
usable cached files and the commit message, and never relabel another commit's files.

Place repository actions in the three-dot menu beside Changes: Commit, Pull,
Push, Fetch, and grouped Changes, Branch and Stash actions. Use familiar names
at entry points; show the exact existing host review before applying. Ctrl+Enter
in the message field also opens the commit review. File and commit overflow menus
contain their contextual actions, including Revert commit. Branch/stash/remote
selection uses a temporary picker; opening the menu performs no network request.
Keep the compact rail, Home project ownership, diff bases and editor guards.
This refines milestone 17; it does not add a full branch graph or new Git commands.

## Frozen initial bounds

| Item | Bound and recovery |
| --- | --- |
| Git jobs | Two executing globally, one per worktree; eight retained jobs; busy requests offer Retry |
| Process | Owned process tree, closed stdin, 30 seconds local / 60 seconds remote; Stop retains uncertain outcomes |
| Combined output | 512 KiB; overflow stops the owned tree and refuses partial parsing |
| Status | 2,000 changed paths within output bound; more offers external Git/filtered cleanup then Refresh |
| Revision checks | HEAD, index (16 MiB), status, stash refs, and changed saved files (16 MiB each / 32 MiB total) |
| Diff/text | 256 KiB patch or text per side, 8 KiB physical lines; unsupported views stay read-only |
| History | 30 commits per page, cursor at most 10,000; pinned HEAD, explicit Load more |
| Retained views | Eight repository owners, eight diff tabs each; inactive clean tabs can be closed |
| Mutation review | One host-generated, single-use token per repo, two-minute lifetime; exact operation and basis |
| Selected paths/hunks | At most 16 literal paths / 32 hunks, 64 KiB patch; stale basis refuses before apply |
| Commit draft | 8 KiB UTF-8; retained on hook failure; configured author and hooks remain enabled |

App mutations are serialized per repository and never hold the global chat lock
while Git, hooks or credential helpers execute. A busy repository refuses another
mutation rather than retrying it silently. Shutdown cancels and reaps owned jobs.
Dirty/pending editor buffers refuse conflicting operations; Save remains explicit.
Revision checks cannot prevent an unrelated writer racing the final Git operation;
Git index locks and patch checks remain additional defenses, not an OS transaction.

Stage/unstage affect selected saved paths only. Commit reviews all staged paths and
the configured author. Hunk actions use host-derived patch fragments, never a
caller-supplied patch. Discard affects one tracked saved file; it cannot delete an
untracked file. Stash creation selects tracked saved paths, without blanket stash
or untracked inclusion. Apply/pop identifies the exact stash; conflicts retain it.
Branch switch and non-merge revert are deliberate; conflicts expose Resolve/Abort.
Hard reset, force push, identity changes and automatic conflict choices are excluded.

Remote actions name the configured remote/branch/effect, use existing credential
helpers and never copy credentials. Fetch is deliberate; Pull is fast-forward-only.
Push reviews the remote ref and local HEAD. An interrupted/uncertain push compares
the remote ref before offering another review. Qualification uses disposable local
bare remotes, never the user's real repository or hosted publication.

Primary references: [status](https://git-scm.com/docs/git-status),
[diff](https://git-scm.com/docs/git-diff), [stash](https://git-scm.com/docs/git-stash),
[push](https://git-scm.com/docs/git-push). Implementation and actual checks are
recorded per brick in [acceptance](../ACCEPTANCE.md).
