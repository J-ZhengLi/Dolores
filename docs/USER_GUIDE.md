# Using Dolores

## Choose tool access

In a saved project or temporary chat, open **Chat actions → Task permissions**. **Review every operation** is the default. **Approve within selected grants** lets you choose file/discovery actions in a relative folder prefix, an exact command, or individual reviewed external tools. **Full access for this chat** skips prompts for advertised tools. Choose a duration and acknowledge the displayed scope before saving.

Grants apply to this chat. They do not enable new plugins or self-updates. File restrictions, task budgets and Stop remain active. Commands and MCP use your account's OS permissions and can affect files outside the working folder. **Revoke grants** is available during work: it stops that chat's run and invalidates pending decisions. Effects already started can remain; inspect progress before resuming. Expired grants need renewal or revocation. Other chats keep their own access settings.

## Task limits and continuing work

Open **Chat actions → Dolores settings** to set task limits for this chat, its project or your user defaults. Model calls, tool operations and the task deadline are separate from output tokens and the model context window. Leaving the task deadline blank retains the existing request-derived deadline.

Dolores preserves saved progress at output/step limits. **Continue** uses a fresh bounded segment and fresh tool decisions. A task allows four segments by default; after that, review its progress and explicitly adjust Task limits before continuing. Applied files remain after a later failure. A saved reply or successful command alone does not establish that the whole task is finished.

## Start the Windows preview

Extract the entire ZIP and open **Start-Dolores.cmd**. Keep all files together. If extraction is incomplete, extract into a new folder and try again. If the launcher reports a missing C++ runtime, install or repair Microsoft's **Visual C++ Redistributable for x64** using its displayed link, then reopen Dolores. It does not download or install anything automatically. History, saved connection and working files remain separate from the extracted app.

If Windows still displays a DLL or startup error after the preflight passes, install/repair the [supported x64 runtime](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist?view=msvc-170) and try a complete fresh extraction. The helper checks file presence; it cannot prove the installed runtime version is usable. The executable can also be opened directly once prerequisites are available. Full third-party notices and a versioned inventory are included in the preview folder.

## Connect a model

Open **Model connection** and enter the OpenAI-compatible API base URL. A local server might use `http://localhost:11434/v1`; its model must already be installed and server running. Hosted providers use their own HTTPS prefix. Enter a key when required, **Fetch models**, select an enabled subset, and save. Manual IDs are available when listing is unsupported.

Pick the active model at the bottom right of the input card. Changes preserve the conversation and draft. **Remember connection** stores the key in your OS vault and restores the connection after restart. A blank key field reuses a current/saved key for the same endpoint; **Use without a key** clears it on save. **Forget saved connection** removes the remembered connection/key while keeping history. If recovery fails, unlock the vault or reconnect; conversations remain available.

Set each model's context window in Model connection; blank uses **128K (131072 tokens)**. The context ring shows an estimate and included context, with reported usage shown separately when available. **Request settings** controls that model's output allowance, deadline and supported reasoning options. Defaults are **2048 output tokens / 180 seconds**. Use provider-specific reasoning options only when your endpoint supports them.

## Attach files and images

Use **Attach file** (the plus button) in the input card. Click its chip to preview the saved snapshot and what Send will share; remove a draft attachment with its chip’s close action. Text files must be UTF-8 within 64 KiB; PNG/JPEG images fit 2 MiB and 4 megapixels, with no side over 4096 pixels. Up to four files fit one message. Changes to the original file after attachment do not change the snapshot. An attachment-only Send asks Dolores to review the files.

For images, open **Model connection**, choose the model in **Model settings** at the top, enable **Supports image input**, and **Save connection**. This setting applies separately to each model and starts disabled. Your provider and model must accept OpenAI-compatible image input. An image-disabled error also offers **Model settings**; your draft and attachments remain so you can change the setting, choose a capable model, or remove the image before sending again. PDF/OCR and other binary formats require an explicit conversion adapter; none is bundled. The context ring includes an approximate image token allowance, separate from actual reported usage.

**Chat actions → Export attachments** copies draft/sent snapshots into a new folder with a manifest. Ordinary conversation exports contain references only. **Clean unused attachments** removes snapshots that no chat or draft still references; deleting a chat does not remove exported copies or securely erase database pages. See [sharing and limits](design/attachments.md).

## Choose where to work

| Mode | Working folder | Best use |
| --- | --- | --- |
| Project | Folder you choose with Open project | Work on existing files. |
| Temporary workspace | Separate app-managed folder, created on first send | Start without choosing a project. |
| Side chat | None | Converse without file tools. |

New chat starts a temporary workspace. Its menu offers Side chat; a project's **+** starts another chat in that folder. Projects group chats in the sidebar; temporary/side chats appear in Recents. The header lets you show the folder or copy its path. Folder associations survive restart. **Temporary does not mean automatically deleted**: deleting a chat never removes working files.

## Write, review and recover

In a saved chat, open **Chat actions → Dolores settings** to choose **User defaults**, **This project** or **This chat**. Project/chat overrides can change output tokens and timeout; interaction controls choose discussion before substantial work or brief explanations, and checking important assumptions. The panel shows the effective values and their source. **Use inherited settings** removes that scope's overrides. Reasoning options and model context windows stay in model configuration/Request settings.

Changes apply to future runs. While a run is active, you can inspect settings but cannot save them. If another update makes your save stale, **Refresh** keeps your draft; review it and explicitly Save again. Tool operations still need review, and these settings do not authorize self-updates. Behavior instructions encourage useful questions and candid recovery; results also depend on the model.

Enter sends ordinary text; Shift+Enter inserts a newline. Headings and fenced code render as editable blocks. Enter inside code inserts a newline; Ctrl/Command+Enter sends. Down Arrow from the final visual code line moves to the next block or creates a paragraph below. Ctrl/Command+A selects the whole draft. Replies have message/code Copy actions.

Working models can request folder listing, literal search, text reads and file changes. Review the exact target, query, content or diff and choose the displayed **Allow/Apply/Create once** or **Deny** action. To work on a file elsewhere, open its containing folder as a project. Searches are bounded and can report partial coverage. Ranged reads return a source digest; snapshot-bound existing-file edits support files up to 1 MiB. Reviewed diffs and ordinary read results remain bounded to 16 KiB; a replacement must match uniquely.

**Changes** retains file before/after snapshots independently of the final reply. Select a record, **Review revert**, inspect the reverse diff, then **Revert once**; a recorded creation offers **Remove once**. Changed files require another review. **Needs check** indicates an uncertain receipt to inspect. Applied files can remain after Stop or a later model failure. Command and MCP effects are outside this journal.

Commands show the executable, literal arguments and folder before **Run once**. Defaults are 30 seconds / 8 KiB capture; a proposal can explicitly request up to 300 seconds / 256 KiB. Large captured logs stay inspectable in the working folder, while model-facing previews remain bounded. Commands and MCP servers run with your account permissions and may access files outside the project or use the network. Enable installed local servers under **Chat actions → External tools (MCP)**: inspect, select tools and explicitly enable. Up to four saved servers share two active external tool slots. Optional keys use the OS vault. Review mode asks for every invocation; selected automatic grants/full chat access use the task-permission boundary described above.

If a task pauses at an output/agent-step limit, inspect retained work and choose **Continue** on the latest paused reply. This starts another bounded run with current settings and fresh approvals. Send or clear any current draft first. **Repair and verify** uses saved failed/incomplete command evidence to repair work and rerun the same check. A model's success claim does not clear failed checks.

| Symptom | Next action |
| --- | --- |
| Connection cannot recover | Unlock the credential store, retry recovery or reconnect. |
| Output cut short or response timed out | Inspect the actual limit; check project/chat overrides in Dolores settings, then Request settings. Explicitly Continue/retry where offered. |
| Model rejects a reasoning option | Choose Provider default or a supported option, then retry. |
| Proposed edit no longer matches | Inspect the current file and request a fresh exact edit. |
| Model says it cannot read files | Check the chat has a working folder and the model supports tool calls. |
| Comparison/feedback cannot save | Keep the editable note or copy the volatile receipt before closing; correct the error and retry explicitly. |

Stop cancels the run. Failed unsaved turns restore the draft; completed tool effects can remain. **Retry message** sends your current edited draft. No automatic failure retry occurs.

## Memory, instructions and skills

Open **Memory** to inspect/edit/disable/delete preferences and source evidence. Automatic learning is enabled by default for eligible explicit durable preferences after saved replies. Working chats use **This working folder**; side chats use **All chats**. Manual edits/disables protect learned entries. Eligible learning may use one extra bounded request; activity/usage is separate from the reply. Turn off **Learn preferences automatically** for manual control, then use **New preference** or **Suggest from this chat**.

**Session summary** reviews an older conversation batch. Generate, correct and explicitly **Save summary**; future context uses it plus recent uncovered turns. Full history remains. Extend, edit or delete as needed. Generated summaries may omit details.

**Instructions** reviews a project's root `AGENTS.md`. Enable explicitly; changed/missing instructions need review or Disable before sending. References stay literal rather than loading other files. Instructions never authorize tools.

Place skills in `.agents/skills/<name>/SKILL.md` within a project, or `~/.agents/skills/<name>/SKILL.md` for all chats. In **Skills**, choose Project/Global, review and activate. Project skills override active global skills with the same name. File changes require review; saved versions support Disable, reviewed rollback and Forget. **Export SKILL.md** copies one retained version to a new file; companion resources are not copied.

**Draft from chat** selects completed exchanges, generates an editable workflow, and compares responses on 1–3 literal tests. Review before **Activate tested skill**; it requires all candidate tests to pass and a strict gain. Draft-only output/deadline overrides leave chat settings unchanged. Truncated/invalid drafts stay unactivated, with selections available for retry.

## Evaluate and inspect history

**Task feedback** records Worked/Needs work with a local note against the original reply. It does not become model instructions or automatic learning input.

**Chat actions → Compare instructions** compares labeled memory/skill snapshots on the same tests. It makes at most six tool-free responses under comparison-only settings. Read exact requests, outcomes, reported usage and timings. A tie, incomplete run or truncated response never counts as improvement. Nothing activates automatically; selected literal tests do not establish general task quality.

Use **Older/Newer/Latest** to browse saved chats/messages. Export the complete selected conversation as Markdown/JSON to a new file. **Trajectory** shows saved run/tool evidence; its live Log holds recent lifecycle events. The context ring reveals prepared-text estimates without a model request.

**Chat actions → Dolores capabilities** inspects the running build, available tools, limits, registry and bounded bundled source locally. A model-requested `inspect_harness` operation in a working chat requires approval before sharing its result. Inspection does not authorize self-updates.

**Chat actions → Run history** shows the latest 20 primary runs, their saved settings/origins and ordered local execution evidence. Interrupted operations can have uncertain effects: inspect **Changes** and any external effects before retrying. Dolores never replays them automatically. This durable history is separate from Trajectory's recent live Log; older chats have no invented run records.

## Data and portable previews

Windows portable previews require the [Microsoft Visual C++ x64 runtime](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist?view=msvc-170). Extract all files together and open `dolores_flutter.exe`; moving the EXE alone breaks the app. The ZIP does not include a model or install system dependencies. A checksum detects corruption; an unsigned local preview is not an authenticated public release.

On Windows, normal data is under `%APPDATA%/dev.dolores.desktop`; macOS/Linux use their account's application data directory. Preferences, conversations, source quotes and change snapshots are unencrypted in `dolores.db`. Remembered keys live in the OS vault, scoped to the data directory. Open one Dolores shell per data directory at a time.

Back up the data directory while Dolores is closed; copying the database alone does not restore vault keys elsewhere. Requests send selected context and approved tool content to your endpoint. Exports can include source evidence, feedback and comparison receipts: review before sharing. Deleting a memory does not erase old provenance/exports, and deleting a chat does not delete working files or independent change records. See [privacy](PRIVACY.md) and [tested limitations](ACCEPTANCE.md).

## Larger files and command output

Dolores can request line ranges from UTF-8 files up to 1 MiB and use the returned snapshot for a reviewed exact edit. A changed file requires a fresh read and preview. Command cards show requested time/capture limits; defaults remain 30 seconds and 8 KiB. Larger output names a local log in your working folder. Open that file locally or ask for selected lines. A shortened preview is distinct from an incomplete capture; stopped commands may leave file changes.

## Recovering an interrupted task

Open Chat actions → Run history, select the latest interrupted, stopped, failed or paused run, and inspect its checkpoint and Changes. Prepare resume draft puts a recovery request in an empty composer; sending it starts fresh work with current settings. Earlier tool calls are never replayed automatically. The checkpoint distinguishes proposed operations, returned receipts and uncertain effects. Large checkpoints need a smaller scoped request.

Drafts in saved chats are kept locally after a short typing pause. Failed sends and interrupted submitted messages can be restored after restart. Save failures keep the text visible; copy it or edit again to retry. The newest keystrokes within the short debounce interval may not survive a crash.

## Forking and compacting a chat

Use Chat actions → Fork conversation to continue from a completed turn. The new chat shares the same files and starts with fresh permissions. In Session summary, enable Automatically compact this chat to permit one bounded summary attempt when older turns would otherwise be omitted. The draft survives failure; review the summary or increase the configured context window when recovery asks you to. See [managed threads](design/managed-threads.md) for coverage and limits.
