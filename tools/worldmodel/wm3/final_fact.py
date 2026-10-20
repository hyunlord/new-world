"""Factual T2 of a WM-3 run on `v2_v19_fact` test (docs/worldmodel/wm3/GOAL.md 8, V6): `wm1.m1_gru.final` with
`--eval-dataset v2_v19_fact`, after importing `wm3.model` so `g.load_predictor` can build an `m1_tfh` run.

usage (DGX): python3 tools/worldmodel/wm3/final_fact.py --run RUN --out DIR
"""
from __future__ import annotations

import argparse
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import wm3  # noqa: F401
from wm1 import common as c
from wm1 import m1_gru as g
from wm3 import model  # noqa: F401  (registers m1_tfh in wm1.m1_gru)


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--root", default=str(c.DEFAULT_ROOT))
    ap.add_argument("--device", default="auto")
    a = ap.parse_args(argv)
    args = argparse.Namespace(run=a.run, out=a.out, root=a.root, device=a.device, force=False, ckpt=None,
                              eval_dataset="v2_v19_fact")
    return g.final(args)


if __name__ == "__main__":
    raise SystemExit(main())
