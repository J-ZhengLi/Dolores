# Dolores token consumption audit

Date: 2026-10-27. Scope: the supplied `ainas-monorepo` conversation export,
its matching local run archive, and the implementation responsible for those
requests. This is an investigation report; no product fix or live-model rerun
was performed.

**Finding:** the supplied run consumed **at least 326,321 provider-reported
tokens without delivering its requested plan**. Of those tokens, **97.9% were
input**. Ineffective repository discovery produced extra model turns, and
each later request carried the accumulated evidence again. A substantial fixed
instruction/tool catalogue and three failed final generation attempts amplified
the problem. The evidence supports a concrete efficiency problem in this run,
but does not establish a multiplier relative to another harness.

## Evidence and accounting

The supplied conversation asked Dolores to read issue I9V6 using the Gitee skill
and propose a fix plan, explicitly postponing implementation. It is therefore
evidence about code investigation/planning overhead, not a completed
implementation benchmark.

- Session: `ff2527d9-7b65-4894-b0aa-0d9987a99656`.
- Run: `266df0c3-99df-4a91-88da-fc1d03d4889b`.
- Recorded build: `9741a7619a3561fd06aad18af4a385f2de81924c`.
- Model: `kimi-k3.1-pro`; configured context: 500,000 tokens.
- Execution: 22:29:44–22:34:34, Asia/Shanghai, approximately 290 seconds.
- Outcome: paused at a generation-failure checkpoint; no completed plan.
- Export SHA-256:
  `9daf1ca80e7036dfd35e6f742320c49f91d99212b5ee4a7aeeee3ef64d64c322`.

The export contains the final five usage entries. A read-only SQLite transaction
recovered the preceding thirteen entries from contiguous `agentEvidence`
fragments. Fragment offsets/lengths, attempt counts, receipt counts and token
arithmetic were checked. Archived and retained tool receipts were deduplicated
by call ID.

| Measure | Observed value |
| --- | ---: |
| Model attempts | 18 |
| Attempts with reported usage | 15 |
| Attempts without reported usage | 3 |
| Reported input tokens | 319,561 |
| Reported output tokens | 6,760 |
| Known input + output | **326,321** |
| Cached input, already included in input | 86,016 |
| Input excluding reported cache hits | 233,545 |
| Reasoning, already included in output | 2,798 |
| Tool attempts | 45 |
| Invoked tool-result events | 42 |
| Blocked attempts | 3 |

Cached tokens and reasoning tokens must not be added again. Provider usage is
distinct from local token estimates. The three unsuccessful attempts may have
consumed additional tokens, but their usage is unavailable. Consequently,
326,321 is a lower bound, not a complete invoice or monetary cost estimate.

## 1. Built-in search is poorly suited to this repository

**Confirmed, and independently reproduced.** The run made ten `search_text`
calls. All returned zero matches; nine reported truncation. They collectively
read 2,112,928 bytes without locating the requested literals. After three model
turns of built-in searches and another turn of unavailable command attempts,
`git grep` returned relevant source locations.

The implementation limits a search to four directory levels, 256 entries,
64 attempted files and 256 KiB read. More seriously, it skips every file larger
than **16 KiB**, before searching its contents. It offers no continuation cursor
or include-pattern argument. See
[discovery implementation](../../crates/dolores-tools-fs/src/discovery.rs),
especially lines 10–14, 189–214 and 357–365.

The relevant `frontend/src/client.ts` was **68,821 bytes**, according to both
the saved ranged-read receipts and the current local file. It contains the exact
failure messages being sought. Built-in search cannot inspect that file even
when its parent folder is selected precisely. `git grep` found those messages
in the saved run; a current read-only check confirmed their presence.

A minimized fixture exercised the actual `dolores-tools-fs` search tool with one
file and one known literal:

| Only changed variable | Tool result |
| --- | --- |
| File size 16,384 bytes | 1 match, 1 scanned file |
| File size 16,385 bytes | 0 matches, 0 scanned files, 1 skipped file |

Both results reported `truncated: false`. The `skippedFiles` counter discloses
the omission, but the result does not explain that the file exceeded the size
limit. The probe's “known literal must be found” assertion passes below the
boundary and fails above it. This is a confirmed discovery capability gap;
it does not imply that every empty search means the text is absent.

**Why this costs tokens:** repository discovery uses more model round trips,
and every such turn pays for the existing instructions, schemas and accumulated
history. The search results themselves totaled only 1,466 bytes; their main
cost was the extra turns they induced. The model correctly pursued alternatives,
but reached the effective `git grep` path late.

## 2. Accumulated evidence is resent until capacity pressure, not cost pressure

**Confirmed mechanism and measured growth.** Provider-reported input rose from
**9,415 tokens on call 1 to 50,108 on call 15**, a 5.32× increase within this
single run. This is context growth, not a comparison with another harness.

| Call | Reported input tokens | Activity |
| --- | ---: | --- |
| 1 | 9,415 | Authentication status and CLI help |
| 5 | 12,658 | Fetch issue |
| 7 | 19,237 | Built-in searches |
| 11 | 21,791 | Successful `git grep` fallback |
| 13 | 32,962 | Source and directory inspection |
| 14 | 41,100 | More source inspection |
| 15 | 50,108 | More source/tests; final successful generation |

The 45 receipts contained **164,440 UTF-8 bytes**: 77,787 from file reads,
76,684 from commands, 8,503 from listings and 1,466 from searches. Three
CodeGraph command responses alone contributed **43,586 bytes**. Some subsequent
file reads revisited files already involved in CodeGraph queries; exact
cross-format duplication was not calculated. Ranged-read receipts do prove
21 overlapping lines in `worker.ts`; not all repeated reads were redundant.

The checkpoint after call 13 retained **105,624 bytes of tool-result content**
in the model's working context. Sealing the local evidence segment did not
compact that context. The source explicitly retains the working messages when
rolling UI/evidence segments; later tool results are appended to those messages.
See [agent loop](../../crates/dolores-core/src/agent.rs), lines 829–865 and
1654 onward, and [segment retention](../../crates/dolores-core/src/task_segments.rs).

Automatic compaction is triggered near 80% of the input allowance. This run's
configured estimated allowance was **442,232 tokens**, after a 25,000-token
headroom and 32,768-token output reservation. That reservation does not mean
32,768 tokens were generated or charged. No model compaction occurred in this
run. A 500K capacity permits substantial cumulative spend while remaining far
below the capacity threshold. See
[capacity measurement](../../crates/dolores-core/src/context_recovery.rs) and
[token settings contract](../design/token-context.md).

Full-history resending is a normal chat protocol pattern. The actionable
Dolores issue is retaining bulky and sometimes unproductive evidence across
many turns without a separate cost-oriented projection policy. This audit
does not prove that all retained evidence can safely be removed.

## 3. A large fixed catalogue is paid on every request

**Confirmed overhead; comparative excess is unmeasured.** The saved initial
context estimate was **11,399 tokens**, consisting of:

| Component | Local estimate |
| --- | ---: |
| System instructions | 4,875 |
| Tool definitions | 6,464 |
| User task | 49 |
| Framing | 11 |

There were no prior conversation turns or images. The saved system content was
19,499 bytes. The first provider-reported input was 9,415 tokens; that difference
demonstrates why the byte heuristic must not be treated as exact tokenization.

All **20 registered tools** were sent, including browser/computer control,
scheduling, delegation and harness self-repair tools. The run used only four
tool types: commands, file reads, search and listings. The loop constructs a
fixed `specs` list, and the provider serializes it on each request. See
[catalogue construction](../../crates/dolores-core/src/agent.rs), lines 651–660,
and [request serialization](../../crates/dolores-provider-openai/src/agent_stream.rs),
lines 309–310.

The three activated skills totaled only **3,503 bytes**. This particular run
does not support blaming a huge skill dump or old conversation history.
The broad tool catalogue is a better optimization target, provided tools
remain discoverable when the task needs them. Another harness may also have a
large prompt; this report has no matched measurement proving otherwise.

## 4. Discovery setup and interrupted generation add avoidable work

**Confirmed events; exact avoidable token savings are unknown.** Before fetching
the issue on call 5, Dolores requested three levels of CLI help, checked
authentication, inspected Git remotes and listed the project root. Calls 1–4
consumed **44,229 known tokens**. Some setup was useful; this number is not an
estimate that all 44,229 tokens could be eliminated. Better use of the activated
Gitee skill's command recipe could shorten this sequence.

Attempts 16–18 each ended after approximately **30.2 seconds** while reasoning
was still arriving. The saved error reports an unfinished provider response.
Two automatic retries followed delays of one and two seconds, then the task
paused. The configured inactivity timeout was 180 seconds. The evidence points
to interrupted generation, not an exhausted context window or a demonstrated
180-second client inactivity timeout. It cannot identify whether the upstream
model service, gateway or another transport component imposed the interruption.

No tools from those failed attempts ran. Their unknown usage must not be
counted as zero. Retries are bounded; this was not an infinite loop. Historical
retry behavior was checked against the recorded build, rather than inferred
from the unrelated retry changes currently being edited in the workspace.

## 5. Cache efficiency and export completeness need attention

**Cache hypothesis:** reported cached input was exactly **6,144 tokens** on
each successful call after the first. Aggregate cache coverage was **26.9%**
of reported input. The agent rewrites its first system message with the current
model-call/tool counts, and later adds capacity notices there. Changing this
early prefix can undermine prefix-cache reuse. See the agent's
`budget_note_for` and system-message update around lines 872–877.

That is a plausible amplifier, not a proven explanation for this provider's
cache behavior. Cache granularity, expiry and provider routing are unavailable.
A request-level comparison with stable instructions is needed before claiming
a particular cache-hit improvement. Reported cache hits reduce neither the
physical context length nor the need for accurate accounting.

**Confirmed export gap:** the JSON export's remaining usage records sum to
**93,348 tokens**. A further **232,973** known tokens exist only in the local
archive. The reconstructed known total is **3.50×** the export-only subtotal.
That ratio is an export-accounting gap, not a harness efficiency comparison.

[Segment sealing](../../crates/dolores-core/src/task_segments.rs) clears
`usage_by_call` after archiving. The
[export writer](../../crates/dolores-store-sqlite/src/history.rs) exports message
metadata without embedding those archived agent-evidence records. The archive
pointer survives, but the standalone JSON is insufficient to reconstruct this
run's cumulative usage. Reliable audits should not require the original profile.

## Recommended order of work

| Priority | Change | Validation required |
| --- | --- | --- |
| 1 | Give repository search a scalable path: installed index/`rg`/Git search where available, or streaming file scans with bounded results and continuation. Explain skipped coverage explicitly. | Find known literals in large files and deep monorepo paths; preserve access/exclusion rules and actionable partial results. |
| 2 | Reduce repeated evidence in the request projection: bounded previews, targeted source ranges and deduplication, while retaining full receipts locally. | Compare request sizes and task success on a fixed trace; preserve unresolved errors, constraints and tool-call/result relationships. |
| 3 | Advertise a smaller relevant tool catalogue, with discoverable access to additional tools. Keep stable instructions separate from changing counters. | Measure schema tokens and actual provider cache hits with identical tasks/model/settings; check tool availability and recovery. |
| 4 | Resolve the approximately 30-second streaming interruption and expose retry usage uncertainty. | Inspect provider/gateway evidence; verify complete streamed responses and bounded recovery without tool replay. |
| 5 | Include cumulative known usage, missing-usage counts and archived evidence in exports. | Round-trip a segmented run and reconstruct its exact known usage from the export alone. |

Do not use a smaller context window or arbitrary low step limit as the primary
repair. They would constrain consumption by interrupting work, without fixing
discovery or evidence handling. A separate configurable spend/compaction policy
may help after request-level measurements exist.

## Verification, reproducibility and limits

The read-only analyzer, sanitized metrics and minimized Rust probe are under
ignored `output/token-audit/`. No credentials, raw private transcript or source
capture were added to tracked documentation. The local profile and
`ainas-monorepo` files were not modified. Existing concurrent source/document
edits were preserved.

Run the evidence audit from the repository root:

```powershell
rtk proxy python output/token-audit/analyze.py
```

The probe invokes the real file-search implementation without a model:

```powershell
rtk proxy cargo run --manifest-path output/token-audit/probe/Cargo.toml --offline --quiet --target-dir target -- 16384
rtk proxy cargo run --manifest-path output/token-audit/probe/Cargo.toml --offline --quiet --target-dir target -- 16385
```

The first passes; the second intentionally fails the known-literal assertion,
reproducing the size boundary. The probe uses current source; the discovery
implementation was unchanged between the recorded build and the inspected tree.

There is no paired run from another harness, no complete usage for the three
failed generations, and no provider invoice/request-body capture. A defensible
cross-harness benchmark still needs the same repository snapshot, task, model,
endpoint, reasoning/output settings and completion criteria, repeated trials,
separate cold/warm cache results, all auxiliary calls and completed-plan or
implementation-quality checks. No numerical savings or comparative multiplier
is claimed before that comparison.
