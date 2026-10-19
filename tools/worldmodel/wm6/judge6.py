"""The WM-6 verdicts (docs/worldmodel/wm6/METRICS.md, GOAL.md 8, 10) from the run files copied to
`docs/worldmodel/wm6/runs/`, beside WM-2 … WM-5's committed files: C1–C6, the learning curves (error and closure
against N per model and lag), the C4 slopes, the pre-registered reading, the WM-5 reproduction, the obs2 transfer, the
per-cell tables, every run's stop reason and time, and the pre-registration order. numpy only; writes `judge.json`
and `judge.md`.

Scoring is WM-5's v2 code unchanged: `wm2.effects.cell_errors` with WM-2's scales on WM-2's 604 test cases (both
replicates), the 12 W3 cells per lag, `cell_stat`, `compare` (world-level paired bootstrap B = 2,000,
`default_rng(0)`, seed-averaged model side) and the W1 rule. Closure = (`zero` − model) / (`zero` − floor), its CI from
the same bootstrap draws; a slope = OLS of the seed-averaged 12-cell average on log2 N, its CI from the same draws.

usage: python3 tools/worldmodel/wm6/judge6.py [--runs DIR] [--out DIR]
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm6  # noqa: F401
from wm1 import bootstrap as bs
from wm1 import common as c
from wm2 import effects as ef
from wm2 import v2
from wm4 import effects4 as e4

RUNS = c.REPO / "docs/worldmodel/wm6/runs"
WM2 = c.REPO / "docs/worldmodel/wm2/runs"
WM3 = c.REPO / "docs/worldmodel/wm3/runs"
WM5 = c.REPO / "docs/worldmodel/wm5/runs"
NS = (400, 800, 1600, 2000)
SEEDS = (0, 1, 2)
NEURAL = ("mlp_obs", "mlp_priv", "seq_obs", "seq_priv")
SINGLE = ("ridge_obs", "ridge_priv", "gbm_obs", "gbm_priv")
MODELS = ("ridge_obs", "mlp_obs", "seq_obs", "gbm_obs", "ridge_priv", "mlp_priv", "seq_priv", "gbm_priv")
LAGS = tuple(v2.LAGS)
LEVEL = bs.LEVEL


def load(p: Path) -> dict | None:
    return json.loads(p.read_text()) if p.exists() else None


def fmt(x, dd: int = 3) -> str:
    return "—" if x is None or (isinstance(x, float) and not np.isfinite(x)) else f"{x:.{dd}f}"


def ci_text(ci: dict, dd: int = 3) -> str:
    return f"{ci['point']:+.{dd}f} [{ci['lo']:+.{dd}f}, {ci['hi']:+.{dd}f}]"


def pct(draws: np.ndarray) -> tuple[float, float]:
    lo, hi = np.nanpercentile(draws, [(100 - LEVEL) / 2, 100 - (100 - LEVEL) / 2])
    return float(lo), float(hi)


def run_names(model: str, n: int) -> list[str]:
    if model in NEURAL:
        return [f"{model}_n{n}_s{s}" for s in SEEDS]
    return [f"{model}_n{n}"]


class Scorer:
    """WM-2's v2 test cases and scales, the reference predictors, every run's errors."""

    def __init__(self, runs: Path):
        base = load(WM2 / "effects_base" / "effects_base.json")
        self.stds = ef.stds_from(base["stds"])
        self.cases = [ef.case_from(j) for j in base["cases"]["test"]]
        self.val = [x for x in (ef.case_from(j) for j in base["cases"]["val"]) if x.arm in v2.W3_ARMS]
        self.worlds = sorted({x.seed for x in self.cases})
        self.val_worlds = sorted({x.seed for x in self.val})
        self.cells = {lag: [(a, o, lag) for a in v2.W3_ARMS for o in v2.OUTCOMES if self.stds[(a, o, lag)] is not None]
                      for lag in LAGS}
        self.counts = ef.draw_counts(len(self.worlds))
        preds = {k: base["test_predictions"][k] for k in ("zero", "an5_arm", "arm_mean")}
        by = {(x.seed, x.arm, x.rep): x for x in self.cases}
        preds["floor"] = {}
        for x in self.cases:                  # v2 floor: the other replicate of the family (as judge5)
            o = by[(x.seed, x.arm, 1 - x.rep)]
            preds["floor"][x.key] = {"effects": {og: {lag: o.truth[og][lag] for lag in x.truth[og] if lag in o.truth[og]}
                                                 for og in v2.OUTCOMES}, "shift": o.shift}
        for s in SEEDS:
            e = load(WM3 / f"wm3_full_s{s}_effects" / "effects.json")
            if e is not None:
                preds[f"wm3_full_s{s}"] = e["predictions"]
        self.ref = {n: ef.cell_errors(self.cases, p, self.stds) for n, p in preds.items()}
        self.ref["wm3_full"] = ef.average_errors([self.ref[f"wm3_full_s{s}"] for s in SEEDS if f"wm3_full_s{s}" in self.ref])
        self.runs = runs
        self.errs: dict = {}
        self.val_errs: dict = {}
        for d_ in sorted(runs.iterdir()) if runs.exists() else []:
            e = load(d_ / "effects.json")
            if e is not None:
                self.errs[d_.name] = ef.cell_errors(self.cases, e["predictions"], self.stds)
            v = load(d_ / "val_predictions.json")
            if v is not None:
                keep = {x.key for x in self.val}
                vv = {k: p for k, p in v.items() if k in keep}
                if len(vv) == len(keep):
                    self.val_errs[d_.name] = ef.cell_errors(self.val, vv, self.stds)

    def stat(self, errs: dict, lag: str, counts=None):
        return ef.cell_stat(errs, self.cells[lag], self.worlds, counts)

    def model_errs(self, model: str, n: int, val: bool = False) -> tuple[dict | None, list[str]]:
        src = self.val_errs if val else self.errs
        names = [r for r in run_names(model, n) if r in src]
        if not names:
            return None, []
        return (ef.average_errors([src[r] for r in names]) if len(names) > 1 else src[names[0]]), names


# ------------------------------------------------------------------ blocks ------------------------------------------
def curves(sc: Scorer) -> dict:
    """Per model, N and lag: the seed-averaged 12-cell average, each seed's, the closure with its CI."""
    out = {}
    z = {lag: sc.stat(sc.ref["zero"], lag) for lag in LAGS}
    f = {lag: sc.stat(sc.ref["floor"], lag) for lag in LAGS}
    zd = {lag: sc.stat(sc.ref["zero"], lag, sc.counts) for lag in LAGS}
    fd = {lag: sc.stat(sc.ref["floor"], lag, sc.counts) for lag in LAGS}
    for m in MODELS:
        for n in NS:
            errs, names = sc.model_errs(m, n)
            if errs is None:
                continue
            node = {"runs": names, "seeds": len(names), "diagnostic": m.endswith("_priv")}
            for lag in LAGS:
                val = sc.stat(errs, lag)
                md = sc.stat(errs, lag, sc.counts)
                with np.errstate(invalid="ignore", divide="ignore"):
                    cd = (zd[lag] - md) / (zd[lag] - fd[lag])
                lo, hi = pct(cd)
                node[lag] = {"value": val, "seed_values": [sc.stat(sc.errs[r], lag) for r in names],
                             "closure": {"point": (z[lag] - val) / (z[lag] - f[lag]), "lo": lo, "hi": hi}}
            out.setdefault(m, {})[str(n)] = node
    return out


def references(sc: Scorer) -> dict:
    z = {lag: sc.stat(sc.ref["zero"], lag) for lag in LAGS}
    f = {lag: sc.stat(sc.ref["floor"], lag) for lag in LAGS}
    out = {}
    for n in ("zero", "floor", "an5_arm", "arm_mean", "wm3_full"):
        if n in sc.ref and sc.ref[n]:
            out[n] = {lag: {"value": sc.stat(sc.ref[n], lag), "closure": (z[lag] - sc.stat(sc.ref[n], lag)) / (z[lag] - f[lag])}
                      for lag in LAGS}
    return out


def largest_n(sc: Scorer) -> int | None:
    """METRICS.md 2: the largest N at which mlp_obs, mlp_priv and seq_obs each have >= 2 seeds evaluated."""
    ok = [n for n in NS if all(len(sc.model_errs(m, n)[1]) >= 2 for m in ("mlp_obs", "mlp_priv", "seq_obs"))]
    return max(ok) if ok else None


def verdict(sc: Scorer, model: str, n: int, base_errs: dict, base_name: str, lag: str) -> dict | None:
    names = [r for r in run_names(model, n) if r in sc.errs]
    if not names or base_errs is None:
        return None
    return ef.compare([sc.errs[r] for r in names], base_errs, sc.cells[lag], sc.worlds) | {
        "model_name": model, "n": n, "runs": names, "baseline_member": base_name, "lag": lag}


def slope(sc: Scorer, model: str, lag: str) -> dict | None:
    pts = [(n, sc.model_errs(model, n)[0]) for n in NS]
    pts = [(n, e) for n, e in pts if e is not None]
    if len(pts) < 2:
        return None
    x = np.log2([n for n, _ in pts])
    xc = x - x.mean()
    vals = np.array([sc.stat(e, lag) for _, e in pts])
    draws = np.stack([sc.stat(e, lag, sc.counts) for _, e in pts], axis=1)        # [B, k]
    point = float(xc @ vals / (xc @ xc))
    sd = (draws - draws.mean(1, keepdims=True)) @ xc / (xc @ xc)
    lo, hi = pct(sd)
    return {"model": model, "lag": lag, "ns": [n for n, _ in pts], "values": vals.tolist(), "point": point, "lo": lo,
            "hi": hi, "excludes_0": bool(hi < 0 or lo > 0), "unit": "12-cell average per doubling of N"}


def rules(sc: Scorer, cur: dict) -> dict:
    nmax = largest_n(sc)
    out = {"largest_n": nmax}
    if nmax is None:
        return out
    # C1 / C2: the observable model chosen on val at the rule's lag
    for rule, lag in (("C1", "7d"), ("C2", "30d")):
        choice = {}
        for m in ("mlp_obs", "seq_obs"):
            ve, names = sc.model_errs(m, nmax, val=True)
            if ve is not None:
                choice[m] = {"val": ef.cell_stat(ve, sc.cells[lag], sc.val_worlds), "runs": names}
        if not choice:
            continue
        chosen = min(choice, key=lambda m: (choice[m]["val"], ("mlp_obs", "seq_obs").index(m)))
        ridge = sc.errs.get(f"ridge_obs_n{nmax}")
        out[rule] = {"chosen": chosen, "val_choice": choice,
                     "verdict": verdict(sc, chosen, nmax, ridge, f"ridge_obs_n{nmax}", lag)}
    out["C3"] = {lag: verdict(sc, "mlp_priv", nmax, sc.errs.get(f"ridge_priv_n{nmax}"), f"ridge_priv_n{nmax}", lag)
                 for lag in LAGS}
    out["C4"] = {}
    for m in ("mlp_obs", "seq_obs", "mlp_priv"):
        b, bn = sc.model_errs(m, 400)
        node = {"verdict_7d": verdict(sc, m, nmax, b, f"{m}_n400 (seeds {len(bn)})", "7d") if nmax != 400 else None,
                "slope": {lag: slope(sc, m, lag) for lag in LAGS}}
        out["C4"][m] = node
    out["C5"] = "see curves (every model's closure at every N and lag, with its CI)"
    out["C6"] = {}
    for a, b in (("mlp_priv", "mlp_obs"), ("seq_priv", "seq_obs")):
        ea, eb = sc.model_errs(a, nmax)[0], sc.model_errs(b, nmax)[0]
        if ea is not None and eb is not None:
            for lag in LAGS:
                r = ef.compare([ea], eb, sc.cells[lag], sc.worlds)
                out["C6"][f"{a} − {b}|{lag}"] = {"a": r["model"], "b": r["baseline"], "ci_diff": r["ci_diff"]}
    out["reading"] = reading(out, cur, nmax)
    return out


def reading(r: dict, cur: dict, nmax: int) -> dict:
    """METRICS.md 4, in order; closures are 7-day point estimates at the largest N."""
    clo = lambda m: (cur.get(m, {}).get(str(nmax), {}).get("7d") or {}).get("closure", {}).get("point")
    c1 = r.get("C1", {}).get("chosen")
    c1_clo = clo(c1) if c1 else None
    c1_slope = (r["C4"].get(c1, {}).get("slope", {}).get("7d") if c1 else None) or {}
    priv_clo = clo("mlp_priv")
    every = {m: clo(m) for m in ("mlp_obs", "seq_obs", "mlp_priv", "seq_priv", "gbm_obs", "gbm_priv") if clo(m) is not None}
    slopes = {m: (r["C4"][m]["slope"]["7d"] or {}) for m in ("mlp_obs", "seq_obs", "mlp_priv")}
    any_slope_excl = any(s.get("excludes_0") for s in slopes.values())
    checks = {
        "learnable_from_observables": bool(c1_clo is not None and (c1_clo >= 0.5 or (c1_slope.get("hi", 1) < 0 and c1_clo >= 0.35))),
        "learnable_only_with_hidden": bool(priv_clo is not None and c1_clo is not None and priv_clo >= 0.5 and c1_clo < 0.35),
        "not_learnable_at_this_scale": bool(every and all(v < 0.35 for v in every.values()) and not any_slope_excl),
    }
    label = next((k for k in ("learnable_from_observables", "learnable_only_with_hidden", "not_learnable_at_this_scale")
                  if checks[k]), "partially_learnable")
    return {"label": label, "checks": checks, "c1_model": c1, "c1_closure_7d": c1_clo,
            "c1_slope_7d": {k: c1_slope.get(k) for k in ("point", "lo", "hi")}, "mlp_priv_closure_7d": priv_clo,
            "closures_7d": every, "slopes_7d": {m: {k: s.get(k) for k in ("point", "lo", "hi", "excludes_0")}
                                               for m, s in slopes.items()}, "largest_n": nmax}


def reproduction(sc: Scorer) -> dict:
    """GOAL.md 4.1: WM-5's ridge re-run on both replicates at N = 400 equals WM-5's committed predictions."""
    out = {}
    for f in ("obs", "priv"):
        name = f"ridge_{f}"
        mine, wm5 = load(sc.runs / f"{name}_repro" / "effects.json"), load(WM5 / name / f"{name}.json")
        vm = load(sc.runs / f"{name}_repro" / "val_metrics.json")
        if mine is None or wm5 is None:
            continue
        a, b = mine["predictions"], wm5["v2_test_predictions"]
        diffs = [abs(a[k]["effects"][o][lag] - b[k]["effects"][o][lag]) for k in b for o in b[k]["effects"] for lag in b[k]["effects"][o]]
        ea, eb = ef.cell_errors(sc.cases, a, sc.stds), ef.cell_errors(sc.cases, b, sc.stds)
        out[name] = {"keys_equal": sorted(a) == sorted(b), "test_max_abs_diff": max(diffs), "n_values": len(diffs),
                     "cell_averages": {lag: {"wm6": sc.stat(ea, lag), "wm5": sc.stat(eb, lag)} for lag in LAGS},
                     "val_check": (vm or {}).get("wm5_check")}
        out[name]["reproduced"] = bool(out[name]["keys_equal"] and out[name]["test_max_abs_diff"] < 1e-6
                                       and (vm or {}).get("wm5_check", {}).get("lambdas_equal"))
    return out


def obs2_block(sc: Scorer, nmax: int | None) -> dict:
    """GOAL.md 9.3 (report only): `mlp_obs` / `seq_obs` at the largest N on the obs2 test r0 cases with WM-4's floored
    scales, beside WM-5's E1 row (`ridge_obs`, v2→obs2 transfer) and `zero`."""
    path = WM5 / "obs2_base" / "obs2_base.json"
    if nmax is None or not path.exists():
        return {}
    from wm5 import effects5 as e5
    _, test_all, fl, _ = e5.load_obs2_base(path)
    test = [x for x in test_all if e5.ref_of(x.rep)[1] == 0]
    keep = {(x.seed, x.rep) for x in test}
    worlds = sorted({x.seed for x in test})
    base = load(path)
    refs = {"zero": base["test_predictions"]["zero"], "floor": base["floor_predictions"]}
    r5 = load(WM5 / "ridge_obs" / "ridge_obs.json")
    if r5:
        refs["ridge_obs (WM-5)"] = r5["obs2_test_predictions"]
    restrict = lambda e: {cell: [r for r in rows if (r[0], r[1]) in keep] for cell, rows in e.items()}
    errs = {n: restrict(ef.cell_errors(test, p, fl)) for n, p in refs.items()}
    out = {"n": nmax, "worlds": len(worlds), "cases": len(test), "models": {}}
    for m in ("mlp_obs", "seq_obs"):
        names = [r for r in run_names(m, nmax) if (sc.runs / r / "effects_obs2.json").exists()]
        if not names:
            continue
        per = [restrict(ef.cell_errors(test, load(sc.runs / r / "effects_obs2.json")["predictions"], fl)) for r in names]
        errs[m] = ef.average_errors(per) if len(per) > 1 else per[0]
        out["models"][m] = {"runs": names}
        for lag in LAGS:
            out["models"][m][lag] = {
                "vs_zero": ef.compare(per, errs["zero"], e4.CELLS[lag], worlds) | {"baseline_member": "zero"},
                "vs_wm5_ridge_obs": (ef.compare(per, errs["ridge_obs (WM-5)"], e4.CELLS[lag], worlds)
                                     if "ridge_obs (WM-5)" in errs else None)}
    out["cell_averages"] = {n: {lag: ef.cell_stat(e, e4.CELLS[lag], worlds) for lag in LAGS} for n, e in errs.items()}
    return out


def per_cell(sc: Scorer, nmax: int | None) -> dict:
    """Scaled MAE per cell (seed-averaged) of every model at the largest N and the references."""
    out = {}
    srcs = {n: sc.ref[n] for n in ("zero", "floor", "an5_arm", "wm3_full")}
    if nmax is not None:
        for m in MODELS:
            e = sc.model_errs(m, nmax)[0]
            if e is not None:
                srcs[f"{m}_n{nmax}"] = e
    for n, e in srcs.items():
        out[n] = {"|".join(cell): float(np.mean([r[2] for r in e[cell]])) for lag in LAGS for cell in sc.cells[lag]}
    return out


def runs_info(runs: Path, prereg_time: int | None) -> dict:
    out = {}
    for d_ in sorted(runs.iterdir()) if runs.exists() else []:
        cfg, val = load(d_ / "config.json"), load(d_ / "val_metrics.json")
        if cfg is None:
            continue
        node = {k: cfg.get(k) for k in ("model", "feat", "n", "seed", "width", "dropout", "n_train_cases",
                                         "n_train_worlds", "train_reps", "device", "code_commit", "diagnostic")}
        if val:
            node |= {k: val.get(k) for k in ("stop", "best_epoch", "epochs", "seconds")}
            node["val_both"] = (val.get("val") or {}).get("both")
        for f in ("train.start", "train.end", "eval.start", "eval.end", "eval_obs2.start", "eval_obs2.end"):
            if (d_ / f).exists():
                node[f] = int((d_ / f).read_text().split()[0])
        if prereg_time is not None:
            ev = [node[f] for f in ("eval.start", "eval_obs2.start") if f in node]
            node["eval_after_prereg"] = all(t > prereg_time for t in ev) if ev else None
        out[d_.name] = node
    return out


def compute_block(runs: Path, info: dict) -> dict:
    """Wall times (s) of the v2x recording (`runs/record/`) and the trainings / evaluations (run files)."""
    rec = runs / "record"
    t = lambda n: (int((rec / f"{n}.end").read_text()) - int((rec / f"{n}.start").read_text())
                   if (rec / f"{n}.start").exists() and (rec / f"{n}.end").exists() else None)
    probes = {p.stem: t(p.stem) for p in sorted(rec.glob("probe_*.start"))}
    out = {"probe_chunks": probes, "probe_total": sum(v for v in probes.values() if v),
           "probe_workers": {p.stem.replace(".workers", ""): int(p.read_text()) for p in sorted(rec.glob("probe_*.workers"))},
           "families": t("families"), "post": {n: t(n) for n in ("post_manifest", "post_prep", "post_describe")}}
    out["recording_total"] = out["probe_total"] + (out["families"] or 0)
    train = {n: r["seconds"] for n, r in info.items() if r.get("seconds") is not None}
    out["training_runs"] = len(train)
    out["training_total"] = sum(train.values())
    out["training_max"] = max(train.values()) if train else None
    eps = [r["epochs"] for n, r in info.items() if r.get("epochs") and r.get("model") in ("mlp", "seq")]
    out["neural_epochs"] = {"min": min(eps), "max": max(eps), "runs": len(eps)} if eps else None
    out["neural_stops"] = sorted({r.get("stop") for r in info.values() if r.get("model") in ("mlp", "seq")})
    ev = [r["eval.end"] - r["eval.start"] for r in info.values() if "eval.start" in r and "eval.end" in r]
    out["evaluations"] = len(ev)
    out["evaluation_total"] = sum(ev)
    return out


def build(runs: Path) -> dict:
    sc = Scorer(runs)
    cur = curves(sc)
    rl = rules(sc, cur)
    pre = load(runs / "_setup" / "prereg.json")
    info = runs_info(runs, (pre or {}).get("time"))
    evals = [r for r in info.values() if r.get("eval_after_prereg") is not None]
    return {"references": references(sc), "curves": cur, "rules": rl, "reproduction": reproduction(sc),
            "compute": compute_block(runs, info),
            "obs2": obs2_block(sc, rl.get("largest_n")), "per_cell": per_cell(sc, rl.get("largest_n")), "runs": info,
            "prereg": pre, "prereg_order_ok": bool(evals) and all(r["eval_after_prereg"] for r in evals),
            "test_worlds": len(sc.worlds), "test_cases": len(sc.cases)}


# ------------------------------------------------------------------ markdown ---------------------------------------
def v_row(name: str, w: dict | None) -> str:
    if not w:
        return f"| {name} | — | — | — | — | NOT_RUN |"
    seeds = ", ".join(fmt(s) for s in w["seeds"])
    return (f"| {name} | {fmt(w['model'])} ({seeds}) | `{w['baseline_member']}` {fmt(w['baseline'])} | {fmt(w.get('ratio'))} | "
            f"{ci_text(w['ci_diff'])} | **{w['verdict']}** |")


def markdown(j: dict) -> str:
    r, cur, ref = j["rules"], j["curves"], j["references"]
    nmax = r.get("largest_n")
    L = ["# WM-6 judge (generated by `tools/worldmodel/wm6/judge6.py`)", "",
         f"v2 test: {j['test_worlds']} worlds, {j['test_cases']} cases (both replicates), WM-2's scales, 12 cells per lag. "
         f"Largest N with `mlp_obs`, `mlp_priv`, `seq_obs` at >= 2 seeds: **{nmax}**. `*_priv` models are diagnostic.", "",
         "References (12-cell averages; closure = (zero − x) / (zero − floor)): " + "; ".join(
             f"`{n}` 7 d {fmt(v['7d']['value'])}, 30 d {fmt(v['30d']['value'])}" for n, v in ref.items()), "",
         "## Rules", "", "| rule | model (seeds) | baseline | ratio | Δ = model − baseline [95 % CI] | verdict |",
         "|---|---|---|---|---|---|"]
    for rule, lag in (("C1", "7d"), ("C2", "30d")):
        if rule in r:
            ch = r[rule]
            vals = ", ".join(f"`{m}` {fmt(v['val'])}" for m, v in ch["val_choice"].items())
            L.append(v_row(f"{rule} v2 {lag}, N = {nmax}: `{ch['chosen']}` (val {vals})", ch["verdict"]))
    for lag, w in (r.get("C3") or {}).items():
        L.append(v_row(f"C3 v2 {lag}, N = {nmax}: `mlp_priv` vs `ridge_priv` (diagnostic)", w))
    for m, node in (r.get("C4") or {}).items():
        L.append(v_row(f"C4 v2 7d: `{m}` N = {nmax} vs N = 400" + (" (diagnostic)" if m.endswith("priv") else ""),
                       node["verdict_7d"]))
    L += ["", "## C4 slopes (12-cell average per doubling of N; OLS on log2 N; 95 % bootstrap CI)", "",
          "| model | lag | N | values | slope [95 % CI] |", "|---|---|---|---|---|"]
    for m, node in (r.get("C4") or {}).items():
        for lag, s in node["slope"].items():
            if s:
                L.append(f"| `{m}` | {lag} | {s['ns']} | {', '.join(fmt(v) for v in s['values'])} | "
                         f"{s['point']:+.4f} [{s['lo']:+.4f}, {s['hi']:+.4f}] |")
    if r.get("C6"):
        L += ["", "## C6 — observable vs hidden at the largest N (Δ with 95 % CI; reported)", "", "| pair | lag | a | b | Δ [95 % CI] |",
              "|---|---|---|---|---|"]
        for k, v in r["C6"].items():
            pair, lag = k.split("|")
            L.append(f"| {pair} | {lag} | {fmt(v['a'])} | {fmt(v['b'])} | {ci_text(v['ci_diff'])} |")
    if r.get("reading"):
        rd = r["reading"]
        L += ["", f"## Pre-registered reading: **{rd['label'].replace('_', ' ')}**", "",
              f"C1 model `{rd['c1_model']}`: 7-day closure {fmt(rd['c1_closure_7d'])}, slope "
              f"{fmt(rd['c1_slope_7d'].get('point'), 4)} [{fmt(rd['c1_slope_7d'].get('lo'), 4)}, {fmt(rd['c1_slope_7d'].get('hi'), 4)}]; "
              f"`mlp_priv` closure {fmt(rd['mlp_priv_closure_7d'])}; closures at 7 d: "
              + ", ".join(f"`{m}` {fmt(v)}" for m, v in rd["closures_7d"].items()) + "; checks " + json.dumps(rd["checks"])]
    L += ["", "## Learning curves (12-cell average and closure [95 % CI] per N)", ""]
    for lag in LAGS:
        L += [f"### {lag}", "", "| model | " + " | ".join(f"N = {n}" for n in NS) + " |", "|---|" + "---|" * len(NS)]
        for m in MODELS:
            if m not in cur:
                continue
            cells = []
            for n in NS:
                node = cur[m].get(str(n))
                if node is None:
                    cells.append("—")
                    continue
                cl = node[lag]["closure"]
                cells.append(f"{fmt(node[lag]['value'])} (s={node['seeds']}); closure {fmt(cl['point'], 2)} [{fmt(cl['lo'], 2)}, {fmt(cl['hi'], 2)}]")
            L.append(f"| `{m}`{' (diag.)' if m.endswith('priv') else ''} | " + " | ".join(cells) + " |")
        L.append("")
    rp = j.get("reproduction") or {}
    if rp:
        L += ["## WM-5 reproduction (ridge, both replicates, N = 400)", "", "| model | reproduced | test max |Δ| | val max |Δ| | "
              "λ equal | 7 d WM-6 / WM-5 | 30 d WM-6 / WM-5 |", "|---|---|---|---|---|---|---|"]
        for n, v in rp.items():
            ca = v["cell_averages"]
            L.append(f"| `{n}` | {v['reproduced']} | {v['test_max_abs_diff']:.2e} | {(v['val_check'] or {}).get('max_abs_diff', float('nan')):.2e} | "
                     f"{(v['val_check'] or {}).get('lambdas_equal')} | {fmt(ca['7d']['wm6'])} / {fmt(ca['7d']['wm5'])} | "
                     f"{fmt(ca['30d']['wm6'])} / {fmt(ca['30d']['wm5'])} |")
        L.append("")
    o = j.get("obs2") or {}
    if o.get("models"):
        L += [f"## obs2 transfer (report only; N = {o['n']}, {o['worlds']} worlds, {o['cases']} r0 cases, WM-4's floored scales)", "",
              "| predictor | 7 d | 30 d |", "|---|---|---|"]
        for n, v in o["cell_averages"].items():
            L.append(f"| `{n}` | {fmt(v['7d'])} | {fmt(v['30d'])} |")
        L += ["", "| model | lag | vs `zero` Δ [CI] | vs WM-5 `ridge_obs` Δ [CI] |", "|---|---|---|---|"]
        for m, node in o["models"].items():
            for lag in LAGS:
                vz, vr = node[lag]["vs_zero"], node[lag]["vs_wm5_ridge_obs"]
                L.append(f"| `{m}` | {lag} | {ci_text(vz['ci_diff'])} ({vz['verdict']}) | "
                         f"{ci_text(vr['ci_diff']) + ' (' + vr['verdict'] + ')' if vr else '—'} |")
        L.append("")
    pc = j.get("per_cell") or {}
    if pc:
        names = list(pc)
        L += ["## Per cell (scaled MAE, seed-averaged, largest N)", "", "| cell | " + " | ".join(f"`{n}`" for n in names) + " |",
              "|---|" + "---|" * len(names)]
        for cell in pc[names[0]]:
            L.append(f"| {cell} | " + " | ".join(fmt(pc[n].get(cell)) for n in names) + " |")
        L.append("")
    L += ["## Runs", "", f"Pre-registration: {json.dumps(j.get('prereg'))}; every test evaluation after it: **{j['prereg_order_ok']}**", "",
          "| run | detail |", "|---|---|"]
    for n, v in j["runs"].items():
        L.append(f"| `{n}` | {json.dumps(v)[:500]} |")
    return "\n".join(L) + "\n"


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--runs", default=str(RUNS))
    ap.add_argument("--out", default=str(RUNS / "judge6"))
    a = ap.parse_args(argv)
    j = build(Path(a.runs))
    c.write_json(Path(a.out) / "judge.json", j)
    (Path(a.out) / "judge.md").write_text(markdown(j))
    print(markdown(j)[:4000])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
