# Task feedback — brick 7.1

Saved replies offer local Worked / Needs work feedback with an optional 2-KiB
note. The assessment is explicitly separate from model claims and command
results. A paused task may receive feedback without losing its recovery action.
The reply's original model, request settings, memory/skill provenance, progress
and tool receipts remain the evidence; they are never rewritten by feedback.

Schema 18 adds one cascading feedback row per reply. Save compares the exact
reply content/metadata and the feedback revision inside an immediate transaction.
Wrong chat, deleted/changed reply, stale revision and invalid notes refuse without
partial writes. Clear keeps a revision tombstone to prevent stale re-creation.
Notes reject credential-like text and are never sent to providers or learning.
They appear explicitly in conversation JSON/Markdown exports. Deleting the chat
cascades feedback; original conversation context stays unchanged.

The compact InspectorFrame uses the existing theme and a fixed wrapping footer.
Failures keep the edited note. Pending Save locks edits, Close and duplicate
actions. Feedback is available in chat and paged trajectory, including legacy
replies whose model/settings are honestly unavailable. No model request, idle
timer, new dependency, retry or automatic skill/memory promotion is introduced.
