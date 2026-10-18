#!/usr/bin/env python3
"""Schedules of the Pass 9 fork experiment (PASS9_GOAL.md section 8). stdlib only.

usage: pass9_fork_schedules.py OUT_DIR

Writes `OUT_DIR/main/s<seed>_<id>.json` for seeds 12000-12031 (dataset v1's
C_cf seeds) with dataset v1's five C_cf variants, unchanged (`gift_wood` and
`rot_wood` 60 at tick 29,400, `rain` 13 h at 33,720, `cold_snap` 2 nights at
35,160, `windfall` 20 at 31,920; `make_dataset.C_VARIANTS_V1`) plus the
one-unit gift (`gift1`: `gift_food` 1 at 29,400), and
`OUT_DIR/placebo/s<seed>_p0_<kind>.json` for seeds 12000-12007: every
intervention kind with amount 0 (the five variants' kinds at their ticks,
`gift_food` and `spoil_food` at 29,400).
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "worldmodel"))
from make_dataset import C_VARIANTS_V1  # noqa: E402

MAIN_SEEDS = range(12000, 12032)
PLACEBO_SEEDS = range(12000, 12008)
GIFT1 = {"tick": 29400, "kind": "gift_food", "amount": 1}
PLACEBO_EXTRA = {"gift_food": 29400, "spoil_food": 29400}


def row(spec: dict, amount: int | None = None) -> list[dict]:
    return [{"tick": spec["tick"], "kind": spec["kind"], "settlement": "home",
             "amount": spec["amount"] if amount is None else amount}]


def schedules() -> dict[str, dict[str, list[dict]]]:
    """{"main": {file name: rows}, "placebo": {file name: rows}}."""
    main, placebo = {}, {}
    for seed in MAIN_SEEDS:
        for variant, spec in C_VARIANTS_V1.items():
            main[f"s{seed}_{variant}.json"] = row(spec)
        main[f"s{seed}_gift1.json"] = row(GIFT1)
    kinds = {spec["kind"]: spec["tick"] for spec in C_VARIANTS_V1.values()} | PLACEBO_EXTRA
    for seed in PLACEBO_SEEDS:
        for kind, tick in sorted(kinds.items()):
            placebo[f"s{seed}_p0_{kind}.json"] = row({"tick": tick, "kind": kind, "amount": 0})
    return {"main": main, "placebo": placebo}


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print(__doc__.strip(), file=sys.stderr)
        return 2
    out = Path(argv[1])
    for group, files in schedules().items():
        (out / group).mkdir(parents=True, exist_ok=True)
        for name, rows in files.items():
            (out / group / name).write_text(json.dumps(rows) + "\n")
        print(f"{group}: {len(files)} schedules")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
