"""M1b-obs2 (docs/worldmodel/wm5/GOAL.md 9.1): WM-2's M1b-act recipe trained from scratch on the obs2 split, exactly as
WM-4's M1b-obs (`wm4/m1_obs.py`) on obs.

`register()` adds `obs2_v19` to `wm1.common.DATASETS` (no edit of `wm1/common.py`): train = the observed train episodes,
val = the observed val episodes (selection on val factual T2 24 h), test = the observed test episodes (factual T2).
Every process that loads an M1b-obs2 run, or reads `obs2_v19` (CRN / CT), calls `register()` first. `train` calls
`m1_tf.main` with WM-4's flags (`wm4.m1_obs.FLAGS`) and its stall guard; `final` calls `m1_gru.main(["final", ...])`.

usage (DGX):
  python3 tools/worldmodel/wm5/m1_obs2.py train --out RUN --seed S --max-steps 8000 --max-seconds N
  python3 tools/worldmodel/wm5/m1_obs2.py final --run RUN --out DIR
"""
from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import wm5  # noqa: F401
from wm1 import common as c
from wm4 import m1_obs

MANIFEST = c.REPO / "docs/worldmodel/wm5/obs2_manifest.json"
FLAGS = m1_obs.FLAGS


def dataset() -> "c.Dataset":
    return c.Dataset("obs2_v19", MANIFEST, {"train": ("obs2_train", "train"), "val": ("obs2_val", "val"),
                                            "test": ("obs2_test", "test")},
                     ("train",), ("train",), ("val",), ("test",), None, None, None, ("T1", "T2", "T2_any"), True, None,
                     c.V17_EXTRA)


def register() -> None:
    if "obs2_v19" not in c.DATASETS:
        c.DATASETS["obs2_v19"] = dataset()


def main(argv=None) -> int:
    argv = list(sys.argv[1:] if argv is None else argv)
    register()
    m1_obs.register()
    if not argv or argv[0] not in ("train", "final"):
        raise SystemExit(__doc__)
    if argv[0] == "train":
        from wm1 import m1_tf
        m1_obs.guard()
        return m1_tf.main(["train", "--dataset", "obs2_v19"] + FLAGS + argv[1:])
    from wm1 import m1_gru as g
    return g.main(["final"] + argv[1:])


if __name__ == "__main__":
    raise SystemExit(main())
