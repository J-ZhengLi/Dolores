# Bounded subagents

Brick 11.1 adds `delegate_tasks` to working chats. The parent can request one
batch of one or two children per explicit run segment. Each has a literal goal,
relative file/folder scope and read-only or writable access. Writable scopes
cannot overlap another child's scope, including case differences and root scope.
Read-only scopes may overlap. This ownership check applies within the batch;
the parent waits for the batch before resuming its own tools.

Children default to the parent's configured model and share its request settings, prepared system
instructions, original user request and task allowance. They do not receive the
full conversation or attachments. At most four model calls and four file tool
operations per child cumulatively across follow-ups, further restricted by the shared parent totals. One model
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

## Child supervision (EH09.4)

An initial task may name an optional `model` from the enabled models frozen at
parent admission, on the same configured connection. Missing model means the
parent model; an unknown model refuses before starting any child. Providers and
request settings remain frozen for the whole parent run. Scope, read-only mode
and the parent permission snapshot cannot be changed by a follow-up. Actual file
operations still recheck the live parent grant, expiry and revision.

After a batch returns, the parent can call `delegate_tasks` with `childId` and
`action: inspect`, `followUp` or `interrupt`. Only `followUp` accepts `message`
(1–512 UTF-8 bytes); only `inspect` accepts a context byte `offset`. A follow-up
waits for that child's new report before parent tools resume. At most two
follow-ups retain the child identity and cumulative four-model/four-file cap;
attempts, including failed/interrupted calls, are never refunded. Parent totals
and its reporting reserve also remain enforced. There is no automatic replay or
child lifetime beyond the parent run.

Context consists of the prepared child instructions, literal goal and bounded
original request. Follow-ups append a bounded, explicitly untrusted report/tool
checkpoint and the new instruction. This is not an exact provider transcript;
the individual tool and public commentary events remain in Run history. Context
inspection pages the saved preparation at UTF-8 boundaries, up to 8 KiB per page
and under 16 KiB encoded. Each preparation page and permission snapshot is saved
before model work, including when a later response is interrupted. Raw reasoning
and attachments are excluded. Approval call IDs include the follow-up generation
so fresh decisions cannot reuse a previous generation's IDs.

Expandable child cards provide context paging and interruption while the parent
is active. Their bridge route binds session, parent client ID and child ID;
stale/wrong owners fail without dispatch. Context read errors retain Refresh and
Close. An individual interruption drops that child's model/approval future and
returns an interrupted report while its sibling and parent can continue. Parent
Stop still reaches all children. Completed writes and observed tool receipts
remain inspectable. Reports always retain `verified: false`; accepting a claim
requires independent parent file/tool evidence. The latest explicit report can
resolve that child's old pause, without resolving a sibling's outstanding pause.

Goal/queue/context lifecycle integration, hook coverage for children and scheduled
package retention belong to EH09.5. This slice adds no child command/MCP access,
cross-provider routing, recursive delegation, persistent workers or remote control.
