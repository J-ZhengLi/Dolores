"""Disposable visible native control fixture for selected-window observation tests.

No credentials, network, user files or persisted state. Closes after five minutes.
"""
import tkinter as tk

window = tk.Tk()
window.title('Dolores observation fixture')
window.geometry('640x360')
window.configure(bg='#f3f4f6')
tk.Label(window, text='Observation test', font=('Arial', 24), bg='#f3f4f6').pack(pady=20)
tk.Label(window, text='Status: Ready', font=('Arial', 16), bg='#f3f4f6').pack()
tk.Entry(window, font=('Arial', 16), width=24).pack(pady=16)
tk.Button(window, text='Save note', font=('Arial', 18), width=18).pack()
window.after(300000, window.destroy)
window.mainloop()
