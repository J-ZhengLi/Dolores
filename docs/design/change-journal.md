# Local change journal and reviewed revert — brick 3.6

Working chats in the selected Flutter host record each approved file replacement independently of the reply. The header's Changes action shows records for that saved working folder, newest first, and offers a separate review and Revert once decision. Side chats have no Changes action. Another chat in the same project can inspect the same journal after the original chat is deleted.

## Plugin boundaries and persistence

The core defines optional change-storage methods and a host-bound `ChangeJournal` port. The filesystem plugin receives that port; it does not import SQLite or accept journal roots/session IDs from model arguments. Flutter explicitly registers journaled folder tools. Existing unjournaled `folder_tools` registration remains for alternate hosts/tests; alternative shells do not claim this feature. An unsupported store rejects intent before an edit can be published.

SQLite schema 7 adds `file_changes`, indexed by private canonical root and descending ID. Rows hold the relative target, originating session ID, creation time, before/after UTF-8 snapshots (each at most 16 KiB), status, and optional reverted-record ID. There is no conversation-owned foreign-key cascade. Deleted chats, failed final saves and app restarts leave these records available. Snapshots and absolute roots are local, unencrypted data; they are excluded from automatic model context and conversation exports. Tool result metadata may include the local change ID and receipt status. Explicitly requested file reads can still share that file's text under the existing approval flow.

The publisher stages and flushes the replacement, checks the target, and commits a pending intent with SQLite FULL synchronous durability before renaming the file. It rechecks bytes, ordinary permissions, direct path and cancellation after that database operation. If intent cannot be saved, no edit is published. A failed/cancelled publication attempts a `notApplied` receipt. Successful publication attempts an `applied` receipt. Revert completion changes its own status and the original's `reverted` status in one database transaction.

Filesystem publication and SQLite cannot share an atomic transaction. A crash or receipt failure can leave `pending` even when the bytes were changed. The result still reports `applied:true` after a successful rename and exposes `journalStatus:pending`; Changes labels it Needs check, never asserts that intent proves execution. Review may prepare a revert for a pending original only when the current file exactly matches its saved after snapshot. No startup reconciliation, automatic rollback or unreviewed recovery write occurs. If it matches neither snapshot, the user must resolve the external edit. Receipt failures may leave an already restored original pending/applied; mismatching current bytes still prevent another revert.

## Revert review

The bridge derives the folder from the saved session, verifies that the record belongs to it, and accepts only an original edit with pending/applied status. A local opaque plan reverses the snapshots and captures current ordinary permissions through the same directory capability and direct-path checks as editing. Its diff is bounded to 16 KiB; no file is modified at preview.

One unpredictable token binds that plan to the exact session, folder and change ID for five minutes. A new preview replaces it; Cancel, Delete, Start and Shutdown discard it. Apply consumes it once, including on failure, validates the session again, and rechecks current bytes/permissions before publishing through the shared staged replacement. The reverse write receives its own durable intent and receipt. A reversed original cannot be reversed again, and reversing a revert is outside this brick. Older edits to the same file may require newest-first reversal; a changed file is refused, never merged or overwritten by inference.

The existing optimistic filesystem checks are not an atomic compare-and-swap against other programs. Ordinary permissions are preserved, without universal ACL/xattr/timestamp/hard-link guarantees. Crashes can leave owned staging files; unknown files are never removed automatically. SQLite serialization does not make concurrent external filesystem writers transactional. Keep one shell per data directory as before.

## UI and resource bounds

Changes uses existing dialog, palette and font tokens, max 720×640 with 16-pixel margins, and literal selectable diffs with a 180-pixel scroll region. It holds one 20-record page and one selected detail/preview; Older replaces the page, Newest refreshes it. Review revert and Revert once are separate actions, Cancel writes nothing, and dismissal is blocked only while a local operation is pending. The rest of the chat is locked while inspection is open. No model request, background indexing, idle polling, extra worker, new dependency or automatic journal purge is added. Disk retention grows with approved edits; bounded snapshot bytes do not imply bounded total disk use.

Existing pre-schema-7 edits have no backfilled journal. Their successful conversation diff remains available in trajectory, but snapshots are not invented and Revert is not offered for them.
