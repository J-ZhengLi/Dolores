# Everyday harness continuation

Local session handoff, October 25, 2026. Read current `AGENTS.md` first.
This file is ignored by Git and is a navigation aid, not a replacement for the
owning specifications or current code.

## Resume here

Continue **EH09.2: remote MCP OAuth sign-in and recovery**, then EH09.3–EH09.5
and EH10 in dependency order. H21 remote task supervision remains deferred.
The user authorized implementing the remaining plan and committing each completed
batch/brick. This handoff request ends the current implementation session; no
runtime changes were made while preparing it.

Start with [current roadmap](ROADMAP.md), the
[batch index](plans/everyday-harness/README.md), then
[EH09](plans/everyday-harness/eh09.md), especially its **Remote MCP implementation
slices** and **OAuth protocol brick** paragraphs. Read the
[MCP specification](design/mcp-connection.md#oauth-protocol-foundation) for the
current protocol contract. Consult the
[umbrella](design/everyday-harness-implementation-plan.md) for cross-batch scope.

## Workflow changed

`142a5c7` replaces the old documentation-heavy workflow. Follow the checked-in
`AGENTS.md` and task-based documentation routes; read only needed sections.

- Report routine checks in the task response/CI; keep raw evidence in ignored
  `output/`. Update a feature specification when its contract changes, the selected
  plan when scope/status changes, and qualification when an acceptance boundary
  changes. Do not recreate dated task diaries in `docs/ACCEPTANCE.md` or duplicate
  the same result across plans/specifications/UI guidance.
- `docs/ACCEPTANCE.md` is now a compact limitations index. Old detailed evidence
  is available through [history lookup](IMPLEMENTATION_HISTORY.md#acceptance-records),
  not as thousands of current startup lines. `docs/UI.md` is reusable design
  guidance; feature workflows belong in their own specification.
- Preserve unrelated work and user profiles. Commit completed work in English;
  do not push unless requested. Use the commit skill before committing.
- Prefix shell commands with `rtk`, following the referenced RTK instructions.
  Use CodeGraph first for indexed-code navigation when `.codegraph/` exists.
  Do not spawn agents unless the user or an applicable instruction requests them.
- Use focused fixture tests, relevant failure/recovery cases and proportional
  checks. Use `python scripts/desktop.py build` / `launch`; serialize shared
  builds. Windows x64 is supported. Native/model fixtures and real-model
  reliability are separate claims.

## Git snapshot and squash

At handoff preparation: branch `master`, HEAD `142a5c7`, clean tracked working
tree, aligned with the locally recorded `origin/master`. Recheck before editing.
The workflow commit was added after the previous implementation session.

The user requested squashing unpublished history. The preceding 35 commits became
15 adjacent topic commits, with every group endpoint and the final content tree
verified identical. Pushed history was preserved; this session did not push.

| Current commit | Relevant delivered scope |
| --- | --- |
| `bb76ea1` | EH06 worktree creation, ownership, transfers, archive/restore |
| `bfae287` | EH07 inline review findings and selected repairs |
| `30b963e` | EH07 side conversations and edited-prompt branches |
| `a0f9de7` | EH08 history search and retained attention |
| `c2e4b4b` | EH09.1 complete packages, resources and capacity recovery |
| `d1c9c53` | EH09.2 remote transport/controls and internal OAuth protocol |
| `5adf662` | Glass UI feedback and shared design guidance |
| `142a5c7` | Current focused documentation workflow |

Some plans/evidence still name original pre-squash commits; these are historical
identities. Recovery branch `codex/before-unpushed-squash-20261025` retains the
original history. Local mapping/verification receipt:
`output/squash-unpushed-20261025/receipt.json`. The former OAuth protocol commit
`8c0b60a` is included in `d1c9c53`. Old acceptance records remain at
`5adf662:docs/ACCEPTANCE.md` via Git; avoid assuming their old anchors exist in
the current compact acceptance index.

## Implemented versus remaining

- EH00–EH08 and EH09.1 are complete within their declared qualification bounds.
- EH09.2 has bounded Streamable HTTP, modern discovery and legacy sessions,
  reviewed endpoint/tool binding, None/Bearer credentials, foreground connection
  controls, searchable discovery and a connected Windows fixture journey.
- OAuth **protocol only** is implemented: challenged/well-known resource
  discovery, OAuth/OIDC issuer metadata, issuer-bound public-client registration,
  S256 PKCE, one-use callback validation, token exchange and refresh. This does
  **not** expose working OAuth sign-in in Settings.
- Remaining EH09.2: native loopback listener/external browser, vault binding and
  serialized rotation/refresh, foreground sign-in/Stop/expired-login recovery,
  and connected changed-catalog/malformed-SSE recovery. Keep its checkbox open.
- EH09.3 hooks, EH09.4 child supervision, EH09.5 lifecycle integration and EH10
  integrated qualification remain open. EH09.5 specifically includes package
  retention for pinned scheduled snapshots and remote-session cleanup after
  failed setup/drop/Stop. Richer child supervision must not implicitly grant
  command/MCP capabilities.

## Next implementation brick

1. Inspect current code and choose a concrete native OAuth integration design.
   The existing protocol API deliberately owns neither browsers nor persistence.
   Locate any existing browser-opening abstraction before adding another one.
2. Own a temporary IP-literal loopback callback listener for each foreground
   attempt, on a dynamically assigned port. Bound request size/count and lifetime;
   validate exact callback path, state and issuer, including error callbacks.
   Stop, disposal, expiry and shutdown must release the listener and discard late
   results. Open authorization in an external browser under explicit Sign in.
3. Extend remote authentication additively, preserving local and None/Bearer
   stored shapes. Keep token values out of SQLite, history, provider context,
   callback/browser diagnostics and ordinary bridge responses. Use native-vault
   records bound to canonical folder, connection, exact endpoint, issuer,
   resource, client and credential purpose; provider/local keys are independent.
4. Design refresh ownership and persistence before wiring calls. Save successful
   rotation before activating it; serialize competing refresh/config changes.
   A cancelled/disconnected refresh may already have rotated at the server.
   Retain local evidence and offer fresh sign-in rather than automatically
   repeating refresh or any dispatched business tool. Failed saves retain usable
   configuration and explicit recovery. Do not claim distributed atomicity.
5. Add foreground sign-in/health/recovery controls inside existing Connections.
   Read `docs/UI.md` for this surface. Preserve drafts/reviews and existing
   None/Bearer/local flows. No fabricated hosted client metadata URL: none is
   currently published. Support issuer-bound supplied client IDs and compatible
   registration using the existing protocol choices.
6. Exercise public local authorization/MCP fixtures through normal bridge and
   rendered controls: sign-in, reopen, refresh/rotation, revoked login, Stop,
   stale issuer/config, locked vault and failed save/retry. Extend the connected
   model/tool journey with changed catalog and malformed/SSE output, preserving
   completed effects and refusing automatic replay. Live third-party login/model
   reliability remains distinct from fixture qualification.
7. Commit the completed brick after proportional checks. Close EH09.2 only when
   its required connected sign-in/recovery gates pass; then proceed in plan order.

## Code entry points

Use CodeGraph for symbols/source, following current repository instructions.

| Concern | Entry points |
| --- | --- |
| OAuth protocol | `crates/dolores-tools-mcp/src/remote/oauth/{mod,challenge,discovery,grant,tests}.rs` |
| Main APIs | `Operation::{discover_resource,discover_server,register_client,refresh}`, `PendingAuthorization::{new,authorization_url,redirect_uri,complete}`, `Tokens` |
| Transport and approved calls | `crates/dolores-tools-mcp/src/remote/{mod,approved,credentials}.rs` and `tests.rs` |
| Stored target contracts | `crates/dolores-core/src/mcp.rs`, `crates/dolores-core/src/mcp/remote.rs` |
| Bridge review/save flow | `crates/dolores-flutter-bridge/src/mcp.rs`, `src/mcp/remote.rs`, `src/mcp/remote/tests.rs`; `InspectRemoteMcp` routing in `src/lib.rs` |
| UI / approval | `apps/dolores_flutter/lib/mcp.dart`, `tool_activity.dart`; `test/mcp_test.dart` |
| Connected fixture | `apps/dolores_flutter/test/native_mcp_remote_ui_test.dart`, `scripts/qualify-native-ui.py --case mcp-remote` |

`Tokens` currently has no plaintext getter, Debug or Serialize implementation;
add persistence/transport integration inside the MCP credential boundary rather
than exposing secrets through foreground DTOs. `PendingAuthorization::complete`
consumes the attempt. Transport auth failures expose bounded untrusted challenges
for OAuth parsing, never direct display/model input. Existing review saving uses
`persist_mcp_review` for vault writes, configuration CAS and retired-key recovery.

Protocol bounds: 30 seconds per network phase, twelve requests, 32 KiB per JSON
document, 128 KiB cumulative responses; sign-in attempt five minutes, callback
8 KiB, tokens 4 KiB, scopes 2 KiB/32 names. HTTP authorization fixtures require
an HTTP loopback MCP target; external resources require HTTPS. Redirects/retries
are refused. These bounds do not grant extra tool/model budgets.

## Verification starting point

Prior OAuth protocol source passed 48 MCP tests, five bridge MCP cases,
MCP/bridge all-target Clippy with warnings denied, and the normal Windows build.
These are historical checks, not a fresh acceptance claim for future changes.
No OAuth browser/vault/UI journey has passed yet. Public None/Bearer native
journey receipt: `output/native-ui/eh09-remote-controls-20261025-3/mcp-remote-receipt.json`.

For the next change, start with focused MCP/bridge tests, then affected Flutter
tests/analysis and the matching native journey after a normal build. Consult
`CONTRIBUTING.md` for current commands. Machine-local SDK is under
`output/toolchains/flutter`; the previous successful build used generator
`Visual Studio 18 2026` and CMake under
`output/toolchains/cmake-4.4.3-windows-x86_64/bin/`. Recheck availability.
No owned preview, active test process or in-progress Git operation is being
handed over. Preserve real model configuration/history; tests need no credentials.

## Suggested first message in the new session

> Read AGENTS.md and docs/HANDOFF.md. Continue EH09.2 from native OAuth sign-in,
> vault integration and recovery, then finish the remaining everyday-harness
> batches in dependency order. Keep H21 deferred, preserve existing features and
> user data, follow the updated focused documentation workflow, and commit each
> completed brick/batch. Do not push unless I ask.
