# Everyday harness batch tasks

Implementation authorized by the user on 2026-10-21. Work through one batch at
a time and commit after each batch. Each file splits the umbrella into independently
verifiable tasks; a task checkmark requires evidence, not only code written.

[Umbrella design and invariants](../../design/everyday-harness-implementation-plan.md)
remain authoritative for scope. H01–H20 are included; H21 remote supervision stays
deferred. Historical roadmap milestone numbers are independent.

| Batch | Outcome | Owned gaps | Dependencies | Status | Commit |
| --- | --- | --- | --- | --- | --- |
| [EH00](eh00.md) | contracts, baseline, and feasibility | Shared prerequisites | None | Complete; later qualification gaps retained | `67449f4` |
| [EH01](eh01.md) | instruction queue and steering | H01, H02 | EH00 | Complete | `645805a` |
| [EH02](eh02.md) | command and skill discovery | H04 | EH01 | Complete | `0774115` |
| [EH03](eh03.md) | goals, checklist, and long-context recovery | H03, H07, H16 | EH01, EH02 | Complete | `59d8cc3` |
| [EH04](eh04.md) | plan mode and context references | H05, H06 | EH02, EH03 | Complete | `2631848` |
| [EH05](eh05.md) | agent command sessions and enforced isolation | H08, H17 | EH00, EH03 | Complete; Windows Node/MCP scope documented | `80c4640` |
| [EH06](eh06.md) | managed task worktrees | H09 | EH03, EH05 | Complete; bounded Windows lifecycle qualified | `42419c3` and EH06 delivery notes |
| [EH07](eh07.md) | review, side conversations, and edited prompts | H10, H11, H14 | EH04, EH06 | Complete; review, side chats and explicit shared/managed edited branches verified | See EH07 delivery notes |
| [EH08](eh08.md) | history and attention | H12, H13 | EH03, EH07 | Complete; search, organization and retained attention under declared Windows bounds | `b71fe68` and EH08 delivery notes |
| [EH09](eh09.md) | workflows, connectors, hooks, and subagents | H15, H18, H19, H20 | EH04, EH05, EH08 | Pending | — |
| [EH10](eh10.md) | integrated qualification and release readiness | Cross-feature acceptance | EH01–EH09 | Pending | — |

## Working policy

Before a batch, recheck Git status, relevant source, UI contracts and acceptance
gaps. Record exact files and design decisions in its task file. Keep task evidence
in ACCEPTANCE; raw local receipts belong under ignored output/. Use public fixture
inputs and isolated data. No credentials are needed for deterministic checks.

The commit unit is the batch, as requested. A batch may contain several small
tasks; do not combine a later batch into its commit or label incomplete acceptance
as complete. If an experiment rejects an approach, record the failed result and
revised design. Feasibility evidence in EH00 does not qualify H17 or replace EH05.
