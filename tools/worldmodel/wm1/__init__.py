"""WM-1: the first learned world models on dataset v0 (docs/worldmodel/wm1/GOAL.md).

Importing the package puts `tools/worldmodel` on `sys.path`, so the modules here can import Pass 7's
`wm_data` and `bench_v0` (read-only) whichever directory a script is started from.
"""
import sys
from pathlib import Path

_PARENT = str(Path(__file__).resolve().parents[1])
if _PARENT not in sys.path:
    sys.path.insert(0, _PARENT)
