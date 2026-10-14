#!/usr/bin/env python3
"""Revitalize Pass 15 scaling analysis (PASS15_GOAL.md 7.4 and 7.5), standard library only.

usage: perf_p15.py --natural DIR --stress DIR --map DIR [--overhead FILE] [--repeat DIR] --out DIR

Reads the per-run JSON files of `worldsim_perf` (and, for pooled percentiles, their
`<stem>.tick_us.bin` sidecars when present) and writes `scaling.json` and `scaling.md`
into --out. `inventory.md` and `audit.md` in --out, when present, are appended to
`scaling.md` (the per-frame caller inventory of 6.5 and the static audit of 7.4).

Slopes are ordinary least squares on logs with a 95 % interval b +- t(0.975, n - 2) * se(b),
se(b) = sqrt(SSE / (n - 2) / Sxx). Classes: b < 1.2 linear or better, 1.2 <= b < 1.7
superlinear, b >= 1.7 near-quadratic. Percentiles are nearest-rank, as worldsim_perf.
"""
import argparse
import json
import math
import os
import statistics
import struct
import sys

B1_MS = 4.0
B2_MS = 8.0
B3_MS = 2.0
FRAME_MS = 16.67
TARGET = 500
WARMUP_TICKS = 60
NATURAL_MIN_ALIVE = 15
STRESS_MIN_ALIVE = 80
CROWDING_MARGIN = 0.5
MINOR_SHARE = 0.01
MAJOR_SHARE = 0.05
OPTIONAL_FIX_SHARE = 0.10

# Per-frame getters (6.5 inventory; PF and PT classes) and their calls per frame.
# "T" = distinct agent tiles (`get_tile_causal_history` runs once per tile, twice a frame);
# "H" = hearths. Getters the inventory lists that do not exist on this branch
# (art: get_society_snapshot, get_intervention_state) cannot be timed here.
PER_FRAME_WIP = {
    "get_world_info": 1, "get_building_snapshot": 1, "get_time_snapshot": 2, "get_tallies": 2,
    "get_tile_causal_history": "2T", "get_agent_snapshot": 6, "get_construction_snapshot": 2,
    "get_resource_snapshot": 1, "get_recent_deaths": 1, "get_settlement_snapshot": 3,
}
PER_FRAME_ART = {
    "get_world_info": 2, "get_terrain_snapshot": 1, "get_building_snapshot": 4, "get_time_snapshot": "13+5H",
    "get_tallies": 2, "get_tile_causal_history": "2T", "get_agent_snapshot": 9, "get_construction_snapshot": 4,
    "get_resource_snapshot": 5, "get_recent_deaths": 3, "get_settlement_snapshot": "10+2H",
}
UNTIMED_ART = ["get_society_snapshot (5 per frame)", "get_intervention_state (4 per frame)"]


# ---------------------------------------------------------------- statistics

def _betacf(a, b, x):
    """Continued fraction of the regularized incomplete beta (Numerical Recipes)."""
    qab, qap, qam = a + b, a + 1.0, a - 1.0
    c, d = 1.0, 1.0 - qab * x / qap
    d = 1.0 / (d if abs(d) > 1e-300 else 1e-300)
    h = d
    for m in range(1, 300):
        m2 = 2 * m
        aa = m * (b - m) * x / ((qam + m2) * (a + m2))
        d = 1.0 + aa * d
        d = 1.0 / (d if abs(d) > 1e-300 else 1e-300)
        c = 1.0 + aa / c if abs(c) > 1e-300 else 1e-300
        h *= d * c
        aa = -(a + m) * (qab + m) * x / ((a + m2) * (qap + m2))
        d = 1.0 + aa * d
        d = 1.0 / (d if abs(d) > 1e-300 else 1e-300)
        c = 1.0 + aa / c if abs(c) > 1e-300 else 1e-300
        delta = d * c
        h *= delta
        if abs(delta - 1.0) < 1e-15:
            break
    return h


def betainc(a, b, x):
    """Regularized incomplete beta I_x(a, b)."""
    if x <= 0.0:
        return 0.0
    if x >= 1.0:
        return 1.0
    lbeta = math.lgamma(a + b) - math.lgamma(a) - math.lgamma(b) + a * math.log(x) + b * math.log(1.0 - x)
    if x < (a + 1.0) / (a + b + 2.0):
        return math.exp(lbeta) * _betacf(a, b, x) / a
    return 1.0 - math.exp(lbeta) * _betacf(b, a, 1.0 - x) / b


def t_cdf(t, df):
    """Student t cumulative distribution."""
    x = df / (df + t * t)
    tail = 0.5 * betainc(df / 2.0, 0.5, x)
    return 1.0 - tail if t >= 0 else tail


def t_quantile(p, df):
    """Inverse Student t CDF by bisection (p in (0, 1))."""
    lo, hi = -1e3, 1e3
    for _ in range(200):
        mid = (lo + hi) / 2.0
        if t_cdf(mid, df) < p:
            lo = mid
        else:
            hi = mid
    return (lo + hi) / 2.0


def ols(xs, ys):
    """y = a + b x by least squares: dict with b, a, n, se, ci (95 %), r2; None below 3 points."""
    n = len(xs)
    if n < 3 or len(set(xs)) < 2:
        return None
    mx, my = sum(xs) / n, sum(ys) / n
    sxx = sum((x - mx) ** 2 for x in xs)
    sxy = sum((x - mx) * (y - my) for x, y in zip(xs, ys))
    b = sxy / sxx
    a = my - b * mx
    sse = sum((y - a - b * x) ** 2 for x, y in zip(xs, ys))
    syy = sum((y - my) ** 2 for y in ys)
    se = math.sqrt(sse / (n - 2) / sxx) if sse > 0 else 0.0
    half = t_quantile(0.975, n - 2) * se
    return {"b": b, "a": a, "n": n, "se": se, "ci": [b - half, b + half], "r2": 1.0 - sse / syy if syy > 0 else 1.0}


def loglog(xs, ys):
    """OLS of log(y) on log(x) over the pairs with x > 0 and y > 0."""
    pairs = [(math.log(x), math.log(y)) for x, y in zip(xs, ys) if x > 0 and y > 0]
    return ols([p[0] for p in pairs], [p[1] for p in pairs])


def classify(b):
    """The growth class of slope b."""
    if b is None:
        return "—"
    if b < 1.2:
        return "linear or better"
    if b < 1.7:
        return "superlinear"
    return "near-quadratic"


def nearest_rank(p, n):
    """1-based nearest rank ceil(p / 100 * n), in integer arithmetic (as worldsim_perf)."""
    milli = int(round(p * 1000))
    return max(1, min(n, -(-milli * n // 100_000)))


def percentile(sorted_values, p):
    """Nearest-rank percentile of an ascending list."""
    return sorted_values[nearest_rank(p, len(sorted_values)) - 1] if sorted_values else 0


def read_tick_us(path):
    """A `.tick_us.bin` sidecar as a list of ints (little-endian u32)."""
    with open(path, "rb") as stream:
        data = stream.read()
    return list(struct.unpack(f"<{len(data) // 4}I", data))


def summary_from_tick_us(tick_us):
    """Mean and nearest-rank p99.9 (ms) of the ticks after the warm-up."""
    steady = sorted(tick_us[WARMUP_TICKS:])
    return {"ticks": len(steady), "mean_ms": sum(steady) / len(steady) / 1000.0,
            "p999_ms": percentile(steady, 99.9) / 1000.0, "max_ms": steady[-1] / 1000.0}


# ---------------------------------------------------------------- loading

def load_runs(directory):
    """Every run JSON (status ok) in `directory`, keyed by stem; worldgen/run failures from runs.json."""
    runs, failures = {}, []
    if not directory or not os.path.isdir(directory):
        return runs, failures
    for name in sorted(os.listdir(directory)):
        if name.endswith(".json") and name not in ("runs.json",) and not name.startswith("overhead_"):
            with open(os.path.join(directory, name)) as stream:
                run = json.load(stream)
            run["_stem"] = name[:-5]
            run["_dir"] = directory
            runs[run["_stem"]] = run
    index = os.path.join(directory, "runs.json")
    if os.path.exists(index):
        with open(index) as stream:
            failures = [r for r in json.load(stream)["runs"] if r.get("status") != "ok"]
    return runs, failures


def system_names(run):
    return [s["name"] for s in run["systems"]]


# ---------------------------------------------------------------- analysis

def natural_points(runs):
    """Per system: (alive_0600, ms that day) over full days with alive >= 15; and the whole tick."""
    points, tick_points = {}, []
    for run in runs.values():
        names = system_names(run)
        tpd = run["ticks_per_day"]
        for day in run["per_day"]:
            alive = day.get("alive_0600")
            if day["ticks"] != tpd or alive is None or alive < NATURAL_MIN_ALIVE:
                continue
            tick_points.append((alive, day["tick_ms"] / tpd))
            for name, ms in zip(names, day["systems_ms"]):
                points.setdefault(name, []).append((alive, ms))
    return points, tick_points


def stress_points(runs, key_x="mean_alive_0600", min_x=STRESS_MIN_ALIVE):
    """Per system: (mean alive, mean µs per call) over runs with mean alive >= min_x; and the whole tick."""
    points, tick_points = {}, []
    for run in runs.values():
        x = run[key_x]
        if x < min_x:
            continue
        tick_points.append((x, run["tick_summary"]["mean_ms"]))
        for s in run["systems"]:
            points.setdefault(s["name"], []).append((x, s["mean_us_per_call"]))
    return points, tick_points


def fit(points):
    return loglog([p[0] for p in points], [p[1] for p in points]) if points else None


def by_size(runs):
    sizes = {}
    for run in runs.values():
        sizes.setdefault(run["agents_requested"], []).append(run)
    return dict(sorted(sizes.items()))


def pooled_steady(runs):
    """Pooled steady tick times (µs) of `runs` from their sidecars; None if any sidecar is missing."""
    pooled = []
    for run in runs:
        path = os.path.join(run["_dir"], run["_stem"] + ".tick_us.bin")
        if not os.path.exists(path):
            return None
        pooled.extend(read_tick_us(path)[WARMUP_TICKS:])
    return sorted(pooled)


def median_snapshot(runs, getter):
    values = [r["snapshot_timings"][getter]["median_us"] for r in runs if r.get("snapshot_timings") and getter in r["snapshot_timings"]]
    return statistics.median(values) if values else None


def context_median(runs, key):
    values = [r["snapshot_timings"]["context"][key] for r in runs if r.get("snapshot_timings")]
    return statistics.median(values) if values else None


def callers_cost(runs, table):
    """(single-call sum µs, ×callers sum µs, rows) for the per-frame getters of `table`."""
    tiles, hearths = context_median(runs, "agent_tiles"), context_median(runs, "hearths")
    single = 0.0
    total = 0.0
    rows = []
    for getter, calls in table.items():
        one = median_snapshot(runs, getter)
        if one is None:
            rows.append((getter, calls, None, None))
            continue
        single += one
        if calls == "2T":
            cost = 2 * (median_snapshot(runs, "get_tile_causal_history_all_agent_tiles") or 0.0)
            count = 2 * tiles
        elif isinstance(calls, str):
            base, per_h = calls.split("+")
            count = int(base) + int(per_h[:-1]) * hearths
            cost = one * count
        else:
            count = calls
            cost = one * calls
        total += cost
        rows.append((getter, count, one, cost))
    return single, total, rows, tiles, hearths


def analyse(natural, stress, map_runs, overhead=None, repeat=None):
    """Everything 7.4 and 7.5 need, as a dict."""
    out = {"n_runs": {"natural": len(natural), "stress": len(stress), "map": len(map_runs)}}
    fed = {k: r for k, r in stress.items() if r["feed"] == "fed"}
    raw = {k: r for k, r in stress.items() if r["feed"] == "raw"}
    sizes_fed = by_size(fed)
    sizes_raw = by_size(raw)

    # Shares: pooled natural (all steady time), and per stress size (median share over seeds).
    names = system_names(next(iter((natural or fed).values())))
    nat_total = sum(r["steady_tick_ms_total"] for r in natural.values())
    nat_share = {n: (sum(next(s["total_ms"] for s in r["systems"] if s["name"] == n) for r in natural.values()) / nat_total if nat_total else 0.0) for n in names}
    size_share = {size: {n: statistics.median(next(s["share_of_tick_time"] for s in r["systems"] if s["name"] == n) for r in runs) for n in names}
                  for size, runs in sizes_fed.items()}
    out["shares"] = {"natural_pooled": nat_share, "fed_by_size": {str(k): v for k, v in size_share.items()}}

    # Slopes.
    nat_pts, nat_tick = natural_points(natural)
    st_pts, st_tick = stress_points(fed)
    raw_pts, raw_tick = stress_points(raw)
    map_pts, map_tick = {}, []
    for run in map_runs.values():
        tiles = run["side"] ** 2
        map_tick.append((tiles, run["tick_summary"]["mean_ms"]))
        for s in run["systems"]:
            map_pts.setdefault(s["name"], []).append((tiles, s["mean_us_per_call"]))
    slopes = {}
    for n in names:
        nat, st, rw, mp = fit(nat_pts.get(n, [])), fit(st_pts.get(n, [])), fit(raw_pts.get(n, [])), fit(map_pts.get(n, []))
        shares_all = [nat_share.get(n, 0.0)] + [v.get(n, 0.0) for v in size_share.values()]
        minor = all(x < MINOR_SHARE for x in shares_all)
        crowding = bool(nat and st and st["b"] >= nat["b"] + CROWDING_MARGIN)
        slopes[n] = {"natural": nat, "stress": st, "stress_raw": rw, "map": mp, "minor": minor,
                     "natural_class": None if minor else classify(nat and nat["b"]),
                     "stress_class": None if minor else classify(st and st["b"]),
                     "map_class": None if minor else classify(mp and mp["b"]),
                     "crowding_driven": crowding, "share_500": size_share.get(TARGET, {}).get(n),
                     "max_share": max(shares_all)}
    out["slopes"] = slopes
    out["tick_slopes"] = {"natural": fit(nat_tick), "stress": fit(st_tick), "stress_raw": fit(raw_tick), "map": fit(map_tick)}

    # Per-size tick summaries.
    def size_rows(sizes):
        rows = {}
        for size, runs in sizes.items():
            means = [r["tick_summary"]["mean_ms"] for r in runs]
            pooled = pooled_steady(runs)
            rows[str(size)] = {
                "seeds": len(runs), "median_mean_ms": statistics.median(means), "max_mean_ms": max(means),
                "mean_alive": statistics.mean(r["mean_alive_0600"] for r in runs),
                "pooled_p999_ms": percentile(pooled, 99.9) / 1000.0 if pooled else None,
                "pooled_ticks": len(pooled) if pooled else None,
                "max_p999_ms": max(r["tick_summary"]["p999_ms"] for r in runs),
                "max_ms": max(r["tick_summary"]["max_ms"] for r in runs),
                "n_over_16_67ms": sum(r["tick_summary"]["n_over_16_67ms"] for r in runs),
                "n_over_8ms": sum(r["tick_summary"]["n_over_8ms"] for r in runs),
                "us_per_agent_tick": statistics.median(r["tick_summary"]["mean_ms"] * 1000.0 / r["mean_alive_0600"] for r in runs if r["mean_alive_0600"]),
                "per_world": [{"seed": r["seed"], "mean_ms": r["tick_summary"]["mean_ms"], "p999_ms": r["tick_summary"]["p999_ms"],
                               "max_ms": r["tick_summary"]["max_ms"], "n_over_16_67ms": r["tick_summary"]["n_over_16_67ms"],
                               "mean_alive": r["mean_alive_0600"], "contended": r["contended"]} for r in sorted(runs, key=lambda r: r["seed"])],
            }
        return rows
    out["fed_sizes"] = size_rows(sizes_fed)
    out["raw_sizes"] = size_rows(sizes_raw)
    map_sides = {}
    for run in map_runs.values():
        map_sides.setdefault(run["side"], []).append(run)
    out["map_sides"] = {str(side): {"seeds": len(rs), "median_mean_ms": statistics.median(r["tick_summary"]["mean_ms"] for r in rs),
                                    "max_ms": max(r["tick_summary"]["max_ms"] for r in rs),
                                    "mean_alive": statistics.mean(r["mean_alive_0600"] for r in rs)} for side, rs in sorted(map_sides.items())}

    # Natural sweep.
    nat_rows = []
    for r in sorted(natural.values(), key=lambda r: r["seed"]):
        t = r["tick_summary"]
        nat_rows.append({"seed": r["seed"], "mean_ms": t["mean_ms"], "p999_ms": t["p999_ms"], "max_ms": t["max_ms"],
                         "n_over_8ms": t["n_over_8ms"], "n_over_16_67ms": t["n_over_16_67ms"],
                         "mean_alive": r["mean_alive_0600"], "mean_settlements": r["mean_settlements_0600"],
                         "b1": t["mean_ms"] <= B1_MS, "b2": t["p999_ms"] <= B2_MS, "contended": r["contended"]})
    pooled_nat = pooled_steady(list(natural.values()))
    out["natural"] = {"worlds": nat_rows,
                      "pooled": {"ticks": len(pooled_nat), "mean_ms": sum(pooled_nat) / len(pooled_nat) / 1000.0,
                                 "p999_ms": percentile(pooled_nat, 99.9) / 1000.0, "max_ms": pooled_nat[-1] / 1000.0} if pooled_nat else None,
                      "max_mean_ms": max((w["mean_ms"] for w in nat_rows), default=None),
                      "max_p999_ms": max((w["p999_ms"] for w in nat_rows), default=None),
                      "max_ms": max((w["max_ms"] for w in nat_rows), default=None)}

    # N_max.
    passing = [int(size) for size, row in out["fed_sizes"].items() if row["median_mean_ms"] <= B1_MS]
    all_sizes = [int(size) for size in out["fed_sizes"]]
    largest_pass = max(passing) if passing else None
    every = bool(all_sizes) and passing == sorted(all_sizes) and all(
        out["fed_sizes"][str(s)]["median_mean_ms"] <= B1_MS for s in all_sizes)
    tick_fit = out["tick_slopes"]["stress"]
    interpolated = math.exp((math.log(B1_MS) - tick_fit["a"]) / tick_fit["b"]) if tick_fit and tick_fit["b"] > 0 else None
    out["n_max"] = {"largest_passing_size": largest_pass, "every_size_passes": every, "largest_size_run": max(all_sizes) if all_sizes else None,
                    "interpolated_alive": interpolated, "interpolated_is_extrapolation": bool(interpolated and all_sizes and interpolated > max(all_sizes)),
                    "half_for_slower_machines": {"largest_passing_size": largest_pass / 2 if largest_pass else None,
                                                 "interpolated_alive": interpolated / 2 if interpolated else None}}

    # Snapshot costs at n = 500 (fed).
    at500 = sizes_fed.get(TARGET, [])
    snap = {}
    for label, table in (("wip", PER_FRAME_WIP), ("art", PER_FRAME_ART)):
        single, total, rows, tiles, hearths = callers_cost(at500, table)
        snap[label] = {"single_sum_us": single, "callers_sum_us": total, "agent_tiles": tiles, "hearths": hearths,
                       "rows": [{"getter": g, "calls_per_frame": c, "single_us": o, "cost_us": t} for g, c, o, t in rows]}
    snap["all_getters_median_us"] = {g: median_snapshot(at500, g) for g in (at500[0]["snapshot_timings"] if at500 and at500[0].get("snapshot_timings") else {}) if g != "context"}
    snap["untimed_art"] = UNTIMED_ART
    out["snapshots_500"] = snap

    # P table.
    p1 = bool(nat_rows) and all(w["b1"] and w["b2"] for w in nat_rows)
    row500 = out["fed_sizes"].get(str(TARGET))
    p2 = bool(row500) and row500["median_mean_ms"] <= B1_MS and (row500["pooled_p999_ms"] is not None and row500["pooled_p999_ms"] <= B2_MS)
    p3 = snap["wip"]["single_sum_us"] / 1000.0 <= B3_MS and snap["art"]["single_sum_us"] / 1000.0 <= B3_MS
    nmax_value = largest_pass if not every else max(all_sizes)
    p4_comfortable = bool(nmax_value and nmax_value >= 2 * TARGET)
    bad = [n for n, s in slopes.items() if not s["minor"] and s["natural"] and s["natural"]["b"] >= 1.2 and (s["share_500"] or 0.0) >= MAJOR_SHARE]
    out["p_table"] = {
        "P1": {"pass": p1, "worlds": len(nat_rows), "failing": [w["seed"] for w in nat_rows if not (w["b1"] and w["b2"])]},
        "P2": {"pass": p2, "median_mean_ms": row500 and row500["median_mean_ms"], "pooled_p999_ms": row500 and row500["pooled_p999_ms"], "seeds": row500 and row500["seeds"]},
        "P3": {"pass": p3, "wip_single_ms": snap["wip"]["single_sum_us"] / 1000.0, "art_single_ms": snap["art"]["single_sum_us"] / 1000.0,
               "wip_callers_ms": snap["wip"]["callers_sum_us"] / 1000.0, "art_callers_ms": snap["art"]["callers_sum_us"] / 1000.0},
        "P4": {"comfortable": p4_comfortable, "largest_passing_size": largest_pass, "every_size_passes": every, "interpolated_alive": interpolated},
        "P5": {"clean": not bad, "natural_superlinear_major": bad,
               "superlinear_or_worse": {n: {"natural": s["natural_class"], "stress": s["stress_class"], "crowding_driven": s["crowding_driven"]}
                                        for n, s in slopes.items() if not s["minor"] and ((s["natural"] and s["natural"]["b"] >= 1.2) or (s["stress"] and s["stress"]["b"] >= 1.2))}},
    }
    # Optional-fix trigger (Part 5).
    trig = [n for n, s in slopes.items() if (s["share_500"] or 0.0) >= OPTIONAL_FIX_SHARE and (
        (s["natural"] and s["natural"]["b"] >= 1.7) or (s["stress"] and s["stress"]["b"] >= 1.7 and not s["crowding_driven"]))]
    out["part5_trigger"] = {"p2_fails": not p2, "near_quadratic_major": trig, "fix_allowed": (not p2) or bool(trig)}

    if overhead:
        out["overhead"] = {k: overhead[k] for k in ("median_plain_seconds", "median_profiled_seconds", "overhead_pct", "same_fingerprint", "plain_wall_seconds", "profiled_wall_seconds")}
    if repeat:
        means = [r["tick_summary"]["mean_ms"] for r in repeat.values()]
        out["repeatability"] = {"runs": len(means), "means_ms": means,
                                "cv": statistics.stdev(means) / statistics.mean(means) if len(means) > 1 else None}
    # Top 10 by share at the sizes the budget names.
    out["top10"] = {str(size): sorted(((n, sh) for n, sh in size_share[size].items()), key=lambda x: -x[1])[:10] for size in size_share}
    out["top10"]["natural"] = sorted(nat_share.items(), key=lambda x: -x[1])[:10]
    return out


# ---------------------------------------------------------------- report

def fmt_ms(value):
    return "—" if value is None else f"{value:.3f}"


def fmt_fit(f):
    if not f:
        return "—"
    return f"{f['b']:.2f} [{f['ci'][0]:.2f}, {f['ci'][1]:.2f}] (n {f['n']})"


def render(out):
    lines = ["# Pass 15 scaling tables (`perf_p15.py`)", ""]
    hdr = ""
    if out.get("overhead") and out["overhead"]["overhead_pct"] > 5.0:
        hdr = f" — profiling overhead {out['overhead']['overhead_pct']:.1f} % (> 5 %)"
    lines += [f"Runs: natural {out['n_runs']['natural']}, stress {out['n_runs']['stress']}, map {out['n_runs']['map']}. "
              "All times are one Mac's (MacBook Pro, Apple M4 Max), release build, single-threaded simulation; "
              "steady state = every tick after the first 60.", ""]
    p = out["p_table"]
    lines += ["## P table" + hdr, "", "| # | rule | measured | verdict |", "|---|---|---|---|"]
    lines.append(f"| P1 | every natural world meets B1 and B2 | {p['P1']['worlds']} worlds; worst mean {out['natural']['max_mean_ms']:.4f} ms, worst p99.9 {out['natural']['max_p999_ms']:.3f} ms, max {out['natural']['max_ms']:.3f} ms | {'PASS' if p['P1']['pass'] else 'FAIL'} |")
    p2 = p["P2"]
    lines.append(f"| P2 | n = 500 fed: median world mean ≤ 4.0 ms, pooled p99.9 ≤ 8.0 ms | {p2['seeds']} worlds; median mean {p2['median_mean_ms']:.4f} ms; pooled p99.9 {fmt_ms(p2['pooled_p999_ms'])} ms | {'PASS' if p2['pass'] else 'FAIL'} |" if p2["seeds"] else "| P2 | — | no n = 500 runs | FAIL |")
    p3 = p["P3"]
    lines.append(f"| P3 | one call of every per-frame getter, summed ≤ 2.0 ms | WIP {p3['wip_single_ms']:.4f} ms, art {p3['art_single_ms']:.4f} ms (×callers: WIP {p3['wip_callers_ms']:.4f} ms, art {p3['art_callers_ms']:.4f} ms) | {'PASS' if p3['pass'] else 'FAIL'} |")
    p4 = p["P4"]
    nm = out["n_max"]
    nm_text = (f"≥ {nm['largest_size_run']:,} (every size passes)" if nm["every_size_passes"] else f"{nm['largest_passing_size']}") + \
              (f"; interpolated {nm['interpolated_alive']:,.0f} alive" + (" (extrapolated beyond the largest size)" if nm["interpolated_is_extrapolation"] else "") if nm["interpolated_alive"] else "")
    lines.append(f"| P4 | N_max reported; comfortable if ≥ 1,000 | {nm_text} | {'comfortable' if p4['comfortable'] else 'not comfortable'} |")
    p5 = p["P5"]
    lines.append(f"| P5 | clean if no system with natural slope ≥ 1.2 has ≥ 5 % of tick time at n = 500 | {', '.join(p5['natural_superlinear_major']) or 'none'} | {'clean' if p5['clean'] else 'not clean'} |")
    lines += ["", "## Tick time per sweep", "", "| sweep | size | worlds | mean alive | median mean ms | pooled p99.9 ms | worst p99.9 ms | max ms | ticks > 8 ms | ticks > 16.67 ms | µs per agent-tick |", "|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|"]
    nat = out["natural"]
    if nat["pooled"]:
        nm_ = statistics.median(w["mean_ms"] for w in nat["worlds"])
        lines.append(f"| natural | game_v19 | {len(nat['worlds'])} | {statistics.mean(w['mean_alive'] for w in nat['worlds']):.1f} | {nm_:.4f} | {nat['pooled']['p999_ms']:.3f} | {nat['max_p999_ms']:.3f} | {nat['max_ms']:.3f} | {sum(w['n_over_8ms'] for w in nat['worlds'])} | {sum(w['n_over_16_67ms'] for w in nat['worlds'])} | {statistics.median(w['mean_ms'] * 1000 / w['mean_alive'] for w in nat['worlds'] if w['mean_alive']):.3f} |")
    for label, rows in (("stress fed", out["fed_sizes"]), ("stress raw", out["raw_sizes"])):
        for size, r in rows.items():
            pooled = f"{r['pooled_p999_ms']:.3f}" if r["pooled_p999_ms"] is not None else "—"
            lines.append(f"| {label} | {int(size):,} | {r['seeds']} | {r['mean_alive']:.1f} | {r['median_mean_ms']:.4f} | {pooled} | {r['max_p999_ms']:.3f} | {r['max_ms']:.3f} | {r['n_over_8ms']} | {r['n_over_16_67ms']} | {r['us_per_agent_tick']:.3f} |")
    for side, r in out["map_sides"].items():
        lines.append(f"| map | side {side} | {r['seeds']} | {r['mean_alive']:.1f} | {r['median_mean_ms']:.4f} | — | — | {r['max_ms']:.3f} | — | — | — |")
    lines += ["", "## Whole-tick slopes (log mean tick on log alive / log tiles)", "",
              f"- natural (ms per tick of a full day on alive at 06:00, days with alive ≥ 15): {fmt_fit(out['tick_slopes']['natural'])}",
              f"- stress fed (mean tick on mean alive, runs with mean alive ≥ 80): {fmt_fit(out['tick_slopes']['stress'])}",
              f"- stress raw: {fmt_fit(out['tick_slopes']['stress_raw'])}",
              f"- map (mean tick on tiles): {fmt_fit(out['tick_slopes']['map'])}", ""]
    lines += ["## Per-system slopes", "",
              "Natural slope: log(that system's ms per day) on log(alive at 06:00), full days with alive ≥ 15, all natural worlds pooled — it pools days, so it also absorbs time trends (more buildings and settlements, winter firewood). "
              "Stress slope: log(mean µs per call) on log(mean alive), fed runs with mean alive ≥ 80 — a density-confounded upper bound (every founder starts on the camp ring). "
              "Map slope: log(mean µs per call) on log(tiles). Intervals are 95 % t-based. *Crowding-driven*: stress slope ≥ natural slope + 0.5. Systems below 1 % of tick time at every size are listed but not classified.", "",
              "| system | interval | share natural | share n=500 | natural slope | class | stress slope (fed) | class | crowding | stress slope (raw) | map slope | class |", "|---|---:|---:|---:|---|---|---|---|---|---|---|---|"]
    intervals = out.get("_intervals", {})
    for name, s in sorted(out["slopes"].items(), key=lambda kv: -(kv[1]["share_500"] or 0.0)):
        share500 = f"{s['share_500'] * 100:.2f} %" if s["share_500"] is not None else "—"
        lines.append(f"| {name} | {intervals.get(name, '')} | {out['shares']['natural_pooled'].get(name, 0) * 100:.2f} % | {share500} | {fmt_fit(s['natural'])} | {s['natural_class'] or 'minor'} | {fmt_fit(s['stress'])} | {s['stress_class'] or 'minor'} | {'yes' if s['crowding_driven'] else ''} | {fmt_fit(s['stress_raw'])} | {fmt_fit(s['map'])} | {s['map_class'] or 'minor'} |")
    lines += ["", "## Top 10 systems by share", ""]
    for key, rows in out["top10"].items():
        label = "natural (pooled)" if key == "natural" else f"fed n = {int(key):,} (median share over seeds)"
        lines.append(f"- **{label}:** " + ", ".join(f"{n} {sh * 100:.1f} %" for n, sh in rows))
    nm = out["n_max"]
    lines += ["", "## N_max", "",
              f"- largest size whose median fed mean tick ≤ 4.0 ms: {'≥ ' + format(nm['largest_size_run'], ',') + ' (every size passes)' if nm['every_size_passes'] else nm['largest_passing_size']}",
              f"- interpolated on the fitted log-log line (stress fed, whole tick): {nm['interpolated_alive']:,.0f} alive" + (" — an extrapolation beyond the largest size run" if nm["interpolated_is_extrapolation"] else "") if nm["interpolated_alive"] else "- interpolated: —",
              f"- factor-2 margin for slower machines: {nm['half_for_slower_machines']}", ""]
    snap = out["snapshots_500"]
    lines += ["## Snapshot collectors at n = 500 (fed; Rust side only, median of the per-run medians of 100 calls)", "",
              f"Distinct agent tiles T = {snap['wip']['agent_tiles']}, hearths H = {snap['wip']['hearths']} (medians over the n = 500 runs).", "",
              "| getter | one call µs | WIP calls/frame | WIP µs/frame | art calls/frame | art µs/frame |", "|---|---:|---:|---:|---:|---:|"]
    art_rows = {r["getter"]: r for r in snap["art"]["rows"]}
    wip_rows = {r["getter"]: r for r in snap["wip"]["rows"]}
    for getter, one in sorted(snap["all_getters_median_us"].items()):
        w, a = wip_rows.get(getter), art_rows.get(getter)
        cell = lambda r: (f"{r['calls_per_frame']}", f"{r['cost_us']:.2f}") if r and r["cost_us"] is not None else ("—", "—")
        lines.append(f"| {getter} | {one:.3f} | {cell(w)[0]} | {cell(w)[1]} | {cell(a)[0]} | {cell(a)[1]} |")
    lines += ["", f"- B3 single calls summed: WIP {snap['wip']['single_sum_us']:.2f} µs, art {snap['art']['single_sum_us']:.2f} µs (limit 2,000 µs).",
              f"- ×callers per frame: WIP {snap['wip']['callers_sum_us']:.2f} µs, art {snap['art']['callers_sum_us']:.2f} µs.",
              f"- Not timeable on this branch (art-only getters): {', '.join(snap['untimed_art'])}.", ""]
    if out.get("overhead"):
        o = out["overhead"]
        lines += ["## Profiling overhead", "", f"- natural seed 15000: median plain {o['median_plain_seconds']:.3f} s, profiled {o['median_profiled_seconds']:.3f} s, overhead {o['overhead_pct']:.2f} %; same fingerprint {o['same_fingerprint']}.", ""]
    if out.get("repeatability"):
        r = out["repeatability"]
        lines += ["## Repeatability (seed 15100, n = 500, fed)", "", f"- mean tick {', '.join(f'{m:.4f}' for m in r['means_ms'])} ms; coefficient of variation {r['cv']:.4f}" if r["cv"] is not None else "- —", ""]
    tr = out["part5_trigger"]
    lines += ["## Part 5 trigger", "", f"- P2 fails: {tr['p2_fails']}; near-quadratic system with ≥ 10 % at n = 500: {tr['near_quadratic_major'] or 'none'} → fix allowed: {tr['fix_allowed']}", ""]
    return "\n".join(lines) + "\n"


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--natural", required=True)
    parser.add_argument("--stress", required=True)
    parser.add_argument("--map", default=None)
    parser.add_argument("--overhead", default=None)
    parser.add_argument("--repeat", default=None)
    parser.add_argument("--out", required=True)
    args = parser.parse_args(argv)
    natural, _ = load_runs(args.natural)
    stress, failures = load_runs(args.stress)
    map_runs, map_failures = load_runs(args.map)
    overhead = None
    if args.overhead:
        with open(args.overhead) as stream:
            overhead = json.load(stream)
    repeat = load_runs(args.repeat)[0] if args.repeat else None
    out = analyse(natural, stress, map_runs, overhead, repeat)
    any_run = next(iter((natural or stress).values()))
    out["_intervals"] = {s["name"]: s["interval"] for s in any_run["systems"]}
    out["failures"] = {"stress": failures, "map": map_failures}
    os.makedirs(args.out, exist_ok=True)
    with open(os.path.join(args.out, "scaling.json"), "w") as stream:
        json.dump(out, stream, indent=1, sort_keys=True)
        stream.write("\n")
    text = render(out)
    if failures or map_failures:
        text += "## Runs that did not complete\n\n" + "\n".join(f"- {f.get('stem')}: {f.get('status')} {f.get('error', '')}".rstrip() for f in failures + map_failures) + "\n\n"
    for extra in ("inventory.md", "audit.md"):
        path = os.path.join(args.out, extra)
        if os.path.exists(path):
            with open(path) as stream:
                text += "\n" + stream.read()
    with open(os.path.join(args.out, "scaling.md"), "w") as stream:
        stream.write(text)
    print(text[:3000])
    return 0


if __name__ == "__main__":
    sys.exit(main())
