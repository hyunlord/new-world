#!/usr/bin/env python3
"""Pass 13: the replicate measure — how much of a world's outcome is its path, not its parameters
(PASS13_GOAL.md 5.4). numpy only.

Data: for a scene, W worlds x R replicates forked by the noise fork at tick 1 (`worldsim_wm --fork-tick 1
--replicates R --schedule-dir <empty dir> --include-factual`). With an empty schedule directory the recorder writes,
per seed, the factual members `<scene>_s<seed>_none_r<k>`, k = 0..R-1, flat into `--out` (one directory per
episode; a `.tmp` directory is an episode still being written and is ignored). `wm_data.load_family` expects one
seed per directory, so this tool groups the members by name itself (`parse_member`).

For variable j and one-based display day d in {30, 60, 90, 118}:

- y_{w,r} = mean of v_j over the rows of day d: the rows r with `calcore_wm11.display_day(r)` == d, i.e. rows
  `row_of(d, 0)` .. `row_of(d + 1, 0) - 1` (`diversity_p12.day_means`, reused).
- v_j: `food_stock`, `wood_stock`, `alive`, `deadwood_total`, `food_on_bushes_total`, `adults_working` as
  `bench_v1.episode_arrays` builds them (food/wood = nansum over settlement slots); `settlements_alive` = number of
  settlement slots with members >= 1 at the row (`diversity_p11.settlements_alive`'s rule, applied to every row).
- Within-world Wd = mean_w var_r(y_{w,r}) (ddof 1); between-world Bd = var_w(ybar_w) (ddof 1) - Wd / R, floored at
  0. Path share PS = Wd / (Wd + Bd), NaN when both are 0. Path spread PSD = sqrt(Wd) / S_j, S_j =
  `bench_v1.split_std` of the scene (std, ddof 0, of v_j over rows >= 24 of all W x R episodes).
- The "mean" column is the mean over d in {60, 90, 118} of the per-day values (days where PS is NaN skipped).
- 90 % intervals: bootstrap over worlds (1,000 resamples, `numpy.random.default_rng(0)`), 5th and 95th percentiles.
  Only Wd and Bd are recomputed per resample; S_j is kept at its full-sample value (so PSD's interval is the
  interval of sqrt(Wd), scaled).
- `--diff A:B` reports A - B, bootstrapped over worlds paired across the two scenes (the same resampled seed
  indices for both; only seeds present in both scenes are used).
- `decompose(y[W, R])` is the decomposition on any per-(world, replicate) scalar (e.g. a first-fission day; NaN
  values propagate, so the caller encodes "never" explicitly).

usage:
  paths_p13.py --scene NAME=DIR [--scene NAME=DIR ...] [--diff A:B ...] [--days 30,60,90,118] [--boot 1000]
               [--check-unforked FORKED_DIR UNFORKED_DIR --seeds a,b] --out OUTDIR
writes OUTDIR/paths.json and OUTDIR/paths.md. `--check-unforked` compares, per seed, replicate 0 of the forked
family (`*_s<seed>_none_r0` in FORKED_DIR) with the unforked recording (`*_s<seed>_none` in UNFORKED_DIR): every
array present in both must be identical (same dtype and shape, `np.array_equal` with NaN = NaN, and the same bytes).
"""
from __future__ import annotations

import argparse
import json
import re
import sys
import time
import warnings
from dataclasses import dataclass, field
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent))
import diversity_p12 as d12  # noqa: E402  (puts tools/worldmodel on sys.path; brings bench_v1, wm_data, calcore)

bv, cc, wm_data = d12.bv, d12.cc, d12.wm_data
VARS = ("alive", "food_stock", "wood_stock", "settlements_alive", "deadwood_total", "food_on_bushes_total",
        "adults_working")
DAYS = (30, 60, 90, 118)
MEAN_DAYS = (60, 90, 118)
N_BOOT = 1000
FIRST = 24  # rows 0..23 are excluded from S_j (bench_v1.split_std uses ep.v[24:])
MEMBER_RE = re.compile(r"^(?P<scene>.+)_s(?P<seed>\d+)_none_r(?P<rep>\d+)$")


# ------------------------------------------------------------------ decomposition --------------------------------
def decompose(y) -> dict:
    """y[W, R, ...] -> {"W", "B", "PS"} over the trailing axes (floats for a 2-D y).

    W = mean_w var_r(y) (ddof 1); B = max(0, var_w(mean_r y) (ddof 1) - W / R); PS = W / (W + B), NaN when both 0."""
    y = np.asarray(y, dtype=np.float64)
    if y.ndim < 2 or y.shape[0] < 2 or y.shape[1] < 2:
        raise ValueError(f"decompose needs y[W >= 2, R >= 2, ...], got shape {y.shape}")
    r = y.shape[1]
    within = y.var(axis=1, ddof=1).mean(axis=0)
    between = np.maximum(y.mean(axis=1).var(axis=0, ddof=1) - within / r, 0.0)
    total = within + between
    with np.errstate(invalid="ignore", divide="ignore"):
        ps = np.where(total > 0, within / np.where(total > 0, total, 1.0), np.nan)
    if y.ndim == 2:
        return {"W": float(within), "B": float(between), "PS": float(ps)}
    return {"W": within, "B": between, "PS": ps}


def _mean_days(x: np.ndarray, days) -> np.ndarray:
    """x[..., D, J] -> [..., J]: nanmean over the MEAN_DAYS positions of `days` (NaN when all are NaN)."""
    idx = [list(days).index(d) for d in MEAN_DAYS if d in days]
    with warnings.catch_warnings():
        warnings.simplefilter("ignore", RuntimeWarning)
        return np.nanmean(x[..., idx, :], axis=-2)


def scene_stats(y: np.ndarray, s: np.ndarray, days=DAYS) -> dict:
    """y[W, R, D, J], s[J] -> W, B, PS, PSD [D, J] and PS_mean, PSD_mean [J]."""
    dec = decompose(y)
    with np.errstate(invalid="ignore", divide="ignore"):
        psd = np.where(s > 0, np.sqrt(dec["W"]) / np.where(s > 0, s, 1.0), np.nan)
    return {"W": dec["W"], "B": dec["B"], "PS": dec["PS"], "PSD": psd,
            "PS_mean": _mean_days(dec["PS"], days), "PSD_mean": _mean_days(psd, days)}


def _draws(n_worlds: int, n_boot: int) -> np.ndarray:
    return np.random.default_rng(0).integers(0, n_worlds, size=(n_boot, n_worlds))


def _percentiles(samples: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    with warnings.catch_warnings():
        warnings.simplefilter("ignore", RuntimeWarning)
        return np.nanpercentile(samples, 5, axis=0), np.nanpercentile(samples, 95, axis=0)


def _boot_samples(y: np.ndarray, s: np.ndarray, days, draws: np.ndarray) -> dict:
    """Per resample: PS, PSD [B, D, J] and their day means [B, J]; S_j fixed at the full-sample value."""
    out = {k: [] for k in ("PS", "PSD", "PS_mean", "PSD_mean")}
    for draw in draws:
        st = scene_stats(y[draw], s, days)
        for k in out:
            out[k].append(st[k])
    return {k: np.array(v) for k, v in out.items()}


def bootstrap_scene(y: np.ndarray, s: np.ndarray, days=DAYS, n_boot: int = N_BOOT) -> dict:
    """90 % intervals ([lo, hi] arrays) of PS, PSD per day and of their day means, worlds resampled."""
    samples = _boot_samples(y, s, days, _draws(y.shape[0], n_boot))
    return {k: _percentiles(v) for k, v in samples.items()}


def paired_bootstrap(ya: np.ndarray, sa: np.ndarray, yb: np.ndarray, sb: np.ndarray, days=DAYS,
                     n_boot: int = N_BOOT) -> dict:
    """A - B with worlds paired: ya[W, Ra, D, J] and yb[W, Rb, D, J] hold the same worlds in the same order; every
    resample applies the same world indices to both. Returns point differences and [lo, hi] per statistic."""
    if ya.shape[0] != yb.shape[0]:
        raise ValueError("paired bootstrap needs the same worlds in both scenes")
    a, b = scene_stats(ya, sa, days), scene_stats(yb, sb, days)
    draws = _draws(ya.shape[0], n_boot)
    sa_, sb_ = _boot_samples(ya, sa, days, draws), _boot_samples(yb, sb, days, draws)
    keys = ("PS", "PSD", "PS_mean", "PSD_mean")
    return {"point": {k: a[k] - b[k] for k in keys}, "ci": {k: _percentiles(sa_[k] - sb_[k]) for k in keys}}


# ------------------------------------------------------------------ loading ---------------------------------------
def parse_member(name: str) -> tuple[int, int] | None:
    """`game_v18_s901003_none_r2` -> (901003, 2); None for anything else (variants, `.tmp`, unforked)."""
    m = MEMBER_RE.match(name)
    return None if m is None else (int(m.group("seed")), int(m.group("rep")))


def group_members(path) -> dict[int, dict[int, Path]]:
    """{seed: {replicate: directory}} of the complete factual members (meta.json present) under `path`."""
    groups: dict[int, dict[int, Path]] = {}
    for p in sorted(Path(path).iterdir()):
        key = parse_member(p.name)
        if key is None or not (p / "meta.json").exists():
            continue
        groups.setdefault(key[0], {})[key[1]] = p
    return groups


def episode_vars(ep) -> np.ndarray:
    """v[T, len(VARS)] of one loaded episode (global.npy and settlements.npy loaded)."""
    base = bv.episode_arrays(ep)["v"]  # bench_v1.VARS order; food/wood = nansum over settlement slots
    t = base.shape[0]
    if ep.arrays["settlements.npy"].shape[1]:
        members = np.nan_to_num(ep.column("settlements.npy", "members"), nan=0.0)
        sa = (members >= 1).sum(axis=1).astype(np.float64)
    else:
        sa = np.zeros(t)
    cols = {name: base[:, bv.VARS.index(name)] for name in bv.VARS}
    cols["settlements_alive"] = sa
    return np.stack([cols[name] for name in VARS], axis=1)


@dataclass
class SceneData:
    name: str
    path: Path
    seeds: list[int]
    r: int
    y: np.ndarray            # [W, R, D, J]
    s: np.ndarray            # [J]
    days: tuple
    dropped: dict = field(default_factory=dict)


def load_scene(name: str, path, days=DAYS, seeds=None, verify: bool = True) -> SceneData:
    """Group, load and reduce one scene. R = the largest replicate count found; worlds without replicates 0..R-1
    are dropped (and listed). Each member's meta must agree with its name (seed, noise_fork.replicate)."""
    path = Path(path)
    groups = group_members(path)
    if seeds is not None:
        groups = {s: g for s, g in groups.items() if s in set(seeds)}
    if not groups:
        raise SystemExit(f"{path}: no complete `*_s<seed>_none_r<k>` members")
    r = max(len(g) for g in groups.values())
    dropped = {str(s): sorted(g) for s, g in groups.items() if sorted(g) != list(range(r))}
    keep = sorted(s for s in groups if str(s) not in dropped)
    if len(keep) < 2 or r < 2:
        raise SystemExit(f"{path}: need >= 2 complete worlds with >= 2 replicates (worlds {len(keep)}, R {r})")
    y = []
    rows24 = []
    scene_names = set()
    for seed in keep:
        reps = []
        for k in range(r):
            ep = wm_data.load_episode(groups[seed][k], verify=verify, arrays=["global.npy", "settlements.npy"])
            fork = ep.meta.get("noise_fork") or {}
            if int(ep.meta["seed"]) != seed or int(fork.get("replicate", -1)) != k:
                raise SystemExit(f"{groups[seed][k]}: meta seed / noise_fork.replicate differ from the name")
            scene_names.add(ep.meta["scene"])
            v = episode_vars(ep)
            reps.append(d12.day_means(v, days))  # [D, J]; rows of display day d (calcore_wm11.display_day)
            rows24.append(v[FIRST:])
        y.append(reps)
    s = np.vstack(rows24).std(axis=0)  # bench_v1.split_std: std (ddof 0) over rows >= 24 of every episode
    if len(scene_names) != 1:
        raise SystemExit(f"{path}: members of several scenes {sorted(scene_names)}")
    return SceneData(name, path, keep, r, np.array(y, dtype=np.float64), s, tuple(days), dropped)


# ------------------------------------------------------------------ unforked check --------------------------------
def _one_match(path: Path, pattern: str) -> Path:
    hits = sorted(p for p in path.glob(pattern) if (p / "meta.json").exists())
    if len(hits) != 1:
        raise SystemExit(f"{path}: expected one `{pattern}`, found {[p.name for p in hits]}")
    return hits[0]


def arrays_identical(a: np.ndarray, b: np.ndarray) -> bool:
    """Same dtype, shape and bytes, and np.array_equal with NaN = NaN for float arrays."""
    if a.dtype != b.dtype or a.shape != b.shape:
        return False
    equal = np.array_equal(a, b, equal_nan=a.dtype.kind in "fc")
    return bool(equal and a.tobytes() == b.tobytes())


def check_unforked(forked, unforked, seeds) -> dict:
    """Replicate 0 of the forked family vs the unforked recording, per seed, over every array present in both."""
    out = {"forked": str(forked), "unforked": str(unforked), "seeds": {}}
    for seed in seeds:
        fp = _one_match(Path(forked), f"*_s{seed}_none_r0")
        up = _one_match(Path(unforked), f"*_s{seed}_none")
        fe, ue = wm_data.load_episode(fp), wm_data.load_episode(up)
        common = sorted(set(fe.arrays) & set(ue.arrays))
        differ = [n for n in common if not arrays_identical(fe.arrays[n], ue.arrays[n])]
        out["seeds"][str(seed)] = {
            "forked_member": fp.name, "unforked_member": up.name, "compared": common, "differ": differ,
            "only_forked": sorted(set(fe.arrays) - set(ue.arrays)),
            "only_unforked": sorted(set(ue.arrays) - set(fe.arrays)),
            "fingerprint_equal": fe.meta.get("final_fingerprint") == ue.meta.get("final_fingerprint"),
            "interventions_equal": fe.interventions == ue.interventions,
            "identical": not differ and bool(common) and fe.meta.get("final_fingerprint") == ue.meta.get("final_fingerprint"),
        }
    out["all_identical"] = all(v["identical"] for v in out["seeds"].values())
    return out


# ------------------------------------------------------------------ report ----------------------------------------
def _num(x):
    x = float(x)
    return None if not np.isfinite(x) else x


def _per_var(arr: np.ndarray, j: int, days) -> dict:
    return {str(d): _num(arr[i, j]) for i, d in enumerate(days)}


def _ci_block(ci: dict, j: int, days) -> dict:
    lo, hi = ci["PS"]
    plo, phi = ci["PSD"]
    return {"PS": {**{str(d): [_num(lo[i, j]), _num(hi[i, j])] for i, d in enumerate(days)},
                   "mean": [_num(ci["PS_mean"][0][j]), _num(ci["PS_mean"][1][j])]},
            "PSD": {**{str(d): [_num(plo[i, j]), _num(phi[i, j])] for i, d in enumerate(days)},
                    "mean": [_num(ci["PSD_mean"][0][j]), _num(ci["PSD_mean"][1][j])]}}


def scene_block(sc: SceneData, n_boot: int) -> dict:
    st = scene_stats(sc.y, sc.s, sc.days)
    ci = bootstrap_scene(sc.y, sc.s, sc.days, n_boot)
    out = {"dir": str(sc.path), "W": len(sc.seeds), "R": sc.r, "seeds": sc.seeds, "dropped": sc.dropped,
           "S": {name: _num(sc.s[j]) for j, name in enumerate(VARS)}, "vars": {}}
    for j, name in enumerate(VARS):
        out["vars"][name] = {"W": _per_var(st["W"], j, sc.days), "B": _per_var(st["B"], j, sc.days),
                             "PS": _per_var(st["PS"], j, sc.days), "PSD": _per_var(st["PSD"], j, sc.days),
                             "PS_mean": _num(st["PS_mean"][j]), "PSD_mean": _num(st["PSD_mean"][j]),
                             "ci90": _ci_block(ci, j, sc.days)}
    return out


def diff_block(a: SceneData, b: SceneData, n_boot: int) -> dict:
    common = sorted(set(a.seeds) & set(b.seeds))
    if len(common) < 2:
        raise SystemExit(f"diff {a.name} - {b.name}: fewer than 2 shared worlds")
    ia = [a.seeds.index(s) for s in common]
    ib = [b.seeds.index(s) for s in common]
    pb = paired_bootstrap(a.y[ia], a.s, b.y[ib], b.s, a.days, n_boot)
    out = {"W": len(common), "seeds": common, "vars": {}}
    for j, name in enumerate(VARS):
        out["vars"][name] = {"PS": _per_var(pb["point"]["PS"], j, a.days),
                             "PSD": _per_var(pb["point"]["PSD"], j, a.days),
                             "PS_mean": _num(pb["point"]["PS_mean"][j]), "PSD_mean": _num(pb["point"]["PSD_mean"][j]),
                             "ci90": _ci_block(pb["ci"], j, a.days)}
    return out


def fmt(x) -> str:
    return "—" if x is None else f"{x:.3f}"


def fmt_ci(ci) -> str:
    return "[—]" if ci is None or None in ci else f"[{ci[0]:.3f}, {ci[1]:.3f}]"


def _table(block: dict, days, stat: str) -> list[str]:
    head = "| Variable | " + " | ".join(f"d{d}" for d in days) + " | mean 60/90/118 [90 %] |"
    lines = [head, "|---|" + "---|" * (len(days) + 1)]
    for name in VARS:
        v = block["vars"][name]
        lines.append(f"| {name} | " + " | ".join(fmt(v[stat][str(d)]) for d in days)
                     + f" | {fmt(v[stat + '_mean'])} {fmt_ci(v['ci90'][stat]['mean'])} |")
    return lines


def markdown(out: dict) -> str:
    days = out["days"]
    lines = ["# Replicate measure (paths_p13.py)", "",
             f"PS = W / (W + B), PSD = sqrt(W) / S_j; days {days}; intervals: {out['n_boot']} world resamples, "
             f"5–95 %, S_j fixed; diffs A − B paired over shared worlds.", ""]
    for name, b in out["scenes"].items():
        lines += [f"## {name} — W {b['W']}, R {b['R']}, seeds {b['seeds'][0]}..{b['seeds'][-1]}"
                  + (f", dropped {b['dropped']}" if b["dropped"] else ""), "", "PS", ""]
        lines += _table(b, days, "PS") + ["", "PSD", ""] + _table(b, days, "PSD") + [""]
    for label, b in out["diffs"].items():
        lines += [f"## {label} — paired, W {b['W']}", "", "ΔPS", ""]
        lines += _table(b, days, "PS") + ["", "ΔPSD", ""] + _table(b, days, "PSD") + [""]
    chk = out.get("check_unforked")
    if chk:
        lines += ["## Replicate 0 vs unforked", ""]
        for seed, c in chk["seeds"].items():
            lines.append(f"- seed {seed}: {'identical' if c['identical'] else 'DIFFERS'} ({len(c['compared'])} arrays"
                         f"{', differ ' + ', '.join(c['differ']) if c['differ'] else ''}; fingerprint "
                         f"{'equal' if c['fingerprint_equal'] else 'differs'})")
        lines.append("")
    return "\n".join(lines) + "\n"


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--scene", action="append", default=[], metavar="NAME=DIR")
    ap.add_argument("--diff", action="append", default=[], metavar="A:B", help="report A - B, paired over worlds")
    ap.add_argument("--days", default=",".join(str(d) for d in DAYS))
    ap.add_argument("--boot", type=int, default=N_BOOT)
    ap.add_argument("--check-unforked", nargs=2, metavar=("FORKED_DIR", "UNFORKED_DIR"))
    ap.add_argument("--seeds", help="seeds of --check-unforked, comma-separated")
    ap.add_argument("--out", required=True)
    args = ap.parse_args(argv)
    if not args.scene and not args.check_unforked:
        ap.error("give --scene and/or --check-unforked")
    if args.check_unforked and not args.seeds:
        ap.error("--check-unforked needs --seeds")
    t0 = time.time()
    days = tuple(int(d) for d in args.days.split(","))
    target = Path(args.out)
    target.mkdir(parents=True, exist_ok=True)
    out = {"vars": list(VARS), "days": list(days), "mean_days": list(MEAN_DAYS), "n_boot": args.boot,
           "ci": "90 % (5th, 95th percentile), worlds resampled with numpy.random.default_rng(0); S_j fixed",
           "scenes": {}, "diffs": {}}
    scenes = {}
    for spec in args.scene:
        name, _, path = spec.partition("=")
        if not path:
            ap.error(f"--scene {spec}: expected NAME=DIR")
        scenes[name] = load_scene(name, path, days)
        out["scenes"][name] = scene_block(scenes[name], args.boot)
        b = out["scenes"][name]
        print(name, f"W {b['W']} R {b['R']}", "PS mean",
              {n: fmt(b["vars"][n]["PS_mean"]) for n in VARS}, f"{time.time() - t0:.0f}s", flush=True)
    for pair in args.diff:
        a, _, b = pair.partition(":")
        if a not in scenes or b not in scenes:
            ap.error(f"--diff {pair}: both scenes must be given with --scene")
        out["diffs"][f"{a} - {b}"] = diff_block(scenes[a], scenes[b], args.boot)
    if args.check_unforked:
        seeds = [int(s) for s in args.seeds.split(",")]
        out["check_unforked"] = check_unforked(args.check_unforked[0], args.check_unforked[1], seeds)
        for seed, c in out["check_unforked"]["seeds"].items():
            print(f"replicate 0 vs unforked, seed {seed}: identical={c['identical']} differ={c['differ']} "
                  f"compared={len(c['compared'])}", flush=True)
    out["seconds"] = time.time() - t0
    (target / "paths.json").write_text(json.dumps(out, indent=1, sort_keys=True) + "\n")
    (target / "paths.md").write_text(markdown(out))
    print(f"done {out['seconds']:.0f}s -> {target}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
