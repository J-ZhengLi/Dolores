# Editor document contract — milestone 16

16.0 freezes this initial envelope for the approved B layout. `re_editor 0.10.0`
and `re_highlight 0.0.3` are the selected editor candidates for 16.4; production
integration and regression checks remain required. No SDK migration is needed.

## Ownership and limits

Rust owns canonical scoped paths, saved-byte revisions and durable private recovery.
One Flutter document registry owns each unsaved buffer and its bounded undo history;
views own only selection, focus and scroll. Duplicate views never mount one widget
controller twice. Navigation, hiding a panel and disposing a view do not discard work.

| Item | Initial bound | Recovery |
| --- | --- | --- |
| Editable UTF-8 file, including BOM | 1 MiB | Larger files use a read-only preview |
| Editable physical line | 8 KiB UTF-8 | Very long lines use a read-only preview |
| Read-only preview | 64 KiB, 2,000 characters per displayed line | Explain truncation; original bytes stay untouched |
| Resident documents | 4, with 4 MiB total saved/edited text | Evict clean inactive buffers; dirty buffers require Save/Close before admitting more |
| File groups | 4 | Refuse another split, retaining the current layout |
| Undo per document | 64 records and 4 Mi UTF-16 units across before/after states | Drop oldest undo records with a visible history-limit indication; retain current edits |
| Delta insert | 64 KiB UTF-8 total, at most 16 replacements | Refuse an oversized edit atomically, retain buffer and explain how to split the paste |
| Serialized delta | 512 KiB | Reject before mutation |
| Explicit snapshot/save payload | 8 MiB serialized, with 1 MiB encoded-file bound | Reject without dropping dirty work |
| Pending document mutations | One acknowledged transaction per document | Coalesce subsequent edits within bounds; no silent replay after stale revision |
| Tree page | 200 entries | Explicit Load more; lazy expansion, no recursive startup scan |

The earlier proposed 5 MiB editable default is not adopted. The 1 MiB four-view
native trial passes 40 replacements at 19.558 ms typing frame p95 and shared
undo/redo. Short-phase incremental peaks are 63.957 MiB working and 59.996 MiB
private; earlier mixed corpus peaks remain recorded. The scratch incremental
envelopes remain 128/192/224 MiB for one/two/four views of one document. Four
distinct documents and normal host overhead are measured at 16.6; these numbers
are not an OS heap guarantee. Stop admission/retain dirty work if the implemented
registry exceeds its text/history bounds; do not solve pressure by discarding edits.

## Versions and messages

Every operation names a stable project ID, document ID and expected document version.
Scoped paths are resolved by the host against the project's canonical root, never
against the visible page or process CWD. A loaded snapshot includes a disk-byte
digest, encoding/BOM/newline metadata, normalized editor text and version zero.

An edit transaction contains ordered non-overlapping replacements expressed in
UTF-16 offsets into the same base text. Reject invalid ranges and offsets inside
surrogate pairs. Apply all replacements to a copy, validate byte/line/message
bounds, then publish exactly one version increment to all views. Stale versions,
malformed/oversized edits and changed project identities make no mutation. Preserve
the local draft and offer Refresh/Compare; never retry against a different base.
Only explicit open/recovery/save sends a whole snapshot; keystrokes do not send
whole-file JSON through the native bridge.

UTF-8 decoding is strict. Preserve the original BOM and uniform LF/CRLF convention;
retain exact original bytes for unchanged files. Invalid UTF-8, binary/NUL content,
mixed newline conventions and lone CR initially stay read-only. Encoding support
may be extended in a later reviewed revision, never by silent conversion.

Save is explicit and autosave starts Off. Compare the expected disk revision before
writing through the directory capability. A changed/deleted/renamed file retains
both usable versions and offers Compare/Reload disk/Keep edits or Save as. Keeping
edits needs a new reviewed base; it is not an overwrite bypass. Failed/interrupted
save keeps the buffer and private recovery. Save as checks the new target/revision
and requires replacement review when it exists. Agent tools read saved bytes only;
unsaved edits enter a model request only through deliberate attachment.

## Batch qualification

The user reports gaming during the old slow measurements and instructs proceeding.
Treat host contention as a plausible explanation, not a proven cause. Retain those
reports without treating them alone as an adoption blocker; 250 ms ordinary-open
and 32 ms typing targets remain unchanged. Production encoding/write/recovery checks
belong to 16.3–16.4. Native physical input/IME/accessibility, highlighting under load,
four distinct documents and integrated resource measurements remain explicit batch
qualification gaps until exercised; controller fixtures do not establish them.
