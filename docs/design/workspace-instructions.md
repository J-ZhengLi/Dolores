# Reviewed workspace instructions — brick 3.9

Root review and activation remain explicit. Current task preparation reads the
root only; repository size, unrelated directory count and nested-file count do
not invalidate it. Before file-tool edits/creation, `read_scoped_instructions`
loads only applicable ancestor AGENTS.md files, including paths deeper than four
directories. It returns bounded pages until `hasMore` is false. The host keeps
those exact scope-labeled bytes through context compaction, and rechecks them
before publication. Changed or unread guidance defers a write with an actionable
read instruction. Legacy combined snapshots normalize to root-only when their
approved root bytes still match. Parent folders outside the chosen root and
linked instructions are not loaded. This supersedes older eager nested scans.

The working-folder header opens Instructions. Root review reads literal `AGENTS.md` with an 8 KiB UTF-8 allocation and shows exact selectable text. Enabling shares it with the model and saves a local snapshot. Enable requires an unexpired, single-use token and matching current bytes. Cancel/Close never enables anything. Nested guidance follows the path-scoped flow above. There is no whole-repository startup scan, watcher, `@` include expansion, remote fetch or execution.

An enabled snapshot belongs to the canonical working folder, so other chats in that folder and restarts reuse it. Side chats have no instructions. Disable removes the active snapshot without reading the file and without changing history. Each next-message preview/send validates the root file against the snapshot; changed, missing, unreadable, linked, binary or oversized files block that request with a manual re-review/disable action. A running request retains its initial snapshot. File operations still need their own approvals; guidance cannot change host policy, request limits or tool scope.

Use the existing directory capability to constrain reads, require a regular root file and refuse links. Includes and path text remain literal guidance, never host file access. Persist at most one bounded snapshot per folder in additive SQLite schema 10. Store source `AGENTS.md`, a random revision, acceptance time and text; never place the absolute folder in provider context, request logs or exports. User-approved file text can itself contain private information; its complete review precedes sharing. No automatic secret classification is claimed.

Append the snapshot to the existing system message after tool guidance, with explicit host/user precedence. Shared preview/send preparation counts the complete text in byte and token budgets and trims only old complete history pairs. Context inspection shows the source/revision and exact prepared system text. Completed replies retain the source/revision/time/byte count for provenance, without a duplicate snapshot in metadata. Legacy replies remain unchanged. Configuration saves are atomic; a failed save leaves the previous enabled snapshot intact.

Reuse InspectorFrame, semantic colors and bounded scroll areas. The header action is absent for Side and a temporary chat before its folder exists. Disable, Refresh, Enable instructions and Close are labeled actions; show Missing/Needs review/Enabled honestly. Lock chat switching/sending while the inspector is open, and prevent dismissal/duplicate decisions during saves. No composer controls or theme changes.

Verify isolated activation, stale-token/content refusal, permission boundaries, changed/deleted-file handling, disable recovery, folder isolation, atomic failure, restart/export provenance, preview/send equality, context budgeting and compact light/dark layouts. Native renderer/controller checks do not substitute for OS keyboard/IME/accessibility acceptance.
