"""The WM-3 verdicts (docs/worldmodel/wm3/METRICS.md, GOAL.md 8) from the evaluation files copied to
`docs/worldmodel/wm3/runs/`, beside WM-2's committed files (`docs/worldmodel/wm2/runs/`), and the analysis tables.
numpy only; writes `judge.json` and `judge.md`.

- V1 / V2 / V5: `wm2.effects.compare` / `rule` on cell errors (world-level paired bootstrap, B = 2,000, `default_rng(0)`).
- V3 / V4 / V6: `wm2.judge_wm2.pw_verdict` on per-world scores. V3's per-world score is the mean over the world's W4
  cases (both arms, both replicates) of |predicted - true split shift| in days; V4's the split-timing absolute error;
  V6's the per-world raw T2 168 h every-hour score (`bootstrap.world_scores` of `T2_any.168.test`).
- A `wm3_full` seed whose selected checkpoint is the init has an untrained hazard head: for V3 / V4 it is left out of
  the seed mean and the verdict is capped at WEAK (KEEP needs every seed); with no trained-hazard seed V3 / V4 are NONE.

usage: python3 tools/worldmodel/wm3/judge3.py [--runs DIR] [--wm2 DIR] [--out DIR]
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm3  # noqa: F401
from wm1 import bootstrap as bs
from wm1 import common as c
from wm2 import effects as ef
from wm2 import judge_wm2 as jw
from wm2 import v2

RUNS = c.REPO / "docs/worldmodel/wm3/runs"
WM2 = c.REPO / "docs/worldmodel/wm2/runs"
FULL = ("wm3_full_s0", "wm3_full_s1", "wm3_full_s2")
CTRL, OS = "wm3_ctrl_s0", "wm3_os_s0"
ALL = FULL + (CTRL, OS)
WM2_BACT = ("wm2_m1bact_v19_s0", "wm2_m1bact_v19_s1", "wm2_m1bact_v19_s2")
V1_CELLS = [(arm, o, "30d") for arm in v2.W3_ARMS for o in v2.OUTCOMES]
V2_CELLS = [(arm, o, "7d") for arm in v2.W3_ARMS for o in v2.OUTCOMES]
V2_BASE = "an5_arm"
V3_BASE = "an5_arm"


def load(path: Path) -> dict | None:
    return json.loads(path.read_text()) if path.exists() else None


def init_of(name: str) -> str:
    return "wm2_m1bact_v19_s" + name[-1]


def run_info(runs: Path, name: str) -> dict | None:
    cfg, val = load(runs / name / "config.json"), load(runs / name / "val_metrics.json")
    if cfg is None or val is None:
        return None
    log = [json.loads(x) for x in (runs / name / "train_log.jsonl").read_text().splitlines()] \
        if (runs / name / "train_log.jsonl").exists() else []
    steps = [x for x in log if not x.get("eval") and x.get("step", 0) > 0]
    evals = [x for x in log if x.get("eval")]
    sps = (steps[-1]["t"] - steps[0]["t"]) / (steps[-1]["step"] - steps[0]["step"]) if len(steps) > 1 else None
    h = cfg["hyper"]
    return {"config": cfg.get("wm3_config"), "stop": val["stop"], "best_step": val["best_step"], "last_step": val["last_step"],
            "selected": val["selected"], "seconds": val["seconds"], "not_converged": val["not_converged"],
            "best_val_eff": val["best_val_eff"], "init_val_eff": val["init_val_eff"], "init_val_t2": val["init_val_t2"],
            "round_counts": val["round_counts"], "s_per_step_incl_eval": sps,
            "eval_seconds_mean": float(np.mean([x["t2_s"] + x["eff_s"] for x in evals[1:]])) if len(evals) > 1 else None,
            "trajectory": val["trajectory"], "code_commit": cfg["code_commit"], "batch": h["batch"], "pair": h["pair"],
            "hazard": h["hazard"], "n_plain": h["n_plain"], "n_cast": h["n_cast"], "partners": h["partners"],
            "max_steps": h["max_steps"], "eval_every": h["eval_every"], "max_seconds": h["max_seconds"],
            "haz_pos_rate_mean": float(np.mean([x["haz_pos_rate"] for x in steps if x.get("haz_pos_rate") is not None]))
            if any(x.get("haz_pos_rate") is not None for x in steps) else None}


def effects_block(runs: Path, wm2: Path) -> dict:
    base = load(wm2 / "effects_base" / "effects_base.json")
    stds = ef.stds_from(base["stds"])
    cases = [ef.case_from(j) for j in base["cases"]["test"]]
    worlds = sorted({x.seed for x in cases})
    preds = dict(base["test_predictions"])
    meta = {}
    for r in WM2_BACT:
        e = load(wm2 / f"{r}_effects" / "effects.json")
        if e is not None:
            preds[r] = e["predictions"]
    for r in ALL:
        e = load(runs / f"{r}_effects" / "effects.json")
        if e is not None:
            preds[r] = e["predictions"]
            meta[r] = {"hazard": e["hazard"], "selected": e["selected"], "round_counts": e["round_counts"],
                       "drift": e["drift"], "code_commit": e["code_commit"]}
            if e["hazard"]:
                # The expected-day shift as the prediction of the split-shift cells (V3).
                preds[f"{r}|expected"] = {k: dict(v, shift=v["shift_expected"]) for k, v in e["predictions"].items()}
    errs = {n: ef.cell_errors(cases, p, stds) for n, p in preds.items()}
    out = {"tables": {}, "V": {}, "meta": meta, "worlds": len(worlds), "spread_test": base["spread"]["test"],
           "distribution_test": base["distribution"]["test"]}
    for n in preds:
        out["tables"][n] = ef.score(cases, preds[n], stds)
        for cell, rows in errs[n].items():
            key = "|".join(cell)
            out["tables"][n]["cells"][key]["mean_pred"] = float(np.mean([r[4] for r in rows]))
            out["tables"][n]["cells"][key]["mean_true"] = float(np.mean([r[3] for r in rows]))
    full = [r for r in FULL if r in preds]
    v1c = [x for x in V1_CELLS if stds[x] is not None]
    v2c = [x for x in V2_CELLS if stds[x] is not None]
    if full:
        out["V"]["V1"] = ef.compare([errs[r] for r in full], errs["zero"], v1c, worlds) | {
            "baseline_member": "zero", "runs": full, "others": {n: ef.cell_stat(errs[n], v1c, worlds) for n in
                                                                ("zero", "arm_mean", "an5_arm") + tuple(r for r in WM2_BACT if r in errs)}}
        out["V"]["V2"] = ef.compare([errs[r] for r in full], errs[V2_BASE], v2c, worlds) | {
            "baseline_member": V2_BASE, "runs": full, "others": {n: ef.cell_stat(errs[n], v2c, worlds) for n in
                                                                 ("zero", "arm_mean", "an5_arm") + tuple(r for r in WM2_BACT if r in errs)}}
        # V3: per-world mean |shift error| over the W4 cases, expected-day shift of the trained-hazard seeds.
        trained = [r for r in full if meta[r]["hazard"] and meta[r]["selected"] != "init"]
        pw = lambda n: shift_per_world(errs[n], worlds)
        node = {"baseline_member": V3_BASE, "runs": trained, "untrained": [r for r in full if r not in trained],
                "right_direction": {r: {a: out["tables"][f"{r}|expected"]["cells"][f"{a}|shift"]["right_direction"]
                                        for a in v2.W4_ARMS} for r in trained},
                "cell_compare": ef.compare([errs[f"{r}|expected"] for r in trained], errs[V3_BASE], ef.W4_CELLS, worlds)
                if trained else None,
                "event_based": {r: ef.cell_stat(errs[r], ef.W4_CELLS, worlds) for r in full},
                "others": {n: ef.cell_stat(errs[n], ef.W4_CELLS, worlds) for n in ("zero", "arm_mean", "an5_arm")
                           + tuple(r for r in WM2_BACT if r in errs)}}
        out["V"]["V3"] = capped(trained, full, [pw(f"{r}|expected") for r in trained], pw(V3_BASE)) | node
    v5 = {}
    for name, a, b_ in (("full_vs_ctrl", "wm3_full_s0", CTRL), ("os_vs_ctrl", OS, CTRL), ("full_vs_os", "wm3_full_s0", OS)):
        if a in errs and b_ in errs:
            v5[name] = ef.compare([errs[a]], errs[b_], v1c, worlds) | {"model_run": a, "baseline_member": b_}
    if v5:
        out["V"]["V5"] = v5
    # Every WM-3 run (and WM-2's seeds) against the baselines on V1's and V2's cells (reported).
    out["per_run"] = {n: {"V1_cells": ef.cell_stat(errs[n], v1c, worlds), "V2_cells": ef.cell_stat(errs[n], v2c, worlds),
                          "W4_cells_event": ef.cell_stat(errs[n], ef.W4_CELLS, worlds)}
                      for n in preds if "|" not in n}
    return out


def shift_per_world(rows_by_cell: dict, worlds: list[int]) -> dict:
    """{seed: mean over the world's W4 cases of |shift error| (days)}."""
    s, n = np.zeros(len(worlds)), np.zeros(len(worlds))
    for cell in ef.W4_CELLS:
        a, b_ = ef.world_sums(rows_by_cell[cell], worlds)
        s, n = s + a, n + b_
    return {str(w): float(s[i] / n[i]) for i, w in enumerate(worlds) if n[i] > 0}


def capped(trained: list[str], full: list[str], seed_pw: list[dict], base_pw: dict) -> dict:
    """`pw_verdict` on the trained-hazard seeds, capped at WEAK when a full seed has an untrained head; NONE with none."""
    if not trained:
        return {"verdict": "NONE", "reason": "no wm3_full seed with a trained hazard head"}
    res = jw.pw_verdict(seed_pw, base_pw)
    if len(trained) < len(full) and res["verdict"] == "KEEP":
        res["verdict"] = "WEAK"
        res["capped"] = "a seed's hazard head is untrained"
    return res


def split_block(runs: Path, wm2: Path) -> dict:
    base = load(wm2 / "split_base" / "split_base.json")
    sel = base["selection"]
    out = {"selection": sel, "baselines": {n: {"mae": v["mae"], "within_7d": v["within_7d"]}
                                           for n, v in base["scores"]["test"].items()}, "runs": {}}
    for r in WM2_BACT:
        s = load(wm2 / f"{r}_split" / "split.json")
        if s is not None:
            out["runs"][r] = {"event": {"mae": s["mae"], "within_7d": s["within_7d"],
                                        "censored_pred": s["censored_pred_share"]}}
    seeds_pw, trained, full = [], [], []
    for r in ALL:
        s = load(runs / f"{r}_split" / "split.json")
        if s is None:
            continue
        node = {"event": {"mae": s["event"]["mae"], "within_7d": s["event"]["within_7d"],
                          "censored_pred": s["event"]["censored_pred_share"]}, "selected": s["selected"]}
        if "expected" in s:
            node["expected"] = {"mae": s["expected"]["mae"], "within_7d": s["expected"]["within_7d"]}
        out["runs"][r] = node
        if r in FULL:
            full.append(r)
            if s["hazard"] and s["selected"] != "init" and "expected" in s:
                trained.append(r)
                seeds_pw.append(s["expected"]["per_world"])
    if full:
        out["V4"] = capped(trained, full, seeds_pw, base["scores"]["test"][sel]["per_world"]) | {
            "baseline_member": sel, "runs": trained}
    return out


def factual_block(runs: Path, wm2: Path) -> dict:
    out = {}
    for r in ALL:
        m = load(runs / f"{r}_fact" / "metrics.json")
        i = load(wm2 / f"{init_of(r)}_fact" / "metrics.json")
        if m is None or i is None:
            continue
        a, b_ = bs.world_scores(m["T2_any"]["168"]["test"]), bs.world_scores(i["T2_any"]["168"]["test"])
        res = jw.pw_verdict([a], b_) | {"baseline_member": init_of(r)}
        res["worse_by_more_than_0.005"] = res["ci_diff"]["lo"] > 0.005
        a24, b24 = bs.world_scores(m["T2_any"]["24"]["test"]), bs.world_scores(i["T2_any"]["24"]["test"])
        res["t2_24"] = {"model": float(np.mean(list(a24.values()))), "init": float(np.mean(list(b24.values()))),
                        "ci_diff": bs.paired(a24, b24)}
        out[r] = res
    return out


def lag_block(runs: Path, wm2: Path) -> dict:
    out = {"wm2": load(wm2 / "diag" / "effect_lag.json")}
    for r in ALL:
        x = load(runs / "diag" / f"effect_lag_{r}.json")
        if x is not None:
            out[r] = x
    return out


def build(runs: Path, wm2: Path) -> dict:
    j = {"runs": {n: run_info(runs, n) for n in ALL if run_info(runs, n) is not None}, "V": {}}
    j["effects"] = effects_block(runs, wm2)
    j["V"].update(j["effects"].pop("V"))
    sp = split_block(runs, wm2)
    j["split"] = sp
    if "V4" in sp:
        j["V"]["V4"] = sp["V4"]
    j["V"]["V6"] = factual_block(runs, wm2)
    j["lag"] = lag_block(runs, wm2)
    j["calib"] = {r: load(runs / f"{r}_calib" / "calib.json") for r in FULL if load(runs / f"{r}_calib" / "calib.json")}
    return j


def fmt(x, dd: int = 3) -> str:
    return "—" if x is None or (isinstance(x, float) and not np.isfinite(x)) else f"{x:.{dd}f}"


def ci_text(ci: dict, dd: int = 3) -> str:
    return f"{ci['point']:+.{dd}f} [{ci['lo']:+.{dd}f}, {ci['hi']:+.{dd}f}]"


def v_row(name: str, w: dict, dd: int = 3) -> str:
    if "ci_diff" not in w:
        return f"| {name} | — | — | — | — | **{w['verdict']}** ({w.get('reason', '')}) |"
    seeds = ", ".join(fmt(s, dd) for s in w["seeds"])
    return (f"| {name} | {fmt(w['model'], dd)} ({seeds}) | `{w['baseline_member']}` {fmt(w['baseline'], dd)} | "
            f"{fmt(w.get('ratio'), 3)} | {ci_text(w['ci_diff'], dd)} | **{w['verdict']}** |")


def markdown(j: dict) -> str:
    L = ["# WM-3 judge (generated by `tools/worldmodel/wm3/judge3.py`)", "",
         "| rule | model (seeds) | baseline | ratio | Δ = model − baseline [95 % CI] | verdict |", "|---|---|---|---|---|---|"]
    V = j["V"]
    names = {"V1": "V1 effects 30 d (`wm3_full`)", "V2": "V2 effects 7 d (`wm3_full`)",
             "V3": "V3 split shift, expected day (`wm3_full`, days)", "V4": "V4 split timing, expected day (`wm3_full`, days)"}
    for k in ("V1", "V2", "V3", "V4"):
        if k in V:
            L.append(v_row(names[k], V[k], 2 if k in ("V3", "V4") else 3))
    for k, w in V.get("V5", {}).items():
        L.append(v_row(f"V5 ablation `{w['model_run']}` vs `{w['baseline_member']}` (30 d cells)", w))
    for r, w in V.get("V6", {}).items():
        L.append(v_row(f"V6 factual T2 168 h `{r}` vs `{w['baseline_member']}`", w, 4))
    if "V3" in V and V["V3"].get("right_direction"):
        L += ["", "V3 right-direction share (expected-day shift, cases with a true shift ≠ 0): " +
              "; ".join(f"`{r}` " + ", ".join(f"{a} {fmt(x, 2)}" for a, x in d_.items()) for r, d_ in V["V3"]["right_direction"].items())]
    if "V6" in V:
        L += ["", "V6 guard: " + "; ".join(f"`{r}` {'worse by > 0.005 (CI lower > 0.005)' if w['worse_by_more_than_0.005'] else 'not worse by > 0.005'}"
                                           for r, w in V["V6"].items())]
    L += ["", "## Runs", "", "| run | stop | best step / last | selected | not converged | s/step (incl. eval) | eval s | seconds | "
          "val_eff init → best | val_t2 init | hazard positive rate |", "|---|---|---|---|---|---|---|---|---|---|---|"]
    for n, r in j["runs"].items():
        L.append(f"| `{n}` | {r['stop']} | {r['best_step']} / {r['last_step']} | {r['selected']} | {'yes' if r['not_converged'] else 'no'} | "
                 f"{fmt(r['s_per_step_incl_eval'], 2)} | {fmt(r['eval_seconds_mean'], 1)} | {r['seconds']:.0f} | "
                 f"{fmt(r['init_val_eff'], 4)} → {fmt(r['best_val_eff'], 4)} | {fmt(r['init_val_t2'], 4)} | {fmt(r['haz_pos_rate_mean'], 5)} |")
    L += ["", "val_eff trajectories (step: val_eff, * = not eligible):", ""]
    for n, r in j["runs"].items():
        L.append(f"- `{n}`: " + ", ".join(f"{t['step']}: {t['val_eff']:.4f}{'' if t['eligible'] else '*'}" for t in r["trajectory"]))
    e = j["effects"]
    preds = [p for p in e["tables"] if "|" not in p]
    L += ["", "## Effects (test families, both replicates; cells: scaled effect MAE / sign accuracy)", "",
          "| arm | outcome | lag | true mean (std) | spread | " + " | ".join(f"`{p}`" for p in preds) + " |",
          "|---|---|---|---|---|" + "---|" * len(preds)]
    for arm in v2.ARMS:
        if arm not in e["distribution_test"]:
            continue
        for o in v2.OUTCOMES:
            for lag in v2.LAGS:
                key = f"{arm}|{o}|{lag}"
                if key not in e["tables"][preds[0]]["cells"]:
                    continue
                dist = e["distribution_test"][arm].get(f"{o}|{lag}", {})
                sp = e["spread_test"].get(key, {}).get("mean")
                cells = [e["tables"][p]["cells"][key] for p in preds]
                L.append(f"| {arm} | {o} | {lag} | {fmt(dist.get('mean'), 2)} ({fmt(dist.get('std'), 2)}) | {fmt(sp)} | "
                         + " | ".join(f"{fmt(x['mae'])} / {fmt(x['sign_accuracy'], 2)}" for x in cells) + " |")
        key = f"{arm}|shift"
        dist = e["distribution_test"][arm]["shift"]
        cells = [e["tables"][p]["cells"][key] for p in preds]
        L.append(f"| {arm} | split shift, event (days) | — | {fmt(dist['mean'], 1)} ({fmt(dist['std'], 1)}) | "
                 f"{fmt(e['spread_test'].get(key, {}).get('mean'), 1)} | "
                 + " | ".join(f"{fmt(x['mae'], 1)} / {fmt(x.get('right_direction'), 2)}" for x in cells) + " |")
        ex = [p for p in e["tables"] if p.endswith("|expected")]
        if ex:
            L.append(f"| {arm} | split shift, expected (days) | — | | | " + " | ".join(
                (f"{fmt(e['tables'][p + '|expected']['cells'][key]['mae'], 1)} / {fmt(e['tables'][p + '|expected']['cells'][key].get('right_direction'), 2)}"
                 if p + "|expected" in e["tables"] else "") for p in preds) + " |")
    if e.get("meta"):
        L += ["", "Drift of the factual rollouts after each test cast row (T2 nMAE of the 8 V, test split std): " +
              "; ".join(f"`{r}` " + ", ".join(f"{k} {fmt(v['mean'])}" for k, v in sorted(m["drift"].items(), key=lambda kv: int(kv[0][:-1])))
                        for r, m in e["meta"].items())]
    L += ["", "Mean predicted vs mean true effect per cell (food store, `rw120E` wood is not an outcome):", "",
          "| arm | lag | true | " + " | ".join(f"`{p}`" for p in preds) + " |", "|---|---|---|" + "---|" * len(preds)]
    for arm in v2.ARMS:
        for lag in v2.LAGS:
            key = f"{arm}|food_stock|{lag}"
            if key in e["tables"][preds[0]]["cells"]:
                L.append(f"| {arm} | {lag} | {fmt(e['tables'][preds[0]]['cells'][key]['mean_true'], 1)} | " +
                         " | ".join(fmt(e["tables"][p]["cells"][key]["mean_pred"], 1) for p in preds) + " |")
    L += ["", "Cell averages (V1 cells = 12 at 30 d, V2 cells = 12 at 7 d, W4 = split shift event-based days):", "",
          "| predictor | V1 cells | V2 cells | W4 event |", "|---|---|---|---|"]
    for n, v in e["per_run"].items():
        L.append(f"| `{n}` | {fmt(v['V1_cells'])} | {fmt(v['V2_cells'])} | {fmt(v['W4_cells_event'], 2)} |")
    s = j["split"]
    L += ["", "## Split timing (start day 25 06:00, eligible test worlds)", "",
          f"baseline chosen on validation: `{s['selection']}`", "", "| predictor | event MAE | event ±7 d | expected MAE | expected ±7 d |",
          "|---|---|---|---|---|"]
    for n, v in s["baselines"].items():
        L.append(f"| `{n}` | {fmt(v['mae'], 2)} | {fmt(v['within_7d'], 3)} | | |")
    for n, v in s["runs"].items():
        x = v.get("expected", {})
        L.append(f"| `{n}` | {fmt(v['event']['mae'], 2)} | {fmt(v['event']['within_7d'], 3)} | {fmt(x.get('mae'), 2)} | {fmt(x.get('within_7d'), 3)} |")
    lag = j["lag"]
    if lag.get("wm2"):
        L += ["", "## One-step / lag curve of the food effect (first 40 validation families; lags in rows)", ""]
        lags = lag["wm2"]["lags"]
        L += ["| arm | predictor | " + " | ".join(str(x) for x in lags) + " |", "|---|---|" + "---|" * len(lags)]
        for arm in v2.ARMS:
            L.append(f"| {arm} | truth | " + " | ".join(fmt(x, 1) for x in lag["wm2"]["truth"][arm]["food_stock"]) + " |")
            for src, node in [("wm2", lag["wm2"])] + [(r, lag[r]) for r in ALL if r in lag]:
                for run, per in node["runs"].items():
                    if arm in per:
                        L.append(f"| {arm} | `{run}` | " + " | ".join(fmt(x, 1) for x in per[arm]["food_stock"]) + " |")
    if j.get("calib"):
        L += ["", "## Hazard calibration (val replicate-0 cast rows; P(split within 30 days) of the factual rollout)", ""]
        for r, cal in j["calib"].items():
            L.append(f"`{r}` (n {cal['n']}, mean predicted {fmt(cal['pred_mean'])}, observed {fmt(cal['obs_rate'])}, "
                     f"Brier {fmt(cal['brier'], 4)} vs climatology {fmt(cal['brier_climatology'], 4)}): " +
                     "; ".join(f"bin {b_['bin']} pred {fmt(b_['pred_mean'])} obs {fmt(b_['obs_rate'])} (n {b_['n']})" for b_ in cal["bins"]))
    return "\n".join(L) + "\n"


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--runs", default=str(RUNS))
    ap.add_argument("--wm2", default=str(WM2))
    ap.add_argument("--out", default=str(RUNS / "judge3"))
    a = ap.parse_args(argv)
    j = build(Path(a.runs), Path(a.wm2))
    c.write_json(Path(a.out) / "judge.json", j)
    (Path(a.out) / "judge.md").write_text(markdown(j))
    print(markdown(j))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
