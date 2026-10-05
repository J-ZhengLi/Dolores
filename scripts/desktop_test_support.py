"""C ABI test client for isolated, synthetic desktop qualification only."""
import ctypes
import json
import os
from pathlib import Path
import time

ROOT=Path(__file__).resolve().parents[1]
BUNDLE=ROOT/'apps/dolores_flutter/build/windows/x64/runner/Release'
def fixture_state(directory):
    for _ in range(50):
        try:return json.loads((Path(directory)/'form-state.json').read_text())
        except (PermissionError,FileNotFoundError,json.JSONDecodeError):time.sleep(.01)
    raise AssertionError('Owned fixture state unavailable after bounded wait')
def wait_for_fixture_focus(target,seconds=25):
    get=ctypes.windll.user32.GetForegroundWindow;get.restype=ctypes.c_void_p
    deadline=time.monotonic()+seconds
    while time.monotonic()<deadline:
        if get()==target['handle']:return
        time.sleep(.05)
    raise AssertionError('Select the disposable fixture window before native input; focus policy was not bypassed.')
class NativeHost:
    def __init__(self,directory,host=None):
        directory=Path(directory).resolve()
        if not directory.is_relative_to(ROOT/'output'):raise ValueError('Disposable output/ directory required')
        os.environ['DOLORES_DATA_DIR']=str(directory/'data');os.environ['DOLORES_GLOBAL_SKILLS_DIR']=str(directory/'skills')
        self.loader=os.add_dll_directory(str(BUNDLE));self.native=ctypes.CDLL(str((host or BUNDLE)/'dolores_flutter_bridge.dll'))
        self.native.dolores_call.argtypes=[ctypes.c_void_p,ctypes.c_size_t];self.native.dolores_call.restype=ctypes.c_void_p;self.native.dolores_free.argtypes=[ctypes.c_void_p]
    def envelope(self,command,**fields):
        data=json.dumps({'command':command,**fields}).encode();buffer=ctypes.create_string_buffer(data);pointer=self.native.dolores_call(buffer,len(data))
        try:return json.loads(ctypes.string_at(pointer))
        finally:self.native.dolores_free(pointer)
    def call(self,command,**fields):
        result=self.envelope(command,**fields);assert result['ok'],result.get('error');return result.get('result')
    def finish(self,identity,callback=None,idle=None,seconds=15):
        events=[];deadline=time.monotonic()+seconds
        while time.monotonic()<deadline:
            for event in self.call('poll',id=identity):
                events.append(event)
                if callback:callback(event)
                if event['type']=='done':return event,events
            if idle:idle()
            time.sleep(.02)
        self.call('cancel',id=identity);raise AssertionError('Bounded fixture deadline exceeded')
    def close(self):self.call('shutdown');self.loader.close()
