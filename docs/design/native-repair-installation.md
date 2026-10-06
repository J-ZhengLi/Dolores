# Reviewed native installation — 20.5 Windows Rust path

2026-10-06. The separate reviewed build, installation and Restore path is
implemented; qualification evidence and remaining user/platform gates are in
[acceptance](../ACCEPTANCE.md). The [20.4 evaluator](harness-self-repair.md) grants
no installation authority. Wasm remains deferred.

## First supported scope

Start with Windows Rust provider implementation and core command-outcome/task-budget repairs, preserving the
normal Flutter executable, assets and bridge API. Build a replacement Rust bridge
DLL from the exact qualified source in a separate owned folder. Copy the existing
complete normal application bundle into a versioned candidate directory and
replace that directory's DLL only. Never overwrite a loaded DLL or write into the
user's project. Flutter/UI, dependency, build-script, schema, credential, approval,
evaluator and launcher changes remain outside this first installation path.

This avoids requiring Flutter SDK setup for each core/provider repair. Installed
Cargo/MSVC and cached locked dependencies are still necessary. Missing tools,
offline dependencies or bounded build exhaustion retain the proposal and logs;
they do not trigger downloads, retries, weakened tests or installation.

## Build and installation are separate reviews

Build review identifies the repair revision, candidate/source/criteria hashes,
successful baseline-failure/candidate/regression receipt, exact installed Cargo
identity, fixed release build command and explicit time/output bounds. Recheck
source and tool identities after build and hash every retained bundle member.
Only the qualified Rust implementation diff may differ from the matching bundle.
Compilation cannot turn an unqualified or stale trial into a ready repair.

The fixed command is `cargo build --offline --locked --release -p
dolores-flutter-bridge --target-dir <host-owned-directory>`, supervised with the existing 300-second/256-KiB command
ceilings. A timeout remains an incomplete build with retained logs; trace a real
bounded-build failure before considering any change to the ceiling. Pin the same
Cargo executable used by the qualifying trial. Reproduction files stay out of the
release source manifest, whose parent/candidate identities must be recorded
explicitly rather than treating a Git-less staged build as the original commit.
Windows qualification reproduced a linker path-length failure in deep profiles.
Compiler outputs use a separate compact `nt/<intent-prefix>/<phase>` directory
inside the same private profile; its full owner ID and exact command are retained.
An existing compiler intent is refused, never silently reused. Sources, logs and
receipts remain in their versioned repair directories. Oversized profile paths
are refused before execution with a shorter-location recovery explanation.
The evaluator's matching source includes its existing Node MCP test helper;
omitting that helper produced regression failures and correctly withheld
qualification during Windows verification.

Installation review identifies the exact ready bundle, current installed bundle,
profile, restart effect and rollback plan. Explain that native code has the
account's permissions. Task Full access cannot grant build or installation.
Reject busy/in-flight work rather than cancelling or replaying it. Save drafts and
completed work before handing off; resume only through the existing explicit task
continuation. Keep the original selected provider/model and credentials local.

## External launcher and durable recovery

Use a bundled, lazy external native launcher so installation does not depend on
Python or on a DLL that must unload itself. Keep its protocol and validation
outside the editable repair scope. No background updater, public download,
signing, elevated service or platform CI is introduced.

The launcher records intent durably, validates process executable/creation time,
waits for acknowledged safe shutdown and refuses an unrelated or changed process.
It retains the previous complete bundle and a consistent local profile snapshot,
starts the candidate as the normal app, and waits for a bounded startup receipt.
Use 30 seconds for startup readiness and 30 seconds for graceful shutdown;
failure retains evidence and does not grant force-termination of busy work.
That receipt must cover bridge loading, expected schema/source identity and usable
idle chat initialization; window presence alone is insufficient. No startup model
request or original-task replay occurs. Schema changes are prohibited in this
first path; credential storage is not copied into candidate source/build outputs.

After healthy startup, expose Applied and Restore with the precise retained
previous version. Restore is separately reviewed and uses the same safe boundary.
On failed startup, stop only the verified candidate process, recover the previous
bundle/profile and report the failure. On launcher/app interruption, reconcile the
durable intent and observed process/version before any action; never blindly
rerun installation or resume a task. Keep failed candidate evidence for inspection.
The child is created suspended, its executable/creation identity is saved, then
it resumes. Candidate bridge commands and composer writes remain blocked until
idle chat initialization, exact source/schema and logical history identity pass.
The launcher cannot reuse a receipt that already records a helper. A fresh direct
user review may supersede an interrupted intent only after its helper/child have
exited and the current normal bundle matches a retained version.
The launcher re-reads the unchanged intent after acquiring its exclusive lock,
then durably records ownership before proceeding.

Profile recovery must not overwrite work created after a healthy installation.
Treat startup failure before handoff completion separately from later manual
Restore: the latter restores application code while preserving current history.
If identity/data compatibility cannot be proved, retain both versions and require
explicit recovery instead of guessing which profile to restore.

The repaired app runs from its retained versioned directory. This first path
does not replace the maintained bundle or retarget an existing desktop shortcut.
Ordinary maintained launch still opens that maintained version; continued use
after quitting requires opening the retained normal executable. Persistent
shortcut routing and a public updater remain later qualification/design work.

## Acceptance before a native installation claim

Exercise one real qualified Rust repair through build, separately reviewed
installation, healthy normal startup and reviewed Restore. Inject a failed startup
and interrupt the launcher between intent/shutdown/start, proving reconciliation
preserves drafts/history and does not replay work. Reject stale source/revision,
changed bundle/helper, a busy task and an unrelated process. Record compiler,
build/startup bounds, added disk/process cost, and evidence separately from model
reliability in `docs/ACCEPTANCE.md`.

Normal maintained desktop build/launch and original-profile preservation remain
required for implementation. Other OS, Flutter replacement, migrations,
containment and unattended native updates remain explicit later work.
