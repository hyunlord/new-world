#!/usr/bin/env python3
"""Pass 13 Part 5 (PASS13_GOAL.md 11.2, J7): the replicate pilot's first-fission decomposition, identification and
J7 verdict. numpy only.

Inputs: the pilot recordings (`<root>/<scene>/<scene>_s<seed>_none_r<k>`, schema 6, noise fork at tick 1), the
`diversity_p11.py pilot` output on replicate 0 (`diversity.json`) and `paths_p13.py` output on all replicates
(`paths.json`).

- First-fission day per world and replicate: the display day (one-based) of the smallest `depart_tick` >= 0 over
  the settlement slots (the first group that left and founded a daughter); a run without one is censored at day
  121 (counted). Decomposed as 5.4 does (`paths_p13.decompose`: within W, between B, path share PS), over every
  world and, as a check, over the worlds whose every replicate split.
- Identification (`game_v19`, replicate 0, worlds that split): Pearson r across worlds of the first-fission day
  with the camp's θ (hidden column `fission_threshold` of slot 0 at its first row > 0) and with early scarcity
  (mean of s at 06:00 over the camp's first seven eligible days: display day >= 11, not winter, members >= 28;
  s = clamp(1000 - food x 1000 / floor(members x 2.4 x 4), 0, 1000), the v19 scarcity term with adult need 2.4 a
  day and `scarcity_days` 4).
- J7 (`game_v19`): (a) on replicate 0, test split: eta2 >= 0.30 and R2_abs <= 0.70 for food_stock, wood_stock and
  alive; (b) PS (mean over days 60/90/118) of alive in [0.10, 0.50] and of food_stock in [0.10, 0.60], and the paired
  90 % interval of PS(alive) v19 - v18 below 0.

usage: pilot_p13.py --root DIR --scenes game_v16,game_v17,game_v18,game_v19 --diversity diversity.json
                    --paths paths.json --out DIR
"""
from __future__ import annotations

import argparse
import json
import math
from pathlib import Path

import numpy as np

import paths_p13 as pp

wm_data = pp.wm_data
TPD, OFFSET, EVERY = 1440, 480, 60
CENSOR = 121
NEED_PER_MEMBER, SCARCITY_DAYS, MIN_MEMBERS, MIN_DAY = 2.4, 4, 28, 11


def display_day_of_tick(tick: int) -> int:
    return (int(tick) + OFFSET) // TPD + 1


def first_fission_day(ep) -> int | None:
    names = [f["name"] for f in ep.meta["fields"]["settlements.npy"]]
    if "depart_tick" not in names or ep.arrays["settlements.npy"].shape[1] == 0:
        return None
    dep = ep.column("settlements.npy", "depart_tick")
    vals = dep[np.isfinite(dep) & (dep >= 0)]
    return None if vals.size == 0 else display_day_of_tick(int(vals.min()))


def camp_theta(ep) -> float | None:
    if ep.arrays["settlements.npy"].shape[1] == 0:
        return None
    th = ep.column("settlements.npy", "fission_threshold")[:, 0]
    ok = np.isfinite(th) & (th > 0)
    return float(th[ok][0]) if ok.any() else None


def early_scarcity(ep) -> float | None:
    if ep.arrays["settlements.npy"].shape[1] == 0:
        return None
    members = ep.column("settlements.npy", "members")[:, 0]
    food = ep.column("settlements.npy", "food_stock")[:, 0]
    out = []
    for day in range(MIN_DAY, 120):
        if ((day - 1) // 7) % 4 == 3:
            continue
        row = ((day - 1) * TPD + 360 - OFFSET) // EVERY
        if row >= len(members) or not np.isfinite(members[row]) or members[row] < MIN_MEMBERS:
            continue
        need = max(1, math.floor(members[row] * NEED_PER_MEMBER * SCARCITY_DAYS))
        out.append(min(1000.0, max(0.0, 1000.0 - float(food[row]) * 1000.0 / need)))
        if len(out) == 7:
            break
    return float(np.mean(out)) if out else None


def pearson(xs, ys) -> float | None:
    x, y = np.asarray(xs, float), np.asarray(ys, float)
    if len(x) < 3 or x.std() == 0 or y.std() == 0:
        return None
    return float(np.corrcoef(x, y)[0, 1])


def scene_fission(root: Path, scene: str) -> dict:
    groups = pp.group_members(root / scene)
    seeds = sorted(groups)
    reps = sorted(set.intersection(*(set(groups[s]) for s in seeds)))
    days = np.full((len(seeds), len(reps)), np.nan)
    thetas, scarcity = {}, {}
    for i, seed in enumerate(seeds):
        for j, k in enumerate(reps):
            ep = wm_data.load_episode(groups[seed][k], arrays=["settlements.npy"], verify=False)
            d = first_fission_day(ep)
            days[i, j] = CENSOR if d is None else d
            if k == 0:
                thetas[seed], scarcity[seed] = camp_theta(ep), early_scarcity(ep)
    all_split = np.all(days < CENSOR, axis=1)
    out = {"worlds": len(seeds), "replicates": len(reps), "censored_runs": int(np.sum(days >= CENSOR)),
           "worlds_split_every_replicate": int(all_split.sum()), "worlds_split_no_replicate": int(np.all(days >= CENSOR, axis=1).sum()),
           "decompose_all": pp.decompose(days),
           "decompose_split_every_replicate": pp.decompose(days[all_split]) if all_split.sum() >= 2 else None,
           "mean_within_world_sd": float(np.sqrt(days.var(axis=1, ddof=1).mean())),
           "between_world_sd_of_means": float(days.mean(axis=1).std(ddof=1)),
           "first_day_r0": {str(s): (None if days[i, 0] >= CENSOR else int(days[i, 0])) for i, s in enumerate(seeds)}}
    split0 = [(s, days[i, 0]) for i, s in enumerate(seeds) if days[i, 0] < CENSOR]
    if any(v is not None for v in thetas.values()):
        pairs_t = [(thetas[s], d) for s, d in split0 if thetas[s] is not None]
        pairs_s = [(scarcity[s], d) for s, d in split0 if scarcity[s] is not None]
        out["identification"] = {
            "r_day_theta": pearson([a for a, _ in pairs_t], [b for _, b in pairs_t]), "n_theta": len(pairs_t),
            "r_day_early_scarcity": pearson([a for a, _ in pairs_s], [b for _, b in pairs_s]), "n_scarcity": len(pairs_s)}
    return out


def j7(diversity: dict, paths: dict, scene: str = "game_v19", ref: str = "game_v18") -> dict:
    sc = diversity["scenes"][scene]
    a = {n: {"eta2": sc["eta2"][n], "r2_abs": sc["r2_abs"][n],
             "passed": sc["eta2"][n] is not None and sc["eta2"][n] >= .30 and sc["r2_abs"][n] is not None and sc["r2_abs"][n] <= .70}
         for n in ("food_stock", "wood_stock", "alive")}
    ps = lambda n: paths["scenes"][scene]["vars"][n]["PS_mean"]
    diff = paths["diffs"].get(f"{scene} - {ref}", {}).get("vars", {}).get("alive")
    hi = diff["ci90"]["PS"]["mean"][1] if diff else None
    b = {"alive_in_0.10_0.50": {"PS": ps("alive"), "passed": ps("alive") is not None and .10 <= ps("alive") <= .50},
         "food_in_0.10_0.60": {"PS": ps("food_stock"), "passed": ps("food_stock") is not None and .10 <= ps("food_stock") <= .60},
         "alive_below_v18": {"diff": diff["PS_mean"] if diff else None, "ci90": diff["ci90"]["PS"]["mean"] if diff else None,
                             "passed": hi is not None and hi < 0}}
    return {"a": a, "b": b, "passed": all(c["passed"] for c in [*a.values(), *b.values()])}


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--root", type=Path, required=True)
    ap.add_argument("--scenes", default="game_v16,game_v17,game_v18,game_v19")
    ap.add_argument("--diversity", type=Path, required=True)
    ap.add_argument("--paths", type=Path, required=True)
    ap.add_argument("--out", type=Path, required=True)
    args = ap.parse_args(argv)
    scenes = [s for s in args.scenes.split(",") if s]
    report = {"first_fission": {s: scene_fission(args.root, s) for s in scenes}}
    report["j7"] = j7(json.loads(args.diversity.read_text()), json.loads(args.paths.read_text()))
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "pilot.json").write_text(json.dumps(report, indent=1, sort_keys=True) + "\n")
    f = lambda x: "—" if x is None or (isinstance(x, float) and x != x) else (f"{x:.3f}" if isinstance(x, float) else str(x))
    lines = ["# Pass 13 replicate pilot — first fission and J7", "",
             "| Scene | worlds × R | censored runs (no split by day 120) | worlds split in every replicate | W (within) | B (between) | PS first-fission day | within-world SD (d) | between-world SD (d) | PS (worlds split in every replicate) | r(day, θ) | r(day, early s) |",
             "|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---|---|"]
    for s, v in report["first_fission"].items():
        d, e, idn = v["decompose_all"], v["decompose_split_every_replicate"], v.get("identification") or {}
        lines.append(f"| {s} | {v['worlds']} × {v['replicates']} | {v['censored_runs']} | {v['worlds_split_every_replicate']} | {f(d['W'])} | {f(d['B'])} | "
                     f"{f(d['PS'])} | {f(v['mean_within_world_sd'])} | {f(v['between_world_sd_of_means'])} | {f(e['PS']) if e else '—'} | "
                     f"{f(idn.get('r_day_theta'))} (n {idn.get('n_theta', '—')}) | {f(idn.get('r_day_early_scarcity'))} (n {idn.get('n_scarcity', '—')}) |")
    j = report["j7"]
    lines += ["", f"J7 (a) replicate 0, test split: " + "; ".join(f"{n} η² {f(c['eta2'])} / R²_abs {f(c['r2_abs'])} {'PASS' if c['passed'] else 'FAIL'}" for n, c in j["a"].items()),
              f"J7 (b): {json.dumps(j['b'])}", f"J7: {'PASS' if j['passed'] else 'FAIL'}"]
    (args.out / "pilot.md").write_text("\n".join(lines) + "\n")
    print("\n".join(lines))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
