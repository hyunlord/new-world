#!/usr/bin/env python3
"""Write intervention schedules and record WorldSim datasets (Pass 7). numpy only.

usage:
  make_dataset.py z6 [--root WM_DATA_ROOT] [--scene game_v13] [--no-run] [--workers 4]
  make_dataset.py v0|v1 [--root WM_DATA_ROOT] [--splits A_obs,B_act,...] [--no-run] [--workers 4]
                        [--docs docs/worldmodel]
  make_dataset.py manifest|manifest_v1 [--root WM_DATA_ROOT] [--docs docs/worldmodel]
  make_dataset.py families --name NAME --scene game_v15 --seeds a..b --days N --fork-tick T --replicates R
                           --schedule-dir DIR --family-splits train=a..b,val=c..d,test=e..f
                           [--agents] [--every 60] [--root WM_DATA_ROOT] [--no-run] [--workers 4]

`z6` writes the Z6 schedules (seeds 0-7 x seven kinds, one intervention at
one-based day 23 12:00 = tick 31,920) under ROOT/z6/schedules and records them
with the factual episodes under ROOT/z6/episodes (game_v13, 30 days, every 60).
With `--scene game_v14` (Pass 8 section 8.1) the same schedules and episodes go
under ROOT/z6_v14.

`v0` writes the schedules of dataset v0 (PASS7_GOAL.md section 12.1), records
every split under ROOT/v0/<split> with `export.sh --resume` (a finished episode
is skipped when its meta hashes verify and its meta records the same scene,
seed, days, every, agent flag, schedule and git sha, so a stopped run resumes
within one commit), then writes
`dataset_v0_manifest.json` and `DATASET_v0.md` into --docs.

`v1` does the same for dataset v1 (PASS8_GOAL.md section 11.1: game_v14,
recorder schema 2, the windfall C_cf variant, OOD_v13) under ROOT/v1 and writes
`dataset_v1_manifest.json` and `DATASET_v1.md`; v0's files are not touched.

`families` (Pass 9 section 10, the format dataset v2 will use) records branch
families: for every root seed, the factual and every schedule of
`--schedule-dir` (`s<seed>_<id>.json`), each as R members that share the
prefix up to the fork tick and use noise replicate k after it (k = 0 is the
canonical run). A family is one directory `ROOT/NAME/<scene>_s<seed>/` holding
its members `<scene>_s<seed>_<id>_r<k>`; `wm_data.load_family` loads it.
`ROOT/NAME/families_manifest.json` lists every family (root seed, split, fork
tick, replicates, members with file hashes). Splits are assigned per root seed
(`--family-splits`, every seed in exactly one split), so the members of one
family never cross splits.

Days are one-based display days; the tick of day D at minute m is
(D - 1) x 1440 + m - 480.
"""
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

import numpy as np

from wm_data import sha256_file

ROOT_DIR = Path(__file__).resolve().parents[2]
DEFAULT_ROOT = Path(os.environ.get("WM_DATA_ROOT", "/Users/rexxa/work/worldsim-wm-data"))
KINDS = ("gift_food", "spoil_food", "gift_wood", "rot_wood", "rain", "cold_snap", "windfall")
TPD, OFFSET = 1440, 480


def tick_of(day: int, minute: int) -> int:
    """Tick of one-based display day `day` at minute-of-day `minute`."""
    return (day - 1) * TPD + minute - OFFSET


# ---------------------------------------------------------------- Z6 -------------------------------------------------
Z6_TICK = tick_of(23, 12 * 60)  # 31,920: the second winter day of year 0
Z6_AMOUNTS = {"gift_food": 60, "spoil_food": 60, "gift_wood": 30, "rot_wood": 30, "rain": 13, "cold_snap": 1, "windfall": 10}
Z6_SEEDS = range(0, 8)


def z6_schedules(out: Path) -> list[Path]:
    out.mkdir(parents=True, exist_ok=True)
    paths = []
    for seed in Z6_SEEDS:
        for kind in KINDS:
            path = out / f"s{seed}_{kind}.json"
            path.write_text(json.dumps([{"tick": Z6_TICK, "kind": kind, "settlement": "home", "amount": Z6_AMOUNTS[kind]}]) + "\n")
            paths.append(path)
    return paths


# ---------------------------------------------------------------- v0 -------------------------------------------------
# B_act: eight interventions a seed, ticks on the hour boundaries from day 2 00:00 to day 118 23:00.
B_FIRST, B_LAST = tick_of(2, 0), tick_of(118, 23 * 60)
B_GRIDS = {
    "gift_food": list(range(20, 121, 10)), "spoil_food": list(range(20, 121, 10)),
    "gift_wood": list(range(10, 81, 10)), "rot_wood": list(range(10, 81, 10)),
    "rain": list(range(6, 25)), "cold_snap": [1, 2, 3], "windfall": list(range(5, 41, 5)),
}
# C_cf: four variants against the factual, all in the first winter (days 22-28).
C_VARIANTS = {
    "gift_wood": {"tick": tick_of(21, 18 * 60), "kind": "gift_wood", "amount": 60},   # 29,400: the last autumn evening
    "rot_wood": {"tick": tick_of(21, 18 * 60), "kind": "rot_wood", "amount": 60},
    "rain": {"tick": tick_of(24, 18 * 60), "kind": "rain", "amount": 13},             # 33,720
    "cold_snap": {"tick": tick_of(25, 18 * 60), "kind": "cold_snap", "amount": 2},    # 35,160
}

# (split, scene, seeds, days, roles {role: seeds}, agent seeds, schedule kind)
SPLITS = {
    "A_obs": dict(scene="game_v13", seeds=range(0, 128), days=120,
                  roles={"train": range(0, 96), "val": range(96, 112), "test": range(112, 128)},
                  agents=[*range(0, 32), *range(96, 100), *range(112, 120)], schedules=None),
    "B_act": dict(scene="game_v13", seeds=range(1000, 1064), days=120,
                  roles={"train": range(1000, 1048), "val": range(1048, 1056), "test": range(1056, 1064)},
                  agents=list(range(1056, 1064)), schedules="b_act"),
    "C_cf": dict(scene="game_v13", seeds=range(2000, 2032), days=35,
                 roles={"val": range(2000, 2008), "test": range(2008, 2032)}, agents=[], schedules="c_cf"),
    "OOD_v12": dict(scene="game_v12", seeds=range(3000, 3032), days=120, roles={"test": range(3000, 3032)},
                    agents=[], schedules=None),
    "OOD_long": dict(scene="game_v13", seeds=range(4000, 4016), days=240, roles={"test": range(4000, 4016)},
                     agents=[], schedules=None),
}


# Dataset v1 (Pass 8 section 11.1): game_v14, disjoint seeds, C_cf adds a windfall.
C_VARIANTS_V1 = {**C_VARIANTS,
                 "windfall": {"tick": tick_of(23, 12 * 60), "kind": "windfall", "amount": 20}}  # 31,920
SPLITS_V1 = {
    "A_obs": dict(scene="game_v14", seeds=range(10000, 10128), days=120,
                  roles={"train": range(10000, 10096), "val": range(10096, 10112), "test": range(10112, 10128)},
                  agents=[*range(10000, 10032), *range(10096, 10100), *range(10112, 10120)], schedules=None),
    "B_act": dict(scene="game_v14", seeds=range(11000, 11064), days=120,
                  roles={"train": range(11000, 11048), "val": range(11048, 11056), "test": range(11056, 11064)},
                  agents=list(range(11056, 11064)), schedules="b_act"),
    "C_cf": dict(scene="game_v14", seeds=range(12000, 12032), days=35,
                 roles={"val": range(12000, 12008), "test": range(12008, 12032)}, agents=[], schedules="c_cf"),
    "OOD_v13": dict(scene="game_v13", seeds=range(13000, 13032), days=120, roles={"test": range(13000, 13032)},
                    agents=[], schedules=None),
    "OOD_long": dict(scene="game_v14", seeds=range(14000, 14016), days=240, roles={"test": range(14000, 14016)},
                     agents=[], schedules=None),
}
DATASETS = {"v0": (SPLITS, C_VARIANTS), "v1": (SPLITS_V1, C_VARIANTS_V1)}


def b_act_schedule(seed: int, kinds=KINDS) -> list[dict]:
    """Eight interventions drawn with numpy.random.default_rng(seed): per row a tick,
    then a kind, then an amount; rows sorted by tick, then draw order."""
    rng = np.random.default_rng(seed)
    hours = (B_LAST - B_FIRST) // 60 + 1
    rows = []
    for draw in range(8):
        tick = B_FIRST + 60 * int(rng.integers(0, hours))
        kind = kinds[int(rng.integers(0, len(kinds)))]
        grid = B_GRIDS[kind]
        amount = grid[int(rng.integers(0, len(grid)))]
        rows.append((tick, draw, {"tick": tick, "kind": kind, "settlement": "home", "amount": amount}))
    return [row for _, _, row in sorted(rows, key=lambda r: (r[0], r[1]))]


def write_schedules(split: str, spec: dict, out: Path, variants: dict = C_VARIANTS) -> None:
    out.mkdir(parents=True, exist_ok=True)
    for seed in spec["seeds"]:
        if spec["schedules"] == "b_act":
            (out / f"s{seed}_b.json").write_text(json.dumps(b_act_schedule(seed)) + "\n")
        elif spec["schedules"] == "c_cf":
            for variant, row in variants.items():
                (out / f"s{seed}_{variant}.json").write_text(
                    json.dumps([{"tick": row["tick"], "kind": row["kind"], "settlement": "home", "amount": row["amount"]}]) + "\n")


def runs(split: str, spec: dict) -> list[tuple[range | list, bool]]:
    """Contiguous seed ranges with the same --agents flag."""
    agents = set(spec["agents"])
    out, start, flag = [], None, None
    seeds = list(spec["seeds"])
    for seed in seeds + [None]:
        f = seed in agents if seed is not None else None
        if seed is None or f != flag:
            if start is not None:
                out.append((range(start, prev + 1), flag))
            start, flag = seed, f
        prev = seed
    return out


def export(args: list[str], log) -> None:
    command = ["bash", str(ROOT_DIR / "tools/worldmodel/export.sh"), "--no-build", *args]
    print("$ " + " ".join(command), file=log, flush=True)
    subprocess.run(command, check=True, stdout=log, stderr=log, cwd=ROOT_DIR)


def role_of(spec: dict, seed: int) -> str:
    for role, seeds in spec["roles"].items():
        if seed in seeds:
            return role
    raise ValueError(f"seed {seed} has no role")


def episode_ids(split: str, spec: dict, variants: dict = C_VARIANTS) -> list[str]:
    if spec["schedules"] == "b_act":
        return ["b"]
    if spec["schedules"] == "c_cf":
        return ["none", *variants]
    return ["none"]


def manifest(root: Path, version: str = "v0") -> dict:
    splits, variants = DATASETS[version]
    episodes, seen = [], {}
    total = 0
    for split, spec in splits.items():
        for seed in spec["seeds"]:
            key = seed
            if key in seen and seen[key] != (split, role_of(spec, seed)):
                raise ValueError(f"seed {seed} appears in {seen[key]} and {(split, role_of(spec, seed))}")
            seen[key] = (split, role_of(spec, seed))
            for eid in episode_ids(split, spec, variants):
                path = root / version / split / f"{spec['scene']}_s{seed}_{eid}"
                if not (path / "meta.json").exists():
                    raise ValueError(f"missing episode {path}")
                meta = json.loads((path / "meta.json").read_text())
                files = {}
                size = 0
                for name in sorted(p.name for p in path.iterdir() if p.is_file() and p.name != "timing.json"):
                    n = (path / name).stat().st_size
                    files[name] = {"sha256": sha256_file(path / name), "size": n}
                    size += n
                    if name in meta["arrays"] and files[name]["sha256"] != meta["arrays"][name]["sha256"]:
                        raise ValueError(f"{path / name}: hash differs from meta.json")
                total += size
                episodes.append({"split": split, "role": role_of(spec, seed), "scene": spec["scene"], "seed": seed,
                                 "schedule_id": eid, "path": f"{version}/{split}/{path.name}", "agents": meta["agents_recorded"],
                                 "days": meta["days"], "rows": meta["rows"], "git_sha": meta["git_sha"],
                                 "recorder_schema": meta["schema_version"], "files": files, "size": size})
    if version == "v0":
        for e in episodes:
            del e["recorder_schema"]  # v0's manifest format stays as committed
    return {"schema_version": 1, "dataset": f"worldsim-wm {version}", "episodes": episodes, "total_size": total,
            "episode_count": len(episodes)}


def dataset_doc(man: dict) -> str:
    lines = ["# WorldSim dataset v0", "",
             "Recorded worlds for the first world-model benchmark (`BENCH_v0.md`). Format: `README.md` (recorder schema 1). "
             "The files live outside git under `WM_DATA_ROOT` (`/Users/rexxa/work/worldsim-wm-data/v0/<split>/<episode>`); "
             "`dataset_v0_manifest.json` lists every episode with the sha256 and size of every file.", "",
             f"Episodes: {man['episode_count']}; total size {man['total_size'] / 2**20:.1f} MiB; recorder built from "
             f"`{sorted({e['git_sha'] for e in man['episodes']})}`.", "",
             "All splits use `--every 60` (a row per simulated hour). Days are one-based display days; the tick of day D at "
             "minute m is (D − 1) × 1440 + m − 480.", "",
             "| Split | Scene | Seeds | Days | Interventions | Roles | `--agents` | Episodes | Size (MiB) |",
             "|---|---|---|---:|---|---|---|---:|---:|"]
    descr = {"A_obs": "none", "B_act": "8 random each (`s<seed>_b.json`)", "C_cf": "factual + 4 variants each",
             "OOD_v12": "none", "OOD_long": "none"}
    for split, spec in SPLITS.items():
        eps = [e for e in man["episodes"] if e["split"] == split]
        roles = ", ".join(f"{r} {min(s)}–{max(s)}" for r, s in spec["roles"].items())
        agents = ", ".join(f"{r.start}–{r.stop - 1}" for r, flag in runs(split, spec) if flag) or "none"
        lines.append(f"| {split} | {spec['scene']} | {min(spec['seeds'])}–{max(spec['seeds'])} | {spec['days']} | {descr[split]} | "
                     f"{roles} | {agents} | {len(eps)} | {sum(e['size'] for e in eps) / 2**20:.1f} |")
    lines += ["", "## Schedules", "",
              "- **B_act** (`make_dataset.py`, `b_act_schedule`): `numpy.random.default_rng(seed)`; per intervention a tick "
              f"uniform over the hour boundaries from day 2 00:00 (tick {B_FIRST}) to day 118 23:00 (tick {B_LAST}), then a kind "
              "uniform over the seven, then an amount uniform on its grid — food 20–120 step 10, wood 10–80 step 10, rain 6–24 "
              "hours, cold snap 1–3 nights, windfall 5–40 step 5; settlement `home`; rows sorted by tick, then draw order.",
              "- **C_cf** variants, each against the same seed's factual episode `none`: " + "; ".join(
                  f"`{v}` {r['kind']} {r['amount']} at tick {r['tick']}" for v, r in C_VARIANTS.items()) + ". "
              "`gift_wood`/`rot_wood` act on the last autumn evening (day 21 18:00), `rain` on day 24 18:00, `cold_snap` "
              "(2 nights) on day 25 18:00.",
              "", "## Splits and leakage", "",
              "No seed appears in two splits or two roles (checked when the manifest is written). OOD_v12 is the Pass 6 "
              "ruleset (no winter severity, unbounded claims): a controlled distribution shift. OOD_long runs 240 days "
              "(eight winters) where training saw 120.", "",
              "## Reproducing", "",
              "```bash", "python3 tools/worldmodel/make_dataset.py v0          # resumable: verified episodes are skipped",
              "python3 tools/worldmodel/bench_v0.py --root $WM_DATA_ROOT --out docs/worldmodel", "```", ""]
    return "\n".join(lines)


def dataset_doc_v1(man: dict) -> str:
    lines = ["# WorldSim dataset v1", "",
             "Recorded worlds for WM-Bench v1 (`BENCH_v1.md`), Pass 8. Recorder schema 2 (`README.md`): the world's "
             "weather — `winter_severity`, the night's burn `night_burn_per_hour`, the day's deadwood fall "
             "`winter_fall_today` — and the values that encode it (`wood_target`, `latent.npy`) are marked `hidden`; "
             "`meta.json` lists `observable_fields`. The files live outside git under `WM_DATA_ROOT` "
             "(`/Users/rexxa/work/worldsim-wm-data/v1/<split>/<episode>`); `dataset_v1_manifest.json` lists every episode "
             "with the sha256 and size of every file.", "",
             f"Episodes: {man['episode_count']}; total size {man['total_size'] / 2**20:.1f} MiB; recorder built from "
             f"`{sorted({e['git_sha'] for e in man['episodes']})}`.", "",
             "All splits use `--every 60` (a row per simulated hour). Days are one-based display days; the tick of day D at "
             "minute m is (D − 1) × 1440 + m − 480.", "",
             "| Split | Scene | Seeds | Days | Interventions | Roles | `--agents` | Episodes | Size (MiB) |",
             "|---|---|---|---:|---|---|---|---:|---:|"]
    descr = {"A_obs": "none", "B_act": "8 random each (`s<seed>_b.json`, v0's sampler)", "C_cf": "factual + 5 variants each",
             "OOD_v13": "none", "OOD_long": "none"}
    for split, spec in SPLITS_V1.items():
        eps = [e for e in man["episodes"] if e["split"] == split]
        roles = ", ".join(f"{r} {min(x)}–{max(x)}" for r, x in spec["roles"].items())
        agents = ", ".join(f"{r.start}–{r.stop - 1}" for r, flag in runs(split, spec) if flag) or "none"
        lines.append(f"| {split} | {spec['scene']} | {min(spec['seeds'])}–{max(spec['seeds'])} | {spec['days']} | {descr[split]} | "
                     f"{roles} | {agents} | {len(eps)} | {sum(e['size'] for e in eps) / 2**20:.1f} |")
    lines += ["", "## Schedules", "",
              "- **B_act**: v0's sampler (`b_act_schedule`, `numpy.random.default_rng(seed)`), eight interventions a seed "
              f"between day 2 00:00 (tick {B_FIRST}) and day 118 23:00 (tick {B_LAST}).",
              "- **C_cf** variants, each against the same seed's factual episode `none`: " + "; ".join(
                  f"`{v}` {r['kind']} {r['amount']} at tick {r['tick']}" for v, r in C_VARIANTS_V1.items()) + ". "
              "`windfall` (new in v1) lands 20 units on the deadwood spots at day 23 12:00, in the first winter; under "
              "`game_v14` it lands on the snow (spots fill to their max) instead of stopping at the winter cap.",
              "", "## Splits and leakage", "",
              "No seed appears in two splits or two roles (checked when the manifest is written), and v1's seeds "
              "(10000–14015) are disjoint from dataset v0 (0–4015) and from the probe's tuning, held-out and fresh seeds "
              "(0–191). OOD_v13 is Pass 7's ruleset (no night or day draws, windfall under the snow cap): a controlled "
              "distribution shift. OOD_long runs 240 days (eight winters).", "",
              "## Reproducing", "",
              "```bash", "python3 tools/worldmodel/make_dataset.py v1          # resumable: matching episodes are skipped",
              "python3 tools/worldmodel/bench_v1.py --root $WM_DATA_ROOT --out docs/worldmodel", "```", ""]
    return "\n".join(lines)


# ---------------------------------------------------------------- branch families (Pass 9) -------------------------
def parse_range(text: str) -> range:
    a, b = text.split("..")
    return range(int(a), int(b))


def family_splits(text: str, seeds: range) -> dict[int, str]:
    """Split of every root seed from `name=a..b,...`; each seed in exactly one split."""
    out: dict[int, str] = {}
    for item in text.split(","):
        name, span = item.split("=")
        for seed in parse_range(span):
            if seed in out:
                raise ValueError(f"seed {seed} is in splits {out[seed]} and {name}")
            out[seed] = name
    missing = [s for s in seeds if s not in out]
    if missing:
        raise ValueError(f"seeds without a split: {missing}")
    return {s: out[s] for s in seeds}


def family_schedule_ids(schedule_dir: Path, seed: int) -> list[str]:
    prefix = f"s{seed}_"
    return sorted(p.name[len(prefix):-len(".json")] for p in schedule_dir.glob(f"{prefix}*.json"))


def families_manifest(root: Path, name: str, scene: str, splits: dict[int, str], fork_tick: int, replicates: int,
                      schedule_dir: Path) -> dict:
    """Every family of ROOT/NAME with its split and members (hashes checked against meta.json)."""
    families = []
    for seed, split in splits.items():
        family_dir = root / name / f"{scene}_s{seed}"
        members = []
        for eid in ["none", *family_schedule_ids(schedule_dir, seed)]:
            for k in range(replicates):
                path = family_dir / f"{scene}_s{seed}_{eid}_r{k}"
                meta = json.loads((path / "meta.json").read_text())
                if meta["noise_fork"] != {"tick": fork_tick, "replicate": k} or meta["seed"] != seed:
                    raise ValueError(f"{path}: noise_fork or seed differs from the family")
                files = {}
                for file in sorted(p.name for p in path.iterdir() if p.is_file() and p.name != "timing.json"):
                    files[file] = sha256_file(path / file)
                    if file in meta["arrays"] and files[file] != meta["arrays"][file]["sha256"]:
                        raise ValueError(f"{path / file}: hash differs from meta.json")
                members.append({"schedule_id": eid, "replicate": k, "path": f"{name}/{family_dir.name}/{path.name}",
                                "files": files})
        families.append({"root_seed": seed, "split": split, "scene": scene, "fork_tick": fork_tick,
                         "replicates": replicates, "path": f"{name}/{family_dir.name}", "members": members})
    return {"schema_version": 1, "dataset": f"worldsim-wm families {name}", "families": families,
            "family_count": len(families), "member_count": sum(len(f["members"]) for f in families)}


def run_families(args, log) -> int:
    seeds = parse_range(args.seeds)
    splits = family_splits(args.family_splits, seeds)
    out = args.root / args.name
    if not args.no_run:
        for seed in seeds:
            extra = ["--agents"] if args.agents else []
            export(["--scene", args.scene, "--seeds", f"{seed}..{seed + 1}", "--days", str(args.days), "--every", str(args.every),
                    *extra, "--schedule-dir", str(args.schedule_dir), "--include-factual", "--fork-tick", str(args.fork_tick),
                    "--replicates", str(args.replicates), "--resume", "--workers", str(args.workers),
                    "--out", str(out / f"{args.scene}_s{seed}")], log)
    man = families_manifest(args.root, args.name, args.scene, splits, args.fork_tick, args.replicates, args.schedule_dir)
    (out / "families_manifest.json").write_text(json.dumps(man, indent=1, sort_keys=True) + "\n")
    print(f"families: {man['family_count']} families, {man['member_count']} members")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("command", choices=("z6", "v0", "manifest", "v1", "manifest_v1", "families"))
    parser.add_argument("--root", type=Path, default=DEFAULT_ROOT)
    parser.add_argument("--splits", default=",".join(SPLITS))
    parser.add_argument("--workers", type=int, default=4)
    parser.add_argument("--no-run", action="store_true")
    parser.add_argument("--docs", type=Path, default=ROOT_DIR / "docs/worldmodel")
    parser.add_argument("--scene", default="game_v13", help="z6 only: the scene (game_v14 records under ROOT/z6_v14)")
    parser.add_argument("--name", help="families: dataset directory under ROOT")
    parser.add_argument("--seeds", help="families: root seeds a..b (half-open)")
    parser.add_argument("--days", type=int, help="families: display days")
    parser.add_argument("--every", type=int, default=60, help="families: ticks between rows")
    parser.add_argument("--fork-tick", type=int, help="families: fork tick")
    parser.add_argument("--replicates", type=int, help="families: members per schedule (R)")
    parser.add_argument("--schedule-dir", type=Path, help="families: s<seed>_<id>.json schedules")
    parser.add_argument("--family-splits", help="families: name=a..b,... (per root seed)")
    parser.add_argument("--agents", action="store_true", help="families: record agents")
    args = parser.parse_args(argv)
    log = sys.stdout
    if args.command == "families":
        for name in ("name", "seeds", "days", "fork_tick", "replicates", "schedule_dir", "family_splits"):
            if getattr(args, name) is None:
                parser.error(f"families needs --{name.replace('_', '-')}")
        return run_families(args, log)
    if args.command == "z6":
        z6 = args.root / ("z6" if args.scene == "game_v13" else f"z6_{args.scene.removeprefix('game_')}")
        schedules = z6 / "schedules"
        z6_schedules(schedules)
        if not args.no_run:
            export(["--scene", args.scene, "--seeds", f"{Z6_SEEDS.start}..{Z6_SEEDS.stop}", "--days", "30", "--every", "60",
                    "--schedule-dir", str(schedules), "--include-factual", "--resume", "--workers", str(args.workers),
                    "--out", str(z6 / "episodes")], log)
        return 0
    version = "v1" if args.command in ("v1", "manifest_v1") else "v0"
    splits, variants = DATASETS[version]
    if args.command in ("v0", "v1"):
        chosen = args.splits.split(",") if args.splits != ",".join(SPLITS) else list(splits)
        for split in chosen:
            spec = splits[split]
            sched = args.root / version / "schedules" / split
            if spec["schedules"]:
                write_schedules(split, spec, sched, variants)
            if args.no_run:
                continue
            for seeds, agents in runs(split, spec):
                extra = ["--agents"] if agents else []
                if spec["schedules"]:
                    extra += ["--schedule-dir", str(sched)]
                    if spec["schedules"] == "c_cf":
                        extra.append("--include-factual")
                export(["--scene", spec["scene"], "--seeds", f"{seeds.start}..{seeds.stop}", "--days", str(spec["days"]),
                        "--every", "60", *extra, "--resume", "--workers", str(args.workers),
                        "--out", str(args.root / version / split)], log)
        if args.no_run:
            return 0
    man = manifest(args.root, version)
    args.docs.mkdir(parents=True, exist_ok=True)
    (args.docs / f"dataset_{version}_manifest.json").write_text(json.dumps(man, indent=1, sort_keys=True) + "\n")
    (args.docs / f"DATASET_{version}.md").write_text(dataset_doc(man) if version == "v0" else dataset_doc_v1(man))
    print(f"manifest: {man['episode_count']} episodes, {man['total_size'] / 2**20:.1f} MiB")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
