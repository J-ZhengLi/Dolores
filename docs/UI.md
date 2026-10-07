# UI design guide

Dolores is a quiet, simple workspace for general users. Use this guide for shared
design direction when adding or changing UI.

Keep this file limited to reusable design rules. Feature behavior belongs in
[design specifications](design/); delivery plans in [ROADMAP.md](ROADMAP.md);
verification results and remaining gaps in [ACCEPTANCE.md](ACCEPTANCE.md).
Update the relevant rule here when a design decision changes; do not append
implementation notes, milestone summaries, test results or copies of feature specs.

## Visual style

- Preserve the established warm neutrals, restrained blue accent, subtle borders
  and quiet selected backgrounds. Status cards use uniform borders, without
  decorative colored side stripes.
- Reuse the semantic colors, typography and shared dimensions in
  [theme.dart](../apps/dolores_flutter/lib/theme.dart). Support System, Light and
  Dark appearance; a feature should fit the existing theme rather than invent one.
- Use readable system fonts, with the existing monospace family for code. Follow
  the 4 px spacing scale and 8–16 px corner radii. Cap prose width for readability.
- Preserve the Material infinity mark (`Icons.all_inclusive_rounded`) and the
  Dolores name. Show the brand once per main layout. Reuse existing Material icons.
- Keep decoration restrained. Avoid additional fonts, blur, ornamental animation
  or image assets unless a deliberate redesign calls for them.

## Layout and navigation

- Keep primary navigation compact and separate from each page's side panel.
  Settings stays at the bottom. Preserve the familiar Home conversation layout.
- Group related controls near the content they affect. Prefer one contextual
  overflow menu to scattered secondary actions or permanent action bars.
- Make side panels resizable and collapsible, with a clear layout toggle and
  keyboard alternative. Hiding or resizing a panel preserves its contents.
- Adapt to compact widths by wrapping controls and using drawers or section
  selectors. Keep primary actions reachable; avoid clipping and layout jumps.
- Use a scrolling body with a fixed, wrapping action footer for long forms and
  reviews. Reuse the shared settings/inspector surfaces rather than nesting forms
  in additional dialogs.

## Wording and settings

- Lead with a short, familiar name and its control. Add at most one brief
  supporting sentence when it helps the user decide. Empty states and success
  notices should also be short and actionable.
- Keep default settings free of paragraphs, On/Off manuals and development
  messages. Performance gates, backend details and qualification results belong
  in documentation, not the product interface.
- Put longer setting explanations behind a circled question-mark icon beside
  the label (`Icons.help_outline`). Its wrapped hint appears only on mouse hover,
  without moving the layout; clicking or holding it must not change the setting.
  Keep the explanation available to assistive technology.
- Keep essential availability, errors and consequences visible in plain language.
  Use wording such as “Not available yet” or “Couldn't save. Try again.” Explain
  the effect and next action; put supporting evidence in optional details.
- Use progressive disclosure for sources, history, usage and technical details.
  Show the controls and current outcome before the explanation.

## Interaction and recovery

- Preserve drafts, selection and usable results across navigation and failures.
  Failed saves retain edits; unsaved changes offer Save / Discard / Keep editing.
- Prevent duplicate actions while work is pending. Keep relevant progress and Stop
  reachable. Close and Escape follow the same pending-work and draft protections.
- Show actual states and outcomes. Distinguish saved from unsaved, partial from
  complete, and uncertain effects from verified success. Avoid invented progress.
- Give failures a specific recovery action beside the affected content. Viewing
  evidence or pressing Refresh must not silently repeat an external action.
- Reviews show the concrete target, effect and sharing consequence before the
  decision. Keep full selectable evidence in a bounded scroll area and decision
  buttons reachable below it. Keep secrets masked.
- Keep ordinary text selectable and code/diffs readable with independent
  horizontal scrolling. Preserve complete source text when offering Copy.

## Accessibility and verification

- Provide semantic names, visible keyboard focus, sufficient contrast and usable
  pointer targets. Icon-only actions need understandable hover/focus hints.
- Provide keyboard/menu alternatives to dragging. Preserve native text editing,
  selection and IME composition; respect reduced motion.
- Check affected light/dark and compact layouts, long labels/content, pending and
  failed states, and recovery without losing work. Match verification to the change:
  documentation needs link checks; appearance can use saved renders; native
  interaction needs an appropriate interaction check. Routine changes do not
  require rebuilding, launching or inspecting the desktop.

## Feature-specific direction

Read the relevant specification when changing that area:

- [Workspace navigation, file tabs and splits](design/developer-workspace.md)
- [Source Control and Git diffs](design/source-control.md)
- [Terminal and language services](design/terminal-language-services.md)
- [Conversation and settings organization](design/ux-simplification.md)
- [Automatic memory](design/automatic-memory.md)

Other feature contracts live under `docs/design/`. Keep their behavior and limits
there; use this guide for their shared appearance and interaction principles.
