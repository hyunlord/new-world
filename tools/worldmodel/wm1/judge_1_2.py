"""WM-1.2 Part 6: the pre-registered Z verdicts and the pilot tables (GOAL_1_2.md 8; METRICS_1_2.md 3).

Reads only result JSON files (copies under docs/worldmodel/wm1/runs/), never data or checkpoints: the calendar
family's validation selection (`calendar_val_<ds>_<sha8>/calendar_val.json`) and test numbers
(`calendar_<ds>_<sha8>/calendar.json`, which also hold persistence and the observable ridge), the models'
`metrics.json` and the probes' `probe.json`. Writes `judge.json` and `judge.md`.

usage: python3 tools/worldmodel/wm1/judge_1_2.py --runs docs/worldmodel/wm1/runs --v16-cal SHA8 --v15-cal SHA8 \
           --out docs/worldmodel/wm1/runs/judge_1_2
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np

from wm1.judge_1_1 import skill, verdict

VARS = ("food_stock", "wood_stock", "fire_lit", "cold_members", "alive", "deadwood_total", "food_on_bushes_total",
        "adults_working")
Z4_VARS = {"food": "food_stock", "alive": "alive", "bushes": "food_on_bushes_total", "working adults": "adults_working"}
M1B_V16 = ("m1tf_p10v16_r3_s0", "m1tf_p10v16_r3_s1", "m1tf_p10v16_r3_s2")
M1B_V15 = ("m1tf_p10v15_r3_s0",)
SPLIT = "test"
# (evaluate path, horizon) of each T2 cell; the selection task of its raw and bias-removed score.
CELLS = {("T2_any", "168"): ("T2any_168", "T2anybr_168"), ("T2_any", "24"): ("T2any_24", "T2anybr_24"),
         ("T2", "168"): ("T2_168", "T2br_168"), ("T2", "24"): ("T2_24", "T2br_24")}


def load(path: Path):
    return json.loads(path.read_text()) if path.exists() else None


def score(result: dict, key: str, h: str, br: bool) -> dict | None:
    """{"mean", "per_var"} of a T2 cell on test, raw or bias-removed."""
    node = (result or {}).get(key, {}).get(h, {}).get(SPLIT)
    if not node:
        return None
    return node["bias_removed"] if br else {"mean": node["mean"], "per_var": node["per_var"]}


def pilot_tables(cal_test: dict, cal_val: dict, models: dict[str, dict]) -> dict:
    """Every predictor's raw and bias-removed T2 scores on test: the nine members, the selected ones, persistence,
    the ridge, each model run and the models' seed mean (per variable)."""
    sel = cal_val["selection"]
    out = {}
    for (key, h), (task_raw, task_br) in CELLS.items():
        for br, task in ((False, task_raw), (True, task_br)):
            name = f"{key}_{h}" + ("_br" if br else "")
            rows = {m: score(r, key, h, br) for m, r in cal_test["test"].items()}
            rows |= {n: score(r, key, h, br) for n, r in cal_test["references"].items()}
            rows["cal_best"] = rows[sel[task]["cal_best"]]
            rows["cal_best_fair"] = rows[sel[task]["cal_best_fair"]]
            seeds = {run: score(r, key, h, br) for run, r in models.items()}
            rows |= seeds
            have = [v for v in seeds.values() if v]
            if have:
                rows["models_mean"] = {"mean": float(np.mean([v["mean"] for v in have])),
                                       "per_var": np.mean([v["per_var"] for v in have], axis=0).tolist(),
                                       "seeds": len(have)}
            out[name] = {"selection": {"cal_best": sel[task]["cal_best"], "cal_best_fair": sel[task]["cal_best_fair"]},
                         "rows": rows}
    return out


def z_rule(models: dict[str, dict], cal_test: dict, cal_val: dict, key: str, h: str, br: bool) -> dict:
    """KEEP / WEAK / NONE of the models' test score against the val-selected calendar member of the same task."""
    task = CELLS[(key, h)][1 if br else 0]
    member = cal_val["selection"][task]["cal_best"]
    cal = score(cal_test["test"][member], key, h, br)
    seeds = {run: score(r, key, h, br) for run, r in models.items()}
    vals = [v["mean"] for v in seeds.values() if v]
    return {"task": task, "against": member, "cal": cal["mean"] if cal else None,
            "seeds": {run: (v["mean"] if v else None) for run, v in seeds.items()},
            "mean": float(np.mean(vals)) if vals else None, "n_seeds": len(vals),
            "verdict": verdict({"test": (vals, cal["mean"])}) if cal and vals else "n/a"}


def z4(models: dict[str, dict], cal_test: dict, cal_val: dict) -> dict:
    """Skill 1 - nMAE / baseline of the models' seed mean, per variable, at T2-any 168 h: raw against `cal_best` and
    `apfit_abs`, bias-removed against `cal_best_anom` and `apfit_abs` (bias-removed)."""
    out = {}
    for br in (False, True):
        task = CELLS[("T2_any", "168")][1 if br else 0]
        best = cal_val["selection"][task]["cal_best"]
        refs = {"cal_best_anom" if br else "cal_best": best, "apfit_abs": "apfit_abs"}
        have = [s for s in (score(r, "T2_any", "168", br) for r in models.values()) if s]
        model = np.mean([s["per_var"] for s in have], axis=0) if have else None
        tab = {}
        for label, var in Z4_VARS.items():
            j = VARS.index(var)
            row = {"model_nmae": None if model is None else float(model[j])}
            for ref_name, member in refs.items():
                base = score(cal_test["test"][member], "T2_any", "168", br)["per_var"][j]
                row[f"{ref_name}_nmae"] = base
                row[f"skill_vs_{ref_name}"] = None if model is None else skill(float(model[j]), base)
            tab[label] = row
        out["bias_removed" if br else "raw"] = {"against": refs, "per_variable": tab, "seeds": len(have)}
    return out


def z5(probes: dict[str, dict]) -> dict:
    """Probe R^2 (and climate accuracy) on test per seed, their mean, and the raw-row reference."""
    if not probes:
        return {"verdict": "n/a (dropped or missing)"}
    params = [k for k in next(iter(probes.values()))["model"]]
    metric = lambda v: v.get("r2_test", v.get("accuracy_test"))
    out = {"per_seed": {run: {p: metric(pr["model"][p]) for p in params} for run, pr in probes.items()},
           "reference_raw_row": {p: metric(next(iter(probes.values()))["reference_raw_row"][p]) for p in params}}
    out["mean"] = {p: (float(np.mean([v[p] for v in out["per_seed"].values() if v[p] is not None]))
                       if any(v[p] is not None for v in out["per_seed"].values()) else None) for p in params}
    out["metric"] = {p: ("accuracy" if "accuracy_test" in next(iter(probes.values()))["model"][p] else "r2") for p in params}
    return out


def fmt(x, digits: int = 3) -> str:
    return "—" if x is None or (isinstance(x, float) and not np.isfinite(x)) else f"{x:.{digits}f}"


def markdown(r: dict) -> str:
    lines = ["# WM-1.2 judge output", "", "## Z table", "", "| # | comparison | models (seeds) | baseline | verdict |",
             "|---|---|---|---|---|"]
    for z in ("Z1", "Z1b", "Z2", "Z3"):
        x = r["Z"][z]
        seeds = ", ".join(fmt(v) for v in x["seeds"].values())
        lines.append(f"| {z} | {x['label']} | {fmt(x['mean'])} ({seeds}) | {x['against']} {fmt(x['cal'])} | **{x['verdict']}** |")
    lines.append("| Z4 | per-variable skill, T2-any 168 h | see below | `cal_best` / `apfit_abs` | reported |")
    lines.append("| Z5 | world identification probe | see below | raw-row ridge | reported |")
    for which in ("raw", "bias_removed"):
        t = r["Z"]["Z4"][which]
        refs = list(t["against"])
        lines += ["", f"### Z4 {which} (against {', '.join(f'{k} = `{v}`' for k, v in t['against'].items())})", "",
                  "| variable | M1b nMAE | " + " | ".join(f"{k} nMAE | skill vs {k}" for k in refs) + " |",
                  "|---|---:|" + "---:|---:|" * len(refs)]
        for label, row in t["per_variable"].items():
            lines.append(f"| {label} | {fmt(row['model_nmae'])} | " + " | ".join(
                f"{fmt(row[f'{k}_nmae'])} | {fmt(row[f'skill_vs_{k}'])}" for k in refs) + " |")
    z = r["Z"]["Z5"]
    if "per_seed" in z:
        params = list(z["mean"])
        lines += ["", "### Z5 probe (test; R², climate: accuracy)", "", "| | " + " | ".join(params) + " |",
                  "|---|" + "---:|" * len(params)]
        for run, v in z["per_seed"].items():
            lines.append(f"| {run} | " + " | ".join(fmt(v[p]) for p in params) + " |")
        lines.append("| seed mean | " + " | ".join(fmt(z["mean"][p]) for p in params) + " |")
        lines.append("| raw observable row (reference) | " + " | ".join(fmt(z["reference_raw_row"][p]) for p in params) + " |")
    for ds, tabs in r["tables"].items():
        for name, t in tabs.items():
            lines += ["", f"## {ds} — {name} (test; cal_best `{t['selection']['cal_best']}`, cal_best_fair "
                      f"`{t['selection']['cal_best_fair']}`)", "", "| predictor | " + " | ".join(VARS) + " | mean |",
                      "|---|" + "---:|" * (len(VARS) + 1)]
            for n, v in t["rows"].items():
                if v:
                    lines.append(f"| {n} | " + " | ".join(fmt(x) for x in v["per_var"]) + f" | {fmt(v['mean'])} |")
    return "\n".join(lines) + "\n"


def run(runs: Path, v16_cal: str, v15_cal: str) -> dict:
    data = {}
    for ds, sha, names in (("p10_v16", v16_cal, M1B_V16), ("p10_v15", v15_cal, M1B_V15)):
        cal_val = load(runs / f"calendar_val_{ds}_{sha}" / "calendar_val.json")
        cal_test = load(runs / f"calendar_{ds}_{sha}" / "calendar.json")
        models = {n: m for n in names if (m := load(runs / n / "metrics.json"))}
        data[ds] = (cal_val, cal_test, models)
    v16, v15 = data["p10_v16"], data["p10_v15"]
    z = {"Z1": z_rule(v16[2], v16[1], v16[0], "T2_any", "168", False) | {"label": "v16 raw T2 168 h every-hour"},
         "Z1b": z_rule(v16[2], v16[1], v16[0], "T2_any", "24", False) | {"label": "v16 raw T2 24 h every-hour"},
         "Z2": z_rule(v16[2], v16[1], v16[0], "T2_any", "168", True) | {"label": "v16 bias-removed T2 168 h every-hour"},
         "Z3": z_rule(v15[2], v15[1], v15[0], "T2_any", "168", False) | {"label": "v15 control, raw T2 168 h every-hour"},
         "Z4": z4(v16[2], v16[1], v16[0]),
         "Z5": z5({n: p for n in M1B_V16 if (p := load(runs / f"probe_{n}" / "probe.json"))})}
    return {"Z": z, "tables": {ds: pilot_tables(d[1], d[0], d[2]) for ds, d in data.items()}}


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--runs", default="docs/worldmodel/wm1/runs")
    ap.add_argument("--v16-cal", required=True)
    ap.add_argument("--v15-cal", required=True)
    ap.add_argument("--out", required=True)
    args = ap.parse_args(argv)
    r = run(Path(args.runs), args.v16_cal, args.v15_cal)
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    (out / "judge.json").write_text(json.dumps(r, indent=1, sort_keys=True, default=float) + "\n")
    (out / "judge.md").write_text(markdown(r))
    print(json.dumps({k: v.get("verdict") for k, v in r["Z"].items()}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
