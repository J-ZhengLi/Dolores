# Working on Dolores

1. Inspect Git status and the affected code. Preserve unrelated work.
2. Read only the documentation needed for the task, using the routes below.
   Search headings/terms and read matching sections; expand when a dependency
   or contradiction requires it.
3. Implement the requested scope. Verify the basic flow and relevant realistic
   failure/recovery cases, preserving drafts/results and actionable errors.
4. Report changed behavior, checks and remaining gaps in the task response.
   Commit completed work in English; follow an explicitly requested batch boundary.

| Task | Read when needed |
| --- | --- |
| UI layout, controls, wording or motion | `docs/UI.md` |
| Build, launch or test commands | `CONTRIBUTING.md` |
| Packaging, CI or release | Relevant sections of `docs/RELEASING.md` |
| Feature behavior or invariants | Matching specification via `docs/README.md` |
| Planning or continuing a batch | `docs/ROADMAP.md`, then the selected batch |
| Acceptance/release claims | Matching qualification record via `docs/ACCEPTANCE.md` |

Update the owning document when behavior, a contract or an acceptance boundary
changes. Routine test runs belong in the task response/CI, with raw evidence in
ignored `output/`; they do not require a tracked diary or duplicate plan/spec edits.
Historical evidence is opt-in through `docs/IMPLEMENTATION_HISTORY.md`.

Choose checks proportional to the change: document checks for documentation,
focused tests for code, saved renders for appearance. Build when compilation or
packaging needs verification; launch for relevant native integration or a requested
preview. Use `python scripts/desktop.py build` / `launch`. Preserve user profiles;
replace only an owned preview. Windows x64 is the supported release target.

Automated tests use credential-free fixtures. Live checks need a suitable explicit
model, bounded usage and isolated data. Keep private transcripts/credentials out of
artifacts. Distinguish fixture, native and live-model evidence. Trace failing limits
before changing defaults; keep retries and continuation bounded.
