# Everyday usability reset

Status: approved. Batch A delivered; Batch B is next, in a separate commit. EH10 remains pending behind this user-requested work.

## Outcome and agreed scope

Make starting work, enabling learning, setting up Browser and understanding app
feedback straightforward. Deliver two separate implementation batches: A fixes
items 1–3; B implements item 4 (notifications, General settings and tray close).

Confirmed user decisions:

- Browser gets an explicit in-app setup action.
- Enable Memory and the two Project facts defaults where no saved choice exists;
  preserve saved choices, including explicit Off values.
- Address these four areas first; record other concrete usability findings for a
  later pass instead of expanding this implementation into a whole-app redesign.

Preserve profiles, drafts, results and unrelated work. No live model calls are
needed. Use the active Rust toolchain through rtk and normal desktop build helpers.
No new daemon, plugin framework, generic workflow engine or background updater.

## Findings driving the design

- Project opening creates a new controller and saved session on each completed
  click. The in-flight guard does not reuse an existing empty session.
- AutomaticMemoryPolicy defaults to false although automatic-memory.md describes
  enabled-by-default behavior. Project KnowledgeState defaults both switches off.
  This drift needs correction in the code and owning documentation together.
- Browser settings exposes technical status and points to repository instructions;
  the user cannot complete setup there. The existing installer already supplies a
  pinned, staged, explicit installation pattern worth retaining.
- Model saves insert success text into the form, shifting its layout. The bell
  already owns durable task attention and Windows notification preferences.
- Windows tray support already exists, but hiding is coupled to the background
  scheduling policy. A normal close preference should not require understanding
  the scheduling implementation.

These are concrete instances of duplicate state creation, implementation details
in user flows, inconsistent contracts, shifting feedback and coupled settings.
They justify targeted simplification; they do not establish that the entire
architecture needs replacement.

## Batch A: session reuse, defaults and Browser setup

### Session creation

Route New chat, project sidebar actions and Your projects cards through one
open-or-create operation. Reuse the newest eligible empty session in the same
conversation kind and canonical project root, including sessions restored from
storage. Concurrent requests for the same destination converge on that session
and navigate to it. The UI creation path must remain idempotent across a delayed
response or retry, rather than relying on a click debounce alone.

An eligible session has no messages, draft, attachments, queued input, run history,
plan or linked conversation context. Preserve existing choices on a reused empty
session. Never clear or repurpose a session containing work. Project and temporary
sessions remain distinct; explicit linked side conversations are never merged.
Retain existing duplicate records rather than automatically deleting history.

Affected areas: AppHost/ChatController navigation, bridge/store session creation,
sidebar/landing callbacks and focused conversation tests.

### Memory defaults

Default automatic preference learning, project evidence learning and feedback
eligibility to On for absent policies. Retain every saved policy as-is. Keep the
existing bounded learning/reflection behavior; enabling feedback eligibility does
not start reflection or add a new background worker. Explain sharing consequences
briefly beside the control and reconcile the memory/privacy specifications.

### Browser setup

Lead with what Browser does and a real next action: Set up browser, Check again,
or a missing-prerequisite action. Detect Node/npm and installed Edge separately
from adapter presence. Ready means verified runtime prerequisites, not merely a
directory existing. Keep versions, limits and capture paths in optional details.

The explicit setup action installs only the pinned adapter into app-managed,
versioned local storage. Embed its manifests in the application; do not require
Python, a source checkout, administrator access or writes beside the executable.
Reuse the existing staging/verification approach, fixed registry and disabled npm
lifecycle scripts. Discover existing packaged adapters compatibly. Missing Node
or Edge gets a specific official download action followed by Check again.

Show bounded installation progress, cancellation and useful retry. A failure
retains any working installation and unrelated files. No browser or runtime is
silently installed. No browser launches until explicitly used or checked through
an explicit test action. Installer logs must not expose private environment data.

Acceptance: repeated/concurrent project or New chat clicks create one eligible
session; restart reuse and failed-response retry do likewise; drafts and differing
projects remain distinct. Fresh/default and saved-Off memory cases survive reopen.
Browser setup works from a packaged app with missing/ready prerequisites, and
installation failure/cancel leaves the previous adapter usable.

## Batch B: notifications and close behavior

### Toast and details

One app-owned notification presentation path, above dialogs and below the title
bar, overlays content without reserving layout height. First migrate model save
feedback, then the existing transient app notices and task attention producers
that fit the same contract. Keep field validation beside its field and decisions
inside their owning task; a toast never substitutes for an approval.

Recommended reversible visual defaults: compact toast 320 x 56 logical pixels,
with a viewport/text-scale fallback; one icon and short title, truncated as needed.
Entry progresses from a small dot to full width, then full height. Use roughly
160 ms per stage, with content revealed after its surface opens. Clicking expands
width first, then height into a bounded details surface (about 420 x 280), with
scrollable long content and explicit actions. Retarget interrupted motion smoothly.
Use existing theme/glass tokens; reduced motion immediately presents stable states.

Auto-hide after about five seconds; pause the timer while hovered, keyboard
focused or expanded. Upward swipe dismisses; provide Escape and a named dismiss
button for mouse/keyboard access. Expanded details stay until dismissed. Show one
toast at a time with a bounded queue; coalesce repeated saves from the same source.
Routine feedback must not steal typing focus.

### Bell and persistence

Extend the existing retained attention model into one notification inbox with
stable IDs, creation time, severity, safe summary/details, seen state and optional
current-target navigation. Avoid a second task event stream. Persist notices when
created so expiry, swipe or restart cannot lose them. Auto-expired notices remain
unread until viewed or explicitly marked read; dismissing a toast does not remove
its history. Keep bounded history (proposed: latest 100 ordinary notices), while
unresolved task decisions remain discoverable through their existing task records.

The bell opens Notifications: recent entries, unread badge, details, mark read and
clear read history. Opening details marks read; Open task resolves current state.
Clearing history never deletes task results or approves/resolves a decision.
Preserve existing stale-target recovery and single native-delivery claims.
Do not store credentials or provider connection secrets in notification details.

### General settings and tray

Move Windows notifications and event choices to Settings > General. Keep native
delivery opt-in and privacy-preserving. The bell is for reading updates.

Add When closing Dolores: Ask every time / Close the app / Minimize to system
tray. With Ask selected, Close opens a compact choice with Remember my choice
(initially unchecked), plus Cancel. General can change/reset the choice. This
deliberately revises the current immediate-hide rule: choose the destination
first; after selection, hide/quit promptly and use bounded best-effort recovery.
An open settings/review surface must not silently veto the chosen close action.

Reuse the existing Windows tray owner, Open/Quit menu, single-instance identity
and Explorer-restart recovery. Minimize keeps current work running in the same
process; Quit stops owned work with existing bounded recovery. Keep the scheduled
background preference explicit under General (Continue scheduled tasks in tray),
preserving its saved value; minimizing alone must not silently enable it.
If tray creation fails, retain/restore the visible app and offer retry or explicit
Quit instead of stranding an invisible process. No startup installation is added.

Affected areas: shared Flutter notification surface, existing AttentionHost/panel,
model/settings producers, retained store/bridge notices, General settings,
DesktopFrame/AppHost/BackgroundHost and Windows availability integration.

Acceptance: success feedback never moves the form; all animation stages work
through rapid clicks, reversal, reduced motion and compact windows. Auto-expiry,
swipe, details, unread state and restart preserve the same notice. Stale task
notices navigate safely. Both close choices, Cancel, remembered choice/reset,
tray Open/Quit, tray failure and active work recovery work in the native app.

## Delivery and validation

1. Approve this design once, then implement and commit Batch A independently.
2. Implement and commit Batch B independently; update owning behavior specs,
   UI guidance, privacy/user guide and qualification boundaries where changed.
3. Use focused Rust/Flutter tests and analyze for affected logic. Build using the
   normal helper, inspect saved Light/Dark/compact renders, and exercise an
   isolated native Windows profile for tray/window and setup interactions.
4. Report fixture/rendered/native evidence separately. Preserve the real user
   profile; no live-model checks or clean-machine claims by inference.
5. Record additional concrete usability findings here without fixing unrelated
   systems. Resume EH10 only after the requested usability work is complete.

## Remaining scope limits

No whole-app settings rewrite, general browser compatibility expansion, saved
browser login support, automatic Node/Edge installation, cross-platform tray
qualification or notification delivery guarantee. Existing tool permissions and
model execution limits remain owned by their current contracts.

## Delivery state

- Batch A: empty-session reuse, missing-policy memory defaults and explicit
  Browser setup delivered. Public C ABI save/reopen checks cover chat identity,
  drafts, saved Off policies and a real adapter installation without a model
  request or browser launch. Evidence: `output/usability-a-native-final/receipt.json`.
  Browser setup unit failures/cancellation and compact Light/Dark renders cover
  their declared local boundary; general website reliability remains outside it.
- Batch B: pending implementation under this approved design.
