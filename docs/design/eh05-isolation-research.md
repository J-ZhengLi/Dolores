# EH05 Windows isolation research

Research date: 2026-10-22. Scope: OS-enforced filesystem and network isolation
for owned agent command processes. This is a documentation review, not backend
qualification. No OS settings, firewall rules, optional features, or runtime code
were changed for this research.

Context: [frozen contracts](everyday-harness-contracts.md) and
[EH05 plan](../plans/everyday-harness/eh05.md). The supplied EH00 evidence has
`TokenIsAppContainer=true`, granted-directory access, sibling read/write failures
with Win32 error 5, and a successful host TCP loopback connection despite zero
capabilities. The current investigation also reports BFE, MpsSvc and all firewall
profiles enabled. These baseline observations establish a failed ordinary-
AppContainer network control. The later `registryRead`-only LPAC result below
must remain distinct; this note does not infer the baseline failure's cause.

## Recommendation and decision boundary

**Implementation decision after qualification:** EH05 selects LPAC with the
documented registryRead runtime capability for direct Node commands and reviewed
MCP servers on Windows. The later transport, crash, offline-build, native UI,
MCP save/restart and paired network evidence is recorded in
[ACCEPTANCE](../IMPLEMENTATION_HISTORY.md#acceptance-records).
The recommendation below preserves the earlier research checkpoint. Other
runtimes remain unsupported in isolated mode; Node children must inherit stdio.
No network-enabling capability is offered by this initial mode.

Keep H17 open and fail requested isolation closed. **LPAC with `registryRead` is
now a promising native candidate:** the controlled follow-up reached the network
test and denied host TCP loopback with error 10013 while its unrestricted control
succeeded. Granted-file operations and sibling denials also passed. Continue
qualifying `CreateProcessW` with this resource capability, narrow filesystem ACLs,
explicit inherited stdio and job ownership before selecting a production backend.
The paired ordinary-AppContainer/LPAC resource control now also verifies the
documented opt-out effect with identical registryRead grants: an
ALL_APPLICATION_PACKAGES-only public canary is readable in ordinary AppContainer
and denied under LPAC. The ordinary variant connects to host loopback; LPAC gets
10013. Both token-class-46 failures remain unknown in the receipt, separate from
this behavioral verification. Evidence:
`output/eh05-isolation/paired-f7417130998c448682fe9062579e9e26/paired-receipt.json`.
The wider native runtime/network matrix remains pending; this does not close H17.
If a qualified usable LPAC runtime still permits forbidden host/network access,
an explicit SID-scoped WFP
enforcement broker is a production candidate, with installation privileges and
crash lifetime design as hard prerequisites. It is not currently a mandatory
dependency. The API details and primary sources are below.

If a privileged broker is unacceptable, use a separately qualified guest runner
with no network device and a deliberately limited host channel. Windows Sandbox
provides a guest boundary but its CLI cannot supply the required streaming stdio.
HCS has documented process pipe handles, but Windows container production usage
on client editions has a material support/licensing gate. None of these options
is a qualified drop-in backend today.

| Candidate | OS enforcement and stdio | Hard boundary before selection |
| --- | --- | --- |
| LPAC + `registryRead` + ACLs | Token/DACL checks; current host TCP loopback denied with positive control; ordinary `CreateProcessW` pipe inheritance | Verify opt-out behavior and wider network/runtime matrix, descendants and lifecycle on supported hosts. |
| AppContainer/LPAC + owned WFP broker | Same filesystem boundary plus explicit network filters keyed by container SID; same stdio | Installer/admin-owned filtering rights; rule lifetime through every child; denial and broker-crash controls. |
| Experimental sandbox API | AppContainer/BFS composition; inherited handles prohibited | Experimental ABI, absent public header, runtime availability, separate IPC design. |
| Windows Sandbox | Separate kernel; configurable network disable and scoped mapped folders | Supported edition/virtualization; CLI has no process I/O; guest helper and controlled result channel. |
| HCS + Hyper-V isolated container | Separate kernel; documented stdin/stdout/stderr pipe handles | Runtime/image provisioning, no-network configuration, production client-edition licensing/support. |
| Owned full Hyper-V VM | Separate guest; AF_HYPERV stream can carry a guest helper protocol without TCP/IP | Hyper-V/admin setup, guest image licensing/updates, helper protocol and VM lifecycle qualification. |

## AppContainer and LPAC: exact launch contract

Microsoft documents `SECURITY_CAPABILITIES` in `STARTUPINFOEX` and
`EXTENDED_STARTUPINFO_PRESENT` to create an AppContainer. LPAC adds
`PROC_THREAD_ATTRIBUTE_ALL_APPLICATION_PACKAGES_POLICY` with a `DWORD`
`PROCESS_CREATION_ALL_APPLICATION_PACKAGES_OPT_OUT`. It removes matching through
`ALL_APPLICATION_PACKAGES`; the documentation does not describe it as an
additional network-denial flag. Keep zero network capabilities for the denied
mode. The launch example is not a guarantee that arbitrary installed tools will
run without additional resource grants.
[Launch an AppContainer](https://learn.microsoft.com/en-us/windows/win32/secauthz/implementing-an-appcontainer).

For FFI verification, Microsoft's SDK header defines the attribute number as 15,
with the input bit `0x00020000`, yielding `0x0002000F`; the opt-out value is
`0x01`. These definitions are Windows-10-gated in the header. Prefer SDK/binding
symbols over duplicated integers.
[Microsoft WinBase.h](https://raw.githubusercontent.com/microsoft/win32metadata/main/generation/WinSDK/RecompiledIdlHeaders/um/WinBase.h).

`TokenIsLessPrivilegedAppContainer` is a public `TOKEN_INFORMATION_CLASS` member;
its enum value is 46, derived from the published sequence starting at
`TokenUser=1`. `TokenIsAppContainer` alone does not establish LPAC. Record the
actual child token's AppContainer SID, capabilities, integrity and LPAC query
result; an unsupported/failed query is not a positive result.
[TOKEN_INFORMATION_CLASS](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ne-winnt-token_information_class),
[Microsoft winnt.h](https://raw.githubusercontent.com/microsoft/win32metadata/main/generation/WinSDK/RecompiledIdlHeaders/um/winnt.h).

The current diagnostic follow-up reports successful LPAC-attribute process
creation on Windows 11 Home build 26200, followed by
`GetTokenInformation(46, DWORD[1])` failing with error 87 while the child remained
suspended; it was then killed and the profile removed. No LPAC network result
was obtained. The WDK enum documentation specifies DWORD for class 46, but
documents Se/Zw token queries; the Win32 page omits its output layout and
class-specific version support. Do not convert the failed call into false/true,
or infer that changing buffer size fixes it.
[WDK TOKEN_INFORMATION_CLASS](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/ne-ntifs-_token_information_class),
[GetTokenInformation](https://learn.microsoft.com/en-us/windows/win32/api/securitybaseapi/nf-securitybaseapi-gettokeninformation).

A subsequent diagnostic LPAC launch reports a Rust Winsock initialization panic
with WSA error 10107 and exit 101; cleanup passed. Its receipt is
`output/eh05-isolation/lpac-23262c69354d4d0483084fcb9f61a1c1/receipt.json`, with
`work/panic.txt` in that directory. This is a runtime compatibility failure before
a meaningful network control, not evidence that a connection was denied. The
ordinary-AppContainer baseline still has the successful host-loopback connection.

### Follow-up: resource capabilities before a broker decision

Microsoft names **`registryRead`**, not `lpacRegistryRead`, as the capability an
LPAC needs for registry access. Its other explicit example is `lpacCom` for COM.
This supports investigating runtime resource dependencies while keeping network
capabilities absent.
[Launch an AppContainer](https://learn.microsoft.com/en-us/windows/win32/secauthz/implementing-an-appcontainer).
The sandbox API's capability table describes `registryRead` as read access to
HKLM registry hives and separately lists `internetClient`,
`internetClientServer` and `privateNetworkClientServer` as network grants.
Adding `registryRead` broadens registry visibility; it does not mean zero total
capabilities or preserve a claim that all non-workspace resources are inaccessible.
[Sandbox capability table](https://learn.microsoft.com/en-us/windows/win32/secauthz/createprocessinsandbox#capabilities-string-default-empty).

No authoritative Microsoft API/resource contract for the exact names
`lpacRegistryRead` or `lpacNetwork` was located in this review. Likewise, no
Microsoft documentation/source specifying the protected resources and network
effects of `lpacIdentityServices` was found. This is a research limitation, not
proof that a name does not exist on Windows. Do not infer a supported Windows
grant from a plausible name or successful name-to-SID conversion; do not add
identity-service or network-related capabilities speculatively.

`DeriveCapabilitySidsFromName` constructs both service-group SIDs and
AppAuthority capability SIDs. It returns `BOOL` (`TRUE` success; `FALSE` then
`GetLastError`), not `HRESULT`. The documented DLL is `KernelBase.dll`, import
library `OneCoreUAP.lib`, and header `securitybaseapi.h`. The call is:

```cpp
DeriveCapabilitySidsFromName(
    L"registryRead", &groupSids, &groupCount, &capSids, &capCount);
```

Keep the capability-array SIDs alive through process creation, then free each SID
and both returned arrays using `LocalFree`. Check failure and counts rather than
assuming allocation succeeded. The service-group array is not the AppContainer
capability list.
[DeriveCapabilitySidsFromName](https://learn.microsoft.com/en-us/windows/win32/api/securitybaseapi/nf-securitybaseapi-derivecapabilitysidsfromname).
Microsoft's launcher source uses `capSids` entries with `SE_GROUP_ENABLED` in
`SID_AND_ATTRIBUTES`, then supplies that list to profile creation and
`SECURITY_CAPABILITIES`; it frees the group array and retains capability SID
ownership until the list is destroyed.
[Microsoft SandboxSecurityTools launcher](https://github.com/microsoft/SandboxSecurityTools/blob/main/LaunchAppContainer/LaunchAppContainer/LaunchAppContainer.cpp).

Error 10107 is `WSASYSCALLFAILURE`. Microsoft explicitly includes registry
function failures manipulating Winsock protocol/namespace catalogs among its
possible causes; it also lists other causes. Missing LPAC registry access is
therefore a supported hypothesis to test, not an established diagnosis.
[Winsock errors](https://learn.microsoft.com/en-us/windows/win32/winsock/windows-sockets-error-codes-2).
`WSAStartup` must succeed before other Winsock calls; record its direct return
code rather than replacing it with a later last-error value.
[WSAStartup](https://learn.microsoft.com/en-us/windows/win32/api/winsock/nf-winsock-wsastartup).

**Controlled follow-up design:** compare the existing LPAC/no-capabilities fixture
with a fresh LPAC profile whose sole capability is `registryRead`. Keep executable,
filesystem grants, owned host listener, stdio and deadline identical. Report the
actual token capability SIDs; `internetClient`, `internetClientServer` and
`privateNetworkClientServer` must remain absent, with no loopback exemption or
firewall mutation. First check `WSAStartup`, then attempt the bounded host
loopback connection only after initialization succeeds. Retain sibling-file
denial and an unrestricted listener-positive control. Preserve the class-46
query gap and use the proposed package-group behavioral canary if LPAC identity
needs independent evidence. Do not bundle additional capability guesses into this
probe. A startup success establishes compatibility only; a refused/blocked
connection still needs its complete control evidence before any isolation claim.

**Reported result, 2026-10-22:** the root's `registryRead`-only LPAC probe reached
the actual host TCP loopback control and returned error 10013; the unrestricted
control returned 0. Granted-file operations succeeded, sibling read/write both
returned error 5, and the outside canary remained preserved. Profile deletion
succeeded and `cleanupErrors` was empty. The receipt is
`output/eh05-isolation/lpac-registry-5894d9879bd345548dad06764528b136/receipt.json`.
The probe still exited 1 solely because the retained class-46 identity query
returned error 87. Treat that as an explicit identity-verification gap, not a
network-test failure or an invented positive LPAC query.

This result separates a usable resource-capability configuration from the earlier
zero-capability Winsock panic. It supports actual denial of this one host TCP
loopback connection with positive control. It does not establish general Internet,
LAN, IPv6, UDP, inbound, descendant or real-toolchain isolation. The planned
`ALL_APPLICATION_PACKAGES`-only resource A/B canary should verify the documented
opt-out access-check effect while preserving the failed token-query receipt.
Continue bounded LPAC qualification before choosing a broker. No registry-write,
identity-service, network-grant or host-settings broadening follows from this
probe result.

Do not substitute `TOKEN_ACCESS_INFORMATION.Flags & 0x80000` as supported LPAC
verification: Microsoft documents `Flags` as reserved, and the reviewed public
SDK header supplies no `TOKEN_NOT_LOW` contract. The documented
`CheckTokenMembershipEx` flag is `CTMF_INCLUDE_APPCONTAINER`, not an LPAC-specific
verification procedure. A controlled resource accessible through
`ALL_APPLICATION_PACKAGES` but not the package SID/capabilities can provide an
A/B behavioral opt-out control; that is proposed evidence of its access-check
effect, not a substitute for network-denial controls.
[TOKEN_ACCESS_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-token_access_information),
[CheckTokenMembershipEx](https://learn.microsoft.com/en-us/windows/win32/api/securitybaseapi/nf-securitybaseapi-checktokenmembershipex).

The supported stdio construction uses `STARTF_USESTDHANDLES` and
`PROC_THREAD_ATTRIBUTE_HANDLE_LIST`, containing only the three child pipe ends.
Those handles must be inheritable and `CreateProcessW.bInheritHandles` must be
true. Keep the opposite parent ends non-inheritable. This is distinct from
Microsoft's no-inheritance launch example. A proposed launch uses
`EXTENDED_STARTUPINFO_PRESENT | CREATE_SUSPENDED | CREATE_UNICODE_ENVIRONMENT`:
establish policy and process ownership before resume; abort on any setup error.
The suspended ordering is a design recommendation, not a tested result.
[UpdateProcThreadAttribute](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-updateprocthreadattribute),
[redirected child input/output](https://learn.microsoft.com/en-us/windows/win32/procthread/creating-a-child-process-with-redirected-input-and-output).

Filesystem isolation is access control, not a virtual copy of the host namespace.
Microsoft explicitly describes read-only access as less restricted and grants
read/write via resource permissions. LPAC narrows ambient package-group access;
it still requires a declared runtime dependency surface.
[AppContainer isolation](https://learn.microsoft.com/en-us/windows/win32/secauthz/appcontainer-isolation).

Proposed filesystem policy: private writable workspace/temp/profile; explicit
read-only executable/runtime roots; no user-profile, credential or broad cache
grant. Journal only owned ACL additions. Remove them after process-tree death
without overwriting concurrent unrelated ACL changes. Qualification must cover
reparse points, hard links, inherited handles, UNC paths and cross-session access;
the sibling-file result does not settle those cases. Shared writable toolchains
and caches are a separate reviewed grant, not an implicit compatibility fix.

## Network denial and the loopback discrepancy

Microsoft describes network isolation as WFP filtering with capability conditions
and default-block filters. Its troubleshooting guidance distinguishes loopback
permission, capability tokens and private-network classification. Enabled services
and profiles do not identify which filter handled this particular connection.
Use the child identity and effective WFP decision to investigate the allow; do
not declare disabled firewall services the cause against the supplied evidence.
The guidance is written for UWP and therefore is useful diagnostic evidence,
not proof of an unpackaged command's exact policy.
[UWP firewall troubleshooting](https://learn.microsoft.com/en-us/windows/security/operating-system-security/network-security/windows-firewall/troubleshooting-uwp-firewall).

Read-only follow-up can enumerate loopback exemptions with
`NetworkIsolationGetAppContainerConfig`; it returns the SIDs allowed to send
loopback traffic. Check the exact generated profile SID. Do not add an exemption
to make the transport work: that directly changes the denied property.
[NetworkIsolationGetAppContainerConfig](https://learn.microsoft.com/en-us/windows/win32/api/networkisolation/nf-networkisolation-networkisolationgetappcontainerconfig).

For explicit enforcement, WFP exposes `FWPM_CONDITION_ALE_PACKAGE_ID` as the
AppContainer SID. Match identity rather than just an executable path, so a
reviewed policy follows commands with different executable names. A proposed
deny-all implementation covers IPv4 and IPv6, host loopback, outbound and inbound;
verify the condition is available at each selected layer.
[WFP condition identifiers](https://learn.microsoft.com/en-us/windows/win32/fwp/filtering-condition-identifiers-).

`ALE_AUTH_CONNECT_V4/V6` classifies TCP connect, initial UDP send tuples and
outbound ICMP; `ALE_AUTH_RECV_ACCEPT_V4/V6` classifies TCP accept, initial inbound
UDP and ICMP. These are concrete candidate enforcement layers, with additional
bind/listen coverage reviewed as needed. Do not skip loopback with an
`IS_LOOPBACK` exception. Retain same-container socket-pair tests so network denial
does not silently hang Python/Node event loops.
[ALE layers](https://learn.microsoft.com/en-us/windows/win32/fwp/ale-layers).

Adding filters requires `FWPM_ACTRL_ADD` and relevant `ADD_LINK` rights. The
default engine descriptor gives administrators full access; ordinary callers'
`OPEN`/`CLASSIFY` rights do not grant filter installation. Thus a desktop-process
WFP design must include an explicitly provisioned privileged component or
appropriately delegated service identity; it cannot promise privilege-free
operation merely because `FwpmEngineOpen0` succeeds.
[WFP access control](https://learn.microsoft.com/en-us/windows/win32/fwp/access-control).

### WFP and process lifetime must be designed together

Dynamic WFP sessions automatically delete their objects when the session/program
ends. This is useful cleanup, but **does not prove fail-closed broker-crash
behavior**: deleting the deny rule while a child survives would reopen the failed
loopback path. That consequence is an inference from documented lifetime rules.
[FwpmEngineOpen0](https://learn.microsoft.com/en-us/windows/win32/api/fwpmu/nf-fwpmu-fwpmengineopen0).

One candidate is journaled, uniquely identified deny filters retained through
broker failure until an independent owner/recovery path confirms all processes
for that session are gone. WFP supports persistent filters. Grant exceptions,
filter arbitration and recovery under BFE restart still need a concrete design;
do not implement a blanket block followed by an assumption that another allow
automatically overrides it.
[FwpmFilterAdd0](https://learn.microsoft.com/en-us/windows/win32/api/fwpmu/nf-fwpmu-fwpmfilteradd0).

Keep the existing kill-on-close job, disallow breakaway and test descendants.
Jobs manage groups and can terminate them; process security must be set
separately. Brokered external execution, scheduled tasks and unsupported helper
launches must fail isolation admission rather than become unrestricted escape
paths. Stop should terminate, wait, drain bounded output, then revoke grants.
[Job objects](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects).

## Experimental CreateProcessInSandbox and transport

`Experimental_CreateProcessInSandbox` requires `inheritHandles=FALSE` and rejects
true with `ERROR_NOT_SUPPORTED`. It is experimental, exported by
`processmodel.dll`, has no public header, and requires runtime symbol discovery.
Its BFS filesystem grants and network policy require `app_container=true`.
Therefore it cannot directly reuse inherited anonymous-pipe stdio. Merely filling
the `STARTUPINFO` standard-handle fields is insufficient. Treat a bootstrap
helper that opens an explicitly granted IPC endpoint and sets its own stdio as a
new protocol, not existing API support. Its ABI and transport need separate
qualification before any production commitment.
[Create Process in Sandbox](https://learn.microsoft.com/en-us/windows/win32/secauthz/createprocessinsandbox).

A host named pipe is not automatically accessible to an AppContainer: Microsoft
documents AppContainer pipe namespace constraints. Default pipe ACLs also expose
read access to Everyone/anonymous; a safe proposal needs an explicit descriptor,
remote-client rejection, first-instance protection, peer identity checks and
bounded overlapped I/O. This research does not establish that such a host-to-LPAC
pipe works on the target build. Do not loosen network policy to compensate.
[CreateNamedPipe](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-createnamedpipea).

Microsoft provides `GetAppContainerNamedObjectPath` and an example of a desktop
process creating an accessible named object. Shared memory plus synchronization
objects is consequently a research candidate, not a ready stdio replacement;
it adds framing, authentication, backpressure and cancellation responsibilities.
[GetAppContainerNamedObjectPath](https://learn.microsoft.com/en-us/windows/win32/api/securityappcontainer/nf-securityappcontainer-getappcontainernamedobjectpath).

## Guest-backed alternatives

### Windows Sandbox

Windows Sandbox uses a separate kernel with hypervisor isolation. Host-installed
software is not automatically present, and closing discards guest state. It is
available on supported Pro/Enterprise/Education editions, not Home. Microsoft's
overview currently says concurrent instances are unsupported; do not assume the
four-host-session contract can become four independent WSB guests.
[Windows Sandbox overview](https://learn.microsoft.com/en-us/windows/security/application-security/application-isolation/windows-sandbox/).

Prerequisites include BIOS virtualization, at least 4 GB RAM, 1 GB free disk and
two cores; feature enabling can require administrator action and a restart.
These are hard environment prerequisites, not changes this research authorizes.
[Install Windows Sandbox](https://learn.microsoft.com/en-us/windows/security/application-security/application-isolation/windows-sandbox/windows-sandbox-install).

Use a `.wsb` configuration with networking disabled, narrowly mapped folders,
read-only tool/bootstrap input and a dedicated writable result area if required.
Explicitly disable clipboard, microphone and other unneeded redirections. Defaults
enable networking and clipboard. Mapped-folder writes remain host writes after
the guest closes. The configuration reference spells networking values
`Enable`/`Disable`; the CLI example uses `Disabled`, so qualify the exact accepted
configuration instead of assuming both spellings work.
[Configure Windows Sandbox](https://learn.microsoft.com/en-us/windows/security/application-security/application-isolation/windows-sandbox/windows-sandbox-configure-using-wsb-file).

The Windows 11 24H2+ `wsb` CLI offers start/list/exec/stop/share. Its `exec`
documentation explicitly states that process I/O is unsupported. It returns an
exit code, not owned interactive stdin/stdout/stderr. `stop` terminates the guest
and releases its resources. A file-based guest helper can be investigated for
bounded batch execution, but streaming input, authenticated messages, output
flooding and interruption remain a separate implementation/acceptance scope.
[Windows Sandbox CLI](https://learn.microsoft.com/en-us/windows/security/application-security/application-isolation/windows-sandbox/windows-sandbox-cli).

### Hyper-V isolated containers through HCS

Hyper-V isolation gives a container its own kernel, unlike process-isolated
containers that share the host kernel. Shared folders, pipes and ports deliberately
pierce this boundary; expose only the reviewed workspace and control channel.
[Container isolation modes](https://learn.microsoft.com/en-us/virtualization/windowscontainers/manage-containers/hyperv-container).

`HcsCreateProcess` in `ComputeCore.dll` starts a process in an owned compute
system. Success starts an asynchronous operation; inspect its completion before
reporting a running session. This is documented for Windows 10 1809 and later.
[HcsCreateProcess](https://learn.microsoft.com/en-us/virtualization/api/hcs/reference/hcscreateprocess).

The process schema has `CreateStdInPipe`, `CreateStdOutPipe`, `CreateStdErrPipe`;
avoid `EmulateConsole` when separate stderr is required, because that mode ignores
the stderr pipe.
[HCS ProcessParameters](https://learn.microsoft.com/en-us/virtualization/api/hcs/schemareference#processparameters).
The result structure returns `StdInput`, `StdOutput` and `StdError` handles.
This is a concrete documented stdio alternative to WSB CLI.
[HCS_PROCESS_INFORMATION](https://learn.microsoft.com/en-us/virtualization/api/hcs/reference/hcs_process_information).

Provisioning still requires Hyper-V/container support, a runtime and compatible
images. Microsoft's FAQ limits Windows Pro/Enterprise container use to testing
and development; production distribution on Dolores' Windows desktop target
requires a resolved licensing/support position before selecting this backend.
Do not call it production-ready simply because the API exists.
[Container requirements](https://learn.microsoft.com/en-us/virtualization/windowscontainers/deploy-containers/system-requirements),
[Windows Containers FAQ](https://learn.microsoft.com/en-us/virtualization/windowscontainers/about/faq).

Cleanup must terminate and await the compute system, then close handles;
`HcsCloseComputeSystem` is documented as closing a handle, whereas
`HcsTerminateComputeSystem` forcefully terminates and requires an operation-result
check. Crash cleanup needs an independently verified lifetime policy.
[HcsCloseComputeSystem](https://learn.microsoft.com/en-us/virtualization/api/hcs/reference/hcsclosecomputesystem),
[HcsTerminateComputeSystem](https://learn.microsoft.com/en-us/virtualization/api/hcs/reference/hcsterminatecomputesystem).

### Owned full VM

Microsoft documents AF_HYPERV/SOCK_STREAM host/guest communication without
TCP/IP networking. A small guest supervisor could own ordinary guest pipes and
multiplex the three streams over this channel while the guest has no network
adapter. This is an architectural inference, not Windows Sandbox integration
support. Service registration requires administrator action. Bind to the owned
VM/service identity, authenticate the protocol, and expose no general host
execution/file broker.
[Hyper-V sockets](https://learn.microsoft.com/en-us/windows-server/virtualization/hyper-v/make-integration-service).

Hyper-V requires supported editions, SLAT-capable hardware, firmware
virtualization and sufficient memory. Guest licensing, image distribution,
patching, workspace transfer and resource costs remain product prerequisites.
[Hyper-V requirements](https://learn.microsoft.com/en-us/windows-server/virtualization/hyper-v/host-hardware-requirements).

## Bounded qualification proposal

The following are proposed negative controls, not checks executed by this note.
Use public temporary canaries and owned listeners only; no provider credentials
or private transcripts. Keep individual connects to a short timeout, output to
the existing 256 KiB/session ceiling and the complete trial within a reviewed
deadline. Every denied endpoint needs an unrestricted positive control first,
so timeout alone cannot pass a network-denial test.

1. **Identity and files:** AppContainer and LPAC A/B launches, distinct profiles,
   same executable/ACL/network inputs. Confirm the actual token, then permit the
   workspace read/write, deny sibling read/write, permit read-only runtime access
   and deny writes there. Repeat denied operations in a child/grandchild and via
   an in-workspace reparse path to a sibling. Retain exact OS errors.
2. **Network matrix:** host listeners on `127.0.0.1` and `::1`, a reachable owned
   host-LAN endpoint, and a controlled external endpoint if explicitly available.
   Cover TCP and UDP, DNS behavior, inbound listener exposure and descendants.
   A second container must not reach the first. Also exercise a same-container
   socket pair to detect compatibility deadlocks. Record unavailable controls as
   gaps; do not invent Internet/private-network acceptance from one loopback test.
3. **Setup and lifetime failure:** deliberately fail attribute/ACL/filter
   installation before resume; prove no command marker was written. Interrupt
   during blocked input and output flood, kill the owning broker, and confirm
   there is no surviving unrestricted child or network-permission window. Retain
   bounded usable output and an actionable failure receipt. Reopen reconciles
   owned residual filters/ACLs/guest IDs without repeating commands.
4. **Real tool compatibility:** only after the boundary passes, run an offline
   Node/Python/build fixture using reviewed runtime roots and caches. Exercise
   Unicode stdin, separate stdout/stderr, EOF, timeout, Stop and child cleanup.
   An unavailable dependency must identify the denied resource and offer explicit
   review or a different backend, not broaden permissions silently.

Hard gates remain: the ordinary-AppContainer loopback allow is unexplained;
the LPAC/`registryRead` host-TCP denial is evidenced, but LPAC identity-query
compatibility, the broader network/runtime matrix,
descendant and crash behavior are unqualified. If a WFP/guest alternative becomes
necessary, its privileged setup, crash-safe policy lifetime or feature/image/
licensing/transport prerequisites remain unresolved. Actual toolchain and
external-helper coverage are unqualified. Keep EH05.3/H17 open until the selected backend passes
both filesystem and network controls with retained receipts in
[ACCEPTANCE.md](../ACCEPTANCE.md).

The reported Windows 11 Home host also fails the published edition prerequisite
for Windows Sandbox and client Hyper-V. These routes require a supported host
edition or a separately supported virtualization backend; they are not immediate
fallbacks for this machine.
[Windows Sandbox editions](https://learn.microsoft.com/en-us/windows/security/application-security/application-isolation/windows-sandbox/),
[Hyper-V installation editions](https://learn.microsoft.com/en-us/windows-server/virtualization/hyper-v/get-started/install-hyper-v?pivots=windows&tabs=gui).

### Runtime implementation follow-up, October 22

An unadvertised Rust LPAC transport now exercises actual grants, filtered launch,
inherited pipes, network controls and recovery; see the current evidence in
[ACCEPTANCE.md](../ACCEPTANCE.md). This is implementation evidence, not completion
of product routing or the full EH05 boundary.

The implementation uses handle-based
[GetSecurityInfo](https://learn.microsoft.com/en-us/windows/win32/api/aclapi/nf-aclapi-getsecurityinfo)
and [SetSecurityInfo](https://learn.microsoft.com/en-us/windows/win32/api/aclapi/nf-aclapi-setsecurityinfo).
It opens explicit READ_CONTROL/WRITE_DAC rights rather than MAXIMUM_ALLOWED and
merges/revokes the unique package trustee using
[SetEntriesInAclW](https://learn.microsoft.com/en-us/windows/win32/api/aclapi/nf-aclapi-setentriesinaclw).
It refuses a null DACL instead of replacing unrestricted access with a new
package-only ACL. A host gate serializes its own ACL mutations. Other-process
ACL races are not turned into a transaction guarantee.

Journaled cleanup may repeat
[DeleteAppContainerProfile](https://learn.microsoft.com/en-us/windows/win32/api/userenv/nf-userenv-deleteappcontainerprofile):
the API documents success when the profile is already absent and recommends
repeating an incomplete deletion. The current fixtures exercise repeated recovery,
retained cleanup evidence and an owner hard kill. Broader filesystem/runtime and
end-to-end app integration remain required before advertising the mode.
