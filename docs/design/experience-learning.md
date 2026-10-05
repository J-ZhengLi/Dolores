# Experience learning — milestone 12 contract

This batch adds project-scoped knowledge, fixed tool trials and conservative
skill adaptation in four separately verified bricks. It extends existing memory
and skill snapshots rather than changing task permissions or model settings.

12.1 stores at most 12 project facts, each at most 512 UTF-8 bytes, with source,
observed/inferred/manual status, revision, time and protection. Learning starts
disabled per working folder. Enabling it permits bounded reuse of approved tool
receipts; feedback-note sharing is a separate disabled-by-default choice. Facts
from files bind their digest and become stale when the direct source changes or
disappears. Command observations expire after seven days and are observations,
not authorization. Manual corrections are protected from automatic replacement.
The next-message context includes at most four facts and 4 KiB of enabled, current facts, quoted
as evidence; stale/disabled facts remain inspectable. No full transcript archive,
resident worker or vector index is introduced.

12.2 uses versioned host-owned fixtures and observable files/check outcomes.
Baseline/candidate receive identical models, settings and allowances; an original
case and independent regression cases are mandatory. Trial tools are restricted
to disposable fixture data. Network, credentials, browsers, MCP and arbitrary
executables are excluded. Simulated command receipts are labeled as such. This
boundary is application-enforced, not an OS sandbox. Larger executable trials
remain unavailable until milestone 13 containment is verified.

12.3 claims each relevant event once, attributes failures before proposing a
targeted skill change, and keeps separate bounded reflection/trial budgets.
Eligibility binds exact source/candidate/evaluator/policy revisions and complete
stored evidence. Automatic activation is limited to host-understood edits inside
the existing project envelope. Unknown instruction impact, global scope, new
authority/dependencies, limit evasion or changed criteria require review or
refusal. A test pass is necessary evidence, not a universal safety classifier.

12.4 exposes reasons, evidence, history, pause/disable and restore/quarantine.
Activation and rollback preserve baseline snapshots and use transactional
revision checks. Monitoring distinguishes matching workflow regressions from
unrelated transport/model failures. Interrupted or conflicting changes remain
explicitly unresolved; completed external effects cannot be rolled back.

The 12.2 fixture suite is `config-command-v1`: the original case enables one
JSON flag; the independent case adds Unicode and a nested flag/list that must
remain unchanged. Both require preserved companion/manifest files and a
successful **simulated** `node verify.cjs` check after the final write. The only
mutable resource is an in-memory `config.json` (2 KiB). All outside paths and
evaluator writes fail; attempted boundary violations prevent qualification.
Each phase receives a fresh world, the same configured model and reasoning
profile, 1024 output tokens per call, five model calls/eight operations and a
30-second whole-case deadline. Four sequential cases are the maximum. No OS
filesystem or process is available. A tie is not improvement. Twenty receipts
per chat are retained; failed saves/Stop/unfinished runs cannot qualify. Full
documents, files, tool receipts and reported usage are local evidence.

The 12.3 activation whitelist is `config-check-v1`: an exact host-created
`project-check` document with a single `node <basename>.cjs` command slot. Only
replacement by `node verify.cjs` qualifies. Approved package.json evidence must
declare that exact check, remain fresh and observed, and the saved failed task
must have used the exact skill version while enabling config.json. Changed
instructions, flags, paths, global skills and generic imported skills cannot
enter this automatic path. Create the optional workflow in Settings → Skills →
Learning; other skills retain Library review. Reflection is deterministic;
the provider runs fixed tool trials, not an open-ended JSON rewrite. Defaults
are off. Private feedback notes are not sent; separate eligibility can trigger
inspection of a locally saved outcome. A claimed event is never replayed;
unfinished evidence cannot activate. Automatic activation remains experimental
until live improvement and regression acceptance succeed.

Acceptance records real-model
results and unsupported paths separately. No general daily-improvement claim is
made from this small corpus.

12.4 keeps the original baseline independently of the five-version Library
window. Restore & quarantine appends a new rollback version and commits the
skill/history together. It checks the exact activated revision/document/enabled
state; a manual change yields a conflict and remains untouched. Storage failure
keeps both prior records, including after restart. Quarantine blocks subsequent
automatic activation of that exact candidate; deliberate Library review remains
available. It restores skill instructions only, never files or command effects.

Monitoring is event-driven, not a worker: one matching failed check after an
activation can use another four-case fixed comparison (same 20-call/32-tool/
120-second maximum). The task must include the exact active skill version and
literal failed check while enabling config.json. Denials, unrelated goals and
output/step limits do not initiate rollback. Restoration requires completed
independent baseline passes and an observed candidate failure, with unchanged
criteria; incomplete/tied checks retain the active version without retry. This
small comparison is a local qualification rule, not statistical certainty about
a model. Automatic restore needs enabled, unpaused automatic policy; manual
restore is available even when learning is disabled. Pending reflection or
monitoring is marked interrupted on startup without replay. If recovery storage
fails, ordinary chat remains available and unfinished state stays ineligible.
History includes receipts from all originating project chats; deleting a chat
removes its trial receipts while project snapshots/reasons remain. The release
gate stays unaccepted until actual model improvement and rollback are demonstrated.
