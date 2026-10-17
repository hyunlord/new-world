"""WM-4: a confounded observational split of game_v19, observational baselines, CRN and the Causal Transformer on both
splits, and WM-3's open ablation and hazard items (docs/worldmodel/wm4/GOAL.md).

Importing the package puts `tools/worldmodel` on `sys.path` (through `wm1`), so the modules here import `wm_data`,
`bench_v0`, `bench_v1`, the `wm1` harness, `wm2` and `wm3` whichever directory a script is started from.
"""
import sys
from pathlib import Path

_PARENT = str(Path(__file__).resolve().parents[1])
if _PARENT not in sys.path:
    sys.path.insert(0, _PARENT)

import wm1  # noqa: E402,F401
