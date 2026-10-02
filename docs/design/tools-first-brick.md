# Brick 3.1 — Approved text-file reads

Tools are opt-in in Flutter. The user chooses a folder for this launch. The built-in `read_text_file {path}` plugin reads only relative paths in that folder, with an Allow once/Deny decision for every read before contents are shared with the selected model. Ordinary chat keeps streaming. Tool mode uses standard non-streaming chat-completions function calls. No writes, shell execution, remembered approval or external plugin loading.

Core owns typed calls, plugin registration, approval and a bounded loop. The host owns the folder capability and run-bound single-use approval channel. Model text and file contents cannot grant permission. File results stay untrusted tool data. Stop/failure never save a partial conversation pair; successful replies save bounded tool records. Usage remains separate per model call rather than falsely presenting one request as the whole run.

Limits: four model calls, four tool calls, 4-KiB arguments, 16-KiB UTF-8 regular-file results, 128-KiB serialized model context, 128-KiB final answer, and at most 300 seconds for the whole run including approvals (or the smaller configured request timeout). Each request keeps its configured output token limit.

Use an opened directory capability through [cap-std](https://github.com/sunfishcode/cap-std/blob/main/README.md). Reject absolute/traversal/control-character paths and common credential/VCS files. Resolve/open within the capability again after approval, including symlinks. Files can change between approval and reading; approval grants one read of the named file rather than an earlier snapshot. Native plugins remain trusted code, not an OS sandbox.

Reuse the shared theme. The sidebar folder action enables/disables tools without moving the composer footer. Inline approvals name the relative file and chosen folder. Bounded expandable result cards appear live and in saved replies/trajectory. Failed-run records remain visible until another conversation/run is selected.

Verify allow/deny, stale/repeated approvals, Stop while waiting, exhaustion, malformed/unknown tools, traversal/symlink escape, binary/oversized files, file prompt injection, provider tool-message ordering, atomic storage/export and compact light/dark UI. Protocol reference: [function calling](https://developers.openai.com/api/docs/guides/function-calling).
