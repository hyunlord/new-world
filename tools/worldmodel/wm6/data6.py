"""WM-6 data (docs/worldmodel/wm6/GOAL.md 6): the v2x counterfactual worlds (game_v19 50000–51599, R = 1, WM-2's four
arms forked at tick 14,280), their manifest and their population check against WM-2's 400 train worlds. numpy only.

- `merge-timeline`: the recorder's timeline probe runs in seed chunks (`launch/record_v2x.sh`, so that its worker count
  can follow the training rule of GOAL.md 0.3.7); `timeline_p14.py` reads only `scene` and `runs` of a summary, so the
  chunk summaries are merged by concatenating their `runs` (every chunk's seeds once, every chunk determinism-checked).
- `manifest`: one entry per episode (WM-4/5's fields: scene, seed, role, split, schedule id, family, arm, replicate,
  fork tick, path, rows, schema, git sha, casts, files with sha256), every episode verified (`wm_data.verify_episode`);
  wm6's own loader because `families_v2.build_manifest` asserts R = {0, 1}. Role `train` for every seed (GOAL.md 4.2).

usage (DGX):
  data6.py merge-timeline --root DIR --lo 50000 --hi 51600 --chunk 100 --out SUMMARY.json
  data6.py manifest --root ROOT --git-sha SHA --out JSON
"""
from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np  # noqa: F401

import wm6  # noqa: F401
from wm_data import verify_episode
from wm2 import v2

SCENE = "game_v19"
V2X = "v2x"
SEEDS = (50000, 51600)                 # GOAL.md 4.2: the new train worlds (half-open)
GIT_SHA = "f5373ab2b7679c4a0d3a5c86b256971c86f93d41"
ARMS_RECORDED = v2.ARMS                # sp240L, sp120L, gf240E, rw120E (schedules_p14 names)


# ------------------------------------------------------------------ timeline ----------------------------------------
def merge_timeline(root: Path, lo: int, hi: int, chunk: int) -> dict:
    """Concatenate the `runs` of the chunk summaries `root/c<a>/summary.json` (a = lo, lo + chunk, …)."""
    runs, scene, det, chunks = [], None, [], []
    for a in range(lo, hi, chunk):
        s = json.loads((root / f"c{a}" / "summary.json").read_text())
        want = list(range(a, min(a + chunk, hi)))
        got = sorted(int(r["seed"]) for r in s["runs"])
        if got != want:
            raise ValueError(f"chunk c{a}: seeds {got[:3]}… ({len(got)}) != {a}..{a + len(want) - 1}")
        if scene is not None and s.get("scene") != scene:
            raise ValueError(f"chunk c{a}: scene {s.get('scene')} != {scene}")
        scene = s.get("scene")
        det.append(s.get("determinism_checked"))
        runs += s["runs"]
        chunks.append({"chunk": a, "seeds": len(got), "determinism_checked": s.get("determinism_checked"),
                       "schema_version": s.get("schema_version")})
    return {"scene": scene, "runs": sorted(runs, key=lambda r: int(r["seed"])), "determinism_checked": all(bool(x) for x in det),
            "merged_from": chunks, "note": "wm6/data6.py merge-timeline: chunk summaries' runs concatenated"}


# ------------------------------------------------------------------ manifest ----------------------------------------
def parse_member(name: str) -> tuple[str, int, str, int]:
    """`<scene>_s<seed>_<id>_r<k>` -> (scene, seed, id, k) (as `families_v2.parse_member`)."""
    head, rep = name.rsplit("_r", 1)
    scene_seed, eid = head.rsplit("_", 1)
    scene, seed = scene_seed.rsplit("_s", 1)
    return scene, int(seed), eid, int(rep)


def entry(root: Path, path: Path, git_sha: str = GIT_SHA, verify: bool = True) -> dict:
    meta = json.loads((path / "meta.json").read_text())
    scene, seed, eid = meta["scene"], int(meta["seed"]), meta["episode_id"]
    fork = meta.get("noise_fork")
    if meta.get("git_sha") != git_sha:
        raise ValueError(f"{path}: git_sha {meta.get('git_sha')} != {git_sha}")
    if scene != SCENE or not SEEDS[0] <= seed < SEEDS[1]:
        raise ValueError(f"{path}: {scene} seed {seed} outside the v2x worlds")
    if int(meta["rows"]) != 2881 or int(meta["days"]) != 120 or int(meta["every"]) != 60:
        raise ValueError(f"{path}: rows / days / every differ from 2881 / 120 / 60")
    if not fork or int(fork["tick"]) != v2.FORK_TICK or int(fork["replicate"]) != 0:
        raise ValueError(f"{path}: not a family member forked at {v2.FORK_TICK} with replicate 0 (R = 1)")
    if eid not in ("none",) + ARMS_RECORDED:
        raise ValueError(f"{path}: unknown arm {eid}")
    if path.name != f"{scene}_s{seed}_{eid}_r0":
        raise ValueError(f"{path}: directory name != {scene}_s{seed}_{eid}_r0")
    if verify and not verify_episode(path):
        raise ValueError(f"{path}: verify_episode failed")
    casts = []
    for rec in json.loads((path / "interventions.json").read_text()):
        tick = int(rec["tick"])
        assert tick % 60 == 0, f"{path}: cast tick {tick} is not a multiple of 60"
        casts.append({"tick": tick, "kind": rec["kind"], "amount": rec["amount"], "applied": rec.get("applied"),
                      "settlement": rec.get("settlement")})
    if eid == "none":
        if casts:
            raise ValueError(f"{path}: a factual member with interventions")
    else:
        kind, amount = v2.ARM_SPEC[eid]
        if len(casts) != 1 or casts[0]["kind"] != kind or int(casts[0]["amount"]) != amount:
            raise ValueError(f"{path}: interventions {casts} differ from the arm {eid}")
    return {"scene": scene, "seed": seed, "role": "train", "split": "v2x_fact" if eid == "none" else "v2x_arm_r0",
            "schedule_id": eid, "family": f"{scene}_s{seed}", "arm": eid, "replicate": 0, "fork_tick": int(fork["tick"]),
            "path": str(path.resolve().relative_to(root.resolve())), "rows": int(meta["rows"]), "days": int(meta["days"]),
            "schema_version": meta["schema_version"], "git_sha": meta["git_sha"], "casts": casts,
            "files": {k: {"sha256": v["sha256"]} for k, v in sorted(meta["arrays"].items())}}


def build_manifest(root: Path, git_sha: str = GIT_SHA, verify: bool = True, schedules: Path | None = None) -> dict:
    flat = root / V2X / "flat"
    eps = [entry(root, p, git_sha, verify) for p in sorted(flat.iterdir()) if (p / "meta.json").exists()]
    keys = [(e["seed"], e["schedule_id"]) for e in eps]
    if len(set(keys)) != len(keys):
        raise ValueError("duplicate episodes")
    seeds = sorted({e["seed"] for e in eps if e["schedule_id"] == "none"})
    if seeds != list(range(*SEEDS)):
        raise ValueError(f"factual seeds {len(seeds)}, expected {SEEDS[1] - SEEDS[0]}")
    missing_sched, extra = [], []
    if schedules is not None:          # every schedule file recorded, no arm member without one
        want = {(int(p.name.split("_")[0][1:]), p.stem.split("_")[1]) for p in schedules.glob("s*_*.json")}
        got = {k for k in keys if k[1] != "none"}
        missing_sched, extra = sorted(want - got), sorted(got - want)
        if missing_sched or extra:
            raise ValueError(f"schedules without an episode {missing_sched[:5]}, episodes without a schedule {extra[:5]}")
    arms: dict = {}
    for e in eps:
        arms[e["schedule_id"]] = arms.get(e["schedule_id"], 0) + 1
    per_world: dict = {}
    for s, eid in keys:
        per_world.setdefault(s, []).append(eid)
    pattern: dict = {}
    for s, ids in per_world.items():
        k = ",".join(sorted(ids))
        pattern[k] = pattern.get(k, 0) + 1
    return {"dataset": "worldsim-wm v2x (WM-6): game_v19 50000-51599, WM-2's four arms forked at 14,280, R = 1",
            "episode_count": len(eps), "git_sha": git_sha, "roles": {SCENE: {"train": [SEEDS[0], SEEDS[1] - 1]}},
            "worlds": len(seeds), "members_per_arm": arms, "member_patterns": pattern, "verified": verify,
            "episodes": eps}


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("merge-timeline")
    p.add_argument("--root", required=True); p.add_argument("--lo", type=int, required=True)
    p.add_argument("--hi", type=int, required=True); p.add_argument("--chunk", type=int, required=True)
    p.add_argument("--out", required=True)
    p = sub.add_parser("manifest")
    p.add_argument("--root", default="/home/hyunlord/work/worldsim-wm-data"); p.add_argument("--git-sha", default=GIT_SHA)
    p.add_argument("--out", required=True)
    a = ap.parse_args(argv)
    t0 = time.time()
    if a.cmd == "merge-timeline":
        out = merge_timeline(Path(a.root), a.lo, a.hi, a.chunk)
        Path(a.out).write_text(json.dumps(out) + "\n")
        print(json.dumps({"runs": len(out["runs"]), "determinism_checked": out["determinism_checked"],
                          "seconds": time.time() - t0}))
        return 0
    root = Path(a.root)
    out = build_manifest(root, a.git_sha, schedules=root / V2X / "schedules")
    out["seconds"] = time.time() - t0
    Path(a.out).parent.mkdir(parents=True, exist_ok=True)
    Path(a.out).write_text(json.dumps(out, indent=1, sort_keys=True) + "\n")
    print(json.dumps({k: v for k, v in out.items() if k != "episodes"}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
