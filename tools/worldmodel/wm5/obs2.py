"""The obs2 split of WM-5 (docs/worldmodel/wm5/GOAL.md 6.2): bookkeeping around the frozen recorder. numpy only.

Roles (GOAL.md 4.2): train 47000–48199, val 48200–48399, test 48400–48639 (`game_v19`).

usage (DGX):
  obs2.py verify --dir DIR --expect N --out JSON        every episode passes wm_data.verify_episode, 2,881 rows
"""
from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import wm5  # noqa: F401
from wm4 import obs_split as os4

SCENE = "game_v19"
ROLES = {"train": (47000, 48200), "val": (48200, 48400), "test": (48400, 48640)}


def role_of(seed: int) -> str:
    for role, (lo, hi) in ROLES.items():
        if lo <= seed < hi:
            return role
    raise ValueError(f"seed {seed} outside the obs2 roles")


def seeds(role: str | None = None) -> list[int]:
    if role is None:
        return list(range(ROLES["train"][0], ROLES["test"][1]))
    return list(range(*ROLES[role]))


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("verify"); p.add_argument("--dir", required=True); p.add_argument("--expect", type=int, required=True)
    p.add_argument("--out", required=True)
    a = ap.parse_args(argv)
    t0 = time.time()
    out = os4.verify(Path(a.dir), a.expect)
    out["seconds"] = time.time() - t0
    Path(a.out).parent.mkdir(parents=True, exist_ok=True)
    Path(a.out).write_text(json.dumps(out, indent=1, sort_keys=True) + "\n")
    print(json.dumps(out)[:2000])
    return 0 if out.get("all_ok", True) else 1


if __name__ == "__main__":
    raise SystemExit(main())
