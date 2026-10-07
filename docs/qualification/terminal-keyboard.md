# Windows terminal keyboard repair — 2026-10-07

## Failure and cause

The user's reported behavior was reproduced in a release Windows diagnostic using
the production `TerminalHost` and `TerminalPage`, with a non-executing fake bridge.
The terminal retained focus and Flutter received an `a` key event, but no text
reached `TerminalHost.input`. Enter produced exactly `\r`. No PTY, command,
provider request or user profile was involved.

xterm 4.0.0 creates `TextInputConfiguration` without `viewId`. The maintained
Flutter 3.47.5 Windows engine rejects that configuration before creating its text
model. Ordinary characters rely on that model; Enter is handled directly by
xterm's keyboard path. See the pinned [Windows engine implementation](https://github.com/flutter/flutter/blob/6a19cca564/engine/src/flutter/shell/platform/windows/text_input_plugin.cc#L241).
The widget test originally allowed a client with a null view ID and therefore
missed native rejection. Adding the actual view-ID assertion produced a failing
regression (`Expected: 0; Actual: null`) before the repair.

## Repair and checks

The application now uses a checked-in xterm 4.0.0 snapshot with one upstream source
line added: `viewId: View.of(context).viewId`. All other 78 upstream files match
the published package byte for byte. The original MIT license and local patch
notes are retained. This fixes attachment without disabling IME, changing shell
ownership or adding a duplicate hardware-character path. Machine-wide package
and Flutter caches were not patched.

The same native fixture, rebuilt with the repair, received exactly `a`, `\r`, `B`
from Shift+B and `\x7f` from Backspace. The typed letters rendered in its terminal.
The before/after receipts are `output/terminal-input-qualification/before/input.json`
and `output/terminal-input-qualification/after/input.json`. Both report zero shell
processes and zero model requests. These are native event-injection checks against
the production terminal widget, separate from a normal-app PTY end-to-end claim.

Nine focused terminal tests and all 305 Flutter tests pass; analysis passes.
The regression verifies the containing view ID before typing and after returning
from Home. Edge coverage verifies split-pane focus routes text only to its owner,
IME composition sends nothing prematurely and commits Chinese text once, Unicode
text remains intact, Enter/Backspace/Left keep their escape sequences, and a stopped
shell closes text input while retaining output. Returning to the remaining live
pane restores input. Previous real PTY/native transport qualification remains in
[terminal and language qualification](terminal-language.md).

## Reproduction and remaining limits

Build `python scripts/desktop.py build --diagnostic terminal-input`, set
`DOLORES_TERMINAL_INPUT_DIR` to an absolute disposable directory under `output`,
then launch using `python scripts/desktop.py launch --allow-diagnostic` with
absolute isolated `--data-directory` and `--pid-file` paths. In the window titled
“Dolores non-executing terminal keyboard fixture”, use A, Enter, Shift+B and
Backspace; inspect `input.json`. The fixture cannot execute any typed command.
Stop only its owned process record before rebuilding and launching normal `main`.

Physical IME candidate selection and accessibility remain unqualified. A separate
Sky `type_text` Unicode attempt emitted no character events and inserted no text;
it does not establish native Unicode/IME success. Composition/Unicode assertions
above are deterministic text-input tests. Exhaustive native focus, other platforms
and sustained idle qualification remain open. This repair changes no numerical,
model, sharing or approval defaults.

The normal Windows main release built and launched visibly with the original
profile after the diagnostics were stopped by their owned process records. All 42
profile table hashes matched across that launch, including provider configuration,
history and recovery. Receipt: `output/terminal-input-qualification/normal-handoff.json`.
