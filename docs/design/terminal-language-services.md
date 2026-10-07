# Terminal and first-language services — milestones 18–19

The user authorizes milestones 18 and 19 as consecutive batches. Keep individual
brick commits. This contract freezes the initial implementation bounds before
dependency adoption; qualification records observed costs separately.

## Terminal ownership and bounds

Pin `portable-pty` 0.9.0 and `xterm` 4.0.0. Rust owns the actual PTY, shell and
cleanup; Flutter owns its emulator and view. Terminal first entry creates one
shell, plus snapshots selected Home project's root or OS home. Missing roots
refuse before spawning and expose Choose folder / Open at home / Retry. Existing
sessions retain initial ownership and any shell-changed working directory.

Allow eight retained sessions and four groups. Each PTY has a 1 MiB bounded output
queue; consumer reads at most 64 KiB, emulator keeps at most 5,000 lines. Drain
output even when hidden, pause the producer at the queue bound, and resume as
the UI drains. Input is limited to 16 KiB per write; resize 2–500 rows/columns.
ANSI and Unicode use the terminal emulator, with incremental UTF-8 decoding.
No clipboard escape sequence can read the clipboard or upload output. Selection
copy is explicit; Ctrl+C otherwise sends the terminal interrupt character.

One host owns sessions across page moves/splits. Close a live shell asks Keep open
or Stop and close; final quit includes live shells. Reap owned descendants; if
cleanup is uncertain, retain the tab and report it. Cold restart restores stopped
display/layout metadata, never pretends the old shell is running. Sharing selected
output chooses a conversation and adds a bounded draft snapshot without sending.
Human shells are separate from agent commands and receive no model credentials.

## First-language services

Discover existing `typescript-language-server` + TypeScript and `rust-analyzer`.
No server starts on Home or file open: an explicit language feature first starts
the server lazily for its project. At most two servers, one per project/language;
JSON-RPC frames up to 4 MiB, retained notifications up to 128, 15-second request
deadline, bounded stderr. Negotiate UTF-16 positions; use current unsaved buffer
versions, reject stale results and scope returned locations to the project.
Diagnostics, completion, hover, definition and references use the same owner.
Restart is explicit after crash. Missing tools retain ordinary editing.

Managed setup installs only pinned artifacts with integrity verification into
Dolores-owned storage, on explicit user action. Cancel/offline failure retains
the prior ready installation. Formatting/rename prepares an exact revisioned
preview; all affected text documents must be within the canonical project,
regular files, and within editor bounds. Refuse unsupported resource operations
or stale versions before applying. Apply must preserve undo/private recovery.

## Qualification and experimental windows

Record zero/one/two PTY and LSP observations, plus Stop/flood/crash/stale/offline
recovery, before calling the batch qualified. Initial adoption ceilings: added
idle host CPU below 1% of one core and retained terminal UI memory below 64 MiB
per session, measured on this host; server process memory is recorded separately.
Targets are not established by a dependency's advertised performance.

Milestone 19 uses one backend on the maintained Flutter SDK, at most two repair
iterations. Trial ceiling: one additional empty window adds at most 128 MiB
private memory and 1% idle core CPU. One app host owns profile, documents, runs,
PTY and language servers. Receive acknowledgement before source disposal. A
failed compatibility, ownership or resource gate leaves detach unavailable with
truthful Experimental status; it does not count as a transfer pass. Existing
Multiple Window preference and Windows keep-awake defaults remain unchanged.

Primary references: [portable-pty 0.9.0](https://docs.rs/portable-pty/0.9.0/portable_pty/),
[xterm 4.0.0](https://pub.dev/packages/xterm/versions/4.0.0),
[LSP specification](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/),
[Flutter desktop windowing](https://flutter.dev/blog/desktop-windowing-apis).

## Brick 18.3 display recovery

Keep at most eight stopped display records, 8 KiB UTF-8 plaintext per tab and 96 KiB per checkpoint. Checkpoints use the existing private workspace-state store under an app-owned key. Cold restore never spawns a process, restores a PID or replays input. A failed checkpoint blocks final quit until explicit retry succeeds. Selected output up to 8 KiB is attached only after choosing a conversation; it does not send a request.

## Brick 18.5 install and edit transaction bounds

Managed downloads pin typescript-language-server 6.0.1, TypeScript 6.0.3 and rust-analyzer 2026-10-05 (Windows x64). SHA-512 npm integrity and SHA-256 release-asset hashes are compiled into the installer. Download cap is 64 MiB per artifact, 120 seconds; tar expansion is 64 MiB and 2,048 regular files; the Rust executable is capped at 128 MiB. No lifecycle scripts or server commands execute during setup. Prior install pointers survive cancellation/offline/checksum failures. Startup checks retained member hashes.

Language edits review one to four regular project files and at most 1,000 replacements per file. UTF-16 ranges, saved-byte revisions and open-buffer versions must match. Apply is an atomic private checkpoint plus buffer publication; Save is separate. Undo language edit restores all reviewed buffers if none changed afterward. Native preview JSON has a dedicated 5 MiB request envelope for its 4 MiB edit bound; ordinary commands retain their existing limit. Preview tokens expire after five minutes and advance once. Resource operations and external project paths are refused.

Pinned source metadata: [TypeScript server](https://registry.npmjs.org/typescript-language-server/6.0.1), [TypeScript](https://registry.npmjs.org/typescript/6.0.3), [Rust release](https://github.com/rust-lang/rust-analyzer/releases/tag/2026-10-05).
