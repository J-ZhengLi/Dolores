# Scoped settings and interaction

Brick 9.4 adds effective user/project/chat settings to primary chat runs. Existing model connection, reasoning adapters and request defaults remain the starting point. This is a host settings contract; it does not install executable mods or establish consistent model personality.

## Resolution and ownership

The base request uses the selected endpoint/model profile, or the saved user generation default when no profile exists. Apply user interaction, then project overrides, then chat overrides. Generation overrides contain output tokens and request timeout together; interaction overrides contain explanation style and assumption checking together. Omitted groups inherit. Reasoning options and context windows remain model configuration; blank context still means 128K (131072 tokens). Output/context compatibility is checked before saving or starting.

User generation defaults stay in Request settings rather than a second competing editor. Project records bind the host's saved project root and apply to its chats; temporary/side chats have no project layer. Chat records bind a saved session and cascade on deletion. The local inspector returns scope names/revisions without private roots. Settings are local nonsecret SQLite data; interaction instructions and effective allowances are used in provider requests.

`scopedSettings` reads effective values, origins and available records locally, including during execution. `saveScopedSettings` requires the captured revision and a strict typed patch; stale, invalid or failed writes retain previous records. Changes are excluded during active work. The settings inspector preserves edited values after refusal; Refresh updates the revision without silently resubmitting the draft. Use inherited settings explicitly clears that scope's overrides.

A smaller selected model window can invalidate an old output reservation. Inspection still returns the saved values, actual window and validation error so the user can reduce/reset the override; sending refuses before a provider request or run record. A parent save is checked without a masking child override. Request settings explains that project/chat overrides take precedence, so changing a lower-priority default does not silently promise to repair the active limit.

Schema 21 adds `scoped_settings` and a chat-delete trigger. Existing tables/history remain unchanged. A primary run freezes effective values, their scope revisions, model reasoning origin and context origin before starting; Run history exposes saved origins. Context preview uses the same resolution and behavior preparation as send. Provider construction uses those effective values in the actual HTTP request, not just in a displayed snapshot. Auxiliary memory/skill/comparison jobs retain their separately bounded settings and existing policies.

## Behavior and authority

The default is to discuss reasoning before substantial work and check important assumptions. Brief explanations remain available. Host instructions ask Dolores to inspect cheaply answerable facts, respectfully correct consequential false premises, proceed once direction is agreed, and explain actual faults while preserving useful work. Advertised file tools resolve supplied relative paths within the working folder; missing absolute host paths are not a reason to ask for a location already supplied.

Calm, kind, candid behavior is expressed through evidence and useful recovery, without flattery, false success or claims of consciousness. These are model instructions, not a mandatory pause protocol or a personality guarantee. Scoped answers currently use the existing explicit preference rules; richer facts/freshness and correction are scheduled in 12.1.

The shared base context now refers to supplied tools/knowledge instead of asserting that every request has no tools or learned memories. Side chats still advertise no file tools; working chats and retrieved preferences no longer contradict that base premise.

Task operations still require review. Settings, skills and tool text cannot confer permission. The inspector separately states that executable self-update activation is unavailable and automatic preferences retain their own policy. Auto approval/full access arrive in 10.2; qualifying skill adaptation and executable activation retain the independent milestones 12/13 gates. No inactive permission or self-update switches are offered here.

## Verification boundary

[Acceptance](../ACCEPTANCE.md) records scoped precedence, stale/failed storage recovery, compact UI and actual native HTTP payload checks. `scripts/test-scoped-settings.py` uses the normal Windows bridge, an isolated child data directory and a local synthetic HTTP server, then releases the owned lock before cleanup. It verifies chat → project → model-profile inheritance, frozen historical settings, stale-save refusal and a disconnected start without a newly created chat. It is not a language model evaluation.

Bounded Qwen and DeepSeek false-premise probes are separate observations. Literal markers alone do not establish correct reasoning or kindness; synthetic replies are reviewed for their evidence and remaining limitations. Physical input/accessibility, other OS execution, representative resources and consistent behavior across real tasks remain open.
