#!/usr/bin/env python3
"""Fork metrics of PASS9_GOAL.md section 8: placebo identity, variance
reduction from pairing, v14 single forks, divergence horizon. stdlib + numpy.

Pass 10 (PASS10_GOAL.md section 12): `--deaths-with-exposure` adds
`deaths_exposure` (recorder schema 4) to the death count of the targets (at the
day-29 06:00 row) and of the divergence series (`bench_v1` itself unchanged);
`--v15 DIR --v15-replicates R` compares each variant's replicate-mean
dark-night effect per seed with Pass 9's game_v15 families ("differs" = the
two seed means differ); per variant the report adds the share of seeds whose
replicate-mean deaths effect is > 0 and the mean deaths effect over seeds.

usage: fork_metrics.py --root DIR --scene game_v15 --seeds 12000..12032 --replicates 8 --fork-tick 28800
                       --variants gift_wood,rot_wood,rain,cold_snap,windfall [--gift1 gift1]
                       [--placebos p0_gift_wood,...] [--placebo-seeds 12000..12008] [--unforked DIR]
                       [--v14 DIR] [--intervention-ticks gift_wood=29400,...] --out DIR

Episodes are recorder schema 3 directories `<scene>_s<seed>_<id>_r<k>` under
`--root` (`id` = `none` for the factual). Targets are dataset v1's C_cf
targets (`bench_v1.c_cf_targets`, unchanged): dark nights of the evenings of
one-based days 22-28, cold member-hours from day 22 18:00 to day 29 07:00, food
store and deaths at the day-29 06:00 row; births at that row are reported only.

Pairing (8.2): for variant v, target y, seed s and replicate k,
`d_paired(s,k) = y(v,s,k) - y(f,s,k)` and `d_unpaired(s,k) = y(v,s,k) - y(f,s,(k+1) mod R)`.
`V_p = mean_s var_k(d_paired(s,.))`, `V_u` likewise (ddof=1, within a seed,
averaged over seeds); VRF = V_u / V_p (V_p = 0 < V_u: infinite; both 0: "no
agent noise", excluded from G0b). The 95 % interval is a percentile cluster
bootstrap over seeds (2,000 resamples, a fresh `numpy.random.default_rng(9)`
per variant and target, so values never depend on the order of computation;
percentiles by `inverted_cdf`, i.e. order statistics, which handle infinite
resamples). A resample of seeds with no agent noise at all has no VRF and is
left out of the percentiles; the count is `bootstrap_quiet_resamples` (shown
beside the interval). A seed's replicates count as constant when their range
is at most 1e-9 x max(1, largest absolute value) (exact for integer targets;
rounding residue of fractional ones), in the variances and the noisy-seed
counts alike.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import sys
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "worldmodel"))
from bench_v1 import Ep, T4_TARGETS, c_cf_targets, episode_arrays, row_of  # noqa: E402
from wm_data import load_episode  # noqa: E402

GATED = ("dark_nights", "cold_member_hours")
RESAMPLES = 2000
BOOT_SEED = 9


# ------------------------------------------------------------------ statistics -------------------------------------
def constant_rows(a: np.ndarray) -> np.ndarray:
    """Per row: whether its values are equal up to rounding (range <= 1e-9 x max(1, max |value|))."""
    scale = np.maximum(1.0, np.abs(a).max(axis=1)) if a.size else np.ones(len(a))
    return np.ptp(a, axis=1) <= 1e-9 * scale


def within_variance(a: np.ndarray) -> float:
    """Mean over rows of the within-row variance (ddof=1); a constant row contributes 0."""
    var = np.var(a, axis=1, ddof=1)
    return float(np.where(constant_rows(a), 0.0, var).mean())


def vrf_of(y_v: np.ndarray, y_f: np.ndarray) -> tuple[float, float, float | None]:
    """(V_p, V_u, VRF) for arrays [S, R] of one variant and target; VRF None = no agent noise.
    Rows constant up to rounding contribute variance 0 (review rounds 1 and 2, L4 / R2-3)."""
    paired = y_v - y_f
    unpaired = y_v - np.roll(y_f, -1, axis=1)
    v_p = within_variance(paired)
    v_u = within_variance(unpaired)
    if v_p == 0.0:
        return v_p, v_u, (math.inf if v_u > 0.0 else None)
    return v_p, v_u, v_u / v_p


def effect_stats(y_v: np.ndarray, y_f: np.ndarray, resamples: int = RESAMPLES, boot_seed: int = BOOT_SEED) -> dict:
    """Section 8.2 statistics for one variant and target ([S, R] arrays, S seeds x R replicates)."""
    y_v, y_f = np.asarray(y_v, dtype=np.float64), np.asarray(y_f, dtype=np.float64)
    if y_v.shape != y_f.shape or y_v.ndim != 2 or y_v.shape[1] < 2:
        raise ValueError("need [seeds, replicates >= 2] arrays of equal shape")
    v_p, v_u, vrf = vrf_of(y_v, y_f)
    paired = y_v - y_f
    seeds = y_v.shape[0]
    lo = hi = None
    quiet = 0
    if vrf is not None:
        rng = np.random.default_rng(boot_seed)
        draws = []
        for _ in range(resamples):
            index = rng.integers(0, seeds, seeds)
            value = vrf_of(y_v[index], y_f[index])[2]
            # A resample of seeds with no agent noise at all has no VRF: excluded
            # (review round 1, L5); counted in `bootstrap_quiet_resamples`.
            if value is None:
                quiet += 1
            else:
                draws.append(value)
        if draws:
            lo = float(np.quantile(draws, 0.025, method="inverted_cdf"))
            hi = float(np.quantile(draws, 0.975, method="inverted_cdf"))
    seed_mean = paired.mean(axis=1)
    # Where the agent noise reaches the outcome: seeds whose replicates differ in each arm.
    f_noisy, v_noisy = ~constant_rows(y_f), ~constant_rows(y_v)
    both = f_noisy & v_noisy
    vrf_both = vrf_of(y_v[both], y_f[both])[2] if both.sum() >= 1 else None
    stable = [bool(np.all(np.sign(row) == np.sign(m))) for row, m in zip(paired, seed_mean)]
    return {
        "v_paired": v_p, "v_unpaired": v_u, "vrf": vrf, "vrf_lo": lo, "vrf_hi": hi,
        "no_agent_noise": vrf is None,
        "passes_g0b": vrf is not None and vrf > 1.0 and lo is not None and lo > 1.0,
        "mean_effect": float(paired.mean()),
        "sign_stability": float(np.mean(stable)),
        "spread_median_std": float(np.median(np.std(paired, axis=1, ddof=1))),
        "median_abs_seed_mean": float(np.median(np.abs(seed_mean))),
        "seeds": int(seeds), "replicates": int(y_v.shape[1]),
        "seeds_factual_noisy": int(f_noisy.sum()), "seeds_variant_noisy": int(v_noisy.sum()), "seeds_both_noisy": int(both.sum()),
        "vrf_seeds_both_noisy": vrf_both, "bootstrap_quiet_resamples": quiet,
    }


# ------------------------------------------------------------------ episodes ---------------------------------------
def episode_name(scene: str, seed: int, eid: str, replicate: int | None) -> str:
    return f"{scene}_s{seed}_{eid}" + ("" if replicate is None else f"_r{replicate}")


def targets_of(path: Path, with_exposure: bool = False) -> dict[str, float]:
    """The C_cf targets of one episode plus births at the day-29 06:00 row (reported only).
    `with_exposure` (Pass 10) adds `deaths_exposure` at that row to the deaths target."""
    loaded = load_episode(path, arrays=["global.npy", "settlements.npy"])
    ep = Ep("", "", int(loaded.meta["seed"]), str(loaded.meta["episode_id"]), **episode_arrays(loaded))
    out = c_cf_targets(ep)
    out["births"] = float(loaded.column("global.npy", "births")[row_of(28, 360)])
    if with_exposure:
        out["deaths"] += float(loaded.column("global.npy", "deaths_exposure")[row_of(28, 360)])
    return out


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def identity(a: Path, b: Path) -> dict:
    """Section 8.1's identity of two episodes: every `.npy` equal by sha256, `interventions.json`
    equal after removing amount-0 records, `meta.json` equal except `episode_id` and `noise_fork`,
    with `schedule` compared after removing amount-0 rows (a placebo's schedule differs from the
    factual's null only by such rows)."""
    npy_a = sorted(p.name for p in a.glob("*.npy"))
    npy_b = sorted(p.name for p in b.glob("*.npy"))
    differing = [n for n in npy_a if n not in npy_b or sha256(a / n) != sha256(b / n)] + [n for n in npy_b if n not in npy_a]
    drop = lambda recs: [r for r in recs if r.get("amount") != 0]
    inter_a = json.loads((a / "interventions.json").read_text())
    inter_b = json.loads((b / "interventions.json").read_text())
    meta_a, meta_b = json.loads((a / "meta.json").read_text()), json.loads((b / "meta.json").read_text())
    schedules_ok = drop(meta_a.get("schedule") or []) == drop(meta_b.get("schedule") or [])
    strip = lambda m: {k: v for k, v in m.items() if k not in ("episode_id", "noise_fork", "schedule")}
    meta_diff = sorted(k for k in set(strip(meta_a)) | set(strip(meta_b)) if strip(meta_a).get(k) != strip(meta_b).get(k))
    same = not differing and drop(inter_a) == drop(inter_b) and not meta_diff and schedules_ok
    return {"identical": same, "npy_differing": differing, "interventions_equal": drop(inter_a) == drop(inter_b),
            "meta_differing": meta_diff, "schedules_equal_without_amount_zero": schedules_ok}


def series(path: Path, with_exposure: bool = False) -> dict:
    """Per-row series behind each target, the agent positions and lineage keys
    (`with_exposure`: exposure deaths counted in the deaths series, Pass 10)."""
    ep = load_episode(path)
    g = lambda name: ep.column("global.npy", name)
    s = ep.arrays["settlements.npy"]
    fire = np.nan_to_num(ep.column("settlements.npy", "fire_lit"), nan=0.0).max(axis=1, initial=0.0) if s.shape[1] else np.zeros(len(g("tick")))
    food = np.nansum(ep.column("settlements.npy", "food_stock"), axis=1) if s.shape[1] else np.zeros(len(g("tick")))
    deaths = sum(g(k) for k in ("deaths_starvation", "deaths_dehydration", "deaths_combat", "deaths_old_age"))
    if with_exposure:
        deaths = deaths + g("deaths_exposure")
    agents = ep.arrays["agents.npy"]
    xy = agents[..., [ep.field_index("agents.npy", "x"), ep.field_index("agents.npy", "y")]]
    return {"tick": g("tick").astype(np.int64), "dark_nights": fire, "cold_member_hours": g("cold_member_hours_total"),
            "food_store": food, "deaths": deaths, "alive": ep.arrays["agent_alive.npy"].astype(bool), "xy": xy,
            "keys": ep.lineage_keys(), "births": g("births")}


def divergence(f: dict, v: dict, start_tick: int) -> dict:
    """Per row from `start_tick` on: the share of agents alive in both arms (matched by lineage
    key) whose position differs, and the first row tick at which each target's series differs."""
    rows = [i for i, t in enumerate(f["tick"]) if t >= start_tick]
    index_f = {int(k): i for i, k in enumerate(f["keys"])}
    index_v = {int(k): i for i, k in enumerate(v["keys"])}
    common = sorted(set(index_f) & set(index_v))
    shares = []
    for i in rows:
        both = [k for k in common if f["alive"][i, index_f[k]] and v["alive"][i, index_v[k]]]
        moved = sum(1 for k in both if not np.array_equal(f["xy"][i, index_f[k]], v["xy"][i, index_v[k]]))
        shares.append(moved / len(both) if both else 0.0)
    first = {}
    for name in (*T4_TARGETS, "births"):
        diff = [int(f["tick"][i]) for i in rows if not np.array_equal(f[name][i], v[name][i], equal_nan=True)]
        first[name] = diff[0] if diff else None
    first_moved = next((int(f["tick"][i]) for i, s in zip(rows, shares) if s > 0), None)
    return {"ticks": [int(f["tick"][i]) for i in rows], "moved_share": shares, "first_target_diff_tick": first,
            "first_moved_tick": first_moved}


def newborn_key_match(f: dict, v: dict, start_tick: int) -> dict:
    """Newborns (first alive row after row 0) born at or after `start_tick` in either arm: how many
    carry a lineage key present among the other arm's newborns (Y6, reported only)."""
    def newborns(arm: dict) -> set[int]:
        out = set()
        for j, key in enumerate(arm["keys"]):
            alive_rows = np.nonzero(arm["alive"][:, j])[0]
            if len(alive_rows) and alive_rows[0] > 0 and arm["tick"][alive_rows[0]] >= start_tick:
                out.add(int(key))
        return out
    a, b = newborns(f), newborns(v)
    union = a | b
    return {"newborns_either": len(union), "newborns_both_equal_key": len(a & b),
            "share": (len(a & b) / len(union)) if union else None}


# ------------------------------------------------------------------ report -----------------------------------------
def parse_range(text: str) -> range:
    a, b = text.split("..")
    return range(int(a), int(b))


def collect(root: Path, scene: str, seeds: range, ids: list[str], replicates: int, with_exposure: bool = False) -> dict:
    """targets[id][target] -> array [S, R]."""
    out = {eid: {t: np.zeros((len(seeds), replicates)) for t in (*T4_TARGETS, "births")} for eid in ids}
    for si, seed in enumerate(seeds):
        for eid in ids:
            for k in range(replicates):
                values = targets_of(root / episode_name(scene, seed, eid, k), with_exposure)
                for t, value in values.items():
                    out[eid][t][si, k] = value
    return out


def build(args) -> dict:
    root, scene, seeds, reps = args.root, args.scene, parse_range(args.seeds), args.replicates
    variants = [v for v in args.variants.split(",") if v]
    report: dict = {"scene": scene, "seeds": [seeds.start, seeds.stop], "replicates": reps, "fork_tick": args.fork_tick,
                    "targets": list(T4_TARGETS), "gated_targets": list(GATED), "resamples": RESAMPLES, "boot_seed": BOOT_SEED}
    ids = ["none", *variants] + ([args.gift1] if args.gift1 else [])
    with_exposure = getattr(args, "deaths_with_exposure", False)
    report["deaths_with_exposure"] = with_exposure
    values = collect(root, scene, seeds, ids, reps, with_exposure)
    report["vrf"] = {}
    for eid in ids[1:]:
        report["vrf"][eid] = {t: effect_stats(values[eid][t], values["none"][t]) for t in (*T4_TARGETS, "births")}
    excluded = [(eid, t) for eid in variants for t in GATED if report["vrf"][eid][t]["no_agent_noise"]]
    report["g0b"] = {"passed": all(report["vrf"][eid][t]["passes_g0b"] for eid in variants for t in GATED
                                   if (eid, t) not in excluded),
                     "excluded_no_agent_noise": [f"{eid}:{t}" for eid, t in excluded]}
    # Placebos (8.1.1) and replicate 0 against the unforked run (8.1.2).
    placebo = []
    if args.placebos:
        for seed in parse_range(args.placebo_seeds):
            for pid in args.placebos.split(","):
                for k in range(reps):
                    result = identity(root / episode_name(scene, seed, pid, k), root / episode_name(scene, seed, "none", k))
                    placebo.append({"seed": seed, "id": pid, "replicate": k, **result})
    replicate0 = replicate0_identity(root, args.unforked, scene, seeds, ids) if args.unforked else []
    report["g0a"] = {"placebo_runs": len(placebo), "placebo_identical": sum(r["identical"] for r in placebo),
                     "replicate0_runs": len(replicate0), "replicate0_identical": sum(r["identical"] for r in replicate0),
                     "failures": [r for r in placebo + replicate0 if not r["identical"]]}
    # Nothing compared is not a pass (review round 2, R2-5): placebos and every
    # family's replicate 0 must be present.
    report["g0a"]["replicate0_expected"] = len(seeds) * len(ids)
    report["g0a"]["passed"] = (len(placebo) > 0 and report["g0a"]["placebo_identical"] == len(placebo)
                               and len(replicate0) == len(seeds) * len(ids)
                               and report["g0a"]["replicate0_identical"] == len(replicate0))
    # Divergence horizon (8.4) and newborn key match (Y6) at replicate 0.
    ticks = dict(item.split("=") for item in args.intervention_ticks.split(",")) if args.intervention_ticks else {}
    report["divergence"] = {}
    report["newborn_keys"] = {}
    for eid in ids[1:]:
        start = int(ticks.get(eid, args.fork_tick))
        per_seed, keys = [], []
        for seed in seeds:
            f = series(root / episode_name(scene, seed, "none", 0), with_exposure)
            v = series(root / episode_name(scene, seed, eid, 0), with_exposure)
            per_seed.append(divergence(f, v, start))
            keys.append(newborn_key_match(f, v, start))
        report["divergence"][eid] = summarize_divergence(per_seed, start)
        either = sum(k["newborns_either"] for k in keys)
        report["newborn_keys"][eid] = {"newborns_either": either,
                                       "newborns_both_equal_key": sum(k["newborns_both_equal_key"] for k in keys),
                                       "share": (sum(k["newborns_both_equal_key"] for k in keys) / either) if either else None}
    if args.v14:
        report["v14_single_fork"] = v14_comparison(args.v14, values, variants, seeds, report)
    report["deaths_effect"] = {eid: deaths_effect(values[eid]["deaths"], values["none"]["deaths"]) for eid in ids[1:]}
    if getattr(args, "v15", None):
        v15 = collect(args.v15, "game_v15", seeds, ids, args.v15_replicates)
        report["v15_dark_night_comparison"] = {
            eid: dark_night_differs(values[eid]["dark_nights"], values["none"]["dark_nights"],
                                    v15[eid]["dark_nights"], v15["none"]["dark_nights"]) for eid in ids[1:]}
        report["v15_dark_night_comparison_source"] = {"root": str(args.v15), "replicates": args.v15_replicates}
    return report


def deaths_effect(y_v: np.ndarray, y_f: np.ndarray) -> dict:
    """Pass 10 (H8): the replicate-mean deaths effect per seed, mean_k (y_v[s,k] - y_f[s,k]);
    the share of seeds where it is > 0 and its mean over seeds."""
    per_seed = (y_v - y_f).mean(axis=1)
    return {"per_seed": [float(x) for x in per_seed], "share_positive": float(np.mean(per_seed > 0)),
            "mean": float(per_seed.mean()), "share_negative": float(np.mean(per_seed < 0))}


def dark_night_differs(v16_v: np.ndarray, v16_f: np.ndarray, v15_v: np.ndarray, v15_f: np.ndarray) -> dict:
    """Pass 10: per seed, whether the replicate-mean dark-night effect of v16 differs from v15's."""
    a, b = (v16_v - v16_f).mean(axis=1), (v15_v - v15_f).mean(axis=1)
    differs = np.abs(a - b) > 1e-9
    return {"share_differs": float(np.mean(differs)), "v16_mean": float(a.mean()), "v15_mean": float(b.mean()),
            "seeds_differing": int(differs.sum()), "seeds": int(len(a))}


def replicate0_identity(root: Path, unforked_root: Path, scene: str, seeds, ids: list[str]) -> list[dict]:
    """8.1.2: replicate 0 of every family against its unforked run; a missing unforked run
    counts as a failure (review round 1, L6)."""
    out = []
    for seed in seeds:
        for eid in ids:
            unforked = unforked_root / episode_name(scene, seed, eid, None)
            if unforked.exists():
                result = identity(root / episode_name(scene, seed, eid, 0), unforked)
            else:
                result = {"identical": False, "missing_unforked_run": True}
            out.append({"seed": seed, "id": eid, **result})
    return out


def summarize_divergence(per_seed: list[dict], start: int) -> dict:
    """Medians over seeds: the moved share per row and the first differing tick per target
    (ticks after the intervention; `never` counts seeds where a target never differs)."""
    shares = np.array([d["moved_share"] for d in per_seed])
    out = {"intervention_tick": start, "rows_after": int(shares.shape[1]),
           "median_moved_share_by_row": [float(x) for x in np.median(shares, axis=0)]}
    for name in (*T4_TARGETS, "births"):
        firsts = [d["first_target_diff_tick"][name] for d in per_seed]
        hit = [f - start for f in firsts if f is not None]
        out[f"first_diff_{name}"] = {"median_ticks_after": float(np.median(hit)) if hit else None,
                                     "never": sum(1 for f in firsts if f is None)}
    moved = [d["first_moved_tick"] - start for d in per_seed if d["first_moved_tick"] is not None]
    out["first_moved"] = {"median_ticks_after": float(np.median(moved)) if moved else None,
                          "never": sum(1 for d in per_seed if d["first_moved_tick"] is None)}
    # The row `hours` after the intervention, by its tick (row 0 is the intervention
    # tick itself, taken before the intervention; review round 1, L1).
    ticks = per_seed[0]["ticks"] if per_seed else []
    for hours in (1, 6, 24, 72):
        row = ticks.index(start + 60 * hours) if start + 60 * hours in ticks else None
        out[f"median_moved_share_after_{hours}h"] = float(np.median(shares[:, row])) if row is not None else None
    return out


def v14_comparison(v14_root: Path, values: dict, variants: list[str], seeds: range, report: dict) -> dict:
    """8.3: mean over seeds of |d_v14(s) - mean_k d_v15(s,k)| beside v15's median within-seed std."""
    out = {}
    for eid in variants:
        out[eid] = {}
        for t in T4_TARGETS:
            diffs = []
            for si, seed in enumerate(seeds):
                f = targets_of(v14_root / f"game_v14_s{seed}_none")[t]
                v = targets_of(v14_root / f"game_v14_s{seed}_{eid}")[t]
                d15 = float((values[eid][t][si] - values["none"][t][si]).mean())
                diffs.append(abs((v - f) - d15))
            out[eid][t] = {"mean_abs_v14_minus_v15_mean": float(np.mean(diffs)),
                           "v15_median_within_seed_std": report["vrf"][eid][t]["spread_median_std"]}
    return out


def fmt(x) -> str:
    if x is None:
        return "–"
    if isinstance(x, float) and math.isinf(x):
        return "∞"
    return f"{x:.3g}" if isinstance(x, float) else str(x)


def markdown(report: dict) -> str:
    lines = [f"# Fork metrics ({report['scene']}, seeds {report['seeds'][0]}–{report['seeds'][1] - 1}, "
             f"R = {report['replicates']}, fork tick {report['fork_tick']})", ""]
    g0a = report["g0a"]
    lines += ["## G0a — placebo identity", "",
              f"- amount-0 placebos identical to the factual of the same replicate: {g0a['placebo_identical']} / {g0a['placebo_runs']}",
              f"- replicate 0 identical to the unforked run: {g0a['replicate0_identical']} / {g0a['replicate0_runs']}",
              f"- **{'PASS' if g0a['passed'] else 'FAIL'}**", ""]
    lines += ["## G0b — variance reduction from pairing", "",
              "VRF = V_unpaired / V_paired, variances over replicates within a seed, averaged over seeds; 95 % interval from a "
              "cluster bootstrap over seeds. The unpaired arm still shares worldgen, weather and the whole prefix up to the fork, "
              "so VRF measures the value of pairing the post-fork agent noise only.", "",
              "Noisy seeds: seeds whose 8 replicates do not all give the same value, in the factual (f), the variant (v) and both; "
              "where the factual is constant within a seed, the paired and unpaired differences have the same variance and that "
              "seed contributes VRF 1. VRF (both noisy) restricts V_p and V_u to the seeds noisy in both arms (reported only). "
              "Bootstrap resamples with no agent noise at all have no VRF and are left out of the interval; their count is shown "
              "in brackets after it.", "",
              "| variant | target | VRF | 95 % CI | V_paired | V_unpaired | noisy seeds f / v / both | VRF (both noisy) | mean effect | sign stability | median std | median abs seed mean | G0b |",
              "|---|---|---:|---|---:|---:|---|---:|---:|---:|---:|---:|---|"]
    for eid, by_target in report["vrf"].items():
        for t, s in by_target.items():
            gate = ("no agent noise" if s["no_agent_noise"] else ("pass" if s["passes_g0b"] else "fail")) if t in GATED else "reported"
            quiet = f" ({s['bootstrap_quiet_resamples']} quiet)" if s.get("bootstrap_quiet_resamples") else ""
            lines.append(f"| {eid} | {t} | {fmt(s['vrf'])} | [{fmt(s['vrf_lo'])}, {fmt(s['vrf_hi'])}]{quiet} | {fmt(s['v_paired'])} | "
                         f"{fmt(s['v_unpaired'])} | {s['seeds_factual_noisy']} / {s['seeds_variant_noisy']} / {s['seeds_both_noisy']} | "
                         f"{fmt(s['vrf_seeds_both_noisy'])} | {fmt(s['mean_effect'])} | {fmt(s['sign_stability'])} | {fmt(s['spread_median_std'])} | "
                         f"{fmt(s['median_abs_seed_mean'])} | {gate} |")
    g0b = report["g0b"]
    lines += ["", f"G0b (dark nights and cold member-hours of every C_cf variant): **{'PASS' if g0b['passed'] else 'FAIL'}**; "
              f"excluded as no agent noise: {', '.join(g0b['excluded_no_agent_noise']) or 'none'}.", ""]
    if "v14_single_fork" in report:
        lines += ["## v14 single forks against v15", "",
                  "| variant | target | mean over seeds of abs(d_v14 − mean_k d_v15) | v15 median within-seed std |", "|---|---|---:|---:|"]
        for eid, by_target in report["v14_single_fork"].items():
            for t, s in by_target.items():
                lines.append(f"| {eid} | {t} | {fmt(s['mean_abs_v14_minus_v15_mean'])} | {fmt(s['v15_median_within_seed_std'])} |")
        lines.append("")
    lines += ["## Divergence horizon (replicate 0, medians over seeds)", "",
              "| variant | intervention tick | first agent moved (ticks after) | moved share after 1 h / 6 h / 24 h / 72 h | "
              "first diff: dark nights | cold member-hours | food store | deaths | births |", "|---|---:|---|---|---|---|---|---|---|"]
    for eid, d in report["divergence"].items():
        cell = lambda name: f"{fmt(d[name]['median_ticks_after'])} (never {d[name]['never']})"
        lines.append(f"| {eid} | {d['intervention_tick']} | {cell('first_moved')} | "
                     f"{fmt(d['median_moved_share_after_1h'])} / {fmt(d['median_moved_share_after_6h'])} / "
                     f"{fmt(d['median_moved_share_after_24h'])} / {fmt(d['median_moved_share_after_72h'])} | "
                     f"{cell('first_diff_dark_nights')} | {cell('first_diff_cold_member_hours')} | {cell('first_diff_food_store')} | "
                     f"{cell('first_diff_deaths')} | {cell('first_diff_births')} |")
    if "deaths_effect" in report:
        lines += ["", "## Deaths effect (replicate mean per seed" + (", exposure deaths included" if report.get("deaths_with_exposure") else "") + ")", "",
                  "Deaths up to the day-29 06:00 row, variant minus factual of the same replicate, averaged over the replicates of each seed.", "",
                  "| variant | seeds with effect > 0 | seeds with effect < 0 | mean effect over seeds | dark-night effect differs from v15 (seeds) |",
                  "|---|---:|---:|---:|---|"]
        comp = report.get("v15_dark_night_comparison", {})
        for eid, d in report["deaths_effect"].items():
            c = comp.get(eid)
            cell = f"{c['share_differs']:.3f} ({c['seeds_differing']} / {c['seeds']}; v16 {c['v16_mean']:.3f}, v15 {c['v15_mean']:.3f})" if c else "–"
            lines.append(f"| {eid} | {d['share_positive']:.3f} | {d['share_negative']:.3f} | {d['mean']:.3f} | {cell} |")
    lines += ["", "## Newborn lineage keys (Y6, reported only)", "",
              "Newborns born at or after the intervention in either arm (replicate 0), and how many carry a key present among the "
              "other arm's newborns.", "", "| variant | newborns (either arm) | equal key in both | share |", "|---|---:|---:|---:|"]
    for eid, k in report["newborn_keys"].items():
        lines.append(f"| {eid} | {k['newborns_either']} | {k['newborns_both_equal_key']} | {fmt(k['share'])} |")
    return "\n".join(lines) + "\n"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--root", required=True, type=Path)
    parser.add_argument("--scene", default="game_v15")
    parser.add_argument("--seeds", required=True)
    parser.add_argument("--replicates", type=int, required=True)
    parser.add_argument("--fork-tick", type=int, required=True)
    parser.add_argument("--variants", required=True)
    parser.add_argument("--gift1")
    parser.add_argument("--placebos")
    parser.add_argument("--placebo-seeds", default="12000..12008")
    parser.add_argument("--unforked", type=Path)
    parser.add_argument("--v14", type=Path)
    parser.add_argument("--intervention-ticks")
    parser.add_argument("--deaths-with-exposure", action="store_true",
                        help="count deaths_exposure (recorder schema 4) in the deaths target and series")
    parser.add_argument("--v15", type=Path, help="Pass 9's game_v15 branch families (read-only) for the dark-night comparison")
    parser.add_argument("--v15-replicates", type=int, default=8)
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args(argv)
    report = build(args)
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "fork_metrics.json").write_text(json.dumps(report, indent=2, sort_keys=True, allow_nan=True) + "\n")
    (args.out / "fork_metrics.md").write_text(markdown(report))
    print(f"G0a {'PASS' if report['g0a']['passed'] else 'FAIL'}  G0b {'PASS' if report['g0b']['passed'] else 'FAIL'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
