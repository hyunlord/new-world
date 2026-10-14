#!/usr/bin/env python3
"""Pass 14 Parts 1 and 3: score a dose-timing map of the god's acts on `game_v19` (PASS14_GOAL.md 6.3, 8, 10). numpy only.

usage: dose_map_p14.py --root DIR --scene game_v19 --seeds a..b --replicates 2 --fork-tick 14280
                       --manifest SCHED_DIR/manifest.json [--manifest ...] [--arms ARM,...] [--prereg ARM,...]
                       [--probe-summary SUMMARY.json] [--unforked DIR] --label "Stage A" --out DIR

DIR holds schema-6 recordings `<scene>_s<seed>_<id>_r<k>` (no --agents) of the factual (`none`) and every arm, forked
at the same tick with replicates 0..R-1; the manifests (`schedules_p14.py`) give each (seed, arm)'s file and cast tick
(the first row's tick). Per episode: every departure = the `depart_tick` of every founded daughter (settlements.npy;
cancelled departures and daughters not founded by the run's end are not seen), the settlements alive (>= 1 member)
at the 08:00 rows of days 60 / 90 / 118, alive at day 118 and the cumulative deaths by cause.

Pairs: for an arm, (arm r_k, factual r_k) of the same seed; for noise, (factual r1, factual r0) of the arm's seeds,
measured after that arm's cast tick (one pair per seed). With cast tick c, a run's departure day = the display day of
its first departure at a tick >= c (the cast is applied before tick c's systems run, so a departure at c is after it).
- direction: both depart -> sign of Δday = arm day - factual day; only the arm departs -> earlier; only the factual
  -> later; neither -> tie. right-direction share = share of all pairs in the arm's direction (earlier for
  spoil_food / rot_wood, later for gift_food; none for rain / cold_snap); ties and wrong signs count against.
- mean |Δday|, share |Δday| > 3 and mean signed Δday over pairs where both depart after c; ENR = mean |Δday| (arm) /
  max(0.5, mean |Δday| (noise)), NA when the noise has fewer than 8 such pairs.
- settlement count at days 60 / 90 / 118: share of pairs that differ; alive at day 118: mean signed and absolute Δ;
  deaths by cause, to the end of the run (day 120) and within 35 days of c (rows c//60 .. (c + 35 d)//60, capped
  at the last row): mean signed Δ (arm - factual) and mean |Δ|; for noise the same with r1 - r0.
- intervals: bootstrap over the arm's seeds (1,000 resamples, numpy default_rng(0), one index matrix per arm shared
  by every statistic, 90 % percentile interval; NaN resamples dropped).
Every arm episode's `interventions.json` must hold one record per schedule row (tick, kind, amount); factual episodes
none; the fork tick and replicate of every episode must match. Applied amounts are reported.
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

import numpy as np

REPO = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO / "tools/worldmodel"))
from wm_data import load_episode  # noqa: E402

TPD, OFFSET, EVERY = 1440, 480, 60
DAYS = (60, 90, 118)
CAUSES = ("starvation", "dehydration", "combat", "old_age", "exposure")
WINDOW = 35 * TPD
RIGHT = {"spoil_food": "earlier", "rot_wood": "earlier", "gift_food": "later"}
B = 1000


def display_day(tick: int) -> int:
    return (tick + OFFSET) // TPD + 1


def row_of(day: int, minute: int = 480) -> int:
    return ((day - 1) * TPD + minute - OFFSET) // EVERY


# ---------------------------------------------------------------- episode facts (I/O)

def episode_facts(path: Path) -> dict:
    ep = load_episode(path, arrays=["global.npy", "settlements.npy"])
    g = ep.arrays["global.npy"]
    s = ep.arrays["settlements.npy"]
    departs: list[int] = []
    counts = {d: 0 for d in DAYS}
    if s.shape[1]:
        dep = ep.column("settlements.npy", "depart_tick")
        departs = sorted({int(v) for v in dep[np.isfinite(dep)] if v >= 0})
        members = np.nan_to_num(ep.column("settlements.npy", "members"), nan=0.0)
        counts = {d: int(np.sum(members[row_of(d)] >= 1)) for d in DAYS}
    deaths = np.stack([ep.column("global.npy", f"deaths_{c}") for c in CAUSES], axis=1).astype(np.float64)
    return {"departs": departs, "settle": counts, "alive118": float(ep.column("global.npy", "alive")[row_of(118)]),
            "deaths": deaths, "interventions": ep.interventions, "schedule": ep.meta.get("schedule"),
            "fork": ep.meta.get("noise_fork"), "rows": g.shape[0]}


# ---------------------------------------------------------------- pure pair / arm statistics

def first_after(departs: list[int], cast: int) -> int | None:
    for t in departs:
        if t >= cast:
            return display_day(t)
    return None


def pair(a: dict, f: dict, cast: int) -> dict:
    """Arm (or replicate 1) `a` against factual (or replicate 0) `f` after cast tick `cast`."""
    da, df = first_after(a["departs"], cast), first_after(f["departs"], cast)
    if da is not None and df is not None:
        delta = da - df
        direction = "earlier" if delta < 0 else "later" if delta > 0 else "tie"
    else:
        delta = None
        direction = "earlier" if da is not None else "later" if df is not None else "tie"
    last = min(a["deaths"].shape[0], f["deaths"].shape[0]) - 1
    r35 = min((cast + WINDOW) // EVERY, last)
    r0 = min(cast // EVERY, last)
    w_a = a["deaths"][r35] - a["deaths"][r0]
    w_f = f["deaths"][r35] - f["deaths"][r0]
    return {"day_a": da, "day_f": df, "delta": delta, "direction": direction,
            "settle_diff": {d: a["settle"][d] != f["settle"][d] for d in DAYS},
            "alive_signed": a["alive118"] - f["alive118"],
            "deaths35": (w_a - w_f).tolist(), "deaths120": (a["deaths"][last] - f["deaths"][last]).tolist()}


def _mean(xs) -> float:
    return float(np.mean(xs)) if len(xs) else float("nan")


def stats(pairs: list[dict], noise: list[dict], kind: str | None) -> dict:
    """Scalar statistics of one arm over `pairs` (all replicates of the chosen seeds) and `noise` (one per seed)."""
    out: dict[str, float] = {}
    n = len(pairs)
    for d in ("earlier", "later", "tie"):
        out[f"share_{d}"] = _mean([p["direction"] == d for p in pairs])
    right = RIGHT.get(kind or "")
    out["right_share"] = _mean([p["direction"] == right for p in pairs]) if right else float("nan")
    both = [p["delta"] for p in pairs if p["delta"] is not None]
    nb = [abs(p["delta"]) for p in noise if p["delta"] is not None]
    out.update(n_pairs=n, n_both=len(both), mean_abs=_mean(np.abs(both)), share_gt3=_mean(np.abs(both) > 3),
               mean_signed=_mean(both), noise_n_both=len(nb), noise_mean_abs=_mean(nb), noise_share_gt3=_mean(np.array(nb) > 3))
    out["enr"] = (out["mean_abs"] / max(0.5, out["noise_mean_abs"])) if len(nb) >= 8 and both else float("nan")
    for d in DAYS:
        out[f"settle_diff_{d}"] = _mean([p["settle_diff"][d] for p in pairs])
        out[f"noise_settle_diff_{d}"] = _mean([p["settle_diff"][d] for p in noise])
    out["alive_signed"] = _mean([p["alive_signed"] for p in pairs])
    out["alive_abs"] = _mean([abs(p["alive_signed"]) for p in pairs])
    out["noise_alive_abs"] = _mean([abs(p["alive_signed"]) for p in noise])
    for w in ("deaths35", "deaths120"):
        arm = np.array([p[w] for p in pairs], dtype=float).reshape(-1, len(CAUSES))
        nz = np.array([p[w] for p in noise], dtype=float).reshape(-1, len(CAUSES))
        for i, c in enumerate(CAUSES):
            out[f"{w}_{c}"] = _mean(arm[:, i])
        out[f"{w}_total"] = _mean(arm.sum(axis=1))
        out[f"{w}_total_abs"] = _mean(np.abs(arm.sum(axis=1)))
        out[f"noise_{w}_total"] = _mean(nz.sum(axis=1))
        out[f"noise_{w}_total_abs"] = _mean(np.abs(nz.sum(axis=1)))
    return out


def resample_index(n_seeds: int, b: int = B) -> np.ndarray:
    return np.random.default_rng(0).integers(0, n_seeds, size=(b, n_seeds))


def summarize(seeds: list[int], pairs_by_seed: dict, noise_by_seed: dict, kind: str | None, b: int = B) -> dict:
    """Point statistics and 90 % bootstrap intervals over seeds."""
    point = stats([p for s in seeds for p in pairs_by_seed[s]], [noise_by_seed[s] for s in seeds], kind)
    boot: dict[str, list[float]] = {k: [] for k in point}
    for row in resample_index(len(seeds), b):
        chosen = [seeds[i] for i in row]
        st = stats([p for s in chosen for p in pairs_by_seed[s]], [noise_by_seed[s] for s in chosen], kind)
        for k, v in st.items():
            boot[k].append(v)
    ci = {}
    for k, vals in boot.items():
        arr = np.array(vals, dtype=float)
        arr = arr[np.isfinite(arr)]
        ci[k] = [float(np.percentile(arr, 5)), float(np.percentile(arr, 95))] if len(arr) else None
    clean = {k: (None if isinstance(v, float) and not np.isfinite(v) else v) for k, v in point.items()}
    return {"point": clean, "ci90": ci, "seeds": seeds}


def paired_right_diff(seeds: list[int], a: dict, e: dict, kind: str, b: int = B) -> dict:
    """K3: right-direction share of arm `a` minus arm `e` on the same seeds, with a seed bootstrap."""
    right = RIGHT[kind]

    def diff(chosen):
        pa = [p["direction"] == right for s in chosen for p in a[s]]
        pe = [p["direction"] == right for s in chosen for p in e[s]]
        return _mean(pa) - _mean(pe), _mean(pa), _mean(pe)

    d, ra, re = diff(seeds)
    boots = np.array([diff([seeds[i] for i in row])[0] for row in resample_index(len(seeds), b)])
    return {"n_seeds": len(seeds), "share_low": ra, "share_eve": re, "diff": d,
            "ci90": [float(np.percentile(boots, 5)), float(np.percentile(boots, 95))], "passed": bool(d > 0)}


# ---------------------------------------------------------------- loading and checks

def load_manifests(paths: list[str]) -> dict:
    """(seed, arm) -> {cast_tick, kind, amount, doses} from schedule manifests."""
    out = {}
    for p in paths:
        m = json.loads(Path(p).read_text())
        for f in m["files"]:
            kind, amount, when, doses = m["arms"][f["arm"]]
            out[(f["seed"], f["arm"])] = {"cast_tick": f["cast_tick"], "kind": kind, "amount": amount, "when": when,
                                          "doses": doses, "sha256": f["sha256"]}
    return out


def check_episode(facts: dict, fork_tick: int, k: int, sched: dict | None) -> list[str]:
    errors = []
    fork = facts["fork"] or {}
    if fork.get("tick") != fork_tick or fork.get("replicate") != k:
        errors.append(f"noise_fork {fork} != tick {fork_tick} replicate {k}")
    recs = facts["interventions"]
    if sched is None:
        if recs or facts["schedule"] is not None:
            errors.append("factual episode has interventions or a schedule")
        return errors
    rows = facts["schedule"] or []
    if len(rows) != len(recs):
        errors.append(f"{len(rows)} schedule rows but {len(recs)} intervention records")
    for r, rec in zip(rows, recs):
        if (r["tick"], r["kind"], r["amount"]) != (rec["tick"], rec["kind"], rec["amount"]):
            errors.append(f"row {r} != record {rec}")
        if not fork_tick <= r["tick"] < (facts["rows"] - 1) * EVERY:
            errors.append(f"row {r} outside [fork, end)")
    if rows and rows[0]["tick"] != sched["cast_tick"]:
        errors.append(f"first row tick {rows[0]['tick']} != manifest cast tick {sched['cast_tick']}")
    return errors


def compare_unforked(unforked: Path, root: Path, scene: str, seeds: list[int]) -> dict:
    """Part 0 item 2: arrays of `<scene>_s<seed>_none` (unforked) against `..._none_r0`."""
    out = {}
    for s in seeds:
        a = json.loads((unforked / f"{scene}_s{s}_none" / "meta.json").read_text())
        b = json.loads((root / f"{scene}_s{s}_none_r0" / "meta.json").read_text())
        same = {n: a["arrays"][n]["sha256"] == b["arrays"].get(n, {}).get("sha256") for n in a["arrays"]}
        other = sorted(k for k in set(a) | set(b) if k not in ("arrays", "noise_fork") and a.get(k) != b.get(k))
        ia = (unforked / f"{scene}_s{s}_none" / "interventions.json").read_text()
        ib = (root / f"{scene}_s{s}_none_r0" / "interventions.json").read_text()
        out[str(s)] = {"arrays_equal": all(same.values()), "n_arrays": len(same), "meta_keys_differing": other,
                       "interventions_equal": ia == ib, "noise_fork": [a.get("noise_fork"), b.get("noise_fork")]}
    return out


def probe_crosscheck(summary_path: Path, facts_r0: dict) -> dict:
    """Founded-daughter departure ticks of the factual r0 against the probe's `pass11.fission_log`."""
    runs = {r["seed"]: r for r in json.loads(summary_path.read_text())["runs"]}
    res = {"seeds": 0, "equal": 0, "differ": [], "probe_unfounded": 0, "probe_cancelled": 0}
    for s, f in facts_r0.items():
        if s not in runs:
            continue
        log = runs[s]["pass11"]["fission_log"]
        founded = sorted(x["depart_tick"] for x in log if x.get("daughter") is not None)
        res["probe_unfounded"] += sum(1 for x in log if x.get("daughter") is None)
        res["probe_cancelled"] += sum(1 for x in log if x.get("cancelled_tick") is not None)
        res["seeds"] += 1
        if founded == f["departs"]:
            res["equal"] += 1
        else:
            res["differ"].append([s, founded, f["departs"]])
    return res


# ---------------------------------------------------------------- report

def fmt(x, nd=3):
    if x is None:
        return "NA"
    if isinstance(x, (int, np.integer)):
        return str(int(x))
    return f"{x:.{nd}f}"


def ci_s(arm: dict, key: str, nd=3) -> str:
    v, c = arm["point"].get(key), arm["ci90"].get(key)
    return f"{fmt(v, nd)} [{fmt(c[0], nd)}, {fmt(c[1], nd)}]" if c and v is not None else fmt(v, nd)


def markdown(r: dict) -> str:
    L = [f"# Pass 14 {r['label']} — dose–timing map (`{r['scene']}`, seeds {r['seeds'][0]}..{r['seeds'][1]}, R = "
         f"{r['replicates']}, fork tick {r['fork_tick']})", "",
         "Scored by `tools/revitalize/dose_map_p14.py` (rules in its module docs). Departure = the first departure of any "
         "group at a tick ≥ the cast tick (schema-6 `depart_tick` of founded daughters); Δday = arm − factual (same seed and "
         "replicate); noise = factual r1 − r0 on the arm's seeds after the arm's cast tick. Intervals: 90 % seed bootstrap "
         "(1,000×, `default_rng(0)`). ENR = mean |Δday| / max(0.5, noise mean |Δday|), NA if the noise has < 8 pairs where "
         "both depart.", "",
         f"Episode checks: {r['checks']['episodes']} episodes read, {len(r['checks']['errors'])} errors."]
    if r.get("probe_crosscheck"):
        pc = r["probe_crosscheck"]
        L.append(f"Factual r0 vs probe fission log (founded daughters' departure ticks): {pc['equal']} / {pc['seeds']} seeds "
                 f"equal; probe departures without a founded daughter (not seen by the recorder) {pc['probe_unfounded']}, "
                 f"cancelled {pc['probe_cancelled']}.")
    if r.get("unforked"):
        u = r["unforked"]
        L.append("Replicate 0 vs unforked factual: " + "; ".join(
            f"seed {s}: arrays {'equal' if v['arrays_equal'] else 'DIFFER'} ({v['n_arrays']}), interventions "
            f"{'equal' if v['interventions_equal'] else 'DIFFER'}, other meta keys differing {v['meta_keys_differing']}"
            for s, v in u.items()))
    L += ["", "## Direction and size of the shift of the first departure after the cast", "",
          "| arm | kind | amount × doses | time | seeds | pairs | right-direction share [90 %] | earlier / later / tie | n both | "
          "mean \\|Δday\\| [90 %] | share \\|Δ\\| > 3 | mean signed Δday | noise n both / mean \\|Δ\\| / share > 3 | ENR [90 %] |",
          "|---|---|---|---|---:|---:|---|---|---:|---|---:|---:|---|---|"]
    for name, a in r["arms"].items():
        p = a["point"]
        L.append(f"| `{name}` | {a['kind']} | {a['amount']} × {a['doses']} | {a['when']} | {len(a['seeds'])} | {p['n_pairs']} | "
                 f"{ci_s(a, 'right_share') if RIGHT.get(a['kind']) else '— (no direction)'} | {fmt(p['share_earlier'])} / "
                 f"{fmt(p['share_later'])} / {fmt(p['share_tie'])} | {p['n_both']} | {ci_s(a, 'mean_abs', 2)} | {fmt(p['share_gt3'])} | "
                 f"{fmt(p['mean_signed'], 2)} | {p['noise_n_both']} / {fmt(p['noise_mean_abs'], 2)} / {fmt(p['noise_share_gt3'])} | "
                 f"{ci_s(a, 'enr', 2)} |")
    L += ["", "## Shape of the world: settlements and people", "",
          "| arm | settlement count differs d60 / d90 / d118 | noise d60 / d90 / d118 | d118 differs − noise | alive d118 Δ signed [90 %] | "
          "alive d118 \\|Δ\\| | noise \\|Δ alive\\| |", "|---|---|---|---:|---|---:|---:|"]
    for name, a in r["arms"].items():
        p = a["point"]
        L.append(f"| `{name}` | {fmt(p['settle_diff_60'])} / {fmt(p['settle_diff_90'])} / {fmt(p['settle_diff_118'])} | "
                 f"{fmt(p['noise_settle_diff_60'])} / {fmt(p['noise_settle_diff_90'])} / {fmt(p['noise_settle_diff_118'])} | "
                 f"{fmt(p['settle_diff_118'] - p['noise_settle_diff_118'])} | {ci_s(a, 'alive_signed', 2)} | {fmt(p['alive_abs'], 2)} | "
                 f"{fmt(p['noise_alive_abs'], 2)} |")
    L += ["", "## Deaths (mean signed Δ arm − factual; noise r1 − r0)", "",
          "| arm | within 35 d of the cast: total [90 %] / \\|Δ\\| / noise \\|Δ\\| | by cause (starv., dehyd., combat, old age, exposure) | "
          "to day 120: total [90 %] / \\|Δ\\| / noise \\|Δ\\| | by cause |", "|---|---|---|---|---|"]
    for name, a in r["arms"].items():
        p = a["point"]
        by = lambda w: ", ".join(fmt(p[f"{w}_{c}"], 2) for c in CAUSES)
        L.append(f"| `{name}` | {ci_s(a, 'deaths35_total', 2)} / {fmt(p['deaths35_total_abs'], 2)} / {fmt(p['noise_deaths35_total_abs'], 2)} | "
                 f"{by('deaths35')} | {ci_s(a, 'deaths120_total', 2)} / {fmt(p['deaths120_total_abs'], 2)} / "
                 f"{fmt(p['noise_deaths120_total_abs'], 2)} | {by('deaths120')} |")
    L += ["", "## Applied amounts", "", "| arm | rows | requested | applied | rows applied < requested | min applied / requested |",
          "|---|---:|---:|---:|---:|---:|"]
    for name, a in r["arms"].items():
        ap = a["applied"]
        L.append(f"| `{name}` | {ap['rows']} | {ap['requested']} | {ap['applied']} | {ap['short_rows']} | {fmt(ap['min_ratio'])} |")
    if r.get("k"):
        L += ["", "## Targets", ""]
        for key, v in r["k"].items():
            L.append(f"- **{key}**: {json.dumps(v)}")
    return "\n".join(L) + "\n"


def k_targets(arms: dict, pairs: dict, seeds_of: dict, prereg: list[str] | None) -> dict:
    k: dict = {}
    if prereg:
        k2 = {}
        for name in prereg:
            if name not in arms:
                continue
            p = arms[name]["point"]
            if not RIGHT.get(arms[name]["kind"]):
                continue
            c1 = p["right_share"] is not None and p["right_share"] >= 0.75
            c2 = p["enr"] is not None and p["enr"] >= 2.0
            c3 = p["settle_diff_118"] >= p["noise_settle_diff_118"] + 0.15
            k2[name] = {"right_share": p["right_share"], "right_ge_0.75": c1, "enr": p["enr"], "enr_ge_2": c2,
                        "settle118_differs": p["settle_diff_118"], "noise_settle118": p["noise_settle_diff_118"],
                        "settle_clause": c3, "passed": bool(c1 and c2 and c3)}
        k["K2"] = k2
    k3 = {}
    for kind, (low, eve) in {"spoil_food": ("sp240L", "sp240E"), "gift_food": ("gf240L", "gf240E")}.items():
        if low in arms and eve in arms:
            both = sorted(set(seeds_of[low]) & set(seeds_of[eve]))
            k3[kind] = paired_right_diff(both, pairs[low], pairs[eve], kind)
    if k3:
        k3["passed"] = all(v["passed"] for v in k3.values())
        k["K3"] = k3
    k4 = {}
    for kind, names in {"spoil_food": ["sp60L", "sp120L", "sp240L"], "gift_food": ["gf60L", "gf120L", "gf240L"]}.items():
        if all(n in arms for n in names):
            rs = [arms[n]["point"]["right_share"] for n in names]
            ma = [arms[n]["point"]["mean_abs"] for n in names]
            inv = [f"{names[i]}→{names[i + 1]} {lab}" for lab, xs in (("right share", rs), ("mean |Δday|", ma))
                   for i in range(2) if xs[i] is not None and xs[i + 1] is not None and xs[i + 1] < xs[i]]
            k4[kind] = {"right_share": rs, "mean_abs": ma, "inversions": inv, "non_decreasing": not inv}
    if k4:
        k4["passed"] = all(v["non_decreasing"] for v in k4.values())
        k["K4"] = k4
    k5 = {}
    for name in ("rw120E", "rw120L", "rain13W", "cs2W"):
        if name in arms:
            a = arms[name]
            k5[name] = {w: {"signed": a["point"][f"{w}_total"], "ci90": a["ci90"][f"{w}_total"],
                            "abs": a["point"][f"{w}_total_abs"], "noise_abs": a["point"][f"noise_{w}_total_abs"]}
                        for w in ("deaths35", "deaths120")}
    if k5:
        k["K5"] = k5
    return k


def best_arms(arms: dict, min_seeds: int = 16) -> dict:
    """PASS14_GOAL.md 7: per kind (spoil_food, gift_food), among arms that are not `*E` and have >= `min_seeds` seeds,
    the largest right-direction share (ties: larger ENR, then smaller total dose = amount x doses; a tie left after
    those three rules goes to the arm with fewer doses, a rule Pass 14 adds)."""
    out = {}
    for kind in ("spoil_food", "gift_food"):
        cands = [(n, a) for n, a in arms.items() if a["kind"] == kind and a["when"] != "E" and len(a["seeds"]) >= min_seeds
                 and a["point"]["right_share"] is not None]
        key = lambda na: (-na[1]["point"]["right_share"], -(na[1]["point"]["enr"] if na[1]["point"]["enr"] is not None else -1.0),
                          na[1]["amount"] * na[1]["doses"], na[1]["doses"])
        ranked = sorted(cands, key=key)
        out[kind] = {"best": ranked[0][0] if ranked else None,
                     "ranking": [[n, a["point"]["right_share"], a["point"]["enr"], a["amount"] * a["doses"], len(a["seeds"])] for n, a in ranked]}
    return out


def run(a) -> dict:
    root = Path(a.root)
    lo, hi = (int(x) for x in a.seeds.split(".."))
    seeds = list(range(lo, hi))
    sched = load_manifests(a.manifest)
    arm_names = a.arms.split(",") if a.arms else list(dict.fromkeys(arm for _, arm in sched))
    errors: list[str] = []
    n_ep = 0
    facts: dict = {}
    for s in seeds:
        for k in range(a.replicates):
            for eid in ["none"] + [n for n in arm_names if (s, n) in sched]:
                path = root / f"{a.scene}_s{s}_{eid}_r{k}"
                f = episode_facts(path)
                n_ep += 1
                errors += [f"{path.name}: {e}" for e in check_episode(f, a.fork_tick, k, sched.get((s, eid)) if eid != "none" else None)]
                facts[(s, eid, k)] = f
    arms, pairs, seeds_of = {}, {}, {}
    for name in arm_names:
        arm_seeds = [s for s in seeds if (s, name) in sched]
        if not arm_seeds:
            continue
        info = sched[(arm_seeds[0], name)]
        pb = {s: [pair(facts[(s, name, k)], facts[(s, "none", k)], sched[(s, name)]["cast_tick"]) for k in range(a.replicates)]
              for s in arm_seeds}
        nb = {s: pair(facts[(s, "none", 1)], facts[(s, "none", 0)], sched[(s, name)]["cast_tick"]) for s in arm_seeds}
        summ = summarize(arm_seeds, pb, nb, info["kind"])
        recs = [rec for s in arm_seeds for k in range(a.replicates) for rec in facts[(s, name, k)]["interventions"]]
        ratios = [rec["applied"] / rec["amount"] for rec in recs if rec["amount"]]
        summ.update(kind=info["kind"], amount=info["amount"], doses=info["doses"], when=info["when"],
                    applied={"rows": len(recs), "requested": sum(rec["amount"] for rec in recs),
                             "applied": sum(rec["applied"] for rec in recs),
                             "short_rows": sum(1 for rec in recs if rec["applied"] < rec["amount"]),
                             "min_ratio": min(ratios) if ratios else None})
        summ["per_seed"] = {str(s): {"cast_tick": sched[(s, name)]["cast_tick"],
                                     "pairs": [{k2: v for k2, v in p.items() if k2 in ("day_a", "day_f", "delta", "direction")} for p in pb[s]],
                                     "noise": {k2: v for k2, v in nb[s].items() if k2 in ("day_a", "day_f", "delta", "direction")}}
                            for s in arm_seeds}
        arms[name], pairs[name], seeds_of[name] = summ, pb, arm_seeds
    out = {"label": a.label, "scene": a.scene, "seeds": [lo, hi], "replicates": a.replicates, "fork_tick": a.fork_tick,
           "checks": {"episodes": n_ep, "errors": errors}, "arms": arms,
           "k": k_targets(arms, pairs, seeds_of, a.prereg.split(",") if a.prereg else None),
           "factual_departs_r0": {str(s): facts[(s, "none", 0)]["departs"] for s in seeds}}
    if a.probe_summary:
        out["probe_crosscheck"] = probe_crosscheck(Path(a.probe_summary), {s: facts[(s, "none", 0)] for s in seeds})
    if a.unforked:
        useeds = sorted(int(p.name.split("_s")[1].split("_")[0]) for p in Path(a.unforked).iterdir() if p.name.endswith("_none"))
        out["unforked"] = compare_unforked(Path(a.unforked), root, a.scene, useeds)
    return out


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--root", required=True)
    ap.add_argument("--scene", default="game_v19")
    ap.add_argument("--seeds", required=True)
    ap.add_argument("--replicates", type=int, default=2)
    ap.add_argument("--fork-tick", type=int, default=14_280)
    ap.add_argument("--manifest", action="append", required=True)
    ap.add_argument("--arms")
    ap.add_argument("--prereg")
    ap.add_argument("--probe-summary")
    ap.add_argument("--unforked")
    ap.add_argument("--label", default="Stage A")
    ap.add_argument("--out", required=True)
    a = ap.parse_args(argv)
    r = run(a)
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    (out / "dose_map.json").write_text(json.dumps(r, indent=1, default=float) + "\n")
    (out / "dose_map.md").write_text(markdown(r))
    print(json.dumps({"episodes": r["checks"]["episodes"], "errors": len(r["checks"]["errors"]), "k": r["k"]}, default=float)[:4000])
    return 1 if r["checks"]["errors"] else 0


if __name__ == "__main__":
    raise SystemExit(main())
