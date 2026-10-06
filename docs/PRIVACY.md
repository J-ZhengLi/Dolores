# Privacy and public repository hygiene

Source Control reads saved files, Git metadata/configuration and bounded diffs
locally for the selected Home project. These human actions do not send content to
the model or expand tool grants. Reviewed remote actions contact the configured
Git remote using existing credential helpers; configured hooks/helpers run with
the OS account's permissions. Remote/helper diagnostics are withheld because
they can contain credential-bearing URLs. Git views and commit drafts are retained
in app memory; private file recovery remains the separate editor mechanism.

Task permission grants can retain literal command arguments locally and in run snapshots/exports. Keep secrets out of grants. The host shares the selected access mode/boundary as context; credentials remain in the vault. Revoking access prevents future/pending dispatch and cannot retract data already shared or undo effects already started. Commands and MCP still use the OS account's permissions outside the application grant boundary.

## When using Dolores

The appearance choice (System, Light or Dark) is stored locally in the app database. Changing it sends no model or network request. Consolidating the Settings window does not change memory sharing, credential storage or tool permissions.

Approved web operations send literal queries to the selected Mwmbl/Brave/SearXNG
search service or URLs to their public HTTPS hosts. Results then enter the chat
model's context and local run evidence/exports. Default Mwmbl needs no account;
Brave uses a separate OS-vault key sent only to its fixed API endpoint. No model
key, files, cookies or login are sent by this adapter. Queries and URL parameters
can themselves contain private information: review them before allowing a call.
Routing-proxy synthetic DNS addresses cause a public hostname lookup through
Cloudflare's fixed DNS endpoint; private destinations remain blocked. Disable
both web tools in **Settings → Web search**. See [boundaries](design/web-search.md).

The optional browser starts a fresh visible profile for each parent run and
shares literal URLs/input with the chosen site. Approved page text, control
state and action arguments enter model context and local evidence/exports.
Existing browser profiles are never imported. Manual login is an explicit user
action, and cookies/storage are discarded when the run ends. Stop cannot undo
submitted remote effects. Local JPEG screenshots remain in `browser-captures`
after browser or chat closure; they are not automatically sent to the model or
deleted. Settings → Browser shows their location and 128-image capacity. Review
or remove them locally before sharing/backing up data. See
[browser boundaries](design/browser-use.md).

Approved delegation shares the prepared system instructions and original user
request with children using the same configured provider, plus their scoped file
results. Children do not receive full chat history or attachments. Goals, file
scopes, reports, usage and bounded child tool/commentary evidence remain in local
parent run records, including after a later parent failure. Completed delegation
receipts also appear in saved conversations and exports. Inspect these before
sharing; reports can contain approved file content.

Selected text/image attachments are immutable local plaintext snapshots, identified by digest and basename. Preview is local; Send shares the included contents with your configured provider. Sent snapshots can be shared again when included in later context. Ordinary conversation exports retain references; the separate attachment export explicitly copies draft/sent bytes. Removing references and cleaning unused snapshots do not securely erase SQLite free pages, backups or export copies. See [attachment boundaries](design/attachments.md).

Image paste reads the clipboard only when requested, performs any Windows bitmap
conversion locally and creates short-lived local copies for the attachment
bridge. Interrupted operations or OS cleanup failures can leave temporary files.
No clipboard polling or automatic upload occurs.

Approved harness inspection shares current bundled source and up to three
recent failed, paused or interrupted run summaries from the current chat with
the configured model. Summaries contain host-authored categories, build/model
and frozen request limits; arbitrary error bodies and private transcripts are
omitted. Selected source content and approved tool receipts remain subject to
the ordinary local evidence/export rules. Provider reasoning needed for tool
continuation is held in a bounded transient protocol cache, returned to the
same provider and excluded from chat text, journals and exports.

Dolores sends selected context, enabled instructions/memories/skills and approved tool results to your configured endpoint. Provider retention belongs to that provider. The desktop app has no background telemetry or automatic model download.

Conversations, preferences, source evidence, comparisons and change snapshots are local unencrypted SQLite data. Remembered model/MCP keys use the OS vault, scoped to the data directory. Launch-only keys stay in memory. Exports can retain approved content and evidence: inspect them before sharing.

Primary run records also retain the input, effective settings and bounded literal tool intent/results locally, separately from chat turns. They are not automatically shared as context or learning input; deleting the chat removes its run records. Scoped settings contain no credentials and their inspector omits private project roots.

Deleting a preference does not erase earlier reply provenance/exports. Deleting a chat preserves working files and independent change records. Commands/MCP servers run with user permissions and can access data outside the folder. See [user controls](USER_GUIDE.md).

## When contributing or publishing

Raw benchmarks, screenshots, traces, databases, audits and backups stay in ignored `output/`; they may contain personal paths, hardware details and activity timestamps. Never force-add them. Keep production credentials out of code/logs/fixtures; synthetic validation strings must stay clearly synthetic.

Inspect publishable files, branches/tags and commit metadata before publishing. Git authorship is public; this repository uses the author's configured identity. Earlier cleanup removed private benchmark/history artifacts, but subsequent author configuration is not anonymized. Secret/personal-pattern scans detect selected leak classes, not every private fact.

Ignored `output/privacy/` may contain recovery copies of private history. Never upload it. Check archives separately: Git scans do not cover ignored data/build outputs. The Windows packager accepts reviewed runtime paths and user documents, refuses unexpected files/links, and excludes workspaces, databases, credentials and diagnostic outputs by construction.

Compiled code can retain source-machine paths even when Git is clean. The Windows build removes known compiler paths and keeps debug symbols private; packaging refuses current workspace/home prefixes in its payload. Review binaries and third-party assets separately before public distribution. This local identity check is not a universal personal-data or secret detector.

Saved-chat unsent drafts are stored in local session_drafts, bounded by the message allowance and deleted with the chat. They are excluded from conversation exports. Recovery checkpoints retain task goals, local tool evidence and parent provenance; explicit recovery shares bounded previews with the configured provider.

Project knowledge is local unencrypted project-scoped SQLite data. Enabling
receipt learning permits selected approved evidence to enter future requests
from that folder. At most four current facts (4 KiB total) enter next-message
context. Direct source files are rechecked locally; no background scanning or
extra learning request is made. Conservative secret checks are not a guarantee
that all sensitive text can be detected: inspect facts before enabling reuse.
Feedback sharing is independently off by default and currently only records
eligibility for milestone 12 reflection; ordinary task feedback remains local.

Skill learning is a separate off-by-default project policy. Its bounded trials
send exact skill snapshots and disposable fixture data to the configured model;
local inspection, settings and restore make no provider request. Private feedback
notes are not uploaded by reflection. Causes, baselines and quarantine history
are local project data; trial receipts belong to their originating chat and are
removed with it. A matching regression check can make one additional bounded
provider comparison when learning is enabled. No resident learning worker,
filesystem scan, telemetry or global automatic skill rewrite is introduced.

## Desktop observation

Window names and identities are listed locally only after an explicit refresh.
A capture targets the selected visible Windows window, with OS capture borders
enabled; there is no full-desktop fallback or continuous recording. The helper
receives no model keys, provider configuration or general inherited environment.

JPEG screenshots and their title/identity/DPI metadata are local, unencrypted
files in the application-data `desktop-captures` cache. They survive restart and
chat deletion; only their originating chat can select them for analysis. The
cache is capped at 64 screenshots / 32 MiB. Remove screenshots before deleting a
chat if you do not want to retain them. Removal is ordinary file deletion, not
secure erasure, and can make an old evidence reference unavailable.

**Analyze this screenshot** explicitly shares the chosen saved screenshot,
its observation metadata and prepared chat context with your configured provider.
It selects one image-capable model without changing your ordinary selection.
Window discovery and local preview do not send images. Capture contents are
untrusted evidence; they cannot grant tool authority. Screenshot analysis does
not run automatic memory/skill reflection. Saved tools and ordinary exports hold
references/digests instead of image bytes. Shared images cannot be retracted by
removing local evidence. Provider retention is governed by that provider's policy.
