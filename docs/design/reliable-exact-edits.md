# Reliable exact edits — brick 6.8

The 6.7 live repair spent multiple steps recovering from multiline proposals
whose LF line breaks did not match CRLF source. A minimized real-plugin test
reproduced Exact edit text was not found before this change and passes after it.
Whitespace/content mismatches and repeated targets remain separate failures.

## Matching and formatting

The existing edit_text_file argument shape remains path, old_text and new_text.
The source snapshot determines whether all actual line breaks use LF or CRLF.
For a uniform source, convert only LF/CRLF line breaks in the old/new proposal
to that style, then require one exact, nonempty, nonoverlapping occurrence.
Spaces, tabs, indentation, case and other characters remain literal. Replace
only that span; never normalize the whole file. Inserted/replacement lines use
the same style, with no added terminal newline or BOM.

If converted old/new text is identical, refuse the no-op. Check file/diff byte
limits after conversion, since CRLF can increase the proposal's size. Existing
64-KiB argument, 16-KiB file/diff and direct-path restrictions remain.

Files mixing LF/CRLF or containing lone CR do not permit inferred adaptation.
Byte-exact proposals keep their previous semantics. When a missing multiline
match needs adaptation, return allowlisted guidance to use an exact single-line
match or copy the original endings. A subsequent proposal requires its own
preview and approval. Proposals containing lone CR also keep byte-exact
semantics. A source with no line breaks supplies no style to infer; its new text
stays literal. This operation does not offer whole-file newline conversion.

## Review and concurrency

Preparation computes the actual after-bytes before producing the existing local
diff. No adaptation grants permission, writes a file, reuses an approval or
changes model limits. The prepared plan, journal snapshots and revert continue
to use raw before/after bytes. Publication still checks the raw current snapshot,
path and permissions before staging and publishing; a change affecting only
line endings invalidates the review. Stop, Deny, single-use binding and failed
publication cleanup are unchanged.

This retains optimistic concurrency, not an OS compare-and-swap guarantee
against every external writer. Revert bypasses proposal adaptation and restores
the exact saved snapshot after separate review.

## Evidence and limits

The regression and its matrix exercise both directions, uniform proposal
variants, insertion/deletion, repeated/overlapping targets, no-ops, different
indentation, expansion beyond the byte cap, Unicode/BOM, missing final newline,
mixed/lone-CR refusal, Stop and a raw newline-only external change.
The Windows FFI/HTTP/SQLite fixture validates a real module after adapted review,
checks mixed-file single-line recovery and restores exact original bytes after
restart/reviewed revert. The live DeepSeek task passes nine unchanged checks
with preserved CRLF/comments in one run; Qwen produces incorrect code and remains
unfinished. The 6.8 prompt differs from 6.7, so these runs do not establish a
controlled model improvement score.

No dependency, schema migration, resident worker, fuzzy matching, hidden retry
or budget increase is added. General coding competence, representative low-end
measurements, native input UAT and macOS/Linux runtime acceptance remain open.
