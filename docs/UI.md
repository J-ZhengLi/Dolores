# UI design guide

Apply this contract when changing desktop appearance, layout, wording or
interaction. **Frost & Graphite** means cool blue-white Light surfaces, neutral
graphite Dark surfaces, glass selection highlights and gentle spring motion.
Design for general users: concise labels, compact contextual controls and deeper
features available through progressive disclosure.

## Apply this contract

1. Identify the affected surfaces and read the matching
   [feature contract](#feature-contracts). Use this guide for shared presentation
   and interaction; use the feature contract for behavior and limits.
2. Apply the relevant rules below through shared components and
   [theme.dart](../apps/dolores_flutter/lib/theme.dart). Each changed surface must
   retain its feature behavior and recovery actions.
3. Run the applicable [verification](#verification). Record exercised flows and
   remaining gaps in [ACCEPTANCE.md](ACCEPTANCE.md); distinguish design intent
   from verified native behavior.

Maintain reusable rules here, feature behavior in [design specifications](design/),
and delivery sequencing in [ROADMAP.md](ROADMAP.md). Edit the owning rule when a
decision changes.

## Visual style

- Support System, Light and Dark appearance. Use a pale ice-blue sidebar and
  slate actions in Light mode; use neutral graphite in Dark mode. Build hierarchy
  with spacing, typography, subtle borders and surface depth.
- Centralize semantic colors, typography and dimensions in the shared theme.
  Align its tokens with these target palette anchors; derive accessible hover,
  focus, disabled and status variants there:

  | Role | Light | Dark |
  | --- | --- | --- |
  | Canvas | `#F5F8FC` | `#1B1B1B` |
  | Raised surface | `#FFFFFF` | `#262626` |
  | Sidebar base | `#E5EDF8` | `#222222` |
  | Primary text | `#253044` | `#EFEFEF` |
  | Secondary text | `#68778B` | `#A6A6A6` |
  | Border | `#DCE4EF` | `#3B3B3B` |
  | Primary action | `#35475F` | `#E2E2E2` |
  | Soft emphasis | `#E4EDF8` | `#353535` |

- Use system fonts and the existing monospace family for code. Follow the 4 px
  spacing scale, generous whitespace, roughly 8–16 px radii for controls/cards
  and 18–24 px for the composer/settings window. Cap prose width for readability.
- Preserve the Material infinity mark (`Icons.all_inclusive_rounded`) and the
  Dolores name. Show the brand once per main layout. Reuse existing Material icons.
- Use uniform borders on status cards. Reserve translucent glass and blur for
  selection and floating controls; keep reading, editing and evidence surfaces
  stable and legible. Small illustrations may support empty states or project
  identity.

## Glass selection

- Use a shared Liquid Glass-inspired highlight behind the selected primary
  navigation row and settings category. Combine a translucent cool fill, soft
  backdrop blur, a fine reflective rim and a restrained shadow. A subtle ice-blue
  sidebar gradient can make the material visible without adding visual noise.
- Keep labels and icons crisp above the glass. In Dark mode, use a low-opacity
  light rim and graphite translucency. Selection must remain distinguishable
  without relying on color alone.
- Move one highlight between entries from its current rendered position, using
  the motion rules below. Keep hit targets and text stationary.
- Choose the rendering technique to meet native performance requirements.
  Provide an opaque selected surface when transparency is reduced or blur is
  unavailable or too costly.

## Layout and navigation

- Use a sidebar with compact labeled primary navigation, grouped projects and
  recent chats, and Settings at the bottom. Home centers a short invitation and
  composer, followed by useful projects and recent work.
- Group related controls near the content they affect. Prefer one contextual
  overflow menu to scattered secondary actions or permanent action bars.
- Choose the smallest surface that comfortably fits the task:

  | Task | Surface |
  | --- | --- |
  | Composer attachment, model or access choice | Compact dropdown anchored to its trigger; current choice and brief decision context |
  | General, Models, Memory, Access, Connections or Advanced settings | One centered floating window with category navigation and one content area |
  | Task review, activity or substantial workspace tool | Contextual side panel with resize/collapse controls and keyboard alternatives |

- Route broader configuration through an explicit dropdown link to the relevant
  settings category. Keep short choices in dropdowns rather than side panels.
- Dropdowns stay within the viewport and flip above their trigger near the bottom
  edge. They dismiss on outside interaction or Escape, support arrow keys and
  Enter, and return focus to the trigger after selection or Escape. Keep the
  surrounding workspace usable.
- The settings window has a clear close action, bounded scrolling, keyboard focus
  containment and focus restoration. Category changes update its content in place.
- Preserve panel contents when hiding or resizing. At compact widths, wrap
  controls and use drawers or section selectors for substantial workspace tools.
- At compact widths, let settings occupy the available viewport and adapt its
  category navigation. Keep short dropdowns anchored and bounded. Use a scrolling
  body with a reachable, wrapping action footer for long forms and reviews.
  Keep primary actions reachable without clipping or layout jumps. Reuse the
  settings window or relevant inspector for deeper forms.

## Wording and settings

- Lead with a short, familiar name and its control. Add at most one brief
  supporting sentence when it helps the user decide. Empty states and success
  notices should also be short and actionable.
- Use progressive disclosure for deeper features, sources, history, usage and
  technical details. Keep them reachable through contextual menus, searchable
  settings or optional details. Put performance gates, backend implementation
  notes and qualification results in documentation.
- Put longer setting explanations behind a circled question-mark icon beside
  the label (`Icons.help_outline`). Show its wrapped hint on hover or keyboard
  focus, and make it accessible on touch without changing the setting or moving
  the layout. Keep the explanation available to assistive technology.
- Keep essential availability, errors and consequences visible in plain language.
  Use wording such as “Not available yet” or “Couldn't save. Try again.” Explain
  the effect and next action; put supporting evidence in optional details.
- Prefer one direct choice per setting and persist it after a successful save.
  Keep the last saved value visible if saving fails.

## Motion

- Use modest spring settling: slight button compression on press, natural release
  and subtle card lift on hover. Keep text and surrounding layout stable.
- Use approximately 150–220 ms for hover and dropdown feedback, 200–300 ms for
  press/release, and 350–450 ms for navigation highlight travel and window or
  panel transitions. Treat these as tuning ranges, with a small overshoot only
  where it helps convey movement.
- Animate position, scale and opacity. New input retargets movement from its
  current rendered state and takes effect immediately. Animate a window entrance
  only when opening it; field and category changes keep the window stationary.
- Respect both the OS reduced-motion preference and an in-app motion preference.
  Reduced motion removes travel, bounce and scale effects while preserving clear
  immediate state feedback. Repeating animation is limited to meaningful active
  progress, with a readable static alternative.

## Interaction and recovery

- Preserve drafts, selection and usable results across navigation, dropdown
  choices, settings changes and failures. Failed saves retain edits; dismissing
  unsaved changes offers Save / Discard / Keep editing.
- Prevent duplicate actions while work is pending. Keep relevant progress and Stop
  reachable. Close and Escape follow the same pending-work and draft protections.
- Show actual states and outcomes. Distinguish saved from unsaved, partial from
  complete, and uncertain effects from verified success.
- Give failures a specific recovery action beside the affected content. Viewing
  evidence or pressing Refresh must not silently repeat an external action.
- Treat a deliberate action as the decision when another review adds no useful
  choice. Preserve validation and recovery at that decision point.
- Reviews show the concrete target, effect and sharing consequence before the
  decision. Keep full selectable evidence in a bounded scroll area and decision
  buttons reachable below it. Keep secrets masked.
- Keep ordinary text selectable and code/diffs readable with independent
  horizontal scrolling. Preserve complete source text when offering Copy.

## Accessibility

- Provide semantic names, visible keyboard focus, sufficient contrast and usable
  pointer targets. Icon-only actions need understandable hover/focus hints.
- Provide keyboard/menu alternatives to dragging. Preserve native text editing,
  selection and IME composition.

## Verification

Complete each applicable check for the changed surfaces. Record any unexercised
native behavior as an acceptance gap.

| Changed area | Completion criterion |
| --- | --- |
| Documentation | Local links resolve; rules are self-contained and internally consistent |
| Appearance/layout | Saved renders inspected in affected Light, Dark and compact layouts, including long labels/content and glass contrast/fallback |
| Motion | Rapid repeated input retargets correctly; reduced motion gives immediate readable state feedback |
| Dropdowns/settings | Edge placement, keyboard navigation/dismissal, focus restoration and draft retention exercised |
| Stateful actions | Pending, failed and recovery paths exercised; usable work retained and next action available |

Use native interaction checks where focus, input or rendering depends on native
behavior. Build and launch only when the change requires that evidence.

## Feature contracts

Read the matching specification when changing:

- **Workspace navigation, file tabs or splits:** [Developer workspace](design/developer-workspace.md).
- **Source Control or Git diffs:** [Source Control](design/source-control.md).
- **Terminal or language services:** [Terminal and language services](design/terminal-language-services.md).
- **Conversation or settings organization:** [UX simplification](design/ux-simplification.md).
- **Automatic memory:** [Automatic memory](design/automatic-memory.md).

For other feature behavior and limits, read the corresponding contract under
[design specifications](design/).
