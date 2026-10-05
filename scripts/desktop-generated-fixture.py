"""Render a bounded model-generated view specification, never generated Python.

Only the frozen PulseBoard corpus schema is accepted. No network or file actions.
"""
import argparse,json
from pathlib import Path
import tkinter as tk
parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--directory',type=Path,required=True);args=parser.parse_args();directory=args.directory.resolve();root=Path(__file__).resolve().parents[1]
if not args.directory.is_absolute() or not directory.is_relative_to(root/'output'):raise SystemExit('Disposable output/ directory required.')
with (directory/'generated-view.json').open('rb') as source:raw=source.read(4097)
if len(raw)>4096:raise SystemExit('Generated view exceeds the fixture byte limit.')
spec=json.loads(raw)
if spec!={'title':'PulseBoard','status':'Ready','cards':[{'label':'Completed','value':42},{'label':'In progress','value':7},{'label':'Blocked','value':3}]}:raise SystemExit('Generated view does not satisfy frozen criteria.')
window=tk.Tk();window.title('Dolores generated PulseBoard');window.geometry('760x360+80+100');window.configure(bg='#17191f')
tk.Label(window,text=spec['title'],font=('Arial',28),fg='#e7e9ef',bg='#17191f').place(x=30,y=30)
tk.Label(window,text=spec['status'],font=('Arial',18),fg='#a2dfb5',bg='#17191f').place(x=30,y=90)
for i,card in enumerate(spec['cards']):
 frame=tk.Frame(window,bg='#292e3e');frame.place(x=30+i*240,y=145,width=220,height=150)
 tk.Label(frame,text=card['label'],font=('Arial',18),fg='#c8cad1',bg='#292e3e').pack(pady=(18,5))
 tk.Label(frame,text=str(card['value']),font=('Arial',30),fg='#a1b1ff',bg='#292e3e').pack()
window.after(200,lambda:window.focus_force());window.after(180000,window.destroy);window.mainloop()
