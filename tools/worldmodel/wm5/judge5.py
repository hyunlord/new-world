"""The WM-5 verdicts (docs/worldmodel/wm5/METRICS.md part b, GOAL.md 8, 10) from the evaluation files copied to
`docs/worldmodel/wm5/runs/`, beside WM-2 … WM-4's committed files, plus the headroom ladder and the per-cell tables.
numpy only; writes `judge.json` and `judge.md`.

- R1 / R2 / R3 / E1 / E2 / D1: `wm2.effects.compare` (world-level paired bootstrap, B = 2,000, `default_rng(0)`; the
  model side's seeds averaged; a multi-seed baseline averaged with `average_errors`) and the W1 rule. obs2 uses WM-4's
  floored scales (`obs2_base.json`), v2 WM-2's (`effects_base.json`).
- R4: from each CRN / CT run's `val_metrics.json` (`reached`).
- F1 / F2 (the headroom ladder): `zero` − floor, `zero` − `ridge_obs`, `ridge_obs` − `ridge_priv`, `ridge_priv` − floor
  with world-bootstrap 95 % CIs (reported, no verdict); every rung on the same cases.
- D2: WM-4's committed CRN / CT predictions vs the rounded re-score on their own test splits.

usage: python3 tools/worldmodel/wm5/judge5.py [--runs DIR] [--out DIR]
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm5  # noqa: F401
from wm1 import bootstrap as bs
from wm1 import common as c
from wm2 import effects as ef
from wm2 import v2
from wm4 import effects4 as e4

RUNS = c.REPO / "docs/worldmodel/wm5/runs"
WM2 = c.REPO / "docs/worldmodel/wm2/runs"
WM3 = c.REPO / "docs/worldmodel/wm3/runs"
WM4 = c.REPO / "docs/worldmodel/wm4/runs"
SEEDS2 = (0, 1)
SEEDS3 = (0, 1, 2)
OBS2_MODELS = ("crn_obs2", "ct_obs2", "m1b_obs2")
V2_MODELS = ("crn_v2", "ct_v2")
CAUSAL = ("crn_obs2", "ct_obs2", "crn_v2", "ct_v2")
M1_REFS = tuple(f"wm2_m1bact_v19_s{s}" for s in SEEDS3) + tuple(f"wm3_full_s{s}" for s in SEEDS3) + \
    tuple(f"m1b_obs_s{s}" for s in SEEDS2)
CAUSAL_REFS = tuple(f"{k}_obs_s{s}" for k in ("crn", "ct") for s in SEEDS2)
LADDER = (("zero", "floor"), ("zero", "ridge_obs"), ("ridge_obs", "ridge_priv"), ("ridge_priv", "floor"))


def load(p: Path) -> dict | None:
    return json.loads(p.read_text()) if p.exists() else None


def ci_text(ci: dict, dd: int = 3) -> str:
    return f"{ci['point']:+.{dd}f} [{ci['lo']:+.{dd}f}, {ci['hi']:+.{dd}f}]"


def fmt(x, dd: int = 3) -> str:
    return "—" if x is None or (isinstance(x, float) and not np.isfinite(x)) else f"{x:.{dd}f}"


def paired(a: dict, b: dict, cells: list, worlds: list[int]) -> dict:
    """A reported comparison: cell-averaged a − b with the world bootstrap (no verdict)."""
    r = ef.compare([a], b, cells, worlds)
    return {"a": r["model"], "b": r["baseline"], "ci_diff": r["ci_diff"]}


def restrict(errs: dict, keep: set) -> dict:
    """Cell errors restricted to the (seed, rep) cases in `keep`."""
    return {cell: [r for r in rows if (r[0], r[1]) in keep] for cell, rows in errs.items()}


def verdicts(errs: dict, model: str, seeds: list[str], base_errs: dict, base_name: str, cells: list,
             worlds: list[int]) -> dict:
    return ef.compare([errs[s] for s in seeds], base_errs, cells, worlds) | {"baseline_member": base_name, "runs": seeds}


def ladder(errs: dict, cells_of: dict, worlds: list[int]) -> dict:
    out = {}
    names = [n for n in ("zero", "ridge_obs", "ridge_priv", "floor") if n in errs]
    for lag, cells in cells_of.items():
        node = {"values": {n: ef.cell_stat(errs[n], cells, worlds) for n in names}, "steps": {}}
        for a, b in LADDER:
            if a in errs and b in errs:
                node["steps"][f"{a} − {b}"] = paired(errs[a], errs[b], cells, worlds)
        out[lag] = node
    return out


# ------------------------------------------------------------------ obs2 -------------------------------------------
def obs2_block(runs: Path) -> dict:
    base = load(runs / "obs2_base" / "obs2_base.json")
    if base is None:
        return {}
    from wm5 import effects5 as e5
    val, test_all, fl, un = e5.load_obs2_base(runs / "obs2_base" / "obs2_base.json")
    test = [x for x in test_all if e5.ref_of(x.rep)[1] == 0]
    worlds = sorted({x.seed for x in test})
    preds = dict(base["test_predictions"])
    preds["floor"] = base["floor_predictions"]
    secondary, meta = {}, {}
    for kind in ("crn_obs2", "ct_obs2"):
        for s in SEEDS2:
            for wh in ("last", "best"):
                e = load(runs / f"{kind}_s{s}_effects" / f"effects_{wh}.json")
                if e is None:
                    continue
                if wh == "last":
                    preds[f"{kind}_s{s}"] = e["predictions"]
                secondary[f"{kind}_s{s}|{wh}|rounded"] = e["predictions"]
                secondary[f"{kind}_s{s}|{wh}|unrounded"] = e["predictions_unrounded"]
    for s in SEEDS2:
        e = load(runs / f"m1b_obs2_s{s}_effects" / "effects.json")
        if e is not None:
            preds[f"m1b_obs2_s{s}"] = e["predictions"]
            meta[f"m1b_obs2_s{s}"] = {k: e.get(k) for k in ("round_counts", "drift", "code_commit")}
    for r in M1_REFS:
        e = load(runs / f"ref_{r}_obs2" / "effects.json")
        if e is not None:
            preds[f"ref_{r}"] = e["predictions"]
    for r in CAUSAL_REFS:
        e = load(runs / f"ref_{r}_obs2" / "effects_last.json")
        if e is not None:
            preds[f"ref_{r}"] = e["predictions_unrounded"]           # as kept by WM-4 (unrounded)
            preds[f"ref_{r}|rounded"] = e["predictions"]
    for name in ("ridge_obs", "ridge_priv"):
        e = load(runs / name / f"{name}.json")
        if e is not None:
            preds[name] = e["obs2_test_predictions"]
    keep = {(x.seed, x.rep) for x in test}
    errs = {n: restrict(ef.cell_errors(test, p, fl), keep) for n, p in preds.items()}
    errs_un = {n: restrict(ef.cell_errors(test, p, un), keep) for n, p in preds.items()}
    errs_sec = {n: restrict(ef.cell_errors(test, p, fl), keep) for n, p in secondary.items()}
    sel = base["selection"]
    out = {"selection": sel, "selection_unfloored": base["selection_unfloored"], "worlds": len(worlds),
           "cases": len(test), "floor_predictions_all_replicates": len(base["floor_predictions"]), "rules": {}, "meta": meta,
           "estimates": base["estimates"], "distribution_test": base["distribution"]["test"],
           "distribution_test_by_ref": base["distribution"].get("test_by_ref"), "scales": base["scales"]}
    seeds_of = lambda kind: [f"{kind}_s{s}" for s in SEEDS2 if f"{kind}_s{s}" in errs]
    for rule, lag in (("R1", "7d"), ("R2", "30d")):
        b = sel[lag]["chosen"]
        out["rules"][rule] = {kind: verdicts(errs, kind, seeds_of(kind), errs[b], b, e4.CELLS[lag], worlds)
                              for kind in OBS2_MODELS if seeds_of(kind)}
    full = [f"ref_wm3_full_s{s}" for s in SEEDS3 if f"ref_wm3_full_s{s}" in errs]
    if "ridge_obs" in errs:
        out["rules"]["E1"] = {lag: verdicts(errs, "ridge_obs", ["ridge_obs"], errs["zero"], "zero", e4.CELLS[lag], worlds)
                              for lag in e4.LAGS}
        out["rules"]["E2"] = {}
        for lag in e4.LAGS:
            for kind in OBS2_MODELS:
                if seeds_of(kind):
                    out["rules"]["E2"][f"{kind}|{lag}"] = verdicts(errs, kind, seeds_of(kind), errs["ridge_obs"],
                                                                   "ridge_obs", e4.CELLS[lag], worlds)
            if full:
                out["rules"]["E2"][f"wm3_full|{lag}"] = verdicts(errs, "wm3_full", full, errs["ridge_obs"], "ridge_obs",
                                                                 e4.CELLS[lag], worlds)
    d1 = {}
    for lag in e4.LAGS:
        cells = e4.CELLS[lag]
        ct = seeds_of("ct_obs2")
        for tag, suffix in (("as_kept", ""), ("rounded", "|rounded")):
            refs = [f"ref_ct_obs_s{s}{suffix}" for s in SEEDS2 if f"ref_ct_obs_s{s}{suffix}" in errs]
            if ct and refs:
                d1[f"ct_obs2_vs_ct_obs_{tag}|{lag}"] = verdicts(errs, "ct_obs2", ct, ef.average_errors([errs[r] for r in refs]),
                                                                f"ct_obs (WM-4, {tag})", cells, worlds)
        m1 = seeds_of("m1b_obs2")
        refs = [f"ref_m1b_obs_s{s}" for s in SEEDS2 if f"ref_m1b_obs_s{s}" in errs]
        if m1 and refs:
            d1[f"m1b_obs2_vs_m1b_obs|{lag}"] = verdicts(errs, "m1b_obs2", m1, ef.average_errors([errs[r] for r in refs]),
                                                        "m1b_obs (WM-4)", cells, worlds)
    out["rules"]["D1"] = d1
    lad_names = [n for n in ("zero", "ridge_obs", "ridge_priv", "floor") if n in errs]
    out["ladder"] = ladder({n: errs[n] for n in lad_names}, e4.CELLS, worlds)
    out["confounding"] = {lag: {"cells": {n: ef.cell_stat(errs[n], e4.CELLS[lag], worlds) for n in e4.BASELINES},
                                "naive_vs_zero": paired(errs["naive"], errs["zero"], e4.CELLS[lag], worlds),
                                "ipw_vs_zero": paired(errs["ipw"], errs["zero"], e4.CELLS[lag], worlds),
                                "ipw_vs_naive": paired(errs["ipw"], errs["naive"], e4.CELLS[lag], worlds)}
                          for lag in e4.LAGS}
    out["cell_averages"] = {n: {lag: ef.cell_stat(errs[n], e4.CELLS[lag], worlds) for lag in e4.LAGS} for n in errs}
    out["cell_averages_unfloored"] = {n: {lag: ef.cell_stat(errs_un[n], e4.CELLS[lag], worlds) for lag in e4.LAGS}
                                      for n in errs_un}
    out["cell_averages_by_ref"] = {ref: {n: {lag: ef.cell_stat(restrict(errs[n], {k for k in keep if
                                                                                  e5.ref_of(k[1])[0] == ref}),
                                                              e4.CELLS[lag], sorted(worlds)) for lag in e4.LAGS}
                                         for n in errs} for ref in ("c1", "c2")}
    out["secondary"] = {n: {lag: ef.cell_stat(errs_sec[n], e4.CELLS[lag], worlds) for lag in e4.LAGS} for n in errs_sec}
    out["tables"] = {n: e4.table(test, p, fl) for n, p in preds.items()}
    return out


# ------------------------------------------------------------------ v2 ---------------------------------------------
def v2_block(runs: Path) -> dict:
    base = load(WM2 / "effects_base" / "effects_base.json")
    stds = ef.stds_from(base["stds"])
    cases = [ef.case_from(j) for j in base["cases"]["test"]]
    worlds = sorted({x.seed for x in cases})
    from wm5 import effects5 as e5
    preds = dict(base["test_predictions"])
    preds["floor"] = {}
    by = {(x.seed, x.arm, x.rep): x for x in cases}
    for x in cases:                       # v2 floor: the other replicate of the family (WM-2's two replicates)
        o = by[(x.seed, x.arm, 1 - x.rep)]
        preds["floor"][x.key] = {"effects": {og: {lag: o.truth[og][lag] for lag in x.truth[og] if lag in o.truth[og]}
                                             for og in v2.OUTCOMES}, "shift": o.shift}
    for s in SEEDS3:
        for name, root in ((f"wm3_full_s{s}", WM3), (f"wm2_m1bact_v19_s{s}", WM2)):
            e = load(root / f"{name}_effects" / "effects.json")
            if e is not None:
                preds[name] = e["predictions"]
    secondary = {}
    for kind in V2_MODELS:
        for s in SEEDS2:
            for wh in ("last", "best"):
                e = load(runs / f"{kind}_s{s}_effects" / f"effects_{wh}.json")
                if e is None:
                    continue
                if wh == "last":
                    preds[f"{kind}_s{s}"] = e["predictions"]
                secondary[f"{kind}_s{s}|{wh}|rounded"] = e["predictions"]
                secondary[f"{kind}_s{s}|{wh}|unrounded"] = e["predictions_unrounded"]
    for name in ("ridge_obs", "ridge_priv"):
        e = load(runs / name / f"{name}.json")
        if e is not None:
            preds[name] = e["v2_test_predictions"]
    errs = {n: ef.cell_errors(cases, p, stds) for n, p in preds.items()}
    errs_sec = {n: ef.cell_errors(cases, p, stds) for n, p in secondary.items()}
    cells = {lag: [(a, o, lag) for a in v2.W3_ARMS for o in v2.OUTCOMES if stds[(a, o, lag)] is not None]
             for lag in v2.LAGS}
    full = [r for r in (f"wm3_full_s{s}" for s in SEEDS3) if r in errs]
    out = {"rules": {"R3": {}}, "worlds": len(worlds), "cases": len(cases)}
    for kind in V2_MODELS:
        ss = [f"{kind}_s{s}" for s in SEEDS2 if f"{kind}_s{s}" in errs]
        if not ss:
            continue
        for lag in v2.LAGS:
            for b in ("zero", "an5_arm"):
                out["rules"]["R3"][f"{kind}|{b}|{lag}"] = verdicts(errs, kind, ss, errs[b], b, cells[lag], worlds)
    if "ridge_obs" in errs:
        out["rules"]["E1"] = {lag: verdicts(errs, "ridge_obs", ["ridge_obs"], errs["zero"], "zero", cells[lag], worlds)
                              for lag in v2.LAGS}
        out["rules"]["E2"] = {}
        for lag in v2.LAGS:
            for kind in V2_MODELS:
                ss = [f"{kind}_s{s}" for s in SEEDS2 if f"{kind}_s{s}" in errs]
                if ss:
                    out["rules"]["E2"][f"{kind}|{lag}"] = verdicts(errs, kind, ss, errs["ridge_obs"], "ridge_obs",
                                                                   cells[lag], worlds)
            if full:
                out["rules"]["E2"][f"wm3_full|{lag}"] = verdicts(errs, "wm3_full", full, errs["ridge_obs"], "ridge_obs",
                                                                 cells[lag], worlds)
    out["ladder"] = ladder({n: errs[n] for n in ("zero", "ridge_obs", "ridge_priv", "floor") if n in errs}, cells, worlds)
    out["spread_wm2"] = {lag: float(np.mean([base["spread"]["test"]["|".join(cell)]["mean"] for cell in cells[lag]]))
                         for lag in v2.LAGS}
    out["cell_averages"] = {n: {lag: ef.cell_stat(errs[n], cells[lag], worlds) for lag in v2.LAGS} for n in errs}
    out["secondary"] = {n: {lag: ef.cell_stat(errs_sec[n], cells[lag], worlds) for lag in v2.LAGS} for n in errs_sec}
    out["tables"] = {n: ef.score(cases, p, stds)["cells"] for n, p in preds.items()}
    out["deaths_split"] = {n: {lag: {"deaths": ef.cell_stat(errs[n], [x for x in cells[lag] if x[1] == "deaths"], worlds),
                                     "other": ef.cell_stat(errs[n], [x for x in cells[lag] if x[1] != "deaths"], worlds)}
                               for lag in v2.LAGS} for n in errs}
    del e5
    return out


# ------------------------------------------------------------------ D2 / R4 / runs -----------------------------------
def d2_block(runs: Path) -> dict:
    out = {}
    obs = load(WM4 / "obs_base" / "obs_base.json")
    base = load(WM2 / "effects_base" / "effects_base.json")
    o_val, o_test, o_fl, _ = e4.load_obs_base(WM4 / "obs_base" / "obs_base.json") if obs else (None, None, None, None)
    v_cases = [ef.case_from(j) for j in base["cases"]["test"]]
    v_stds = ef.stds_from(base["stds"])
    v_cells = {lag: [(a, o, lag) for a in v2.W3_ARMS for o in v2.OUTCOMES if v_stds[(a, o, lag)] is not None]
               for lag in v2.LAGS}
    for split in ("obs", "v2"):
        cases, stds, cells = (o_test, o_fl, e4.CELLS) if split == "obs" else (v_cases, v_stds, v_cells)
        if cases is None:
            continue
        worlds = sorted({x.seed for x in cases})
        for kind in ("crn", "ct"):
            per = {}
            for s in SEEDS2:
                run = f"{kind}_{split}_s{s}"
                old = load(WM4 / f"{run}_effects" / "effects.json")
                new = load(runs / f"rescored_{run}" / "rescored_rounded.json")
                if old is None or new is None:
                    continue
                per[run] = {"wm4": ef.cell_errors(cases, old["predictions"], stds),
                            "rounded": ef.cell_errors(cases, new["predictions"], stds),
                            "unrounded": ef.cell_errors(cases, new["predictions_unrounded"], stds)}
            if not per:
                continue
            node = {"runs": sorted(per), "values": {}, "compare": {}}
            for lag in v2.LAGS:
                for r, e in per.items():
                    node["values"][f"{r}|{lag}"] = {k: ef.cell_stat(v, cells[lag], worlds) for k, v in e.items()} | {
                        "deaths": {k: ef.cell_stat(v, [x for x in cells[lag] if x[1] == "deaths"], worlds)
                                   for k, v in e.items()}}
                node["compare"][lag] = ef.compare([per[r]["rounded"] for r in sorted(per)],
                                                  ef.average_errors([per[r]["wm4"] for r in sorted(per)]), cells[lag],
                                                  worlds) | {"baseline_member": f"{kind}_{split} as committed by WM-4"}
            out[f"{kind}_{split}"] = node
    return out


def runs_block(runs: Path) -> dict:
    out = {}
    for d_ in sorted(runs.iterdir()) if runs.exists() else []:
        cfg, val = load(d_ / "config.json"), load(d_ / "val_metrics.json")
        if cfg is None or val is None:
            continue
        node = {"seconds": val.get("seconds")}
        if "plan" in cfg:                                              # a WM-5 CRN / CT run
            stages = ["encoder", "decoder"] if cfg["method"] == "crn" else ["ct"]
            node |= {"method": cfg["method"], "dataset": cfg["dataset"], "seed": cfg["seed"], "plan": cfg["plan"],
                     "stages": {s: val[s] for s in stages}, "reached": val.get("reached"), "val": val.get("val"),
                     "budget": cfg.get("budget"), "safety_cap_seconds": cfg.get("safety_cap_seconds"),
                     "device": cfg.get("device"), "torch": cfg.get("torch"), "code_commit": cfg.get("code_commit"),
                     "manifest_sha256": cfg.get("manifest_sha256")}
        else:
            node |= {"model": cfg.get("model"), "dataset": cfg.get("dataset"), "stop": val.get("stop"),
                     "best_step": val.get("best_step"), "last_step": val.get("last_step"),
                     "best_val_t2_24": val.get("best_val_t2_24"), "selected": val.get("selected"),
                     "not_converged": val.get("not_converged"), "round_counts": val.get("round_counts")}
        for f in ("train.start", "train.end"):
            if (d_ / f).exists():
                node[f] = int((d_ / f).read_text().split()[0])
        out[d_.name] = node
    return out


def r4_block(runs_info: dict) -> dict:
    out = {}
    for name, r in runs_info.items():
        if "plan" not in r or name.startswith("_"):
            continue
        rc = r["reached"] or {}
        ok = (rc.get("steps_fraction_min", 0) >= 0.9 and rc.get("alpha_fraction_min", 0) >= 0.9)
        out[name] = {"steps_fraction_min": rc.get("steps_fraction_min"), "alpha_fraction_min": rc.get("alpha_fraction_min"),
                     "planned_equal_authors": rc.get("planned_equal_authors"), "stop": rc.get("stop"),
                     "planned": r["plan"]["planned"], "steps": {s: v["steps"] for s, v in r["stages"].items()},
                     "verdict": "trained as the authors" if ok else "under-trained"}
    return out


def factual_block(runs: Path) -> dict:
    out = {}
    for s in SEEDS2:
        m = load(runs / f"m1b_obs2_s{s}_fact" / "metrics.json")
        if m is not None:
            out[f"m1b_obs2_s{s}"] = {h: float(np.mean(list(bs.world_scores(m["T2_any"][h]["test"]).values())))
                                     for h in ("24", "168")}
    return out


GROUPS = {
    "zero": {"v2": ["zero"], "obs": ["zero"], "obs2": ["zero"]},
    "naive": {"obs": ["naive"], "obs2": ["naive"]},
    "ipw": {"obs": ["ipw"], "obs2": ["ipw"]},
    "match5": {"obs": ["match5"], "obs2": ["match5"]},
    "an5_arm": {"v2": ["an5_arm"]},
    "arm_mean": {"v2": ["arm_mean"]},
    "floor": {"v2": ["floor"], "obs2": ["floor"]},
    "ridge_obs": {"v2": ["ridge_obs"], "obs2": ["ridge_obs"]},
    "ridge_priv": {"v2": ["ridge_priv"], "obs2": ["ridge_priv"]},
    "m1b_v2": {"v2": [f"wm2_m1bact_v19_s{s}" for s in SEEDS3], "obs": [f"ref_wm2_m1bact_v19_s{s}_obs" for s in SEEDS3],
               "obs2": [f"ref_wm2_m1bact_v19_s{s}" for s in SEEDS3]},
    "wm3_full": {"v2": [f"wm3_full_s{s}" for s in SEEDS3], "obs": [f"ref_wm3_full_s{s}_obs" for s in SEEDS3],
                 "obs2": [f"ref_wm3_full_s{s}" for s in SEEDS3]},
    "m1b_obs": {"obs": [f"m1b_obs_s{s}" for s in SEEDS2], "obs2": [f"ref_m1b_obs_s{s}" for s in SEEDS2]},
    "m1b_obs2": {"obs2": [f"m1b_obs2_s{s}" for s in SEEDS2]},
    "crn_obs_wm4": {"obs": [f"crn_obs_s{s}" for s in SEEDS2], "obs2": [f"ref_crn_obs_s{s}" for s in SEEDS2]},
    "ct_obs_wm4": {"obs": [f"ct_obs_s{s}" for s in SEEDS2], "obs2": [f"ref_ct_obs_s{s}" for s in SEEDS2]},
    "crn_v2_wm4": {"v2": ["wm4:crn_v2_s0", "wm4:crn_v2_s1"]},
    "ct_v2_wm4": {"v2": ["wm4:ct_v2_s0", "wm4:ct_v2_s1"]},
    "crn_obs2": {"obs2": [f"crn_obs2_s{s}" for s in SEEDS2]},
    "ct_obs2": {"obs2": [f"ct_obs2_s{s}" for s in SEEDS2]},
    "crn_v2": {"v2": [f"crn_v2_s{s}" for s in SEEDS2]},
    "ct_v2": {"v2": [f"ct_v2_s{s}" for s in SEEDS2]},
}


def main_table(j: dict) -> dict:
    """Seed-averaged 12-cell scores per method group and split (the paper's main table): v2 and obs2 from this judge,
    obs (and WM-4's v2 CRN / CT) from WM-4's committed judge. The mean over seeds equals the seed-averaged model's score
    (the cell MAE is linear in the per-case errors)."""
    j4 = load(WM4 / "judge4" / "judge.json") or {}
    src = {"v2": j["v2"].get("cell_averages", {}), "obs2": j["obs2"].get("cell_averages", {}),
           "obs": (j4.get("obs") or {}).get("cell_averages", {})}
    v2_4 = (j4.get("v2") or {}).get("cell_averages", {})
    out = {}
    for g, splits in GROUPS.items():
        for split, names in splits.items():
            vals = []
            for n in names:
                ca = v2_4.get(n[4:]) if n.startswith("wm4:") else src[split].get(n)
                if ca is not None:
                    vals.append(ca)
            if vals:
                out.setdefault(g, {})[split] = {lag: float(np.mean([v[lag] for v in vals])) for lag in ("7d", "30d")} | {
                    "seeds": len(vals)}
    return out


def build(runs: Path) -> dict:
    info = runs_block(runs)
    return {"obs2": obs2_block(runs), "v2": v2_block(runs), "D2": d2_block(runs), "runs": info, "R4": r4_block(info),
            "factual_obs2": factual_block(runs), "describe": load(runs / "describe" / "describe.json")}


def cost_block(runs: Path) -> dict:
    """Every cost check (GOAL.md 9.1), including the two CT-obs2 measurements taken beside our own evaluations
    (`_aborted/`, DECISIONS L4.4)."""
    out = {}
    paths = sorted(runs.glob("_cost_*/cost.json")) + sorted(runs.glob("_aborted/_cost_*/cost.json"))
    for p in paths:
        cst = load(p)
        name = p.parent.name if p.parent.parent == runs else "aborted:" + p.parent.name
        out[name] = {"stages": cst["stages"], "plan": cst["plan"], "peak_gpu_mib": cst.get("peak_gpu_mib")}
    return out


def compute_block(runs: Path, info: dict) -> dict:
    """Wall times (s) of the recording stages (`runs/record/*.start|end`) and of every training (`val_metrics.json`)."""
    rec = runs / "record"
    out = {"recording": {}, "training": {}}
    for name in ["r0"] + [f"round{k}_record" for k in range(1, 7)] + ["fam_val_record", "fam_test_record"]:
        if (rec / f"{name}.start").exists() and (rec / f"{name}.end").exists():
            out["recording"][name] = int((rec / f"{name}.end").read_text()) - int((rec / f"{name}.start").read_text())
    out["recording_total"] = sum(out["recording"].values())
    for n, r in info.items():
        if r.get("seconds") is not None and not n.startswith("_"):
            out["training"][n] = r["seconds"]
    out["training_total_hours"] = sum(out["training"].values()) / 3600
    return out


def posthoc_block(j: dict, runs: Path) -> dict:
    """Post hoc (review round 1, M1 / M2; not pre-registered): the direct regressions against the arm-level baselines on
    v2, and the support shift between v2's training cases and the obs2 test cases (cast rows per arm)."""
    out = {"note": "post hoc (REVIEW.md round 1, M1 / M2); not a pre-registered rule"}
    base = load(WM2 / "effects_base" / "effects_base.json")
    stds = ef.stds_from(base["stds"])
    cases = [ef.case_from(x) for x in base["cases"]["test"]]
    worlds = sorted({x.seed for x in cases})
    cells = {lag: [(a, o, lag) for a in v2.W3_ARMS for o in v2.OUTCOMES if stds[(a, o, lag)] is not None]
             for lag in v2.LAGS}
    preds = {k: base["test_predictions"][k] for k in ("arm_mean", "an5_arm", "zero")}
    for name in ("ridge_obs", "ridge_priv"):
        e = load(runs / name / f"{name}.json")
        if e is not None:
            preds[name] = e["v2_test_predictions"]
    errs = {n: ef.cell_errors(cases, p, stds) for n, p in preds.items()}
    out["v2_vs_arm_level"] = {}
    for r in ("ridge_obs", "ridge_priv"):
        for b in ("arm_mean", "an5_arm"):
            if r in errs:
                for lag in v2.LAGS:
                    out["v2_vs_arm_level"][f"{r}|{b}|{lag}"] = ef.compare([errs[r]], errs[b], cells[lag], worlds) | {
                        "baseline_member": b}
    train = [ef.case_from(x) for x in base["cases"]["train"]]
    o2 = load(runs / "obs2_base" / "obs2_base.json")
    if o2 is not None:
        test = [x for x in o2["cases"]["test"] if x["rep"] in (0, 2)]
        out["support"] = {}
        for a_obs, a_v2 in (("gf240", "gf240E"), ("sp240", "sp240L"), ("rw120", "rw120E")):
            rows_tr = {x.cast_row for x in train if x.arm == a_v2}
            rows_te = [x["cast_row"] for x in test if x["arm"] == a_v2]
            out["support"][a_obs] = {"v2_train_cast_rows": sorted(rows_tr), "obs2_test_distinct_rows": len(set(rows_te)),
                                     "obs2_test_cases": len(rows_te),
                                     "share_obs2_cases_at_a_v2_train_row": sum(r in rows_tr for r in rows_te) / max(1, len(rows_te))}
    return out


def build_all(runs: Path) -> dict:
    j = build(runs)
    j["main_table"] = main_table(j)
    j["cost_checks"] = cost_block(runs)
    j["compute"] = compute_block(runs, j["runs"])
    j["posthoc"] = posthoc_block(j, runs)
    return j


# ------------------------------------------------------------------ markdown ---------------------------------------
def v_row(name: str, w: dict, dd: int = 3) -> str:
    seeds = ", ".join(fmt(s, dd) for s in w["seeds"])
    return (f"| {name} | {fmt(w['model'], dd)} ({seeds}) | `{w['baseline_member']}` {fmt(w['baseline'], dd)} | "
            f"{fmt(w.get('ratio'), 3)} | {ci_text(w['ci_diff'], dd)} | **{w['verdict']}** |")


def markdown(j: dict) -> str:
    o, b2, r4 = j["obs2"], j["v2"], j["R4"]
    under = lambda kind: [n for n, r in r4.items() if n.startswith(kind + "_s") and r["verdict"] != "trained as the authors"]
    tag = lambda kind: (f" (under-trained: {', '.join(under(kind))})" if under(kind) else "")
    L = ["# WM-5 judge (generated by `tools/worldmodel/wm5/judge5.py`)", "",
         "| rule | model (seeds) | baseline | ratio | Δ = model − baseline [95 % CI] | verdict |",
         "|---|---|---|---|---|---|"]
    for rule in ("R1", "R2"):
        for kind, w in o.get("rules", {}).get(rule, {}).items():
            L.append(v_row(f"{rule} obs2 {'7 d' if rule == 'R1' else '30 d'}: `{kind}`{tag(kind)}", w))
    for key, w in b2["rules"].get("R3", {}).items():
        kind, b, lag = key.split("|")
        L.append(v_row(f"R3 v2 {lag}: `{kind}` vs `{b}`{tag(kind)}", w))
    for split, blk in (("v2", b2), ("obs2", o)):
        for lag, w in blk.get("rules", {}).get("E1", {}).items():
            L.append(v_row(f"E1 {split} {lag}: `ridge_obs` vs `zero`", w))
    for split, blk in (("v2", b2), ("obs2", o)):
        for key, w in blk.get("rules", {}).get("E2", {}).items():
            kind, lag = key.split("|")
            L.append(v_row(f"E2 {split} {lag}: `{kind}` vs `ridge_obs`", w))
    for key, w in o.get("rules", {}).get("D1", {}).items():
        L.append(v_row(f"D1 obs2 {key.split('|')[1]}: {key.split('|')[0]}", w))
    L += ["", "## R4 — training reached", "", "| run | planned | steps | steps reached (min stage) | α / α_max (min stage) | "
          "planned = authors' | stop | verdict |", "|---|---|---|---|---|---|---|---|"]
    for n, r in r4.items():
        L.append(f"| `{n}` | {r['planned']} | {r['steps']} | {fmt(r['steps_fraction_min'])} | {fmt(r['alpha_fraction_min'], 4)} | "
                 f"{r['planned_equal_authors']} | {r['stop']} | {r['verdict']} |")
    for split, blk in (("v2 (F1)", b2), ("obs2 (F2)", o)):
        if not blk.get("ladder"):
            continue
        L += ["", f"## Headroom ladder, {split}: 12-cell averages and steps (Δ with world-bootstrap 95 % CI)", "",
              "| lag | zero | ridge_obs | ridge_priv (diagnostic) | floor | " + " | ".join(a + " − " + b for a, b in LADDER) + " |",
              "|---|---|---|---|---|" + "---|" * len(LADDER)]
        for lag, node in blk["ladder"].items():
            v = node["values"]
            L.append(f"| {lag} | {fmt(v.get('zero'))} | {fmt(v.get('ridge_obs'))} | {fmt(v.get('ridge_priv'))} | "
                     f"{fmt(v.get('floor'))} | " + " | ".join(ci_text(node['steps'][f'{a} − {b}']['ci_diff'])
                                                         if f'{a} − {b}' in node['steps'] else '—' for a, b in LADDER) + " |")
    if b2.get("spread_wm2"):
        L.append(f"\nWM-2's between-replicate spread (|e0 − e1| / √2 / std, 12-cell average): 7 d {fmt(b2['spread_wm2']['7d'])}, "
                 f"30 d {fmt(b2['spread_wm2']['30d'])}; the floor's MAE is √2 times it.")
    if o:
        L += ["", f"obs2 baselines chosen on the val families (12-cell average, floored scales): 7 d `{o['selection']['7d']['chosen']}`, "
              f"30 d `{o['selection']['30d']['chosen']}`; val values " + "; ".join(
                  f"{lag}: " + ", ".join(f"{n} {fmt(v)}" for n, v in o["selection"][lag]["val"].items()) for lag in ("7d", "30d")),
              "", "## obs2 confounding (test, 12-cell averages; Δ with world-bootstrap 95 % CI)", "",
              "| lag | zero | naive | ipw | match5 | naive − zero | ipw − zero | ipw − naive |", "|---|---|---|---|---|---|---|---|"]
        for lag, v in o["confounding"].items():
            cc = v["cells"]
            L.append(f"| {lag} | {fmt(cc['zero'])} | {fmt(cc['naive'])} | {fmt(cc['ipw'])} | {fmt(cc['match5'])} | "
                     f"{ci_text(v['naive_vs_zero']['ci_diff'])} | {ci_text(v['ipw_vs_zero']['ci_diff'])} | {ci_text(v['ipw_vs_naive']['ci_diff'])} |")
        L += ["", "Pooled estimates on the observed train worlds (food store; constants per arm and lag), counts and effective sample sizes:", "",
              "| arm | lag | naive | ipw | treated | treated with an earlier cast | ESS treated | controls | ESS controls | max weight treated / controls |",
              "|---|---|---|---|---|---|---|---|---|---|"]
        est = o["estimates"]
        for a in e4.OBS_ARMS:
            for lag in e4.LAGS:
                cn = est["counts"][a][lag]
                L.append(f"| {a} | {lag} | {fmt(est['naive'][a][lag]['food_stock'], 1)} | {fmt(est['ipw'][a][lag]['food_stock'], 1)} | "
                         f"{cn['treated']} | {cn.get('treated_with_earlier_cast')} | {fmt(cn['ess_treated'], 1)} | {cn['controls']} | "
                         f"{fmt(cn['ess_controls'], 1)} | {fmt(cn['max_weight_treated'], 1)} / {fmt(cn['max_weight_controls'], 2)} |")
        L += ["", "## obs2 test: 12-cell averages (floored scales; unfloored in brackets; c1 / c2 cases separately)", "",
              "| predictor | 7 d | 30 d | 7 d c1 / c2 | 30 d c1 / c2 |", "|---|---|---|---|---|"]
        for n, v in o["cell_averages"].items():
            u = o["cell_averages_unfloored"][n]
            br = o["cell_averages_by_ref"]
            L.append(f"| `{n}` | {fmt(v['7d'])} ({fmt(u['7d'])}) | {fmt(v['30d'])} ({fmt(u['30d'])}) | "
                     f"{fmt(br['c1'][n]['7d'])} / {fmt(br['c2'][n]['7d'])} | {fmt(br['c1'][n]['30d'])} / {fmt(br['c2'][n]['30d'])} |")
        if o.get("secondary"):
            L += ["", "Secondary checkpoints and readouts (obs2, 12-cell averages):", "", "| run / checkpoint / readout | 7 d | 30 d |",
                  "|---|---|---|"]
            for n, v in o["secondary"].items():
                L.append(f"| `{n}` | {fmt(v['7d'])} | {fmt(v['30d'])} |")
    L += ["", "## v2 test: 12-cell averages (WM-2 scales); deaths cells / other nine cells (post hoc, WM-4 L5.1)", "",
          "| predictor | 7 d | 30 d | 7 d deaths / other | 30 d deaths / other |", "|---|---|---|---|---|"]
    for n, v in b2["cell_averages"].items():
        ds = b2["deaths_split"][n]
        L.append(f"| `{n}` | {fmt(v['7d'])} | {fmt(v['30d'])} | {fmt(ds['7d']['deaths'])} / {fmt(ds['7d']['other'])} | "
                 f"{fmt(ds['30d']['deaths'])} / {fmt(ds['30d']['other'])} |")
    if b2.get("secondary"):
        L += ["", "Secondary checkpoints and readouts (v2, 12-cell averages):", "", "| run / checkpoint / readout | 7 d | 30 d |",
              "|---|---|---|"]
        for n, v in b2["secondary"].items():
            L.append(f"| `{n}` | {fmt(v['7d'])} | {fmt(v['30d'])} |")
    if j["D2"]:
        L += ["", "## D2 — rounding alone (WM-4's runs on their own test splits, 12-cell averages; deaths cells in brackets)", "",
              "| run | lag | WM-4 committed | re-scored rounded | re-scored unrounded |", "|---|---|---|---|---|"]
        for key, node in j["D2"].items():
            for rl, v in node["values"].items():
                r, lag = rl.split("|")
                L.append(f"| `{r}` | {lag} | {fmt(v['wm4'])} ({fmt(v['deaths']['wm4'])}) | {fmt(v['rounded'])} ({fmt(v['deaths']['rounded'])}) | "
                         f"{fmt(v['unrounded'])} ({fmt(v['deaths']['unrounded'])}) |")
        L += ["", "| pair (seed-averaged) | lag | rounded | WM-4 | Δ [95 % CI] |", "|---|---|---|---|---|"]
        for key, node in j["D2"].items():
            for lag, w in node["compare"].items():
                L.append(f"| `{key}` rounded vs committed | {lag} | {fmt(w['model'])} | {fmt(w['baseline'])} | {ci_text(w['ci_diff'])} |")
    if o:
        preds = list(o["tables"])
        L += ["", "## obs2 test: per cell (scaled effect MAE / sign accuracy; true mean (std))", "",
              "| arm | outcome | lag | truth | " + " | ".join(f"`{p}`" for p in preds) + " |", "|---|---|---|---|" + "---|" * len(preds)]
        for a in e4.OBS_ARMS:
            for out_ in v2.OUTCOMES:
                for lag in e4.LAGS:
                    key = f"{a}|{out_}|{lag}"
                    dist = o["distribution_test"].get(a, {}).get(f"{out_}|{lag}", {})
                    L.append(f"| {a} | {out_} | {lag} | {fmt(dist.get('mean'), 2)} ({fmt(dist.get('std'), 2)}) | " + " | ".join(
                        (f"{fmt(o['tables'][p][key]['mae'])} / {fmt(o['tables'][p][key]['sign_accuracy'], 2)}" if key in o["tables"][p] else "—")
                        for p in preds) + " |")
    preds = list(b2["tables"])
    L += ["", "## v2 test: per cell (scaled effect MAE / sign accuracy)", "",
          "| arm | outcome | lag | " + " | ".join(f"`{p}`" for p in preds) + " |", "|---|---|---|" + "---|" * len(preds)]
    for a in v2.W3_ARMS:
        for out_ in v2.OUTCOMES:
            for lag in v2.LAGS:
                key = f"{a}|{out_}|{lag}"
                L.append(f"| {a} | {out_} | {lag} | " + " | ".join(
                    (f"{fmt(b2['tables'][p][key]['mae'])} / {fmt(b2['tables'][p][key]['sign_accuracy'], 2)}" if key in b2["tables"][p] else "—")
                    for p in preds) + " |")
    if j.get("factual_obs2"):
        L += ["", "M1b-obs2 factual T2 (raw, every hour, obs2 test): " + "; ".join(
            f"`{r}` 24 h {fmt(v['24'], 4)}, 168 h {fmt(v['168'], 4)}" for r, v in j["factual_obs2"].items())]
    L += ["", "## Runs", "", "| run | detail |", "|---|---|"]
    for n, r in j["runs"].items():
        L.append(f"| `{n}` | {json.dumps({k: v for k, v in r.items() if k not in ('stages',)})[:600]} |")
    return "\n".join(L) + "\n"


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--runs", default=str(RUNS))
    ap.add_argument("--out", default=str(RUNS / "judge5"))
    a = ap.parse_args(argv)
    j = build_all(Path(a.runs))
    c.write_json(Path(a.out) / "judge.json", j)
    (Path(a.out) / "judge.md").write_text(markdown(j))
    print(markdown(j)[:6000])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
