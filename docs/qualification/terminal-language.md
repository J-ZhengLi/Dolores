# Terminal and language-service qualification — 2026-10-07

Milestone 18 delivers real local shells, independent draggable terminal splits,
retained stopped output, explicit selected-output sharing and lazy first-language
services. This report separates automated/programmatic evidence from physical
keyboard, IME, accessibility and sustained resource qualification.

## Exercised behavior

The release native corpus opened PowerShell in two separate projects and HOME,
including a Unicode path, resized the PTY, received Unicode output and interrupted
a foreground Node command while preserving the shell. Four MiB of output exercised
the one-MiB queue and 64-KiB polling bound. Stop removed all six observed owned
processes, including descendants; no late child write occurred. Stopped checkpoints
restored no live process, and sharing selected text changed only the chosen draft.

The upstream pinned TypeScript server 6.0.1/compiler 6.0.3 and rust-analyzer
2026-10-05 were downloaded and verified in disposable private directories. Later
corpus repetitions reused those receipts and verified retained member hashes.
TypeScript returned three diagnostics, completion, hover, definition and four
references against unsaved Unicode text. A two-file rename passed preview, Apply
and Undo. Stale results were rejected. Rust returned hover in its isolated project
with build scripts disabled. A transient analysis response retained editing and
allowed an explicit bounded retry.

Qualification found and corrected three integration defects: PowerShell required
standard PATHEXT for bare executable discovery; TypeScript emits unversioned
diagnostics and normalizes Windows file URIs; the pinned Rust archive includes an
optional PDB. Unversioned diagnostics are visibly identified as potentially lagging
and remain subject to current-buffer/project checks. Unknown archive members still
refuse installation, and debug symbols are not installed.

The production Flutter terminal corpus exercised two actual PTYs with full
5,000-line scrollback, moves without duplicated owners, compact/wide light/dark
renders, Stop and checkpoint. Saved renders were inspected. Shell labels now use
short names and stopped state precedes CWD so narrow headers retain that status.
This corpus used programmatic input, not physical keyboard automation.

Configured DeepSeek V4.1 Flash read one saved fixture and performed one approved
`node calculate.cjs 7` call, producing marker `SAVED_ONLY_18` and answer 42.
The native editor held a distinct unsaved marker that was absent from model input
and output. The run took 6.53 seconds and 633 output tokens. Both requested tools
completed; no file mutation or retry occurred. All 42 original profile table hashes
remained unchanged. Credentials and private conversations were excluded from
artifacts.

## Resource observations and limits

| Sample | Observed cost | Interpretation |
| --- | --- | --- |
| Cold native host | 0–0.521% of one core; approximately 12–13 MiB private | No PTY or language process before explicit use |
| Two real language families | 0.104% added host CPU; 16.39 MiB host private; 777.38 MiB owned language process tree | Compiler descendants included; not a claim of low total language memory |
| First Flutter comparison | 323.17 to 362.41 MiB private; 19.62 MiB added per terminal; added CPU -24.154 percentage points | Short settled samples meet incremental envelopes |
| Final Flutter comparison | 420.48 to 380.66 MiB private; -19.91 MiB per terminal; added CPU -25.858 points | Allocation/startup variation; negative values do not establish memory savings |

Absolute Flutter idle samples remained high and variable (approximately 24–50% of
one core). Sustained idle efficiency is an acceptance gap, despite these incremental
samples. Physical input/IME, exhaustive native focus, accessibility, low-end hosts
and other platforms remain unqualified. Snippet/additional-edit completion,
arbitrary language servers, debugger and remote development are outside this brick.

## Checks and artifacts

146 bridge library tests and 302 Flutter tests pass. Flutter analysis and strict
bridge Clippy pass. The maintained normal Windows release built and launched
visibly with the original profile; all 42 table hashes were preserved.

Disposable public evidence is retained under `output/terminal-language-qualification/`
(final report `2b5590ae-b83e-487c-ada9-18101fb58086/report.json`) and
`output/terminal-desktop-qualification/` (final renders
`285fb71fe1d14a65ad2a537ca8781366`). These are diagnostic receipts, separate from
the normal release launch. Reproduction helpers are
[native corpus](../../scripts/test-terminal-language.py) and
[desktop corpus](../../scripts/test-terminal-desktop.py).

Follow-up: [Windows keyboard repair](terminal-keyboard.md) reproduces the Enter-only
input failure and qualifies ordinary/shifted letters through native event injection.
It corrects the missing text-input view ID; physical IME and the wider gaps above
remain open.
