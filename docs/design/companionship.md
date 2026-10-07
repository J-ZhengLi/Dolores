# In-app companionship — milestone 23

Companionship is Off by default. One switch enables occasional labeled Home
conversations without switching the selected chat/model or touching its draft.
Settings show short controls; longer guidance uses the shared hover help icon.
No desktop notifications, tools, scheduled task creation or publication is granted.

## Frozen initial policy

- Enabled model chosen separately from Home; suggest configured Qwen3.5-2B only
  when it is enabled. No configured choice means quiet, never model fallback.
- System IANA timezone initially; allowed hours 09:00–21:00, 2 attempts per local
  day, a minimum 3-hour gap. Zero cap means quiet. Hours/cap/model are adjustable.
- At most one candidate and one unread initiated conversation. A candidate expires
  after 15 minutes; restart drops unfinished generation without replay.
- Random first opportunity 5–30 minutes after enabling/returning; later opportunity
  after the minimum gap plus 0–60 minutes of jitter. Persist the opportunity,
  attempt day/count and last attempt; no catch-up flood after absence or clock jumps.
- Generate only while Dolores is foreground, the user was active within 5 minutes
  but has not interacted for 90 seconds, and ordinary work is not running/queued.
  Native presence and host ownership checks revalidate before delivery.
- One no-tool request, 256 output tokens, 20-second total generation deadline,
  16 KiB input, 2 KiB returned text. Attempt admission counts before the request;
  failures consume the same daily/cooldown budget and stay quiet.
- Retain 16 activity records and at most one pending generation. Separate usage
  stays in those records; unknown token totals remain unknown. No new idle process;
  a conditional 60-second eligibility timer performs no model polling.
- Not now dismisses the unread message and snoozes for 24 hours. Dismiss marks it
  seen. Fewer messages lowers the cap to 1; Mute turns Off and never re-enables it.

## Sources and tone

Rotate generic chat, vetted timeless fact, one current memory or open-work source.
Facts come from a small reviewed static corpus with primary references, never
unverified recent claims. Model output is a short invitation around supplied data,
not a source of truth. Memory topics require Memory On, the same project scope,
an exact retained source and a current non-forgotten record. Open-work reminders
also require that the record still represents unresolved work.

The initial open-work check additionally requires its source to be the latest
user turn; later conversation suppresses it instead of guessing completion.
The initial static fact is sunlight's approximate eight-minute journey to Earth,
verified against [NASA Earth facts](https://science.nasa.gov/earth/facts/).
Non-Windows OS presence is currently unavailable, so generation stays quiet there.

Revalidate policy revision, candidate token, presence/quiet/busy state and source
immediately before committing. Off/Forget/completion/deletion/stale state prevents
delivery even if generation finished. No old private transcript is copied into a
generic prompt. Keep source metadata inspectable beside the initiated message.

Messages must not claim human feelings/consciousness, invented shared experiences,
guilt, exclusivity or pressure to engage. A memory/open-work prompt quotes a bounded
source as evidence and asks rather than presuming. Generic messages never imply
knowledge of a private event. An unverified or malformed answer is discarded.

## Qualification

Deterministic clocks cover hours/DST/caps/cooldown/random opportunities, restart,
busy/absence, unread suppression, mute during generation, stale revisions, Forget
and completed work. UI cases cover narrow/wide light/dark, failed saves and calm
delivery without changing Home. A bounded configured weaker-model case verifies
actual generation separately from the clock fixtures. Technical correctness and
the user's subjective welcome/annoyance are separate acceptance results; no
Meta Muse parity is claimed. Closed-UI scheduling belongs to milestone 24.
