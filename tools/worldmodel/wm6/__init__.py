"""WM-6: is the headroom learnable? A learning curve of nonlinear direct effect models (MLP, GRU, GBM; observable and
privileged) over five times the counterfactual data (v2x: game_v19 50000–51599, replicate 0), then the paper's
conclusion (docs/worldmodel/wm6/GOAL.md).

Importing the package puts `tools/worldmodel` on `sys.path` (through `wm5` → `wm4` → `wm1`). WM-2 … WM-5 code is used
by importing and wrapping it, never by editing it (GOAL.md 2).
"""
import sys
from pathlib import Path

_PARENT = str(Path(__file__).resolve().parents[1])
if _PARENT not in sys.path:
    sys.path.insert(0, _PARENT)

import wm5  # noqa: E402,F401
