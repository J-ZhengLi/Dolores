"""Bounded EH00 AppContainer feasibility probe, not production isolation.

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
import subprocess
import sys
import uuid

ROOT = Path(__file__).resolve().parents[1]


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


def probe(directory):
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
    outside = directory / "outside.txt"
    outside.write_text("private-probe", encoding="utf-8")
    (work / "input.txt").write_text("public-probe", encoding="utf-8")
    executable = bin_dir / "probe.exe"
    command(["rustc", "--edition=2021", "-C", "target-feature=+crt-static", "-O",
             str(ROOT / "scripts/fixtures/harness-isolation-child.rs"), "-o", str(executable)])
    hr = create_profile(name, "Dolores EH00 probe", "Disposable feasibility probe", None, 0, c.byref(sid))
    if hr < 0:
        raise OSError(f"CreateAppContainerProfile HRESULT 0x{hr & 0xffffffff:08x}")
    report = {"diagnosticOnly": True, "networkScope": "loopback only", "profileDeleted": False}
    granted = []
    try:
        checked(sid_text(sid, c.byref(text)))
        principal = "*" + text.value
        # The base and every ACL target are freshly created children of output.
        for target, grant in [(bin_dir, "(OI)(CI)RX"), (work, "(OI)(CI)M")]:
            command(["icacls", str(target), "/grant", f"{principal}:{grant}", "/T", "/Q"])
            granted.append(target)
        with socket.socket() as listener:
            listener.bind(("127.0.0.1", 0))
            listener.listen(4)
            address = f"127.0.0.1:{listener.getsockname()[1]}"
            args = [str(executable), str(work), str(outside), address]
            command(args)
            report["control"] = json.loads((work / "receipt.json").read_text())
            outside.write_text("private-probe", encoding="utf-8")
            (work / "receipt.json").unlink()
            size = c.c_size_t()
            initialize(None, 1, 0, c.byref(size))
            attributes = c.create_string_buffer(size.value)
            checked(initialize(attributes, 1, 0, c.byref(size)))
            try:
                caps = Capabilities(sid, None, 0, 0)
                checked(update(attributes, 0, 0x20009, c.byref(caps), c.sizeof(caps), None, None))
                startup = StartupEx()
                startup.startup.cb = c.sizeof(startup)
                startup.attributes = c.cast(attributes, c.c_void_p)
                info = Process()
                line = c.create_unicode_buffer(subprocess.list2cmdline(args))
                # No inherited handles; no network capabilities; hidden fixture.
                checked(create(str(executable), line, None, None, False, 0x08080004,
                               None, str(work), c.byref(startup), c.byref(info)))
                try:
                    token, is_container, used = w.HANDLE(), w.DWORD(), w.DWORD()
                    checked(open_token(info.process, 8, c.byref(token)))
                    try:
                        checked(token_info(token, 29, c.byref(is_container), c.sizeof(is_container), c.byref(used)))
                        report["tokenIsAppContainer"] = is_container.value == 1
                    finally:
                        close(token)
                    resume = api(kernel, "ResumeThread", w.DWORD, [w.HANDLE])
                    if resume(info.thread) == 0xffffffff:
                        raise c.WinError(c.get_last_error())
                    if wait(info.process, 10000) != 0:
                        checked(terminate(info.process, 3))
                        wait(info.process, 2000)
                        raise TimeoutError("AppContainer fixture exceeded ten seconds")
                    code = w.DWORD()
                    checked(exit_code(info.process, c.byref(code)))
                    report["exitCode"] = code.value
                    report["isolated"] = json.loads((work / "receipt.json").read_text())
                except BaseException:
                    terminate(info.process, 3)
                    wait(info.process, 2000)
                    raise
                finally:
                    close(info.thread)
                    close(info.process)
            finally:
                destroy(attributes)
        report["outsidePreserved"] = outside.read_text() == "private-probe"
    finally:
        cleanup_errors = []
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
                            outsideWriteError=0, networkError=0)
    expected_isolated = dict(allowedRead=True, allowedWrite=True, outsideReadError=5,
                             outsideWriteError=5, networkError=10013)
    if control != expected_control or isolated != expected_isolated:
        raise AssertionError(f"Isolation candidate failed the declared boundary: {report}")
    if not (report["tokenIsAppContainer"] and report["outsidePreserved"]
            and report["profileDeleted"] and not cleanup_errors):
        raise AssertionError(f"Isolation identity or cleanup failed: {report}")
    print("PASS: granted file access, denied outside read/write and loopback; disposable profile removed")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--directory", required=True, type=Path)
    args = parser.parse_args()
    if os.name != "nt":
        parser.error("Windows only")
    directory = args.directory.resolve()
    if not directory.is_relative_to((ROOT / "output").resolve()):
        parser.error("Choose a fresh directory under this repository's output/")
    directory.mkdir(parents=True, exist_ok=False)
    probe(directory)


if __name__ == "__main__":
    main()
