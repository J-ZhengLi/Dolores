"""Bounded EH00/EH05 AppContainer/LPAC probe, not production isolation.

Creates a disposable profile and changes ACLs only under a fresh owned output
directory. No firewall rules, loopback exemptions, installs or elevation. Tests
local loopback networking only; Internet/enterprise policy remains unqualified.
"""
import argparse
import ctypes as c
from ctypes import wintypes as w
import json
import os
from pathlib import Path
import socket
import shutil
import subprocess
import sys
import uuid
import threading
from contextlib import contextmanager

ROOT = Path(__file__).resolve().parents[1]


@contextmanager
def loopback_listener():
    """Drain the owned positive control; never leave a growing accept backlog."""
    stop = threading.Event()
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        listener.listen(8)
        listener.settimeout(0.1)
        def accept():
            while not stop.is_set():
                try:
                    connection, _ = listener.accept()
                    connection.close()
                except socket.timeout:
                    continue
                except OSError:
                    break
        worker = threading.Thread(target=accept, daemon=True)
        worker.start()
        try:
            yield listener
        finally:
            stop.set()
            worker.join(timeout=1)


class Startup(c.Structure):
    _fields_ = [("cb", w.DWORD), ("reserved", w.LPWSTR), ("desktop", w.LPWSTR),
                ("title", w.LPWSTR)] + [(n, w.DWORD) for n in
                ("x", "y", "xsize", "ysize", "xchars", "ychars", "fill", "flags")] + [
                ("show", w.WORD), ("reserved_size", w.WORD), ("reserved_bytes", c.c_void_p),
                ("stdin", w.HANDLE), ("stdout", w.HANDLE), ("stderr", w.HANDLE)]


class StartupEx(c.Structure):
    _fields_ = [("startup", Startup), ("attributes", c.c_void_p)]


class Process(c.Structure):
    _fields_ = [("process", w.HANDLE), ("thread", w.HANDLE),
                ("pid", w.DWORD), ("tid", w.DWORD)]


class Capabilities(c.Structure):
    _fields_ = [("sid", c.c_void_p), ("capabilities", c.c_void_p),
                ("count", w.DWORD), ("reserved", w.DWORD)]

class SidAndAttributes(c.Structure):
    _fields_ = [("sid", c.c_void_p), ("attributes", w.DWORD)]

class SecurityAttributes(c.Structure):
    _fields_ = [("size", w.DWORD), ("descriptor", c.c_void_p), ("inherit", w.BOOL)]


class PipeIo:
    """Tiny public stdio control; bounded reads only after the fixture exits."""
    def __init__(self, kernel):
        self.kernel, self.handles = kernel, []
        self.close_handle = api(kernel, "CloseHandle", w.BOOL, [w.HANDLE])
        create_pipe = api(kernel, "CreatePipe", w.BOOL,
                          [c.POINTER(w.HANDLE), c.POINTER(w.HANDLE), c.POINTER(SecurityAttributes), w.DWORD])
        set_info = api(kernel, "SetHandleInformation", w.BOOL, [w.HANDLE, w.DWORD, w.DWORD])
        attributes = SecurityAttributes(c.sizeof(SecurityAttributes), None, True)
        pairs = []
        try:
            for _ in range(3):
                read, write = w.HANDLE(), w.HANDLE()
                checked(create_pipe(c.byref(read), c.byref(write), c.byref(attributes), 4096))
                self.handles.extend([read.value, write.value])
                pairs.append((read.value, write.value))
            self.input, self.output, self.error = pairs
            for handle in [self.input[1], self.output[0], self.error[0]]:
                checked(set_info(handle, 1, 0))
            self.children = (w.HANDLE * 3)(self.input[0], self.output[1], self.error[1])
        except BaseException:
            self.close()
            raise

    def send(self):
        for handle in self.children:
            self.close_handle(handle)
            self.handles.remove(handle)
        value = "PUBLIC_STDIN 世界\n".encode()
        write = api(self.kernel, "WriteFile", w.BOOL,
                    [w.HANDLE, c.c_void_p, w.DWORD, c.POINTER(w.DWORD), c.c_void_p])
        count = w.DWORD()
        checked(write(self.input[1], value, len(value), c.byref(count), None))
        if count.value != len(value):
            raise AssertionError("Public stdin control was not fully written")
        self.close_handle(self.input[1])
        self.handles.remove(self.input[1])

    def receive(self):
        peek = api(self.kernel, "PeekNamedPipe", w.BOOL,
                   [w.HANDLE, c.c_void_p, w.DWORD, c.c_void_p, c.POINTER(w.DWORD), c.c_void_p])
        read = api(self.kernel, "ReadFile", w.BOOL,
                   [w.HANDLE, c.c_void_p, w.DWORD, c.POINTER(w.DWORD), c.c_void_p])
        result = {}
        for name, handle in [("stdout", self.output[0]), ("stderr", self.error[0])]:
            available, count = w.DWORD(), w.DWORD()
            if not peek(handle, None, 0, None, c.byref(available), None):
                if c.get_last_error() != 109:
                    checked(False)
            buffer = c.create_string_buffer(min(available.value, 8192))
            if available.value:
                checked(read(handle, buffer, len(buffer), c.byref(count), None))
            result[name] = buffer.raw[:count.value].decode('utf-8', errors='replace')
            result[name + "Truncated"] = available.value > 8192
        return result

    def close(self):
        for handle in self.handles:
            self.close_handle(handle)
        self.handles.clear()

def api(dll, name, result, arguments):
    fn = getattr(dll, name)
    fn.restype, fn.argtypes = result, arguments
    return fn


def checked(ok):
    if not ok:
        raise c.WinError(c.get_last_error())


def command(args):
    return subprocess.run(args, check=True, capture_output=True, text=True,
                          encoding="utf-8", errors="replace", timeout=60)


def probe(directory, policy, registry_read, runtime="rust", node_entry="file", stdio=False):
    kernel = c.WinDLL("kernel32", use_last_error=True)
    userenv = c.WinDLL("userenv", use_last_error=True)
    advapi = c.WinDLL("advapi32", use_last_error=True)
    create_profile = api(userenv, "CreateAppContainerProfile", c.c_long,
                         [w.LPCWSTR, w.LPCWSTR, w.LPCWSTR, c.c_void_p, w.DWORD, c.POINTER(c.c_void_p)])
    delete_profile = api(userenv, "DeleteAppContainerProfile", c.c_long, [w.LPCWSTR])
    sid_text = api(advapi, "ConvertSidToStringSidW", w.BOOL, [c.c_void_p, c.POINTER(w.LPWSTR)])
    free_sid = api(advapi, "FreeSid", c.c_void_p, [c.c_void_p])
    local_free = api(kernel, "LocalFree", c.c_void_p, [c.c_void_p])
    initialize = api(kernel, "InitializeProcThreadAttributeList", w.BOOL,
                     [c.c_void_p, w.DWORD, w.DWORD, c.POINTER(c.c_size_t)])
    update = api(kernel, "UpdateProcThreadAttribute", w.BOOL,
                 [c.c_void_p, w.DWORD, c.c_size_t, c.c_void_p, c.c_size_t, c.c_void_p, c.c_void_p])
    destroy = api(kernel, "DeleteProcThreadAttributeList", None, [c.c_void_p])
    create = api(kernel, "CreateProcessW", w.BOOL,
                 [w.LPCWSTR, w.LPWSTR, c.c_void_p, c.c_void_p, w.BOOL, w.DWORD,
                  c.c_void_p, w.LPCWSTR, c.POINTER(StartupEx), c.POINTER(Process)])
    close = api(kernel, "CloseHandle", w.BOOL, [w.HANDLE])
    wait = api(kernel, "WaitForSingleObject", w.DWORD, [w.HANDLE, w.DWORD])
    terminate = api(kernel, "TerminateProcess", w.BOOL, [w.HANDLE, w.UINT])
    exit_code = api(kernel, "GetExitCodeProcess", w.BOOL, [w.HANDLE, c.POINTER(w.DWORD)])
    open_token = api(advapi, "OpenProcessToken", w.BOOL,
                     [w.HANDLE, w.DWORD, c.POINTER(w.HANDLE)])
    token_info = api(advapi, "GetTokenInformation", w.BOOL,
                     [w.HANDLE, w.DWORD, c.c_void_p, w.DWORD, c.POINTER(w.DWORD)])
    name = "Dolores.EH00.Probe." + uuid.uuid4().hex
    sid, text = c.c_void_p(), w.LPWSTR()
    bin_dir, work = directory / "bin", directory / "work"
    bin_dir.mkdir()
    work.mkdir()
    package_dir = directory / "package-only"
    package_dir.mkdir()
    package_file = package_dir / "public.txt"
    package_file.write_text("public-package-probe", encoding="utf-8")
    outside = directory / "outside.txt"
    outside.write_text("private-probe", encoding="utf-8")
    (work / "input.txt").write_text("public-probe", encoding="utf-8")
    executable = bin_dir / "probe.exe"
    if runtime == "node":
        source = shutil.which("node")
        if source is None:
            raise RuntimeError("Node is unavailable; no runtime or policy changed")
        shutil.copy2(source, executable)
        shutil.copy2(ROOT / "scripts/fixtures/harness-isolation-child.cjs", work / "probe.cjs")
    else:
        command(["rustc", "--edition=2021", "-C", "target-feature=+crt-static", "-O",
                 str(ROOT / "scripts/fixtures/harness-isolation-child.rs"), "-o", str(executable)])
    hr = create_profile(name, "Dolores EH00 probe", "Disposable feasibility probe", None, 0, c.byref(sid))
    if hr < 0:
        raise OSError(f"CreateAppContainerProfile HRESULT 0x{hr & 0xffffffff:08x}")
    report = {"diagnosticOnly": True, "policy": policy, "runtime": runtime,
              "nodeEntry": node_entry if runtime == "node" else None,
              "resourceCapabilities": ["registryRead"] if registry_read else [],
              "networkCapabilities": [],
              "networkScope": "loopback only", "profileDeleted": False}
    granted = []
    package_granted = False
    groups, capabilities = c.POINTER(c.c_void_p)(), c.POINTER(c.c_void_p)()
    group_count, capability_count = w.DWORD(), w.DWORD()
    try:
        checked(sid_text(sid, c.byref(text)))
        principal = "*" + text.value
        # The base and every ACL target are freshly created children of output.
        for target, grant in [(bin_dir, "(OI)(CI)RX"), (work, "(OI)(CI)M")]:
            command(["icacls", str(target), "/grant", f"{principal}:{grant}", "/T", "/Q"])
            granted.append(target)
        # ALL_APPLICATION_PACKAGES only; never grant this canary to the package
        # SID or registryRead capability. The paired run changes only LPAC policy.
        command(["icacls", str(package_dir), "/grant", "*S-1-15-2-1:(OI)(CI)RX", "/T", "/Q"])
        package_granted = True
        with loopback_listener() as listener:
            address = f"127.0.0.1:{listener.getsockname()[1]}"
            entry = (["-e", (work / "probe.cjs").read_text(encoding="utf-8")] if node_entry == "eval" else (["--preserve-symlinks-main"] if node_entry == "preserve-file" else []) + [str(work / "probe.cjs")]) if runtime == "node" else []
            args = [str(executable)] + entry + [str(work), str(outside), address, str(package_file)] + (["stdio"] if stdio else [])
            control_process = subprocess.run(args, input="PUBLIC_STDIN 世界\n".encode() if stdio else None,
                                             check=True, capture_output=True, timeout=10)
            report["control"] = json.loads((work / "receipt.json").read_text())
            if runtime == "node":
                report["controlRuntimeErrors"] = report["control"].pop("nodeErrorDetails", {})
                if stdio:
                    report["controlStdio"] = report["control"].pop("nodeStdio", {})
                    report["controlPipes"] = {"stdout":control_process.stdout.decode('utf-8', errors='replace'),"stderr":control_process.stderr.decode('utf-8', errors='replace')}
            outside.write_text("private-probe", encoding="utf-8")
            (work / "receipt.json").unlink()
            size = c.c_size_t()
            attribute_count = (2 if policy == "lpac" else 1) + int(stdio)
            initialize(None, attribute_count, 0, c.byref(size))
            attributes = c.create_string_buffer(size.value)
            checked(initialize(attributes, attribute_count, 0, c.byref(size)))
            pipes = None
            try:
                caps = Capabilities(sid, None, 0, 0)
                if registry_read:
                    derive = api(c.WinDLL("kernelbase", use_last_error=True),
                                 "DeriveCapabilitySidsFromName", w.BOOL,
                                 [w.LPCWSTR, c.POINTER(c.POINTER(c.c_void_p)), c.POINTER(w.DWORD),
                                  c.POINTER(c.POINTER(c.c_void_p)), c.POINTER(w.DWORD)])
                    checked(derive("registryRead", c.byref(groups), c.byref(group_count),
                                   c.byref(capabilities), c.byref(capability_count)))
                    entries = (SidAndAttributes * capability_count.value)(
                        *(SidAndAttributes(capabilities[i], 4) for i in range(capability_count.value)))
                    caps.capabilities = c.cast(entries, c.c_void_p)
                    caps.count = capability_count.value
                checked(update(attributes, 0, 0x20009, c.byref(caps), c.sizeof(caps), None, None))
                # ALL_APPLICATION_PACKAGES_POLICY / OPT_OUT from WinBase.h.
                # Keep the attribute value alive until CreateProcessW returns.
                package_policy = w.DWORD(1)
                if policy == "lpac":
                    checked(update(attributes, 0, 0x2000F, c.byref(package_policy),
                                   c.sizeof(package_policy), None, None))
                startup = StartupEx()
                startup.startup.cb = c.sizeof(startup)
                startup.attributes = c.cast(attributes, c.c_void_p)
                if stdio:
                    pipes = PipeIo(kernel)
                    checked(update(attributes, 0, 0x20002, pipes.children, c.sizeof(pipes.children), None, None))
                    startup.startup.flags = 0x100
                    startup.startup.stdin, startup.startup.stdout, startup.startup.stderr = pipes.children
                info = Process()
                line = c.create_unicode_buffer(subprocess.list2cmdline(args))
                # No inherited handles; no network capabilities; hidden fixture.
                checked(create(str(executable), line, None, None, stdio, 0x08080004,
                               None, str(work), c.byref(startup), c.byref(info)))
                try:
                    token, is_container, used = w.HANDLE(), w.DWORD(), w.DWORD()
                    checked(open_token(info.process, 8, c.byref(token)))
                    try:
                        checked(token_info(token, 29, c.byref(is_container), c.sizeof(is_container), c.byref(used)))
                        report["tokenIsAppContainer"] = is_container.value == 1
                        is_lpac = w.DWORD()
                        if token_info(token, 46, c.byref(is_lpac),
                                      c.sizeof(is_lpac), c.byref(used)):
                            report["tokenIsLessPrivilegedAppContainer"] = is_lpac.value != 0
                        else:
                            # Preserve unsupported-query evidence and still exercise
                            # the boundary. Never turn a failed query into true/false.
                            # A separate paired resource control tests opt-out behavior.
                            report["tokenIsLessPrivilegedAppContainer"] = None
                            report["lpacQueryError"] = c.get_last_error()
                    finally:
                        close(token)
                    resume = api(kernel, "ResumeThread", w.DWORD, [w.HANDLE])
                    if resume(info.thread) == 0xffffffff:
                        raise c.WinError(c.get_last_error())
                    if pipes:
                        pipes.send()
                    if wait(info.process, 10000) != 0:
                        checked(terminate(info.process, 3))
                        wait(info.process, 2000)
                        raise TimeoutError("AppContainer fixture exceeded ten seconds")
                    code = w.DWORD()
                    checked(exit_code(info.process, c.byref(code)))
                    report["exitCode"] = code.value
                    if pipes:
                        report["isolatedPipes"] = pipes.receive()
                    receipt_path = work / "receipt.json"
                    report["isolated"] = json.loads(receipt_path.read_text()) if receipt_path.exists() else {}
                    if not receipt_path.exists():
                        report["runtimeFailure"] = "Process exited without a receipt; no denied-access verdict can be inferred"
                    if runtime == "node":
                        report["isolatedRuntimeErrors"] = report["isolated"].pop("nodeErrorDetails", {})
                        if stdio:
                            report["isolatedStdio"] = report["isolated"].pop("nodeStdio", {})
                    if (work / "panic.txt").exists():
                        report["childPanic"] = (work / "panic.txt").read_text(encoding="utf-8")
                except BaseException:
                    terminate(info.process, 3)
                    wait(info.process, 2000)
                    raise
                finally:
                    close(info.thread)
                    close(info.process)
            finally:
                if pipes:
                    pipes.close()
                destroy(attributes)
        report["outsidePreserved"] = outside.read_text() == "private-probe"
    finally:
        cleanup_errors = []
        if package_granted:
            try:
                command(["icacls", str(package_dir), "/remove:g", "*S-1-15-2-1", "/T", "/Q"])
            except Exception as error:
                cleanup_errors.append(type(error).__name__)
        for array, count in [(groups, group_count.value), (capabilities, capability_count.value)]:
            if array:
                for i in range(count):
                    local_free(array[i])
                local_free(c.cast(array, c.c_void_p))
        if text:
            for target in granted:
                try:
                    command(["icacls", str(target), "/remove:g", "*" + text.value, "/T", "/Q"])
                except Exception as error:
                    cleanup_errors.append(type(error).__name__)
            local_free(c.cast(text, c.c_void_p))
        free_sid(sid)
        report["profileDeleted"] = delete_profile(name) >= 0
        report["cleanupErrors"] = cleanup_errors
        (directory / "receipt.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
    control, isolated = report["control"], report["isolated"]
    expected_control = dict(allowedRead=True, allowedWrite=True, outsideReadError=0,
                            outsideWriteError=0, packageReadError=0, networkAttemptCompleted=True, networkError=0)
    expected_isolated = dict(allowedRead=True, allowedWrite=True, outsideReadError=5,
                             outsideWriteError=5, packageReadError=5 if policy == "lpac" else 0,
                             networkAttemptCompleted=True, networkError=10013)
    if runtime == "node":
        for field in ("outsideReadError", "outsideWriteError"):
            expected_isolated[field] = isolated.get(field) if isolated.get(field) in ("EACCES", "EPERM") else "DENIAL_REQUIRED"
        expected_isolated["packageReadError"] = (isolated.get("packageReadError") if isolated.get("packageReadError") in ("EACCES", "EPERM") else "DENIAL_REQUIRED") if policy == "lpac" else 0
        expected_isolated["networkError"] = "EACCES"
    if control != expected_control or isolated != expected_isolated:
        raise AssertionError(f"Isolation candidate failed the declared boundary: {report}")
    if stdio:
        expected_stdio = {"input":"PUBLIC_STDIN 世界\n","stdoutBytes":len("PUBLIC_STDOUT 世界\n".encode()),"stderrBytes":len("PUBLIC_STDERR 世界\n".encode())}
        if report.get("controlStdio") != expected_stdio or report.get("isolatedStdio") != expected_stdio:
            raise AssertionError("Public Unicode stdin/output pipe receipt failed")
        if any(report["isolatedPipes"].get(k) != v for k,v in {"stdout":"PUBLIC_STDOUT 世界\n","stderr":"PUBLIC_STDERR 世界\n","stdoutTruncated":False,"stderrTruncated":False}.items()):
            raise AssertionError("Public stdout/stderr capture failed")
    if not (report["tokenIsAppContainer"]
            and (policy != "lpac" or report["tokenIsLessPrivilegedAppContainer"] is True)
            and report["outsidePreserved"]
            and report["profileDeleted"] and not cleanup_errors):
        raise AssertionError(f"Isolation identity or cleanup failed: {report}")
    print("PASS: granted file access, denied outside read/write and loopback; disposable profile removed")

def paired_lpac(directory, runtime="rust", node_entry="file", stdio=False):
    trials = {}
    errors = {}
    for policy in ("appcontainer", "lpac"):
        trial = directory / policy
        trial.mkdir()
        try:
            probe(trial, policy, True, runtime, node_entry, stdio)
        except AssertionError as error:
            # The ordinary AppContainer is a positive resource/network control,
            # not an expected denied-mode pass. Preserve every candidate verdict.
            errors[policy] = str(error)
        trials[policy] = json.loads((trial / "receipt.json").read_text())
    ordinary, candidate = trials["appcontainer"], trials["lpac"]
    file_denials = (5,) if runtime == "rust" else ("EACCES", "EPERM")
    network_denial = 10013 if runtime == "rust" else "EACCES"
    controls_passed = all(
        r["tokenIsAppContainer"] and r["outsidePreserved"] and r["profileDeleted"]
        and not r["cleanupErrors"] and r["exitCode"] == 0
        and r["networkCapabilities"] == [] and r["resourceCapabilities"] == ["registryRead"]
        and r["control"]["networkAttemptCompleted"] and r["control"]["networkError"] == 0
        and r["control"]["packageReadError"] == 0
        and r["isolated"].get("allowedRead") and r["isolated"].get("allowedWrite")
        and r["isolated"].get("outsideReadError") in file_denials and r["isolated"].get("outsideWriteError") in file_denials
        for r in trials.values())
    opt_out = ordinary["isolated"].get("packageReadError") == 0 and candidate["isolated"].get("packageReadError") in file_denials
    network = (ordinary["isolated"].get("networkAttemptCompleted") and ordinary["isolated"].get("networkError") == 0
               and candidate["isolated"].get("networkAttemptCompleted") and candidate["isolated"].get("networkError") == network_denial)
    candidate_network = (candidate["control"].get("networkError") == 0
                         and candidate["isolated"].get("networkAttemptCompleted")
                         and candidate["isolated"].get("networkError") == network_denial)
    stdio_passed = None
    if stdio:
        expected_stdio = {"input":"PUBLIC_STDIN 世界\n","stdoutBytes":21,"stderrBytes":21}
        expected_pipes = {"stdout":"PUBLIC_STDOUT 世界\n","stderr":"PUBLIC_STDERR 世界\n","stdoutTruncated":False,"stderrTruncated":False}
        stdio_passed = all(r.get("controlStdio") == expected_stdio and r.get("isolatedStdio") == expected_stdio
                          and r.get("isolatedPipes") == expected_pipes for r in trials.values())
    result = {"diagnosticOnly": True, "runtime": runtime, "networkScope": "TCP IPv4 loopback only",
              "controlsPassed": controls_passed, "lpacOptOutBehaviorPassed": opt_out,
              "networkDenialPassed": bool(network), "candidateNetworkDenialPassed": bool(candidate_network),
              "ordinaryNetworkAllowObserved": ordinary["isolated"].get("networkError") == 0,
              "stdioPassed": stdio_passed, "trials": trials, "individualVerdicts": errors}
    (directory / "paired-receipt.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
    if not (controls_passed and opt_out and network and stdio_passed is not False):
        raise AssertionError("Paired LPAC control failed; inspect paired-receipt.json")
    print("PASS: paired AppContainer/LPAC resource opt-out and TCP IPv4 loopback denial. Diagnostic only; token query status retained.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--directory", required=True, type=Path)
    parser.add_argument("--policy", choices=("appcontainer", "lpac"), default="appcontainer")
    parser.add_argument("--registry-read", action="store_true",
                        help="Diagnostic LPAC registryRead runtime capability; no network capabilities")
    parser.add_argument("--paired-lpac", action="store_true",
                        help="Compare ordinary AppContainer and LPAC with identical registryRead grants")
    parser.add_argument("--runtime", choices=("rust", "node"), default="rust",
                        help="Copy a public Node fixture into owned output, or use the static Rust fixture")
    parser.add_argument("--node-entry", choices=("file", "eval", "preserve-file"), default="file")
    parser.add_argument("--stdio", action="store_true", help="Public Node Unicode inherited-pipe control")
    args = parser.parse_args()
    if os.name != "nt":
        parser.error("Windows only")
    if args.registry_read and args.policy != "lpac":
        parser.error("--registry-read requires --policy lpac")
    if args.stdio and args.runtime != "node":
        parser.error("--stdio currently requires --runtime node")
    directory = args.directory.resolve()
    if not directory.is_relative_to((ROOT / "output").resolve()):
        parser.error("Choose a fresh directory under this repository's output/")
    directory.mkdir(parents=True, exist_ok=False)
    if args.paired_lpac:
        paired_lpac(directory, args.runtime, args.node_entry, args.stdio)
    else:
        probe(directory, args.policy, args.registry_read, args.runtime, args.node_entry, args.stdio)


if __name__ == "__main__":
    main()
