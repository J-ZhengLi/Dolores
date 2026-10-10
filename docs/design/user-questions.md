# Structured questions in chat

Foreground human turns offer `request_user_input` alongside ordinary tools,
including side chats. The model asks only when available evidence cannot resolve
a clarification or preference. Questions and choices use the user's language;
ordinary reply text is never parsed into actionable choices.

One request contains 1–3 questions with unique IDs, each with 2–4 distinct choices
and optional short descriptions. Every question also has a custom-answer field.
Nothing is preselected or submitted automatically. Next/Back retain answers;
Send answer submits the complete batch and the same run continues. The card body
scrolls at compact heights while its action remains reachable.

The host validates the complete arguments within the existing 4 KiB tool bound.
Question text is limited to 512 UTF-8 bytes, labels to 128, descriptions to 256
and each answer to 1 KiB. Invalid submissions retain the pending question. The
host binds answers to a fresh run/question identity and consumes a valid reply
once. Wrong identities, repeated replies, tool-approval commands and cancelled
runs cannot answer a question. Waiting excludes the active task clock and sends
no model request. Stop/closed execution clears pending state and restores the
original task draft; saved progress remains. Submission failure preserves answers.

Question/answer events and completed tool receipts retain the actual choices
and answers in local run evidence and completed history. These are user intent,
not a permission grant. File/command/browser approvals continue through their
existing policy. Scheduled/background work and delegated children cannot open
interactive question cards. Interrupted questions require a fresh request after
restart; they are never silently replayed.

Context previews include the advertised question schema in token estimates.
The compiled registry owns this tool explicitly. Qualification belongs in
[ACCEPTANCE.md](../ACCEPTANCE.md); native input and real-model reliability remain
separate from deterministic fixtures.

After a normal Windows build, `python scripts/test-user-questions.py --directory
<absolute-path-under-output>` runs a credential-free loopback round trip. For an
authorized live check, add `--source <profile-db>` and `--model <enabled-model>`;
an optional `--reasoning` override is explicit. It copies configuration without
history into disposable data and checks the original profile remains unchanged.
The probe sends only public report/date prompts and bounds output and time.
