# In-app companionship qualification — 2026-10-07

Milestone 23 delivers opt-in quiet eligibility, bounded generation, atomic
source-checked delivery, and concise settings/dismissal. Technical behavior and
subjective welcome are separate results; no Meta Muse parity is claimed.

## Exercised

- 296 core/store/bridge library tests pass; eight companionship cases additionally
  pass after final source cleanup. One explicitly invoked live-only test is ignored
  during ordinary tests. All-target Clippy and Flutter analysis pass.
- 332 Flutter tests pass. Narrow/wide light/dark settings, failed saves with retained
  edits, deliberate opening, Not now and failed feedback recovery are included.
  Saved renders were inspected; explanations stay behind help and Details.
- Packaged Off state performs 100 quiet checks without generation or state changes;
  median 0.0403 ms, p95 0.0480 ms. Stale saves preserve policy; a separate-process
  restart retains the choice/cap and creates no conversation. This measures quiet
  call latency, not sustained idle CPU or whole-process RSS.
- Atomic store fixtures cover duplicate finish, Off/restart during generation,
  publication rollback, Memory Off, Forget and later completed work. Forget removes
  companion source metadata/pending candidates in the same transaction; existing
  conversation history is retained. Deleting an unread note no longer blocks
  subsequent eligibility forever.

## Bounded live result

Two public prompts use configured Qwen/Qwen3.5-2B through the actual no-tool Rust
collector with 256 output tokens and a 20-second deadline. Generic chat is rejected;
the synthetic Cedar recollection invitation passes. Reported usage is 230 input,
57 output, 287 total tokens. No retries or fallback were used. This establishes
one live success and one miss, not reliable general generation or physical delivery.
OS presence/admission/publication are separate fixtures; the probe does not bypass
presence in the running app. Credentials stay in the child environment, not artifacts.

The static sunlight fact was checked against [NASA Earth facts](https://science.nasa.gov/earth/facts/).
Grounded wording is supplied by the host; the model supplies only a short invitation.

## Normal desktop and remaining gaps

Normal Windows main-entry build passes. Schema 37 adds the companion singleton;
all 45 pre-scheduler tables remain byte-for-byte equivalent in their row values.
No companion policy or task was enabled in the original profile. Earlier successful
build evidence precedes the last source/failed-save refinements; the final batch
build verifies those before handoff.

Physical foreground/last-input timing, interruption during real generation,
subjective usefulness/annoyance, sustained enabled idle CPU/RSS, and non-Windows
presence remain unaccepted. Non-Windows generation stays quiet. Open-work checks
are deliberately narrow: a later user turn suppresses the reminder. The model
cannot dispatch tools or tasks. Original provider settings/history are preserved.

Receipts remain under ignored output: `companionship-live.json`,
`companionship-packaged-2/qualification.json`, `companionship-renders/`, and
`companionship-normal-preservation.json`. Public reproductions:
`scripts/qualify-companionship{,-live}.py`.
