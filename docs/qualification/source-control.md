# Source Control qualification — milestone 17, 2026-10-07

## Scope and evidence

Milestone 17.1–17.6 implements lazy project-bound status, saved diff tabs and paged
history; reviewed file/hunk staging, commits, stashes, branches, discard/revert;
and deliberate Fetch, fast-forward Pull and ordinary Push. Home conversation
selection owns the project. There is no project picker or automatic Git mutation.
The frozen [contract](../design/source-control.md) retains all initial bounds.

The final full suites pass: **133 Rust bridge tests**, **287 Flutter tests**, strict
Clippy and Flutter analysis. Twelve Source Control native tests cover real temporary
repositories, malformed NUL status, missing repository, saved/index/HEAD/root-commit
bases, stale reviews/hunks, selective staging, hook refusal/retry, exact stash pop,
conflicted pop retention, branches/discard, reversing commit/Abort, local bare
remote push/fetch/pull, stale remote refs, divergence and injected uncertain-push
reconciliation. The byte reader enforces caps on the bytes actually read, including
file growth between metadata and read; errors cannot masquerade as absent blobs.

The release C ABI corpus (`scripts/test-source-control.py`) uses two disposable
repositories and a local bare remote. Nine checks pass for owner isolation,
30+6 pinned history pages, new-file review content, native unsaved-buffer refusal,
hook failure/fresh reviewed commit, selected stash/discard, local remote actions,
real hook cancellation with concurrent B reads, and no idle Git process. The
completed public record is ignored output
`source-control-17-0c6243fa56d64ab38f13a0f097246190/result.json`.

## Measurements and saved renders

The C ABI run observed status **587–961 ms**, one saved diff **1,911 ms**, history
pages **522–551 ms**, reviews **1,364–4,493 ms**, and successful applies
**1,266–2,613 ms**. Stop/reap of a real sleeping hook took **512 ms**, after which
no owned child remained and a fresh reviewed commit succeeded. Python-host private
memory increased **0.293 MiB** across this small corpus. These are observations
on this Windows machine, not latency targets or Flutter memory measurements.

The separately labeled diagnostic native release renders the production workspace
and Source Control widgets with a fresh public profile. Final v3 opens eight
files of **245,783 bytes** each, retaining eight read-only comparisons. Light/dark,
inline/side-by-side and 420×480 renders were inspected with `view_image`; tabs,
base labels, collapsed hunk choices, title-bar panel toggle and bottom Settings
remain readable. Review Cancel passes through the native bridge.

Observed private peaks were **378.418 MiB** before diffs, **347.406 MiB** with eight
retained diffs and **445.395 MiB** during compact renders/captures; working peaks
were **341.707**, **332.727** and **429.785 MiB** respectively. Four-second idle
phase CPU samples consumed **0.016 / 0.078 CPU seconds**, with zero owned Git/helper
processes at completion. Capture/render peaks include PNG/GPU work; lower retained
memory does not establish a negative incremental cost or a universal ceiling.
Artifacts: `output/source-control-17-renders-v3/`.

Earlier render v1/v2 used tiny files and mixed capture work into the purported
baseline; those CPU/memory comparisons are invalid. The release dirty-buffer
probe initially exposed ordinary versus Windows extended-prefix path mismatches.
Native editor/task guards and Flutter owner comparisons now normalize that
boundary, preserving stored project identities. A regression test retains diff
tabs across equivalent project paths; the final compact renders keep their tabs.
Earlier failed fixtures are retained as failed evidence, not acceptance passes.

## Practical limits

- Every mutation needs a fresh host-generated review. External writers can race
  the final operation; index locks and patch checks do not provide an OS transaction.
- Initial side-by-side compares full saved text with independent scrolling; inline
  shows the actual colored patch. There is no aligned merge editor, full graph,
  hosted review service, force push or automatic conflict choice.
- New/renamed/binary/mode-changing files use whole-file actions. Renamed stash,
  merge-commit comparisons, oversized views and larger branch/stash sets direct
  users to external Git. Initial branch/stash/remotes lists are bounded to 50/30/32.
- Commit drafts and uncertain-push warnings last for the current launch. Every
  fresh Push still reads the remote ref and refuses an already-present HEAD.
- Actual hosted authentication failure, real interrupted network publication,
  physical input/IME/accessibility, low-end/sustained/multi-platform resources and
  general model reliability are unqualified. No model calls were needed for these
  human Git flows, and no configured model/settings were changed.

The final normal `main` build/launch and original-profile preservation are recorded
separately in acceptance. Diagnostic renders do not establish normal-app startup.
