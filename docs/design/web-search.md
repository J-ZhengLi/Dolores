# Attributable web search — brick 11.2

Working project and temporary chats register `web_search` and `read_web_page`
by default. Side chats and delegated children receive neither tool. A global,
revisioned connection is pinned at run preparation; changing it requires the
current run to stop. Disabling it removes both tools from future runs/context.
The compiled adapter is optional and owns no idle process or polling service.

## Connections

- Default: [Mwmbl's unauthenticated API](https://github.com/mwmbl/mwmbl/blob/master/mwmbl/tinysearchengine/search.py),
  `https://api.mwmbl.org/search/?s=…`. No user key or account. A smaller index
  means sparse/empty results are possible; this is not equivalent to exhaustive
  commercial search. Service availability is external and not guaranteed.
- [Brave Search API](https://api-dashboard.search.brave.com/api-reference/web/search/get):
  fixed public endpoint, separately supplied OS-vault key, `q`, count five and
  text decorations disabled. Its subscription/quota remains the user's choice.
- [SearXNG](https://docs.searxng.org/dev/search_api.html): a user-selected public
  HTTPS endpoint with JSON search enabled. No login, query-embedded credential,
  private-network instance or arbitrary authentication-header support.

Changing providers is explicit. An unavailable default never silently falls
back to a paid service, starts a browser, bypasses a challenge or retries.
An empty result describes that index only; refine the query, change providers
or give a primary source URL. Page reads do not depend on search coverage.

## Request and evidence bounds

Search accepts one nonempty literal query, at most 256 UTF-8 bytes. URLs are
at most 1024 bytes. Each operation shares the existing task/tool allowance and
has a 20-second total deadline, including resolution and body reading. One
search yields at most five validated HTTPS source URLs, 256-byte titles and
768-byte snippets. HTTP bodies are capped at 256 KiB, including chunked bodies;
overruns discard partial bodies rather than claiming complete evidence.

Page reads support UTF-8 HTML/plain text. Static HTML extraction removes
scripts, styling, document headers and explicitly hidden elements; it does not
execute a browser or load subresources. HTML extraction additionally bounds
markup/tree nodes to 32768 and visible-text ancestry to 128; overly complex
documents refuse with simpler-source guidance. The model-facing excerpt is at
most 8 KiB, with explicit partial status and a Unicode-character continuation offset.
A subsequent read fetches the page again; a changed page can invalidate an offset.
Unsupported PDF/binary/encoding/redirect responses need another source or a local
text conversion. Redirect destinations are never automatically fetched.

Receipts retain provider/query or exact requested URL, configuration revision,
retrieval time, source links/excerpt and coverage notes. Snippets are not proof
that the whole page was read. Agent instructions require attributable primary
links, honest limitations and explicit distinction between evidence and inference.

## Network and authority boundary

Only public HTTPS on port 443 is accepted, with no URL userinfo or fragment.
Encoded/literal loopback, private/link-local/multicast/documentation/tunnel
addresses and local hostnames are refused. System DNS answers are validated
and pinned to the HTTP connection to avoid resolving twice. Environment proxies
are disabled for this adapter; TLS verifies the original hostname. No cookies,
model credentials or ordinary files are forwarded by the adapter.

Routing proxies sometimes return only synthetic `198.18.0.0/15` addresses.
In that case the hostname alone is resolved with a bounded HTTPS DNS query to
Cloudflare's fixed `1.1.1.1` endpoint. Only validated public IPv4 answers are
pinned; synthetic/private/mixed answers remain refused. Ordinary private DNS
answers do not trigger this path. This third-party DNS disclosure is shown in
settings. Failure remains actionable rather than weakening the network boundary.

External text stays quoted, untrusted tool data. It cannot modify permission
state, add a tool, supply an adapter credential or directly execute a command.
Follow-up proposals still pass existing host checks and Review/Auto/Full policy.
This does not promise semantic prompt-injection immunity: Full access still
permits advertised operations, and a model can misunderstand hostile text.

Literal search queries/URLs go to the named service/site; extracted results go
to the chat model and local run evidence. Approval views disclose this before
execution. A caller can include sensitive words or URL query parameters; the
adapter cannot infer their sensitivity. Inspect what is shared.

## Settings and recovery

`webSettings` is local and usable while busy. `saveWebSettings` is a revisioned
mutation refused during active work; stale/invalid saves retain the UI draft.
Refresh updates the revision while retaining edits. Schema 26 adds one global
JSON configuration row; an absent row means enabled default Mwmbl. SQLite stores
opaque credential references only. Brave keys are fixed-endpoint, sensitive
headers; errors omit raw service bodies and known-key echoes are redacted.

Keys are written before configuration publication, with best-effort cleanup on
failure. Old references are retained until deletion succeeds; cleanup bookkeeping
failure preserves retryable references. If vault and database failures coincide,
best-effort cleanup can leave an unreferenced vault entry; this is not a distributed
transaction. Existing working configuration is preserved.

Cancellation drops owned request futures. No partial request replay or quota
retry occurs. Search errors name the actual service/limit and suggest a provider
change or explicit source. Saved progress survives later model/task limits under
the existing run-journal/Continue contracts. Browser interaction is brick 11.3.
