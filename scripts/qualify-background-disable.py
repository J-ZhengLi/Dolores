"""Normal packaged Close after disabling background mode; public fixture only."""
import argparse,ctypes,json,subprocess,sys,time
from pathlib import Path
from desktop_test_support import NativeHost,ROOT
import desktop

p=argparse.ArgumentParser();p.add_argument('--directory',type=Path,required=True);p.add_argument('--initialize',action='store_true');a=p.parse_args();root=a.directory.resolve();assert root.is_relative_to(ROOT/'output')
if a.initialize:
    h=NativeHost(root);h.call('bootstrap');policy=h.call('backgroundPolicy');h.call('setBackgroundPolicy',enabled=False,revision=policy['revision']);h.close();sys.exit(0)
subprocess.run([sys.executable,__file__,'--directory',str(root),'--initialize'],check=True,capture_output=True)
record=root/'disable-owner.json';app=None
try:
    result=subprocess.run([sys.executable,str(ROOT/'scripts/desktop.py'),'launch','--data-directory',str(root/'data'),'--pid-file',str(record),'--replace-owned'],capture_output=True,text=True);assert not result.returncode,result.stderr
    identity=json.loads(record.read_text());pid=identity['pid'];hwnd=desktop.window_visible(pid)
    k=ctypes.windll.kernel32;k.OpenProcess.restype=ctypes.c_void_p;app=k.OpenProcess(0x1000|0x100000,False,pid)
    get=k.GetExitCodeProcess;get.argtypes=[ctypes.c_void_p,ctypes.POINTER(ctypes.c_uint)]
    user=ctypes.windll.user32;user.PostMessageW.argtypes=[ctypes.c_void_p,ctypes.c_uint,ctypes.c_size_t,ctypes.c_ssize_t]
    time.sleep(3);user.PostMessageW(hwnd,0x0010,0,0)
    deadline=time.monotonic()+15
    while time.monotonic()<deadline:
        code=ctypes.c_uint();assert get(app,ctypes.byref(code))
        if code.value!=259:break
        time.sleep(.1)
    assert code.value==0,hex(code.value)
    assert desktop.profile_window(root/'data') is None
    receipt={'backgroundDisabled':True,'normalCloseExitCode':code.value,'ownerReleased':True,'originalProfileUsed':False};(root/'disable-receipt.json').write_text(json.dumps(receipt,indent=2));print(json.dumps(receipt))
finally:
    if app:
        code=ctypes.c_uint();get(app,ctypes.byref(code))
        if code.value==259:desktop.process_identity(pid,terminate_expected=identity)
        k.CloseHandle.argtypes=[ctypes.c_void_p];k.CloseHandle(app)
