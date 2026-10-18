#!/usr/bin/env python3
"""Pass 16 evaluation: does temperament show in behaviour? (PASS16_GOAL.md 8.2, 8.4, 9, 10)

Reads probe output directories written with ``worldsim_probe --agent-trace``:
``<scene>_s<seed>_agents.jsonl`` (one line per person who was ever a non-elder
adult), ``_people.jsonl`` (every person), ``_fissions.jsonl`` and the probe
``summary.json``.

Statistics (section 9):
- Spearman's rho with average ranks over non-elder adults pooled across worlds
  (>= 10 non-elder adult days; idle distance needs >= 50 samples, bedtime >= 10
  nights, loads >= 10 deposits of that kind);
- 95 % intervals from a world-level bootstrap (B = 2,000; a fresh
  ``numpy.random.default_rng(0)`` per statistic; worlds resampled with
  replacement, every person of a drawn world included, percentile interval);
- T1-dep: mean NS of departers minus mean NS of eligible stayers, pooled over
  completed fissions;
- T3: paired over worlds, v20 - v19 (alive at the last day, non-old-age deaths,
  old-age deaths, fissions, first split day);
- T4: OLS slope of child on mid-parent over births with two known parents, and
  each axis's SD over people alive at the end vs the founders'.

usage:
  evaluate_pass16.py --v20 DIR --inert DIR [--v19 DIR] [--tuning] --out-md F --out-json F
In --tuning mode (8.2) only T1 without T1-dep and the health comparison of
v20 against the placebo (inert = v19 by construction) are computed.
"""
import argparse
import glob
import json
import os
import sys

import numpy as np

AXES = ("ns", "ha", "rd", "p")
MIN_ADULT_DAYS = 10
MIN_IDLE_SAMPLES = 50
MIN_NIGHTS = 10
MIN_DEPOSITS = 10
BOOTSTRAP = 2000
OLD_AGE = "old_age"

# Pre-registered bars (section 9).
T1_BARS = {"ns": 0.25, "ha": -0.25, "rd": 0.25, "p_food": 0.25}
T1_DEP_BAR = 80.0
T2_RHO = 0.10
T2_DEP = 40.0
T3_ALIVE_SHARE = 0.15
T3_DEATHS = 1.5
T4_SLOPE = (0.35, 0.65)
T4_SD_SHARE = 0.20


def rank_avg(values):
    """Ranks 1..n with ties given their average rank."""
    values = np.asarray(values, dtype=float)
    order = np.argsort(values, kind="mergesort")
    ranks = np.empty(len(values), dtype=float)
    sorted_values = values[order]
    i = 0
    while i < len(values):
        j = i
        while j + 1 < len(values) and sorted_values[j + 1] == sorted_values[i]:
            j += 1
        ranks[order[i:j + 1]] = (i + j) / 2.0 + 1.0
        i = j + 1
    return ranks


def spearman(x, y):
    """Spearman's rho with average ranks (Pearson on the ranks); nan if undefined."""
    if len(x) < 3:
        return float("nan")
    rx, ry = rank_avg(x), rank_avg(y)
    rx, ry = rx - rx.mean(), ry - ry.mean()
    den = np.sqrt((rx * rx).sum() * (ry * ry).sum())
    return float((rx * ry).sum() / den) if den > 0 else float("nan")


def ols_slope(x, y):
    """Least-squares slope of y on x; nan if undefined."""
    x, y = np.asarray(x, dtype=float), np.asarray(y, dtype=float)
    if len(x) < 3:
        return float("nan")
    dx = x - x.mean()
    den = (dx * dx).sum()
    return float((dx * (y - y.mean())).sum() / den) if den > 0 else float("nan")


VALID_DRAWS = []  # valid (non-nan) bootstrap draws of each call, in call order


def bootstrap(per_world, statistic, b=BOOTSTRAP):
    """World-level bootstrap: (point, lo, hi). ``per_world`` is a list of
    per-world payloads; ``statistic`` maps a list of payloads to a number.
    Draws whose statistic is undefined (nan: fewer than 3 pairs, no departer)
    are dropped; the number of valid draws is appended to ``VALID_DRAWS``."""
    point = statistic(per_world)
    rng = np.random.default_rng(0)
    n = len(per_world)
    values = []
    for _ in range(b):
        idx = rng.integers(0, n, size=n)
        v = statistic([per_world[i] for i in idx])
        if v == v:  # drop nan draws
            values.append(v)
    VALID_DRAWS.append(len(values))
    if not values:
        return point, float("nan"), float("nan")
    lo, hi = np.percentile(values, [2.5, 97.5])
    return point, float(lo), float(hi)


def read_json(path):
    with open(path) as f:
        return json.load(f)


def read_jsonl(path):
    with open(path) as f:
        return [json.loads(line) for line in f if line.strip()]


def load_scene(directory):
    """{seed: {"agents": [...], "people": {id: row}, "fissions": [...]}} and the summary."""
    worlds = {}
    for path in sorted(glob.glob(os.path.join(directory, "*_agents.jsonl"))):
        stem = os.path.basename(path)[: -len("_agents.jsonl")]
        seed = int(stem.rsplit("_s", 1)[1])
        base = path[: -len("_agents.jsonl")]
        worlds[seed] = {
            "agents": read_jsonl(path),
            "people": {row["id"]: row for row in read_jsonl(base + "_people.jsonl")},
            "fissions": read_jsonl(base + "_fissions.jsonl"),
            "meta": read_json(base + "_trace_meta.json"),
        }
    summary_path = os.path.join(directory, "summary.json")
    summary = read_json(summary_path) if os.path.exists(summary_path) else None
    return worlds, summary


def adult_rows(world):
    return [a for a in world["agents"] if a["adult_days"] >= MIN_ADULT_DAYS and a["temperament"] is not None]


def t1_pairs(world, measure):
    """(axis value, behaviour) pairs of one world for a T1 measure."""
    out = []
    for a in adult_rows(world):
        t = a["temperament"]
        if measure == "ns" and a["idle_samples"] >= MIN_IDLE_SAMPLES:
            out.append((t["ns"], a["idle_dist_sum"] / a["idle_samples"]))
        elif measure == "ha" and len(a["bed_minutes"]) >= MIN_NIGHTS:
            out.append((t["ha"], sum(a["bed_minutes"]) / len(a["bed_minutes"])))
        elif measure == "rd":
            out.append((t["rd"], a["social_completed"] / a["adult_days"]))
        elif measure == "p_food" and a["deposit_food_count"] >= MIN_DEPOSITS:
            out.append((t["p"], a["deposit_food_sum"] / a["deposit_food_count"]))
        elif measure == "p_wood" and a["deposit_wood_count"] >= MIN_DEPOSITS:
            out.append((t["p"], a["deposit_wood_sum"] / a["deposit_wood_count"]))
    return out


def pooled_rho(payloads):
    pairs = [p for world in payloads for p in world]
    if len(pairs) < 3:
        return float("nan")
    x, y = zip(*pairs)
    return spearman(x, y)


def departure_payload(world):
    """(departer NS values, eligible stayer NS values) over completed fissions."""
    dep, stay = [], []
    people = world["people"]
    for f in world["fissions"]:
        if f["status"] != "completed":
            continue
        members = set(f["members"])
        for pid in f["eligible"]:
            t = people.get(pid, {}).get("temperament")
            if t is None:
                continue
            (dep if pid in members else stay).append(t["ns"])
    return dep, stay


def dep_diff(payloads):
    dep = [v for d, _ in payloads for v in d]
    stay = [v for _, s in payloads for v in s]
    if not dep or not stay:
        return float("nan")
    return float(np.mean(dep) - np.mean(stay))


def quintile_table(worlds, measure):
    """Axis quintile (by pooled rank) -> (n, mean behaviour)."""
    pairs = [p for seed in sorted(worlds) for p in t1_pairs(worlds[seed], measure)]
    if not pairs:
        return []
    x = np.array([p[0] for p in pairs], dtype=float)
    y = np.array([p[1] for p in pairs], dtype=float)
    ranks = rank_avg(x)
    q = np.minimum((ranks - 1) * 5 // len(x), 4).astype(int)
    rows = []
    for k in range(5):
        sel = q == k
        if sel.any():
            rows.append({"quintile": k + 1, "n": int(sel.sum()), "axis_mean": float(x[sel].mean()), "behaviour_mean": float(y[sel].mean())})
    return rows


def mean_world_rho(payloads):
    """Sensitivity (review round 2, L4): the mean of per-world rho over worlds
    with >= 3 pairs — free of between-world differences in level."""
    rhos = [pooled_rho([p]) for p in payloads if len(p) >= 3]
    rhos = [r for r in rhos if r == r]
    return float(np.mean(rhos)) if rhos else float("nan")


def cohort_rho(world_list, measure, ticks_per_day=1440, split_day=60):
    """Sensitivity (review round 2, L3): pooled rho by birth cohort —
    founders, born before ``split_day``, born on or after it."""
    out = {}
    for name, keep in (
        ("founders", lambda a: a.get("founder")),
        (f"born_before_day_{split_day}", lambda a: not a.get("founder") and a["born_tick"] < split_day * ticks_per_day),
        (f"born_from_day_{split_day}", lambda a: not a.get("founder") and a["born_tick"] >= split_day * ticks_per_day),
    ):
        pairs = [p for w in world_list for p in t1_pairs({"agents": [a for a in w["agents"] if keep(a)]}, measure)]
        out[name] = {"rho": pooled_rho([pairs]), "n": len(pairs), "mean_behaviour": float(np.mean([b for _, b in pairs])) if pairs else float("nan")}
    return out


def t1_block(worlds, with_dep=True):
    seeds = sorted(worlds)
    out = {}
    for measure in ("ns", "ha", "rd", "p_food", "p_wood"):
        payloads = [t1_pairs(worlds[s], measure) for s in seeds]
        rho, lo, hi = bootstrap(payloads, pooled_rho)
        out[measure] = {"rho": rho, "lo": lo, "hi": hi, "n": sum(len(p) for p in payloads), "valid_draws": VALID_DRAWS[-1],
                        "mean_world_rho": mean_world_rho(payloads)}
    if with_dep:
        payloads = [departure_payload(worlds[s]) for s in seeds]
        diff, lo, hi = bootstrap(payloads, dep_diff)
        out["dep"] = {
            "valid_draws": VALID_DRAWS[-1],
            "diff": diff,
            "lo": lo,
            "hi": hi,
            "departers": sum(len(d) for d, _ in payloads),
            "stayers": sum(len(s) for _, s in payloads),
            "completed_fissions": sum(1 for s in seeds for f in worlds[s]["fissions"] if f["status"] == "completed"),
            "cancelled_fissions": sum(1 for s in seeds for f in worlds[s]["fissions"] if f["status"] == "cancelled"),
        }
    return out


def excludes_zero(block):
    return block["lo"] == block["lo"] and (block["lo"] > 0 or block["hi"] < 0)


def t1_verdicts(t1):
    v = {}
    for measure, bar in T1_BARS.items():
        b = t1[measure]
        ok = b["rho"] == b["rho"] and (b["rho"] >= bar if bar > 0 else b["rho"] <= bar) and excludes_zero(b)
        v[measure] = "PASS" if ok else "FAIL"
    if "dep" in t1:
        b = t1["dep"]
        if b["departers"] == 0:
            v["dep"] = "not computable (no completed fission)"
        else:
            v["dep"] = "PASS" if b["diff"] == b["diff"] and b["diff"] >= T1_DEP_BAR and excludes_zero(b) else "FAIL"
    return v


def t2_verdict(t1, with_wood=True):
    """Section 9: every T1 statistic |rho| <= 0.10 and |dep| <= 40. The wood
    rho has no verdict in T1; ``with_wood`` decides whether it counts here
    (the primary, stricter reading includes it; both are reported)."""
    measures = ("ns", "ha", "rd", "p_food", "p_wood") if with_wood else ("ns", "ha", "rd", "p_food")
    rhos = [abs(t1[m]["rho"]) for m in measures]
    ok = all(r == r and r <= T2_RHO for r in rhos)
    if "dep" in t1 and t1["dep"]["diff"] == t1["dep"]["diff"]:
        ok = ok and abs(t1["dep"]["diff"]) <= T2_DEP
    return "PASS" if ok else "FAIL"


def health(summary):
    """{seed: {...}} from a probe summary."""
    out = {}
    for run in summary["runs"]:
        causes = run["death_census"]["counts_by_cause"]
        out[run["seed"]] = {
            "alive_end": run["alive_daily"][-1],
            "non_old_age_deaths": sum(v for k, v in causes.items() if k != OLD_AGE),
            "old_age_deaths": causes.get(OLD_AGE, 0),
            "causes": causes,
            "fissions": run["pass11"]["fissions"],
            "first_split_day": run["pass11"]["first_fission_day"],
        }
    return out


def paired(a, b, key, seeds):
    diffs = [a[s][key] - b[s][key] for s in seeds]
    return bootstrap(diffs, lambda d: float(np.mean(d)) if d else float("nan"))


def t3_block(h20, href, seeds):
    alive_ref = float(np.mean([href[s]["alive_end"] for s in seeds]))
    alive_20 = float(np.mean([h20[s]["alive_end"] for s in seeds]))
    d_alive = paired(h20, href, "alive_end", seeds)
    d_deaths = paired(h20, href, "non_old_age_deaths", seeds)
    d_old = paired(h20, href, "old_age_deaths", seeds)
    d_fis = paired(h20, href, "fissions", seeds)
    both = [s for s in seeds if h20[s]["first_split_day"] is not None and href[s]["first_split_day"] is not None]
    d_split = paired(h20, href, "first_split_day", both) if both else (float("nan"),) * 3
    share = (alive_20 - alive_ref) / alive_ref if alive_ref else float("nan")
    ok = abs(share) <= T3_ALIVE_SHARE and d_deaths[0] <= T3_DEATHS
    causes = {}
    for scene, h in (("v20", h20), ("ref", href)):
        total = {}
        for s in seeds:
            for k, v in h[s]["causes"].items():
                total[k] = total.get(k, 0) + v
        causes[scene] = total
    return {
        "alive_end_mean": {"v20": alive_20, "ref": alive_ref, "share": share, "diff": d_alive},
        "non_old_age_deaths_diff": d_deaths,
        "non_old_age_deaths_mean": {"v20": float(np.mean([h20[s]["non_old_age_deaths"] for s in seeds])), "ref": float(np.mean([href[s]["non_old_age_deaths"] for s in seeds]))},
        "old_age_deaths_diff": d_old,
        "old_age_deaths_mean": {"v20": float(np.mean([h20[s]["old_age_deaths"] for s in seeds])), "ref": float(np.mean([href[s]["old_age_deaths"] for s in seeds]))},
        "fissions_diff": d_fis,
        "fissions_mean": {"v20": float(np.mean([h20[s]["fissions"] for s in seeds])), "ref": float(np.mean([href[s]["fissions"] for s in seeds]))},
        "first_split_day_diff": d_split,
        "first_split_worlds": {"v20": sum(1 for s in seeds if h20[s]["first_split_day"] is not None), "ref": sum(1 for s in seeds if href[s]["first_split_day"] is not None), "both": len(both)},
        "causes": causes,
        "verdict": "PASS" if ok else "FAIL",
    }


def t4_block(worlds):
    seeds = sorted(worlds)

    def births(world):
        rows = []
        for p in world["people"].values():
            pt = p.get("parent_temperaments") or [None, None]
            if p["founder"] or p["temperament"] is None or pt[0] is None or pt[1] is None:
                continue
            rows.append(([(pt[0][a] + pt[1][a]) // 2 for a in AXES], [p["temperament"][a] for a in AXES]))
        return rows

    payloads = [births(worlds[s]) for s in seeds]
    out = {"births_two_parents": sum(len(p) for p in payloads), "slope": {}, "sd": {}}
    for i, axis in enumerate(AXES):
        def stat(ps, i=i):
            pairs = [r for w in ps for r in w]
            return ols_slope([r[0][i] for r in pairs], [r[1][i] for r in pairs]) if pairs else float("nan")
        out["slope"][axis] = dict(zip(("value", "lo", "hi"), bootstrap(payloads, stat)))

    def pooled(ps):
        pairs = [r for w in ps for r in w]
        x = [r[0][i] for r in pairs for i in range(4)]
        y = [r[1][i] for r in pairs for i in range(4)]
        return ols_slope(x, y) if pairs else float("nan")

    out["slope"]["pooled"] = dict(zip(("value", "lo", "hi"), bootstrap(payloads, pooled)))
    founders = [p for s in seeds for p in worlds[s]["people"].values() if p["founder"] and p["temperament"]]
    alive = [p for s in seeds for p in worlds[s]["people"].values() if p["alive_end"] and p["temperament"]]
    ok = all(T4_SLOPE[0] <= out["slope"][a]["value"] <= T4_SLOPE[1] for a in AXES)
    for axis in AXES:
        sd_f = float(np.std([p["temperament"][axis] for p in founders]))
        sd_a = float(np.std([p["temperament"][axis] for p in alive])) if alive else float("nan")
        share = (sd_a - sd_f) / sd_f if sd_f else float("nan")
        out["sd"][axis] = {"founders": sd_f, "alive_end": sd_a, "share": share, "n_founders": len(founders), "n_alive": len(alive)}
        ok = ok and abs(share) <= T4_SD_SHARE
    out["verdict"] = "PASS" if ok else "FAIL"
    return out


def founders_table(worlds, h):
    rows = []
    for s in sorted(worlds):
        founders = [p for p in worlds[s]["people"].values() if p["founder"] and p["temperament"]]
        if not founders:
            continue
        row = {"seed": s, "founders": len(founders)}
        for a in AXES:
            row[a] = float(np.mean([p["temperament"][a] for p in founders]))
        if h and s in h:
            row["alive_end"] = h[s]["alive_end"]
            row["first_split_day"] = h[s]["first_split_day"]
        rows.append(row)
    return rows


def trace_meta(worlds):
    keys = ("born_event_misses", "component_mismatches", "fissions_completed", "fissions_cancelled", "fissions_pending", "people", "traced_adults")
    return {k: sum(w["meta"].get(k, 0) for w in worlds.values()) for k in keys}


def fmt(x, d=3):
    return "n/a" if x is None or x != x else f"{x:.{d}f}"


def render(result):
    lines = [f"# Pass 16 evaluation ({result['mode']})", ""]
    lines.append(f"Worlds: {result['seeds'][0]}–{result['seeds'][-1]} ({len(result['seeds'])}). Statistics: Spearman's rho (average ranks) over non-elder adults pooled across worlds; 95 % world-level bootstrap intervals (B = {BOOTSTRAP}, default_rng(0) per statistic).")
    lines.append("")
    lines.append("## T1 / T2 — rho per scene")
    lines.append("")
    lines.append("| measure | scene | rho | 95 % interval | n | valid draws | mean per-world rho (sensitivity) |")
    lines.append("|---|---|---|---|---|---|---|")
    for measure in ("ns", "ha", "rd", "p_food", "p_wood"):
        for scene, t1 in result["t1"].items():
            b = t1[measure]
            lines.append(f"| {measure} | {scene} | {fmt(b['rho'])} | [{fmt(b['lo'])}, {fmt(b['hi'])}] | {b['n']} | {b.get('valid_draws')} | {fmt(b.get('mean_world_rho'))} |")
    if any("dep" in t1 for t1 in result["t1"].values()):
        lines.append("")
        lines.append("| departure | scene | NS departers − stayers | 95 % interval | departers / stayers | completed / cancelled fissions | valid draws |")
        lines.append("|---|---|---|---|---|---|---|")
        for scene, t1 in result["t1"].items():
            if "dep" in t1:
                b = t1["dep"]
                lines.append(f"| T1-dep | {scene} | {fmt(b['diff'], 1)} | [{fmt(b['lo'], 1)}, {fmt(b['hi'], 1)}] | {b['departers']} / {b['stayers']} | {b['completed_fissions']} / {b['cancelled_fissions']} | {b.get('valid_draws')} |")
    lines.append("")
    lines.append("## Verdicts")
    lines.append("")
    for k, v in result["verdicts"].items():
        lines.append(f"- {k}: {v}")
    lines.append("")
    lines.append("## Axis quintile → behaviour mean")
    for measure, scenes in result["quintiles"].items():
        lines.append("")
        lines.append(f"**{measure}**")
        lines.append("")
        lines.append("| quintile | " + " | ".join(f"{s} axis / behaviour (n)" for s in scenes) + " |")
        lines.append("|---|" + "---|" * len(scenes))
        for k in range(5):
            cells = []
            for s, rows in scenes.items():
                row = next((r for r in rows if r["quintile"] == k + 1), None)
                cells.append(f"{fmt(row['axis_mean'], 0)} / {fmt(row['behaviour_mean'], 2)} ({row['n']})" if row else "—")
            lines.append(f"| {k + 1} | " + " | ".join(cells) + " |")
    lines.append("")
    lines.append("## T3 — health (paired, v20 − reference)")
    lines.append("")
    t3 = result["t3"]
    a = t3["alive_end_mean"]
    lines.append(f"- reference: {result['t3_reference']}")
    lines.append(f"- alive at the last day: v20 {fmt(a['v20'], 2)}, reference {fmt(a['ref'], 2)}, share {fmt(100 * a['share'], 1)} %, paired diff {fmt(a['diff'][0], 2)} [{fmt(a['diff'][1], 2)}, {fmt(a['diff'][2], 2)}]")
    d = t3["non_old_age_deaths_diff"]
    m = t3["non_old_age_deaths_mean"]
    lines.append(f"- non-old-age deaths per world: v20 {fmt(m['v20'], 2)}, reference {fmt(m['ref'], 2)}, paired diff {fmt(d[0], 2)} [{fmt(d[1], 2)}, {fmt(d[2], 2)}]")
    d = t3["old_age_deaths_diff"]
    m = t3["old_age_deaths_mean"]
    lines.append(f"- old-age deaths per world (reported): v20 {fmt(m['v20'], 2)}, reference {fmt(m['ref'], 2)}, paired diff {fmt(d[0], 2)} [{fmt(d[1], 2)}, {fmt(d[2], 2)}]")
    d = t3["fissions_diff"]
    m = t3["fissions_mean"]
    lines.append(f"- fissions per world (reported): v20 {fmt(m['v20'], 2)}, reference {fmt(m['ref'], 2)}, paired diff {fmt(d[0], 2)} [{fmt(d[1], 2)}, {fmt(d[2], 2)}]")
    d = t3["first_split_day_diff"]
    w = t3["first_split_worlds"]
    lines.append(f"- first split day (reported, worlds where both split: {w['both']}; v20 {w['v20']}, reference {w['ref']}): paired diff {fmt(d[0], 2)} [{fmt(d[1], 2)}, {fmt(d[2], 2)}]")
    lines.append(f"- deaths by cause: v20 {json.dumps(t3['causes']['v20'], sort_keys=True)}; reference {json.dumps(t3['causes']['ref'], sort_keys=True)}")
    lines.append(f"- T3 verdict: {t3['verdict']}")
    if "t4" in result:
        t4 = result["t4"]
        lines.append("")
        lines.append("## T4 — inheritance (v20)")
        lines.append("")
        lines.append(f"Births with two known parents: {t4['births_two_parents']}.")
        lines.append("")
        lines.append("| axis | slope on mid-parent | 95 % interval | SD founders | SD alive at end | change |")
        lines.append("|---|---|---|---|---|---|")
        for axis in AXES:
            s = t4["slope"][axis]
            sd = t4["sd"][axis]
            lines.append(f"| {axis} | {fmt(s['value'])} | [{fmt(s['lo'])}, {fmt(s['hi'])}] | {fmt(sd['founders'], 1)} | {fmt(sd['alive_end'], 1)} | {fmt(100 * sd['share'], 1)} % |")
        s = t4["slope"]["pooled"]
        lines.append(f"| pooled | {fmt(s['value'])} | [{fmt(s['lo'])}, {fmt(s['hi'])}] | | | |")
        lines.append("")
        lines.append(f"T4 verdict: {t4['verdict']}")
    if result.get("founders"):
        lines.append("")
        lines.append("## Founder temperament means vs outcomes (v20, descriptive — no verdict)")
        lines.append("")
        lines.append("| seed | founders | NS | HA | RD | P | alive at end | first split day |")
        lines.append("|---|---|---|---|---|---|---|---|")
        for r in result["founders"]:
            lines.append(f"| {r['seed']} | {r['founders']} | {fmt(r['ns'], 0)} | {fmt(r['ha'], 0)} | {fmt(r['rd'], 0)} | {fmt(r['p'], 0)} | {r.get('alive_end')} | {r.get('first_split_day')} |")
    if result.get("p_food_cohorts"):
        lines.append("")
        lines.append("## T1-P by birth cohort (sensitivity, no verdict)")
        lines.append("")
        lines.append("| scene | cohort | rho(P, food load) | n | mean load |")
        lines.append("|---|---|---|---|---|")
        for scene, cohorts in result["p_food_cohorts"].items():
            for name, c in cohorts.items():
                lines.append(f"| {scene} | {name} | {fmt(c['rho'])} | {c['n']} | {fmt(c['mean_behaviour'], 2)} |")
    if result.get("tuning"):
        lines.append("")
        lines.append("## What the tuning rounds changed (seeds 16100–16115, 60 days)")
        lines.append("")
        lines.append("| round | spans (NS / HA / RD / P food, wood) | rho NS | rho HA | rho RD | rho P food | rho P wood | T3 |")
        lines.append("|---|---|---|---|---|---|---|---|")
        for row in result["tuning"]:
            t = row["t1"]
            lines.append(f"| {row['round']} | {row['spans']} | {fmt(t['ns']['rho'])} | {fmt(t['ha']['rho'])} | {fmt(t['rd']['rho'])} | {fmt(t['p_food']['rho'])} | {fmt(t['p_wood']['rho'])} | {row['t3']} |")
        lines.append("")
        lines.append("Round 1 used section 6.2's start spans; T1-P (food) missed its bar, so round 2 raised only P's spans (food 2 → 3, wood 1 → 2) and the spans were frozen (8.3).")
    lines.append("")
    lines.append("## Notes on the measures (review round 1)")
    lines.append("")
    lines.append("- Pooled rho also carries between-world differences in level (e.g. the placebo's HA rho −0.058 has an interval excluding 0, while its mean per-world rho is near 0); the mean per-world rho is listed as a sensitivity column.")
    lines.append("- Bootstrap draws whose statistic is undefined (fewer than 3 pairs, no departer) are dropped; the valid-draw count is listed with each interval.")
    lines.append("- T2 is judged on every T1 statistic including the wood rho (which has no verdict in T1); the reading without it is listed beside it.")
    lines.append("- Idle distance is measured from the hearth of the first settlement listing the person. Departers stay on the parent's roster while walking to the new site (up to 2 days), so an Idle departer on the trek is sampled far from the old hearth; in v20 departers lean high-NS, so these few samples add to T1-NS through departure rather than the tether (the placebo is unaffected).")
    lines.append("- A deposit's load is the amount carried on the tick before the drop (section 6.5); a pickup inside the deposit radius on the same tick as the drain is not included, so loads are slightly undercounted alike in every scene.")
    lines.append("- Would-be (v19) temperaments use the AgentBorn parents alive at the end of the previous tick; in v20 the observer's computation is compared with the component (`component_mismatches`).")
    lines.append("")
    lines.append("## Trace bookkeeping")
    lines.append("")
    for scene, m in result["trace_meta"].items():
        lines.append(f"- {scene}: {json.dumps(m, sort_keys=True)}")
    return "\n".join(lines) + "\n"


def evaluate(v20_dir, inert_dir, v19_dir=None, tuning=False, tuning_json=()):
    scenes = {"v20": load_scene(v20_dir), "inert": load_scene(inert_dir)}
    if v19_dir:
        scenes["v19"] = load_scene(v19_dir)
    seeds = sorted(scenes["v20"][0])
    for name, (worlds, _) in scenes.items():
        if sorted(worlds) != seeds:
            raise SystemExit(f"{name}: seeds {sorted(worlds)} differ from v20's {seeds}")
    result = {"mode": "tuning" if tuning else "evaluation", "seeds": seeds, "t1": {}, "quintiles": {}, "trace_meta": {}}
    for name, (worlds, _) in scenes.items():
        result["t1"][name] = t1_block(worlds, with_dep=not tuning)
        result["trace_meta"][name] = trace_meta(worlds)
    for measure in ("ns", "ha", "rd", "p_food", "p_wood"):
        result["quintiles"][measure] = {name: quintile_table(worlds, measure) for name, (worlds, _) in scenes.items()}
    verdicts = {f"T1-{k}": v for k, v in t1_verdicts(result["t1"]["v20"]).items()}
    for name in scenes:
        if name != "v20":
            verdicts[f"T2-{name}"] = t2_verdict(result["t1"][name])
            verdicts[f"T2-{name} (without the wood rho)"] = t2_verdict(result["t1"][name], with_wood=False)
    ref_name = "inert" if tuning or "v19" not in scenes else "v19"
    h20 = health(scenes["v20"][1])
    href = health(scenes[ref_name][1])
    result["t3_reference"] = ref_name
    result["t3"] = t3_block(h20, href, seeds)
    verdicts["T3"] = result["t3"]["verdict"]
    if not tuning:
        result["t4"] = t4_block(scenes["v20"][0])
        verdicts["T4"] = result["t4"]["verdict"]
        result["t4_placebo"] = {name: t4_block(scenes[name][0]) for name in scenes if name != "v20"}
        result["founders"] = founders_table(scenes["v20"][0], h20)
        result["p_food_cohorts"] = {name: cohort_rho([w for _, w in sorted(scenes[name][0].items())], "p_food") for name in scenes}
    result["tuning"] = []
    for i, (spans, path) in enumerate(tuning_json):
        t = read_json(path)
        result["tuning"].append({"round": i + 1, "spans": spans, "t1": t["t1"]["v20"], "t3": t["verdicts"]["T3"]})
    result["verdicts"] = verdicts
    return result


def main(argv):
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--v20", required=True)
    ap.add_argument("--inert", required=True)
    ap.add_argument("--v19")
    ap.add_argument("--tuning", action="store_true")
    ap.add_argument("--out-md", required=True)
    ap.add_argument("--out-json", required=True)
    ap.add_argument("--tuning-json", action="append", default=[], metavar="SPANS=PATH",
                    help="a tuning round's eval.json with its spans label, e.g. '4 / 60 / 500 / 2, 1=path'")
    args = ap.parse_args(argv)
    tuning_json = [tuple(item.rsplit("=", 1)) for item in args.tuning_json]
    result = evaluate(args.v20, args.inert, args.v19, args.tuning, tuning_json)
    with open(args.out_json, "w") as f:
        json.dump(result, f, indent=1, sort_keys=True)
        f.write("\n")
    with open(args.out_md, "w") as f:
        f.write(render(result))
    for k, v in result["verdicts"].items():
        print(k, v)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
