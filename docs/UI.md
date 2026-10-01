# Universal UI style

Dolores follows the system's light/dark preference using semantic palettes in Flutter. Iced and Svelte retain the shared style as alternatives. The visual language is a quiet workspace: warm neutrals, restrained blue, readable system fonts, subtle borders and generous space. No external fonts, image assets, animation framework or blur effects are needed.

The selected Flutter shell uses the shared palette, 252 px sidebar, readable content cap, rounded composer and system theme. Replies are selectable with Copy; Enter sends and Shift+Enter inserts a newline, with an IME composition guard. Under 760 px the sidebar becomes a drawer. The settings dialog contains focus and hides the key. Remember connection explains OS secure storage; a blank field can retain the saved key. Forget removes the connection, preserving conversations. Recovery warnings appear above the composer with Retry when a remembered connection could not load. Controls are disabled during generation or a connection change. Widget tests cover recovery, key omission, forgetting, Enter/Shift+Enter/Stop and the narrow form; real OS input, theme changes and screen-reader behavior still require UAT.

Connection setup fetches models after the endpoint/key are entered. A searchable checkbox list enables up to 32 choices. Save requires at least one choice. Manual entry is an optional fallback; no model name is required to fetch. Changing the endpoint clears the staged list. The composer model picker preserves the current conversation/draft and is disabled while streaming or changing settings.

- Consistent 4 px spacing scale; 8–16 px corner radii. Content width is capped for readable conversations.
- Sidebar: identity, new conversation, saved sessions, connection settings. Main area: conversation, model status, composer.
- Welcome state explains the next action. Connection, streaming, stopped, failed, empty, and offline-preview states are explicit.
- Use real labels, keyboard focus and sufficient contrast. Flutter and web chat use Enter to send / Shift+Enter for a newline. The alternative Iced chat uses Ctrl/Command+Enter to send and Enter for a newline. Respect IME composition and reduced motion; actual OS input still needs UAT.
- Keep messages as plain text in the first brick. A later audited Markdown renderer can add code and links without interpreting model HTML.
- On narrow screens Flutter uses a sidebar drawer. Flutter/web settings use a dialog with focus containment. Alternative Iced settings occupy the main panel with Back and uncaptured Escape to return; Iced has a whole-message Copy action and lacks arbitrary transcript selection.
- New UI bricks must reuse the tokens and these interactions. Avoid disabled placeholders for features that do not exist.

Acceptance: inspect both themes, narrow layout, long content, settings navigation, streaming and cancellation. Native light/dark/narrow screenshots and controller checks pass on Windows; actual system-theme switching, keyboard/IME input and each platform still need UAT. Iced lacks screen-reader integration, so the shared accessibility contract is not fully met. Retain the webview alternative and do not claim universal accessibility.
