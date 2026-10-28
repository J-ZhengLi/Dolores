# UI design guidelines

Use this guide when designing or changing a page, layout, control, wording or
animation. Aim for a calm, precise interface that makes the next action obvious
and keeps deeper features easy to discover. Design for people using Dolores,
without requiring them to understand its implementation.

## Use and maintain this guide

1. Identify the user's task and primary action. Sketch the information hierarchy
   before adding controls or explanatory copy.
2. Apply the relevant guidance below. Reuse shared components and semantic tokens
   from [the theme](../apps/dolores_flutter/lib/theme.dart).
3. Review the result at wide and compact widths, in Light and Dark, with long
   content, keyboard focus and reduced motion. Retain usable work and recovery.

This file owns reusable design guidance. Change it when a principle applies to
multiple kinds of surface. For feature workflows, exact destinations, shortcuts
and state rules, find the owning specification through the [documentation
index](README.md). Put rationale in that specification, delivery notes in the
changelog. Report routine checks in the task response/CI; update a qualification
record only when its acceptance boundary changes.

Keep the guide within 180 lines. The documentation check enforces that budget,
routes feature-specific references away from this file and rejects dated section
headings. Review each addition for general applicability even when checks pass.

## Hierarchy and space

- Give each surface one clear purpose and one visually primary action. Group
  related controls together; show secondary actions in a contextual menu.
- Establish hierarchy with whitespace, alignment, type weight and surface depth.
  Use a short heading only when it helps orient the user. A new card or border
  should separate meaningful groups rather than decorate every row.
- Choose the smallest comfortable surface: an anchored dropdown for short
  choices, a centered window for settings, and a resizable panel for substantial
  tools. Keep the main workspace stable while these surfaces open.
- Align headings, labels, controls and action rows to shared edges. Use a 4 px
  spacing scale, consistent row heights and optical centering for icon/text pairs.
  List and disclosure rows use 16 px horizontal insets by default, including
  expanded evidence. Use zero inset only inside an already padded, aligned group.
  Inline tool history uses quiet rows with 12 px insets and 8 px between rows;
  reserve bordered cards for separate evidence groups.
  Keep title-bar branding compact and show it once per main layout.
  Put icon-only search and the Notifications bell before Settings in that title bar.
  Search opens with a single input; results appear as the user types.
- Reserve sidebars for the current task's content. Floating navigation overlays
  the workspace without reserving height; it remains discoverable and keeps
  underlying controls reachable when hidden.
- At compact widths, wrap rows, reduce navigation to named icons, and use bounded
  scrolling with reachable action footers. Long content and text scaling must
  retain both the primary action and a clear way back.

## Wording and progressive disclosure

- Start with the useful content or control. Use a familiar, short name; add at
  most one brief supporting sentence when it changes the user's decision.
- Put extended explanations behind a nearby help icon that works on hover,
  keyboard focus and tap. Keep the layout stationary when help opens. Reuse
  `HelpTip` / `HelpLabel` for consistent presentation.
- Put technical settings, provenance, receipts and breakdowns in optional details.
  Lead summaries with the outcome and a few aligned values, then disclose evidence.
- Keep errors, validation, meaningful limits and sharing consequences visible at
  the decision point. Give a specific next action and preserve the user's input.
- Label fields above their filled controls using `FieldLabel`. Align switch rows
  and disclosure rows with surrounding fields; keep labels clear of input borders.
- Use an icon alone only for a familiar action with a semantic name and hover/focus
  hint. Avoid repeating the same meaning as a heading, badge, subtitle and paragraph.
- Review every visible sentence: retain it if it helps identify, decide, act or
  recover. Move background explanation into help; remove repeated instructions.

## Color, typography and surfaces

- **White & Graphite:** neutral white Light surfaces, soft gray sidebars, graphite
  Dark surfaces and charcoal actions. Avoid blue surface tints. Support System, Light and Dark appearance.
  Semantic tokens in the theme own exact values; use them consistently.
- Keep reading, editing and evidence surfaces quiet and legible. Use glass for
  floating navigation and selection, giving those controls a distinct material.
- Use system typography and the shared monospace family for code. Limit prose
  width and use weight/size sparingly. Align numeric summaries for scanning.
- Use roughly 8–16 px radii for ordinary controls and 18–24 px for larger floating
  surfaces. Match curvature within a group; keep shadows soft and localized.
- Reuse the existing icon family. Keep icon weight, visual size and spacing
  consistent; preserve the infinity mark and Dolores name.

## Liquid Glass material

- Take inspiration from native Apple controls: a translucent body, softly blurred
  backdrop, directional reflective edge, subtle inner shade and a grounded shadow.
  The effect should read as shaped glass rather than a flat white/gray pill.
- Keep the dock's outer material quieter than its selected lens. Let underlying
  content influence the tint without competing with labels. Use neutral white reflections
  in Light and restrained neutral reflections in Dark.
- Draw one selected lens behind stationary icons and labels. Keep the text sharp;
  the lens carries the moving reflection and depth. A small stretch along travel
  can suggest fluidity, returning to the normal shape at rest.
- Selection owns the fill. Show hover with a subtle rim and keyboard focus with a
  clear outline. Press feedback is a small compression; each state stays readable
  when combined with selection and without relying on color alone.
- Bound blur to the glass shape. Reuse the shared material across dock and
  selection so edges, tint and fallback stay consistent. Avoid stacking multiple
  expensive backdrop filters for one surface.
- Use portable Flutter painting/compositing on Windows. Native-only demos are
  visual references, not platform guarantees. High contrast and reduced-motion
  modes use a stable, opaque selected surface with a clear border.

## Motion and feedback

- Make feedback immediate and settling gentle. Use approximately 120–180 ms for
  hover/press, 220–300 ms for surface entry/exit and 320–420 ms for selection travel.
  Use a restrained spring or eased settling; motion serves orientation.
- Retarget from the current rendered position on rapid input. Keep click targets
  and surrounding content stationary; animate paint transforms and opacity.
- Give floating navigation a short upward reveal and a softer downward dismissal.
  Hover and keyboard use keep it available; hidden navigation releases input and
  focus. Reversing a reveal should continue smoothly from its current position.
- Fade/slightly slide page content, preserving its draft ownership. Animate a
  window's entrance when it opens; category changes keep the window in place.
- Respect the OS and saved app reduced-motion preference. Remove travel, stretch
  and press scaling while retaining immediate state feedback. Animation runs only
  during interaction or meaningful active progress; resting controls stay still.

## Interaction and accessibility

- Preserve drafts, selection, scroll position and usable results through navigation
  and failures. Failed saves retain edits and expose retry; unsaved dismissal
  offers a clear choice to save, discard or keep editing.
- Provide semantic names, selected states, visible keyboard focus, sufficient
  contrast and comfortable targets. Keep native editing and IME composition.
- Anchor dropdowns to their triggers, fit them inside the viewport, and flip them
  near an edge. Use the shared `AppDropdown` for select fields; match rounded
  trigger and menu corners with inset rounded option rows. Use a compact anchored
  menu for Notifications. Support outside dismissal, Escape, arrow keys and Enter; restore
  focus to the trigger. Contain focus within modal windows.
- Keep app-level notices in compact floating surfaces below the title bar; they
  must not reserve layout height. Keep errors dismissible and details accessible.
- Close follows the saved Ask/Quit/Tray preference. Ask allows Cancel and Remember
  my choice. After a choice, hide/quit promptly with bounded recovery; never let a
  busy action or failed save veto Quit. Confirm tray availability before hiding.
- Use the shared notification overlay for app feedback: fixed compact size,
  width-then-height entry/details, retained history, named dismissal and reduced
  motion. Keep field validation at the decision point.
- Make pending work visible, prevent duplicate actions and keep Stop reachable.
  Viewing evidence or refreshing status leaves external actions unchanged.
- Keep running status inside the response: elapsed task time above progress and
  a softly animated status below its latest activity. Vary wording within the
  actual phase; review, questions, queued work and Stop use explicit stable labels.
  Timer updates must not rebuild or scroll the chat. Reduced motion keeps the
  label still. Starting work must not add a header row or shift the workspace.
  Context maintenance keeps the same task and timer, with Updating context inline.
  Keep conversation output visible in chronological order through maintenance
  and reopening. Consecutive tool executions share one collapsed, expandable row
  until the next assistant output; empty model steps and storage segments do not
  split that group. Internal segments must not add archive buttons, Continue
  cards, synthetic user turns or completion notifications.
  Recoverable model waits show Reconnecting to the model inside the same task;
  keep Stop reachable and mark earlier interrupted response text as evidence.
- Keep text selectable, code horizontally scrollable and Copy faithful to the
  complete source. Offer named keyboard/menu alternatives to dragging.
  Branching belongs in each completed reply's footer actions and uses that exact
  turn, including earlier loaded history. Avoid a separate truncated turn picker.

## Design review

Before finishing, inspect the changed surface and answer each applicable question:

- Is the primary action obvious, with useful content visible before explanation?
- Are edges, row rhythm, icon sizes and spacing consistent with nearby surfaces?
- Are long explanations disclosed while errors and decision consequences remain visible?
- Is glass legible over busy content, in both themes and its opaque fallback?
- Do rapid input, interrupted motion and reduced motion leave a clear selected state?
- Can keyboard and compact-window users complete the task and recover their work?

Use saved renders for appearance and focused interaction checks for behavior.
Report checks/gaps in the task response. Update the owning qualification record
when its boundary changes. A smooth fixture animation alone does not establish
native frame timing.
