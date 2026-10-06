# Developer workspace specification

**Resumed — 2026-10-06.** The user authorized milestones 16–19, one milestone
batch at a time, starting with 16. The 16.0 prototype and editor trial are in
progress; production workspace features are not delivered. The user requested
VS Code-style draggable tab strips after inspecting the first prototype: remove
the pane action bar, keep tab close/overflow controls and breadcrumbs, and support
reorder, move and edge split. Review the revised prototype before UI rollout.
Flutter/Rust, the existing visual identity and the self-evolving harness vision
remain the foundation. Platform CI (8.4) remains explicitly deferred.
The [16.0 qualification record](../qualification/workspace-editor.md) preserves
the revised prototype and failed post-large-file typing target; the candidate is
not adopted and resource/input/ownership gates remain open.

## Everyday outcome

Open two projects in Dolores, ask each agent to work, inspect and edit their files,
review the real Git changes, run commands and commit from the same application.
Arrange each project's chat beside its code, or move views onto another monitor.
Switching views must not change a running task's project, permissions or model.

The first useful release is a single-window workspace with an editor and split
views. Git, terminal/language support and detached windows then have separate
exit gates. Existing learning and computer-use acceptance gaps remain visible in
the [reliability ledger](../qualification/remaining-work.md).

## Interaction design

### Feature rail and project panel

A narrow left rail contains **Chats**, **Files** and **Git** icons with hover/focus
hints and accessible names. A single **Settings** gear remains at the bottom left.
Use the existing icon family, infinity brand, palette and soft selected background;
avoid decorative colored left edges or a second app name in the title strip.

The adjacent resizable panel changes with the selected feature:

| Feature | Panel | Main-area action |
| --- | --- | --- |
| Chats | Existing projects and session list, with running/needs-attention state | Open the conversation as a tab |
| Files | Active project's folders/files; quick file search | Open a file tab; create, rename or delete deliberately |
| Git | Active repository, branch, changed files and history | Open a diff/history tab or perform a named operation |

Focusing a project-owned view establishes the active project for Files/Git and
new terminals. The project name is visible in the panel header. Users can explicitly
pick another open project there; doing so does not retarget already-open views.
Paths in tabs, confirmations and terminal headers disambiguate equal file names.
Side chats remain folderless; Files/Git offers Choose project without silently
binding that chat. Temporary working sessions can browse their existing folder.
A folder without a repository remains fully editable; Git offers a deliberate
Initialize repository action instead of initializing on navigation.

On narrow windows, the rail opens the existing drawer behavior and one pane takes
focus. Other panes remain reachable as tabs; shrinking cannot lose documents or
collapse the persisted wide-window layout permanently. Keep the themed title bar
and native window controls reachable independently of pane focus.

### Tabs and panes

Chats, files, diffs, history and terminals are view types in one workspace, rather
than unrelated modal dialogs. Each pane owns its tab strip. Drag to reorder within
a strip, to move into another pane, or to an edge to split left/right/up/down.
Show a quiet drop preview. Provide equivalent tab-menu/keyboard actions; drag is
never the only route. Split dividers are resizable, resettable and keyboard usable.
Do not place a permanent move/split/save action bar below every tab strip.
Use close buttons and a compact pane overflow menu; file actions are contextual
and Save remains available through Ctrl/Cmd+S. Breadcrumbs disambiguate paths.

The user's reference arrangement is a required acceptance case:

```text
rail | feature panel | Project A chat | Project A file
                     |---------------|---------------
                     | Project B chat | Project B file
```

Single-clicking a file uses one replaceable preview tab per pane. Editing or
double-clicking pins it. A dirty tab shows a small unsaved marker and cannot be
replaced silently. Tabs keep cursor, selection, scroll and draft state when moved.
Duplicated file views share one document buffer and undo history; duplicated chat
views share one session state, draft and run, not independent sends. Focus belongs
to the active view. Closing a chat tab does not delete its conversation or replay
or cancel its work; its running status remains in Chats.

Terminal opens from a short toolbar/command action, initially in a bottom pane;
it can later move like another tab. Keep common actions small and contextual.
Retain icon-only chat header actions with hints. Do not add a permanently visible
row of developer status controls to every conversation.

### Editing and collaboration with the agent

Provide syntax highlighting, line numbers, indentation, undo/redo, selection,
find/replace, go to line and quick file opening. Ctrl/Cmd+S explicitly saves;
autosave starts off. Preserve UTF-8/BOM and existing line endings, and do not rewrite
unmodified files. Unsupported encoding or binary data gets a truthful read-only
view rather than lossy decoding and a dangerous Save button.

The editor buffer, disk file, Git index and committed file are distinct versions.
Unsaved changes stay local and are not automatically sent to a model. Attach a
selection or file to a chosen conversation explicitly, with its source revision.
An agent reading the filesystem sees the saved version; the UI makes that clear.

For each saved document, retain the base disk fingerprint. If the agent, terminal
or another application changes a clean file, reload it while preserving reasonable
view state. If the buffer is dirty, retain both versions and offer Compare / Reload
disk / Keep my edits. Keeping edits requires a deliberate comparison-backed save
against the new base; Save may not silently overwrite a stale disk revision.
Handle external rename/deletion without discarding the buffer. Back up dirty
buffers in private app data with an explicit restore/discard flow after a crash.
Do not commit recovery contents or expose them in routine diagnostics.

Agent changes and editor changes use the same disk change notifications. Existing
tool receipts remain attributable evidence; they are not a substitute for Git's
current working-tree/index diff. The Changes action can open the corresponding
workspace view while retaining links to the original run evidence.

### Git workflows

The default Git view shows Changes, Staged changes and History. Support working
tree versus index, index versus HEAD, and historical commit/file comparisons.
Inline and side-by-side diffs are selectable. Preserve rename, binary, untracked
and conflict status. History is paged, with parent/branch information and file
history; a full graphical history explorer is not an initial dependency.

Deliver whole-file stage/unstage and commit first, then selected hunk operations,
stash/apply/pop, branch switching and commit revert. Use **Discard file changes**
for restoring a working file and **Revert commit** for creating a reversing commit;
never use one ambiguous Revert label for both. Stash lists include the target
repository and preview; a conflicted apply/pop preserves the stash and provides
an actionable conflict view. Hunk operations recheck the displayed patch basis.

Remote actions are separate Fetch / Pull / Push controls. Show branch, remote and
outgoing effect before a push. Use configured Git credential helpers without
copying credentials into the app database. Initial Pull uses fast-forward-only;
divergence offers an explicit next step, not an automatic reset or rebase.
Keep Git hooks enabled and show bounded useful failure output. A timed-out push
is an uncertain remote effect: inspect remote refs before offering a retry.
Force push, destructive reset and automatic conflict resolution are excluded.

Direct user actions in the editor/Git/terminal are not model tool grants. Model
operations retain the existing host policy and reviews. No AI request may stage,
push or expand access simply because a human-visible control now exists.

### Terminal and language support

Use a real interactive PTY, with shell input, ANSI output, resize, scrollback,
copy/paste and Ctrl+C behavior. Each terminal retains its original project/cwd;
switching the active pane never changes an existing shell's directory. Selecting
text then Ctrl+C copies; without a selection it interrupts the foreground process.
User shell sessions and model run_command receipts remain separate. Terminal
scrollback is not included in prompts unless explicitly attached.

Closing a busy terminal or quitting with live work makes the effect clear and
offers Keep open / Stop and close. A stopped shell must reap owned descendants;
an uninterruptible child gets an honest stopped/unknown state. A restart can
restore the tab and bounded saved display, not claim that a terminated process
is still alive. Ordinary runs continue when tabs move or detach; final application
shutdown follows explicit work/dirty-document checks.

**Confirmed first language families: TypeScript/JavaScript and Rust.** Initial LSP
features are diagnostics, completion, hover, go to definition and references.
Formatting and rename follow with a preview of affected files and revision checks.
Language servers see the current buffer but receive no model API credentials.
Discover existing project/toolchain installations first; if missing, show a short
Install language support action with a reviewed pinned managed installation.
Opening a file alone cannot start a download. Offline editing still works.
Routine setup must not require users to write command strings or configuration JSON.

### Experimental options

The user requested a clearly labeled **Experimental** Settings page containing:

- **Multiple Window**, enabled by default. This is a saved user preference;
  enabling it does not open another window on startup. Before milestone 19's
  backend exists, show its availability truthfully. Existing explicit Off values
  survive upgrades. Off prevents new detach/create actions; existing detached
  views must rejoin with acknowledgement before their windows close.
- A Windows keep-awake option requested as **Prevent Windows From Locked**.
  Default Off; scope is the running Dolores application. The implementation must
  disclose the actual supported effect: keep the display/system awake, release
  the request when disabled or the host exits, and report a failed request.
  It must not promise to prevent secure screen savers, manual locking, lid-close
  sleep or organization-enforced locks. Do not rewrite authentication/security
  policy, simulate user input, or leave a permanent OS preference change.

Introduce the page with workspace navigation in 16.2; qualify the window backend
and activate detach controls in 19. The user prefers limited effort on experimental
multi-window support: one bounded Windows backend trial, with honest unsupported
states, rather than an extended backend comparison or an SDK channel switch.
Experimental status does not waive document preservation, one-host ownership,
acknowledged transfer or cleanup. See Microsoft's
[execution-state contract](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-setthreadexecutionstate)
and [power-request lifetime](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-powersetrequest).

## Architecture and boundaries

### Current fit and required changes

The current Flutter app constructs one ChatController and NativeBridge. ChatController
owns the selected session/workspace and blocks selection while busy; the Rust
Engine also guards one active run. Adding tabs alone cannot make two projects run
independently. Existing typed plugins, session/run persistence, scoped permissions,
off-UI bridge work and change evidence can be reused, but ownership needs refactoring
before concurrent chats or detached windows are safe.

The editor is a human UI capability, not a new model tool. Keep its view plugin
separate from file/document services and host access policy. Protected host services
remain outside self-editable mods; neither an editor extension nor a learned skill
can replace the permission gate, document write validation or run scheduler.

### Proposed ownership model

| Owner | State/responsibility |
| --- | --- |
| App host | One profile/storage/credential owner; run registry and bounded scheduling; shared service lifetimes |
| Project | Stable ID/root; file watches, repository service, lazy language servers and scoped task coordination |
| Conversation/run | Thread/run IDs; immutable project/model/permission snapshot, approvals, continuation and cancellation |
| Document | Canonical project/path ID; buffer/undo, encoding, dirty state, disk revision and private recovery |
| Terminal | Terminal ID, immutable project/cwd, PTY/process tree and bounded scrollback |
| Window/pane/view | Layout, focus, tab references, cursor/scroll and subscriptions; no independent database recovery or credentials |

Use a versioned binary split tree whose leaves are tab groups. A ViewRef names its
kind and resource ID; it does not copy live ownership. Commands carry explicit IDs
and expected revisions, never rely on whichever project happens to be focused.
View disposal unsubscribes; only the app host shuts down shared services.

The proposed initial scheduler permits two active project runs globally, one
mutating agent run per project; additional work is visibly queued and cancellable.
Freeze these new limits in 16.1 after measurements. Existing per-run/model budgets
remain unchanged. Stop and approval route by globally unique run/call IDs. Duplicate
panes cannot double-send, approve another run or reset a continuation allowance.
Within a shared root, queued agents avoid competing writes; humans and outside
tools can still modify files, so revision conflicts must be handled rather than
claiming a universal filesystem lock. Automatic worktree creation is not included.

Rust owns file snapshots/writes, Git subprocesses, PTY lifetimes and LSP supervision
behind typed ports. Flutter owns rendering, buffer interaction and layout. Define
bounded revision/delta messages so every keystroke does not reserialize an entire
large document through the current JSON bridge. Decide buffer authority and crash
recovery in 16.0; one resource owner must be authoritative across views/windows.
Background operations use bounded workers and coalesced event subscriptions.

Serialize app-owned Git mutations per repository and revalidate HEAD/index/file
basis before applying a reviewed action. Agent Git commands must participate in
that coordination; external shell commands remain detectable outside writers.
Do not hold a global host mutex across network, shell, language-server or Git hook
work. Closing a pane cannot invoke NativeBridge.close/shutdown on shared work.

Project traversal remains canonical-root scoped, including Windows case handling,
symlinks/junctions and nested repositories. Expanded folders load lazily. Watches
exclude generated trees by default and rescan after overflow; manual Refresh remains
available. Model filesystem exposure continues through its existing tool policy.

### Component choices and feasibility gates

| Area | Preferred direction | Decision gate |
| --- | --- | --- |
| Editor | Flutter-native editor adapter; evaluate re_editor first | Real IME, shortcuts, long lines, large buffers, undo and LSP edits must pass before adopting/pinning |
| Git | Installed Git CLI behind typed Rust service | Machine-readable output, path safety, hooks, credentials and cancellation; no parsing localized human status |
| Terminal | Flutter terminal emulator plus Rust PTY adapter | Evaluate xterm.dart and portable-pty with Windows resize/interrupt/cleanup; candidate packages are not yet dependencies |
| LSP | Standard protocol, lazy supervised server per project/language | Correct negotiated Unicode positions, revisioned replies, bounded messages and managed install/recovery |
| Docking | Small view-agnostic layout model plus Flutter adapter | Four panes, dirty buffers, keyboard moves and restoration; library only if it meets ownership/theme constraints |
| Detached windows | Shared app host with interchangeable window backend | Pinned SDK support, focus/IME, transfer correctness and measured per-window cost before selection |

Avoid building an editor engine from scratch. re_editor exposes editing,
highlighting and search/replace primitives; it does not establish Dolores's LSP,
save-safety or accessibility acceptance. A web editor is a fallback only if native
editor qualification fails; it requires an explicit resource/packaging comparison.
Do not silently add a WebView/browser runtime to every file tab.

The local maintained SDK reports **Flutter 3.47.5 stable**. Flutter's official
windowing article currently describes an experimental main-channel API. The
desktop_multi_window package documents separate Flutter engines per window.
Neither source establishes acceptable Dolores resource use on the pinned SDK.
Do not switch to main or promise cheap detached windows without a spike. If a
community backend is chosen, all engines attach to one host rather than each
opening/recovering the profile database. Cross-process backends require bounded
local authenticated communication and stale-client handling, not a public server.

## Qualification and resource envelope

These are proposed targets, not achieved measurements. Freeze the corpus and
limits before runtime work, with actual machine/SDK/build identity in acceptance.
The previous sustained idle-CPU gap remains open; compare against the same normal
build/profile rather than claiming it was solved by adding an editor.

- Synthetic projects A/B: equal-named files, ordinary UTF-8 and CRLF/BOM, an emoji
  and non-Latin path, a 1 MiB text file, a 100 KiB single line, and a 20,000-entry
  tree whose unopened folders do not render eagerly. Include unsupported encoding,
  binary data and an inaccessible file with useful recovery.
- Proposed initial automatic editable-file limit: 5 MiB. Larger files get bounded
  read-only preview and an explicit explanation; do not load entire binary/huge
  files on click. Freeze this limit after the editor spike, not from marketing
  claims about package performance.
- Measure cold/warm opening, typing/paste/find/undo, layout moves, startup, 60-second
  idle CPU, working/private memory and owned descendants. Initial target: warm
  ordinary-file opening under 250 ms and typing frame p95 under 32 ms on the recorded
  Windows host, with no new periodic idle work. These are host-specific targets.
- Freeze a numeric incremental memory budget in 16.0 from matched baseline/editor
  measurements before adopting dependencies. Measure one/two/four panes and
  zero/one/two terminals/LSP servers independently, then detached-window cost in
  19.1. Basic chat may not launch an editor engine, shell or language server.
- Include one or two realistic failures per brick, verifying retained work and the
  next action: stale disk/index, failed write, interrupted drag/save, blocked hook,
  failed remote, PTY flood/exit, LSP crash/stale reply and window transfer failure.
- Use deterministic injected failures; run bounded Qwen routine and DeepSeek
  harder checks only where model integration is exercised. Preserve the original
  provider/model/history and keep keys/private files/transcripts out of artifacts.
- Windows normal release needs native dark/light/wide/compact visual checks.
  Physical IME/screen-reader, macOS/Linux and representative low-end hardware
  remain separate gates until genuinely exercised. Runtime bricks build/launch
  visibly through Python helpers and commit separately in English.

## Boundaries and delivery order

1. **16 — Workspace and editor:** prototype/component gates, multi-project host,
   feature rail, safe document editing and the requested four-pane layout.
2. **17 — Git workflow:** actual repository diffs/history, stage/commit, local
   recovery operations and deliberate remote actions.
3. **18 — Terminal and language support:** interactive processes, lazy first-language
   support, managed missing-tool setup and revision-safe language edits.
4. **19 — Detached windows:** qualify a backend, move live views without duplicating
   owners, restore layouts and test multiple-monitor recovery.

The dependency/acceptance details are in [the roadmap](../ROADMAP.md). Full VS Code
extension compatibility, debugger, remote containers/SSH development, notebook
editing, cloud sync and an extension marketplace are outside this series. No
milestone here closes unrelated computer-use/learning/idle qualification gaps.

## Primary sources checked on 2026-10-06

- [re_editor maintainer documentation](https://github.com/reqable/re-editor): editor
  primitives and configurable integration; package claims require local testing.
- [xterm.dart maintainer documentation](https://github.com/TerminalStudio/xterm.dart)
  and [portable-pty API](https://docs.rs/portable-pty/latest/portable_pty/): terminal
  rendering and cross-platform process adapter candidates, not a complete app service.
- [Git status documentation](https://git-scm.com/docs/git-status): stable porcelain
  output and NUL-delimited paths for machine consumers.
- [Language Server Protocol](https://github.com/microsoft/language-server-protocol):
  shared language-service protocol; servers/toolchains have their own prerequisites.
- [Flutter Desktop Windowing API](https://flutter.dev/blog/desktop-windowing-apis)
  and [desktop_multi_window maintainer documentation](https://github.com/Devolutions/desktop_multi_window):
  experimental official API versus multiple-engine plugin design.
