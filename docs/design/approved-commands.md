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

One invocation has a 30-second deadline and an 8-KiB combined raw stdout/stderr capture cap; exceeding the cap stops the process. JSON output is further shortened if escaping would exceed the existing 16-KiB result budget, with explicit truncation and lossy-UTF-8 indicators. A read failure or incomplete 200-ms drain marks outputError and truncated rather than claiming complete output. Nonzero exits preserve their code and output. Configured model, tool-call and whole-agent limits still apply. Where review is required, no command runs before approval, on Deny or on Stop at approval.

Windows creates a hidden suspended process, binds it to a kill-on-close Job object, then resumes it. Failure to establish the job refuses execution. Only the three standard pipe handles are inherited. Job cleanup runs on completion, timeout, output limit, cancellation and dropped async invocations. Unix uses a new process group and kills that group on cleanup; deliberately detached or externally launched processes are outside that guarantee. Commands are not security-isolated on either platform. Platform checks and acceptance are recorded separately; Windows is the local validation target.

Windows implementation follows Microsoft's [process creation](https://learn.microsoft.com/en-us/windows/win32/procthread/creating-processes), [Job objects](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects), [process flags](https://learn.microsoft.com/en-us/windows/win32/procthread/process-creation-flags) and [explicit handle inheritance](https://learn.microsoft.com/en-us/windows/win32/procthread/inheritance) contracts. The plugin owns active process/pipe handles and bounded reader workers only during invocation; no service, shell sidecar or idle polling is added. Arbitrary command CPU/memory use is not capped and must not be included in claims about harness idle performance.
