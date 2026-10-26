# Releases

Windows x64 is the first supported release target. Releases provide an unsigned
EXE installer, SHA-256 checksum and reviewed changelog. macOS PKG and Linux DEB
packaging commands are experimental and excluded from tag releases until native
qualification. Build results do not establish model reliability or clean-machine
acceptance. See [installer contract](design/installers.md).

## Record changes

Add a short user-visible bullet under **Unreleased** in [CHANGELOG.md](../CHANGELOG.md)
when behavior changes. Use Added, Changed, Deprecated, Removed, Fixed, Security or
Known limitations. Group related changes; omit commit IDs, session notes and
implementation diaries. Fixes explain the observed problem and resulting behavior.
Documentation-only edits and internal refactors need an entry only when they affect users.

## Prepare a version

1. Review the user-visible notes and limitations under Unreleased.
2. Run one preparation command with an explicit date:
   `python scripts/release.py prepare --version 0.1.0-alpha --date 2026-10-26`.
   It aligns Cargo/Flutter/workspace lock versions, increments the Flutter build
   number when the version changes, promotes the reviewed notes to a dated section
   and leaves a new Unreleased section. Dependencies retain their versions.
   Invalid versions, duplicate releases or inconsistent inputs refuse before
   mutation; ordinary write failures restore the previous source bytes.
3. Run `python scripts/release.py check --tag v0.1.0-alpha` and the relevant tests.
   Commit the prepared files. Do not run `prepare` again for a version already
   recorded in the changelog.
4. Create and push the matching annotated tag at that reviewed commit:

   ```text
   git tag -a v0.1.0-alpha -m "Dolores 0.1.0-alpha"
   git push origin master
   git push origin v0.1.0-alpha
   ```

   The tag workflow validates source versions and dated notes before building,
   runs full checks/native fixtures, creates the installer and retains it both
   as an Actions artifact and on a published GitHub release. Alpha/RC tags are
   marked prerelease and do not become Latest. Publication follows successful native
   regression and installer lifecycle checks. Ordinary branch/PR checks never
   package or create a release. Manual **Release installers** runs create downloadable Actions
   artifacts; selecting a version tag also publishes its release.

`python scripts/release.py notes --version 0.1.0-alpha` prints the
exact version's notes; no notes are generated from commit messages or a model.
Failed checks stop packaging/release creation; correct the inputs and use a new
workflow run rather than reusing stale artifacts. Existing published tags are retained.

Before pushing, run the relevant direct checks from
[Contributing](../CONTRIBUTING.md#checks-and-recovery) using the active toolchain.
For a local installer, build with `desktop.py`, collect notices and package with
the dedicated tools described in [installer assembly](design/installers.md).
Keep logs and artifacts under a fresh `output/` directory; no local CI replica
or matching hosted toolchain is required.
For hosted packaging without retagging, choose **Release installers → Run
workflow** and select the branch containing the fix. A branch manual run retains
the downloadable installer artifact and does not create a GitHub release.

Tag CI extracts the reviewed dated section from that immutable commit; it does
not edit `CHANGELOG.md` or push a new commit while releasing. If a release already
exists, publication refuses rather than overwriting its assets. Review/recover
the existing release explicitly, or release a new version for changed source.

## Prepare the public history

Run `python scripts/check-publication.py` before committing. This checks tracked
files for known personal-path, credential and local-instruction patterns, printing
locations rather than values. Review new binaries and less predictable personal
content separately. Genuine credential exposure also requires revocation.

Session `HANDOFF.md` files stay untracked. To clean old revisions while retaining
commit history and author attribution, install `git-filter-repo==2.47.0`, commit
the reviewed tree, and run:

```text
python scripts/prepare-publication.py --directory <absolute-repository-path>/output/publication
```

The command creates a separate bare repository with all branch/tag history,
removes historical handoffs/local artifacts, cleans known personal text, and audits
reachable history. It verifies commit count and attribution, retains empty commits,
and changes commit IDs where content changes. Local tooling refs are excluded.
The original repository and its private recovery history remain intact. The copy
has no remote and nothing is pushed. Use that cleaned copy for the initial GitHub
upload; pushing the original repository would reintroduce removed history.

Keep cleanup receipts and the old/new commit map local. If collaborators already
have an older public history, coordinate replacement and fresh clones rather than
silently force-pushing. The pattern audit is not proof that all possible personal
content or secrets have been identified.

## Workflow maintenance

The Checks and Release installers workflows use read-only permissions for
checks/builds. Only the tag publication job can publish a release; it consumes
the current run's verified artifact. Fork pull requests use no secrets and no write permission. Actions and
the Flutter source revision are pinned; update pins deliberately and rerun the
checks. [GitHub permissions](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#permissions)
and [Flutter Windows prerequisites](https://docs.flutter.dev/platform-integration/windows/setup)
describe the upstream requirements.

Checks use the runner's active Rust toolchain, the selected Flutter SDK and shared
dependency/compiler caches. They run
formatting, Clippy, Rust/browser/script tests, Flutter analysis and widget tests
without a native release build, notice collection or installer compilation.
Only tag/manual release runs perform the normal Windows release build and native
fixtures. Latest branch/PR runs cancel obsolete checks; release runs are retained.
