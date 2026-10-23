# Using Dolores

## Home and files

Home puts the composer above your projects and recent chats. Use the sidebar for
Files, Scheduled tasks, Source Control and Terminal; Library opens skill management.
The side panel can still be resized or hidden. Narrow windows use an icon rail
and a collapsible panel.

The composer’s **+**, model and access controls open compact menus. Model settings
and Access settings open the corresponding section of the centered Settings
window. Your draft stays in place. Settings → General contains theme, Gentle
motion, Personalization and Companionship; tool setup is under Connections.
Turning Gentle motion off removes travel and bounce. Your device’s reduced-motion
preference also takes precedence, even when Gentle motion is on.

Type `/` at the start of a plain line to filter commands and enabled, reviewed
skills. Use Up/Down and Enter or Tab to choose; Escape dismisses the picker and
keeps the text. Paths and fenced code keep normal editing, and IME composition
does not select or submit. `/stop` stops this chat's response; `/resume-queue`
resumes its inspected, paused queue. `/goal` opens Goal and progress; `/plan`
opens Plan and decisions.

Use **Planning mode** to inspect local files and discuss a change. The model can
read/list/search and ask questions; commands, writes, delegation and external
tools are unavailable. A normally saved reply becomes a draft plan. Review or
edit that plan and your decisions, choose **Save plan**, then **Accept saved
plan**. Neither action starts execution. **Execute accepted plan** starts the
reviewed revision through normal task ownership and permissions. An existing
goal is bound by its ID and unchanged checklist; a changed goal requires a new
acceptance. Plans and accepted versions survive reopening without starting work.

Type `@` at the start of a plain line to choose a file, folder, selected code,
terminal excerpt, message or conversation. Preview the selected text and size,
then **Share snapshot**. Context chips let you inspect or remove it. Files use
the current working folder. Folders share one listing page (up to 200 entries),
without recursive contents. Messages/conversations are restricted to the same
working folder and latest 80 messages; attachments are omitted. For unsaved code
or terminal output, first use **Attach selection** in Folders or Terminal, then
refresh that category. Remove the original attachment if you only want the
reference snapshot; the existing text-only queue still refuses draft attachments.

Four reference snapshots can share up to 24 KiB total, 16 KiB each. Sources are
checked again between preview and selection; changed/deleted sources require
reselection. After selection, the retained text is historical and stays unchanged
even if the source changes or disappears. It is sent to the chat's configured
model as source data, never as tool permission. References persist until removed;
queued/steered instructions freeze their selected snapshots. Narrow or remove
context when a limit is reached; the draft and prior selections remain. A new
plan or reference selection requires accepting the plan again.

A selected skill becomes a chip showing its scope and reviewed version. Click
the chip to inspect its instructions, or remove it with its close button. Up to
three selections stay with the chat until removed. They apply to normal requests,
context preview and snapshots saved with Queue/Steer; later chip changes do not
rewrite saved inputs. Implicit skill-name/keyword matching still works. These
selections do not change a scheduled task's pinned skill. If a skill changes,
is disabled or is overridden by a project skill, delivery stops without choosing
another version. Remove the stale chip, refresh the picker and reselect. For an
already saved instruction, cancel it and submit again with the new selection.
Failed saves keep the draft and prior chips; no skill is activated by the picker.

While an ordinary chat task runs, keep typing in the composer. **Queue** saves a
text follow-up for the next turn; **Steer now** guides the current task after its
running model/tool operation settles. Steering can prevent later operations;
completed changes remain. Enter uses Queue while busy and never silently steers.
Tool approvals and answers remain separate decisions.

The **Instructions** list shows saved input and delivery receipts. Its menu offers
Edit, Move up/down and Cancel while an entry is pending. Up to eight unresolved
inputs are retained per chat. Applied means inserted into a request, not that the
task succeeded; inspect the associated run's outcome. Stop, failure, a resource
pause or reopening pauses automatic queue dispatch. Inspect retained work, then
choose **Resume queue**. An uncertain receipt is never automatically retried;
inspect its run before acknowledging uncertainty or submitting a new instruction.
JSON and Markdown conversation exports include all instruction receipts.

Queued input currently supports text for ordinary foreground chat tasks. Send
attachments after the active task finishes. Scheduled and computer-use runs keep
their existing controls. Failed saves keep your draft, and subsequent queued
turns preserve newer unsubmitted text. Queued work runs while its chat owner is
available; background scheduling does not make it run after closing the app.

Home keeps the conversation list and chat. The compact left rail opens Folders;
Settings stays at its bottom. Selecting a Home conversation also selects its
project for developer pages. Side chats have no project. The top-left title-bar
button hides or shows the adjacent panel; dragging its divider can hide it too.

When Dolores needs a choice, an **Answer question** card offers clickable answers
and a field for your own response. Choose an answer, use Next for additional
questions, then Send answer to continue. Back keeps your choices. A failed send
keeps your answers for retry; Stop ends the request. Answers do not change tool access.

Folders shows that project's tree. Click a file for a preview tab, double-click
or edit to keep it. Drag tabs to reorder, move to another group or split at an
editor edge; tab menus and Ctrl+Backslash offer alternatives. Up to four groups
share file text and undo while retaining independent cursor and scroll positions.
Small windows expose group selectors. Ctrl+P opens a relative path; Ctrl+F/H
finds/replaces, Ctrl+G goes to a line and Ctrl+S saves.

Save is explicit; autosave is Off. UTF-8 files up to 1 MiB and physical lines up
to 8 KiB are editable; unsupported files have a bounded read-only preview. Syntax
highlighting stops above 64 KiB, with a plain-text status. A changed disk file
requires Compare, then Reload disk or Keep my edits against that reviewed base.
Save as creates a new target and refuses an existing one. Failed saves keep edits.

Private recovery preserves settled drafts locally without writing source files.
Recovery and layout failures offer Retry. Closing the last dirty view offers
Save, Discard or Keep editing; closing the app can save files or keep recovery.
If recovery cannot be stored, the app stays open. File editing shares nothing
with a model; Attach selection deliberately chooses a conversation and snapshot.

Settings → Advanced → Experimental contains Multiple Window (On by default;
additional windows are currently unavailable because their performance checks did
not pass). Your choice is saved; use existing file/diff/terminal split views.
Prevent Windows From Locked is Off by default.
The Windows option requests display/system wakefulness while Dolores runs; it
does not override manual locks, screen-saver security or enforced policy.

## Goals and context recovery

For sustained work in a project or temporary working chat, use **`/goal`** to save
an objective, verifiable success criteria and constraints, then choose **Start
goal**. Saving alone never starts execution. The goal panel shows criteria,
receipt references and cumulative model/tool/segment/time accounting. The agent
can update progress but cannot rewrite the human's objective or criteria. A final
answer alone cannot complete a goal: every criterion needs retained successful
work or verification receipts and a normally saved final response. Inspect those
receipts; their existence is not a guarantee that a model judged them correctly.

Eligible continuation runs through the existing chat owner while the app is open.
Drafts, attachments, pending user input and decisions take priority. Ordinary
queued user turns remain separate from goal segments. Stop or a failed/limited
run pauses progress; reopening requires explicit **Resume goal**, then **Start
goal**. Inspect retained instructions and uncertain effects before resuming.
Two segments without new work/verification receipts block continuation. Resume
does not reset cumulative counters. At an exhausted allowance, inspect progress,
clear the prior goal and explicitly save a smaller new one. Prior goals and run
receipts remain in conversation exports; working files are retained.

Goal runs and chats with automatic compaction enabled can recover context at
settled model/tool boundaries. Near capacity, the agent receives a host estimate
that distinguishes context room, response output and cumulative task limits.
The estimate includes the public byte ceiling and output reserve. Recovery keeps
original user instructions, goals and receipt previews; full evidence remains in
Run history. Two mid-run recovery attempts are allowed per segment, charged to
the same model budget. Malformed, oversized or unusable summaries stop for
inspection without replaying tools. Images require explicit recovery with their
retained attachments. The reminders do not establish model reliability or remove
the need to verify results.

## Scheduled tasks and companionship

Ask in a chat, for example: “Every weekday at 9pm, write my daily report using
skill daily-report.” Enable that skill first. Dolores returns the saved schedule;
check its time, timezone and model. **Scheduled** shows progress, results and errors,
with Pause, Resume, Skip next, Run now, Stop and Delete. Ask in the original chat
to change a task. Ambiguous or unsupported model replies may require clarification.
Tool actions still need approval; creating a task does not grant future access.

Tasks normally run while Dolores is open. On Windows, turn on **Settings →
General → Background tasks → Run tasks in background** to close it to the tray.
Open Dolores from the tray or launch it again to restore the same window and see
results. Tray **Quit Dolores** stops it. Your computer must be awake and connected;
Dolores does not start with Windows or wake it. After interruption, inspect the
retained result before **Run now**; uncertain actions are never replayed.

**Settings → Personalization → Companionship** is optional and Off by default.
Choose an enabled model and hours. The **Frequency** slider runs from Quiet (0)
to Chatty (100 attempts a day); the saved default is 2. Higher settings shorten
the spacing, while zero keeps Dolores quiet. Home
offers Open, Not now, Dismiss, Fewer messages and Turn off. Notes stay quiet during
work or while the window is hidden; they do not send messages or run tasks for you.

## Terminal and language tools

Terminal opens a real local shell directly. Its initial folder is the selected
Home conversation's project, or your home folder without a project. Plus uses the
current selection; existing shells retain their own folder. Drag tabs to reorder,
move or split; tab menus provide alternatives. Hiding the page keeps shells alive.
Ctrl+C copies a selection, otherwise interrupts the foreground command. Stop keeps
displayed output and closes owned processes. Closing live shells asks first.

After restart, retained tabs show stopped output; Start opens a new shell rather
than replaying commands. A failed recovery save keeps the app open for Retry.
Attach selection explicitly chooses a conversation and adds text to its draft;
it does not send the draft or share the entire scrollback.

Folders' language actions support TypeScript/JavaScript and Rust: diagnostics,
completion, hover, definition and references. Ctrl+Space opens completion, F12
goes to definition and Shift+F12 shows references. Servers start only when needed.
Missing tools have explicit setup/install; merely opening a file downloads nothing.
Node is required for TypeScript. Diagnostics may lag when a server omits versions;
the result shows that notice, and Refresh requests another result.

Formatting and Rename symbol first show Before/After across affected files. Apply
changes private editor buffers; saving source files remains explicit. Undo language
edit works while those buffers remain unchanged. A stale or out-of-project edit is
refused before partial application. Model file reads continue to see saved bytes,
unless you deliberately attach an unsaved selection.

## Source Control

Select a project conversation on Home, then open Source Control in the rail.
Working Changes and Staged Changes show saved Git state for its repository. Select a
file for an inline or side-by-side comparison; eight diff tabs can stay open per
project. Inline shows the changed sections and their old/new file line numbers.
Side by side aligns changes with red deletions, green additions and shared scrolling.
Large files are viewed through their changes; large patches have Previous/Next
changes inside Dolores. If saved content changes between pages, the readable page
stays visible; Refresh diff starts a fresh comparison. Binary files show Git's
change summary. Refresh reconciles external edits. History loads 30 commits
at a time. Click a commit to expand its changed files, then click a file to open
its exact non-merge commit comparison. Click the commit again to collapse it.
Eight commit file sets can stay expanded; collapse one to inspect another.
An unavailable commit shows Retry files underneath it and retains other views.

Use + beside Working Changes to stage all saved changes, or − beside Staged
Changes to unstage them. Each file has the same quick action. File menus also offer
stash and discard. Choose hunks in a text diff for partial stage/unstage. The three-dot menu
beside Changes contains Commit, Pull, Push, Fetch and grouped Changes, Branch and
Stash actions. Selecting an action runs it directly; Ctrl+Enter commits the message.
Commit uses staged files, or stages the saved working changes if nothing is staged.
Git hooks stay enabled; failed commits retain the message and index.
Save or close affected unsaved editors first. Stash conflicts retain the stash;
a conflicted revert offers Abort.
Branch switching requires clean tracked files. These actions use your OS account;
they grant the chat model no additional access.

Fetch, Pull and Push offer a configured remote picker, then run the chosen
operation using existing credentials. Pull and Push require upstream tracking;
Pull is fast-forward-only. Opening the action menu does not contact remotes.
Changed refs require
a fresh action; divergent Pull refuses the merge. Stop can leave effects, so
Refresh and reconcile before retrying. Uncertain push warnings last for this
launch; every new Push still checks the current remote ref. No force push or
automatic conflict resolution is offered. Commit messages are retained in the
current app; commit or clear them before final Close.

## Settings

Open **Settings** at the bottom of the left rail. The six sections are **General**, **Models**, **Personalization**, **Memory**, **Tools** and **Advanced**. Search settings to find a specific control. Chat actions contains Branch chat, Export and Chat details; the header icons open Changes and Activity.

**General** offers System, Light and Dark. System follows the device; selecting a theme saves it locally and changes the whole app immediately. A failed save retains the previous theme and offers retry.

**Advanced** starts with a short list of tasks. Testing/learning and diagnostics
expand only when needed. Within each editor, Details contains exact limits and
technical explanations. Escape closes a form through the same unsaved-change
checks as Close; it cannot dismiss an operation still saving or running.

Drag the sidebar’s right border to make it wider or narrower. Double-click the
border to reset its width. You can also focus it and use Left/Right arrows to
resize or Home to reset. The chosen width stays for the current window; smaller
windows use an overlay panel and restore your width when expanded.

The top window strip follows the same theme. Drag its empty space to move Dolores
or double-click to maximize/restore. Window controls remain available with Settings
open. On Windows, right-click the strip for the window menu; standard window
shortcuts and border resizing are available too.

**Models** lists your enabled models. Connect a provider, then choose a model to edit its context window, image support and optional response limits. **Personalization** edits All chats by default; project/chat customization is deliberate. Advanced contains response defaults, scope overrides and execution limits. Edited forms retain drafts across pages. Close offers Save, Discard or Keep editing; failed saves retain your values.


## Repair Dolores on Windows

In a working chat, Dolores can inspect its matching source, propose a repair in
separate storage, then ask to run a frozen reproduction and regressions. A passing
baseline withholds improvement. A complete baseline failure followed by candidate
and regression passes permits a separate release-build review. Native compilation
and tests use your account permissions; they are not an OS sandbox. Full access
does not skip these reviews.

Open **Settings → Advanced → Diagnostics & storage → Native repairs** to inspect
retained builds and restart evidence. For a ready build, **Review installation**
shows the exact versions, profile and restart plan before **Install & restart**.
Finish or stop current work and save other Settings edits first. Chat drafts are
saved before restart; your selected provider and history remain in the same profile.
Dolores verifies idle startup before allowing new work. It never replays the task.

**Review Restore** separately returns to the retained previous code while keeping
current history, including work created after installation. Failed startup instead
recovers the pre-startup snapshot. Refresh only reads evidence. An interrupted or
changed intent needs inspection and a fresh review from a verified retained version;
unknown ownership/compatibility leaves both versions and the history backup intact.
This first path supports provider and selected Rust task-limit handling repairs.
UI, credentials, policy, schema, dependencies and build scripts require other work.
The repaired app runs from a retained folder. This first version does not change
your existing shortcut or maintained launch command; those still open the original
installation. Reopen the retained app to keep using its repair after quitting.

## Attach images

To share an image in chat, copy it and paste into the message box, or use
**+ → Attach file**. A thumbnail appears before you send and stays in the sent
message. Click it for a larger local preview. Enable **Supports image input**
for a compatible model under Settings → Models. Pasting previews locally;
Send shares the image with your configured provider.

## Inspect a Windows application

In any foreground chat, including a side chat without a working folder, ask Dolores to inspect or work in an application, for example “Look at your current interface.” It can list visible windows and capture the intended one directly, including its own window. The screenshot goes to your current provider, so enable **Supports image input** for a compatible model under Settings → Models. You do not need to start a separate Settings workflow or attach a project. Ambiguous targets need clarification; Dolores should describe only what its screenshot actually shows.

When you ask Dolores to use an application, its clicks, typing, scrolling and supported keys run automatically for that task. You can inspect each action in chat and use Stop at any time. It can also operate its own window, for example to change its theme, and verifies input with fresh screenshots. Another chat cannot use the computer while this run owns it. You can still choose **+ → Share window** or use **Settings → Tools → Computer use** for manual selection, local preview and the existing scoped workflow. A bounded image check helps identify transport problems; it does not prove that a model can reliably control an application.

Desktop control is currently Windows-only and limited to the selected window. The model must observe fresh state before input and verify afterward. The separate manual-grant workflow retains its input reviews. Stop ends access; input already dispatched may have effects, so inspect before repeating. Closed/minimized windows and unsupported models produce a recovery instead of switching targets or providers.

**Settings → Tools → Computer use** retains local capture, saved screenshots and recovery controls. Capture/preview sends nothing to the model. Saved images are snapshots, not live views. An uncertain task should be inspected with fresh screenshots and saved receipts before an explicit continuation; approvals are never replayed.

## Choose tool access


Use the access selector beside the composer, or **Settings → Tools → Chat access**. Choose **Review**, **Auto edit** or **Full access**; the choice saves immediately and is remembered for this chat. Review asks before operations. Auto edit allows project-file changes and asks before commands or external tools. Full access skips routine tool prompts; browser input, manual desktop grants and self-updates keep their own reviews. Ordinary-chat computer use runs automatically in all three modes. New chats start in Review.

File restrictions, budgets and Stop remain active. Commands can use literal paths to access other folders with your account's permissions, so cross-project work can proceed through the selected access mode or a command review. Choose **Review** during work to revoke automatic access and stop that chat's run. Started effects can remain; inspect progress before resuming. Existing custom policies stay intact until you choose a replacement mode. Other chats keep their own settings.

## Task limits and continuing work

Tasks use **Automatic** model/tool cycles by default, so ordinary work can proceed past four operations without Continue. Open **Settings → Advanced → Task limits** for optional custom limits for this chat, its project or your user defaults. Leave call fields blank for Automatic; existing saved numbers still apply. Task time is separate from response inactivity and model context. Blank task time adds no overall deadline; an explicit task clock excludes waiting for your review. A reached resource/context limit saves a checkpoint. Repeated tool failures pause with their receipts so Dolores can change approach instead of repeating the same failed operation.

When provider-default output ends after useful public progress, Dolores continues once within the current task allowance. Incomplete tool calls are discarded and any new operation receives fresh review. Explicit output caps, reasoning-only exhaustion and repeated truncation pause instead of looping. Dolores preserves saved progress at output/step limits. **Continue** uses a fresh bounded segment and fresh tool decisions. A task allows four segments by default; after that, review its progress and explicitly adjust Task limits before continuing. Applied files remain after a later failure. A saved reply or successful command alone does not establish that the whole task is finished.

## Delegate scoped work

In a project or temporary chat, a capable model can use `delegate_tasks` to run
one batch of up to two children. Ask for separate goals and non-overlapping file
ownership. Review each child's goal, relative file/folder scope and read-only or
writable access before allowing the batch. Allowing delegation does not approve
later file actions in Review mode. Side chats have no subagents.

Children share your selected model, prepared instructions, original request,
current permissions and total task allowance. They do not receive the full chat
or attachments. Each has at most four model calls/four file operations; children
cannot run commands, use external tools or spawn more children. The parent can
run a separately approved verification command after they return. For a batch
with two tool-using children, the four-call default may be too small; narrow the
task or explicitly adjust Task limits, for example eight model calls, before
sending. Dolores does not raise these settings automatically.

Expandable progress and saved tool cards show child reports and statuses. A
report is not proof of success: inspect detailed evidence in **Activity → Earlier tasks** and
file snapshots in **Changes**. Paused or failed children leave completed work
intact and offer the parent's explicit Continue flow. **Stop** and **Revoke
grants** reach both children and queued decisions. Restart never replays them.

## Search and read public sources

Project and temporary chats have web search and public page reading available
by default. Ask Dolores to research a topic and cite sources. Review the exact
query or URL before allowing it, under your selected task permission mode.
Side chats have no tools. Search snippets and partial page excerpts are labeled
in expandable tool cards and **Activity → Earlier tasks**.

Open **Settings → Tools → Web search** to disable both web tools or change the global
search connection. **Default (Mwmbl)** needs no account/key but has a smaller
index. **Brave Search API** uses your separately supplied key and plan quota;
blank keeps a saved key. **Custom SearXNG** needs a public HTTPS search endpoint
that enables JSON results. Saving sends no test request. Changes apply to future
working runs; stop active work before saving. Failed saves keep edits, and
**Refresh (keep edits)** lets you review and retry after a stale change.

Queries/URLs go to the displayed service, and results go to your chat model and
local evidence. If a routing proxy supplies synthetic DNS addresses, Cloudflare
resolves the public hostname. Private networks, cookies, login, scripts and
redirects are unavailable. Each request has a 20-second deadline/256 KiB download
bound; a page excerpt is at most 8 KiB. Empty results are not proof that no source
exists. Refine the query, choose another provider or supply a direct primary URL.
No paid fallback or retry occurs automatically.

## Use a browser

Open **Settings → Tools → Browser** to check the optional adapter. In a project or
temporary chat, ask Dolores to inspect a website or your local development page.
It opens a visible browser with a fresh profile for that run. Your everyday
browser, cookies and saved login are not used. The browser closes when the run
finishes or you press Stop. A continued run opens a new browser.

Review the literal URL and operation. Clicks and input always ask for approval,
including under Full access, because they can send data or submit actions. Page
changes or expired control references return fresh state without performing the
requested action. An uncertain result needs inspection before another action.
Stop closes the owned browser; it does not undo submissions.

Only resources from the opened origin load. Popups, WebSockets, uploads,
downloads and automated password input are unavailable; some sites will not
work under these limits. HTTPS sites and literal `http://127.0.0.1:<port>` local
development pages are supported. Screenshots capture the viewport and remain
local. Expand a browser tool card and choose **View local screenshot** to see
one; this does not repeat the action or send an image to the model.

### Browser setup

This optional feature needs Node 20 or newer and an installed Edge on Windows,
or Chrome on Linux/macOS. It does not download a browser. From a source checkout,
install the pinned adapter next to your built desktop executable:

```text
python scripts/install-browser-adapter.py
```

The Python command also selects the executable's adapter directory on Linux/macOS.
Use `--destination <directory>` for a separate installation. Failed downloads leave
the target unchanged; existing unrelated or incomplete directories are retained.
Restart Dolores
if you installed Node while it was open, then use **Refresh** in Browser settings.
Runtime readiness does not verify that every website or model can use it.
Screenshots stay in the local folder shown in Browser settings after the run.
At 128 saved captures, remove older images there or use page text inspection;
Dolores does not delete them automatically.

## Start the Windows preview

Extract the entire ZIP and open **Start-Dolores.cmd**. Keep all files together. If extraction is incomplete, extract into a new folder and try again. If the launcher reports a missing C++ runtime, install or repair Microsoft's **Visual C++ Redistributable for x64** using its displayed link, then reopen Dolores. It does not download or install anything automatically. History, saved connection and working files remain separate from the extracted app.

If Windows still displays a DLL or startup error after the preflight passes, install/repair the [supported x64 runtime](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist?view=msvc-170) and try a complete fresh extraction. The helper checks file presence; it cannot prove the installed runtime version is usable. The executable can also be opened directly once prerequisites are available. Full third-party notices and a versioned inventory are included in the preview folder.

## Connect a model

Open **Settings → Models** and enter the OpenAI-compatible API base URL. A local server might use `http://localhost:11434/v1`; its model must already be installed and server running. Hosted providers use their own HTTPS prefix. Enter a key when required, **Fetch models**, select an enabled subset, and save. Manual IDs are available when listing is unsupported.

Pick the active model at the bottom right of the input card. Changes preserve the conversation and draft. **Remember connection** stores the key in your OS vault and restores the connection after restart. A blank key field reuses a current/saved key for the same endpoint; **Use without a key** clears it on save. **Forget saved connection** removes the remembered connection/key while keeping history. If recovery fails, unlock the vault or reconnect; conversations remain available.

Set each model's context window in **Settings → Models**; blank uses **128K (131072 tokens)**. The context ring shows an estimate and included context, with reported usage shown separately when available. **Settings → Models → selected model → Responses** controls that model's output allowance, stall timeout and supported reasoning options. Blank output uses the **provider default**, with **180 seconds without new model data** for stalled responses. An explicit output allowance remains available; existing saved values are preserved. Active streaming and approval review can take longer. Use provider-specific reasoning options only when your endpoint supports them.

During a working chat, **Thinking** opens a preview when the provider supplies reasoning text. Long previews keep the latest portion; the final answer remains separate. Providers that do not supply reasoning still show the current phase and elapsed time. Stop stays available. A stalled model saves a paused task; Continue makes another explicit request and never runs an incomplete tool call.

## Attach files and images

Use **Attach file** (the plus button) in the input card. Click its chip to preview the saved snapshot and what Send will share; remove a draft attachment with its chip’s close action. Text files must be UTF-8 within 64 KiB; PNG/JPEG images fit 2 MiB and 4 megapixels, with no side over 4096 pixels. Up to four files fit one message. Changes to the original file after attachment do not change the snapshot. An attachment-only Send asks Dolores to review the files.

For images, open **Settings → Models**, choose the model, enable **Supports image input**, and **Save model**. This setting applies separately to each model and starts disabled. Your provider and model must accept OpenAI-compatible image input. An image-disabled error also offers **Model settings**; your draft and attachments remain so you can change the setting, choose a capable model, or remove the image before sending again. PDF/OCR and other binary formats require an explicit conversion adapter; none is bundled. The context ring includes an approximate image token allowance, separate from actual reported usage.

**Chat actions → Export → Attachments** copies draft/sent snapshots into a new folder with a manifest. Ordinary conversation exports contain references only. **Settings → Advanced → Attachment storage → Clean unused attachments** removes snapshots that no chat or draft still references; deleting a chat does not remove exported copies or securely erase database pages. See [sharing and limits](design/attachments.md).

## Choose where to work

| Mode | Working folder | Best use |
| --- | --- | --- |
| Project | Folder you choose with Open project | Work on existing files. |
| Temporary workspace | Separate app-managed folder, created on first send | Start without choosing a project. |
| Side chat | None | Converse without file tools. |

New chat starts a temporary workspace. Its menu offers Side chat; a project's **+** starts another chat in that folder. Projects group chats in the sidebar; temporary/side chats appear in Recents. The header lets you show the folder or copy its path. Folder associations survive restart. **Temporary does not mean automatically deleted**: deleting a chat never removes working files.

## Write, review and recover

Open **Settings** at the bottom left. **Models → Scope overrides** chooses output and timeout overrides for a project or chat; **Personalization** controls discussion before substantial work, brief explanations and checking important assumptions. **Task limits** controls execution budgets. Each scoped panel offers **User defaults**, **This project** and **This chat** where available, and shows effective values and origins. **Use inherited settings** clears only that panel’s overrides at the selected scope; other sections are preserved. Reasoning and context windows remain under Models.

Changes apply to future runs. While a run is active, run-related changes wait until it finishes; theme changes remain available. If another update makes your save stale, **Refresh** keeps your draft; review it and explicitly Save again. Tool operations still need review, and these settings do not authorize self-updates. Behavior instructions encourage useful questions and candid recovery; results also depend on the model.

Enter sends ordinary text; Shift+Enter inserts a newline. Headings and fenced code render as editable blocks. Enter inside code inserts a newline; Ctrl/Command+Enter sends. Down Arrow from the final visual code line moves to the next block or creates a paragraph below. Ctrl/Command+A selects the whole draft. Replies have message/code Copy actions.

Working models can request folder listing, literal search, text reads and file changes. Review the exact target, query, content or diff and choose the displayed **Allow/Apply/Create once** or **Deny** action. To work on a file elsewhere, open its containing folder as a project. Searches are bounded and can report partial coverage. Ranged reads return a source digest; snapshot-bound existing-file edits support files up to 1 MiB. Reviewed diffs and ordinary read results remain bounded to 16 KiB; a replacement must match uniquely.

**Changes** retains file before/after snapshots independently of the final reply. Select a record, **Review revert**, inspect the reverse diff, then **Revert once**; a recorded creation offers **Remove once**. Changed files require another review. **Needs check** indicates an uncertain receipt to inspect. Applied files can remain after Stop or a later model failure. Command and MCP effects are outside this journal.

Commands show the executable, literal arguments and folder before **Run once**. Defaults are 30 seconds / 8 KiB capture; a proposal can explicitly request up to 300 seconds / 256 KiB. Large captured logs stay inspectable in the working folder, while model-facing previews remain bounded. Native commands and MCP servers run with your account permissions and may access files outside the project or use the network. Enable installed local servers under **Settings → External tools**: inspect, select tools and explicitly enable. Up to four saved servers share two active external tool slots. Optional keys use the OS vault. Review mode asks for every invocation; selected automatic grants/full chat access use the task-permission boundary described above.

Open **Commands** from Activity or Earlier tasks to inspect a task-owned process,
read incremental output, send exact input, close stdin, interrupt or close its
handle. Two live commands per task and four per host are allowed. Each input is
at most 4 KiB, with one pending write and 64 KiB total per session. Model-proposed
input requires review; **Send input** sends the text you entered. Delivery to
stdin does not prove the command acted on it. Commands end with their owning run;
saved output remains available afterward. Restart marks lost commands and
uncertain input rather than reconnecting or replaying. **Retry saving** retries
only receipt storage and preserves the visible output and draft.

For supported Node commands, explicitly request isolated execution. For an MCP
connection, choose **Run in isolation** before inspecting it. Windows isolation
grants write access to the selected working folder and read access to up to eight
additional listed folders. Review shows those exact paths, including in Full
access mode. Network access is denied; unsupported runtimes and failed setup
refuse without switching to native. A changed grant folder needs a fresh review.
**Retry cleanup** in Commands revokes retained grants without replaying the command.

Keep isolated Node scripts/modules inside the reviewed folders. Use explicit
`--preserve-symlinks` and `--preserve-symlinks-main` arguments before a local script.
Child processes must inherit stdio: Node-created child pipes can stall, and the
host deadline or **Interrupt** stops the owned tree. Human Terminal tabs, browser
helpers and native repair builds keep their existing native execution mode.

If a task pauses at an output/agent-step limit, inspect retained work and choose **Continue** on the latest paused reply. This starts another bounded run with current settings and fresh approvals. Send or clear any current draft first. **Repair and verify** uses saved failed/incomplete command evidence to repair work and rerun the same check. A model's success claim does not clear failed checks.

| Symptom | Next action |
| --- | --- |
| Connection cannot recover | Unlock the credential store, retry recovery or reconnect. |
| Output cut short or response timed out | Inspect the actual limit; check Settings → Models → Scope overrides, then Models → Responses. Explicitly Continue/retry where offered. |
| Model rejects a reasoning option | Choose Provider default or a supported option, then retry. |
| Proposed edit no longer matches | Inspect the current file and request a fresh exact edit. |
| Model says it cannot read files | Check the chat has a working folder and the model supports tool calls. |
| Comparison/feedback cannot save | Keep the editable note or copy the volatile receipt before closing; correct the error and retry explicitly. |

Stop cancels the run. Failed unsaved turns restore the draft; completed tool effects can remain. **Retry message** sends your current edited draft. No automatic failure retry occurs.

## Memory, instructions and skills

**Settings → Memory** has one switch for automatically remembering useful facts,
decisions and preferences from completed conversations and recalling relevant
entries later. No manual population is required. Inspect, correct or forget entries
and their sources there. Off stops capture and recall while keeping saved entries
available to inspect or forget. Learning can use an extra bounded model request;
eligible excerpts are shared with your configured provider. Activity and reported
usage are separate from the chat reply; a completed task does not guarantee a
useful memory was extracted. Manual corrections protect your changes.

**Session summary** reviews an older conversation batch. Generate, correct and explicitly **Save summary**; future context uses it plus recent uncovered turns. Full history remains. Extend, edit or delete as needed. Generated summaries may omit details.

**Instructions** reviews a project's root `AGENTS.md`. Enable explicitly; changed/missing instructions need review or Disable before sending. References stay literal rather than loading other files. Instructions never authorize tools.

Place skills in `.agents/skills/<name>/SKILL.md` within a project, or `~/.agents/skills/<name>/SKILL.md` for all chats. In **Skills**, choose Project/Global, review and activate. Project skills override active global skills with the same name. File changes require review; saved versions support Disable, reviewed rollback and Forget. **Export SKILL.md** copies one retained version to a new file; companion resources are not copied.

**Draft from chat** selects completed exchanges, generates an editable workflow, and compares responses on 1–3 literal tests. Review before **Activate tested skill**; it requires all candidate tests to pass and a strict gain. Draft-only output/deadline overrides leave chat settings unchanged. Truncated/invalid drafts stay unactivated, with selections available for retry.

## Evaluate and inspect history

**Task feedback** records Worked/Needs work with a local note against the original reply. It does not become model instructions or automatic learning input.

**Chat actions → Compare instructions** compares labeled memory/skill snapshots on the same tests. It makes at most six tool-free responses under comparison-only settings. Read exact requests, outcomes, reported usage and timings. A tie, incomplete run or truncated response never counts as improvement. Nothing activates automatically; selected literal tests do not establish general task quality.

Use **Older/Newer/Latest** to browse saved chats/messages. Export the complete selected conversation as Markdown/JSON to a new file. **Trajectory** shows saved run/tool evidence; its live Log holds recent lifecycle events. The context ring reveals prepared-text estimates without a model request.

**Chat actions → Dolores capabilities** inspects the running build, available tools, limits, registry and bounded bundled source locally. A model-requested `inspect_harness` operation in a working chat requires approval before sharing its result. Inspection does not authorize self-updates.

**Activity → Earlier tasks** shows the latest 20 primary runs, their saved settings/origins and ordered local execution evidence. Interrupted operations can have uncertain effects: inspect **Changes** and any external effects before retrying. Dolores never replays them automatically. This durable history is separate from Trajectory's recent live Log; older chats have no invented run records.

## Data and portable previews

Windows portable previews require the [Microsoft Visual C++ x64 runtime](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist?view=msvc-170). Extract all files together and open `dolores_flutter.exe`; moving the EXE alone breaks the app. The ZIP does not include a model or install system dependencies. A checksum detects corruption; an unsigned local preview is not an authenticated public release.

On Windows, normal data is under `%APPDATA%/dev.dolores.desktop`; macOS/Linux use their account's application data directory. Preferences, conversations, source quotes and change snapshots are unencrypted in `dolores.db`. Remembered keys live in the OS vault, scoped to the data directory. Open one Dolores shell per data directory at a time.

Back up the data directory while Dolores is closed; copying the database alone does not restore vault keys elsewhere. Requests send selected context and approved tool content to your endpoint. Exports can include source evidence, feedback and comparison receipts: review before sharing. Deleting a memory does not erase old provenance/exports, and deleting a chat does not delete working files or independent change records. See [privacy](PRIVACY.md) and [tested limitations](ACCEPTANCE.md).

## Larger files and command output

Dolores can request line ranges from UTF-8 files up to 1 MiB and use the returned snapshot for a reviewed exact edit. A changed file requires a fresh read and preview. Command cards show requested time/capture limits; defaults remain 30 seconds and 8 KiB. Larger output names a local log in your working folder. Open that file locally or ask for selected lines. A shortened preview is distinct from an incomplete capture; stopped commands may leave file changes.

## Recovering an interrupted task

Open Activity → Earlier tasks, select the latest interrupted, stopped, failed or paused run, and inspect its checkpoint and Changes. Prepare resume draft puts a recovery request in an empty composer; sending it starts fresh work with current settings. Earlier tool calls are never replayed automatically. The checkpoint distinguishes proposed operations, returned receipts and uncertain effects. Large checkpoints need a smaller scoped request.

Drafts in saved chats are kept locally after a short typing pause. Failed sends and interrupted submitted messages can be restored after restart. Save failures keep the text visible; copy it or edit again to retry. The newest keystrokes within the short debounce interval may not survive a crash.

## Forking and compacting a chat

Use Chat actions → Branch chat to continue from a completed turn. The new chat shares the same files and starts with fresh permissions. In Context (the composer ring), enable Automatically compact this chat to permit one bounded summary attempt when older turns would otherwise be omitted. The draft survives failure; review the summary or increase the configured context window when recovery asks you to. See [managed threads](design/managed-threads.md) for coverage and limits.

## Project knowledge

Settings → Memory → Project facts retains small project facts separately
from personal preferences. Learning from approved task evidence starts off per
working folder. Enable it to retain complete successful command receipts,
package script declarations and direct conventional folders. A declaration is
not a successful execution. Facts show their source and freshness. Changed or
missing sources and observations older than seven days are excluded from future
context. Correct a fact to protect your choice, or disable it to stop reuse.
Feedback-note eligibility is a separate opt-in; this brick does not upload notes
or run reflection. Activity → Requests reports learning failures without losing the
saved reply. Refresh and explicitly retry a correction after a revision conflict.

Settings → Advanced → Skill testing compares an enabled project skill with an edited
candidate on fixed disposable config tasks. It shares those skill snapshots
with the selected provider, uses separate fixed allowances and preserves chat
settings. Inspect files, tool calls and simulated checks in the receipts.
Nothing activates from this page. Stop retains completed cases; a tie, partial
response, unfinished run or failed evidence save cannot qualify. The narrow
fixture suite does not establish general coding ability. Global skills and real
process/network trials are excluded.

Settings → Advanced → Learning experiments offers optional project reflection, automatic
activation and pause controls. These start off. The experimental automatic path
supports only the optional host-created project-check workflow: repairing one
stale check command from current package evidence after independent trials.
Other imported/global skills retain reviewed Library updates. Inspect reasons
and receipts here; partial, tied or failed trials keep the original skill.
Trials send skill snapshots and disposable fixtures, not your private feedback
notes. This is a narrow workflow feature, not general autonomous self-rewriting.

Expand a learning event to inspect its command, cause and source snapshots.
For an active update, **Restore baseline…** previews the retained command and
asks you to restore and quarantine it. Quarantine prevents automatic reactivation
of that candidate; Library still supports deliberate review. Files and completed
commands remain. A manual skill edit blocks restoration and produces a conflict
notice. Pause/disable stop new learning without removing history. One matching
workflow failure can trigger an additional bounded check under the enabled policy;
only complete independent regression evidence permits automatic restoration.
Stop, failed storage and restart do not replay an unfinished learning attempt.

## Experimental recovery mods

Open a working chat, then **Settings → Advanced → Harness extensions**. A mod can suggest one of
Dolores's built-in recovery views for output/context/tool limits, denied access
or interrupted work. It cannot run an action, change permissions or raise limits.
Side chats do not have a project mod.

**Test source** runs local fixed checks without a model request. **Draft a repair**
sends only the current mod source to your selected model, with one bounded
request. Inspect the retained source, candidate/baseline results and change
history. A complete strict improvement offers **Activate tested mod**. Automatic
activation is a separate switch, off by default, for this narrow recovery feature.

Stop and incomplete output preserve received source for editing; complete it and
test explicitly. Refresh reloads state without resubmitting. A full trial history
refuses new trials rather than dropping evidence. **Restore baseline…** separately
confirms restoration and quarantine, and remains available with a full audit.
Restart reconciles interrupted activation without replaying it. Failed mod health
restores the retained baseline; ordinary chat remains available. This experimental
feature has a small tested scope, described in [acceptance](ACCEPTANCE.md).
