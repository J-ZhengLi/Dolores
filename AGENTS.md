# Dolores project guidance

For each implemented brick, verify the basic flow and add one or two realistic edge-case tests when applicable. Choose failures that users can encounter: exhausted output/context/tool budgets, interruption, stale configuration, unavailable tools, or malformed provider responses. Verify the recovery UX as well as the failure: preserve usable work, explain the actual limit, and provide an actionable next step. Document what was exercised and any remaining acceptance gaps in `docs/ACCEPTANCE.md`.

When a limit causes real tasks to fail, trace its effect before changing defaults. Prefer bounded, explicit continuation and useful recovery over silent retries or unbounded execution. Keep changes scoped to the current brick and commit each completed brick.

Use the configured provider's Qwen3.5-2B model for routine live tests and DeepSeek V4.1 Flash for harder live cases. Keep prompts and token usage bounded, preserve the user's selected model/settings, and keep credentials and private transcripts out of test artifacts. Use deterministic fixtures for failure injection; a fixture pass alone does not establish real-model reliability.

Use `rtk` for shell commands. If `.codegraph/` exists, use CodeGraph before searching source to understand or locate code. Follow `docs/UI.md` when changing desktop UI.
