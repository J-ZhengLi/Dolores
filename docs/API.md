# Dolores API

Brick 1 exposes **local Tauri IPC commands**, not an HTTP server. The UI calls native commands; the native provider makes outbound model requests. No arbitrary shell/filesystem API is exposed. All failures reject the invocation with a user-facing string. Error strings exclude raw provider bodies, authorization headers and endpoint credentials.

See [the desktop contract](desktop/desktop-api.md) and [the machine-readable contract](openapi.json). The OpenAPI file has an empty `paths` object because there are no HTTP endpoints; `x-tauri-commands` describes the actual IPC surface.

Generation first sends an empty delta to acknowledge native run reservation, then text deltas. This lets the UI forward a Stop request made during preparation. Invocation completion, not the channel, determines whether the turn was saved.

Outgoing provider contract: `POST <baseUrl>/chat/completions`, optional `Authorization: Bearer <apiKey>`, JSON `{model, messages: [{role, content}], stream: true, max_tokens: 2048}`. Requires `text/event-stream`, reads `choices[0].delta.content`, detects provider `error`, output truncation, and incomplete streams. Only the text chat-completions subset is supported. No tool calls, images or Responses API support is claimed.
