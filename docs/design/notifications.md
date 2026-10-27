# Notifications and closing Dolores

The title-bar bell opens retained notifications. One app-owned overlay presents
save confirmations, existing app notices and new task attention; field validation
and task decisions stay in their owning surfaces. The overlay sits below the
title bar, above dialogs, and never reserves layout height or steals typing focus.

## Toast and inbox

Task alerts are for background work: another conversation, or work running while
the app is unfocused, minimized or in the tray. The focused conversation reports
completion, failures, reviews and questions inline, without creating a toast or
inbox entry. Collection advances its event cursor even for skipped foreground
events, so leaving the task does not replay them later. Ordinary app feedback
retains its existing behavior. A focus change during pending native handoff
suppresses the late alert; existing background history remains navigable.

A compact toast uses a fixed 320 x 56 logical-pixel surface, constrained for small
viewports and enlarged text. Entry expands a dot horizontally, then vertically.
Click expands width, then height into bounded scrollable details (up to 420 x 280).
Motion retargets during input; reduced motion presents stable states immediately.
The timer pauses for hover, focus and expanded details. Upward swipe, the named
Dismiss button and Escape dismiss. Expanded details remain until dismissed.

Toasts expire after approximately five seconds. Their records are saved before
presentation, so expiry, dismissal and restart preserve history. One toast is
visible, with at most four queued; excess notices remain in the inbox. Repeated
ordinary notices from the same source coalesce within five seconds. The inbox
retains the latest 100 ordinary app notices and the existing 256 task notices in
50-entry pages. Opening details marks read; Clear read removes only notice records,
not conversations, results, drafts or pending decisions. Run evidence remains in
task history. Windows delivery remains opt-in, generic and independently claimed.

The inbox shows time, unread state, details and Open task where applicable. Open
task resolves current state; it cannot approve, answer or replay a saved event.
An unavailable target retains recovery guidance. A live originating surface can
offer its existing explicit Retry action; callbacks are not persisted or replayed
after restart. Failed notification storage keeps a local fallback with an honest
retention warning, without changing the operation's acknowledged outcome.

SQLite migration preserves existing notice identities, delivery claims, preferences
and source cursors. App notices do not create artificial chats or run records.
Ordinary notices use bounded nonsecret summaries/details and never native OS
delivery. The app uses the existing process/profile owner, without a new service.

## General and Close

Settings > General owns Windows notifications, its task-event choices, When closing
Dolores, and Continue scheduled tasks in tray. The duplicate Background page is
removed. Close preferences use revisioned local storage and default to Ask every
time for profiles without a saved choice. Existing background-scheduling values
are preserved independently.

Ask offers Close the app, Minimize to system tray, Cancel and an initially unchecked
Remember my choice. General can reset the preference to Ask. An explicit choice
works while another settings/review surface is open; Cancel preserves that surface.
After Quit is selected the window hides promptly, performs bounded best-effort
recovery, stops owned work and exits. Saved data is retained; a failed checkpoint
cannot guarantee recovery of the latest unsaved edits or veto Quit.

Minimize creates the Windows tray icon before hiding. Current runs and terminals
stay in the same process; new scheduled dispatch while hidden follows the saved
background policy. Companionship stays quiet. A tray failure retains/restores the
visible window and offers an explicit Quit. Open/another launch restores the same
profile owner, and tray Quit exits without reopening the window. No startup entry,
second process, OS wake promise or automatic effect approval is introduced.

See [closed UI scheduling](closed-ui-scheduling.md) for scheduled ownership and
[qualification](../qualification/usability-reset.md) for the evidence boundary.
