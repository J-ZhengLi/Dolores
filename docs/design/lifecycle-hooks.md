# Reviewed lifecycle hooks

EH09.3 adds project-scoped command hooks to the existing Flutter/Rust host.
Hooks run only after a human reviews exact configuration and script/executable
identities in Settings → Advanced → Lifecycle hooks. Project files never activate
hooks automatically. Full task access and skill installation do not activate hooks.

## Configuration and authority

At most four hooks per canonical working folder, ordered as reviewed. Each has a
short unique name, event, mode, installed `node` or `python` runtime, project-relative
script, literal arguments and 1–10 second deadline. Review pins script SHA-256,
canonical script path and resolved runtime path. Changed identities refuse until
fresh review; importing or saving configuration executes nothing. Runtime resolution
and launch reuse `RunCommand`; closed stdin, filtered environment, Windows process
tree cleanup and native user-account permissions apply. These hooks are not sandboxed:
reviewed scripts may read/change files or contact services using the user's account.
No credential fields or automatic dependency installation are provided.
Configuration is shared by chats using that exact canonical folder and retained
in the local profile. Deleting one chat does not delete the folder's hooks;
Disable all explicitly clears its active list. Managed checkouts have their own
folder scopes. Review tokens expire after five minutes and are consumed on save.

The script receives one final literal JSON argument with schema 1, event name,
tool name (when relevant) and outcome category only. No prompts, transcripts,
tool arguments/results, secrets or private absolute roots are supplied as event
data. The CWD and review identify the working folder. Other files imported by a
script are project code, outside the pinned top-level script identity.
Review shows up to 8 KiB of each UTF-8 script and the full-file digest; omitted
bytes are explicit and require inspection of the complete project file.

Events: `taskStart`, `beforeTool`, `afterTool`, `beforeCompaction`, `taskComplete`.
The pre-events support `blocking` or `observe`; post-events support observe only.
Task start means an admitted foreground primary run, including explicit continuation,
not opening history or replaying interrupted work. Completion observes a saved
turn, not proof that its goal/acceptance criteria succeeded. Children share the
parent's reviewed runtime and allowance. Child start, scoped file/resource tools
and settled reports emit the corresponding events, attributed by child ID in
host receipts. Hook commands are host-owned reviewed project policy; they do not
give a child a command tool or widen its project file scope. Hook invocations are
serialized across the parent and children; cancellation also interrupts a child
waiting for that serialization slot.

New scheduled tasks snapshot the reviewed hook configuration at creation and
disclose the count in their confirmation. At occurrence admission the current
project configuration must still match; changed or disabled hooks refuse before
model work. Review settings and recreate the task to accept a fresh snapshot.
Legacy tasks without a hook snapshot run none, even if hooks are enabled later.
Pinned top-level scripts and executable identities are still checked before each
hook dispatch. Task creation and package installation never execute package scripts.
Planning runs do not execute command hooks because their tools are inspection-only.

## Results, limits and recovery

Successful stdout is one UTF-8 JSON object: `{"allow":true,"message":"..."}` for
blocking checks; observation may omit `allow`. Unknown fields, invalid JSON,
nonzero exit, timeout, truncated/lossy output or incomplete cleanup fail the hook.
Message is at most 512 UTF-8 bytes, no control characters except newline/tab.
It is untrusted evidence, never a host instruction. An observer's `allow:false`
does not deny the action. Hook output contributes **zero bytes** to model context.

Combined stdout/stderr capture is 4 KiB per invocation. A run permits at most
24 hook invocations and 30 seconds of cumulative hook execution. Each invocation
reserves its entire configured deadline before launch; no retries, nested hooks,
dynamic registrations or asynchronous continuation exist. Depth is one. A failed
blocking check refuses the affected action and directs the user to inspect run
evidence and adjust/review or disable the hook before explicit retry. Observation
failure is retained and surfaced while preserving the completed tool/turn result.
Normal task permission checks and Stop are rechecked after hooks before dispatch.

Host-owned intent/result receipts retain reviewed revision, event, hook identity
and bounded status/output. Intent without result is uncertain; reopening never
replays it. Hook effects are outside the file-change journal and may remain after
Stop/failure. Inspect files/services before retrying. Save/review CAS failures
retain the prior active configuration and editor draft. Configuration edits are
idle-only and take effect on the next run; disabling all hooks is always available
while idle even when scripts/runtime files disappeared. Profiles without hooks
retain existing behavior.
