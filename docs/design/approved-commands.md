# Approved commands — brick 3.8

`run_command {program,args}` runs an executable with literal arguments and the
saved working folder as its current directory. Programs are not restricted to a
fixed name list: use a name on PATH, an absolute executable path, or a path
relative to the working folder, including project-local tools. Windows names
with or without `.exe` work. Shells and interpreters may be selected explicitly;
shell expansion and Windows wrapper scripts are not invoked implicitly.
Program names/paths allow up to 8 KiB without control characters. Arguments are
required: at most 32, up to 8 KiB each and 16 KiB combined; complete JSON allows
32 KiB. `command_session` start uses the same resolution and validation.

Execution follows the chat's command access mode. Where review is required, the
approval shows the resolved executable's absolute path locally, the program and
each argument separately, and the working folder. Run once/Deny binds that exact
proposal; changing an argument or executable requires a fresh review. A directory
is not a sandbox. The command has user-account permissions, may access/change
files outside the working folder and use the network. Interpreter arguments and
project build hooks may execute project code. There is no elevation. Command file
effects are not covered by Changes, cannot be rolled back automatically and
remain after Stop or failed conversation saving. Output is shared with the
selected model and retained/exported with a completed reply.

Child environment is rebuilt from a small platform/runtime allowlist; arbitrary
inherited API-key/token variables are excluded, but executables can still read
user files and credentials with normal user permissions. Input is closed,
terminal prompts/colors are discouraged, and no visible console is created on
Windows. Relative PATH entries resolve against the working folder. Resolution
pins the selected executable's canonical path; metadata is checked again after
approval, including for project-local tools. This is not protection against
concurrent replacement of system executables.

Both command tools accept optional `inherit_env`, a list of variable names from
the Dolores process, such as `NAS_ROOT_PASSWORD` or `SSH_AUTH_SOCK`. Values are
captured in memory when preparing that invocation and passed only to its child;
tool arguments, command approval and records contain names only. A missing or
invalid requested name fails before launch. Set a variable before launching
Dolores; a child listing no credential variables only establishes that they were
filtered, not that the parent lacks them. Existing automatic literal-command
grants do not cover added environment access; Full access retains its existing
authority. An invoked program can read or print inherited values, so use trusted
helpers and never request credential output. Session stdin is reviewed and
recorded; it is not a protected password-entry channel.

Windows native process creation uses conventional drive/UNC spelling for the
process image and cwd, while canonical identities remain authoritative for
validation. Verbatim `\\?\` executable paths break .NET Framework PowerShell
initialization. Ordinary executables retain CRT argument quoting; explicitly
selected `cmd.exe /c` or `/k` preserves the following shell command string with
CMD quoting instead of inserting CRT backslash escapes. Shell syntax executes
only when the caller explicitly selects that shell. Filtered environments retain
standard local username/domain identity needed by Python/SSH runtimes, without
forwarding arbitrary credentials, tokens or compiler overrides.

One invocation defaults to a 30-second deadline and 8-KiB inline stdout/stderr capture. Filling capture never stops a command: pipe readers continue draining and spill complete output to Dolores-managed files outside the working folder. `outputLogs` identifies stdout/stderr files; session receipts use `localLog`. Memory previews remain bounded and UTF-8 safe. `outputRetained` identifies complete durable output despite a shortened inline capture; `logError` and `outputError` identify actual loss. Zero exit plus successful cleanup and retained output is not a failed check merely because its preview is short. A read failure or incomplete final drain remains explicit. Nonzero exits retain their code. Explicit task budgets and approval decisions still apply.

Normal app runs store these files under the profile's `command-logs/<run-id>/`
directory. Receipts contain absolute paths; native helpers without a profile use
Dolores-owned temporary storage. Formatted preview logs follow the same rule.
The command's working directory is unchanged. Failed log storage reports output
loss without falling back to repository files or stopping a productive command.
Local file tools retain their project scope; external log text can be inspected
with literal command arguments under the existing command permission policy.

Explicit one-shot limits remain 1–300 seconds. For longer builds or tests,
`command_session` accepts an exact reviewed `timeout_seconds` up to 21,600
(six hours), while retaining the existing 30-second default. It remains owned
by the active run, with the same process cleanup, bounded preview and stdin review.
Use `read` with its returned output cursor and `wait_ms` up to 30,000 to avoid
repeated immediate polling. Canceling a read leaves the command alive; Stop or
run completion cancels the owned process. Existing short-command automatic
grants are not widened by these session limits.

Windows creates a hidden suspended process, binds it to a kill-on-close Job object, then resumes it. Failure to establish the job refuses execution. Only the three standard pipe handles are inherited. Job cleanup runs on completion, timeout, cancellation and dropped async invocations. Unix uses a new process group and kills that group on cleanup; deliberately detached or externally launched processes are outside that guarantee. Commands are not security-isolated on either platform. Platform checks and acceptance are recorded separately; Windows is the local validation target.

Windows implementation follows Microsoft's [process creation](https://learn.microsoft.com/en-us/windows/win32/procthread/creating-processes), [Job objects](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects), [process flags](https://learn.microsoft.com/en-us/windows/win32/procthread/process-creation-flags) and [explicit handle inheritance](https://learn.microsoft.com/en-us/windows/win32/procthread/inheritance) contracts. The plugin owns active process/pipe handles and bounded reader workers only during invocation; no service, shell sidecar or idle polling is added. Arbitrary command CPU/memory use is not capped and must not be included in claims about harness idle performance.
