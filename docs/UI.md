# Universal UI style

**Target page contract — 2026-10-07:** retain the existing agent/session/message/
composer design on Home. Following the user's ChatGPT desktop screenshot, use a
compact 64-pixel icon rail for Home, Scheduled, Folders, Source Control and Terminal,
with Settings anchored at the bottom, hover/focus names and semantic labels.
The wider chat/file page panel remains beside it; brand appears once there. The previous
mixed-pane and A/B/C prototypes are historical explorations. The amended page
direction below is final; existing [editor qualification](qualification/workspace-editor.md)
gates remain open.

**Final page decision — 2026-10-07:** developer-pages **B**, with a top-left
title-bar sidebar-layout toggle and a draggable divider that shrinks the page panel
to hidden. Do not substitute a Close/folder icon. Home conversation selection owns
the selected project; developer pages have no project dropdown. Terminal enters
its real view directly, initially at that project's root or OS home, then reuses
existing shells. The user requested no further prototype changes; these amendments
are implementation requirements, not interaction checks already performed.

Native repairs uses the existing Advanced → Diagnostics & storage entry and
embedded inspector. Show retained build/handoff status and recovery reasons before
Details. Build results say “Build ready · installation needs review”; withheld
trials show “Candidate: not run” without fabricated counts. Saved build outcomes
precede collapsed build review details; failed builds point to
retained logs and fresh review, while qualified condensed receipts keep regression
detail availability explicit. Use readable recovery labels rather than raw states.
Installation and Restore have separate exact reviews and fixed wrapping footer
actions. Pending restart
blocks Close/navigation; failed or stale review preserves drafts, brings the error
into view and requires fresh review. Unsaved edits in other Settings pages prevent
restart. No composer/header button or idle updater is added.

The proposed [developer workspace](design/developer-workspace.md) gives each page
its own side panel and main content. Folders has a selected-project file tree and
VS Code-style draggable file tabs/splits, compact close/overflow controls and
breadcrumbs, without a permanent pane action bar. Source Control has project-bound
changes and Git diffs. Terminal has separate tabs/splits; plus starts at the
selected project root, or OS user home if no project is selected. Home chats stay
in their current UI. Keep this document's palette, icons, focus and compact rules.
The planned Experimental page contains Multiple Window (default On, backend
availability explicit) and Windows keep-awake (default Off, actual power/security
limits disclosed). These targets are not yet the running rail/editor.

The [memory/scheduling contract](design/memory-scheduling-companionship.md) plans
one-switch useful automatic memory with inspect/forget, natural-language task
creation and a management-only Scheduled page. No manual Create task form is
planned. Companionship is a separate opt-in feature: occasional in-app messages
during chosen hours, with a daily cap and a configured weaker model. Keep settings
simple; details expose evidence and limits when they help a decision.

Match verification to the change. Prefer saved renders inspected with `view_image`
for appearance; use computer-use when interaction/native behavior needs it.
Routine tasks do not require a desktop launch or visual check. Relevant integration
checks and user-requested previews retain configuration/history.

Milestone 15's [UX audit and target contract](design/ux-simplification.md) defines
conversation-first computer use, simpler settings navigation and progressive
disclosure across configuration. Current sections below describe implemented UI.
The selected A hierarchy is implemented; [qualification](qualification/everyday-ux.md)
records native/model/platform gaps. Preserve this document's palette/brand/focus/compact rules during
the UX work. The contract now also covers conversation controls, summary, activity,
changes and recovery; three layouts are captured for review on
`codex/prototype-everyday-ux`. The user selected A's drawer layout with the existing
icon set, icon-only header actions with hover/focus hints and retained theme preview
tiles. Status cards use the normal surface and uniform border; avoid decorative
colored left edges.
Earlier restrictions on adding task controls are revised by the
named milestone 15 scope when that brick is implemented.

Advanced starts with a short task chooser, with testing/learning and diagnostics/
storage groups closed. Nested pages show controls and status before explanation;
named Details retains exact budgets, source and provenance. Sharing and experimental
status remain visible beside their actions. Escape follows Close's draft/pending
checks. Expanded editors retain state when errors or status notices appear.

Task limits show Automatic when model/tool counts are blank, with optional numeric
overrides under the existing scope control. Preserve saved numeric settings.
Repeated-failure pauses and context/resource checkpoints have distinct short
labels, saved progress and the existing Continue action. Do not suggest changing
model settings to resolve a tool failure or replay an incomplete request.

## Desktop window frame

The normal desktop app uses a 32-pixel custom title strip above the conversation
layout. It uses one uninterrupted effective-theme background across its full
width and adds no name or icon; keep the original sidebar brand once.
Windows/Linux use quiet 46-pixel Minimize, Maximize/Restore and Close buttons with
tooltips, keyboard focus and semantic labels. macOS reserves space for native
traffic lights. Drag the empty strip to move; double-click it to maximize/restore.
Windows also offers the native window menu on right-click. Native borders retain
resize operations and the desktop minimum is 420×480 logical pixels. Controls sit
above the Navigator and remain reachable while settings/detail dialogs are open.
Use OS maximize events rather than polling. A failed action preserves the body
and offers explicit Retry/Dismiss; failed initialization restores the native frame.

The wide sidebar starts at 252 pixels and resizes from its right divider, with a
9-pixel pointer target and horizontal resize cursor. Use the accent only during
an active drag; release/cancel returns to the normal border color even while
hovered or focused. Hover/focus slightly thickens the normal divider.
Clamp it to 220–360 pixels and preserve at least 480 pixels for the conversation.
Keep the chosen width for this window, temporarily clamp it on window shrink and
restore it on expansion; below 760 pixels use the existing 252-pixel drawer.
Double-click the divider or press Home when focused to reset; Left/Right arrows
and semantic increase/decrease resize by 16 pixels. Resizing preserves drafts,
conversation state and section expansion. Sidebar content starts 12 pixels below
the title strip, with 16 pixels between the brand and New chat; Settings stays
anchored at the bottom. Section labels ellipsize before clipping their chevrons.

## Unified settings

One labeled gear entry, **Settings**, sits below the sidebar divider at the bottom left. Remove separate Memory, Model connection and Request settings entries and configuration entries from Chat actions. The latter uses `Icons.more_horiz` in every size and keeps Branch chat / Export / Chat details actions. Skills management lives in Settings too; workspace instructions remain a conversation-context action.

Use one 960×720 maximum system-themed window with 16-pixel outer margins, a 200-pixel scrolling category list and a scrolling editor with fixed wrapping save controls. Selected categories use the existing soft selected-session color. Sections are General, Models, Personalization, Memory, Tools and Advanced. Below 640 pixels, use a labeled section dropdown. Tools and Advanced retain named subpages and direct search links. Models lists enabled models; its detail editor combines context, image and response controls. Avoid nesting settings dialogs; explicit detail/review dialogs remain separate modal routes.

Create editors only on first visit and retain their drafts for this window. Pending operations block category switching, Close and Escape; failed saves preserve edits with visible recovery. Close and Escape offer Save / Discard / Keep editing for unsaved drafts. Appearance offers System / Light / Dark cards using local preview illustrations, applies and persists after storage acknowledgement, and retains the old theme on failure. Theme changes update open panels and the app. Default System follows device brightness. Other saves retain explicit scope/revision/credential/review rules; each scope reset affects only its section. Deep links from recovery open the relevant category in this same window. Workspace-dependent pages explain prerequisites rather than disappearing. No theme-dependent network requests or idle polling.


## Conversation messages

User messages sit in a right-aligned, naturally sized rounded bubble within
88% of the conversation width. Use the same soft background and normal text color
as a highlighted sidebar session in both themes; text remains left-aligned and selectable.
Assistant replies use the full left-aligned content width. Neither role shows a
sender name or profile image. Copy and a local saved timestamp appear below the
message on hover or keyboard focus; reserve that space to avoid layout jumps.
Keep Copy reachable by keyboard, preserve the complete original Markdown source,
and omit timestamps for older messages without recorded times. Streaming replies
keep their working indicator and do not offer whole-message copy. Attachments,
tool evidence, usage and recovery controls stay available in their existing forms.

## Browser use

Settings → Tools offers Skills; Advanced offers Skill testing and Learning experiments as cached embedded editors.
Learning exposes off-by-default reflection/automatic activation, project pause,
the optional host check workflow, local causes and independent receipts. Failed
saves retain drafts; explicit Refresh never replays a claimed event. No new
composer/header controls are added. Automatic activation is labeled experimental.
Learning history expands reasons/command/source snapshots. Restore baseline…
opens a concrete command preview with Cancel / Restore & quarantine. Keep
recovery notices and active progress above the fixed footer actions so scrolling
cannot hide the outcome. A conflict keeps the active card and manual version.
Pending trials block switching/closing and expose Stop; saved receipts, errors
and candidate drafts remain inspectable. Trial settings are frozen separately
from chat settings. Do not describe simulated checks as executed OS commands.

Settings → Tools → Browser uses the same embedded inspector with a scrolling status/setup
body and fixed Refresh action. It starts no browser. Reuse tool approvals for
literal browser JSON, profile/sharing/effects disclosure and fresh click/input
review. Browser cards label stale state as no action dispatched, and uncertain
actions require inspection. View local screenshot loads one bounded saved JPEG
on demand in the expanded card; failed previews retain receipts and a retry that
does not repeat the browser action. Add no header/composer control or idle polling.

## Web search

Settings → Tools → Web search offers a global connection editor using
InspectorFrame/system palette. Default Mwmbl needs no setup; the scrolling body
offers enabled/provider, public SearXNG endpoint and obscured Brave key/removal.
Disclose index coverage, query/URL/model sharing, quota, public network restrictions,
routing-proxy DNS fallback and literal download/excerpt/deadline bounds. Use an
expanded dropdown and fixed wrapping Save/Refresh controls in compact themes.
Save errors remain visible and retain edits; Refresh keeps the draft and updates
the revision. Busy work permits reading but locks saving/expansion. No test query
is sent on Save. Keys are never returned into the form. Keep the composer unchanged.

Inline web approvals show literal service/URL and query/offset with sharing and
untrusted-content disclosure. Saved tool cards show selectable exact source URLs,
snippets/excerpts and partial continuation offsets; literal errors remain usable.
Use existing Allow once/Deny/Stop and task recovery rather than a new retry loop.

## Attachments

Images use bounded thumbnails in the composer and user message bubble, including
while the response is pending. Keep only one pending copy in view. Hover/focus
reveals the filename; opening the thumbnail shows the existing local full preview.
Preview failure offers an explicit retry without replaying Send. Failed sends
restore the draft and attachment references together.

Ctrl+V/Cmd+V and Shift+Insert probe for an image on explicit paste, then fall back
to normal text insertion when there is none. Do not poll the clipboard or replace
selected text when adding an image. Windows clipboard bitmaps are converted to
bounded PNG snapshots; image size/capability rules remain in force.

Anchor the composer’s quiet plus action for Attach file at the left of its footer, vertically aligned with Send and the context ring. Reserve remaining width for a right-aligned model selector so short or long model names cannot push the plus toward the middle; keep model/context/send in that order. Snapshot chips wrap above the editable content and offer local text/image preview with sharing disclosure; draft removal is disabled during execution or state changes. Preview uses the system dialog theme and bounded scrolling/image dimensions. Chat actions → Export offers snapshot export; Advanced → Attachment storage offers unused-asset cleanup. Settings → Models → Connection & models starts with the selected model’s settings: Supports image input (disabled by default) and the context window, followed by connection/model-list controls. Errors involving attached images offer Model settings directly. Failed saves bring the error into view and retain edits for explicit retry; Cancel leaves capabilities, draft and attachments unchanged. No silent OCR or upload is implied. Context inspection shows the approximate image token allowance only when images are included. Failed chat selection leaves the previous draft and its attachment scope together; attachments count as an occupied draft for Continue/recovery.

## Task permissions

Working saved chats offer Permissions in Settings, including during execution. Reuse InspectorFrame/system palette with a scrolling mode, grant, expiry and acknowledgement form and fixed Refresh/Revoke grants/Save access controls. Full access and selected grants require explicit acknowledgement before Save; defaults are review mode with no automatic grant. Show saved mode/revision/expiry and actual user-account command/MCP boundary. Busy mode permits inspection/revocation, with expansion disabled. Stale saves retain the draft and Refresh requires renewed acknowledgement; errors remain visible. Budget/interaction reset does not change access. No composer/theme change or idle polling is introduced.

## Task limits

Settings → Task limits includes scoped overrides in its scrollable editor: model calls, tool operations, total task segments and an optional active-work time limit. Blank time limit means no whole-task clock. Human review is excluded even from an explicit time limit. A reached task limit saves a paused turn with public progress and completed receipts; Continue starts fresh requests and never replays pending approvals. Fixed Save/Refresh/Use inherited controls retain drafts after invalid/stale saves. Continue consumes a segment and grants no access. Keep the composer, system palette and native typography unchanged. Segment refusal keeps the saved paused card and points to Task limits.

## Run evidence

Active model calls show a quiet activity label with elapsed seconds: Waiting for
model, Thinking, Preparing tool call or Responding. Labels reflect transport
activity, not invented progress percentages. The label stays visible after public
text appears and changes to Waiting for your review at an approval. A hover hint
gives the model stall allowance, measured without new decoded data; an active
response can take longer. Review has no timeout. Stop stays available. Updates stop
when the run ends and stale step events cannot change the current reply.

Provider-supplied thinking appears in a collapsed Thinking panel, with a bounded
scrollable preview and a shortening label when needed. Keep it separate from the
answer, executable arguments and diagnostic journals; do not invent a reasoning
summary for models that supply none. Saved local reply metadata retains the preview.
Use existing palette and typography, without a colored side border. Model stalls
save a paused turn and expose explicit Continue and response settings; no automatic
retry or settings increase is performed.

Activity keeps Earlier tasks available during execution; capabilities live under Advanced. Other actions that prepare or mutate state remain disabled. Run history uses InspectorFrame, a scrollable latest-20 list, selectable ordered evidence and a fixed Refresh footer. Show interruption and uncertain effects literally with inspection guidance, never an automatic Replay action. Changing the foreground chat still waits for Stop/completion; read-only backend inspection of another chat is allowed. No idle polling, new palette or composer change is added.

## Capability inspection

Advanced → Dolores capabilities offers local inspection using InspectorFrame, system palette and fixed wrapping footer. Local inventory explains tools, approval/containment, window origin and limits without model requests. Source selection reads bounded bundled lines; Next/First lines and Compare checkout are explicit local actions. Stale/missing source and refresh errors remain readable, with usable compact controls. Model-requested inspection uses the existing approval card with read-only/sharing disclosure and Harness inspection result label. Composer layout is unchanged.

Keep Extension registry collapsed initially. Expand it for each entry's enabled/available resolution, kind, version, configuration revision, health or actionable incompatibility reason. Registry errors never appear as tool approval or automatic authority. Local refresh and source inspection remain usable when an entry is unavailable.

## Composer input and accessibility

Name native editable blocks as Message plus their heading level, code language or paragraph position. Retain editable values and native focus in the semantic tree; expose the code-language button's current language and enabled state without repeating its tooltip. Send and code-language changes wait for active IME composition to commit, preserving unfinished candidates; Stop remains available during a response. Keep the existing model/context/send footer and palette. See [native input checks](design/native-input-verification.md) for verified flows and pending physical-keyboard, real-IME and spoken screen-reader acceptance.

## Subagent reports

Scoped delegation reuses the inline tool approval, with exact literal goals,
file scopes, access mode and shared-budget disclosure. Child file approvals show
their child identity and retain Allow once/Deny and Stop. Active children share
the existing bounded tool-progress scroll area above the composer. Expandable
cards show goal, scope, status and literal bounded report; saved delegation cards
retain the same reports plus shared usage. Label reports as requiring parent
verification. Run history holds parent-linked detailed child evidence; Changes
holds applied file snapshots. Failed/stopped runs retain live child status until
changing chats or retrying; no automatic child replay. Preserve the theme,
composer controls and footer positions. No idle worker or new help label.

## Native repair review (20.2)

20.4's separate native execution card says Build and test this repair? / Run once.
Show the exact candidate plus frozen reproduction and bounded command sequence;
disclose account permissions, three command limits, unchanged tests and separate
installation authority. Keep disclosure/diff scrollable and buttons reachable.
Saved native cards say Improvement withheld or Tests qualified · installation
needs review rather than ordinary Completed. Incomplete/stopped results retain
the proposal and give a fresh-review/reproduction next step.

Reuse the ordinary tool review card and selectable diff for `harness_repair`.
Label it Repair proposal / Review this repair step?, with Storage: Dolores repair
workspace and Exact repair step. Show the matching source and diff before Allow
once/Decline; disclose separate storage, provider sharing and separately reviewed
native execution/installation. Keep buttons outside the scrolling body and allow
the same 270-pixel body as file edits so compact light/dark layouts remain usable.
Saved results show status, revision, file identities and an actionable integrity
notice. Never label a retained proposal as tested, installed or improved. No new
composer control, idle polling, decorative border or settings pane is introduced.

## Brand mark

Keep the original `Icons.all_inclusive_rounded` infinity mark and existing semantic accent colors in Flutter's home and sidebar. Chat messages have no sender avatars. `assets/dolores.svg` retains that exact Material Icons outline for Windows, macOS, Linux and web/Tauri icons on the dark rounded tile. Do not redesign it as closed loops or reintroduce a letter monogram. Regenerate platform assets with `scripts/generate-icons.py`; Pillow is development-only and normal builds use checked-in outputs. Preserve the upstream attribution/license in `assets/LICENSE.material-icons`. The visible app name is Dolores; framework names stay out of its window title.

## Instruction comparisons

Use the saved-chat **Settings → Advanced → Compare instructions** entry and shared InspectorFrame/system palette. Keep two labeled instruction snapshots, source-copy selectors, 1–3 literal tests and comparison-only token/deadline fields in one scroll area. Fixed footer Run/Refresh and Stop remain reachable; busy operations lock dismissal and duplicate writes. Editing a copied snapshot clears its source binding. Refusals retain the draft and reveal actionable errors at the top.

Saved receipts show complete/failed/stopped/unfinished status, two separate pass counts, actual output/elapsed/reported usage and expandable exact requests. Only complete strict gain earns the improvement label. Failed storage shows Copy receipt before closing; never imply a volatile result was saved. Older/Newest replaces the bounded saved page. Use as draft is explicit and never resumes an old run. Leave the composer unchanged and add no startup/background evaluation.

## Task feedback

Saved assistant replies and trajectory rows offer a quiet Task feedback action beside existing run details. Use the shared InspectorFrame, palette and scrollable compact form with Worked / Needs work, optional note and fixed Save / Clear controls. Explain that the local assessment is separate from command evidence and included in exports; it never changes model context or learning. Show the original reply details and pause reason. Pending saves exclude duplicate edits/dismissal; failed saves preserve notes and bring errors into view. Keep the composer unchanged.

## Failed command checks

Command cards label Failed with the actual exit, Verification incomplete for incomplete capture/process evidence, and Exited 0 for complete zero-exit receipts. Keep stdout/stderr literal and expandable, including legacy receipts. A commandReview pause uses muted explanatory text and Repair and verify in the existing Continue position. Show the inherited saved command receipts even when the latest segment has none. Preserve draft exclusion, latest-message binding, loading/duplicate locks and fresh-approval explanation. Step/output pauses keep Continue. The quiet palette, composer and layout stay unchanged; never present model prose as a verified result.

## Paused tasks

An explicit model output limit or agent step limit renders the saved partial response with a muted pause reason, actual saved output allowance where applicable, and the existing progress/tool/usage details. Show Continue only on the latest saved paused assistant message. Disable it during loading or generation and when a composer draft is present; explain that the draft must be sent or cleared. A continuation needs a configured connection, starts a separate bounded run and asks for fresh tool approval. Incomplete calls have not run. Keep the original paused segment and receipts visible after success, failure, Stop and restart. Failed continuation leaves the composer empty and the saved response available for retry. No automatic retries or new budget controls are introduced. Paused replies do not trigger automatic preference learning.

## Skill drafts and evaluation

Before Generate, show Draft output tokens and Draft timeout with the saved model settings as initial values. Edits apply to this draft only; explain that larger allowances may increase usage while the skill stays within 8 KiB. Lock these fields during requests. An output-limit error names the allowance and points to Draft output tokens; preserve selected exchanges for manual retry and bring the error into view. No hidden drafting token/deadline cap or automatic increased-budget retry is permitted.

Skills offers Draft from chat in the selected Project/Global scope. Use a nested InspectorFrame with a scrolling body and fixed wrapping Generate/Evaluate/Activate actions. Completed exchanges start unchecked; show full expandable You/Dolores text and explain what Generate sends. The editable draft has name, trigger description and Markdown instructions, with source evidence expandable. Test prompts and required/forbidden exact snippets are editable; allow up to three tests. Explain tool-free fresh-response context, request limits and the limited meaning of literal checks. Show baseline/candidate scores, full responses, usage, settings and expandable exact tested requests. Editing invalidates the prior activation action, and a tie or incomplete comparison never enables it. Activate tested skill is separate from Evaluate. Pending calls disable edits/closing/duplicates while keeping Stop; errors preserve corrections. Saved generated versions show their historical receipt and local origin without a misleading missing-file notice. Keep chat changes locked throughout and preserve the existing composer/theme.

## Reviewed skills

Retained-version review adds Export SKILL.md to the fixed wrapping footer. Use the native Save dialog with suggested SKILL.md and the selected version in its confirmation label. Explain the matching skill-folder name and document-only copy beside the retained version. Never export an unsaved draft or current unretained file through this action. Cancel leaves the review intact; pending export locks dismissal/scope/duplicates. Success/failure appears at the top of the scrolling body after the frame update, so compact users see it. Preserve existing activation and all palette/composer behavior.

Settings → Tools → Skills offers the library even before first send. Reuse InspectorFrame, literal selectable text, system palette and fixed wrapping footer with a scrolling body. A fixed This project / All projects selector switches scope; side chats start with Global and disable Project. Show the local source directory and label enabled global entries overridden here by a same-name project skill. Limits apply per scope. List Available/Enabled/Disabled records; Review file shows exact source/description, while Saved versions works even for missing/invalid source. Explain activation scope, saved-snapshot behavior, limits, tool approval and literal resources before the body. Review a retained version before Activate version; show rollback provenance and keep files unchanged. Back/Refresh/Disable/Forget remain reachable in compact layouts. Close cancels review without activation; failed activation requires Refresh and preserves old state. Lock dismissal, scope switching and duplicates during writes and chat changes while open. Context and reply usage show Project/Global, version and source; local context includes only effective exact snapshots. No composer, palette or automatic model activity change.

## Automatic preferences

Memory offers Learn preferences automatically, enabled by default. Explain current-folder versus All chats scope, limited eligibility and the optional 10-second/512-token extraction. The switch returns to manual review without deleting saved entries. Label automatically learned records and keep source/quote/model/time inspectable. Existing Edit/Disable/Delete remain; edit/disable protects a record from replacement. A collapsible current-chat latest activity shows status, saved/skipped counts, date and separate reported learning usage. Trajectory shows reply saved then learning/result. Learning Stop or failure preserves the completed reply. Reuse the palette, InspectorFrame and fixed footer with a scrollable compact light/dark body; keep the composer unchanged.

## Reviewed session summaries

The composer ring opens Context in the right details panel, combining usage, summary and compaction. Reuse the system theme and InspectorFrame. Show the current summary, covered-turn count, revision, drafting model and review time. Review next turns exposes the exact next contiguous batch through expandable literal You/Dolores rows, an explicit remaining-turn notice and the prior summary. Disclose that Generate shares those turns and that replies may quote private files. Generate draft is explicit; Stop remains available, Close stays disabled until completion, and no retry or save happens automatically. An editable draft has Save summary and Discard. Existing summaries have Edit/Delete; failed Save preserves the correction. Delete restores recent context, not a shortened transcript. The fixed wrapping footer remains reachable with a scrolling body in compact light/dark layouts. Context inspection and reply details count coverage separately from omitted raw turns; local context additionally exposes exact summary text. Keep the model/context/send composer order unchanged.

## Reviewed memory suggestions

Memory offers Suggest from this chat for a saved conversation. Review exact literal user messages, newest 20 with an earlier-history notice, no preselection. Explain which model receives selected text and that replies/tools/files/preferences/draft are excluded. Generate is explicit; show Generating suggestions and Stop, preserve unchanged selected sources after failures, never retry failed generation automatically. Results show model, title, preference and exact quote; empty results say nothing was saved. Review suggestion reuses the editable form and scope choice; Save preference is the only write. Failed saves retain correction; successful candidates are marked Saved and cannot be saved twice. Discard/Close never save other candidates. Explain before Save that quotes are retained locally and in reply details/exports after chat deletion. Saved preferences distinguish Added by you from From a reviewed chat and label unavailable sources while retaining their quote. Use shared inspector, colors and bounded scroll/footer layouts in both themes. No composer change or idle extraction.

## Manual preferences

Sidebar Memory opens in every chat mode, including a new temporary chat before its folder exists. Reuse InspectorFrame, semantic colors, typography and wrapping fixed footer controls. List exact selectable preference text with title, All chats/This working folder, Enabled/Disabled, Added by you, update time and revision. Explain explicit sharing, keeping secrets out, bounded selection and deletion's effect on future messages. New preference opens an inline form with scope, title, literal multiline text and Use in new messages. Default to the current folder when one exists, otherwise All chats; editing fixes scope. Save preference is the only activation action. Close/Cancel never save; Edit, Disable/Enable and Delete require no model request. Failed saves retain the form, and stale revisions require Cancel/Refresh/review. Lock chat changes while open and duplicate controls/dismissal while saving. Context shows exact selected entries and used/left-out counts; saved reply details keep compact source/revision/scope provenance. Memory failures offer a manual Memory action. No composer or palette change, background extraction or idle worker.

## Workspace instructions

Working-folder headers include a quiet Instructions icon beside Changes and Trajectory; Side and uncreated temporary chats omit it. Reuse InspectorFrame and existing palette/type/spacing. A scrollable literal `AGENTS.md` review shows Not enabled, Enabled or Needs review, exact current text, previous reviewed revision and date, and missing/invalid-file feedback. Explain text sharing/local snapshot storage, scope to this folder, literal references and unchanged per-tool approvals before the source. Fixed wrapping footer actions are Refresh, Disable and Enable instructions; Close/Escape never activate. Enable is excluded for unchanged active guidance; failed/expired activation requires Refresh. Disable remains available when the enabled file is missing. Lock chat switching/send while open and disable dismissal/duplicate actions during local saves. Context inspection and saved reply details display source/revision alongside exact prepared system text or summary respectively. A stale-file failure offers an Instructions action, with the draft retained. No composer change, palette addition, idle discovery or watcher.

Command approval reuses the shared card with Run this command? and explicit Run once/Deny. Its permission/output/effect notice precedes the full local working folder, resolved executable and numbered JSON-quoted literal arguments in a scrollable 270-pixel review area. Controls remain below that area. Saved command cards retain program/arguments and selectable literal stdout/stderr, actual exit code or stop reason, plus truncation/UTF-8 labels. Absolute executable paths appear only in the local approval preview. Command argument text has its own storage key, separate from expansion state. No palette, font or composer change is introduced.

File edits use the shared approval card with Apply this file change?, a selectable literal unified diff and Apply once/Deny below a bounded 270-pixel review area. The diff independently scrolls in both directions within 180 pixels. Removed/added lines reuse error-text/syntax-string tokens; no new palette or font is introduced. The card explains changed-file refusal and that applied edits remain if the reply stops/fails. Saved chat/trajectory cards show File edit · edited, readable byte counts and the same diff. Controls remain disabled during a pending decision or Stop.

Working chats stream current public model text before/after approval. Earlier commentary uses an expandable Agent progress row above the final reply, with bounded selectable text in chat/trajectory and a Not saved label while transient. Text resets between calls; commentary never replaces or duplicates the final answer. Existing palette, typography, approval controls and composer footer order remain.

This file is the UI design contract for new Dolores features. Keep one contract here rather than a competing THEME.md. Flutter's semantic colors and shared layout dimensions live in `apps/dolores_flutter/lib/theme.dart`; the shared Material theme is defined there too. Reuse existing tokens before adding a new one. A deliberate restyle updates this contract and the shared implementation together, with light/dark and compact captures reviewed. Routine feature work does not introduce a new palette, font family, layout scale or decorative effect.

The contract fixes visual relationships, not one brightness: system light/dark preference remains the default. No theme setting or background polling is added.

| Token | Light | Dark |
| --- | --- | --- |
| Background | `#FAF9F7` | `#191B20` |
| Sidebar | `#F1F0ED` | `#15171B` |
| Surface | `#FFFFFF` | `#22252B` |
| Text | `#292B30` | `#E4E6EB` |
| Secondary text | `#676D76` | `#A0A6B2` |
| Border | `#DEDFDF` | `#353941` |
| Accent | `#345FCA` | `#9CB6FF` |
| Selected surface | `#E5EBF8` | `#2A3552` |
| Error surface / text | `#FBECEC` / `#9C3030` | `#3B262B` / `#FFB5BB` |
| Code keyword | `#963464` | `#FF9CCC` |
| Code name | `#6940A5` | `#C8A4FF` |
| Code string | `#28743D` | `#93D69B` |
| Code value | `#985221` | `#FFAE78` |

Shared layout: 252 logical-pixel sidebar, drawer below 760, 824-wide outer conversation column with inner padding; normal message text 14 px / 1.65 line height, labels 11–14 px. Use Segoe UI with platform fallback for controls and Georgia with fallback for the existing identity/welcome headings. Component radii range from 6 px for status tags to 14 px for the composer, with 18 px reserved for the welcome mark. Existing measurements may include optical adjustments; new spacing follows the 4 px scale. Material widgets may derive their interaction colors from the shared accent. Prefer labeled text actions and existing Material icons.

Dolores follows the system's light/dark preference using semantic palettes in Flutter. Iced and Svelte retain the shared style as alternatives. The visual language is a quiet workspace: warm neutrals, restrained blue, readable system fonts, subtle borders and generous space. No external fonts, image assets, animation framework or blur effects are needed.

The selected Flutter shell uses the shared palette, 252 px sidebar, readable content cap, rounded composer and system theme. Replies are selectable with Copy; Enter sends and Shift+Enter inserts a newline, with an IME composition guard. Under 760 px the sidebar becomes a drawer. The settings dialog contains focus and hides the key. Remember connection explains OS secure storage; a blank field can retain the saved key. Forget removes the connection, preserving conversations. Recovery warnings appear above the composer with Retry when a remembered connection could not load. Controls are disabled during generation or a connection change. Widget tests cover recovery, key omission, forgetting, Enter/Shift+Enter/Stop and the narrow form; real OS input, theme changes and screen-reader behavior still require UAT.

Connection setup fetches models after the endpoint/key are entered. A searchable checkbox list enables up to 32 choices. Save requires at least one choice. Manual entry is an optional fallback; no model name is required to fetch. Changing the endpoint clears the staged list. The composer model picker preserves the current conversation/draft and is disabled while streaming or changing settings.

Request settings starts with the active model and provides an independent enabled-model dropdown, output allowance, timeout and explicit reasoning control. Saving another model leaves the current chat model/draft intact. Restore defaults stages removal of only that profile until Save; Cancel and failed saves retain the prior profile. The scrollable form follows the shared compact/light/dark style. It explains that output may include reasoning and that provider support varies. A named generation-setting rejection exposes Request settings without automatic retry; malformed calls expose Model connection and preserve completed Changes. Output-limit pause text shows reported reasoning tokens when available. See [profile design](design/model-generation-profiles.md).

History uses one 50-conversation sidebar page and one 80-message transcript page, with Older/Newer and Latest actions making omitted history visible. Paging replaces the displayed page; it does not accumulate hidden message widgets. Export in the conversation header offers complete Markdown or JSON and opens the platform Save dialog. Existing filenames are rejected with a clear choose-another-name message; cancel does nothing. Paging/export are disabled during generation. In-process switching restores drafts and scroll positions for the 20 most recently visited views; they are not durable drafts. New conversation has its own draft. Sending from an earlier page returns to the latest page first. Browsing is separate from the model's bounded context window.

- Consistent 4 px spacing scale; 8–16 px corner radii. Content width is capped for readable conversations.
- Sidebar: identity, new conversation, saved sessions, connection settings. Main area: conversation, model status, composer.
- Welcome state explains the next action. Connection, streaming, stopped, failed, empty, and offline-preview states are explicit.
- Use real labels, keyboard focus and sufficient contrast. Flutter and web chat use Enter to send / Shift+Enter for a newline. The alternative Iced chat uses Ctrl/Command+Enter to send and Enter for a newline. Respect IME composition and reduced motion; actual OS input still needs UAT.
- Flutter assistant replies render GitHub-flavored Markdown with selectable headings, lists, tables, emphasis, quotes and inline code. User prompts stay literal. Code blocks use the existing sidebar surface, 8 px radius, a language label (or Code), Copy code, and independent horizontal scrolling. Tables scroll horizontally at compact widths. Whole-message Copy retains Markdown source; code Copy retains the parsed code including indentation and its trailing newline. Code uses locally installed Consolas, with Menlo/DejaVu Sans Mono/monospace fallback; no syntax highlighter or additional font is loaded.
- Model HTML is visible literal text. All Markdown images show their alt text without network, local-file, asset or data-URI loading. HTTP/HTTPS links show the complete destination in a contained dialog with Close/Copy link; they never launch automatically. Other schemes, credential-bearing URLs and control-character destinations are rejected. Opening a browser is not part of this brick.
- Active Markdown updates coalesce at 80 ms; completed replies retain their parsed widget until their source or theme changes. Incomplete fences still render. Replies over 32,768 UTF-16 code units or 800 newline characters use selectable plain text with an explicit label, preserving all content and source Copy. No idle rendering timer is added. These are layout bounds, separate from provider output/context limits.
- The input box uses native editable blocks for Markdown headings and fenced code. Heading markers and fences are hidden; the first-level heading is 24 px and code sits on the sidebar surface with a 12 px radius, syntax colors and a language menu. Ordinary paragraphs retain literal inline Markdown. There are no Source or Add text controls. Down Arrow on the last visual line of a code card moves into the next block, or creates and focuses a paragraph below the final card. Leaving incomplete code closes its fence; new separators follow the draft's line endings. The same exit works after a final heading. Wrapped lines retain native line navigation; Shift+arrows retain selection and active IME composition suppresses block exits and sending. Native drag selection remains per block. Ctrl/Command+A selects every block and highlights both focused and unfocused content. Copy/cut retains canonical Markdown, including hidden fences; Backspace/Delete, typing and paste replace the entire selected draft. Clicking or an unmodified arrow returns to local editing. Backspace removes an empty heading or code card, including its hidden markers, and works at an adjacent empty-block boundary; heading-start Backspace removes heading formatting while retaining its text. These structural edits participate in document undo. Active IME composition retains the focused native field until it commits. Enter inside code inserts a newline, Ctrl/Command+Enter sends, and Shift+Enter inserts a newline anywhere. The canonical Markdown retains its markers, whitespace and line endings when sending; body edits map back to source ranges. A bounded 40-edit document undo history includes keyboard-created exits. No links/images load.
- Creating or removing heading/code formatting preserves the native editor, its controller and input connection so typing continues without refocusing. Shift+Enter inserts a visible newline, replaces a local selection when present, and breaks a heading into a following paragraph. User-entered leading/trailing paragraph newlines remain visible; structural Markdown separators are hidden. Enter in a code card or after an opening fence inserts a newline. These transitions are tested through continuous text-input updates with no extra focus request.
- Composer block layout falls back to complete source above 32,768 UTF-16 code units, 800 newlines or 64 blocks. Code syntax uses the selectively registered Rust, Dart, JavaScript, TypeScript, Python, shell, JSON, Go and C++ grammars from pinned `highlight` 0.7.0. Unknown languages stay editable without coloring. Coloring falls back to plain code above 8,192 code units, 200 newlines or 2,048 token nodes, and during native composition. Tokenization caches cursor-only changes and adds no idle timer. These bounds limit rich processing; they do not cap the complete draft or plain-text shaping cost.
- On narrow screens Flutter uses a sidebar drawer. Flutter/web settings use a dialog with focus containment. Alternative Iced settings occupy the main panel with Back and uncaptured Escape to return; Iced has a whole-message Copy action and lacks arbitrary transcript selection.
- New UI bricks must reuse the tokens and these interactions. Avoid disabled placeholders for features that do not exist.

Acceptance: inspect both themes, narrow layout, long content, settings navigation, streaming and cancellation. Native light/dark/narrow screenshots and controller checks pass on Windows; actual system-theme switching, keyboard/IME input and each platform still need UAT. Iced lacks screen-reader integration, so the shared accessibility contract is not fully met. Retain the webview alternative and do not claim universal accessibility.

## Usage and context

Completed assistant replies have a small muted usage action using the shared theme. It shows reported input/output tokens or Usage unavailable. Its dialog shows the request model, nullable totals/cache/reasoning breakdowns and saved request context; zero is explicit and no count or price is inferred. Legacy replies retain unavailable metadata.

The composer footer sits inside the input card at the bottom right: model picker, context ring immediately to its right, then Send/Stop. The ring is 18 pixels with the standard Material icon-button touch target. Remove the bottom keyboard-help label; existing keyboard shortcuts remain. The ring is determinate only when a summary is known, otherwise a static empty ring with a question mark. Its tooltip/semantic label names the source (last saved/current/attempted request or next-message preview) and the percentage of the saved model context window, with Estimated or Provider reported provenance. Use the latest model call's reported total, never accumulated session totals; opening the inspector replaces that reading with a fresh next-message estimate. Typing invalidates a previous draft preview; no fetch or idle timer runs on typing. Clicking previews latest saved history plus the current draft on demand. Inspection is disabled during generation/loading/other actions.

The context inspector uses a compact percentage/readout, segmented token-composition bar and legend, included/saved/omitted turn counts, and expandable system/history/draft groups showing exact selectable prepared text. Tool definitions and approximate framing have separate token components; advertised schemas are expandable without folder access. Show response reservation and the input allowance after 5% headroom. Preserve unknown legacy readings. Models → Connection & models has a Model settings selector and per-model Context window (tokens) field: blank uses 128K (131072); overrides persist after Save, Close preserves saved values and endpoint changes clear draft overrides. The header trajectory action opens a separate inspector with Trajectory and Log tabs. Trajectory uses independently paged, expandable saved message/reply rows, original model, usage detail and Older/Newer/Latest controls. Log shows local lifecycle observations with timestamps and monotonic durations; it explicitly states its in-process 200-event retention and has honest empty states. There are no fabricated tool steps or historical timings. Paging preserves the main transcript/draft/scroll; the live view subscribes only while open. Both inspectors use a scrollable body, max 720-pixel width / 640-pixel height, 16-pixel viewport margins, native dialog focus/Escape dismissal, and existing semantic colors/fonts.

Interaction inspiration: DeepSeek Harness's [context meter](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/client/ui-conversation/src/client/skeleton/ContextMeter.tsx) and [trajectory timeline](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/client/ui-trajectory/src/client/TrajectoryTimeline.tsx): circular disclosure, composition legend, and inspectable chronological records. Dolores uses its own Flutter implementation; [token accounting research](research/token-context-accounting.md) records pinned upstream conventions. Byte guards remain internal resource limits.

## Request controls and recovery

Brick 9.4 adds scoped controls, now organized under Settings → Models / Personalization / Task limits, using InspectorFrame and the existing system theme. Keep User defaults / This project / This chat, effective origins, grouped generation/interaction overrides and model-window provenance in a scrollable body. Refresh, Use inherited settings and Save stay in a wrapping footer. Preserve drafts after invalid/stale saves and require an explicit new Save after Refresh. Disable mutation during execution and duplicate actions/dismissal during pending storage. Run history shows frozen settings origins when present; legacy snapshots have no invented origins. Keep the header/composer width unchanged; no extra header icon or idle polling is added.

Models → Responses is a Settings section with an embedded scrollable editor, existing theme tokens, labeled output-token and timeout fields, inline whole-number validation, Restore defaults and Save; Close discards unsaved edits. Restoring defaults changes the form until saved. Settings apply to the next message and persist across restarts. A pending save keeps the dialog visible and blocks duplicate actions; failures retain editable values.

The existing error surface shows brief local guidance and manual actions: Retry message when appropriate, Model response settings for timeout/output-limit failures, and Model settings for connection/access errors. Retry uses the currently edited draft; no countdown, background retry or additional composer/footer control is added. Stop restores the draft without a failure-retry action. Completed reply details show the actual saved output limit/timeout when available. Keep the model picker, context ring and Send/Stop inside the input card in their existing order.

## Approved folder tools

Brick 6.1 places **External tools** in the Settings window; Side chats show a working-folder prerequisite. Reuse InspectorFrame, the system palette and scrollable compact layouts. Explicit Inspect server discloses external process permissions, lists literal tool names/descriptions and expandable JSON schemas, and stops the server before selection. Selection never enables automatically. Editing launch fields discards the review; failed Save retains the review for retry. Keep Enable selected tools, Disable and Forget distinct, with Stop during inspection and disabled duplicate actions/dismissal while busy. The surrounding chat stays locked.

External approvals reuse the bounded card above the composer: show server, original tool, revision, exact selectable JSON arguments and the disclosure of broader process access/unjournaled effects. Run once and Deny remain visible beneath the scroll area. Record cards and trajectory retain that provenance and literal text, showing Tool reported an error when `isError` is true. Do not add a composer toggle, automatic server startup, background polling or a new color system.

Brick 3.3 replaces the launch-wide folder toggle with working sessions. The default starter is Temporary workspace: its folder is created on first send. Open project uses the native folder picker and starts a new chat in that folder; the sidebar uses collapsible Projects and Recents sections inspired by the supplied reference. Project chats nest under expandable folder rows, with + to start another chat in that project. Temporary/side chats appear in Recents with a mode label. New chat is a quiet compose-icon text action that starts a temporary workspace; an adjacent menu offers Temporary chat and Side chat. One scroll area contains the grouped current 50-chat page and recent-project roots; shared Older/Newer controls remain fixed below it. The header identifies the current workspace with a folder icon and menu for Open project, Temporary chat, Side chat and Show working folder/Copy path. The model selector stays in the composer; remove the duplicate header model badge to leave space for the folder identity. Nested project rows use their parent folder for scope; recent rows show a temporary/chat icon and Temporary/Side chat label. A selected conversation restores its own folder, and selecting/creating one closes the compact drawer. All mode/project changes are excluded while loading, generating or preparing another action.

Project/temporary chats advertise tools automatically. Side chat explicitly has no file access and uses a separate starter message. Legacy conversations retain Side mode. Temporary folders and project bindings persist with their conversations, and deleting a conversation does not delete working files. Startup and context inspection create no new workspace. The first-send creation failure retains the draft and does not send a tools-disabled request. Keep per-operation Allow once/Deny, literal/partial result cards, and the established composer/model/context/send order. No palette, animation, idle work or additional bottom help label is introduced.

Use the project/temporary session folder; switching sessions or modes is disabled during a run. Reuse existing surfaces, spacing, typography and native focus behavior. The inline approval card above the composer shows the complete selectable relative target, chosen folder name, selected model and the disclosure that one read shares and saves file contents. Allow once and Deny disable while submitting a decision or stopping; Stop stays available. No remembered or blanket approval control is added.

Live and saved tool records use bounded expandable cards with status, relative target and selectable literal result text. Text scrolls inside a 180-pixel result area and never renders file instructions as UI controls or Markdown links. Saved trajectory rows include the same cards. Failures keep their live records until changing run/view; no historical log entries are invented. Successful reply details show each model call's reported usage without an inferred aggregate. Context labels distinguish the initial agent request from later tool-result input. The existing bottom-right model picker, context ring and Send/Stop retain their order.

Listing and search use those same cards, with distinct action titles. Listing explains that it shares bounded names; search shows the exact selectable query, scope and scan limits, explains snippet sharing/storage and requires a separate decision for a full read. Approval text scrolls within 180 pixels while Allow once/Deny remain accessible below it. Result cards display readable names or file/line/snippet text with skipped counts and an explicit Partial results label when truncated. Query selection uses a distinct saved-state key so expanding a search card cannot mix its scroll position with the expansion state. Tool output stays literal and the composer layout remains unchanged.


## Changes and reviewed revert

New-file proposals use the same literal colored diff card with Create this file?, Create once and Deny. Show the complete addition and state that an occupied path is never replaced; snapshots survive reply failures and removal requires another review. Keep the established composer order, system palette and bounded scrolling. Changes labels Created/Edited/Removed independently of Applied/Not applied/Reverted/Needs check, including zero-byte files. A recorded creation opens Remove this created file? with a full deletion diff, Cancel and Remove once. Explain that removal requires the saved bytes and cannot be reversed here. Receipt failures report the actual removal plus Needs check, never imply the file remains present.

Working-folder headers add a quiet Changes icon beside Trajectory; Side chats omit it. Keep the composer and existing model/context/send order unchanged. Reuse InspectorFrame, system theme, literal EditDiff colors and 180-pixel scroll region. Changes holds one 20-row page, newest first, and one selected detail; Newest refreshes, Older replaces the page. Relative path, local time, byte counts, Applied/Not applied/Reverted/Needs check and a Revert label distinguish records without exposing the private root. Records remain available from another chat in the same folder after deleting the original chat.

Review revert checks the current file before opening a separate reverse-diff decision, with Cancel and Revert once. Viewing and cancelling never write. The pending intent disclosure explains uncertain completion; completion failure must never be presented as an unapplied write. Operations disable duplicate decisions and dismissal while pending; closing a ready preview cancels it. The surrounding chat stays locked during this local inspector, with no network call, idle polling or additional worker. Long paths/diffs and errors scroll within the existing 720×640 dialog and 16-pixel margins.

## MCP credentials

External tools keeps the existing InspectorFrame and semantic system palette. Optional credential rows sit after literal arguments: environment name, masked secret value and Remove. Add credential is bounded to eight rows. Explain that Inspect sends keys to the selected program, Enable stores them in secure storage, and a blank saved value reuses it; a changed launch needs re-entry. No reveal, general environment editor or composer label is added. Editing any row discards the review. Enable sends only the review token/tool names and reloads blank masked fields; values never appear in notices, approval or saved tool cards. Per-call approval lists credential names only. Disable retains keys. Forget failure refreshes the disabled state and revision while showing the unlock-and-retry instruction, keeping the next Forget usable. Long forms scroll inside the existing dialog with controls retained in its footer.

## Multiple MCP servers

Brick 6.3 keeps the existing External tools inspector and palette. Place the saved server list and Add server above the selected server form; show per-server enabled/disabled state and the shared two-slot count. Switching servers discards the current review and reloads only that server's fields. Enable, Disable and Forget operate independently, including masked credentials. Failed capacity saves retain the form, selected tools and review. Provide inline Disable beside another enabled server so users can free a slot and retry Enable without switching the form. Scrolling keeps the compact form reachable; status/error feedback returns to the top and footer actions remain available. Forget selects a remaining server or a new blank form. Busy operations disable duplicate actions.

## Practical tool evidence

Read approvals label start/count ranges. Command approvals expose actual requested deadline/capture. Expandable receipts distinguish shortened previews, incomplete captures and log-write errors, name the local log and offer local opening/ranged-read guidance. Keep the existing palette, approval controls and composer unchanged.

## Task checkpoints and draft recovery

Run history keeps its existing system-themed inspector and scrollable evidence, adding original goal, unverified operation plan, uncertain effects and the saved draft. Prepare resume draft is explicit, excludes running/completed work and never replaces an occupied composer. Errors retain selection and text. Returning to the composer requires a normal Send. Saved chats restore local drafts; persistence failures explain copying or retrying without clearing the text.

## Managed threads

Fork conversation uses a completed-turn chooser with explicit shared-folder disclosure; disable it while running. Session summary keeps its inspector layout and adds the chat-specific automatic compaction switch only when the host advertises support. Progress appears in trajectory and durable run evidence; failures preserve the composer draft and link manual recovery. Scoped guidance displays relative paths and precedence in the existing Instructions review.

## Harness mods

Use the unified Settings entry and existing InspectorFrame surfaces. Display a
working-folder prerequisite, separate default-off automatic policy, recovery-case
chooser, literal guidance card, bounded source editor and expandable versions and
history. Refresh, Draft a repair, Test source and Stop stay in the footer while
content scrolls. Errors preserve editable source and explain explicit recovery;
Refresh never replays generation. A separate restore confirmation explains
quarantine. Latest recovery receipts remain visible even with a full history.
No arbitrary HTML/Dart, mod-supplied handlers, credential fields or new composer
controls are contributed by generated mods. Theme and existing interaction
constraints apply equally to the supported literal card.

## Computer use observation

Keep Computer use inside the unified Settings window, using InspectorFrame and
the existing theme. Window refresh, selection and Capture locally are separate
from explicit provider sharing. Show the saved local image, dimensions/DPI,
snapshot age, removal and an independent configured image-model chooser. Keep
the ordinary composer/model/header unchanged. Use a scrollable body and wrapping
footer with Refresh, Stop observation while pending, and Analyze this screenshot.
Pin bounded scrollable errors above the body so recovery remains visible on a
compact viewport. Block duplicate local operations and dismissal while pending.

Unavailability and working-folder/image-model prerequisites must be explicit.
Retain local evidence and inspection text on failed analysis; display a retained
text draft as the inspection question without replacing unrelated draft edits or
attachments. Tool cards use the existing literal evidence style and optional
View local screenshot. No automatic image loading/sharing, recording toggle,
desktop input or new palette is introduced.

For scoped input, Computer use adds explicit captured-window consent, an ordinary
typing/navigation switch, Enable selected-window access and Revoke desktop access.
Show the granted title and 15-minute/restart boundary. Keep Analyze separate from
Start computer-use task, preserve selected evidence/goal on failed grants, and pin
errors above the scrolling form. During a run show the active title and Revoke
above the existing composer, with the normal Stop control. Input approvals display
literal JSON in existing cards and disclose possible external effects and the
need for fresh post-input observation. No permanent idle composer toggle is added.
# Computer-use recovery (14.3)

Paused or stopped computer use routes to Settings → Computer use. Show the saved
original goal, bounded receipts and uncertain effects; distinguish input insertion
from verified application success. Recovery requires a new local screenshot,
current selected-window access and explicit inspection. Do not replay an action
or an approval. Preserve an unrelated composer draft. Compact light/dark layouts
keep inspection controls scrollable and error guidance visible.

## Conversation window sharing (15.2)

The composer plus menu offers Attach file or image and Share window for working
chats. A model can request the same local picker through a saved Window access
pause. Use the existing icons, palette and uniform borders. Show View only /
Control this window, local window titles, an explicit model choice and one Share
and continue action. A first-use image check shares only a generated color image;
failed checks keep the task and offer another model. Do not list private windows
to the provider before sharing or silently change the chat's selected model.

Closing the picker shares nothing. A saved handoff preserves the original goal,
remaining calls/deadline/segments, and an unrelated draft with its attachments.
Reusing enabled access must match the exact chosen target. Input reviews, Stop,
revocation and fresh post-input observation remain required. The legacy desktop
inspector remains available for reconciliation while the unified recovery view
is implemented in 15.7.

## Everyday settings (15.3)

Use six main sections: General, Models, Personalization, Memory, Tools and
Advanced. Search names and legacy terms route to the original typed destination;
Tools and Advanced use a secondary view selector. Cached editors stay mounted
while searching and across wide/compact navigation. General opens from the
sidebar; contextual model recovery still opens Models directly.

Personalization starts at All chats, with explanation/assumption controls visible.
Project/chat customization is deliberate. Close and changing a dirty scope ask
Save / Discard / Keep editing; failed saves retain edits at their destination.
Theme changes persist before applying. The System preview paints a single
miniature window in two palettes, sharing one layout and sidebar.

## Model setup (15.4)

Models shows enabled models as rows, with context capacity and image readiness.
Connect or choose models opens a guided connection form; Connect fetches the list,
manual entry remains available on failure, and Save returns to the model library.
Select a row for one model-details dialog: context, image input and expandable
response tuning. Blank context means 128K. Image flags are configuration, not
proof inferred from a model name. Window sharing offers the bounded transport
check. Keep response defaults and scoped overrides under Advanced/search.

Model details save atomically with a comparison against the loaded values.
Refresh retains edits; a failed save or Close never silently discards them.
An acknowledged save followed by a failed list refresh still reports Saved.


Milestone 15.5: Tools begins with readiness, optional search connections stay
closed, and ready browser details do not lead with installation instructions.
Composer access is a short saved-policy label beside Add; unknown/expired access
must never imply a grant. Connections imports configure only: concrete launch
review precedes inspection and tool activation is separate. Approval cards retain
target/effect/sharing before the decision, with full bounds under Operation details.
No colored left status stripe is introduced.

## Memory and skill authoring (15.6)

Use My preferences / Project facts and All projects / This project labels. Remember this opens editable text with explicit Save. Import/Create skills require exact review before activation; failed/stale reviews retain text. Global skills are accessible before first send. Experimental learning and trials live under Advanced; privacy and activation choices stay independent. Keep details closed by default and never claim a narrow test proves general improvement.
