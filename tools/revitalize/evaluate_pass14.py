#!/usr/bin/env python3
"""Pass 14 Part 4.2: the evaluation amendments of PASS14_GOAL.md 4.3, applied to Pass 13's outputs without rerunning.

usage: evaluate_pass14.py --j6 LABEL=P13_V19N_TARGETS/j_targets.json [--j6 ...] --paths P13_PILOT/paths.json
                          --diversity P13_PILOT/diversity.json --out DIR

- **J6 judged relative** (`game_v19_nofission` vs `game_v16` on the same seeds, Pass 6 Y9 clause): each Y9 clause
  (`Y9_work_1200`, `Y9_work_1500`, `Y9_idle_1200`) passes when the v19n median is no worse than v16's by more than
  0.03 in the clause's better direction (larger work share, smaller idle share); its absolute bound (0.35 / 0.25 /
  0.60) is reported only. Every other J6 clause keeps Pass 13's verdict. J6 passes when every clause does.
  Pass 13's own verdict is read from the same file and written beside the amended one, never overwritten.
- **J7b reported, not gated**: the path share PS (mean over days 60 / 90 / 118, 90 % world bootstrap) of `alive`
  and `food_stock` for game_v19 and the paired PS(alive) difference v19 - v18 are reported; J7 is then judged on
  its (a) clause (replicate-0 eta^2 / R^2_abs), Pass 13's numbers.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path

TOLERANCE = 0.03
Y9 = {"Y9_work_1200": (+1, 0.35), "Y9_work_1500": (+1, 0.25), "Y9_idle_1200": (-1, 0.60)}


def rescore_j6(report: dict) -> dict:
    j6 = report["targets"]["J6"]
    clauses = {}
    for name, c in j6["clauses"].items():
        if name in Y9:
            better, bound = Y9[name]
            obs = c["observed"]
            mine = obs.get("v19")
            ref = obs.get("game_v16")
            if mine is None or ref is None:
                raise ValueError(f"J6 clause {name} lacks the v19n or game_v16 observation")
            gap = better * (float(ref["median"]) - float(mine["median"]))  # > 0: v19n worse than v16
            ok = gap <= TOLERANCE + 1e-12
            clauses[name] = {"pass13_passed": c["passed"], "v19n_median": mine["median"], "v16_median": ref["median"],
                             "worse_by": gap, "passed": ok, "margin": TOLERANCE - gap,
                             "absolute_bound_reported": {"bound": bound, "v19n_meets": better * (float(mine["median"]) - bound) >= 0,
                                                         "v16_meets": better * (float(ref["median"]) - bound) >= 0}}
        else:
            clauses[name] = {"pass13_passed": c["passed"], "passed": c["passed"]}
    return {"pass13_passed": j6["passed"], "passed": all(c["passed"] for c in clauses.values()),
            "seeds": report.get("seeds"), "failing_clauses_pass13": [n for n, c in clauses.items() if not c["pass13_passed"]],
            "failing_clauses_amended": [n for n, c in clauses.items() if not c["passed"]], "clauses": clauses}


def j7(paths: dict, diversity: dict) -> dict:
    v19 = paths["scenes"]["game_v19"]["vars"]
    diff = paths["diffs"]["game_v19 - game_v18"]["vars"]["alive"]
    a = diversity["scenes"]["game_v19"]["j7"]
    ps = {v: {"PS_mean": v19[v]["PS_mean"], "ci90": v19[v]["ci90"]["PS"]["mean"]} for v in ("alive", "food_stock")}
    return {"a_passed": a["passed"], "a_clauses": a["clauses"], "passed": a["passed"],
            "b_reported": {"PS": ps, "PS_alive_v19_minus_v18": {"mean": diff["PS_mean"], "ci90": diff["ci90"]["PS"]["mean"]},
                           "retired_band": {"alive": [0.10, 0.50], "food_stock": [0.10, 0.60]},
                           "band_met": {v: 0.10 <= ps[v]["PS_mean"] <= (0.50 if v == "alive" else 0.60) for v in ps},
                           "v19_below_v18": diff["ci90"]["PS"]["mean"][1] < 0},
            "pass13_passed": bool(a["passed"] and all(0.10 <= ps[v]["PS_mean"] <= (0.50 if v == "alive" else 0.60) for v in ps)
                                  and diff["ci90"]["PS"]["mean"][1] < 0)}


def markdown(r: dict) -> str:
    L = ["# Pass 14 — evaluation amendments applied to Pass 13's outputs (PASS14_GOAL.md 4.3; no rerun)", "",
         "Scored by `tools/revitalize/evaluate_pass14.py`. Pass 13's verdicts are read from its committed `j_targets.json` "
         "and stand as written; the amended verdict is stated beside each.", "",
         "## J6 judged relative (`game_v19_nofission` vs `game_v16`, same seeds; Y9 clauses: no worse than v16 by more than 0.03)", "",
         "| run | Pass 13 J6 | amended J6 | clause | v19n median | v16 median | v19n worse by | amended clause | absolute bound (reported): v19n / v16 meet |",
         "|---|---|---|---|---:|---:|---:|---|---|"]
    for label, j in r["J6"].items():
        for name, c in j["clauses"].items():
            if name not in Y9:
                continue
            ab = c["absolute_bound_reported"]
            L.append(f"| {label} | {'PASS' if j['pass13_passed'] else 'FAIL'} | {'PASS' if j['passed'] else 'FAIL'} | {name} | "
                     f"{c['v19n_median']:.4f} | {c['v16_median']:.4f} | {c['worse_by']:+.4f} | {'PASS' if c['passed'] else 'FAIL'} (margin {c['margin']:.4f}) | "
                     f"{ab['bound']}: {'yes' if ab['v19n_meets'] else 'no'} / {'yes' if ab['v16_meets'] else 'no'} |")
        L.append(f"| {label} | | | other clauses: {len(j['clauses']) - 3}, failing in Pass 13 besides Y9: "
                 f"{[n for n in j['failing_clauses_pass13'] if n not in Y9] or 'none'} | | | | | |")
    j = r["J7"]
    b = j["b_reported"]
    L += ["", "## J7 with J7b reported, not gated (Pass 13 Part 5 replicate pilot, worlds 26000–26063, R = 4)", "",
          f"- Pass 13 verdict (recomputed from its rule): {'PASS' if j['pass13_passed'] else 'FAIL'}. Amended verdict: J7 = (a) = **{'PASS' if j['passed'] else 'FAIL'}** "
          f"(food η² {j['a_clauses']['food_stock']['eta2']:.3f} / R²_abs {j['a_clauses']['food_stock']['r2_abs']:.3f}, wood "
          f"{j['a_clauses']['wood_stock']['eta2']:.3f} / {j['a_clauses']['wood_stock']['r2_abs']:.3f}, alive "
          f"{j['a_clauses']['alive']['eta2']:.3f} / {j['a_clauses']['alive']['r2_abs']:.3f}).",
          f"- J7b reported: PS(alive) {b['PS']['alive']['PS_mean']:.3f} [{b['PS']['alive']['ci90'][0]:.3f}, {b['PS']['alive']['ci90'][1]:.3f}], "
          f"PS(food_stock) {b['PS']['food_stock']['PS_mean']:.3f} [{b['PS']['food_stock']['ci90'][0]:.3f}, {b['PS']['food_stock']['ci90'][1]:.3f}] "
          f"(retired bands [0.10, 0.50] / [0.10, 0.60] {'met' if all(b['band_met'].values()) else 'not met'}); PS(alive) v19 − v18 "
          f"{b['PS_alive_v19_minus_v18']['mean']:.3f} [{b['PS_alive_v19_minus_v18']['ci90'][0]:.3f}, {b['PS_alive_v19_minus_v18']['ci90'][1]:.3f}] "
          f"({'below 0' if b['v19_below_v18'] else 'not below 0'})."]
    return "\n".join(L) + "\n"


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--j6", action="append", required=True, help="LABEL=path/to/j_targets.json")
    ap.add_argument("--paths", required=True)
    ap.add_argument("--diversity", required=True)
    ap.add_argument("--out", required=True)
    a = ap.parse_args(argv)
    r = {"J6": {}, "inputs": {"j6": a.j6, "paths": a.paths, "diversity": a.diversity}}
    for item in a.j6:
        label, path = item.split("=", 1)
        r["J6"][label] = rescore_j6(json.loads(Path(path).read_text()))
    r["J7"] = j7(json.loads(Path(a.paths).read_text()), json.loads(Path(a.diversity).read_text()))
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    (out / "amendments.json").write_text(json.dumps(r, indent=1) + "\n")
    (out / "amendments.md").write_text(markdown(r))
    for label, j in r["J6"].items():
        print(f"J6 {label}: Pass 13 {'PASS' if j['pass13_passed'] else 'FAIL'} -> amended {'PASS' if j['passed'] else 'FAIL'}")
    print(f"J7: amended {'PASS' if r['J7']['passed'] else 'FAIL'} (b reported)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
