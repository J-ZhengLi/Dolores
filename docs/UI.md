# Universal UI style

Dolores follows the system's light/dark preference using semantic CSS tokens. The visual language is a quiet workspace: warm neutrals, a restrained blue accent, readable system fonts, subtle borders, and generous space. The name can use a system serif; controls and content use system sans-serif. No external fonts, image assets, animation framework, or blur effects are needed.

- Consistent 4 px spacing scale; 8–16 px corner radii. Content width is capped for readable conversations.
- Sidebar: identity, new conversation, saved sessions, connection settings. Main area: conversation, model status, composer.
- Welcome state explains the next action. Connection, streaming, stopped, failed, empty, and offline-preview states are explicit.
- Use real labels, keyboard focus rings, sufficient contrast in both themes, a polite status region, and Enter to send / Shift+Enter for a newline. Respect IME composition and reduced motion.
- Keep messages as plain text in the first brick. A later audited Markdown renderer can add code and links without interpreting model HTML.
- On narrow screens the sidebar becomes a compact horizontal region and a conversation selector preserves access to saved history. Settings use a native dialog with focus containment and Escape support.
- New UI bricks must reuse the tokens and these interactions. Avoid disabled placeholders for features that do not exist.

Acceptance: inspect both system themes, a narrow viewport, long content, settings keyboard navigation, streaming and cancellation. Desktop webviews must also be checked on each supported platform before release.
