"""WM-3: audit of the one-step response and paired fine-tuning on the dataset-v2 families (docs/worldmodel/wm3/GOAL.md).

Importing the package puts `tools/worldmodel` on `sys.path` (through `wm1`), so the modules here import `wm_data`,
`bench_v0`, `bench_v1`, the `wm1` harness and `wm2` whichever directory a script is started from.
"""
import sys
from pathlib import Path

_PARENT = str(Path(__file__).resolve().parents[1])
if _PARENT not in sys.path:
    sys.path.insert(0, _PARENT)

import wm1  # noqa: E402,F401
