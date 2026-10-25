# Remote MCP qualification

EH09.2 is qualified within the fixture and Windows x64 scopes below. The owning
[connection specification](../design/mcp-connection.md) defines behavior and bounds;
[EH09](../plans/everyday-harness/eh09.md) owns remaining lifecycle work.

| Evidence | Qualified boundary |
| --- | --- |
| MCP protocol and credential fixtures | Streamable HTTP JSON/SSE and legacy sessions; reviewed targets/catalogs; challenge and issuer discovery, S256 PKCE, registration and callback validation; no redirects or automatic business-call replay. |
| OAuth persistence fixtures | Bound vault records, fresh-owner reuse, per-credential refresh serialization, durable pending intent, saved rotation, cancellation after dispatch and failed-save recovery without repeating refresh. Changed folder/connection/target/purpose refuses credential reuse. |
| Bridge and SQLite fixtures | Sign-in produces review before saving; failed configuration/vault saves retain the review and previous usable configuration. Browser failure, wrong issuer, Stop and locked-vault recovery remain actionable. |
| Rendered Connections controls | Compact Light/Dark sign-in options, pending Stop, preserved drafts/reviews, failed Enable retry and explicit expired-login recovery. |
| Normal Windows build and connected journey | Public loopback authorization opens in the default external browser and reaches the owned callback. Actual native-vault storage supports saved-login reuse and refresh; revoked access recovers through fresh sign-in and reviewed Enable. None/Bearer/local behavior retains focused regression coverage. |
| Connected model/tool recovery | A public loopback provider requests an approved tool. Changed catalog refuses before invocation; malformed SSE after an effect retains the file and an error receipt without replay. Tokens remain absent from provider messages and saved connection metadata. |

The connected journey reopens Settings in the running app; fresh-owner durable
reuse is covered by storage/bridge fixtures. A complete process-restart/migration
journey remains part of EH10. Failed/locked storage is injected in deterministic
fixtures; native-vault evidence covers successful storage, reuse, rotation and
removal. Generated fixture credentials are removed on teardown, including failures.

These checks do not qualify real third-party accounts, arbitrary authorization
servers, real-model reliability, physical browser/keyboard/IME/accessibility,
clean-machine support or non-Windows sign-in. Supplied public client registration
and hosted metadata choices have protocol/foreground coverage; the connected
browser fixture uses compatible native dynamic registration. No hosted Dolores
client metadata document is published or assumed.

EH09.5 qualifies bounded legacy-session cleanup after failed setup/drop/Stop,
including joined primary-run receipts and uncertainty when termination is not
confirmed or supported. The [combined lifecycle fixture](harness-lifecycle.md)
covers goal/queue/scheduled/child ownership with a public legacy connector.
Abrupt process loss still cannot establish server-side cleanup. H21 remote task
supervision stays deferred. Raw receipts and renders remain under ignored output/;
routine reruns do not expand this qualification boundary.
