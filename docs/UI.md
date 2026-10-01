# Universal UI style

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

Shared layout: 252 logical-pixel sidebar, drawer below 760, 824-wide outer conversation column with inner padding; normal message text 14 px / 1.65 line height, labels 11–14 px. Use Segoe UI with platform fallback for controls and Georgia with fallback for the existing identity/welcome headings. Component radii range from 6 px for status tags to 14 px for the composer, with 18 px reserved for the welcome mark. Existing measurements may include optical adjustments; new spacing follows the 4 px scale. Material widgets may derive their interaction colors from the shared accent. Prefer labeled text actions and existing Material icons.

Dolores follows the system's light/dark preference using semantic palettes in Flutter. Iced and Svelte retain the shared style as alternatives. The visual language is a quiet workspace: warm neutrals, restrained blue, readable system fonts, subtle borders and generous space. No external fonts, image assets, animation framework or blur effects are needed.

The selected Flutter shell uses the shared palette, 252 px sidebar, readable content cap, rounded composer and system theme. Replies are selectable with Copy; Enter sends and Shift+Enter inserts a newline, with an IME composition guard. Under 760 px the sidebar becomes a drawer. The settings dialog contains focus and hides the key. Remember connection explains OS secure storage; a blank field can retain the saved key. Forget removes the connection, preserving conversations. Recovery warnings appear above the composer with Retry when a remembered connection could not load. Controls are disabled during generation or a connection change. Widget tests cover recovery, key omission, forgetting, Enter/Shift+Enter/Stop and the narrow form; real OS input, theme changes and screen-reader behavior still require UAT.

Connection setup fetches models after the endpoint/key are entered. A searchable checkbox list enables up to 32 choices. Save requires at least one choice. Manual entry is an optional fallback; no model name is required to fetch. Changing the endpoint clears the staged list. The composer model picker preserves the current conversation/draft and is disabled while streaming or changing settings.

History uses one 50-conversation sidebar page and one 80-message transcript page, with Older/Newer and Latest actions making omitted history visible. Paging replaces the displayed page; it does not accumulate hidden message widgets. Export in the conversation header offers complete Markdown or JSON and opens the platform Save dialog. Existing filenames are rejected with a clear choose-another-name message; cancel does nothing. Paging/export are disabled during generation. In-process switching restores drafts and scroll positions for the 20 most recently visited views; they are not durable drafts. New conversation has its own draft. Sending from an earlier page returns to the latest page first. Browsing is separate from the model's bounded context window.

- Consistent 4 px spacing scale; 8–16 px corner radii. Content width is capped for readable conversations.
- Sidebar: identity, new conversation, saved sessions, connection settings. Main area: conversation, model status, composer.
- Welcome state explains the next action. Connection, streaming, stopped, failed, empty, and offline-preview states are explicit.
- Use real labels, keyboard focus and sufficient contrast. Flutter and web chat use Enter to send / Shift+Enter for a newline. The alternative Iced chat uses Ctrl/Command+Enter to send and Enter for a newline. Respect IME composition and reduced motion; actual OS input still needs UAT.
- Keep messages as plain text in the first brick. A later audited Markdown renderer can add code and links without interpreting model HTML.
- On narrow screens Flutter uses a sidebar drawer. Flutter/web settings use a dialog with focus containment. Alternative Iced settings occupy the main panel with Back and uncaptured Escape to return; Iced has a whole-message Copy action and lacks arbitrary transcript selection.
- New UI bricks must reuse the tokens and these interactions. Avoid disabled placeholders for features that do not exist.

Acceptance: inspect both themes, narrow layout, long content, settings navigation, streaming and cancellation. Native light/dark/narrow screenshots and controller checks pass on Windows; actual system-theme switching, keyboard/IME input and each platform still need UAT. Iced lacks screen-reader integration, so the shared accessibility contract is not fully met. Retain the webview alternative and do not claim universal accessibility.
