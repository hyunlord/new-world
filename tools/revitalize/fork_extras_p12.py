#!/usr/bin/env python3
"""Pass 12 Part 6 (PASS12_GOAL.md 12.1): fork extras beside `fork_metrics.py`. numpy only.

For each variant and the factual, from the recorder (schema 5, `--every 60`):
- the drop in the home settlement's stored wood over the rain hours inside the burn window (18:00-07:00): for each
  hourly row r of the variant with `rain_ticks_left > 0` on the home slot (slot 0) and its minute in the burn window,
  wood[r - 1] - wood[r] (the hour's withdrawal; rows whose previous row had no rain are left out, so the soak at
  application is not counted), summed per episode; the factual's drop over the same rows beside it
  (a dry night's burn). Under v18's wet burn the two are equal up to dynamics; under v17 the variant's is ~0.
- the first-fission day (display day of the first row with `fissions_total >= 1`) per episode, mean and the share of
  episodes whose day differs from the factual of the same replicate.

usage: fork_extras_p12.py --root DIR --scene game_v18 --seeds 12000..12032 --replicates 4
                          --variants gift_wood,rot_wood,rain,cold_snap,windfall,gift1 --out DIR
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

import numpy as np

REPO = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO / "tools/worldmodel"))
from wm_data import load_episode  # noqa: E402

TPD, OFFSET, EVERY = 1440, 480, 60
BURN = (1080, 420)


def load(path: Path) -> dict:
    ep = load_episode(path, arrays=["global.npy", "settlements.npy"])
    tick = ep.column("global.npy", "tick").astype(np.int64)
    fissions = ep.column("global.npy", "fissions_total")
    s = ep.arrays["settlements.npy"]
    wood = ep.column("settlements.npy", "wood_stock")[:, 0] if s.shape[1] else np.full(len(tick), np.nan)
    rain = ep.column("settlements.npy", "rain_ticks_left")[:, 0] if s.shape[1] else np.zeros(len(tick))
    hit = np.nonzero(fissions >= 1)[0]
    first = None if not len(hit) else int((int(tick[hit[0]]) + OFFSET) // TPD + 1)
    return {"tick": tick, "wood": wood, "rain": np.nan_to_num(rain, nan=0.0), "first": first}


def rain_rows(ep: dict) -> np.ndarray:
    minute = (ep["tick"] + OFFSET) % TPD
    burning = (minute >= BURN[0]) | (minute < BURN[1])
    rows = np.nonzero((ep["rain"] > 0) & burning)[0]
    rows = rows[rows >= 1]
    # Rows where the rain was already in force at the previous row: the soak at
    # application (a row whose previous row had no rain) is excluded.
    return rows[ep["rain"][rows - 1] > 0]


def drop(ep: dict, rows: np.ndarray) -> float:
    w = ep["wood"]
    d = w[rows - 1] - w[rows]
    return float(np.nansum(d))


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--root", required=True, type=Path)
    ap.add_argument("--scene", default="game_v18")
    ap.add_argument("--seeds", required=True)
    ap.add_argument("--replicates", type=int, required=True)
    ap.add_argument("--variants", required=True)
    ap.add_argument("--out", required=True, type=Path)
    args = ap.parse_args(argv)
    a, b = args.seeds.split("..")
    seeds = range(int(a), int(b))
    variants = [v for v in args.variants.split(",") if v]
    name = lambda s, e, k: args.root / f"{args.scene}_s{s}_{e}_r{k}"
    out = {"scene": args.scene, "seeds": [seeds.start, seeds.stop], "replicates": args.replicates, "variants": {}}
    factual = {(s, k): load(name(s, "none", k)) for s in seeds for k in range(args.replicates)}
    out["factual_first_fission_day_mean"] = float(np.mean([f["first"] for f in factual.values() if f["first"] is not None])) \
        if any(f["first"] is not None for f in factual.values()) else None
    for v in variants:
        drops, fdrops, firsts, differ, rain_hours = [], [], [], [], []
        for s in seeds:
            for k in range(args.replicates):
                ep, f = load(name(s, v, k)), factual[(s, k)]
                rows = rain_rows(ep)
                rain_hours.append(len(rows))
                if len(rows):
                    drops.append(drop(ep, rows))
                    fdrops.append(drop(f, rows))
                if ep["first"] is not None:
                    firsts.append(ep["first"])
                differ.append(ep["first"] != f["first"])
        out["variants"][v] = {"rain_hours_in_burn_window_mean": float(np.mean(rain_hours)),
                              "wood_drop_over_rain_hours_mean": float(np.mean(drops)) if drops else None,
                              "factual_drop_same_hours_mean": float(np.mean(fdrops)) if fdrops else None,
                              "first_fission_day_mean": float(np.mean(firsts)) if firsts else None,
                              "episodes_with_fission": len(firsts),
                              "share_first_fission_day_differs": float(np.mean(differ))}
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "fork_extras.json").write_text(json.dumps(out, indent=1, sort_keys=True) + "\n")
    fmt = lambda x: "—" if x is None else f"{x:.2f}"
    lines = ["# Pass 12 fork extras (12.1)", "",
             f"Scene `{args.scene}`, seeds {seeds.start}..{seeds.stop}, R = {args.replicates}. Wood drop = the home store's "
             "hourly decrease over the variant's rain hours inside the burn window (factual over the same rows beside it). "
             f"Factual mean first-fission display day {fmt(out['factual_first_fission_day_mean'])}. `fork_extras_p12.py`.", "",
             "| Variant | Rain hours in burn window | Wood drop over them | Factual drop, same hours | First-fission day mean (episodes) | Share with a different first-fission day |",
             "|---|---:|---:|---:|---:|---:|"]
    for v, m in out["variants"].items():
        lines.append(f"| {v} | {fmt(m['rain_hours_in_burn_window_mean'])} | {fmt(m['wood_drop_over_rain_hours_mean'])} | "
                     f"{fmt(m['factual_drop_same_hours_mean'])} | {fmt(m['first_fission_day_mean'])} ({m['episodes_with_fission']}) | "
                     f"{fmt(m['share_first_fission_day_differs'])} |")
    (args.out / "fork_extras.md").write_text("\n".join(lines) + "\n")
    print("\n".join(lines))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
