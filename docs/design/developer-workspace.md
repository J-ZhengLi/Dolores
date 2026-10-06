# Developer workspace specification — revised 2026-10-07

**Planning contract, not delivered behavior.** The user replaced mixed chat/file
panes with the familiar Home agent view and separate project-bound developer pages.
Implement one roadmap brick at a time. Earlier prototypes remain references; no
A/B/C selection is required for this new page structure. Review its navigation/
page prototype at 16.0 before UI rollout. Existing editor feasibility failures in
[qualification](../qualification/workspace-editor.md) remain open.

## Navigation and page layout

Use a ChatGPT-inspired primary sidebar in this order: **Home, Scheduled, Folders,
Source Control, Terminal**; **Settings** anchors at the bottom. Preserve Dolores's
infinity brand, palette, system appearance, message/composer and conversation
controls. Inspiration does not mean copying assets or exact dimensions. Primary
navigation is separate from each page's side panel. Keep familiar icons/hints,
quiet selected backgrounds and contextual actions; no permanent pane action bars.

| Page | Side panel | Main panel | Empty state / availability |
| --- | --- | --- | --- |
| Home | Current projects, sessions and recents | Current agent conversation, receipts, draft and controls | Existing New chat; chats are not editor tabs |
| Scheduled | Auto-created tasks and status filters | Next run, progress, result/error and run history | Create through chat; no New task form |
| Folders | Selected project's lazy file hierarchy | VS Code-inspired file tabs/editor and file-only splits | Open folder if no project is selected |
| Source Control | Selected repo, Changes/Staged/History | Inline/side-by-side Git diff and history | Open folder without a project; explain missing repo/Git |
| Terminal | Terminal sessions if useful | Interactive terminal tabs/splits and plus button | New terminal at selected root or OS user home |
| Settings | Existing categories | Current editors plus named new controls as implemented | Always reachable at the bottom |

Scheduled remains truthfully unavailable until 22; manual schedule creation is
deferred indefinitely. No unavailable page shows fake results or starts downloads.
Full VS Code extension parity, debugger, notebooks, remote/container development
and marketplace remain excluded.

## Selected project and retained work

Home's project chat or an explicit project picker sets the selected project; show
its name/root on Folders, Source Control and Terminal. A side chat clears selection
rather than borrowing a previous folder. Temporary folders may be selected
explicitly. Open folder creates/reuses a canonical project record and selects it;
it does not send a message, create a chat or grant model/tool access.

Navigation changes visibility, never the project/model/grants of existing work.
File focus does not select another project. Folders/Source Control retain tabs and
layouts per project: switching A to B hides A's views while preserving dirty
buffers. Equal names use project/path breadcrumbs. Cross-project splits in one
page are excluded initially. Home retains conversation scroll/draft; background
runs stay visible in sessions. No duplicate chat controllers or double sends.

Terminal plus snapshots selection at click time: selected root or OS home. Existing
terminals retain their original project/cwd, including a user-changed shell cwd.
Headers identify this basis. Missing roots offer Choose folder/Open at home rather
than silent fallback. Selecting B cannot retarget A's running shell.

Compact navigation/page panels use accessible drawers; one file/terminal group
can take focus while all groups stay reachable. Restore the wide layout on expansion.
Settings, Stop and native Close remain accessible. Native interaction checks are
separate from screenshot inspection.

## File tabs and editing

Slim strips have close/dirty markers, breadcrumbs and overflow menus. Drag to
reorder, move among file groups or edge-split; provide keyboard/menu equivalents
and resizable dividers. Preview tabs pin on edit/double-click and cannot replace
dirty content. Chats never join these groups. Source Control owns diff/history
tabs; Terminal owns terminal groups.

Provide highlighting, line numbers, indentation, clipboard, selection, undo/redo,
find/replace, go to line and quick open. Save is explicit; autosave starts Off.
Duplicate file views share one buffer/undo with independent cursor/selection/scroll.
Qualify that adapter: sharing the candidate's widget controller is insufficient.

Keep buffer, disk, Git index and commit distinct. Preserve UTF-8/BOM/line endings
and unchanged bytes; binary/unsupported encoding stays read-only. The proposed
5 MiB editable limit is unqualified; freeze after the spike. Larger files get
bounded preview. Clean outside changes reload; dirty changes retain both versions
and offer Compare/Reload disk/Keep edits. Keep edits requires a revision-checked
save against the reviewed new base. Failed writes, rename/deletion or interruption
retain private recovery and offer Retry/Save as/Restore; no silent stale overwrite.

File/selection attachment explicitly chooses a chat and source revision. Unsaved
edits are not automatically sent to the model; agent reads see saved bytes.
Deliberate create/rename/delete checks open buffers and canonical root/link scope.

## Source Control

Real Git state is the truth for working/index/history diffs; run receipts remain
separate linked evidence from Home. Support working tree/index, index/HEAD and
historical comparisons, inline/side-by-side views, paged history and rename,
binary, untracked and conflict states. Use NUL-delimited machine output.

Deliver status/diff/history, then whole-file stage/unstage/commit, then selected
hunks and stash/branches/discard/revert, followed by deliberate remote actions.
Serialize app-owned mutations per repo, recheck displayed HEAD/index/file bases,
keep hooks enabled and preserve commit drafts/index on failure. Conflicted stash
pop retains the stash. Discard file changes and Revert commit are distinct.
Branch/revert failures offer Resolve/Abort without destructive cleanup.

Fetch/Pull/Push show branch/remote/effect. Pull starts fast-forward-only. Reconcile
uncertain pushes before retry; no force push, hard reset or hidden rebase/conflict
choice. Reuse Git credential helpers, never store credential copies. Qualify with
disposable local bare remotes. Human Git actions do not expand model grants.

## Terminal and first-language services

Use real PTY input/output, ANSI, resize, bounded scrollback, copy/paste and Ctrl+C
(copy selected text, otherwise interrupt). Terminal tabs split independently from
files. Visiting Home does not spawn shells. Output sharing explicitly targets a
chat. Moving/hiding terminals preserves owners; busy close/final quit offers Keep
open/Stop and close. Reap descendants or report stopped/unknown honestly. Cold
restart restores a tab/display, not a dead shell. Human/model shells stay separate.

First LSP families remain TypeScript/JavaScript and Rust. Lazy project/language
servers provide diagnostics, completion, hover, definition/references; negotiate
Unicode positions and reject stale replies. Formatting/rename has a revisioned
preview. Discover existing tools before deliberate pinned installs; opening a file
does not download. Missing/offline support keeps editing usable. No model keys
enter shell/LSP environments.

## Experimental windows and keep-awake

Settings → Experimental contains **Multiple Window**, default On: a saved preference,
not automatic startup windows. Detach is unavailable until 19 qualifies it; preserve
explicit Off values. Use one bounded Windows backend trial on the maintained SDK,
without channel migration or extended comparison. Failure leaves useful single-window
pages. Freeze the investigation ceiling at 19.1: one backend, at most two repair
iterations, then record unsupported/blocked status instead of extending the trial.
The earlier SDK/library measurements are historical; reverify at adoption.

Detach file/diff/terminal groups first. Home remains the familiar agent view and
is not a draggable editor tab; extra Home windows are excluded initially. The
receiving view acknowledges transfer before source disposal. One host owns profile,
vault/runs/documents/PTY; windows never independently open/recover storage. Failed
transfer retains source. Off blocks new windows; existing windows rejoin with
acknowledgement. Restore safe bounds after monitor removal and recover dirty work.

**Prevent Windows From Locked** is the requested Windows keep-awake label; default
Off. Explain its actual display/system-awake effect and release on disable/exit.
It cannot promise prevention of secure screen savers, manual/lid/policy locks.
Do not change authentication policy, simulate input or leave permanent OS changes.
Report backend failure with Retry. Microsoft's
[execution-state contract](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-setthreadexecutionstate)
and [power-request lifetime](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-powersetrequest)
define the limit. These controls do not grant unattended task authority.

## Ownership and resource envelope

| Owner | Responsibility |
| --- | --- |
| App host | One profile/vault owner; run registry, queue and shared lifetimes |
| Project | Canonical root/ID, lazy watches/Git/LSP and mutation coordination |
| Conversation/run | Draft, immutable project/model/grants; approvals/Stop |
| Document | Buffer/history, encoding, disk revision and private recovery |
| Terminal | Initial project/cwd, PTY/process tree and bounded output |
| Page/window/view | Navigation, resource references, layout/focus/cursor/subscriptions |

16.1 separates today's selected-controller/single-active-run assumptions. Proposed
admission: two global primary runs, one mutating agent run per project, visible
cancellable queue; freeze after measurement. Globally unique run/call IDs route
approvals/Stop independently of visible page. Existing per-run limits/grants stay.
App locks do not prevent external writers; disk revisions still matter. No global
mutex across network/Git hooks/PTY/LSP. Page disposal only unsubscribes.

Rust owns scoped snapshots/writes/processes and disk revisions; Flutter owns editor
interaction/layout. Freeze bounded delta/version messages and document authority
in 16.0; avoid whole-file JSON on each keystroke. Lazy traversal excludes generated
trees; coalesce watches and recover overflow with rescan/Refresh. No eager global
index from visiting Home.

Keep the public corpus: A/B equal names/Unicode, UTF-8/CRLF/BOM, 1 MiB text, 100 KiB
line, 20,000-entry lazy tree, binary/unsupported/inaccessible files. Targets remain
warm ordinary open below 250 ms and typing frame p95 below 32 ms on the recorded
host. Post-large-file typing failed earlier; retain that failure. Freeze numeric
incremental memory and message limits from matched measurements before dependency
adoption. Measure one/two/four file groups and zero/one/two terminal/LSP instances,
then window overhead. Prior idle CPU, native IME/accessibility, other OS and low-end
qualification stay separate.

## Delivery and verification

16 delivers Home/navigation and Folders/editor/file splits; 17 Source Control;
18 Terminal tabs/splits and first-language support; 19 bounded experimental detach.
Each comprises separately tested/committed bricks. [ROADMAP](../ROADMAP.md) owns
scope/basic/failure/exit gates; [memory/scheduling/companionship](memory-scheduling-companionship.md)
owns the next capabilities. No timeline or full IDE parity is promised.

Use focused tests/recovery. Prefer saved renders/screenshots with `view_image` for
UX; use computer-use only for relevant interaction/native checks. No mandatory
visual check/build/launch after every task. Build normally when code/packaging
needs it; launch for relevant integration or explicit user request. Python helpers,
original-profile preservation and bounded Qwen/DeepSeek probes remain. CI 8.4 is deferred.

Primary component references retained as candidates:
[re_editor](https://github.com/reqable/re-editor),
[xterm.dart](https://github.com/TerminalStudio/xterm.dart),
[portable-pty](https://docs.rs/portable-pty/latest/portable_pty/),
[Git status](https://git-scm.com/docs/git-status),
[LSP](https://github.com/microsoft/language-server-protocol),
[Flutter windowing](https://flutter.dev/blog/desktop-windowing-apis),
[desktop_multi_window](https://github.com/Devolutions/desktop_multi_window).
