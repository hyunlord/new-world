"""The WM-2 verdicts (METRICS.md, GOAL.md 8): W1-W6 from the run files copied to `docs/worldmodel/wm2/runs/`, with the
world-level paired bootstrap, and the analysis tables (floor and headroom, effects per arm and outcome beside the true
effect distribution, split shift, split timing and theta, drift, convergence). numpy only; writes `judge.json` and
`judge.md`.

usage: python3 tools/worldmodel/wm2/judge_wm2.py [--runs DIR] [--out DIR]
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm2  # noqa: F401
import bench_v0 as b
from wm1 import bootstrap as bs
from wm1 import common as c
from wm2 import effects as ef
from wm2 import v2

RUNS = c.REPO / "docs/worldmodel/wm2/runs"
M1BACT = ("wm2_m1bact_v19_s0", "wm2_m1bact_v19_s1", "wm2_m1bact_v19_s2")
M1LONG = ("wm2_m1long_v19_s0", "wm2_m1long_v19_s1")
V16 = ("wm2_m1b_v16_s0",)


def load(runs: Path, *parts) -> dict | None:
    p = runs.joinpath(*parts)
    return json.loads(p.read_text()) if p.exists() else None


def fmt(x, d: int = 3) -> str:
    return "—" if x is None or (isinstance(x, float) and not np.isfinite(x)) else f"{x:.{d}f}"


def ci_text(ci: dict, d: int = 3) -> str:
    return f"{ci['point']:+.{d}f} [{ci['lo']:+.{d}f}, {ci['hi']:+.{d}f}]"


def pw_verdict(seed_pw: list[dict], base_pw: dict) -> dict:
    """The W1 rule on per-world scores (`wm1.bootstrap`; one seed: WEAK at most)."""
    model = bs.average(seed_pw)
    ci = bs.paired(model, base_pw)
    base = float(np.mean(list(base_pw.values())))
    seeds = [float(np.mean(list(s.values()))) for s in seed_pw]
    return {"verdict": ef.rule(seeds, base, ci), "model": float(np.mean(seeds)), "seeds": seeds, "baseline": base,
            "ratio": float(np.mean(seeds)) / base, "ci_diff": ci, "keep_threshold": 0.8 * base,
            "seed_ci_diff": [bs.paired(s, base_pw) for s in seed_pw]}


def trained(runs: Path, names) -> list[str]:
    return [n for n in names if load(runs, n, "config.json") is not None]


def run_info(runs: Path, name: str) -> dict | None:
    cfg, val = load(runs, name, "config.json"), load(runs, name, "val_metrics.json")
    if cfg is None or val is None:
        return None
    last, best = val["last_step"], val["best_step"]
    log = [json.loads(x) for x in (runs / name / "train_log.jsonl").read_text().splitlines()] if (runs / name / "train_log.jsonl").exists() else []
    evals = [x for x in log if x.get("eval")]
    steps = [x for x in log if not x.get("eval") and x["k"] == cfg["hyper"]["k_max"]]
    sps = (steps[-1]["t"] - steps[0]["t"]) / (steps[-1]["step"] - steps[0]["step"]) if len(steps) > 1 else None
    return {"stop": val["stop"], "best_step": best, "last_step": last, "seconds": val["seconds"],
            "converged": not (val["stop"] in ("max_steps", "time") and best >= 0.9 * last),
            "best_val_t2_24": val["best_val_t2_24"], "s_per_step_k_max_incl_eval": sps,
            "eval_seconds_mean": float(np.mean([x["eval_s"] for x in evals if "eval_s" in x])) if evals else None,
            "max_steps": cfg["hyper"]["max_steps"], "round_counts": val["round_counts"], "code_commit": cfg["code_commit"],
            "dataset": cfg["dataset"], "model": cfg["model"], "burn_in": cfg["hyper"].get("burn_in"),
            "val_frac": cfg["hyper"].get("val_frac"), "val_starts": cfg.get("val_starts"), "train_episodes": cfg["train_episodes"]}


def calendar_nodes(runs: Path, tag: str, key: str, horizon: str, br: bool = False) -> tuple[str, dict, dict]:
    val = load(runs, f"calendar_val_{tag}", "calendar_val.json")
    test = load(runs, f"calendar_{tag}", "calendar.json")
    sel = val["selection"][key]
    member = sel["cal_best_anom"] if br else sel["cal_best"]
    node = test["test"][member]["T2_any"][horizon]["test"]
    return member, node, test


def factual(runs: Path, tag: str, fact_runs: list[str], horizon: str, suffix: str = "_fact") -> dict:
    member, node, test = calendar_nodes(runs, tag, f"T2any_{horizon}", horizon)
    base = bs.world_scores(node)
    seeds = [bs.world_scores(load(runs, f"{r}{suffix}", "metrics.json")["T2_any"][horizon]["test"]) for r in fact_runs]
    out = pw_verdict(seeds, base) | {"baseline_member": member, "runs": fact_runs}
    refs = {}
    for name in ("an5", "apfit_abs", "cal_abs"):
        refs[name] = float(np.mean(list(bs.world_scores(test["test"][name]["T2_any"][horizon]["test"]).values())))
    for name in ("persistence", "ridge"):
        refs[name] = float(np.mean(list(bs.world_scores(test["references"][name]["T2_any"][horizon]["test"]).values())))
    out["references"] = refs
    return out


def effects_block(runs: Path, fact_seeds: dict[str, list[str]]) -> dict:
    base = load(runs, "effects_base", "effects_base.json")
    stds = ef.stds_from(base["stds"])
    cases = [ef.case_from(j) for j in base["cases"]["test"]]
    worlds = sorted({x.seed for x in cases})
    w3 = ef.w3_cells(stds)
    preds = {n: p for n, p in base["test_predictions"].items()}
    for label, names in fact_seeds.items():
        for r in names:
            e = load(runs, f"{r}_effects", "effects.json")
            if e is not None:
                preds[r] = e["predictions"]
    errs = {n: ef.cell_errors(cases, p, stds) for n, p in preds.items()}
    out = {"selection": base["selection"], "dropped_cells": base["dropped_cells"], "worlds": len(worlds),
           "spread_test": base["spread"]["test"], "distribution_test": base["distribution"]["test"],
           "distribution_train": base["distribution"]["train"], "tables": {}, "W": {}}
    for n in preds:
        out["tables"][n] = ef.score(cases, preds[n], stds)
    spread = base["spread"]["test"]
    for label, names in fact_seeds.items():
        have = [r for r in names if r in errs]
        if not have:
            continue
        for w, cells, sel in (("W3", w3, base["selection"]["W3"]["chosen"]), ("W4", ef.W4_CELLS, base["selection"]["W4"]["chosen"])):
            res = ef.compare([errs[r] for r in have], errs[sel], cells, worlds)
            res |= {"baseline_member": sel, "runs": have, "cells": ["|".join(x) for x in cells],
                    "spread": float(np.mean([spread["|".join(x)]["mean"] for x in cells]))}
            if w == "W3":
                res["others"] = {n: ef.cell_stat(errs[n], cells, worlds) for n in ("zero", "arm_mean", "an5_arm")}
            else:
                res["others"] = {n: ef.cell_stat(errs[n], cells, worlds) for n in ("zero", "arm_mean", "an5_arm")}
                res["right_direction"] = {r: {a: out["tables"][r]["cells"][f"{a}|shift"]["right_direction"] for a in v2.W4_ARMS} for r in have}
                res["censored_pred"] = {}
            out["W"][f"{w}{'' if label == 'm1bact' else '-' + label}"] = res
    out["drift"] = {r: load(runs, f"{r}_effects", "effects.json")["drift"] for label, names in fact_seeds.items()
                    for r in names if load(runs, f"{r}_effects", "effects.json") is not None}
    return out


def split_block(runs: Path, bact: list[str], blong: list[str]) -> dict:
    base = load(runs, "split_base", "split_base.json")
    sel = base["selection"]
    base_pw = {k: v for k, v in base["scores"]["test"][sel]["per_world"].items()}
    out = {"selection": sel, "val": {n: v["mae"] for n, v in base["scores"]["val"].items()},
           "test": {n: {"mae": v["mae"], "within_7d": v["within_7d"]} for n, v in base["scores"]["test"].items()},
           "eligible_counts": base["eligible_counts"], "censored_share": base["censored_share"]}
    models = {}
    for r in bact + blong:
        s = load(runs, f"{r}_split", "split.json")
        if s is not None:
            models[r] = s
            out["test"][r] = {"mae": s["mae"], "within_7d": s["within_7d"], "censored_pred": s["censored_pred_share"]}
    lw = [models[r]["per_world"] for r in blong if r in models]
    bw = [models[r]["per_world"] for r in bact if r in models]
    if lw:
        out["W5"] = pw_verdict(lw, base_pw) | {"baseline_member": sel, "runs": [r for r in blong if r in models]}
    if lw and bw:
        out["W5b"] = pw_verdict(lw, bs.average(bw)) | {"baseline_member": "M1b-act (seed mean)",
                                                       "runs": [r for r in blong if r in models]}
    out["explain"] = load(runs, "split_explain", "split_explain.json")
    return out


def floor_block(runs: Path) -> dict:
    out = {}
    for tag in ("v19", "v16"):
        f = load(runs, f"floor_{tag}", "floor.json")
        if f is None:
            continue
        out[tag] = {"floor": {h: {"floor": v["floor"], "uncorrected": v["floor_uncorrected"], "per_var": v["per_var"],
                                  "per_start": v["per_start"]} for h, v in f["floor"].items()},
                    "matched": {n: {h: v[h]["score"] for h in v} for n, v in f["predictors"].items()},
                    "headroom": f["headroom"], "cal_best": f["cal_best"]}
    return out


def build(runs: Path) -> dict:
    bact, blong, b16 = trained(runs, M1BACT), trained(runs, M1LONG), trained(runs, V16)
    out = {"runs": {n: run_info(runs, n) for n in bact + blong + b16}, "W": {}}
    fb = [r for r in bact if load(runs, f"{r}_fact", "metrics.json")]
    if fb and load(runs, "calendar_v19", "calendar.json"):
        out["W"]["W1"] = factual(runs, "v19", fb, "168")
        out["W"]["W1b"] = factual(runs, "v19", fb, "24")
        fl = [r for r in blong if load(runs, f"{r}_fact", "metrics.json")]
        if fl:
            out["factual_long"] = {"168": factual(runs, "v19", fl, "168"), "24": factual(runs, "v19", fl, "24")}
    f16 = [r for r in b16 if load(runs, r, "metrics.json")]
    if f16 and load(runs, "calendar_v16", "calendar.json"):
        out["W"]["W6"] = factual(runs, "v16", f16, "168", suffix="")
        out["W6_24"] = factual(runs, "v16", f16, "24", suffix="")
    if load(runs, "effects_base", "effects_base.json"):
        eff = effects_block(runs, {"m1bact": bact, "long": blong})
        out["effects"] = eff
        for k, v in eff["W"].items():
            out["W"][k] = v
    if load(runs, "split_base", "split_base.json"):
        sp = split_block(runs, bact, blong)
        out["split"] = sp
        for k in ("W5", "W5b"):
            if k in sp:
                out["W"][k] = sp[k]
    out["floor"] = floor_block(runs)
    return out


def w_row(name: str, w: dict, unit: str = "") -> str:
    seeds = ", ".join(fmt(s) for s in w["seeds"])
    return (f"| {name} | {fmt(w['model'])} ({seeds}) | `{w['baseline_member']}` {fmt(w['baseline'])} | "
            f"{fmt(w['ratio'], 3) if w.get('ratio') is not None else '—'} | {ci_text(w['ci_diff'])} | **{w['verdict']}** |")


def markdown(j: dict) -> str:
    L = ["# WM-2 judge (generated by `tools/worldmodel/wm2/judge_wm2.py`)", "",
         "| rule | model (seeds) | baseline | ratio | Δ = model − baseline [95 % CI] | verdict |", "|---|---|---|---|---|---|"]
    names = {"W1": "W1 factual raw T2 168 h", "W1b": "W1b factual raw T2 24 h", "W3": "W3 effects (M1b-act)",
             "W3-long": "W3-long effects (M1-long)", "W4": "W4 split shift (M1b-act, days)",
             "W4-long": "W4 split shift (M1-long, days)", "W5": "W5 split timing (M1-long vs best day baseline)",
             "W5b": "W5b split timing (M1-long vs M1b-act)", "W6": "W6 control game_v16 raw T2 168 h"}
    for k in ("W1", "W1b", "W3", "W3-long", "W4", "W4-long", "W5", "W5b", "W6"):
        if k in j["W"]:
            L.append(w_row(names[k], j["W"][k]))
    L += ["", "## Runs", "", "| run | stop | best step / last | converged | s/step (k max, incl. eval) | eval s | seconds | val T2 24 h |",
          "|---|---|---|---|---|---|---|---|"]
    for n, r in j["runs"].items():
        if r:
            L.append(f"| `{n}` | {r['stop']} | {r['best_step']} / {r['last_step']} | {'yes' if r['converged'] else 'no'} | "
                     f"{fmt(r['s_per_step_k_max_incl_eval'], 2)} | {fmt(r['eval_seconds_mean'], 1)} | {r['seconds']:.0f} | {fmt(r['best_val_t2_24'], 4)} |")
    if j.get("floor"):
        L += ["", "## Floor and floor-matched scores", ""]
        for tag, f in j["floor"].items():
            L.append(f"**{tag}**: floor 24 h {fmt(f['floor']['24']['floor'])} (uncorrected {fmt(f['floor']['24']['uncorrected'])}), "
                     f"168 h {fmt(f['floor']['168']['floor'])} (uncorrected {fmt(f['floor']['168']['uncorrected'])}); headroom "
                     f"(cal_best − floor) 24 h {ci_text(f['headroom']['24'])}, 168 h {ci_text(f['headroom']['168'])}")
            L += ["", "| predictor | 24 h | 168 h |", "|---|---|---|"]
            for n, v in f["matched"].items():
                L.append(f"| `{n}` | {fmt(v.get('24'))} | {fmt(v.get('168'))} |")
            L.append("")
    if j.get("effects"):
        e = j["effects"]
        L += ["## Effects (test families, both replicates)", "",
              f"Baseline chosen on validation: W3 `{e['selection']['W3']['chosen']}` ({json.dumps({k: round(v, 4) for k, v in e['selection']['W3']['val'].items()})}), "
              f"W4 `{e['selection']['W4']['chosen']}` ({json.dumps({k: round(v, 3) for k, v in e['selection']['W4']['val'].items()})})", ""]
        preds = list(e["tables"])
        L += ["| arm | outcome | lag | true mean (std) | spread | " + " | ".join(f"`{p}`" for p in preds) + " |",
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
            L.append(f"| {arm} | split shift (days) | — | {fmt(dist['mean'], 1)} ({fmt(dist['std'], 1)}) | "
                     f"{fmt(e['spread_test'].get(key, {}).get('mean'), 1)} | "
                     + " | ".join(f"{fmt(x['mae'], 1)} / {fmt(x.get('right_direction'), 2)}" for x in cells) + " |")
        L += ["", "(cells: scaled effect MAE / sign accuracy; split shift: MAE in days / right-direction share)", ""]
        if e.get("drift"):
            L += ["Drift of the factual rollouts (T2 nMAE of the 8 V after the cast): " +
                  "; ".join(f"`{r}` " + ", ".join(f"{k} {fmt(v['mean'])}" for k, v in dr.items()) for r, dr in e["drift"].items()), ""]
    if j.get("split"):
        s = j["split"]
        L += ["## Split timing (start day 25 06:00, worlds without a split yet)", "",
              f"eligible (train / val / test): {s['eligible_counts']}; censored share of the truth {json.dumps(s['censored_share'])}; "
              f"baseline chosen on validation `{s['selection']}` (val MAE {json.dumps({k: round(v, 2) for k, v in s['val'].items()})})", "",
              "| predictor | MAE (days) | within ±7 d |", "|---|---|---|"]
        for n, v in s["test"].items():
            L.append(f"| `{n}` | {fmt(v['mae'], 2)} | {fmt(v['within_7d'], 3)} |")
        if s.get("explain"):
            L += ["", "theta share of the signed error (LOO R² with theta + window summary − summary only): " +
                  ", ".join(f"`{n}` {fmt(v['theta_share'])}" for n, v in s["explain"]["per_predictor"].items())]
    return "\n".join(L) + "\n"


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--runs", default=str(RUNS))
    ap.add_argument("--out", default=str(RUNS / "judge_wm2"))
    a = ap.parse_args(argv)
    j = build(Path(a.runs))
    c.write_json(Path(a.out) / "judge.json", j)
    (Path(a.out) / "judge.md").write_text(markdown(j))
    print(markdown(j))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
