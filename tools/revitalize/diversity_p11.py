#!/usr/bin/env python3
"""Pass 11 Part 5: seed diversity with dynamics (PASS11_GOAL.md 11.2-11.4). numpy only.

Everything of `diversity_p10.py` (WM-1.1's exact definitions: eta2, R2_abs, R2_year, divergence day, spread,
calendar-family and persistence nMAE at 24 h and 168 h from every-hour starts; `calcore_wm11.py` unchanged), plus:

- dyn-eta2: for each episode e and variable j, a_e[r] = v_e[r] - mean(v_e[rows of display days 3..10]); dyn-eta2
  is calcore's eta2 on `a` over the rows of display days >= 11 (one-based display days, `calcore.display_day`).
  It is near 0 when seeds differ only by constant offsets and grows when histories diverge.
- settlements alive (>= 1 member, from `settlements.npy`) at the 08:00 row of days 30 / 60 / 90 / 118 per test
  seed, with the across-seed variance and distribution (one value per seed, not a per-row eta2).
- attribution: v16 -> v17_nofission (exposure v2 + rain + regions) and v17_nofission -> v17 (fission): delta
  eta2 and delta dyn-eta2 per variable.
- J7 (judged on game_v17): eta2 >= 0.30 and R2_abs <= 0.70 for food_stock, wood_stock and alive, and dyn-eta2 >=
  0.25 for alive and food_stock.

usage:
  diversity_p11.py check-v1 --root /Users/rexxa/work/worldsim-wm-data --out DIR     (as diversity_p10.py)
  diversity_p11.py pilot --root /Users/rexxa/work/worldsim-wm-data/p11/pilot --scenes game_v16,game_v17_nofission,game_v17
                         --train 22000..22096 --test 22096..22112 --out DIR
                         [--reference-root /Users/rexxa/work/worldsim-wm-data/p10/pilot --reference-scenes game_v15,game_v16
                          --reference-train 20000..20096 --reference-test 20096..20112]
"""
from __future__ import annotations

import argparse
import json
import time
from pathlib import Path

import numpy as np

import diversity_p10 as d10

cc, bv, wm_data = d10.cc, d10.bv, d10.wm_data
NAMES = d10.NAMES
J7_VARS = ("food_stock", "wood_stock", "alive")
J7_DYN_VARS = ("alive", "food_stock")
SETTLEMENT_DAYS = (30, 60, 90, 118)


def dyn_eta2(v: np.ndarray) -> np.ndarray:
    """dyn-eta2 of v[E, T, J] (see the module docstring)."""
    days = cc.display_day(np.arange(v.shape[1]))
    early = (days >= 3) & (days <= 10)
    if not early.any():
        raise SystemExit("no rows of display days 3..10")
    a = v - v[:, early, :].mean(axis=1, keepdims=True)
    first = int(np.argmax(days >= 11))
    return cc.eta2(a, first)


def settlements_alive(path: Path) -> list[int]:
    """Settlements with >= 1 member at the 08:00 row of each of SETTLEMENT_DAYS."""
    ep = wm_data.load_episode(path, arrays=["global.npy", "settlements.npy"])
    members = ep.column("settlements.npy", "members") if ep.arrays["settlements.npy"].shape[1] else None
    out = []
    for day in SETTLEMENT_DAYS:
        r = cc.row_of(day, 480)
        out.append(0 if members is None else int(np.sum(np.nan_to_num(members[r], nan=0.0) >= 1)))
    return out


def scene_block(root: Path, scene: str, train_seeds: range, test_seeds: range) -> dict:
    train = d10.load_dir(root, scene, train_seeds, "train")
    test = d10.load_dir(root, scene, test_seeds, "test")
    d = d10.evaluate(train, test)
    d["dyn_eta2"] = d10.per(dyn_eta2(d10.stack(test)))
    everyone = train + test
    d["deaths_mean_by_day120"] = {k: float(np.mean([e.deaths_by_cause.get(k, 0.0) for e in everyone]))
                                  for k in ("deaths_starvation", "deaths_dehydration", "deaths_combat", "deaths_old_age",
                                            "deaths_exposure")}
    d["schema"] = sorted({e.schema for e in everyone})
    counts = {str(seed): settlements_alive(root / scene / f"{scene}_s{seed}_none") for seed in test_seeds}
    arr = np.array(list(counts.values()), dtype=np.float64)
    d["settlements_alive"] = {
        "days": list(SETTLEMENT_DAYS), "per_test_seed": counts,
        "variance": [float(x) for x in arr.var(axis=0)], "mean": [float(x) for x in arr.mean(axis=0)],
        "distribution": [{str(k): int(v) for k, v in zip(*np.unique(arr[:, i], return_counts=True))} for i in range(arr.shape[1])],
    }
    return d


def j7(d: dict) -> dict:
    out = {}
    for name in J7_VARS:
        out[name] = {"eta2": d["eta2"][name], "r2_abs": d["r2_abs"][name],
                     "passed": (d["eta2"][name] or 0) >= .30 and d["r2_abs"][name] is not None and d["r2_abs"][name] <= .70}
    for name in J7_DYN_VARS:
        out[f"dyn_{name}"] = {"dyn_eta2": d["dyn_eta2"][name], "passed": (d["dyn_eta2"][name] or 0) >= .25}
    return {"passed": all(v["passed"] for v in out.values()), "clauses": out}


def pilot(args) -> int:
    t0 = time.time()
    out = {"vars": list(NAMES), "scenes": {}, "reference": {}, "j7_vars": list(J7_VARS), "j7_dyn_vars": list(J7_DYN_VARS),
           "root": args.root, "train": args.train, "test": args.test}
    for scene in args.scenes.split(","):
        d = scene_block(Path(args.root), scene, d10.parse_range(args.train), d10.parse_range(args.test))
        d["j7"] = j7(d)
        out["scenes"][scene] = d
        print(scene, "J7", "PASS" if d["j7"]["passed"] else "FAIL",
              {k: v for k, v in d["j7"]["clauses"].items()}, f"{time.time() - t0:.0f}s", flush=True)
    if args.reference_root:
        for scene in args.reference_scenes.split(","):
            d = scene_block(Path(args.reference_root), scene, d10.parse_range(args.reference_train),
                            d10.parse_range(args.reference_test))
            out["reference"][scene] = d
            print("reference", scene, "dyn-eta2", {k: d10.fmt(d["dyn_eta2"][k]) for k in NAMES}, flush=True)
    pairs = [("exposure v2 + rain + regions", "game_v16", "game_v17_nofission"),
             ("fission", "game_v17_nofission", "game_v17")]
    out["attribution"] = {}
    for label, a, b in pairs:
        if a in out["scenes"] and b in out["scenes"]:
            out["attribution"][label] = {
                metric: {n: (None if out["scenes"][b][metric][n] is None or out["scenes"][a][metric][n] is None
                             else out["scenes"][b][metric][n] - out["scenes"][a][metric][n]) for n in NAMES}
                for metric in ("eta2", "dyn_eta2", "r2_abs")}
    out["seconds"] = time.time() - t0
    target = Path(args.out)
    target.mkdir(parents=True, exist_ok=True)
    (target / "diversity.json").write_text(json.dumps(out, indent=1, sort_keys=True) + "\n")
    (target / "diversity.md").write_text(markdown(out))
    print(f"done {out['seconds']:.0f}s -> {target}")
    return 0


def markdown(out: dict) -> str:
    fmt = d10.fmt
    blocks = {**{f"{k} (Pass 10 pilot)": v for k, v in out["reference"].items()}, **out["scenes"]}
    lines = ["# Pass 11 diversity pilot", "",
             f"Train {out['train']}, test {out['test']} (fresh seeds; `diversity_p11.py`). eta2 / dyn-eta2 / R2_abs on the "
             "test split; the reference rows re-score Pass 10's pilot recordings read-only.", "",
             "| Variable | " + " | ".join(blocks) + " |", "|---|" + "---|" * len(blocks)]
    for n in NAMES:
        cells = [f"{fmt(b['eta2'][n])} / {fmt(b['dyn_eta2'][n])} / {fmt(b['r2_abs'][n])}" for b in blocks.values()]
        lines.append(f"| {n} | " + " | ".join(cells) + " |")
    lines += ["", "## T2 nMAE at 24 h -> 168 h (test split std, every-hour starts)", ""]
    for scene, b in blocks.items():
        for n in ("food_stock", "wood_stock", "alive"):
            t24, t168 = b["t2"]["24"], b["t2"]["168"]
            lines.append(f"- {scene} {n}: persistence {fmt(t24['persistence'][n])} -> {fmt(t168['persistence'][n])}, "
                         f"apfit_abs {fmt(t24['apfit_abs'][n])} -> {fmt(t168['apfit_abs'][n])}, cal_abs {fmt(t168['cal_abs'][n])}, "
                         f"an5 {fmt(t168['an5'][n])}")
    lines += ["", "## Settlements alive at days 30 / 60 / 90 / 118 (test seeds)", "",
              "| Scene | Mean | Variance | Distribution (count: seeds) |", "|---|---|---|---|"]
    for scene, b in blocks.items():
        s = b["settlements_alive"]
        lines.append(f"| {scene} | {[round(x, 3) for x in s['mean']]} | {[round(x, 3) for x in s['variance']]} | {s['distribution']} |")
    lines += ["", "## Attribution (delta eta2 / delta dyn-eta2)", ""]
    for label, block in out["attribution"].items():
        lines.append(f"- {label}: " + ", ".join(f"{n} {fmt(block['eta2'][n])} / {fmt(block['dyn_eta2'][n])}" for n in NAMES))
    lines += ["", "## Deaths per episode by day 120 (mean)", ""]
    for scene, b in blocks.items():
        lines.append(f"- {scene}: " + ", ".join(f"{k} {v:.2f}" for k, v in b["deaths_mean_by_day120"].items()))
    if "game_v17" in out["scenes"]:
        j = out["scenes"]["game_v17"]["j7"]
        lines += ["", f"**J7 (game_v17): {'PASS' if j['passed'] else 'FAIL'}** — " +
                  ", ".join(f"{k}: {'PASS' if v['passed'] else 'FAIL'}" for k, v in j["clauses"].items())]
    return "\n".join(lines) + "\n"


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    a = sub.add_parser("check-v1")
    a.add_argument("--root", default="/Users/rexxa/work/worldsim-wm-data")
    a.add_argument("--out", required=True)
    b = sub.add_parser("pilot")
    b.add_argument("--root", required=True)
    b.add_argument("--scenes", default="game_v16,game_v17_nofission,game_v17")
    b.add_argument("--train", default="22000..22096")
    b.add_argument("--test", default="22096..22112")
    b.add_argument("--reference-root")
    b.add_argument("--reference-scenes", default="game_v15,game_v16")
    b.add_argument("--reference-train", default="20000..20096")
    b.add_argument("--reference-test", default="20096..20112")
    b.add_argument("--out", required=True)
    args = ap.parse_args(argv)
    return d10.check_v1(args) if args.cmd == "check-v1" else pilot(args)


if __name__ == "__main__":
    raise SystemExit(main())
