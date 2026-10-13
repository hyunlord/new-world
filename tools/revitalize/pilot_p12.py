#!/usr/bin/env python3
"""Pass 12 Part 5 (PASS12_GOAL.md 11.2-11.4): combine Pass 11's pilot metrics and Pass 12's UBS into one table,
judge J7 on `game_v18` and attribute the changes. Stdlib only.

Inputs: `diversity_p11.py pilot` output (`diversity.json`: eta2, R2_abs, R2_year, dyn-eta2, divergence, spread,
nMAE, settlements alive) and `diversity_p12.py score` output (`ubs.json`: R2_static, UBS, paired bootstraps) for
the same four scenes on the same seeds.

J7 (test split, game_v18): eta2 >= 0.30 and R2_abs <= 0.70 for food_stock, wood_stock and alive. J7b (all
episodes, reference game_v17 on the same seeds): for alive and food_stock the paired-bootstrap 90 % interval of
delta UBS (v18 - v17, mean over days 60/90/118) lies above 0, and mean R2_static(alive) of v18 <= that of v17.

usage: pilot_p12.py --metrics DIR/diversity.json --ubs DIR/ubs.json --out DIR
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path

J7_VARS = ("food_stock", "wood_stock", "alive")
J7B_VARS = ("alive", "food_stock")
UBS_VARS = ("food_stock", "wood_stock", "alive", "deadwood_total", "food_on_bushes_total", "adults_working")
PAIRS = (("rain wet burn, region fill, daughter fixes, J6 fix", "game_v17", "game_v18_nofission"),
         ("hazard fission (with the region fill)", "game_v18_nofission", "game_v18"),
         ("Pass 12 as a whole", "game_v17", "game_v18"))


def fmt(x) -> str:
    return "—" if x is None else f"{x:.3f}"


def j7(metrics: dict, ubs: dict) -> dict:
    v18 = metrics["scenes"]["game_v18"]
    clauses = {}
    for n in J7_VARS:
        e, r = v18["eta2"][n], v18["r2_abs"][n]
        clauses[n] = {"eta2": e, "r2_abs": r, "passed": e is not None and e >= .30 and r is not None and r <= .70}
    bt = ubs["bootstrap"]["game_v18 - game_v17"]
    for n in J7B_VARS:
        lo, hi = bt["ci90"][n]
        clauses[f"ubs_{n}"] = {"delta": bt["delta_ubs_mean"][n], "ci90": [lo, hi], "passed": lo > 0}
    a18, a17 = ubs["scenes"]["game_v18"]["r2_static_mean"]["alive"], ubs["scenes"]["game_v17"]["r2_static_mean"]["alive"]
    clauses["r2_static_alive"] = {"v18": a18, "v17": a17, "passed": a18 is not None and a17 is not None and a18 <= a17}
    return {"passed": all(c["passed"] for c in clauses.values()), "clauses": clauses}


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--metrics", required=True, type=Path)
    ap.add_argument("--ubs", required=True, type=Path)
    ap.add_argument("--out", required=True, type=Path)
    args = ap.parse_args(argv)
    metrics = json.loads(args.metrics.read_text())
    ubs = json.loads(args.ubs.read_text())
    scenes = [s for s in ("game_v16", "game_v17", "game_v18_nofission", "game_v18") if s in metrics["scenes"]]
    out = {"scenes": scenes, "j7": j7(metrics, ubs), "attribution": {}, "table": {}}
    for s in scenes:
        m, u = metrics["scenes"][s], ubs["scenes"][s]
        out["table"][s] = {n: {"eta2": m["eta2"].get(n), "r2_abs": m["r2_abs"].get(n), "dyn_eta2": m["dyn_eta2"].get(n),
                               "r2_static": u["r2_static_mean"].get(n), "ubs": u["ubs_mean"].get(n)} for n in UBS_VARS}
        out["table"][s]["_settlements_alive"] = m["settlements_alive"]
        out["table"][s]["_t2_168"] = {n: {k: m["t2"]["168"][k][n] for k in ("persistence", "cal_abs", "apfit_abs")}
                                      for n in J7_VARS}
    for label, a, b in PAIRS:
        if a in scenes and b in scenes:
            out["attribution"][label] = {n: {"d_eta2": None if out["table"][b][n]["eta2"] is None or out["table"][a][n]["eta2"] is None
                                             else out["table"][b][n]["eta2"] - out["table"][a][n]["eta2"],
                                             "d_ubs": None if out["table"][b][n]["ubs"] is None or out["table"][a][n]["ubs"] is None
                                             else out["table"][b][n]["ubs"] - out["table"][a][n]["ubs"]} for n in UBS_VARS}
    out["bootstrap"] = ubs.get("bootstrap", {})
    lines = ["# Pass 12 diversity pilot (Part 5)", "",
             f"Seeds 24000-24111 (train 24000-24095, test 24096-24111), 120 days, `--every 60`, frozen recorder. eta2, R2_abs and "
             "dyn-eta2 on the test split (`diversity_p11.py`); R2_static and UBS on all 112 episodes (`diversity_p12.py`, mean over "
             "days 60/90/118).", "",
             "| Variable | " + " | ".join(scenes) + " |", "|---|" + "---|" * len(scenes)]
    for n in UBS_VARS:
        lines.append(f"| {n} | " + " | ".join(
            f"η² {fmt(out['table'][s][n]['eta2'])}, R²abs {fmt(out['table'][s][n]['r2_abs'])}, R²st {fmt(out['table'][s][n]['r2_static'])}, "
            f"UBS {fmt(out['table'][s][n]['ubs'])}, dyn {fmt(out['table'][s][n]['dyn_eta2'])}" for s in scenes) + " |")
    lines += ["", "## Settlements alive at days 30/60/90/118 (test seeds): mean / distribution", ""]
    for s in scenes:
        sa = out["table"][s]["_settlements_alive"]
        lines.append(f"- {s}: mean {[round(x, 2) for x in sa['mean']]}, distribution {sa['distribution']}")
    lines += ["", "## Baseline nMAE at 168 h (test split): persistence / cal_abs / apfit_abs", ""]
    for s in scenes:
        lines.append(f"- {s}: " + ", ".join(f"{n} {fmt(v['persistence'])} / {fmt(v['cal_abs'])} / {fmt(v['apfit_abs'])}"
                                            for n, v in out["table"][s]["_t2_168"].items()))
    lines += ["", "## Attribution (delta eta2 / delta UBS)", ""]
    for label, block in out["attribution"].items():
        lines.append(f"- {label}: " + ", ".join(f"{n} {fmt(v['d_eta2'])} / {fmt(v['d_ubs'])}" for n, v in block.items()))
    lines += ["", "## Paired bootstraps of delta UBS (90 % intervals)", ""]
    for label, bt in out["bootstrap"].items():
        lines.append(f"- {label}: " + ", ".join(f"{n} {fmt(bt['delta_ubs_mean'][n])} [{bt['ci90'][n][0]:.3f}, {bt['ci90'][n][1]:.3f}]"
                                                 for n in UBS_VARS))
    j = out["j7"]
    lines += ["", f"**J7 (game_v18): {'PASS' if j['passed'] else 'FAIL'}** — " +
              ", ".join(f"{k}: {'PASS' if v['passed'] else 'FAIL'}" for k, v in j["clauses"].items()), "",
              "```json", json.dumps(j["clauses"], indent=1, sort_keys=True), "```"]
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "diversity.json").write_text(json.dumps(out, indent=1, sort_keys=True) + "\n")
    (args.out / "diversity.md").write_text("\n".join(lines) + "\n")
    print("\n".join(lines))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
