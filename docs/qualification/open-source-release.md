# Open-source and Windows release preparation — 2026-10-08

Brick 8.4 adds Windows-only checks and unsigned portable packaging. Version tags
require aligned versions and dated changelog notes, then create a draft release.
No repository, tag or release has been uploaded by this preparation.

## Local results

| Check | Result and boundary |
| --- | --- |
| Rust | Formatting and strict workspace/all-target Clippy pass; 436 tests pass, 2 ignored. |
| Flutter | Clean analysis and 341 tests pass; normal Windows main entry builds in an isolated checkout. |
| Native persistence/recovery | All eight save/restore pairs pass, 16 stages, zero live requests. Includes feedback, context, continuation, generation profiles, command repair, exact edits, memory and skill drafts. |
| Python | Publication 3, release 2, package 4, launcher 1, regression-runner 2 and browser-installer 3 checks pass. |
| Browser | Both real-browser synthetic-page checks pass; no personal page or account is used. |
| Workflow | actionlint 1.7.12 passes. Hosted GitHub execution is pending the initial upload. |
| Documentation | Both root READMEs and their image/local links are checked. Chat and Files use current widgets with synthetic data; Source Control and Memory renders are also inspected. |
| History | A cleaned trial retains all five branches, commit count and author/committer attribution, with no known-pattern findings or historical handoff filenames. Final copies include committed work only and receive a fresh audit/receipt. |
| Original profile | Schema 38 and all 45 original tables retain their row values. No schedule, companion or background rows are added. The normal app process is preserved. |

The local toolchain is Flutter 3.47.5/Dart 3.13.4, Rust 1.95.0, Python 3.13 and
Visual Studio 2026 through the normal desktop helper's CMake fallback. Hosted CI
uses Windows 2025 and Visual Studio 2022; local results do not qualify that runner.

## Package and startup

The verified `Dolores-0.1.0-1-windows-x64.zip` is **24,820,665 bytes**, with 18 runtime
files, 8 guidance/notice files and a per-file manifest. Its SHA-256 is
`4ed7e62c8c5171da160c6c19d8b28d58f9872da130e6136a7916785c0fa66e25`.
The conservative dependency inventory contains **352 components**, not a count
of binary-linked packages. The native update launcher, changelog and full notices
are included; optional installed browser modules are excluded.

ZIP checksum, exact-member and per-file checks pass. Fresh extraction passes
`Start-Dolores.cmd --check`; the extracted native bridge loads and bootstraps an
empty isolated database, then shuts down. A separate normal app launch through
`scripts/desktop.py launch` opens a window using an empty profile, followed by
`stop-owned`. Window presence and bridge bootstrap do not establish clean-machine
installation, physical keyboard/IME/accessibility or screenshot-level UX acceptance.

## Failures and explicit recovery

- Earlier Rust examples shared `probe.exe` output. Unique names remove that collision.
- A borrowed local Cargo target first prevented helper copying; the isolated
  checkout's target junction restores the helper's expected layout. Hosted CI uses
  its ordinary checkout target and does not need this local arrangement.
- Current Flutter no longer creates root `native_assets.json`. Packaging and launcher
  preflight now use the actual bundle; the asset NativeAssetsManifest remains required.
- Clearing generation settings restores provider-default output allowance. The
  obsolete 2,048-token fixture expectation is corrected and verified.
- The command-repair fixture expected the retired four-call default. It now explicitly
  saves a bounded task policy and verifies failure, Stop, stale-edit and repair recovery.
- Wasmi family 0.46.0 and wasmparser 0.228.0 omit license files in published crates.
  Full upstream texts are frozen at each crate's recorded revision, including LLVM
  exception text and provenance. Packaging hashes bind those inputs to the inventory.
- The first cleaned-history audit caught wrapped older model guidance. The cleanup
  now handles those forms; a second trial passes. Session handoff paths are excluded
  throughout reachable branch/tag history, including renamed design handoffs.

Failed logs/artifacts remain in ignored output. Checks were repeated only where
the failure or changed inputs required recovery; no automatic model retry occurred.

## Evidence and remaining gates

Local evidence is retained under `output/ci-check-source/output/`: the complete
check log, the recovered 16-stage report, package log, notice inventory, ZIP and
extraction receipt. Publication audit/attribution receipts belong under
`output/github-publication/`. These private evidence directories are excluded from
the publication copy. Historical implementation evidence is retained as historical
observations, without requiring contributors to reproduce a personal provider setup.

The original repository keeps private recovery history. Only the separate cleaned
copy should be uploaded. Author attribution is deliberately retained. The pattern
audit covers known paths, credentials and instructions; it is not a universal
personal-data classifier. Hosted workflow execution, signing, clean-machine startup
and the existing platform/input/model qualification gaps remain open. Follow the
[release procedure](../RELEASING.md) for version notes, tags and final review.
