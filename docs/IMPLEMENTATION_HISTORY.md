# Historical records

Routine development evidence lives in Git/CI and ignored local outputs. This
index is for explicit historical lookup; it is not a startup reading list.
Current limitations are in [ACCEPTANCE.md](ACCEPTANCE.md), priorities in
[ROADMAP.md](ROADMAP.md), and feature contracts in the [documentation index](README.md).

The cleanup baseline is Git commit `5adf66221b2b3451186cfd29f38f9bd4cf00033d`. Its original records are
preserved byte-for-byte in Git; no history rewrite was performed. A shallow clone
may need that commit fetched before these optional commands work.

## Acceptance records

The baseline `docs/ACCEPTANCE.md` contains the former dated evidence ledger,
including exact cases, counts, failures and local receipt paths. Search it with
bounded output when an old claim matters:

```powershell
git show 5adf66221b2b3451186cfd29f38f9bd4cf00033d:docs/ACCEPTANCE.md | rg -n -m 1 -A 20 'EH09 remote MCP OAuth'
```

Old acceptance-section links route here. Their link labels and surrounding text
identify the feature/record; search the original heading to recover its detail.
The current compact acceptance index is maintained only for material boundaries.

## Roadmap and delivery

The same baseline retains the complete former `docs/ROADMAP.md`,
`docs/IMPLEMENTATION_HISTORY.md`, `CONTRIBUTING.md`, superseded workspace planning
and the retired general reliability ledger. Use the exact file path with:

```powershell
git log --oneline -- docs/ROADMAP.md
git show 5adf66221b2b3451186cfd29f38f9bd4cf00033d:docs/ROADMAP.md | rg -n -m 1 -A 20 '^## Milestone 20'
```

Feature-specific qualification reports and completed batch checklists remain in
the checkout for targeted lookup. Their dated evidence retains its original
scope; rerunning the relevant check is required for a new qualification claim.
