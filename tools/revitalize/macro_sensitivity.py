#!/usr/bin/env python3
"""Pass 11 Part 6.2: macro sensitivity of game_v17 to a small act of the god (PASS11_GOAL.md 12.2). numpy only.

usage: macro_sensitivity.py --root DIR --scene game_v17 --seeds 12000..12032 --variants rot_wood,gift_wood
                            --replicates 2 --out DIR

DIR holds 120-day schema-5 recordings `<scene>_s<seed>_<id>_r<k>` (no --agents) of the factual (`none`) and
each variant, forked at the same tick with replicates 0..R-1. Per episode the macro shape is: settlements alive
(>= 1 member, from settlements.npy) at the 08:00 row of days 60 / 90 / 118, the first-fission day (display day of
the first row with fissions_total >= 1, None without a fission) and total alive at day 118. Reported:
(a) noise only: the share of seeds whose shape differs between the factual's replicates 0 and 1, and the mean
|delta alive day 118|; (b) per variant: the share of (seed, replicate) pairs whose shape differs from the factual
of the same replicate, per seed the share of replicates that differ, and the mean |delta alive day 118|.
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

DAYS = (60, 90, 118)
TPD, OFFSET, EVERY = 1440, 480, 60


def row_of(day: int, minute: int) -> int:
    """Row of one-based display day `day` at minute-of-day `minute` (rows every 60 ticks)."""
    return ((day - 1) * TPD + minute - OFFSET) // EVERY


def shape_of(path: Path) -> dict:
    ep = load_episode(path, arrays=["global.npy", "settlements.npy"])
    members = ep.column("settlements.npy", "members") if ep.arrays["settlements.npy"].shape[1] else None
    alive_col = ep.column("global.npy", "alive")
    fissions = ep.column("global.npy", "fissions_total")
    tick = ep.column("global.npy", "tick").astype(np.int64)
    counts = [0 if members is None else int(np.sum(np.nan_to_num(members[row_of(d, 480)], nan=0.0) >= 1)) for d in DAYS]
    hit = np.nonzero(fissions >= 1)[0]
    first = None if not len(hit) else int((int(tick[hit[0]]) + OFFSET) // TPD + 1)
    return {"settlements": counts, "first_fission_day": first, "alive_118": float(alive_col[row_of(118, 480)])}


def differs(a: dict, b: dict) -> bool:
    return a["settlements"] != b["settlements"] or a["first_fission_day"] != b["first_fission_day"]


def build(root: Path, scene: str, seeds: range, variants: list[str], replicates: int) -> dict:
    shapes = {(seed, eid, k): shape_of(root / f"{scene}_s{seed}_{eid}_r{k}")
              for seed in seeds for eid in ("none", *variants) for k in range(replicates)}
    noise = [differs(shapes[(s, "none", 0)], shapes[(s, "none", 1)]) for s in seeds] if replicates >= 2 else []
    noise_alive = [abs(shapes[(s, "none", 0)]["alive_118"] - shapes[(s, "none", 1)]["alive_118"]) for s in seeds] if replicates >= 2 else []
    out = {"scene": scene, "seeds": [seeds.start, seeds.stop], "replicates": replicates, "days": list(DAYS),
           "noise": {"share_of_seeds_differing": float(np.mean(noise)) if noise else None,
                     "mean_abs_delta_alive_118": float(np.mean(noise_alive)) if noise_alive else None},
           "variants": {}, "per_seed": {}}
    for eid in variants:
        pairs = [differs(shapes[(s, eid, k)], shapes[(s, "none", k)]) for s in seeds for k in range(replicates)]
        per_seed = {str(s): float(np.mean([differs(shapes[(s, eid, k)], shapes[(s, "none", k)]) for k in range(replicates)]))
                    for s in seeds}
        alive = [abs(shapes[(s, eid, k)]["alive_118"] - shapes[(s, "none", k)]["alive_118"]) for s in seeds for k in range(replicates)]
        out["variants"][eid] = {"share_of_pairs_differing": float(np.mean(pairs)),
                                "share_of_seeds_differing_in_any_replicate": float(np.mean([v > 0 for v in per_seed.values()])),
                                "share_of_seeds_differing_in_every_replicate": float(np.mean([v == 1 for v in per_seed.values()])),
                                "mean_abs_delta_alive_118": float(np.mean(alive)), "per_seed": per_seed}
    out["per_seed"] = {str(s): {eid: [shapes[(s, eid, k)] for k in range(replicates)] for eid in ("none", *variants)} for s in seeds}
    return out


def markdown(r: dict) -> str:
    lines = ["# Pass 11 macro sensitivity (12.2)", "",
             f"Scene `{r['scene']}`, seeds {r['seeds'][0]}..{r['seeds'][1]}, R = {r['replicates']}, 120 days, fork at 28,800. "
             "Shape = settlements alive at days 60 / 90 / 118 and the first-fission day.", "",
             "| Comparison | Share differing | Mean abs delta alive day 118 |", "|---|---:|---:|",
             f"| (a) factual r0 vs r1 (noise only) | {r['noise']['share_of_seeds_differing']} | {r['noise']['mean_abs_delta_alive_118']} |"]
    for eid, v in r["variants"].items():
        lines.append(f"| (b) {eid} vs factual, same replicate (seed-replicate pairs) | {v['share_of_pairs_differing']:.3f} | "
                     f"{v['mean_abs_delta_alive_118']:.3f} |")
    lines += ["", "Per variant: share of seeds differing in at least one / in every replicate: " +
              ", ".join(f"{eid} {v['share_of_seeds_differing_in_any_replicate']:.3f} / {v['share_of_seeds_differing_in_every_replicate']:.3f}"
                        for eid, v in r["variants"].items())]
    return "\n".join(lines) + "\n"


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--root", required=True, type=Path)
    ap.add_argument("--scene", default="game_v17")
    ap.add_argument("--seeds", required=True)
    ap.add_argument("--variants", default="rot_wood,gift_wood")
    ap.add_argument("--replicates", type=int, default=2)
    ap.add_argument("--out", required=True, type=Path)
    args = ap.parse_args(argv)
    a, b = args.seeds.split("..")
    report = build(args.root, args.scene, range(int(a), int(b)), [v for v in args.variants.split(",") if v], args.replicates)
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "macro.json").write_text(json.dumps(report, indent=1, sort_keys=True) + "\n")
    (args.out / "macro.md").write_text(markdown(report))
    print(markdown(report))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
