"""WM-5: a fair rematch and a paper draft — CRN and CT trained on their authors' schedule (step plan, α on real progress,
last checkpoint, counts rounded at readout), a larger multi-cast observational split `obs2`, the headroom ladder
(floor ← privileged regression ← observable regression ← `zero`), and the benchmark paper (docs/worldmodel/wm5/GOAL.md).

Importing the package puts `tools/worldmodel` on `sys.path` (through `wm4` → `wm1`), so the modules here import
`wm_data`, `bench_v0`, `bench_v1`, the `wm1` harness, `wm2`, `wm3` and `wm4` whichever directory a script is started
from. WM-4 code is extended by importing and wrapping it, never by editing it (GOAL.md 2).
"""
import sys
from pathlib import Path

_PARENT = str(Path(__file__).resolve().parents[1])
if _PARENT not in sys.path:
    sys.path.insert(0, _PARENT)

import wm4  # noqa: E402,F401
