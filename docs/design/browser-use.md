# On-demand browser — brick 11.3

Dolores uses a compiled Rust tool and a supervised Node/Playwright worker.
Playwright is optional; an installed Edge (Windows) or Chrome (Linux/macOS)
supplies Chromium. There is no browser download, idle service, extension or
connection to an existing user browser. The adapter lives beside the desktop
executable in `browser-adapter`, with Node available on PATH. Missing dependencies
produce setup guidance in Settings → Browser rather than starting an installer.

Each parent run owns one browser, one nonpersistent context and one page. First
use launches a visible browser with a fresh profile; no cookies, storage state,
credentials or extensions are imported. Finish, Stop, deadline or dropped owner
terminates its supervised process tree. Windows uses a kill-on-close Job Object;
Unix uses a process group. These are lifetime controls, not an OS sandbox.
Children receive no browser tool. A continued run starts with a new session and
must navigate/inspect again; earlier action tokens cannot be reused.

The `browser` tool supports open, state, click, fill, press, scroll, screenshot and
close. No arbitrary script, CSS selector, upload, download or password input is
exposed. Open accepts HTTPS and literal loopback HTTP for a local development
site. Only the approved origin can load network resources; cross-origin requests,
popups, service workers, WebSockets and downloads are blocked. This limits
many real sites. Same-origin requests use ordinary browser networking: this is
not the DNS-pinned public-page adapter and does not promise SSRF containment.

State exposes at most 60 visible controls and 8 KiB of page text, with explicit
partial status. Password/hidden fields are omitted. Controls receive opaque
references bound to the current page digest and a run-specific state token.
Actions require that token; changes to URL, text or controls invalidate it.
Each action consumes it before dispatch, so uncertain completion cannot be
replayed with the same token. Recover using state, inspect the actual result and
make a fresh decision. This optimistic check cannot eliminate DOM races.

Each operation has a 20-second host deadline, a 5-second action timeout and a
16 KiB result limit. Screenshot captures only the 1000×700 viewport as JPEG,
at most 512 KiB, to a generated `<id>.jpg` in Dolores's local capture folder.
It is local evidence, not automatic image input to the model. Screenshot IDs,
state and literal arguments are retained in existing tool/run records. Local
screenshots remain after the browser closes and can be viewed from tool cards.
The cache accepts at most 128 JPEGs (each at most 512 KiB), with explicit local
folder/removal guidance at capacity and no automatic deletion or remote replay.
The preview loads on explicit View local screenshot; it never fetches a site or
repeats an action. Missing/corrupt captures keep the literal receipt and offer retry.

Review/Auto retain existing per-operation authority. Full access can cover
navigation/inspection/scroll/capture/close, but click/fill/press always require a
fresh user decision because the host cannot classify a website's consequences.
Approval reveals the operation, exact URL or session target and literal JSON.
User review must establish intent for sending, purchases and deployment; page
text cannot supply that authority. Manual authentication is a separate explicit
user action in this fresh visible browser, never silent profile reuse; this first
adapter does not automate passwords or persist login between runs.

Receipts mark webpage content untrusted. A failed action explains uncertainty,
preserves previously saved evidence and instructs a fresh state inspection.
Timeout/Stop ends the browser; already submitted remote effects are not undone.
Missing/closed browser recovery does not impair file/search tools. Browser
results do not prove an entire task is correct, and synthetic fixtures do not
establish general website reliability.

References: [Playwright isolation](https://playwright.dev/docs/browser-contexts),
[browser launch/lifetime](https://playwright.dev/docs/api/class-browsertype),
[locator actions](https://playwright.dev/docs/api/class-locator).
