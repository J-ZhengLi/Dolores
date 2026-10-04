# Practical file and command work

`read_text_file` retains ordinary text responses up to 16 KiB. Larger UTF-8 files, bounded to 1 MiB, use `start_line` and `line_count` (1–120). The returned JSON exposes exact line coverage, omitted lines and a SHA-256 snapshot. A line or encoded range that cannot fit is refused with a smaller-range/local-editor recovery.

`edit_text_file` still replaces one unique exact occurrence and reviews a bounded diff. Files above 16 KiB require `expected_snapshot` from a ranged read; any supplied snapshot must match. The prepared full snapshot is rechecked before publication. Unicode and uniform LF/CRLF behavior is retained. Changes keeps local before/after snapshots up to 1 MiB, enabling reviewed conflict-aware revert. This is not an atomic multi-file transaction.

`run_command` accepts optional `timeout_seconds` (1–300) and `capture_bytes` (1024–262144). Defaults remain 30 seconds and 8192 bytes. The reviewed request exposes actual limits; task deadlines can stop a command earlier. A literal command automatic grant covers only limits within the defaults; larger limits require a new review unless full task access is enabled. No shell fallback or command rollback is added.

Large captured output is written to a uniquely created `dolores-command-log-*.txt` in the working folder. A bounded preview names this local artifact; it can be opened locally or read in ranges. Capture truncation, preview shortening, lossy decoding, process exit and log-write failure remain distinct. Exceeding the capture allowance stops the owned process. Logs can contain private command output and are user-managed working-folder files; exporting a conversation includes the preview/reference, not the complete local log. No resident service or automatic log upload occurs.
