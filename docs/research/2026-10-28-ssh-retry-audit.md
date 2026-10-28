# SSH retry investigation

Scope: the supplied conversation export ending in `50317a96-68c5-444b-aba4-f5a8f1738ccf`.
Only run metadata, SSH-related command receipts, and the final explanation were
inspected. Command argument bodies are omitted by this export, so intent cannot
always be reconstructed from a receipt. No live model or authenticated device
session was used for this investigation.

## What the latest run actually establishes

The latest run, `cf12196a-9b81-4c02-88b5-3bb124cb4789`, used build
`a510bb3`. Earlier runs used older builds and retain the previous Windows
username/PowerShell failures; those are not evidence of a regression in this run.

| Evidence in the latest run | Supported conclusion |
| --- | --- |
| Sequence 10: child reports `ENV NAMES ONLY: []` | No matching variables were visible in that filtered child. This does not establish absence in the app or Windows environment. |
| Sequence 26: connection established, `curve25519-sha256`, `ssh-ed25519`, then authentication failure | Transport and key exchange succeeded. KEX was not the remaining blocker for this attempt. |
| Sequence 26: a private-key identity is discovered, but no `Offering public key` line | This trace does not prove the server rejected that private key. The exact authentication options are absent from the export. |
| Sequence 31: `Permission denied (publickey,password,keyboard-interactive)` | This particular attempt did not authenticate; it does not prove no usable credential exists. |
| Sequence 42: direct Python import raises `ModuleNotFoundError: asyncssh` | The global interpreter import failed. This says nothing about the helper-managed pinned cache. |

The latest run read helper source and used direct SSH/Python probes, but its
command results contain no structured `host-key` result from the skill helper.
Its final recommendation to set a password before launching Dolores could not
work: command children excluded arbitrary credential variables and had no way to
request a selected one. This is a Dolores defect, separate from model reasoning.

## Targeted verification and correction

A presence-only check found `NAS_ROOT_PASSWORD` in the investigating process;
its value was neither printed nor recorded. This does not prove the already
running app inherited it, but demonstrates why a filtered-child check is
insufficient. A deterministic native fixture reproduces the loss with an invented
credential, without using the real password.

The installed skill has a working pinned AsyncSSH cache even though a plain
global import fails. Its actual `ssh_host_key.py` completed an identity-only
scan, reporting an RSA key and explicitly reporting no authentication or
`known_hosts` update. Its RSA fingerprint matches the older project reference.
The log's other fingerprint belongs to an ED25519 key. Different host-key types
normally have different fingerprints; this comparison alone does not establish a
changed or untrusted device. Neither scan is automatic trust approval.

## Implemented improvement

`run_command` and `command_session` now accept `inherit_env`, containing names
only. For the skill's password flow, an invocation can request
`["NAS_ROOT_PASSWORD"]` and pass `--password-env NAS_ROOT_PASSWORD` to the bundled
helper. An SSH-agent flow can request `SSH_AUTH_SOCK` where appropriate. Values
stay in the prepared invocation's memory and child environment, outside tool
arguments and approval records. Missing names fail before launch with an
actionable error. Existing literal-command automatic grants do not implicitly
grant the extra environment access; Full access continues to apply normally.

The tool schema explains filtering so an empty child listing is no longer the
only information available to the model. Default credential filtering remains.
Programs can print inherited values; trusted helpers must consume them without
echoing them. Session input and ordinary chat are not protected password channels.

## Remaining boundary

This fixes the missing credential transport, not all model decisions. The agent
must use the actual helper before declaring its fallback unavailable, compare
host keys by type, and distinguish failed authentication from absent credentials.
The investigation did not test password validity, authorize a public key, modify
NAS configuration, or establish a successful authenticated session. After
restarting the rebuilt app from an environment containing the selected variable,
the next skill invocation can use it through `inherit_env`.
