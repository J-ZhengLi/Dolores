# Windows input and accessibility checks

Brick 8.3 improves the existing Flutter composer without adding an editor engine or changing its theme. Heading, code and paragraph fields expose their type and block position as accessible names, alongside their editable value and focus. The code-language button exposes its language and enabled state. This is semantic-tree evidence, not a screen-reader speech certification.

Send and code-language changes wait while the native input value has an active composing range. After the IME commits, those actions become available again. This prevents submitting unfinished candidates or replacing their composing range through a language change. Stop remains available during a response.

## Repeatable checks

From `apps/dolores_flutter`, run:

```powershell
flutter test --no-pub test/input_accessibility_test.dart
```

The four widget cases check accessible names/values/focus, Windows whole-draft selection/delete/undo, Send during active Chinese composition followed by a successful committed send, and a disabled language menu followed by a committed language change that preserves text and focus. Composition is injected at the platform text-input seam; these cases do not exercise a real candidate window.

Native desktop checks use isolated application data, synthetic unsent drafts and the normal release executable. No provider requests are needed. Observe settled state after each key: an immediate snapshot can precede a focus or selection update.

| Flow | Required result | Current evidence |
| --- | --- | --- |
| Type heading, insert a newline, type a fence and code | Formatting folds and typing continues without clicking again | Native keyboard smoke passed |
| Shift+Enter; Down from the last code line | Newline stays editable; Down moves to following prose | Native keyboard smoke passed |
| Select all, Backspace, undo across heading/code/prose | Entire canonical draft is removed and restored | Native right-Control smoke and Windows widget case passed |
| Tab into Model connection; Escape | A real field receives focus; dialog closes without saving | Native smoke passed |
| Inspect composer accessibility nodes | Named editable blocks and a named language button | Native tree and widget case passed |
| Physical left Ctrl+A, Ctrl+X/V/Z and Shift+arrows | Canonical whole-draft shortcuts; local selection remains native | Physical keyboard check pending |
| Chinese IME candidates in heading/code/prose | Enter commits a candidate without sending; continued typing, Shift+Enter and block exit work after commit | Real IME check pending; composing-value guards covered by widgets |
| Narrator reading and Tab traversal | Sensible spoken names/values, focus order and reachable actions | Spoken screen-reader check pending |
| Compact layout, both themes, higher display scale | Readable fields and reachable footer/menu without clipping | Existing compact/theme widgets pass; native display-scale check pending |

The native automation helper's left-Control chord delivered the letter event with Control already released. A temporary key-only probe established this discrepancy; right-Control delivered the modifier correctly. The probe was removed before the final build. Do not treat the injected left-Control failure as a confirmed physical-keyboard bug or use the right-Control smoke to certify physical left-Control behavior.

## Manual follow-up

Use a new unsent draft rather than a private transcript. Create a heading, a Rust code card and a prose paragraph. With a physical keyboard, select all from each block, copy/cut/paste and undo; remove an empty heading/card with Backspace. Enable a Chinese IME and repeat continuous typing, candidate commit, newline and Down exit in each block. Confirm that clicking Send or the language menu during composition preserves the draft and that both work after commit. Finally traverse controls with Tab and a screen reader at normal and enlarged display scale. Record the Windows/IME/screen-reader versions and actual failures before declaring these remaining gates accepted.
