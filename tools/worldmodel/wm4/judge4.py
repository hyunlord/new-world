"""The WM-4 verdicts (docs/worldmodel/wm4/METRICS.md part b, GOAL.md 8) from the evaluation files copied to
`docs/worldmodel/wm4/runs/`, beside WM-2's and WM-3's committed files, and the analysis tables. numpy only; writes
`judge.json` and `judge.md`.

- O1 / O2 / O3 / B1 / B2 / A1: `wm2.effects.compare` / `rule` on cell errors (world-level paired bootstrap, B = 2,000,
  `default_rng(0)`; the model side's seeds averaged, multi-seed baselines averaged with `average_errors`). The obs split
  uses the floored scales of `obs_base.json`; v2 uses WM-2's `effects_base.json` scales.
- O4: `naive`, `ipw` and `zero` on the obs test (reported, with paired intervals).
- H1: `wm2.judge_wm2.pw_verdict` on per-world scores (WM-3's V3 / V4 construction) of the recalibrated heads; Brier and
  the 5-bin calibration of the original and recalibrated heads on val and test.

usage: python3 tools/worldmodel/wm4/judge4.py [--runs DIR] [--out DIR]
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm4  # noqa: F401
from wm1 import bootstrap as bs
from wm1 import common as c
from wm2 import effects as ef
from wm2 import judge_wm2 as jw
from wm2 import v2
from wm3 import judge3 as j3
from wm4 import effects4 as e4

RUNS = c.REPO / "docs/worldmodel/wm4/runs"
WM2 = c.REPO / "docs/worldmodel/wm2/runs"
WM3 = c.REPO / "docs/worldmodel/wm3/runs"
SEEDS2 = (0, 1)
SEEDS3 = (0, 1, 2)
OBS_MODELS = ("crn_obs", "ct_obs", "m1b_obs")
V2_MODELS = ("crn_v2", "ct_v2")
REFS = tuple(f"ref_wm2_m1bact_v19_s{s}_obs" for s in SEEDS3) + tuple(f"ref_wm3_full_s{s}_obs" for s in SEEDS3)
FULL = tuple(f"wm3_full_s{s}" for s in SEEDS3)


def load(p: Path) -> dict | None:
    return json.loads(p.read_text()) if p.exists() else None


def ci_text(ci: dict, dd: int = 3) -> str:
    return f"{ci['point']:+.{dd}f} [{ci['lo']:+.{dd}f}, {ci['hi']:+.{dd}f}]"


def fmt(x, dd: int = 3) -> str:
    return "—" if x is None or (isinstance(x, float) and not np.isfinite(x)) else f"{x:.{dd}f}"


def paired_cells(a: dict, b: dict, cells: list, worlds: list[int]) -> dict:
    """A reported comparison: cell-averaged a − b with the world bootstrap (no verdict)."""
    r = ef.compare([a], b, cells, worlds)
    return {"a": r["model"], "b": r["baseline"], "ci_diff": r["ci_diff"]}


# ------------------------------------------------------------------ obs split --------------------------------------
def obs_block(runs: Path) -> dict:
    base = load(runs / "obs_base" / "obs_base.json")
    if base is None:
        return {}
    val, test, fl, un = e4.load_obs_base(runs / "obs_base" / "obs_base.json")
    worlds = sorted({x.seed for x in test})
    preds = dict(base["test_predictions"])
    meta = {}
    for kind in OBS_MODELS:
        for s in SEEDS2:
            e = load(runs / f"{kind}_s{s}_effects" / "effects.json")
            if e is not None:
                preds[f"{kind}_s{s}"] = e["predictions"]
                meta[f"{kind}_s{s}"] = {k: e.get(k) for k in ("test_fact_roll", "round_counts", "drift", "code_commit")}
    for r in REFS:
        e = load(runs / r / "effects.json")
        if e is not None:
            preds[r] = e["predictions"]
    errs = {n: ef.cell_errors(test, p, fl) for n, p in preds.items()}
    errs_un = {n: ef.cell_errors(test, p, un) for n, p in preds.items()}
    sel, sel_un = base["selection"], base["selection_unfloored"]
    out = {"selection": sel, "selection_unfloored": sel_un, "worlds": len(worlds), "cases": len(test), "rules": {},
           "secondary": {}, "meta": meta, "estimates": base["estimates"],
           "tables": {n: e4.table(test, p, fl) for n, p in preds.items()},
           "tables_unfloored": {n: e4.table(test, p, un) for n, p in preds.items()},
           "distribution_test": base["distribution"]["test"], "scales": base["scales"],
           "scales_unfloored": base["scales_unfloored"]}
    seeds_of = lambda kind: [f"{kind}_s{s}" for s in SEEDS2 if f"{kind}_s{s}" in errs]
    for rule, lag in (("O1", "7d"), ("O2", "30d")):
        cells = e4.CELLS[lag]
        b = sel[lag]["chosen"]
        out["rules"][rule] = {}
        out["secondary"][rule + "_unfloored"] = {}
        for kind in OBS_MODELS:
            ss = seeds_of(kind)
            if not ss:
                continue
            out["rules"][rule][kind] = ef.compare([errs[s] for s in ss], errs[b], cells, worlds) | {
                "baseline_member": b, "runs": ss}
            bu = sel_un[lag]["chosen"]
            out["secondary"][rule + "_unfloored"][kind] = ef.compare([errs_un[s] for s in ss], errs_un[bu], cells, worlds) | {
                "baseline_member": bu, "runs": ss}
    full = [f"ref_wm3_full_s{s}_obs" for s in SEEDS3 if f"ref_wm3_full_s{s}_obs" in errs]
    out["rules"]["O3"] = {}
    for lag in ("7d", "30d"):
        for kind in OBS_MODELS:
            ss = seeds_of(kind)
            if not full or not ss:
                continue
            base_avg = ef.average_errors([errs[s] for s in ss])
            out["rules"]["O3"][f"{kind}|{lag}"] = ef.compare([errs[r] for r in full], base_avg, e4.CELLS[lag], worlds) | {
                "baseline_member": f"{kind} (seeds {', '.join(s[-1] for s in ss)})", "runs": full}
    o4 = {}
    for lag in ("7d", "30d"):
        cells = e4.CELLS[lag]
        o4[lag] = {"cells": {n: ef.cell_stat(errs[n], cells, worlds) for n in e4.BASELINES},
                   "naive_vs_zero": paired_cells(errs["naive"], errs["zero"], cells, worlds),
                   "ipw_vs_zero": paired_cells(errs["ipw"], errs["zero"], cells, worlds),
                   "ipw_vs_naive": paired_cells(errs["ipw"], errs["naive"], cells, worlds),
                   "per_arm": {a: {n: ef.cell_stat(errs[n], [x for x in cells if x[0] == e4.TO_V2[a]], worlds)
                                   for n in e4.BASELINES} for a in e4.OBS_ARMS}}
    out["rules"]["O4"] = o4
    out["cell_averages"] = {n: {lag: ef.cell_stat(errs[n], e4.CELLS[lag], worlds) for lag in e4.LAGS} for n in preds}
    out["cell_averages_unfloored"] = {n: {lag: ef.cell_stat(errs_un[n], e4.CELLS[lag], worlds) for lag in e4.LAGS}
                                      for n in preds}
    return out


# ------------------------------------------------------------------ v2 ---------------------------------------------
def v2_block(runs: Path) -> dict:
    base = load(WM2 / "effects_base" / "effects_base.json")
    stds = ef.stds_from(base["stds"])
    cases = [ef.case_from(j) for j in base["cases"]["test"]]
    worlds = sorted({x.seed for x in cases})
    preds = dict(base["test_predictions"])
    for s in SEEDS3:
        for name, root in ((f"wm3_full_s{s}", WM3), (f"wm2_m1bact_v19_s{s}", WM2)):
            e = load(root / f"{name}_effects" / "effects.json")
            if e is not None:
                preds[name] = e["predictions"]
        for kind in ("ctrl", "os"):
            name = f"wm3_{kind}_s{s}"
            e = load((WM3 if s == 0 else runs) / f"{name}_effects" / "effects.json")
            if e is not None:
                preds[name] = e["predictions"]
    meta = {}
    for kind in V2_MODELS:
        for s in SEEDS2:
            e = load(runs / f"{kind}_s{s}_effects" / "effects.json")
            if e is not None:
                preds[f"{kind}_s{s}"] = e["predictions"]
                meta[f"{kind}_s{s}"] = {"test_fact_roll": e.get("test_fact_roll"), "code_commit": e.get("code_commit")}
    errs = {n: ef.cell_errors(cases, p, stds) for n, p in preds.items()}
    cells = {lag: [(a, o, lag) for a in v2.W3_ARMS for o in v2.OUTCOMES if stds[(a, o, lag)] is not None]
             for lag in v2.LAGS}
    full = [r for r in FULL if r in errs]
    full_avg = ef.average_errors([errs[r] for r in full]) if full else None
    out = {"rules": {"B1": {}, "B2": {}, "A1": {}}, "meta": meta, "worlds": len(worlds), "cases": len(cases)}
    for kind in V2_MODELS:
        ss = [f"{kind}_s{s}" for s in SEEDS2 if f"{kind}_s{s}" in errs]
        if not ss:
            continue
        out["rules"]["B1"][f"{kind}|an5_arm"] = ef.compare([errs[s] for s in ss], errs["an5_arm"], cells["7d"], worlds) | {
            "baseline_member": "an5_arm", "runs": ss}
        if full_avg is not None:
            out["rules"]["B1"][f"{kind}|wm3_full"] = ef.compare([errs[s] for s in ss], full_avg, cells["7d"], worlds) | {
                "baseline_member": "wm3_full (seeds 0, 1, 2)", "runs": ss}
        out["rules"]["B2"][f"{kind}|zero"] = ef.compare([errs[s] for s in ss], errs["zero"], cells["30d"], worlds) | {
            "baseline_member": "zero", "runs": ss}
    for kind in ("ctrl", "os"):
        abl = [f"wm3_{kind}_s{s}" for s in SEEDS3 if f"wm3_{kind}_s{s}" in errs]
        if not abl or not full:
            continue
        avg = ef.average_errors([errs[r] for r in abl])
        for lag in ("7d", "30d"):
            out["rules"]["A1"][f"full_vs_{kind}|{lag}"] = ef.compare([errs[r] for r in full], avg, cells[lag], worlds) | {
                "baseline_member": f"wm3_{kind} (seeds {', '.join(r[-1] for r in abl)})", "runs": full}
    out["tables"] = {n: ef.score(cases, p, stds) for n, p in preds.items()}
    out["cell_averages"] = {n: {lag: ef.cell_stat(errs[n], cells[lag], worlds) for lag in v2.LAGS} for n in preds}
    return out


# ------------------------------------------------------------------ hazard -----------------------------------------
def hazard_block(runs: Path) -> dict:
    base = load(WM2 / "effects_base" / "effects_base.json")
    split_base = load(WM2 / "split_base" / "split_base.json")
    stds = ef.stds_from(base["stds"])
    cases = [ef.case_from(j) for j in base["cases"]["test"]]
    worlds = sorted({x.seed for x in cases})
    out = {"recal": {}, "calib": {}, "rules": {}}
    v3_pw, v4_pw, used = [], [], []
    an5 = ef.cell_errors(cases, base["test_predictions"]["an5_arm"], stds)
    for s in SEEDS3:
        rc = load(runs / f"wm4_haz_s{s}" / "recal.json")
        if rc is None:
            continue
        out["recal"][s] = {k: rc[k] for k in ("fit", "train_brier", "kept", "train_pred_mean")} | {
            "val_original": rc["val_original"], "val_recal": rc["val_recal"]}
        name = f"wm4_haz_s{s}" if rc["kept"] else f"wm3_full_s{s}"
        root = runs if rc["kept"] else WM3
        eff = load(root / f"{name}_effects" / "effects.json")
        sp = load(root / f"{name}_split" / "split.json")
        if eff is None or sp is None:
            continue
        exp = {k: dict(v, shift=v["shift_expected"]) for k, v in eff["predictions"].items()}
        v3_pw.append(j3.shift_per_world(ef.cell_errors(cases, exp, stds), worlds))
        v4_pw.append(sp["expected"]["per_world"])
        used.append(name)
        out["calib"][s] = {"test_recal": load(runs / f"wm4_haz_s{s}_calib" / "calib_test.json"),
                           "test_original": load(runs / f"wm3_full_s{s}_calib" / "calib_test.json")}
        for k in ("test_recal", "test_original"):
            if out["calib"][s][k]:
                out["calib"][s][k].pop("per_row", None)
    if used:
        out["rules"]["H1_timing"] = jw.pw_verdict(v4_pw, split_base["scores"]["test"][split_base["selection"]]["per_world"]) | {
            "baseline_member": split_base["selection"], "runs": used}
        out["rules"]["H1_shift"] = jw.pw_verdict(v3_pw, j3.shift_per_world(an5, worlds)) | {
            "baseline_member": "an5_arm", "runs": used}
        out["rules"]["H1_shift"]["right_direction"] = {}
        for name in used:
            root = runs if name.startswith("wm4") else WM3
            eff = load(root / f"{name}_effects" / "effects.json")
            exp = {k: dict(v, shift=v["shift_expected"]) for k, v in eff["predictions"].items()}
            sc = ef.score(cases, exp, stds)["cells"]
            out["rules"]["H1_shift"]["right_direction"][name] = {a: sc[f"{a}|shift"]["right_direction"] for a in v2.W4_ARMS}
    wm3 = load(WM3 / "judge3" / "judge.json")
    if wm3:
        out["wm3_V3"], out["wm3_V4"] = wm3["V"].get("V3"), wm3["V"].get("V4")
    return out


def runs_block(runs: Path) -> dict:
    out = {}
    for d_ in sorted(runs.iterdir()) if runs.exists() else []:
        cfg, val = load(d_ / "config.json"), load(d_ / "val_metrics.json")
        if cfg is None or val is None or (d_ / "recal.json").exists():
            continue
        node = {"seconds": val.get("seconds"), "stop": val.get("stop"), "best_step": val.get("best_step"),
                "last_step": val.get("last_step"), "selected": val.get("selected")}
        if "method" in cfg:                                   # CRN / CT: stop and best epoch may be per stage
            stage = val.get("decoder", val)
            node["stop"] = stage.get("stop")
            node["best_step"] = f"epoch {stage.get('best_epoch')}" + (" (decoder)" if "decoder" in val else "")
            node |= {"method": cfg["method"], "dataset": cfg["dataset"], "overrides": cfg.get("overrides"),
                     "val_roll": val.get("val_roll"), "result": {k: v for k, v in val.items() if k != "seconds"}}
        else:
            node |= {"model": cfg.get("model"), "dataset": cfg.get("dataset"), "best_val_t2_24": val.get("best_val_t2_24"),
                     "best_val_eff": val.get("best_val_eff"), "init_val_eff": val.get("init_val_eff"),
                     "not_converged": val.get("not_converged")}
        for f in ("train.start", "train.end"):
            if (d_ / f).exists():
                node[f] = int((d_ / f).read_text().split()[0])
        out[d_.name] = node
    return out


def factual_block(runs: Path) -> dict:
    out = {}
    for s in SEEDS2:
        m = load(runs / f"m1b_obs_s{s}_fact" / "metrics.json")
        if m is not None:
            out[f"m1b_obs_s{s}"] = {h: float(np.mean(list(bs.world_scores(m["T2_any"][h]["test"]).values())))
                                    for h in ("24", "168")}
    return out


def build(runs: Path) -> dict:
    j = {"obs": obs_block(runs), "v2": v2_block(runs), "hazard": hazard_block(runs), "runs": runs_block(runs),
         "factual_obs": factual_block(runs)}
    for blk in ("obs", "v2"):
        j[blk].pop("_", None)
    return j


# ------------------------------------------------------------------ markdown ---------------------------------------
def v_row(name: str, w: dict, dd: int = 3) -> str:
    if "ci_diff" not in w:
        return f"| {name} | — | — | — | — | **{w.get('verdict', '—')}** |"
    seeds = ", ".join(fmt(s, dd) for s in w["seeds"])
    return (f"| {name} | {fmt(w['model'], dd)} ({seeds}) | `{w['baseline_member']}` {fmt(w['baseline'], dd)} | "
            f"{fmt(w.get('ratio'), 3)} | {ci_text(w['ci_diff'], dd)} | **{w['verdict']}** |")


def markdown(j: dict) -> str:
    L = ["# WM-4 judge (generated by `tools/worldmodel/wm4/judge4.py`)", "",
         "| rule | model (seeds) | baseline | ratio | Δ = model − baseline [95 % CI] | verdict |",
         "|---|---|---|---|---|---|"]
    o, b2, hz = j["obs"], j["v2"], j["hazard"]
    for rule in ("O1", "O2"):
        for kind, w in o.get("rules", {}).get(rule, {}).items():
            L.append(v_row(f"{rule} obs {'7 d' if rule == 'O1' else '30 d'}: `{kind}`", w))
    for key, w in o.get("rules", {}).get("O3", {}).items():
        kind, lag = key.split("|")
        L.append(v_row(f"O3 obs {lag}: `wm3_full` vs `{kind}`", w))
    for rule in ("B1", "B2"):
        for key, w in b2["rules"].get(rule, {}).items():
            L.append(v_row(f"{rule} v2 {'7 d' if rule == 'B1' else '30 d'}: `{key.split('|')[0]}`", w))
    for key, w in b2["rules"].get("A1", {}).items():
        L.append(v_row(f"A1 v2 {key.split('|')[1]}: `wm3_full` vs `{key.split('|')[0][8:]}`", w))
    for key in ("H1_timing", "H1_shift"):
        if key in hz["rules"]:
            L.append(v_row(f"{key.replace('_', ' ')} (days)", hz["rules"][key], 2))
    if o:
        L += ["", f"Obs baselines chosen on the val families (12-cell average, floored scales): 7 d `{o['selection']['7d']['chosen']}`, "
              f"30 d `{o['selection']['30d']['chosen']}`; val values " + "; ".join(
                  f"{lag}: " + ", ".join(f"{n} {fmt(v)}" for n, v in o["selection"][lag]["val"].items()) for lag in ("7d", "30d")),
              f"Unfloored scales: 7 d `{o['selection_unfloored']['7d']['chosen']}`, 30 d `{o['selection_unfloored']['30d']['chosen']}`.", "",
              "## O4 — confounding visible (obs test, 12-cell averages; Δ with world-bootstrap 95 % CI)", "",
              "| lag | zero | naive | ipw | match5 | naive − zero | ipw − zero | ipw − naive |", "|---|---|---|---|---|---|---|---|"]
        for lag, v in o["rules"]["O4"].items():
            cc = v["cells"]
            L.append(f"| {lag} | {fmt(cc['zero'])} | {fmt(cc['naive'])} | {fmt(cc['ipw'])} | {fmt(cc['match5'])} | "
                     f"{ci_text(v['naive_vs_zero']['ci_diff'])} | {ci_text(v['ipw_vs_zero']['ci_diff'])} | {ci_text(v['ipw_vs_naive']['ci_diff'])} |")
        L += ["", "Per arm (4-cell averages):", "", "| lag | arm | zero | naive | ipw | match5 |", "|---|---|---|---|---|---|"]
        for lag, v in o["rules"]["O4"].items():
            for a, pa in v["per_arm"].items():
                L.append(f"| {lag} | {a} | " + " | ".join(fmt(pa[n]) for n in e4.BASELINES) + " |")
        L += ["", "Pooled estimates on the observed train worlds (food store; `naive` and `ipw` are constants per arm and lag):", "",
              "| arm | lag | naive | ipw | treated | ESS treated | controls | ESS controls | max weight treated / controls |",
              "|---|---|---|---|---|---|---|---|---|"]
        est = o["estimates"]
        for a in e4.OBS_ARMS:
            for lag in e4.LAGS:
                cn = est["counts"][a][lag]
                L.append(f"| {a} | {lag} | {fmt(est['naive'][a][lag]['food_stock'], 1)} | {fmt(est['ipw'][a][lag]['food_stock'], 1)} | "
                         f"{cn['treated']} | {fmt(cn['ess_treated'], 1)} | {cn['controls']} | {fmt(cn['ess_controls'], 1)} | "
                         f"{fmt(cn['max_weight_treated'], 1)} / {fmt(cn['max_weight_controls'], 2)} |")
        L += ["", "## Obs test: 12-cell averages of every method (floored scales; unfloored in brackets)", "",
              "| predictor | 7 d | 30 d |", "|---|---|---|"]
        for n, v in o["cell_averages"].items():
            u = o["cell_averages_unfloored"][n]
            L.append(f"| `{n}` | {fmt(v['7d'])} ({fmt(u['7d'])}) | {fmt(v['30d'])} ({fmt(u['30d'])}) |")
        preds = list(o["tables"])
        L += ["", "## Obs test: per cell (scaled effect MAE / sign accuracy; true mean (std))", "",
              "| arm | outcome | lag | truth | " + " | ".join(f"`{p}`" for p in preds) + " |", "|---|---|---|---|" + "---|" * len(preds)]
        for a in e4.OBS_ARMS:
            for out_ in v2.OUTCOMES:
                for lag in e4.LAGS:
                    key = f"{a}|{out_}|{lag}"
                    dist = o["distribution_test"].get(a, {}).get(f"{out_}|{lag}", {})
                    L.append(f"| {a} | {out_} | {lag} | {fmt(dist.get('mean'), 2)} ({fmt(dist.get('std'), 2)}) | " + " | ".join(
                        (f"{fmt(o['tables'][p][key]['mae'])} / {fmt(o['tables'][p][key]['sign_accuracy'], 2)}" if key in o["tables"][p] else "—")
                        for p in preds) + " |")
        L += ["", "Mean predicted vs true food-store effect (obs test):", "", "| arm | lag | true | " +
              " | ".join(f"`{p}`" for p in preds) + " |", "|---|---|---|" + "---|" * len(preds)]
        for a in e4.OBS_ARMS:
            for lag in e4.LAGS:
                key = f"{a}|food_stock|{lag}"
                L.append(f"| {a} | {lag} | {fmt(o['tables'][preds[0]][key]['mean_true'], 1)} | " +
                         " | ".join(fmt(o["tables"][p][key]["mean_pred"], 1) for p in preds) + " |")
    L += ["", "## v2 test: 12-cell averages (WM-2 scales)", "", "| predictor | 7 d | 30 d |", "|---|---|---|"]
    for n, v in b2["cell_averages"].items():
        L.append(f"| `{n}` | {fmt(v['7d'])} | {fmt(v['30d'])} |")
    if hz.get("recal"):
        L += ["", "## Hazard head (H1): recalibration and calibration", "",
              "| seed | a | b | kept | train Brier before → after (clim.) | val Brier orig → recal (clim.) | val mean pred orig → recal (obs) | "
              "test Brier orig → recal (clim.) | test mean pred orig → recal (obs) |", "|---|---|---|---|---|---|---|---|---|"]
        for s, r in hz["recal"].items():
            ct = hz["calib"].get(s, {})
            to, tr = ct.get("test_original") or {}, ct.get("test_recal") or {}
            L.append(f"| {s} | {fmt(r['fit']['a'])} | {fmt(r['fit']['b'])} | {r['kept']} | {fmt(r['train_brier']['before'])} → "
                     f"{fmt(r['train_brier']['after'])} ({fmt(r['train_brier']['climatology'])}) | {fmt(r['val_original']['brier'])} → "
                     f"{fmt(r['val_recal']['brier'])} ({fmt(r['val_recal']['brier_climatology'])}) | {fmt(r['val_original']['pred_mean'])} → "
                     f"{fmt(r['val_recal']['pred_mean'])} ({fmt(r['val_recal']['obs_rate'])}) | {fmt(to.get('brier'))} → {fmt(tr.get('brier'))} "
                     f"({fmt(tr.get('brier_climatology'))}) | {fmt(to.get('pred_mean'))} → {fmt(tr.get('pred_mean'))} ({fmt(tr.get('obs_rate'))}) |")
        for s, ct in hz["calib"].items():
            for k in ("test_original", "test_recal"):
                cal = ct.get(k)
                if cal:
                    L.append(f"- s{s} {k}: " + "; ".join(f"bin {b_['bin']} pred {fmt(b_['pred_mean'])} obs {fmt(b_['obs_rate'])} (n {b_['n']})"
                                                       for b_ in cal["bins"]))
        if hz["rules"].get("H1_shift", {}).get("right_direction"):
            L.append("- H1 split-shift right-direction share: " + "; ".join(
                f"`{r}` " + ", ".join(f"{a} {fmt(x, 2)}" for a, x in d_.items()) for r, d_ in hz["rules"]["H1_shift"]["right_direction"].items()))
    if j.get("factual_obs"):
        L += ["", "M1b-obs factual T2 (raw, every hour, obs test): " + "; ".join(
            f"`{r}` 24 h {fmt(v['24'], 4)}, 168 h {fmt(v['168'], 4)}" for r, v in j["factual_obs"].items())]
    L += ["", "## Runs", "", "| run | stop | best | seconds | val | detail |", "|---|---|---|---|---|---|"]
    for n, r in j["runs"].items():
        val = r.get("val_roll") if "method" in r else r.get("best_val_eff") if r.get("best_val_eff") is not None else r.get("best_val_t2_24")
        detail = json.dumps(r.get("result"))[:300] if "method" in r else f"selected {r.get('selected')}, not converged {r.get('not_converged')}"
        L.append(f"| `{n}` | {r.get('stop')} | {r.get('best_step')} | {fmt(r.get('seconds'), 0)} | {fmt(val, 4)} | {detail} |")
    return "\n".join(L) + "\n"


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--runs", default=str(RUNS))
    ap.add_argument("--out", default=str(RUNS / "judge4"))
    a = ap.parse_args(argv)
    j = build(Path(a.runs))
    c.write_json(Path(a.out) / "judge.json", j)
    (Path(a.out) / "judge.md").write_text(markdown(j))
    print(markdown(j))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
