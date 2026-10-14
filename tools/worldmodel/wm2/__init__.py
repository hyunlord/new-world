"""WM-2: dataset v2 on `game_v19`, the noise floor, intervention effects and memory (docs/worldmodel/wm2/GOAL.md).

Importing the package puts `tools/worldmodel` on `sys.path` (through `wm1`), so the modules here import `wm_data`,
`bench_v0`, `bench_v1` and the `wm1` harness whichever directory a script is started from.
"""
import sys
from pathlib import Path

_PARENT = str(Path(__file__).resolve().parents[1])
if _PARENT not in sys.path:
    sys.path.insert(0, _PARENT)

import wm1  # noqa: E402,F401
