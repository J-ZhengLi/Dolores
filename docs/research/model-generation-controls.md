# Per-model generation controls

Verified against official documentation on 2026-10-03. These findings establish direct-provider contracts; they do not establish support in the configured OpenAI-compatible gateway. No credentials, private conversations, or live requests were used for this research.

## Official contracts

OpenAI Chat Completions uses top-level `reasoning_effort`. The documented values are `none`, `minimal`, `low`, `medium`, `high`, `xhigh`, and `max`; individual models support subsets. Lower effort can reduce latency and reasoning token usage. `max_completion_tokens` bounds both visible output and reasoning; `max_tokens` is deprecated and incompatible with o-series models. A user-facing output allowance must therefore not promise that many visible answer tokens. [OpenAI Chat Completions reference](https://developers.openai.com/api/reference/resources/chat/subresources/completions/methods/create)

DeepSeek Chat Completions accepts top-level `thinking` with an enabled or disabled type. Thinking defaults to enabled; effort defaults to high. Its native effort values are `none`, `low`, `high`, and `max`, where none disables thinking. Its output cap is `max_tokens`; a length finish reason indicates exhaustion. The currently documented model IDs are `deepseek-flash` and `deepseek-v4-pro`, so the configured `deepseek-v4.1-flash` alias alone establishes no contract. Required or named tool choice returns HTTP 400 in thinking mode. Tool arguments still require local validation. [DeepSeek Chat Completions reference](https://api-docs.deepseek.com/api/create-chat-completion/)

The OpenAI SDK expresses DeepSeek's thinking extension through `extra_body`; raw HTTP JSON contains the actual top-level `thinking` object, not an `extra_body` wrapper. For requests containing tools, DeepSeek requires complete prior `reasoning_content` replay in subsequent requests, including turns without a tool call; omission is documented to cause HTTP 400. Without tools, that field is ignored. [DeepSeek thinking guide](https://api-docs.deepseek.com/guides/thinking_mode/)

## Proposed explicit adapters

These are Dolores design choices, not automatically discovered capabilities. An adapter and output-cap field should be stored per model, with provider defaults as the migration-safe choice.

| Selected control | Extra request fields | Output-cap field |
| --- | --- | --- |
| Provider default | Omit `reasoning_effort` and `thinking` | Preserve the existing compatible cap field |
| OpenAI reasoning effort | `{"reasoning_effort":"low"}` or the explicitly selected effort | `max_completion_tokens` |
| DeepSeek thinking off | `{"thinking":{"type":"disabled"}}` | `max_tokens` |

Brick 6.6 exposes only `providerDefault`, `deepseekThinkingOff`, `openaiLow`, `openaiMedium`, and `openaiHigh`. Provider default retains the existing `max_tokens` field. The three OpenAI choices send their matching effort and use `max_completion_tokens`. DeepSeek off sends only its thinking toggle and uses `max_tokens`. DeepSeek thinking on is deliberately unavailable until complete reasoning replay is supported; its future wire form would be `{"thinking":{"type":"enabled"}}`.

If DeepSeek effort is exposed later, send a selected low, high, or max only with enabled thinking. Omit effort when disabling thinking to avoid contradictory settings. Do not send OpenAI `reasoning_effort: none` as a universal thinking-off switch. Do not send both output-cap fields.

The official documents do not establish what the gateway honors for either configured model, including `Qwen/Qwen3.5-2B`. Acceptance requires checking outbound fields and bounded live behavior separately. A successful response proves acceptance, not that an intermediary honored the control. Report reasoning usage when supplied; its absence does not prove thinking was disabled.

## Failure and compatibility boundaries

Unsupported-setting errors should retain the draft and completed tool work, identify the selected adapter, and offer a user-directed change to provider default or a supported setting. Do not silently remove controls, retry, switch models, or increase budgets. Fixture-test a rejected setting and subsequent explicit retry. Retain cancellation and partial-work behavior when a timeout or reasoning budget expires.

Enabling direct DeepSeek thinking with tools requires satisfying its reasoning replay contract within the harness's context and storage limits. If complete replay is unsupported, record that acceptance gap rather than claiming compatibility. An OpenAI-compatible endpoint and a models listing do not certify generation controls or full tool-loop compatibility.
