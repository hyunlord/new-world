"""WM-1.1 Part 4: skill against the calendar and the pre-registered X verdicts (GOAL_1_1.md 9, 10; METRICS_1_1.md 3).

Reads only result JSON files (copies under docs/worldmodel/wm1/runs/), never data or checkpoints:
the calendar family's test numbers (`calendar_<sha8>/calendar.json`, with the val selection), the models'
`metrics.json`, `ridge_variants_9e0f7b3b/ridge_variants.json`, M3 / M3-noday and `diversity_<sha8>/diversity.json`.
Writes `judge.json` and `judge.md` (the X table and the skill tables).

usage: python3 tools/worldmodel/wm1/judge_1_1.py --runs docs/worldmodel/wm1/runs --calendar calendar_9f4fd3f3 \
           --diversity diversity_1076b53d --out docs/worldmodel/wm1/runs/judge_1_1
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np

VARS = ("food_stock", "wood_stock", "fire_lit", "cold_members", "alive", "deadwood_total", "food_on_bushes_total",
        "adults_working")
SPLITS = ("A_test", "B_test", "OOD_v12", "OOD_long")
X_SPLITS = ("A_test", "B_test", "OOD_long")
PATHS = {"T1": ("T1",), "T2_24": ("T2", "24"), "T2_168": ("T2", "168"), "T2any_24": ("T2_any", "24"),
         "T2any_168": ("T2_any", "168")}
M1B = ("m1tf_main_r3_s0", "m1tf_main_r3_s1", "m1tf_main_r3_s2")
GRU = ("m1gru_main_r3_s0", "m1gru_main_r3_s1", "m1gru_main_r3_s2")
SEVON = "m1tf_sevon_aa_r3_s0"
RIDGES = ("ridge_nosev", "ridge_action_nosev")


def load(path: Path):
    return json.loads(path.read_text()) if path.exists() else None


def node(d: dict, task: str, split: str, ridge: str | None = None) -> dict | None:
    x = d
    for k in PATHS[task]:
        x = x.get(k, {})
    x = x.get(split)
    if ridge is not None and x is not None:
        x = x.get(ridge)
    return x or None


def verdict(cells: dict[str, tuple[list[float], float]]) -> str:
    """KEEP: every cell's mean <= 0.8 * cal and every seed < cal; WEAK: every cell's mean < cal; NONE otherwise."""
    if not cells or any(not seeds or not np.isfinite(cal) for seeds, cal in cells.values()):
        return "n/a"
    keep = all(np.mean(s) <= 0.8 * cal and all(x < cal for x in s) for s, cal in cells.values())
    if keep:
        return "KEEP"
    return "WEAK" if all(np.mean(s) < cal for s, cal in cells.values()) else "NONE"


def skill(model: float, cal: float) -> float | None:
    if cal is None or not np.isfinite(cal) or cal == 0 or model is None or not np.isfinite(model):
        return None
    return 1.0 - model / cal


def fmt(x, d=3) -> str:
    return "—" if x is None or (isinstance(x, float) and not np.isfinite(x)) else f"{x:.{d}f}"


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--runs", type=Path, required=True)
    ap.add_argument("--calendar", required=True)
    ap.add_argument("--diversity", required=True)
    ap.add_argument("--out", type=Path, required=True)
    args = ap.parse_args(argv)
    runs = args.runs
    cal = load(runs / args.calendar / "calendar.json")
    sel = cal["selection"]
    test = cal["test"]
    models = {name: load(runs / name / "metrics.json") for name in M1B + GRU + (SEVON,)}
    ridge = load(runs / "ridge_variants_9e0f7b3b" / "ridge_variants.json")
    m3, m3nd = load(runs / "m3_s0" / "metrics.json"), load(runs / "m3_noday_s0" / "metrics.json")
    div = load(runs / args.diversity / "diversity.json")

    def cal_val(task: str, split: str, which: str) -> dict:
        member = sel[task][which]
        return {"member": member, **node(test[member], task, split)}

    def model_node(name: str, task: str, split: str) -> dict | None:
        if name in RIDGES:
            return node(ridge, task, split, name)
        return node(models[name], task, split) if models.get(name) else None

    out: dict = {"selection": {t: {k: sel[t][k] for k in ("cal_best", "cal_best_fair")} for t in sel},
                 "calendar_test": {}, "skill": {}, "X": {}}
    # Calendar test numbers: every member, task, split (mean nMAE).
    for m, res in test.items():
        out["calendar_test"][m] = {t: {s: (node(res, t, s) or {}).get("mean") for s in SPLITS} for t in PATHS}
        out["calendar_test"][m]["T3_brier"] = res.get("T3", {}).get("brier")
        out["calendar_test"][m]["T3b_mae"] = {s: res["T3b"].get(s, {}).get("mae") for s in SPLITS}
    # Skill of every model, split, task.
    names = M1B + GRU + (SEVON,) + RIDGES
    for name in names:
        out["skill"][name] = {}
        for t in PATHS:
            for s in SPLITS:
                mn = model_node(name, t, s)
                if mn is None:
                    continue
                cb, cf = cal_val(t, s, "cal_best"), cal_val(t, s, "cal_best_fair")
                out["skill"][name].setdefault(t, {})[s] = {
                    "nmae": mn["mean"], "cal_best": cb["mean"], "cal_best_fair": cf["mean"],
                    "skill_best": skill(mn["mean"], cb["mean"]), "skill_fair": skill(mn["mean"], cf["mean"]),
                    "per_var_skill_best": {v: skill(a, b) for v, a, b in zip(VARS, mn["per_var"], cb["per_var"])},
                    "per_var_skill_fair": {v: skill(a, b) for v, a, b in zip(VARS, mn["per_var"], cf["per_var"])},
                    "per_var_nmae": dict(zip(VARS, mn["per_var"])),
                    "per_var_cal_best": dict(zip(VARS, cb["per_var"]))}

    def cells(seed_names, tasks, which):
        c = {}
        for t in tasks:
            for s in X_SPLITS:
                vals = [model_node(n, t, s) for n in seed_names]
                if any(v is None for v in vals):
                    return {}
                c[f"{t}/{s}"] = ([v["mean"] for v in vals], cal_val(t, s, which)["mean"])
        return c

    def rule(seed_names, tasks):
        res = {}
        for which in ("cal_best", "cal_best_fair"):
            c = cells(seed_names, tasks, which)
            res[which] = {"verdict": verdict(c), "cells": {k: {"seeds": v[0], "mean": float(np.mean(v[0])),
                                                                 "cal": v[1], "ratio": float(np.mean(v[0])) / v[1]}
                                                             for k, v in c.items()}}
        return res

    # X1
    a = div["groups"]["A_test"]
    out["X"]["X1"] = {"calendar_determined_A_test": [v for v in VARS if (a["r2_abs"][v] or -1) >= 0.9],
                      "A_test": {k: a[k] for k in ("r2_abs", "r2_year", "eta2")}}
    # X2, X3, X5
    long_tasks, short_tasks = ("T2_168", "T2any_168"), ("T1", "T2_24", "T2any_24")
    out["X"]["X2"] = rule(M1B, long_tasks)
    out["X"]["X3"] = rule(M1B, short_tasks)
    out["X"]["X3_by_task"] = {t: rule(M1B, (t,)) for t in short_tasks}
    out["X"]["X5"] = {"ridge_nosev": rule(("ridge_nosev",), long_tasks), "m1_gru_main_r3": rule(GRU, long_tasks),
                      "ridge_action_nosev (extra)": rule(("ridge_action_nosev",), long_tasks)}
    # X4
    a24b, a24f = cal_val("T2_24", "A_test", "cal_best"), cal_val("T2_24", "A_test", "cal_best_fair")
    calabs = node(test["cal_abs"], "T2_24", "A_test")
    x4 = {"cal_best": a24b, "cal_best_fair": a24f, "cal_abs": calabs}
    for name, m in (("m3_s0", m3), ("m3_noday_s0", m3nd)):
        if m is None:
            x4[name] = None
            continue
        v = m["m3"]["mean"]
        x4[name] = {"nmae": v, "per_var": dict(zip(VARS, m["m3"]["per_var"])), "n": m["n"],
                    "parse_rate": m["parse_rate"], "ratio_to_cal_best": v / a24b["mean"],
                    "verdict": "skill beyond the calendar" if v <= 0.8 * a24b["mean"]
                    else "consistent with calendar memorization",
                    "skill_vs_cal_best": skill(v, a24b["mean"]), "skill_vs_cal_abs": skill(v, calabs["mean"]),
                    "skill_vs_cal_best_fair": skill(v, a24f["mean"])}
    if m3 and m3nd:
        x4["m3_minus_noday"] = {"mean": m3["m3"]["mean"] - m3nd["m3"]["mean"],
                                "per_var": {v: a - b for v, a, b in zip(VARS, m3["m3"]["per_var"], m3nd["m3"]["per_var"])}}
    out["X"]["X4"] = x4
    # X6
    sevon = models.get(SEVON)
    x6 = {}
    if sevon and all(models.get(n) for n in M1B):
        blind = [models[n] for n in M1B]
        for s in SPLITS:
            x6.setdefault("T3b_mae", {})[s] = {"sev_on": sevon["T3b"][s]["mae"],
                                               "sev_blind": [b["T3b"][s]["mae"] for b in blind]}
            w = lambda d: d["T2_any"]["168"][s].get("by_target_season", {}).get("3", {}).get("mean")
            x6.setdefault("T2any_168_winter", {})[s] = {"sev_on": w(sevon), "sev_blind": [w(b) for b in blind]}
        for var in sevon["T4"]["scores"]:
            g = lambda d: d["T4"]["scores"][var]["dark_nights"]
            x6.setdefault("T4_dark_nights", {})[var] = {"sev_on": g(sevon), "sev_blind": [g(b) for b in blind]}
    out["X"]["X6"] = x6
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "judge.json").write_text(json.dumps(out, indent=1, sort_keys=True) + "\n")
    (args.out / "judge.md").write_text(markdown(out))
    print(markdown(out)[:3000])
    return 0


def markdown(o: dict) -> str:
    L = ["# WM-1.1 judge output (generated by judge_1_1.py)", "", "## X table", "",
         "| # | verdict vs `cal_best` | vs `cal_best_fair` | evidence |", "|---|---|---|---|"]
    x = o["X"]
    L.append(f"| X1 | calendar-determined on A_test (R²_abs >= 0.9): {', '.join(x['X1']['calendar_determined_A_test']) or 'none'} | — | "
             "R²_abs / R²_year / η² in DIVERSITY.md |")
    for key, label in (("X2", "M1b 168 h"), ("X3", "M1b T1 + 24 h")):
        ev = "; ".join(f"{k}: mean {fmt(c['mean'])} ({', '.join(fmt(s) for s in c['seeds'])}) vs {fmt(c['cal'])} = {fmt(c['ratio'], 2)}×"
                       for k, c in x[key]["cal_best"]["cells"].items())
        L.append(f"| {key} ({label}) | **{x[key]['cal_best']['verdict']}** | {x[key]['cal_best_fair']['verdict']} | {ev} |")
    x4 = x["X4"]
    for name in ("m3_s0", "m3_noday_s0"):
        if x4.get(name):
            m = x4[name]
            L.append(f"| X4 ({name}) | **{m['verdict']}** | — | nMAE {fmt(m['nmae'])} vs cal_best `{x4['cal_best']['member']}` "
                     f"{fmt(x4['cal_best']['mean'])} ({fmt(m['ratio_to_cal_best'], 2)}×), CalAbs {fmt(x4['cal_abs']['mean'])}, "
                     f"cal_best_fair `{x4['cal_best_fair']['member']}` {fmt(x4['cal_best_fair']['mean'])} |")
    if "m3_minus_noday" in x4:
        L.append(f"| X4 (M3 − M3-noday) | {fmt(x4['m3_minus_noday']['mean'])} | — | per variable in judge.json |")
    for name, r in x["X5"].items():
        ev = "; ".join(f"{k}: {fmt(c['mean'])} vs {fmt(c['cal'])} = {fmt(c['ratio'], 2)}×" for k, c in r["cal_best"]["cells"].items())
        L.append(f"| X5 ({name}) | **{r['cal_best']['verdict']}** | {r['cal_best_fair']['verdict']} | {ev} |")
    L.append("| X6 | descriptive | — | see the severity table |")
    L += ["", "## X3 by task", "", "| task | vs cal_best | vs cal_best_fair |", "|---|---|---|"]
    for t, r in x["X3_by_task"].items():
        L.append(f"| {t} | {r['cal_best']['verdict']} | {r['cal_best_fair']['verdict']} |")
    L += ["", "## Calendar family on test (mean nMAE; T3 Brier; T3b MAE)", ""]
    tasks = ("T1", "T2_24", "T2_168", "T2any_24", "T2any_168")
    for s in ("A_test", "B_test", "OOD_v12", "OOD_long"):
        L += [f"### {s}", "", "| member | " + " | ".join(tasks) + " | T3b MAE |" + (" T3 Brier |" if s == "A_test" else ""),
              "|---" * (len(tasks) + 2 + (s == "A_test")) + "|"]
        for m, r in o["calendar_test"].items():
            cells = [fmt(r[t][s]) for t in tasks] + [fmt(r["T3b_mae"][s])]
            if s == "A_test":
                cells.append(fmt(r["T3_brier"], 4))
            L.append(f"| `{m}` | " + " | ".join(cells) + " |")
        L.append("")
    L += ["## Skill = 1 − nMAE_model / nMAE_cal_best (in brackets: against cal_best_fair)", ""]
    for t in tasks:
        L += [f"### {t}", "", "| model | " + " | ".join(SPLITS) + " |", "|---" * (len(SPLITS) + 1) + "|"]
        for name, d in o["skill"].items():
            if t not in d:
                continue
            L.append(f"| `{name}` | " + " | ".join(
                f"{fmt(d[t][s]['nmae'])}: {fmt(d[t][s]['skill_best'], 2)} [{fmt(d[t][s]['skill_fair'], 2)}]" if s in d[t] else "—"
                for s in SPLITS) + " |")
        L.append("")
    L += ["## Per-variable skill against cal_best, A_test", ""]
    for t in ("T2_24", "T2_168", "T2any_168"):
        L += [f"### {t}", "", "| model | " + " | ".join(VARS) + " |", "|---" * (len(VARS) + 1) + "|"]
        for name, d in o["skill"].items():
            if t in d and "A_test" in d[t]:
                L.append(f"| `{name}` | " + " | ".join(fmt(d[t]["A_test"]["per_var_skill_best"][v], 2) for v in VARS) + " |")
        L.append("")
    if o["X"]["X6"]:
        x6 = o["X"]["X6"]
        L += ["## X6: M1b sev-on vs sev-blind (3 seeds: mean [min, max])", "", "| measure | split / variant | sev-on | sev-blind |",
              "|---|---|---|---|"]
        for meas, d in x6.items():
            for k, v in d.items():
                if meas == "T4_dark_nights":
                    on = f"MAE {fmt(v['sev_on']['mae'])}, nMAE {fmt(v['sev_on']['nmae'])}"
                    bl = [b["mae"] for b in v["sev_blind"]]
                else:
                    on, bl = fmt(v["sev_on"]), v["sev_blind"]
                L.append(f"| {meas} | {k} | {on} | {fmt(float(np.mean(bl)))} [{fmt(min(bl))}, {fmt(max(bl))}] |")
    return "\n".join(L) + "\n"


if __name__ == "__main__":
    raise SystemExit(main())
