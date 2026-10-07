# Releases

Windows x64 is the only release target. Packages are unsigned portable ZIPs with
checksums, dependency notices and the changelog. Build results do not establish
model reliability or clean-machine acceptance.

## Record changes

Add a short user-visible bullet under **Unreleased** in [CHANGELOG.md](../CHANGELOG.md)
when behavior changes. Use Added, Changed, Deprecated, Removed, Fixed, Security or
Known limitations. Group related changes; omit commit IDs, session notes and
implementation diaries. Fixes explain the observed problem and resulting behavior.
Documentation-only edits and internal refactors need an entry only when they affect users.

## Prepare a version

1. Choose the version. Keep `Cargo.toml` workspace version, `package.json` and the
   Flutter version in `apps/dolores_flutter/pubspec.yaml` aligned. Flutter also has
   an incrementing build number. Refresh workspace package versions in `Cargo.lock`
   when changing them; review the lock diff so dependency upgrades stay deliberate.
2. Review Unreleased, including known limitations. Promote it with an explicit date:
   `python scripts/release.py prepare --version 0.1.0 --date YYYY-MM-DD`.
   This validates the existing versions, creates the dated section and leaves a new
   Unreleased section. It refuses duplicate versions and empty release notes.
3. Run `python scripts/release.py check --tag v0.1.0`, the contributor checks and
   Windows build/package workflow. Inspect the ZIP, checksum and dependency inventory.
   Verify extraction/launch on a clean Windows machine before calling that tested.
4. Commit the version and changelog, then create and push the matching `v0.1.0` tag
   when ready. Tag packaging uses only that commit and its dated changelog section.
   The Windows workflow creates a **draft** GitHub release. Review it before publishing.
   A GitHub release is not created for pull requests or ordinary branch pushes.

For branch/manual builds, CI uploads an unsigned Windows preview artifact without
publishing a release. `python scripts/release.py notes --version 0.1.0` prints the
exact version's notes; no notes are generated from commit messages or a model.
Failed checks stop packaging/release creation; correct the inputs and use a new
workflow run rather than reusing stale artifacts. Existing published tags are retained.

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

The Windows workflow uses read-only permissions for checks/builds. Only its tag
release job can create a draft release; it consumes the current run's verified
artifact. Fork pull requests use no secrets and no write permission. Actions and
the Flutter source revision are pinned; update pins deliberately and rerun the
checks. [GitHub permissions](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#permissions)
and [Flutter Windows prerequisites](https://docs.flutter.dev/platform-integration/windows/setup)
describe the upstream requirements.
