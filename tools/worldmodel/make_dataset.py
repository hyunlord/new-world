#!/usr/bin/env python3
"""Write intervention schedules and record WorldSim datasets (Pass 7). numpy only.

usage:
  make_dataset.py z6 [--root WM_DATA_ROOT] [--scene game_v13] [--no-run] [--workers 4]
  make_dataset.py v0 [--root WM_DATA_ROOT] [--splits A_obs,B_act,...] [--no-run] [--workers 4]
                     [--docs docs/worldmodel]

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


def write_schedules(split: str, spec: dict, out: Path) -> None:
    out.mkdir(parents=True, exist_ok=True)
    for seed in spec["seeds"]:
        if spec["schedules"] == "b_act":
            (out / f"s{seed}_b.json").write_text(json.dumps(b_act_schedule(seed)) + "\n")
        elif spec["schedules"] == "c_cf":
            for variant, row in C_VARIANTS.items():
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


def episode_ids(split: str, spec: dict) -> list[str]:
    if spec["schedules"] == "b_act":
        return ["b"]
    if spec["schedules"] == "c_cf":
        return ["none", *C_VARIANTS]
    return ["none"]


def manifest(root: Path) -> dict:
    episodes, seen = [], {}
    total = 0
    for split, spec in SPLITS.items():
        for seed in spec["seeds"]:
            key = seed
            if key in seen and seen[key] != (split, role_of(spec, seed)):
                raise ValueError(f"seed {seed} appears in {seen[key]} and {(split, role_of(spec, seed))}")
            seen[key] = (split, role_of(spec, seed))
            for eid in episode_ids(split, spec):
                path = root / "v0" / split / f"{spec['scene']}_s{seed}_{eid}"
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
                                 "schedule_id": eid, "path": f"v0/{split}/{path.name}", "agents": meta["agents_recorded"],
                                 "days": meta["days"], "rows": meta["rows"], "git_sha": meta["git_sha"],
                                 "files": files, "size": size})
    return {"schema_version": 1, "dataset": "worldsim-wm v0", "episodes": episodes, "total_size": total,
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


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("command", choices=("z6", "v0", "manifest"))
    parser.add_argument("--root", type=Path, default=DEFAULT_ROOT)
    parser.add_argument("--splits", default=",".join(SPLITS))
    parser.add_argument("--workers", type=int, default=4)
    parser.add_argument("--no-run", action="store_true")
    parser.add_argument("--docs", type=Path, default=ROOT_DIR / "docs/worldmodel")
    parser.add_argument("--scene", default="game_v13", help="z6 only: the scene (game_v14 records under ROOT/z6_v14)")
    args = parser.parse_args(argv)
    log = sys.stdout
    if args.command == "z6":
        z6 = args.root / ("z6" if args.scene == "game_v13" else f"z6_{args.scene.removeprefix('game_')}")
        schedules = z6 / "schedules"
        z6_schedules(schedules)
        if not args.no_run:
            export(["--scene", args.scene, "--seeds", f"{Z6_SEEDS.start}..{Z6_SEEDS.stop}", "--days", "30", "--every", "60",
                    "--schedule-dir", str(schedules), "--include-factual", "--resume", "--workers", str(args.workers),
                    "--out", str(z6 / "episodes")], log)
        return 0
    if args.command == "v0":
        for split in args.splits.split(","):
            spec = SPLITS[split]
            sched = args.root / "v0" / "schedules" / split
            if spec["schedules"]:
                write_schedules(split, spec, sched)
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
                        "--out", str(args.root / "v0" / split)], log)
        if args.no_run:
            return 0
    man = manifest(args.root)
    args.docs.mkdir(parents=True, exist_ok=True)
    (args.docs / "dataset_v0_manifest.json").write_text(json.dumps(man, indent=1, sort_keys=True) + "\n")
    (args.docs / "DATASET_v0.md").write_text(dataset_doc(man))
    print(f"manifest: {man['episode_count']} episodes, {man['total_size'] / 2**20:.1f} MiB")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
