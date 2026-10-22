# UI design guide

Dolores is a clean, approachable workspace for general users: few words, clear
choices and rich capabilities revealed where they are useful. The approved
direction is **Frost & Graphite**, using the sidebar layout from prototype A,
cool light surfaces, glass selection highlights and responsive, gentle motion.
This guide defines the target design; it does not imply that the native app has
already adopted or qualified it.

Keep this file limited to reusable design rules. Feature behavior belongs in
[design specifications](design/); delivery plans in [ROADMAP.md](ROADMAP.md);
verification results and remaining gaps in [ACCEPTANCE.md](ACCEPTANCE.md).
Update the relevant rule here when a design decision changes; do not append
implementation notes, milestone summaries, test results or copies of feature specs.

## Visual style

- Use cool blue-white surfaces, a pale ice-blue sidebar and restrained slate
  actions in Light mode. Dark mode uses neutral graphite surfaces. Create
  hierarchy through spacing, typography and surface depth; avoid saturated
  gradients, decorative colored stripes and competing accent colors.
- Centralize semantic colors, typography and shared dimensions in
  [theme.dart](../apps/dolores_flutter/lib/theme.dart). Bring those shared tokens
  into alignment with this guide when implementing the redesign; do not preserve
  the former warm palette or introduce feature-local themes. Support System,
  Light and Dark appearance.
- Use these palette anchors, with accessible semantic variants for hover, focus,
  disabled controls, errors and other statuses:

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

- Use readable system fonts, with the existing monospace family for code. Follow
  the 4 px spacing scale, generous whitespace and restrained type sizes. Use
  roughly 8–16 px radii for controls and cards, and 18–24 px for the composer and
  floating settings surface. Cap prose width for readability.
- Preserve the Material infinity mark (`Icons.all_inclusive_rounded`) and the
  Dolores name. Show the brand once per main layout. Reuse existing Material icons.
- Keep decoration purposeful. Small illustrations may support an empty state or
  project identity. Reserve translucent glass and blur for selection and floating
  control surfaces; keep reading, editing, code and evidence surfaces stable and
  legible.

## Glass selection

- Use a shared Liquid Glass-inspired highlight behind the selected primary
  navigation row and settings category. Combine a translucent cool fill, soft
  backdrop blur, a fine reflective rim and a restrained shadow. A subtle ice-blue
  sidebar gradient can make the material visible without adding visual noise.
- Keep labels and icons crisp above the glass. In Dark mode, use a low-opacity
  light rim and graphite translucency. Selection must remain distinguishable
  without relying on color alone.
- Move one highlight between entries instead of flashing separate backgrounds.
  Begin interrupted movement at its current visual position and settle gently
  on the newly selected row. Keep hit targets and text stationary.
- Glass is a visual treatment, not a required shader or package. Provide a clear
  opaque selected surface when transparency is reduced or blur is unavailable
  or too costly. Qualify native rendering and performance separately from the
  browser prototype.

## Layout and navigation

- Use the selected sidebar layout: compact labeled primary navigation, grouped
  projects and recent chats, with Settings at the bottom. Home centers a clear
  invitation and composer, followed by useful projects and recent work. Avoid
  filling unused space with explanatory copy or permanent tool controls.
- Group related controls near the content they affect. Prefer one contextual
  overflow menu to scattered secondary actions or permanent action bars.
- Choose the smallest surface that comfortably fits the task:
  - Composer attachment, model and access controls open compact dropdowns
    anchored to their trigger. A small choice must not open a full side panel.
    Show the current choice and only the brief context needed to choose.
  - Settings opens a centered floating window with category navigation and one
    content area. General, Models, Memory, Access, Connections and Advanced stay
    inside that window. Switch categories without replaying the window entrance.
    An explicit settings link in a dropdown can open the relevant category.
  - Task reviews, activity and substantial workspace tools may use contextual
    side panels. Make them resizable and collapsible, with a clear layout toggle
    and keyboard alternative. Hiding or resizing preserves their contents.
- Dropdowns stay within the viewport and flip above their trigger near the bottom
  edge. They dismiss on outside interaction or Escape, support arrow keys and
  Enter, and return focus to the trigger after selection or Escape. Keep the
  surrounding workspace usable and preserve the composer draft.
- The settings window has a clear close action, bounded scrolling, keyboard focus
  containment and focus restoration. Preserve the underlying workspace and
  unsaved edits; dismissal follows the same draft protections as other forms.
- Adapt to compact widths by wrapping controls and using drawers or section
  selectors. Keep primary actions reachable; avoid clipping and layout jumps.
- At compact widths, let settings occupy the available viewport and adapt its
  category navigation. Keep short dropdowns anchored and bounded. Use a scrolling
  body with a reachable, wrapping action footer for long forms and reviews.
  Reuse the settings window or relevant inspector instead of stacking dialogs.

## Wording and settings

- Lead with a short, familiar name and its control. Add at most one brief
  supporting sentence when it helps the user decide. Empty states and success
  notices should also be short and actionable.
- Keep the initial view understandable without technical knowledge. Preserve
  features through contextual menus, searchable settings and progressive
  disclosure, rather than placing every option or explanation on screen.
- Keep default settings free of paragraphs, On/Off manuals and development
  messages. Performance gates, backend details and qualification results belong
  in documentation, not the product interface.
- Put longer setting explanations behind a circled question-mark icon beside
  the label (`Icons.help_outline`). Show its wrapped hint on hover or keyboard
  focus, and make it accessible on touch without changing the setting or moving
  the layout. Keep the explanation available to assistive technology.
- Keep essential availability, errors and consequences visible in plain language.
  Use wording such as “Not available yet” or “Couldn't save. Try again.” Explain
  the effect and next action; put supporting evidence in optional details.
- Use progressive disclosure for sources, history, usage and technical details.
  Show the controls and current outcome before the explanation.
- Prefer one direct choice for a setting and remember it after a successful save.
  Avoid stacking a selector, duration, acknowledgement and Save button for one
  preference. Keep the last saved value visible if saving fails.

## Motion

- Make interaction feel lively through quick feedback and modest spring settling.
  Buttons can compress slightly on press and release naturally; cards may lift
  subtly on hover. Avoid dramatic scaling, elastic text or continuous decoration.
- Use approximately 150–220 ms for hover and dropdown feedback, 200–300 ms for
  press/release, and 350–450 ms for navigation highlight travel and window or
  panel transitions. Treat these as tuning ranges, with a small overshoot only
  where it helps convey movement.
- Animate position, scale and opacity without shifting surrounding content.
  Respond immediately to repeated input; do not queue animations or delay an
  action until a transition finishes. Animate a window on opening, not every
  time a field or category changes.
- Respect both the OS reduced-motion preference and an in-app motion preference.
  Reduced motion removes travel, bounce and scale effects while preserving clear
  immediate state feedback. Repeating animation is limited to meaningful active
  progress, with a readable static alternative.

## Interaction and recovery

- Preserve drafts, selection and usable results across navigation and failures.
  Failed saves retain edits; unsaved changes offer Save / Discard / Keep editing.
- Prevent duplicate actions while work is pending. Keep relevant progress and Stop
  reachable. Close and Escape follow the same pending-work and draft protections.
- Show actual states and outcomes. Distinguish saved from unsaved, partial from
  complete, and uncertain effects from verified success. Avoid invented progress.
- Give failures a specific recovery action beside the affected content. Viewing
  evidence or pressing Refresh must not silently repeat an external action.
- Treat a deliberate action as the decision when another review adds no useful
  choice. Avoid duplicate approval screens; preserve validation and recovery.
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
- Check affected light/dark and compact layouts, glass contrast and opaque fallback,
  interrupted and reduced motion, dropdown edge placement and keyboard dismissal,
  long labels/content, pending and failed states, and recovery without losing
  work. Match verification to the change:
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
