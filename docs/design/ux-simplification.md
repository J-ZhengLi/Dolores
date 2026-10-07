# Everyday Dolores UX — milestone 15

Status: **audit, target contract and selected design**, 2026-10-05; implementation
delivered in milestone 15. The audit below preserves its original baseline;
[qualification](../qualification/everyday-ux.md) distinguishes delivered behavior from acceptance gaps. The user asked
for conversation-first access, useful defaults and simpler settings throughout
the app. Milestone 14's model-reliability gaps remain open independently.

## Audit evidence

Audited the normal Windows release at `c1e4f7e`, with an isolated synthetic project,
two example model IDs and no provider credentials or network requests. Visually
inspected **all 12 settings categories and 17 views**, including Models' three
pages, Memory's two pages and Skills' three pages. Configuration handlers, forms,
empty/prerequisite conditions and related conversation surfaces were also read.
No permissions, sharing, learning policies or credentials were changed during
the native inspection. No screenshots, personal profiles or transcripts are
included in this document.

Native inspection covered the wide dark window. Compact and light behavior was
reviewed in source and existing tests, not requalified natively by this audit.
Populated collections, physical assistive technology and other platforms need
qualification during implementation. This is an expert/source audit, not a
user-study score or a claim of reduced task time.

### Findings and required changes

| Surface | Current friction | Target experience | Delivery |
| --- | --- | --- | --- |
| Settings navigation | Twelve peers give routine settings and executable-mod editors equal weight; nested model selectors and technical subtitles add decisions | Six clearly named sections, settings search, advanced details closed; keep useful deep links | 15.3 |
| Appearance | Three theme cards already work directly; unlike other pages they save immediately | Retain System/Light/Dark and the existing visual style; put under General | 15.3 |
| Models: connection | Capabilities/context controls precede the connection; fetching, choosing, configuring and saving models share one long form | Connect once, fetch automatically after an explicit connection action, select models, edit one model in place | 15.4 |
| Models: responses | A second model selector, raw output/timeout/reasoning fields and precedence paragraphs obscure the effective configuration | One model detail view with context, image capability and response controls; numeric tuning under Advanced | 15.4 |
| Models: overrides | Defaults, model profiles and project/chat overrides are separate pages; the user must understand precedence | Effective value with an origin badge; Change for this project/chat is an optional contextual action | 15.4, 15.7 |
| Personalization | Defaults to This chat; useful explanation controls are hidden behind Override interaction for this scope; unrelated authority text follows | Show actual style controls immediately, edit All chats by default, expose project/chat customization deliberately | 15.3 |
| Memory: preferences | Sharing, filters, request bounds and policy mechanics dominate the empty list | Short learning control, readable memory list, Add memory and context-specific Remember this; details disclose sharing/cost | 15.6 |
| Memory: project knowledge | Learning, feedback-sharing and reflection dependencies are expressed through technical policies | Clearly separate Project facts from My preferences; retain provenance/corrections and explicit feedback-sharing choice | 15.6 |
| Web search | Works without setup but gives DNS/download limits and multiple policy paragraphs before ordinary use | Show Default search ready; another provider is optional; query/service disclosure when a search needs review | 15.5 |
| Browser | Ready screen still displays install instructions, adapter versions, cache paths, limits and extensive restrictions | Ready / Needs setup with one relevant action; guided first-use prerequisite setup when necessary; technical details expandable | 15.5 |
| Computer use | Task entry is buried in Settings; separate refresh/capture/consent/grant/model/question/start actions; ordinary Send never receives desktop control | A normal request can ask for window access in chat; one local window picker with purpose and consent, then continue the original task | 15.2 |
| External tools | Empty page is already a long launch/argument/credential form; users must understand stdio processes and tool selection | Connection list with Add connection; guided local-MCP import/manual setup and a concrete launch/tool review | 15.5 |
| Skills: library | Empty state instructs users to create `.agents/skills/.../SKILL.md` and YAML; Project/Global labels and filesystem paths dominate | Import skill, Create skill, Draft from chat; All projects / This project labels and readable skill cards | 15.6 |
| Skills: trials | Baseline/candidate source, fixture suite names and resource figures appear beside the ordinary skill library | Advanced → Skill testing; keep evidence and unchanged evaluation rules | 15.6 |
| Skills: learning | Reflection, activation and Pause combine into several switches; long paragraphs explain a narrow experimental workflow | Readable learning status and suggestions; experimental policies/tests in Advanced; never conflate the separate opt-ins | 15.6 |
| Harness mods | Recovery category, ABI test envelope and raw WebAssembly source are top-level settings | Advanced → Harness extensions; readable active version/restore state first, source and evaluations in details | 15.6 |
| Permissions | Revision/expiry/containment text precedes the mode; custom paths/exact commands are the main automation editor | Visible per-chat access selector, concise scope/effect summary; custom grants and literal evidence remain accessible | 15.5 |
| Task limits | Model/tool counts, segments and inherited request deadlines are exposed as routine preferences | Advanced execution tuning; when paused, show the actual limit and the relevant Continue/Adjust action | 15.7 |
| Context and summary | Context and summary are separate header actions; compaction configuration requires another inspector | Keep the context ring; one Context view for usage, summary and automatic compaction, with detail expansion | 15.7 |
| Instructions / comparisons / capabilities | Authoring/evaluation/source inspection competes with normal chat actions and uses snapshot/byte/revision jargon | Instructions stays a useful project action; comparisons/source diagnostics live under Advanced or Run details | 15.7 |
| Attachments / model picker / workspace | Capability errors send users away to settings; temporary features may require an initial message; labels still mention removed Model connection entry | Fix in place; explicit compatible-model choice; create a working session when the requested operation needs it; preserve draft and attachments | 15.2, 15.4, 15.7 |
| Feedback / approval / recovery | Literal arguments and generic setup text obscure the proposed effect; errors can require another settings hunt | Short task-specific cards with target, effect and one next step; exact parameters and evidence expand below | 15.5, 15.7 |

Relevant UI owners are `settings.dart`, `settings_frame.dart`, `model_settings.dart`,
`request_settings.dart`, `dolores_settings.dart`, `memory.dart`, `knowledge.dart`,
`web_settings.dart`, `browser_settings.dart`, `desktop_settings.dart`, `mcp.dart`,
`skills.dart`, `skill_draft.dart`, `tool_trials.dart`, `learning.dart`, `mods.dart`,
`task_permissions.dart`, `session_summary.dart`, `usage_details.dart`,
`instructions.dart`, `comparison.dart`, `capabilities.dart`, `attachments.dart`,
`workspace_picker.dart`, `tool_activity.dart`, `run_history.dart` and `main.dart`
under `apps/dolores_flutter/lib/`.

### Cross-cutting causes

1. Settings currently doubles as a task launcher, evidence viewer and developer
   console. A single entry alone did not simplify the user's work.
2. Technical bounds were repeatedly copied into visible text. Keep the bounds
   enforced; put their explanations where the user needs them.
3. Scope machinery is the first decision on otherwise simple preference screens.
   The current chat can silently become the editing target when users expect a
   general setting.
4. Features may exist but be absent from the agent's current tools. There is no
   conversation path for negotiating the missing capability; the model reports
   total absence rather than an actionable access/setup requirement.
5. Some text has drifted: connection recovery still says Model connection;
   generic settings text says executable self-update activation is unavailable;
   skill text assumes every tool always needs approval. These should use actual
   capability/policy state instead of shared hard-coded claims.
6. Save, immediate-save toggles, Refresh, Reload and inherited reset semantics
   differ, and Close discards drafts without a visible unsaved-state warning.

## Conversation surfaces — expanded review and prototype

The 2026-10-05 follow-up extends this milestone beyond configuration. The user
wants everyday work to feel simple without removing capabilities: fewer concepts
to learn, clear actions in context, and information revealed when it is useful.
Reducing prose alone is insufficient if the same confusing navigation remains.

Source inspection confirms five separate header affordances for instructions,
changes, summary, trajectory and Chat actions. The eight-item Chat actions menu
mixes exports, storage cleanup, branching, run evidence and harness diagnostics.
Session summary separately exposes compaction, source review, draft generation,
editing and deletion, while Context and reply usage have their own inspectors.
Run history puts IDs, event counts, effective-setting origins and raw event data
in its routine view. These are source findings; this follow-up did not re-inspect
every native conversation inspector or claim a user-study result.

| Current surface | Proposed everyday experience | Deeper controls retained |
| --- | --- | --- |
| Header icons / Chat actions | Fewer familiar icons with hover/focus hints; a short chat menu for branching, export and chat details | Workspace guidance, evidence and developer tools remain reachable with explicit names |
| Session summary / Context ring | One Context view: estimated usage, readable summary and per-chat automatic compaction | Source review, correct/update/remove summary, provenance, covered turns, token breakdown and output reserve |
| Trajectory / Run history / reply usage | Activity: readable task steps, outcome and pending verification, with earlier tasks available | Exact tool/command receipts, child reports, checkpoints, saved events and per-call usage; unavailable usage remains unknown |
| Changes | File names and purpose first; select a file to inspect its diff | Full paths, before/after snapshots, status, conflict checks and concrete restore review |
| Project instructions / comparisons | Readable project guidance; deliberate edit/review | Source bindings, exact prompts, baseline/candidate evaluation and retained receipts under Advanced |
| Branch / export / attachments | Branch chat and one Export chat flow; Markdown as the ordinary format | Completed-turn selection, JSON and attachment-copy options; unused-asset cleanup under storage settings |
| Feedback / approvals | One short decision beside the relevant reply or operation | Optional notes, exact arguments and evidence expand; preserve effect/target/sharing disclosure before approval |
| Pause / unavailable state | Actual problem and one useful next step beside the task | Limits, saved progress, diagnostics and uncertain-effect reconciliation; no blind replay |

### Prototype before implementation

The throwaway source is captured on **`codex/prototype-everyday-ux`** at
`apps/dolores_flutter/prototype-chat-ux.html`, with its run/review instructions in
`apps/dolores_flutter/PROTOTYPE-CHAT-UX.md`. It uses the existing Dolores shell,
palette, infinity outline and synthetic project data. A single browser route
switches `?variant=A/B/C`; nothing changes the production Flutter app or profile.
Because the desktop app has no browser route to swap, this is a standalone HTML
sketch of that existing page, located next to its Flutter owner on the prototype
branch. It is not a proposed web implementation or new product route.

| Option | Information hierarchy | Cost |
| --- | --- | --- |
| A — quiet header / details drawer | Existing Changes/Activity icons with hover/focus hints; Context/summary opens from the existing ring; details appear only on request | Drawer temporarily covers part of the conversation |
| B — labeled workspace / docked panel | An explicit Workspace panel groups Overview, Changes, Activity and Context | Easier discovery, more occupied space and visible controls |
| C — conversation / contextual actions | Change review, activity and recovery sit beside the relevant response; details expand in the conversation | Strong task association, more scrolling with expanded evidence |

**Selected decision, 2026-10-05:** the design adopts A as the overall design, retaining
the existing New chat/project/chat and other icons. Changes, Activity and Chat
actions remain icon-only with hover hints and keyboard-accessible names/hints.
Keep System/Light/Dark theme preview tiles. Use ordinary surfaces and uniform
borders for paused/error/information cards; do not add decorative colored left
edges. These refinements preserve the app's identity while keeping A's simpler
information hierarchy. C's additional inline change layout has not been selected.
B/C remain archived alternatives. Rewrite A using native components and the actual
handlers; do not merge the mock HTML or switcher into the release. The refined
prototype remains a preview, not a delivered desktop feature.

### Information and copy rules

- First view answers: what is happening, what needs my attention, and what can I
  do next? Technical diagnostics should not compete with the answer.
- Preserve the established icon set and theme previews. Header actions use those
  icons with concise hover/focus hints and semantic names. Status cards use normal
  surfaces/borders; an accent-colored left edge is not the information hierarchy.
- Use one concept per destination: Context for what the model sees, Activity for
  what the task did, Changes for files, Settings for preferences/connections.
  Different data may share a view while retaining its original provenance.
- Lead with one useful sentence. Put IDs, revisions, raw events, exact limits and
  long explanations behind named details. Show a pending verification or uncertain
  effect beside the outcome; it must not disappear into diagnostics.
- Keep sharing, authority, destructive effects and the exact target visible before
  deciding. Expand exact evidence without replacing it with model-written claims.
- Label unknown values and unavailable features honestly; give the relevant retry,
  setup or inspection action instead of a lecture or a generic Refresh button.
- Preserve all existing controls through an explicit old-to-new map in 15.7 and
  validate those paths in 15.8. Fewer routine controls is not permission to remove
  capabilities, change defaults, auto-share, weaken review or discard evidence.

Prototype browser review exercised all three layouts, unified Context, compact
light Settings, simulated summary-save failure/retry and occupied-draft
continuation. These are design affordances only. Native keyboard/accessibility,
full nested editor behavior, truthful durable save/recovery and performance remain
runtime acceptance work. The documented interface design is recorded above; it is
not a measured usability or native acceptance result.

## Target information architecture

| Settings section | Routine content | Optional deeper content |
| --- | --- | --- |
| General | Theme | App/runtime information |
| Models | Connection and selected models; one model's context/image/response settings | Provider-specific reasoning, numeric overrides and connection details |
| Personalization | Explanation style and useful assumption checking | Deliberate project/chat customization |
| Memory | My preferences and Project facts; inspect/edit what is remembered | Learning activity, provenance and sharing details |
| Tools | Built-in readiness, Skills and external Connections | Custom search connection; advanced connection details |
| Advanced | Explicit navigation to execution limits, scoped overrides, skill testing/learning, harness extensions and diagnostics | Source, exact receipts and runtime limits |

Access mode belongs beside the chat composer, with project/chat management under
Tools. Computer-use task initiation and inspection belong in chat. Settings keeps
preferences and readiness; it does not remain the required task-start destination.
Search matches old and new names and returns the exact page/setting, including
advanced settings. Existing recovery links continue to resolve through typed routes.

Use the current Flutter palette, infinity brand, typography, theme following,
bottom-left Settings and bottom-right model/context/send controls. This is an
interaction and information redesign, not a new visual theme. Additional chat
controls must fit in the existing footer and remain usable at 420×480.

## Conversation-first computer use

Desired flow: **ask normally → choose/share a window if needed → Dolores acts →
inspect results in the conversation**. No image capability checkbox, local capture
button or separate Settings task entry should be part of the ordinary path.

Expose a host-mediated access-request capability to ordinary working runs when
the desktop backend is installed. It reports readiness and requests the user's
choice; it cannot grant itself access, discover private windows for the provider
or send screenshots before the user chooses to share. A composer Share window
action offers the same flow without relying on the model choosing the tool.

The local chooser shows the requested purpose and View only / Control this window.
One confirmation creates the appropriate current-chat grant, captures the chosen
window, resolves an image-capable configured model and continues the original
goal. Reuse valid access for the same explicitly requested target with a visible
indicator. Expiry, revoked access or restart asks in the conversation again.
Stop/revoke stays immediately available.

The implementation needs an explicit, durable task-mode handoff: an ordinary run
currently has a fixed tool set, while a desktop run excludes filesystem/browser
tools. Preserve the original goal, task lineage, draft, attachments and **remaining
allowances** across that handoff. Do not manufacture a new unrelated user message,
reset limits, replay earlier inputs or broaden the desktop run's authority. Test
denial, switching targets and interruption during the handoff before live input.

Capability discovery should use documented provider/model metadata when present,
cached successful image support and known profiles with visible provenance.
OpenAI-compatible model lists do not universally advertise vision. Unknown support
gets a bounded check at first use and an in-place recovery; genuine text-only
models get an explicit compatible-model choice. Do not infer arbitrary capability
from a model name or silently switch providers/models. An image test must not send
private pixels before consent, and failure must occur before any desktop input.

The current selected-window Windows backend remains the scope. This UX milestone
does not implement universal desktop automation, other OS backends or undo for
external application effects. Shared-window selection and meaningful effect
reviews are part of using a feature, not a configuration chore. Existing desktop
input review rules remain enforced unless a separately tested explicit policy
change qualifies coverage; fewer screens must not mean hidden authority expansion.

## Defaults, wording and save behavior

Follow-up refinement, 2026-10-06: Advanced opens a short task chooser. Individual
editors show status and necessary controls first; long explanations, budgets,
provenance and source use named details. Sharing, experimental status and action
effects remain beside the relevant decision. Advanced is not exempt from the
short-copy targets below.

- Routine settings show the current value directly. Global preferences default to
  All chats. Optional project/chat overrides show their source and a clear reset;
  existing overrides remain effective after migration.
- Keep current numerical budgets/context defaults and privacy/access policies for
  this UX work. Task failure evidence can justify a separate bounded profile change;
  simply hiding settings cannot fix exhausted allowances.
- A page starts with its useful controls/list, one short explanation and one primary
  action. Aim for at most seven routine controls before Advanced, excluding entries
  in a model/memory/skill list. Normal descriptions use at most two short sentences.
  Revisions, IDs, ABI names, cache paths, byte limits and literal JSON go in details.
- Put essential target/scope/sharing/effect information beside the decision that
  needs it. Retain exact command/arguments, tool identity and evidence in expandable
  details; users can review them before approving. Do not replace them with vague
  success or safety claims.
- Safe immediate preferences such as theme/style acknowledge Saved only after
  storage success and revert the visible value on failure. Multi-field connections
  and grants use one explicit Save/Connect/Allow action. No independent staged
  capability save masquerades as an atomic connection save.
- Edited forms show Unsaved changes. Navigation retains the draft; closing asks
  Save/discard where possible, with a cancel path. Background refresh must not
  replace edits or silently overwrite a newer configuration.
- Recoveries explain what failed and what remains. Show Retry for a failed save,
  Review changes for a stale revision, Configure model for missing credentials,
  and Continue only where the recorded task permits it. Details retain the actual
  error with credentials redacted. Do not use one generic Refresh recommendation.
- Missing prerequisites offer the next action in place. Global preferences/skills
  are accessible without an initial chat message. Side chats keep their explicit
  no-tools identity; converting to working mode is a deliberate action that
  preserves the conversation. Do not quietly grant tools to side chats.

## First-use goals and acceptance

The only universal setup requirement is a usable model connection. Reuse the
user's configured connection. Hosted provider credentials/subscriptions and a
model's actual image support cannot be supplied by a UI redesign.

| Scenario | Required result |
| --- | --- |
| Ask to search with a configured model | Default search can be used from chat without visiting Settings; optional providers remain configurable |
| Ask to inspect/type into a desktop window | Window consent/model readiness resolved in chat; no detour to Settings; observable post-action evidence |
| New model connection | One guided connect/select flow; discovered models populate the picker; context defaults to 128K; image limitations have an in-place recovery |
| Change explanation style | Controls visible immediately; default All chats; origin/reset for deliberate overrides |
| Manage memory/skills before first message | General management remains accessible; project-only operations clearly offer a working-session action |
| Stop or exhaust output/context/tool allowance | Work/draft retained; actual limit shown; appropriate bounded continuation/inspection action; no silent replay |
| Save fails or another window changes settings | Unsaved values remain; saved state is truthful; conflict recovery does not discard edits or overwrite newer settings |

Measure clicks/detours on the same fixture tasks before and after each relevant
brick. The desktop ordinary path must require **zero settings visits**, one target
selection and at most one initial window-sharing confirmation when the model is
ready; consequential action reviews are counted separately. The proposed six
top-level categories replace twelve. These are targets, not accomplished metrics.

For every runtime brick, check light/dark, 420×480 and normal native release,
keyboard/focus and long/error text. Use deterministic unavailable/stale/denied
fixtures plus bounded explicitly selected available-model live checks when behavior needs
a model. Keep model quality separate from UI/host contract passes. Preserve provider
configuration, selected model, history, explicit access policies, memory/skill
versions and local evidence. Commit each brick and record gaps in acceptance.
