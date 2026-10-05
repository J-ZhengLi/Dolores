"""Owned, disposable native form for desktop input and recovery qualification.

Only an output/ directory is accepted. The fixture publishes its own control
state, not another application's UI. No network or user data; exits in five minutes.
"""
import argparse
import ctypes
import json
from pathlib import Path
import tkinter as tk

parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--directory',type=Path,required=True)
args=parser.parse_args()
root=Path(__file__).resolve().parents[1]
directory=args.directory.resolve()
if not directory.is_relative_to(root/'output') or not directory.is_dir():
    raise SystemExit('Use an existing disposable directory under output/.')
try: ctypes.windll.user32.SetProcessDpiAwarenessContext(ctypes.c_void_p(-4))
except (AttributeError,OSError): pass
window=tk.Tk();window.title('Dolores local form fixture');window.geometry('640x360+70+90')
window.configure(bg='#f3f4f6');status=tk.StringVar(value='Ready');saved=[];clicks=0
tk.Label(window,text='Local note',font=('Arial',24),bg='#f3f4f6').place(x=40,y=20)
tk.Label(window,textvariable=status,font=('Arial',16),bg='#f3f4f6').place(x=40,y=65)
entry=tk.Entry(window,font=('Arial',16));entry.place(x=40,y=110,width=420,height=40)
def save():
    global clicks
    clicks+=1;saved.append(entry.get());status.set('Saved: '+entry.get());publish()
button=tk.Button(window,text='Save note',font=('Arial',18),command=save);button.place(x=40,y=180,width=230,height=48)
entry.bind('<Return>',lambda _:save())
def publish():
    data={'text':entry.get(),'saved':saved,'saveClicks':clicks,'status':status.get(),
          'entry':{'x':entry.winfo_rootx()+100,'y':entry.winfo_rooty()+20},
          'button':{'x':button.winfo_rootx()+100,'y':button.winfo_rooty()+24}}
    temporary=directory/'form-state.tmp';temporary.write_text(json.dumps(data),encoding='utf-8');temporary.replace(directory/'form-state.json')
entry.bind('<KeyRelease>',lambda _:publish())
last=None
def tick():
    global last
    commands=directory/'form-command.json'
    if commands.is_file():
        value=json.loads(commands.read_text())
        if value!=last:
            last=value
            if value.get('operation')=='move': window.geometry('640x360+120+140')
            elif value.get('operation')=='minimize':window.iconify()
            elif value.get('operation')=='restore':window.deiconify();window.lift();window.focus_force();entry.focus_force()
            elif value.get('operation')=='clear':entry.delete(0,tk.END);status.set('Ready');entry.focus_force()
            elif value.get('operation')=='close':window.destroy();return
    publish();window.after(50,tick)
window.after(200,lambda:entry.focus_force());window.after(300,tick)
window.after(300000,window.destroy);window.mainloop()
