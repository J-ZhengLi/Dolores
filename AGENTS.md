# Dolores project guidance

For each implemented brick, verify the basic flow and add one or two realistic edge-case tests when applicable. Choose failures that users can encounter: exhausted output/context/tool budgets, interruption, stale configuration, unavailable tools, or malformed provider responses. Verify the recovery UX as well as the failure: preserve usable work, explain the actual limit, and provide an actionable next step. Document what was exercised and any remaining acceptance gaps in `docs/ACCEPTANCE.md`.

When a limit causes real tasks to fail, trace its effect before changing defaults. Prefer bounded, explicit continuation and useful recovery over silent retries or unbounded execution. Keep changes scoped to the current brick and commit each completed brick.

Write commit subjects and bodies in English.

When a build or launch is needed, use `python scripts/desktop.py build` and `python scripts/desktop.py launch`; do not use PowerShell helpers. Keep platform CI (8.4) deferred until the user explicitly requests it.

Choose verification that matches the change. Do not build, launch or visually inspect the desktop after every task. Documentation-only work needs document/link checks; runtime changes need focused tests and a normal build when compilation or packaging is affected. For UX changes, prefer saved renders/screenshots inspected with `view_image`; use the computer-use skill only when an interaction or native behavior needs it. Launch for a relevant integration check or when the user asks to see the app. Preserve provider configuration/history and replace only an owned preview process. Diagnostic evidence must be labeled; it does not establish normal-app behavior.

Use the configured provider's Qwen3.5-2B model for routine live tests and DeepSeek V4.1 Flash for harder live cases. Keep prompts and token usage bounded, preserve the user's selected model/settings, and keep credentials and private transcripts out of test artifacts. Use deterministic fixtures for failure injection; a fixture pass alone does not establish real-model reliability.

Use `rtk` for shell commands. If `.codegraph/` exists, use CodeGraph before searching source to understand or locate code. Follow `docs/UI.md` when changing desktop UI.
