# Local file and image attachments

Brick 10.6 adds immutable user-selected snapshots without changing text-only history or installing an OCR service. The composer offers **Attach file**, local preview and removal. Sending an attachment-only draft uses “Please review the attached files.” Files can be shared in Side chat without granting file tools.

## Storage and context

Explicit image paste reads the native clipboard once. PNG/JPEG bytes reuse the
snapshot pipeline; Windows BMP data is converted locally to PNG after raw-byte
and dimension checks. Encoded snapshots retain the 2-MiB allowance. Conversion
does not add OCR or new provider formats. A unique temporary copy is removed
after the existing attach-file bridge acknowledges it; removal failure is shown.
The native clipboard adapter can also use a temporary bitmap during conversion.
Process interruption or OS failure can leave temporary files; cleanup is not
secure erasure. Native decoding can allocate before the Dart checks, so this is
not a complete defense against malicious or extremely large native clipboard data.

The chat is reserved while the clipboard read is pending, preventing a delayed
paste from attaching to another conversation. No image means normal text paste.
Errors retain the draft and earlier attachments. Thumbnails appear immediately
in pending user bubbles and use the same session-scoped saved snapshots after
completion and restart. Text attachment chips retain their existing behavior.

`Message` and stored message pages retain optional `parts`: lowercase SHA-256 digest, basename, MIME and byte count. Older messages deserialize with no parts. Schema 24 adds content-addressed assets, draft references, message references and endpoint-specific image model settings. Snapshots and metadata are local plaintext; original absolute paths are not retained in references or sent by this adapter.

Text must be UTF-8, contain no NUL and fit 64 KiB. PNG/JPEG snapshots fit 2 MiB and declare at most 4096 pixels per side / 4 megapixels. Header checks bound preview dimensions; they are not a full image decoder or malware scan. Empty/unsupported/binary files require a different file or explicit conversion adapter. No PDF parser, OCR, audio, video, remote asset hosting or automatic conversion is installed.

There are four distinct snapshots per draft and a 64-MiB logical asset allowance. Digests deduplicate stored bytes across chats; access still requires a reference in the requesting chat. Sends publish a completed user/assistant turn and move draft references transactionally. Failed sends retain text and attachments. Forks copy references, share immutable assets and preserve access when the parent is deleted. Assets are revalidated before sharing; a missing/damaged asset blocks the request and gives removal/reattachment recovery.

Reattaching bytes with the original verified digest repairs a missing/damaged stored asset, including references in sent history and forks. Different source bytes create a different snapshot and cannot repair the old digest. Combined fixed text must still fit the existing 128-KiB preparation allowance; use working-folder ranged reads for larger project files.

Prepared text contains the complete included text snapshots, explicitly labeled untrusted evidence. Existing text/token budgets still apply. Image bytes are hydrated only for included messages, deduplicated and bounded to 16 snapshots / 8 MiB per request. Images reserve an approximate 4096 tokens each; this is an application allowance, not a model-specific tokenizer or provider-reported usage. Local context preview shows references and the image allowance without making a model request.

## Explicit provider adapter

Image input is disabled by default. **Settings → Models → Supports image input** records an explicit choice for an enabled model at that endpoint. Unsupported images are refused before HTTP while preserving the draft. Provider image rejection also preserves the draft and offers a capable model or removal; it never retries or silently converts.

The OpenAI-compatible Chat Completions adapter keeps text-only `content` as a string and uses text plus `image_url` data URLs with `detail: low` for images. The wire format follows the [official image-input documentation](https://developers.openai.com/api/docs/guides/images-vision); other providers must support this format. The switch declares support, rather than proving that the endpoint/model accepts or understands every image. Vision comprehension and formatting reliability remain model-dependent.

## Sharing, export and retention

Preview explains that Send shares included snapshot content with the configured provider. Source-file changes after attachment do not alter the saved snapshot. Sent image references may be included in later context; deleting an original file does not retract a snapshot already shared.

Ordinary JSON/Markdown conversation exports retain references, not asset bytes or unsent drafts. **Chat actions → Export attachments** explicitly copies draft and sent snapshots into a new uniquely named folder, with digest-based filenames and a manifest. It refuses chats beyond 240 messages or 16 unique snapshots / 32 MiB before creating output, and never overwrites a prior export. A later filesystem failure reports an incomplete new folder to inspect.

Removing a draft reference or deleting a chat does not immediately erase shared/orphaned bytes. **Clean unused attachments** deletes assets with no remaining draft/message reference; referenced snapshots stay available. This reclaims logical allowance. SQLite free pages, backups and explicit export copies can retain bytes; secure erasure and physical database shrinking are not promised.

## Evidence

`scripts/test-attachments.py` exercises the normal release library across process restart: immutable Unicode text, compatible text wire, explicit image wire, disabled/rejected-image recovery, scope, count/size limits, export coverage refusal, damaged snapshots and safe orphan cleanup. Provider unit tests inspect actual content-part shape and private-body suppression. Compact light/dark widget checks exercise preview/removal after rejected Send and isolation after a failed chat switch. Real-model results and remaining acceptance belong in [acceptance](../ACCEPTANCE.md).
