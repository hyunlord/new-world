#!/usr/bin/env python3
"""M1.5 evaluation scorer (goal 9): bars D1-D5 and J1 from the colony probe's outputs.

usage: m1_5_score.py EVAL_DIR [--markdown]

EVAL_DIR holds the probe runs of the evaluation (seeds 200-231):
  years3/   colony, 72 days (3 years), no orders     -> D1, D2, D3, D4
  years17/  colony, 408 days (17 years), no orders   -> D3 (longer), D5
  j1/       colony, 10 days, tools/v8/u_script.json   -> J1
Each holds summary.json (per seed: deaths, jobs, `demography` from sim_test::probe::demography).
Prints one line per bar with the verdict and, with --markdown, the per-seed appendix table.
"""
import json
import sys
from pathlib import Path

YEAR_TICKS = 34_560


def seeds(path):
    return json.loads((path / "summary.json").read_text())["seeds"]


def main(argv):
    if not argv:
        print(__doc__)
        return 2
    root = Path(argv[0])
    md = "--markdown" in argv
    y3, y17, j1 = seeds(root / "years3"), seeds(root / "years17"), seeds(root / "j1")
    n = len(y3)
    lines, verdicts = [], {}

    # D1: births per woman-year (fertile, paired, fed days) in [0.15, 0.40] on the mean; no seed > 0.6.
    rates = [s["demography"]["births_per_woman_year"] for s in y3]
    births = sum(s["demography"]["births"] for s in y3)
    woman_years = sum(s["demography"]["woman_years"] for s in y3)
    mean_rate = sum(rates) / n
    pooled = births / woman_years if woman_years else 0.0
    d1 = 0.15 <= mean_rate <= 0.40 and max(rates) <= 0.6
    verdicts["D1"] = d1
    lines.append(f"D1 births/woman-year: mean over seeds {mean_rate:.3f} (pooled {pooled:.3f}: {births} births in "
                 f"{woman_years:.1f} woman-years), range {min(rates):.3f}-{max(rates):.3f} -> {'PASS' if d1 else 'FAIL'}")

    # D1b (supplementary, M1.5 DECISIONS D16): conceptions per at-risk woman-day against the rule's 33 permille.
    risk = sum(s["demography"].get("at_risk_days", 0) for s in y3)
    conc = sum(s["demography"].get("conceptions", 0) for s in y3)
    if risk:
        lines.append(f"D1b conceptions per at-risk woman-day: {conc}/{risk} = {conc/risk:.4f} (rule 0.033)")

    # D2: population after 3 years in [8, 22], no extinction, on >= 28 / 32.
    pops = [s["demography"]["alive_end"] for s in y3]
    ok2 = sum(8 <= p <= 22 and not s["demography"]["extinct"] for p, s in zip(pops, y3))
    d2 = ok2 >= 28
    verdicts["D2"] = d2
    lines.append(f"D2 population after 3 years: {min(pops)}-{max(pops)} (mean {sum(pops)/n:.1f}); in [8, 22] on {ok2}/{n} "
                 f"-> {'PASS' if d2 else 'FAIL'}")

    # D3: no old-age death under 40 years; founders' old-age deaths in [45, 85] years; 32 / 32 (3- and 17-year runs).
    def d3_ok(s):
        for death in s["demography"]["old_age_deaths"]:
            age = death["age_years"]
            if age < 40 or (death["founder"] and not 45 <= age <= 85):
                return False
        return True
    old3 = sum(len(s["demography"]["old_age_deaths"]) for s in y3)
    old17 = sum(len(s["demography"]["old_age_deaths"]) for s in y17)
    ages17 = sorted(d["age_years"] for s in y17 for d in s["demography"]["old_age_deaths"])
    ok3 = sum(d3_ok(s) and d3_ok(t) for s, t in zip(y3, y17))
    d3 = ok3 == n
    verdicts["D3"] = d3
    lines.append(f"D3 old-age deaths: {old3} in 3 years, {old17} in 17 years (ages {ages17[:1] and ages17[0]}-"
                 f"{ages17[-1:] and ages17[-1]} years); bar holds on {ok3}/{n} -> {'PASS' if d3 else 'FAIL'}")

    # D4: no hunger / thirst / cold death in 3 years, >= 30 / 32.
    def bad(s):
        d = s["deaths"]
        return d["starvation"] + d["dehydration"] + d["exposure"]
    ok4 = sum(bad(s) == 0 for s in y3)
    d4 = ok4 >= 30
    verdicts["D4"] = d4
    misses = [(s["seed"], s["deaths"]) for s in y3 if bad(s)]
    lines.append(f"D4 no hunger/thirst/cold death in 3 years: {ok4}/{n} -> {'PASS' if d4 else 'FAIL'}"
                 + (f"; misses {misses}" if misses else ""))

    # D5: a child born in the first year (year index 0) is an adult at 15 years and carries no Child after;
    # 32 / 32 where such a birth exists. The 17-year run is long enough for every year-0 and year-1 child.
    ok5 = with5 = 0
    detail5 = []
    for s in y17:
        dem = s["demography"]
        kids = [c for c in dem["year_one_children"]]
        early = [c for c in kids if c["born_tick"] < YEAR_TICKS]
        if not early:
            continue
        with5 += 1
        good = dem["child_marker_past_child_age"] == 0 and all(
            c["came_of_age_tick"] is not None and c["age_years_at_coming_of_age"] == 15
            and 15 * YEAR_TICKS <= c["age_ticks_at_coming_of_age"] < 15 * YEAR_TICKS + 60 for c in early)
        ok5 += good
        detail5.append((s["seed"], len(early), good))
    d5 = with5 > 0 and ok5 == with5
    verdicts["D5"] = d5
    lines.append(f"D5 year-1 children adult at 15: {ok5}/{with5} seeds with a year-1 birth "
                 f"({sum(d[1] for d in detail5)} children) -> {'PASS' if d5 else 'FAIL' if with5 else 'NO DATA'}")

    # J1: jobs finished / taken >= 0.5 with the U-script, 10 days, on >= 28 / 32.
    ratios = [s["jobs_done"] / s["jobs_taken"] if s["jobs_taken"] else 0.0 for s in j1]
    ok_j = sum(r >= 0.5 for r in ratios)
    taken = sum(s["jobs_taken"] for s in j1)
    done = sum(s["jobs_done"] for s in j1)
    deaths_j = sum(sum(s["deaths"].values()) for s in j1)
    j = ok_j >= 28 and deaths_j == 0
    verdicts["J1"] = j
    lines.append(f"J1 finished/taken >= 0.5: {ok_j}/{n} (range {min(ratios):.3f}-{max(ratios):.3f}; {done}/{taken} = "
                 f"{done/taken:.3f}); deaths with the script {deaths_j} -> {'PASS' if j else 'FAIL'}")

    print("\n".join(lines))
    if md:
        print("\n| seed | births (3 y) | births / woman-year | woman-years | people year 1/2/3 | hunger/thirst/cold deaths (3 y) "
              "| people at 17 y | old-age deaths (17 y, ages) | year-1 children → adult at 15 | jobs taken / done (U-script) | ratio |")
        print("|---|---|---|---|---|---|---|---|---|---|---|")
        for a, b, c in zip(y3, y17, j1):
            d, e = a["demography"], b["demography"]
            pop = d["population_by_year"]
            kids = [k for k in e["year_one_children"] if k["born_tick"] < YEAR_TICKS]
            kid_txt = ", ".join(f"{k['age_years_at_coming_of_age']} y" for k in kids) or "—"
            old = ", ".join(str(x["age_years"]) for x in e["old_age_deaths"]) or "—"
            print(f"| {a['seed']} | {d['births']} | {d['births_per_woman_year']:.3f} | {d['woman_years']:.1f} | "
                  f"{pop.get('1')}/{pop.get('2')}/{pop.get('3')} | {a['deaths']['starvation']}/{a['deaths']['dehydration']}/"
                  f"{a['deaths']['exposure']} | {e['alive_end']} | {old} | {kid_txt} | {c['jobs_taken']} / {c['jobs_done']} | "
                  f"{(c['jobs_done']/c['jobs_taken'] if c['jobs_taken'] else 0):.3f} |")
    return 0 if all(verdicts.values()) else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
