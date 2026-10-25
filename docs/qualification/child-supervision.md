# Child supervision qualification

EH09.4 qualifies the bounded Windows x64 host and rendered controls with
credential-free loopback fixtures. It does not establish live-model competence,
physical pointer/keyboard/accessibility behavior, cross-provider routing or
support on other platforms. The [subagent contract](../design/subagents.md)
owns behavior and limits.

## Accepted boundary

- The initial batch can select a child model from the parent's frozen enabled
  models. Unknown selections refuse without fallback or starting a child.
  Scope, read-only mode, request settings and inherited permissions cannot be
  rewritten through follow-ups. Child tools remain scoped file tools only.
- Explicit follow-ups retain identity and cumulative attempts. At most two
  follow-ups and four model calls/four file operations per child are enforced
  alongside parent totals and its reporting reserve. Exhaustion cannot reset an
  allowance; failures and interruption do not refund attempts.
- UTF-8 context paging exposes prepared instructions and bounded untrusted
  report/tool checkpoints. Preparation pages and permission snapshots remain
  in run events after the parent ends. They exclude reasoning and attachments;
  this is not an exact provider transcript.
- Individual interruption ends the selected child's model/approval future while
  preserving completed writes and sibling/parent progress. Parent Stop during a
  serialized child file approval cancels both children without approving it.
- The connected fixture creates a public file, interrupts that writer, follows
  up with its sibling under the same identity, and independently reads both
  files through parent tools. Reports retain `verified: false`; a report alone
  never establishes completion.
- Active inspection/interruption binds session, parent client ID and child ID.
  Wrong/stale owners refuse without changing the pending approval. Context read
  failure retains Refresh and Close, including Close while a read is pending.
- Compact Light/Dark renders keep inspector paging, Close and the composer
  reachable. The child area scrolls within a bounded height when reports and
  parent receipts coexist.

## Reproduction and limits

Build the normal bundle with `python scripts/desktop.py build`, then run
`python scripts/qualify-native-ui.py --case subagents`. Fresh profile, rendered
Light/Dark images and a public receipt are retained under ignored `output/`.
Rust subagent fixtures cover frozen selection, scope/tampering, cumulative
follow-ups, bounded reports/context pages, shared exhaustion and dropped models.
Core child tests cover recovery classification and latest-report ownership;
Flutter child tests cover read failure, paging, pending Close and stale status.

EH09.5 still owns goal/queue/context lifecycle integration, child hook coverage,
pinned scheduled package retention and connector cleanup. Live provider/model
reliability remains unqualified. No child command/MCP capability, recursive
delegation, persistent child worker or H21 remote control is admitted here.
