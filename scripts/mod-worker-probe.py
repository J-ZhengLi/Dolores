"""Read only the owned sentinel passed by measure-mod-runtime.ps1."""
from pathlib import Path
import sys

print("worker_unmediated_read=" + Path(sys.argv[1]).read_text())
