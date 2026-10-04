# Bounded subagents

Brick 11.1 adds `delegate_tasks` to working chats. The parent can request one
batch of one or two children per explicit run segment. Each has a literal goal,
relative file/folder scope and read-only or writable access. Writable scopes
cannot overlap another child's scope, including case differences and root scope.
Read-only scopes may overlap. This ownership check applies within the batch;
the parent waits for the batch before resuming its own tools.

Children share the parent's configured model, request settings, prepared system
instructions, original user request and task allowance. They do not receive the
full conversation or attachments. At most four model calls and four file tool
operations per child, further restricted by the shared parent totals. One model
call stays reserved for the parent report. Delegation itself spends a tool
operation. Defaults remain unchanged: small allowances may pause a batch.

Only scoped file tools are available to children. Their grants are the parent's
current grants intersected with child scope and read-only restrictions. In Review
mode, allowing the batch does not approve later file operations. Decisions are
serialized and identify the child. Full access still respects child scope.
Children cannot run commands, use MCP, inspect/update the harness, choose a new
provider, grant access or spawn more children. Parent commands remain available
for separate verification. This is a file-tool boundary, not an OS sandbox.

The parent owns child futures and cancellation. Stop, permission revocation and
the parent's elapsed deadline stop running and queued children. Applied writes
remain in Changes. Durable parent-linked events retain child identity, goal,
scope, state, model usage and tool evidence. Interrupted work is never replayed
automatically; inspect Run history/Changes and explicitly Continue. Reports are
bounded and labeled as reports, not proof that the requested goal was achieved.

Each child answer is at most 2 KiB of UTF-8; its encoded receipt is at most
6 KiB, and the batch result is at most 16 KiB. Shortening is explicit. Streamed
child commentary is coalesced into an at-most-8-KiB snapshot per model step,
rather than consuming a durable event for every text fragment. Tool evidence
previews are bounded separately; Changes retains full write snapshots. Stop
may discard an unfinished commentary snapshot, while applied tool receipts
remain. Children add no idle worker or background service.

Acceptance includes two independent scoped writes, shared budget exhaustion,
conflicting ownership, scope/grant escape attempts, malformed plans, interrupted
running/queued approvals, retained results after another child fails and bounded
reports. Fixtures establish host behavior; live tests separately check model use.
