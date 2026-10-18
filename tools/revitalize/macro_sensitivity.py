#!/usr/bin/env python3
"""Pass 11 Part 6.2: macro sensitivity of game_v17 to a small act of the god (PASS11_GOAL.md 12.2). numpy only.

usage: macro_sensitivity.py --root DIR --scene game_v17 --seeds 12000..12032 --variants rot_wood,gift_wood
                            --replicates 2 --out DIR

DIR holds 120-day schema-5 recordings `<scene>_s<seed>_<id>_r<k>` (no --agents) of the factual (`none`) and
each variant, forked at the same tick with replicates 0..R-1. Per episode the macro shape is: settlements alive
(>= 1 member, from settlements.npy) at the 08:00 row of days 60 / 90 / 118, the first-fission day (display day of
the first row with fissions_total >= 1, None without a fission) and total alive at day 118. Reported:
(a) noise only: the share of seeds whose shape differs between the factual's replicates 0 and 1, and the mean
|delta alive day 118|; (b) per variant: the share of (seed, replicate) pairs whose shape differs from the factual
of the same replicate, per seed the share of replicates that differ, and the mean |delta alive day 118|.

Pass 12 (PASS12_GOAL.md 12.2; `--post-fork`): with autumn departures many worlds split before the fork, so the
shape compares the **first fission founded after the fork tick** instead (display day of the smallest
`founded_tick` > fork tick among settlements with a parent, from `settlements.npy`; None without one). Reported
besides: among seeds where both episodes split after the fork, the mean |delta post-fork first-fission day|.

Pass 13 (PASS13_GOAL.md 10 J11, 12.2; `--j11 FORK_TICK`, schema 6): "post-fork fission" = the first fission whose
group **departed** after the fork (settlements.npy `depart_tick` > fork tick, a founded daughter; a cancelled
founding leaves no settlement and is not seen); its day = the display day of that departure. A fission in one run
and none in the other counts as differing, none in both as equal; a mean over pairs needs n >= 12, else "not
measured". (a) noise only (factual r0 vs r1): the share of seeds whose post-fork day differs by more than 3 days
(<= 0.35), the mean abs delta of that day over seeds where both split after the fork (<= 7) and the mean abs delta
alive at day 118 (<= 5). (b) the signed shift (variant day - factual day, same seed and replicate, both split after
the fork): `spoil_food` < 0 in >= 0.60 of those pairs, `gift_food` > 0 in >= 0.60 (ties count against); the share
with shift != 0 is reported. (c) noise_c = share of seeds whose r0 and r1 differ in the day-118 settlement count or
in the post-fork day by more than 3 days; variant_c = the same rule per (seed, replicate) against the same-replicate
factual; pass when variant_c >= noise_c + 0.10 for at least two of `rot_wood`, `spoil_food`, `gift_food`.
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

DAYS = (60, 90, 118)
TPD, OFFSET, EVERY = 1440, 480, 60


def row_of(day: int, minute: int) -> int:
    """Row of one-based display day `day` at minute-of-day `minute` (rows every 60 ticks)."""
    return ((day - 1) * TPD + minute - OFFSET) // EVERY


def post_fork_first_fission(path: Path, fork_tick: int) -> int | None:
    """Display day of the first daughter founded after `fork_tick` (settlements.npy `founded_tick`, `parent_slot`)."""
    ep = load_episode(path, arrays=["global.npy", "settlements.npy"])
    if ep.arrays["settlements.npy"].shape[1] == 0:
        return None
    founded = ep.column("settlements.npy", "founded_tick")
    parent = ep.column("settlements.npy", "parent_slot")
    ticks = []
    for s in range(founded.shape[1]):
        f, p = founded[:, s], parent[:, s]
        ok = np.isfinite(f) & np.isfinite(p) & (np.nan_to_num(p, nan=-1.0) >= 0)
        if ok.any():
            t = int(f[ok][0])
            if t > fork_tick:
                ticks.append(t)
    return None if not ticks else int((min(ticks) + OFFSET) // TPD + 1)


def post_fork_depart_day(path: Path, fork_tick: int) -> int | None:
    """Pass 13: display day of the first departure after `fork_tick` among founded daughters (schema-6 `depart_tick`)."""
    ep = load_episode(path, arrays=["settlements.npy"])
    if ep.arrays["settlements.npy"].shape[1] == 0:
        return None
    depart = ep.column("settlements.npy", "depart_tick")
    ticks = [int(v) for v in np.unique(depart[np.isfinite(depart)]) if v >= 0 and int(v) > fork_tick]
    return None if not ticks else int((min(ticks) + OFFSET) // TPD + 1)


def j11_shape(path: Path, fork_tick: int) -> dict:
    ep = load_episode(path, arrays=["global.npy", "settlements.npy"])
    members = ep.column("settlements.npy", "members") if ep.arrays["settlements.npy"].shape[1] else None
    row = row_of(118, 480)
    count = 0 if members is None else int(np.sum(np.nan_to_num(members[row], nan=0.0) >= 1))
    return {"settlements_118": count, "alive_118": float(ep.column("global.npy", "alive")[row]),
            "post_fork_day": post_fork_depart_day(path, fork_tick)}


def day_differs(a: int | None, b: int | None, tol: int = 3) -> bool:
    """A fission in one run and none in the other differs; none in both is equal."""
    if a is None and b is None:
        return False
    if a is None or b is None:
        return True
    return abs(a - b) > tol


def j11_pair_differs(a: dict, b: dict) -> bool:
    """(c)'s rule: day-118 settlement count or post-fork day by more than 3 days."""
    return a["settlements_118"] != b["settlements_118"] or day_differs(a["post_fork_day"], b["post_fork_day"])


def signed_shifts(pairs: list[tuple[dict, dict]]) -> list[int]:
    """variant day - factual day over pairs (variant, factual) where both split after the fork."""
    return [a["post_fork_day"] - b["post_fork_day"] for a, b in pairs
            if a["post_fork_day"] is not None and b["post_fork_day"] is not None]


def build_j11(shapes: dict, scene: str, seeds: range, variants: list[str], replicates: int, fork_tick: int,
              min_n: int = 12) -> dict:
    """Pass 13 J11 on `shapes[(seed, id, k)]` = `j11_shape` (see the module docs)."""
    mean_or_none = lambda xs: (float(np.mean(xs)) if len(xs) >= min_n else None)
    noise_pairs = [(shapes[(s, "none", 0)], shapes[(s, "none", 1)]) for s in seeds]
    both = [abs(a["post_fork_day"] - b["post_fork_day"]) for a, b in noise_pairs
            if a["post_fork_day"] is not None and b["post_fork_day"] is not None]
    noise_day_share = float(np.mean([day_differs(a["post_fork_day"], b["post_fork_day"]) for a, b in noise_pairs]))
    noise_alive = float(np.mean([abs(a["alive_118"] - b["alive_118"]) for a, b in noise_pairs]))
    noise_c = float(np.mean([j11_pair_differs(a, b) for a, b in noise_pairs]))
    noise_mean_day = mean_or_none(both)
    a_clauses = {"share_day_differs_gt3": {"value": noise_day_share, "passed": noise_day_share <= .35},
                 "mean_abs_delta_day": {"value": noise_mean_day, "n": len(both),
                                        "passed": None if noise_mean_day is None else noise_mean_day <= 7},
                 "mean_abs_delta_alive_118": {"value": noise_alive, "passed": noise_alive <= 5}}
    out = {"scene": scene, "seeds": [seeds.start, seeds.stop], "replicates": replicates, "fork_tick": fork_tick,
           "noise": {**a_clauses, "noise_c": noise_c, "n_both_split_after_fork": len(both)}, "variants": {}}
    for eid in variants:
        pairs = [(shapes[(s, eid, k)], shapes[(s, "none", k)]) for s in seeds for k in range(replicates)]
        shifts = signed_shifts(pairs)
        variant_c = float(np.mean([j11_pair_differs(a, b) for a, b in pairs]))
        absd = [abs(x) for x in shifts]
        out["variants"][eid] = {
            "n_both_split_after_fork": len(shifts), "signed_shift_mean": mean_or_none(shifts),
            "share_shift_negative": float(np.mean([x < 0 for x in shifts])) if shifts else None,
            "share_shift_positive": float(np.mean([x > 0 for x in shifts])) if shifts else None,
            "share_shift_nonzero": float(np.mean([x != 0 for x in shifts])) if shifts else None,
            "mean_abs_delta_day": mean_or_none(absd),
            "share_day_differs_gt3": float(np.mean([day_differs(a["post_fork_day"], b["post_fork_day"]) for a, b in pairs])),
            "mean_abs_delta_alive_118": float(np.mean([abs(a["alive_118"] - b["alive_118"]) for a, b in pairs])),
            "variant_c": variant_c, "variant_c_minus_noise_c": variant_c - noise_c}
    def shift_clause(eid: str, sign: int) -> dict:
        v = out["variants"].get(eid)
        if v is None or v["n_both_split_after_fork"] < min_n:
            return {"value": None if v is None else (v["share_shift_negative"] if sign < 0 else v["share_shift_positive"]),
                    "n": 0 if v is None else v["n_both_split_after_fork"], "passed": None}
        share = v["share_shift_negative"] if sign < 0 else v["share_shift_positive"]
        return {"value": share, "n": v["n_both_split_after_fork"], "passed": share >= .60}
    c_ok = [eid for eid in ("rot_wood", "spoil_food", "gift_food") if eid in out["variants"]
            and out["variants"][eid]["variant_c"] >= noise_c + .10]
    out["j11"] = {"a": a_clauses, "b": {"spoil_food_earlier": shift_clause("spoil_food", -1), "gift_food_later": shift_clause("gift_food", +1)},
                  "c": {"noise_c": noise_c, "variants_over": c_ok, "passed": len(c_ok) >= 2}}
    clauses = [*(c["passed"] for c in a_clauses.values()), *(c["passed"] for c in out["j11"]["b"].values()), out["j11"]["c"]["passed"]]
    out["j11"]["passed"] = all(c is True for c in clauses)
    out["j11"]["not_measured"] = [name for name, c in [*a_clauses.items(), *out["j11"]["b"].items()] if c["passed"] is None]
    out["per_seed"] = {str(s): {eid: [shapes[(s, eid, k)] for k in range(replicates)] for eid in ("none", *variants)} for s in seeds}
    return out


def markdown_j11(r: dict) -> str:
    fmt = lambda x: "—" if x is None else (f"{x:.3f}" if isinstance(x, float) else str(x))
    n = r["noise"]
    lines = [f"# Pass 13 macro sensitivity (12.2, J11) — `{r['scene']}`", "",
             f"Seeds {r['seeds'][0]}..{r['seeds'][1]}, R = {r['replicates']}, 120 days, fork at {r['fork_tick']}. Post-fork fission = "
             "the first fission whose group departed after the fork (schema-6 `depart_tick`); its day = the departure's display day.", "",
             "| Comparison | n both split after fork | share post-fork day differs > 3 d | mean abs Δ day | signed shift mean | share shift < 0 / > 0 / ≠ 0 | mean abs Δ alive 118 | c-rule share |",
             "|---|---:|---:|---:|---:|---|---:|---:|",
             f"| (a) factual r0 vs r1 (noise only) | {n['n_both_split_after_fork']} | {fmt(n['share_day_differs_gt3']['value'])} | "
             f"{fmt(n['mean_abs_delta_day']['value'])} | — | — | {fmt(n['mean_abs_delta_alive_118']['value'])} | {fmt(n['noise_c'])} |"]
    for eid, v in r["variants"].items():
        lines.append(f"| (b) {eid} vs factual, same replicate | {v['n_both_split_after_fork']} | {fmt(v['share_day_differs_gt3'])} | "
                     f"{fmt(v['mean_abs_delta_day'])} | {fmt(v['signed_shift_mean'])} | {fmt(v['share_shift_negative'])} / "
                     f"{fmt(v['share_shift_positive'])} / {fmt(v['share_shift_nonzero'])} | {fmt(v['mean_abs_delta_alive_118'])} | {fmt(v['variant_c'])} |")
    j = r["j11"]
    lines += ["", f"J11 (a): {json.dumps(j['a'])}", f"J11 (b): {json.dumps(j['b'])}", f"J11 (c): {json.dumps(j['c'])}",
              f"J11: {'PASS' if j['passed'] else 'FAIL'}" + (f" (not measured: {', '.join(j['not_measured'])})" if j["not_measured"] else "")]
    return "\n".join(lines) + "\n"


def differs_post(a: dict, b: dict) -> bool:
    return a["settlements"] != b["settlements"] or a["post_fork_first_fission_day"] != b["post_fork_first_fission_day"]


def shape_of(path: Path, fork_tick: int | None = None) -> dict:
    ep = load_episode(path, arrays=["global.npy", "settlements.npy"])
    members = ep.column("settlements.npy", "members") if ep.arrays["settlements.npy"].shape[1] else None
    alive_col = ep.column("global.npy", "alive")
    fissions = ep.column("global.npy", "fissions_total")
    tick = ep.column("global.npy", "tick").astype(np.int64)
    counts = [0 if members is None else int(np.sum(np.nan_to_num(members[row_of(d, 480)], nan=0.0) >= 1)) for d in DAYS]
    hit = np.nonzero(fissions >= 1)[0]
    first = None if not len(hit) else int((int(tick[hit[0]]) + OFFSET) // TPD + 1)
    out = {"settlements": counts, "first_fission_day": first, "alive_118": float(alive_col[row_of(118, 480)])}
    if fork_tick is not None:
        out["post_fork_first_fission_day"] = post_fork_first_fission(path, fork_tick)
    return out


def delta_day(pairs: list[tuple[dict, dict]]) -> dict:
    both = [abs(a["post_fork_first_fission_day"] - b["post_fork_first_fission_day"]) for a, b in pairs
            if a["post_fork_first_fission_day"] is not None and b["post_fork_first_fission_day"] is not None]
    return {"n_both_split_after_fork": len(both), "mean_abs_delta_day": float(np.mean(both)) if both else None,
            "share_with_post_fork_day_differing": float(np.mean([a["post_fork_first_fission_day"] != b["post_fork_first_fission_day"]
                                                                  for a, b in pairs])) if pairs else None}


def differs(a: dict, b: dict) -> bool:
    return a["settlements"] != b["settlements"] or a["first_fission_day"] != b["first_fission_day"]


def build(root: Path, scene: str, seeds: range, variants: list[str], replicates: int, fork_tick: int | None = None) -> dict:
    shapes = {(seed, eid, k): shape_of(root / f"{scene}_s{seed}_{eid}_r{k}", fork_tick)
              for seed in seeds for eid in ("none", *variants) for k in range(replicates)}
    if fork_tick is not None:
        return build_post(shapes, scene, seeds, variants, replicates, fork_tick)
    noise = [differs(shapes[(s, "none", 0)], shapes[(s, "none", 1)]) for s in seeds] if replicates >= 2 else []
    noise_alive = [abs(shapes[(s, "none", 0)]["alive_118"] - shapes[(s, "none", 1)]["alive_118"]) for s in seeds] if replicates >= 2 else []
    out = {"scene": scene, "seeds": [seeds.start, seeds.stop], "replicates": replicates, "days": list(DAYS),
           "noise": {"share_of_seeds_differing": float(np.mean(noise)) if noise else None,
                     "mean_abs_delta_alive_118": float(np.mean(noise_alive)) if noise_alive else None},
           "variants": {}, "per_seed": {}}
    for eid in variants:
        pairs = [differs(shapes[(s, eid, k)], shapes[(s, "none", k)]) for s in seeds for k in range(replicates)]
        per_seed = {str(s): float(np.mean([differs(shapes[(s, eid, k)], shapes[(s, "none", k)]) for k in range(replicates)]))
                    for s in seeds}
        alive = [abs(shapes[(s, eid, k)]["alive_118"] - shapes[(s, "none", k)]["alive_118"]) for s in seeds for k in range(replicates)]
        out["variants"][eid] = {"share_of_pairs_differing": float(np.mean(pairs)),
                                "share_of_seeds_differing_in_any_replicate": float(np.mean([v > 0 for v in per_seed.values()])),
                                "share_of_seeds_differing_in_every_replicate": float(np.mean([v == 1 for v in per_seed.values()])),
                                "mean_abs_delta_alive_118": float(np.mean(alive)), "per_seed": per_seed}
    out["per_seed"] = {str(s): {eid: [shapes[(s, eid, k)] for k in range(replicates)] for eid in ("none", *variants)} for s in seeds}
    return out


def build_post(shapes: dict, scene: str, seeds: range, variants: list[str], replicates: int, fork_tick: int) -> dict:
    """Pass 12: the comparisons of `build` on the post-fork shape."""
    noise_pairs = [(shapes[(s, "none", 0)], shapes[(s, "none", 1)]) for s in seeds] if replicates >= 2 else []
    out = {"scene": scene, "seeds": [seeds.start, seeds.stop], "replicates": replicates, "days": list(DAYS),
           "fork_tick": fork_tick, "post_fork": True,
           "noise": {"share_of_seeds_differing": float(np.mean([differs_post(a, b) for a, b in noise_pairs])) if noise_pairs else None,
                     "mean_abs_delta_alive_118": float(np.mean([abs(a["alive_118"] - b["alive_118"]) for a, b in noise_pairs])) if noise_pairs else None,
                     **delta_day(noise_pairs)},
           "variants": {}}
    for eid in variants:
        pairs = [(shapes[(s, eid, k)], shapes[(s, "none", k)]) for s in seeds for k in range(replicates)]
        out["variants"][eid] = {"share_of_pairs_differing": float(np.mean([differs_post(a, b) for a, b in pairs])),
                                "mean_abs_delta_alive_118": float(np.mean([abs(a["alive_118"] - b["alive_118"]) for a, b in pairs])),
                                **delta_day(pairs)}
    split_before = [s for s in seeds if shapes[(s, "none", 0)]["first_fission_day"] is not None
                    and shapes[(s, "none", 0)]["post_fork_first_fission_day"] is None]
    out["factual_r0"] = {"seeds_with_a_fission": sum(1 for s in seeds if shapes[(s, "none", 0)]["first_fission_day"] is not None),
                         "seeds_with_a_post_fork_fission": sum(1 for s in seeds if shapes[(s, "none", 0)]["post_fork_first_fission_day"] is not None),
                         "seeds_split_only_before_the_fork": len(split_before)}
    out["per_seed"] = {str(s): {eid: [shapes[(s, eid, k)] for k in range(replicates)] for eid in ("none", *variants)} for s in seeds}
    return out


def markdown_post(r: dict) -> str:
    fmt = lambda x: "—" if x is None else f"{x:.3f}"
    lines = ["# Pass 12 macro sensitivity (12.2)", "",
             f"Scene `{r['scene']}`, seeds {r['seeds'][0]}..{r['seeds'][1]}, R = {r['replicates']}, 120 days, fork at {r['fork_tick']}. "
             "Shape = settlements alive at days 60 / 90 / 118 and the display day of the first fission founded after the fork. "
             f"Factual r0: {r['factual_r0']}.", "",
             "| Comparison | Share differing (counts or post-fork day) | Share with post-fork day differing | Mean abs delta post-fork day (n both split) | Mean abs delta alive day 118 |",
             "|---|---:|---:|---:|---:|"]
    rows = [("(a) factual r0 vs r1 (noise only)", r["noise"])] + [(f"(b) {eid} vs factual, same replicate", v) for eid, v in r["variants"].items()]
    for label, v in rows:
        lines.append(f"| {label} | {fmt(v['share_of_pairs_differing'] if 'share_of_pairs_differing' in v else v['share_of_seeds_differing'])} | "
                     f"{fmt(v['share_with_post_fork_day_differing'])} | {fmt(v['mean_abs_delta_day'])} ({v['n_both_split_after_fork']}) | "
                     f"{fmt(v['mean_abs_delta_alive_118'])} |")
    return "\n".join(lines) + "\n"


def markdown(r: dict) -> str:
    lines = ["# Pass 11 macro sensitivity (12.2)", "",
             f"Scene `{r['scene']}`, seeds {r['seeds'][0]}..{r['seeds'][1]}, R = {r['replicates']}, 120 days, fork at 28,800. "
             "Shape = settlements alive at days 60 / 90 / 118 and the first-fission day.", "",
             "| Comparison | Share differing | Mean abs delta alive day 118 |", "|---|---:|---:|",
             f"| (a) factual r0 vs r1 (noise only) | {r['noise']['share_of_seeds_differing']} | {r['noise']['mean_abs_delta_alive_118']} |"]
    for eid, v in r["variants"].items():
        lines.append(f"| (b) {eid} vs factual, same replicate (seed-replicate pairs) | {v['share_of_pairs_differing']:.3f} | "
                     f"{v['mean_abs_delta_alive_118']:.3f} |")
    lines += ["", "Per variant: share of seeds differing in at least one / in every replicate: " +
              ", ".join(f"{eid} {v['share_of_seeds_differing_in_any_replicate']:.3f} / {v['share_of_seeds_differing_in_every_replicate']:.3f}"
                        for eid, v in r["variants"].items())]
    return "\n".join(lines) + "\n"


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--root", required=True, type=Path)
    ap.add_argument("--scene", default="game_v17")
    ap.add_argument("--seeds", required=True)
    ap.add_argument("--variants", default="rot_wood,gift_wood")
    ap.add_argument("--replicates", type=int, default=2)
    ap.add_argument("--post-fork", type=int, metavar="FORK_TICK", help="Pass 12: compare the first fission founded after this tick")
    ap.add_argument("--j11", type=int, metavar="FORK_TICK", help="Pass 13: J11 on the first departure after this tick (schema 6)")
    ap.add_argument("--out", required=True, type=Path)
    args = ap.parse_args(argv)
    a, b = args.seeds.split("..")
    if args.j11 is not None:
        seeds = range(int(a), int(b))
        variants = [v for v in args.variants.split(",") if v]
        shapes = {(seed, eid, k): j11_shape(args.root / f"{args.scene}_s{seed}_{eid}_r{k}", args.j11)
                  for seed in seeds for eid in ("none", *variants) for k in range(args.replicates)}
        report = build_j11(shapes, args.scene, seeds, variants, args.replicates, args.j11)
        text = markdown_j11(report)
        args.out.mkdir(parents=True, exist_ok=True)
        (args.out / "macro.json").write_text(json.dumps(report, indent=1, sort_keys=True) + "\n")
        (args.out / "macro.md").write_text(text)
        print(text)
        return 0
    report = build(args.root, args.scene, range(int(a), int(b)), [v for v in args.variants.split(",") if v], args.replicates,
                   args.post_fork)
    text = markdown_post(report) if args.post_fork is not None else markdown(report)
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "macro.json").write_text(json.dumps(report, indent=1, sort_keys=True) + "\n")
    (args.out / "macro.md").write_text(text)
    print(text)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
