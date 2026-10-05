# Learning reliability follow-up — 2026-10-06

## Exact response preferences

Simple explicit response-style preferences now use a bounded literal extractor.
It stores the original wording, excluding a separate temporary acknowledgement.
Unknown vocabulary or clauses still use the reviewed model extraction path.
This does not establish reliable general-language extraction.

Two independent Qwen3.5-2B runs passed exact capture and an explicit correction.
Native fixtures verified scope, duplicate handling, protected manual corrections,
restart provenance, interruption, timeout, low context, malformed and denied
extraction. Failed learning preserves the ordinary reply. The local path makes
no extra provider request; source/revision/policy checks remain in storage.

## Approved evidence after a failed command

A real DeepSeek project task exposed missing knowledge after a `commandReview`
pause. The reply contained an approved complete package read and the exact failed
workflow command, but post-chat learning excluded every paused reply. Skill repair
therefore lacked the current declared check command.

The fix admits already approved receipt observations after command-review pauses.
It changes neither execution permission nor continuation. Output/step/desktop and
subagent pauses remain excluded. Existing source freshness, secret checks and
protected corrections apply; failed commands are not learned as verified successes.

The native bridge regression failed before the fix and passed after it. It also
verifies failed independent trials retain the baseline and reply, output exhaustion
does not learn observations, and a denied manifest cannot supply a repair fact.

## Bounded live qualification

The isolated `config-check-v1` task changes only one configuration flag. Unrelated
files and the check scripts are hashed and preserved. Only known file reads,
the exact configuration edit and two literal check invocations are approved.
The test injects neither saved knowledge nor fabricated trial receipts.

At 1024 output tokens, DeepSeek hit the response limit before changing the flag;
the original project remained. A separate 4096-token task changed the flag and
preserved every unrelated file, but exposed the missing-evidence issue above.
No app defaults were raised. The final 4096-token task retained the command-review
pause, learned the actual declaration and ran all four independent trials under
the existing fixed trial allowances and criteria.

Both baseline and candidate passed both trial cases. The result is a tie, so no
skill was activated. This qualifies the evidence/trial path and refusal to activate
an unproven improvement; it does **not** demonstrate useful automatic improvement
or real-model regression restoration. Those gates remain open. Existing deterministic
rollback, stale-policy and quarantine coverage is separate from live reliability.

Reproduce with a normal desktop build and fresh absolute ignored output directories:

```text
python scripts/test-learning-evidence.py --directory <absolute-output-directory>
python scripts/qualify-skill-learning.py --directory <fresh-absolute-output-directory> --model deepseek-v4.1-flash --max-output-tokens 4096
```

The live runner requires operator-supplied `DOLORES_TEST_BASE_URL` and
`DOLORES_TEST_API_KEY`. It returns failure when the useful-improvement gate is
unproven, including a tie. Local databases and transcripts stay private; public
results contain bounded synthetic outcome counters only.
