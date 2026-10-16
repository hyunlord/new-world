"""WM-2's one-step / lag diagnostic (`wm2.diag_effect_lag`) for WM-3 runs (docs/worldmodel/wm3/GOAL.md 10): the same
first validation families (`--worlds 40`), after importing `wm3.model` so `g.load_predictor` can build an `m1_tfh` run.

usage (DGX): python3 tools/worldmodel/wm3/diag_lag.py --run RUN [RUN ...] --worlds 40 --out JSON
"""
from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import wm3  # noqa: F401
from wm3 import model  # noqa: F401  (registers m1_tfh in wm1.m1_gru)
from wm2 import diag_effect_lag

if __name__ == "__main__":
    raise SystemExit(diag_effect_lag.main())
