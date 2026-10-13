#!/usr/bin/env python3
"""Pass 10 Part 5: seed diversity with WM-1.1's exact definitions (PASS10_GOAL.md 11.2-11.4). numpy only.

The array math is `calcore_wm11.py` (a verbatim copy of WM-1.1's `calcore.py` at `ecf8c316`, loaded by file
path); episodes are loaded with `wm_data.load_episode` and turned into the eight V variables by
`bench_v1.episode_arrays` (imported read-only from `tools/worldmodel`). The calendar statistics are fitted on
a train split and every diagnostic is scored on a test split, as WM-1.1's `diversity_v1.py` does (`K_MAX` 168,
`bench_v1.split_std` per split, `T_TR` 2881, `calcore.split_diagnostics`).

usage:
  diversity_p10.py check-v1 --root /Users/rexxa/work/worldsim-wm-data --out DIR
      dataset v1, A_train fit, A_test scored; compared with WM-1.1's published A_test numbers (+-0.002)
  diversity_p10.py pilot --root /Users/rexxa/work/worldsim-wm-data/p10/pilot --scenes game_v15,game_v16_neutral,game_v16
                         --train 20000..20096 --test 20096..20112 --out DIR
"""
from __future__ import annotations

import argparse
import importlib.util
import json
import sys
import time
from pathlib import Path

import numpy as np

REPO = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO / "tools/worldmodel"))
import bench_v1 as bv  # noqa: E402
import wm_data  # noqa: E402

_spec = importlib.util.spec_from_file_location("calcore_wm11", Path(__file__).resolve().parent / "calcore_wm11.py")
cc = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(cc)

K_MAX, T_TR = 168, 2881
NAMES = tuple(bv.VARS)
MEMBERS = ("cal_abs", "apfit_abs", "apfit_year", "an5", "an5_off")
# WM-1.1's published dataset-v1 A_test values (REPORT_WM1_1.md), the implementation check of 11.2.
V1_ETA2 = {"food_stock": .195, "wood_stock": .049, "fire_lit": .039, "cold_members": .598, "alive": .061,
           "deadwood_total": .027, "food_on_bushes_total": .136, "adults_working": .284}
V1_R2_ABS = {"food_stock": .768, "wood_stock": .947, "fire_lit": .959, "alive": .932}
H7_VARS = ("food_stock", "wood_stock", "alive")


class Fit:
    """Train-only calendar statistics (WM-1.1 diversity_v1.Fit)."""

    def __init__(self, lib: np.ndarray, std: np.ndarray):
        self.lib = lib
        self.m_abs, self.m_year = cc.calendar_means(lib)
        rows = np.arange(lib.shape[1])
        self.alpha = {k: cc.fit_alpha(lib, cc.mean_at(k, rows, self.m_abs, self.m_year), K_MAX) for k in ("abs", "year")}
        self.std = std
        self.inv_std = np.where(std > 0, 1.0 / np.where(std > 0, std, 1.0), 1.0)

    def predict(self, member: str, v: np.ndarray, starts: np.ndarray, horizon: int) -> np.ndarray:
        return cc.predict_member(member, v, starts, horizon, self.lib, self.m_abs, self.m_year, self.alpha,
                                 self.inv_std)[:, -1]


def per(arr) -> dict:
    return {n: (None if not np.isfinite(x) else float(x)) for n, x in zip(NAMES, arr)}


def severity_sequence(ep) -> tuple:
    """Hidden severity at 18:00 of each winter's first day (a grouping label, never an input)."""
    out = []
    for day in cc.winter_days(len(ep.tick)):
        r = cc.row_of(day, 1080)
        out.append(int(ep.hidden_severity[r]))
    return tuple(out)


def t2_nmae(fit: Fit, eps: list, std: np.ndarray, horizon: int) -> dict:
    """nMAE at every-hour starts (`bench_v1.rollout_starts(n, H, True)`) for the calendar members and persistence."""
    out = {}
    for member in (*MEMBERS, "persistence"):
        preds, trues = [], []
        for ep in eps:
            starts = np.array(bv.rollout_starts(len(ep.tick), horizon, True), dtype=np.int64)
            preds.append(ep.v[starts] if member == "persistence" else fit.predict(member, ep.v, starts, horizon))
            trues.append(ep.v[starts + horizon])
        out[member] = per(bv.nmae(np.vstack(preds), np.vstack(trues), std))
    return out


def stack(eps: list) -> np.ndarray:
    rows = {len(e.tick) for e in eps}
    if len(rows) != 1:
        raise SystemExit(f"episodes differ in length {sorted(rows)}")
    return np.stack([e.v for e in eps]).astype(np.float64)


def evaluate(train: list, test: list, horizons=(24, 168)) -> dict:
    if {len(e.tick) for e in train} != {T_TR}:
        raise SystemExit(f"train rows {sorted({len(e.tick) for e in train})} != {T_TR}")
    for e in train:
        if not np.array_equal(e.tick, 60 * np.arange(T_TR)):
            raise SystemExit(f"seed {e.seed}: tick != 60 * arange(rows)")
    fit = Fit(stack(train), bv.split_std(train))
    if not np.isfinite(fit.m_year).all():
        raise SystemExit("m_year leaves a year position uncovered")
    std = bv.split_std(test)
    diag = cc.split_diagnostics(stack(test), fit.m_abs, fit.m_year, std, [severity_sequence(e) for e in test], NAMES)
    diag["seeds"] = [int(e.seed) for e in test]
    diag["train_seeds"] = [int(e.seed) for e in train]
    diag["t2"] = {str(h): t2_nmae(fit, test, std, h) for h in horizons}
    return diag


def load_dir(root: Path, scene: str, seeds: range, role: str) -> list:
    eps = []
    for seed in seeds:
        path = root / scene / f"{scene}_s{seed}_none"
        ep = wm_data.load_episode(path, arrays=["global.npy", "settlements.npy"])
        e = bv.Ep(scene, role, seed, "none", **bv.episode_arrays(ep))
        g = lambda name: ep.column("global.npy", name)
        e.deaths_exposure = g("deaths_exposure") if "deaths_exposure" in [f["name"] for f in ep.meta["fields"]["global.npy"]] else None
        e.deaths_by_cause = {k: float(g(k)[-1]) for k in ("deaths_starvation", "deaths_dehydration", "deaths_combat",
                                                          "deaths_old_age", "deaths_exposure")
                             if k in [f["name"] for f in ep.meta["fields"]["global.npy"]]}
        e.variety = ep.world_variety()
        e.schema = ep.meta["schema_version"]
        eps.append(e)
    return eps


def check_v1(args) -> int:
    t0 = time.time()
    manifest = json.loads((REPO / "docs/worldmodel/dataset_v1_manifest.json").read_text())
    data = bv.load_split(Path(args.root), manifest, "A_obs")
    train = [e for e in data if e.role == "train"]
    test = [e for e in data if e.role == "test"]
    d = evaluate(train, test, horizons=(168,))
    rows = []
    ok = True
    for name, want in V1_ETA2.items():
        got = d["eta2"][name]
        good = got is not None and abs(got - want) <= .002
        ok &= good
        rows.append({"metric": "eta2", "var": name, "published": want, "reproduced": got, "ok": good})
    for name, want in V1_R2_ABS.items():
        got = d["r2_abs"][name]
        good = got is not None and abs(got - want) <= .002
        ok &= good
        rows.append({"metric": "r2_abs", "var": name, "published": want, "reproduced": got, "ok": good})
    out = {"dataset": "v1", "split": "A_obs: fit A_train, score A_test", "counts": {"train": len(train), "test": len(test)},
           "reproduced": ok, "rows": rows, "diagnostics": d, "seconds": time.time() - t0}
    Path(args.out).mkdir(parents=True, exist_ok=True)
    (Path(args.out) / "check_v1.json").write_text(json.dumps(out, indent=1, sort_keys=True) + "\n")
    for r in rows:
        print(f"{r['metric']:7s} {r['var']:22s} published {r['published']:.3f} reproduced {r['reproduced']:.4f} {'OK' if r['ok'] else 'DIFF'}")
    print("REPRODUCED" if ok else "NOT REPRODUCED")
    return 0 if ok else 1


def parse_range(text: str) -> range:
    a, b = text.split("..")
    return range(int(a), int(b))


def fmt(x, digits=3) -> str:
    return "–" if x is None else f"{x:.{digits}f}"


def pilot(args) -> int:
    t0 = time.time()
    root = Path(args.root)
    scenes = args.scenes.split(",")
    train_seeds, test_seeds = parse_range(args.train), parse_range(args.test)
    out = {"root": str(root), "train": [train_seeds.start, train_seeds.stop], "test": [test_seeds.start, test_seeds.stop],
           "vars": list(NAMES), "scenes": {}, "h7_vars": list(H7_VARS)}
    for scene in scenes:
        train = load_dir(root, scene, train_seeds, "train")
        test = load_dir(root, scene, test_seeds, "test")
        d = evaluate(train, test)
        everyone = train + test
        deaths = {k: float(np.mean([e.deaths_by_cause.get(k, 0.0) for e in everyone]))
                  for k in ("deaths_starvation", "deaths_dehydration", "deaths_combat", "deaths_old_age", "deaths_exposure")}
        d["deaths_mean_by_day120"] = deaths
        d["deaths_with_exposure_mean"] = float(np.mean([sum(e.deaths_by_cause.values()) for e in everyone]))
        d["schema"] = sorted({e.schema for e in everyone})
        h7 = {name: {"eta2": d["eta2"][name], "r2_abs": d["r2_abs"][name],
                     "passed": (d["eta2"][name] or 0) >= .30 and d["r2_abs"][name] is not None and d["r2_abs"][name] <= .70}
              for name in H7_VARS}
        d["h7"] = {"passed": all(v["passed"] for v in h7.values()), "vars": h7}
        out["scenes"][scene] = d
        print(scene, "H7", "PASS" if d["h7"]["passed"] else "FAIL",
              {k: (fmt(v['eta2']), fmt(v['r2_abs'])) for k, v in h7.items()}, f"{time.time() - t0:.0f}s", flush=True)
    # Review round 1, L2: v15 -> v16_neutral is exposure plus Part 3's tighter birth gate (utilization 0.78).
    pairs = [("exposure (+ utilization 0.78)", "game_v15", "game_v16_neutral"), ("variety", "game_v16_neutral", "game_v16")]
    out["attribution"] = {}
    for label, a, b in pairs:
        if a in out["scenes"] and b in out["scenes"]:
            out["attribution"][label] = {
                metric: {n: (None if out["scenes"][b][metric][n] is None or out["scenes"][a][metric][n] is None
                             else out["scenes"][b][metric][n] - out["scenes"][a][metric][n]) for n in NAMES}
                for metric in ("eta2", "r2_abs", "r2_year")}
    out["seconds"] = time.time() - t0
    target = Path(args.out)
    target.mkdir(parents=True, exist_ok=True)
    (target / "diversity.json").write_text(json.dumps(out, indent=1, sort_keys=True) + "\n")
    (target / "diversity.md").write_text(markdown(out))
    print(f"done {out['seconds']:.0f}s -> {target}")
    return 0


def markdown(out: dict) -> str:
    scenes = list(out["scenes"])
    lines = ["# Pass 10 diversity pilot", "",
             f"Train seeds {out['train'][0]}..{out['train'][1] - 1} (calendar statistics fitted here), test seeds "
             f"{out['test'][0]}..{out['test'][1] - 1} (every number below). WM-1.1's definitions (`calcore_wm11.py`): "
             "R²_abs / R²_year against the train calendar means, η² = between-seed share of the variance, divergence day "
             "over test pairs, spread = std over seeds at 06:00 / split std, T2 nMAE at every-hour starts.", ""]
    for metric, title in (("eta2", "η²"), ("r2_abs", "R²_abs"), ("r2_year", "R²_year")):
        lines += [f"## {title}", "", "| Variable | " + " | ".join(scenes) + " |", "|---|" + "---:|" * len(scenes)]
        for n in out["vars"]:
            lines.append(f"| {n} | " + " | ".join(fmt(out["scenes"][s][metric][n]) for s in scenes) + " |")
        lines.append("")
    lines += ["## H7 (η² ≥ 0.30 and R²_abs ≤ 0.70 for food, wood, alive)", "", "| Scene | Verdict | " +
              " | ".join(out["h7_vars"]) + " |", "|---|---|" + "---|" * len(out["h7_vars"])]
    for s in scenes:
        h = out["scenes"][s]["h7"]
        lines.append(f"| {s} | {'PASS' if h['passed'] else 'FAIL'} | " +
                     " | ".join(f"η² {fmt(h['vars'][n]['eta2'])}, R²_abs {fmt(h['vars'][n]['r2_abs'])}" for n in out["h7_vars"]) + " |")
    lines += ["", "## Divergence day (test pairs: min / median / max; never)", "", "| Variable | " + " | ".join(scenes) + " |",
              "|---|" + "---|" * len(scenes)]
    for n in ["(any)", *out["vars"]]:
        cells = []
        for s in scenes:
            dv = out["scenes"][s]["divergence"]
            st = dv if n == "(any)" else dv["per_var"][n]
            cells.append(f"{st['min_day']} / {st['median_day']} / {st['max_day']}; {st['never']}")
        lines.append(f"| {n} | " + " | ".join(cells) + " |")
    lines += ["", "## Between-seed spread at 06:00 (std over test seeds / split std)", ""]
    for s in scenes:
        lines += [f"**{s}**", "", "| Day | " + " | ".join(out["vars"]) + " |", "|---|" + "---:|" * len(out["vars"])]
        for day, row in out["scenes"][s]["spread"].items():
            lines.append(f"| {day} | " + " | ".join(fmt(row[n]) for n in out["vars"]) + " |")
        lines.append("")
    for h in ("24", "168"):
        lines += [f"## T2 {h} h nMAE (every-hour starts, test split std)", ""]
        for s in scenes:
            t2 = out["scenes"][s]["t2"][h]
            lines += [f"**{s}**", "", "| Member | " + " | ".join(out["vars"]) + " | mean |", "|---|" + "---:|" * (len(out["vars"]) + 1)]
            for m, row in t2.items():
                vals = [row[n] for n in out["vars"]]
                mean = float(np.nanmean([v for v in vals if v is not None]))
                lines.append(f"| {m} | " + " | ".join(fmt(v) for v in vals) + f" | {mean:.3f} |")
            lines.append("")
    lines += ["## Attribution (difference of the metric)", ""]
    for label, block in out["attribution"].items():
        lines += [f"**{label}**", "", "| Variable | Δη² | ΔR²_abs | ΔR²_year |", "|---|---:|---:|---:|"]
        for n in out["vars"]:
            lines.append(f"| {n} | {fmt(block['eta2'][n])} | {fmt(block['r2_abs'][n])} | {fmt(block['r2_year'][n])} |")
        lines.append("")
    lines += ["## Deaths (mean per episode by day 120, all 112 seeds)", "", "| Scene | Starvation | Dehydration | Old age | Exposure | All |",
              "|---|---:|---:|---:|---:|---:|"]
    for s in scenes:
        d = out["scenes"][s]["deaths_mean_by_day120"]
        lines.append(f"| {s} | {d['deaths_starvation']:.2f} | {d['deaths_dehydration']:.2f} | {d['deaths_old_age']:.2f} | "
                     f"{d['deaths_exposure']:.2f} | {out['scenes'][s]['deaths_with_exposure_mean']:.2f} |")
    return "\n".join(lines) + "\n"


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    a = sub.add_parser("check-v1")
    a.add_argument("--root", default="/Users/rexxa/work/worldsim-wm-data")
    a.add_argument("--out", required=True)
    b = sub.add_parser("pilot")
    b.add_argument("--root", required=True)
    b.add_argument("--scenes", default="game_v15,game_v16_neutral,game_v16")
    b.add_argument("--train", default="20000..20096")
    b.add_argument("--test", default="20096..20112")
    b.add_argument("--out", required=True)
    args = ap.parse_args(argv)
    return check_v1(args) if args.cmd == "check-v1" else pilot(args)


if __name__ == "__main__":
    raise SystemExit(main())
