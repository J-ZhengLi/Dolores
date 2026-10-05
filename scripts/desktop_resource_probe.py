"""Read-only Windows memory sampling of an owned host and its desktop helpers.

No other process names, paths, account information or screenshots are retained.
Sampling may miss very short-lived processes; values are observed lower bounds.
"""
import ctypes
from ctypes import wintypes as w
import os
import threading
import time

def cpu_seconds(pid):
    kernel=ctypes.windll.kernel32;kernel.OpenProcess.restype=w.HANDLE;kernel.CloseHandle.argtypes=[w.HANDLE]
    handle=kernel.OpenProcess(0x1000,False,pid)
    if not handle:raise RuntimeError('Owned preview exited during CPU sampling')
    try:
        values=[w.FILETIME() for _ in range(4)]
        fn=kernel.GetProcessTimes;fn.argtypes=[w.HANDLE]+[ctypes.POINTER(w.FILETIME)]*4
        if not fn(handle,*(ctypes.byref(v) for v in values)):raise OSError('Process time query failed')
        return sum((v.dwHighDateTime<<32)+v.dwLowDateTime for v in values[2:])/10_000_000
    finally:kernel.CloseHandle(handle)

class Entry(ctypes.Structure):
    _fields_=[('size',w.DWORD),('usage',w.DWORD),('pid',w.DWORD),('heap',ctypes.c_size_t),('module',w.DWORD),('threads',w.DWORD),('parent',w.DWORD),('priority',w.LONG),('flags',w.DWORD),('name',w.WCHAR*260)]
class Memory(ctypes.Structure):
    _fields_=[('size',w.DWORD),('faults',w.DWORD)]+[(n,ctypes.c_size_t) for n in ['peakWorking','working','peakPaged','paged','peakNonPaged','nonPaged','pagefile','peakPagefile','private']]

def memory(pid):
    kernel=ctypes.windll.kernel32;kernel.OpenProcess.restype=w.HANDLE;kernel.CloseHandle.argtypes=[w.HANDLE]
    handle=kernel.OpenProcess(0x1000|0x10,False,pid)
    if not handle:return None
    try:
        value=Memory();value.size=ctypes.sizeof(value)
        get=ctypes.windll.psapi.GetProcessMemoryInfo;get.argtypes=[w.HANDLE,ctypes.POINTER(Memory),w.DWORD]
        return {'workingBytes':value.working,'privateBytes':value.private} if get(handle,ctypes.byref(value),value.size) else None
    finally:kernel.CloseHandle(handle)

def children(parent):
    kernel=ctypes.windll.kernel32;kernel.CreateToolhelp32Snapshot.restype=w.HANDLE;kernel.CloseHandle.argtypes=[w.HANDLE]
    snapshot=kernel.CreateToolhelp32Snapshot(2,0)
    if snapshot in (None,ctypes.c_void_p(-1).value):return []
    result=[];entry=Entry();entry.size=ctypes.sizeof(entry)
    first=kernel.Process32FirstW;next_=kernel.Process32NextW
    for fn in [first,next_]:fn.argtypes=[w.HANDLE,ctypes.POINTER(Entry)]
    try:
        found=first(snapshot,ctypes.byref(entry))
        while found:
            if entry.name.lower() in ['dolores-desktop-helper.exe','native-desktop-helper.exe']:
                result.append((entry.pid,entry.parent))
            found=next_(snapshot,ctypes.byref(entry))
    finally:kernel.CloseHandle(snapshot)
    owned={parent};changed=True
    while changed:
        before=len(owned);owned.update(pid for pid,ppid in result if ppid in owned);changed=len(owned)!=before
    return sorted(owned-{parent})

class ResourceProbe:
    def __init__(self,pid=None):
        if os.name!='nt':raise RuntimeError('Windows qualification only')
        self.pid=pid or os.getpid();self.end=threading.Event();self.thread=None;self.seen=set();self.peak_host={};self.peak_helpers={};self.peak_tree={};self.samples=0
    def sample(self):
        host=memory(self.pid) or {};helpers=children(self.pid);total={key:0 for key in ['workingBytes','privateBytes']}
        self.seen.update(helpers)
        for pid in helpers:
            value=memory(pid) or {}
            for key,n in value.items():total[key]+=n;self.peak_helpers[key]=max(self.peak_helpers.get(key,0),n)
        for key,n in host.items():self.peak_host[key]=max(self.peak_host.get(key,0),n)
        for key in total:self.peak_tree[key]=max(self.peak_tree.get(key,0),total[key]+host.get(key,0))
        self.samples+=1
    def start(self):
        def run():
            while not self.end.is_set():self.sample();self.end.wait(.025)
        self.thread=threading.Thread(target=run,daemon=True);self.thread.start();return self
    def finish(self):
        self.end.set();self.thread.join(2);self.sample()
        return {'sampleIntervalMs':25,'samples':self.samples,'hostPeak':self.peak_host,'oneHelperPeak':self.peak_helpers,'ownedTreePeak':self.peak_tree,'helpersObserved':len(self.seen),'helpersStillRunning':len(children(self.pid)),'limit':'Observed sample peaks; Python C ABI host is not Flutter desktop memory. Short-lived helpers can be missed.'}
