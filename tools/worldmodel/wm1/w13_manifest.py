"""The WM-1.3 manifest (GOAL_1_3.md 6.2-6.3): one entry per episode of `w13/{game_v16,game_v17}`.

Roles (fixed by GOAL_1_3.md 4.2): game_v16 train 30000-30399, val 30400-30479, test 30480-30559; game_v17 train
32000-32399, val 32400-32479, test 32480-32559. Each entry: scene, seed, role, `split` = the scene, `schedule_id` =
`none`, path relative to the data root, the array files with sha256 (from `meta.json`, re-checked against the files by
`wm_data.verify_episode`), rows, days, schema, `git_sha`. Every episode must verify, carry `git_sha` = `--git-sha`,
have 2,881 rows and the expected scene / seed / directory name; the seed set must be exactly the roles' seeds.

usage (DGX): python3 tools/worldmodel/wm1/w13_manifest.py --git-sha SHA [--root WM_DATA_ROOT] [--out JSON]
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import wm1  # noqa: F401
from wm_data import verify_episode

SUBDIR = "w13"
ROLES = {"game_v16": (("train", range(30000, 30400)), ("val", range(30400, 30480)), ("test", range(30480, 30560))),
         "game_v17": (("train", range(32000, 32400)), ("val", range(32400, 32480)), ("test", range(32480, 32560)))}


def role_of(scene: str, seed: int) -> str:
    for role, seeds in ROLES[scene]:
        if seed in seeds:
            return role
    raise ValueError(f"{scene} seed {seed} is outside the WM-1.3 roles")


def build(root: Path, git_sha: str, verify: bool = True) -> dict:
    episodes = []
    for scene in ROLES:
        for path in sorted((root / SUBDIR / scene).iterdir(), key=lambda p: p.name):
            meta = json.loads((path / "meta.json").read_text())
            if meta["scene"] != scene or meta.get("episode_id") != "none" or meta.get("schedule") is not None:
                raise ValueError(f"{path}: unexpected scene / episode id / schedule")
            if path.name != f"{scene}_s{meta['seed']}_none":
                raise ValueError(f"{path}: directory name does not match the seed")
            if meta.get("git_sha") != git_sha:
                raise ValueError(f"{path}: git_sha {meta.get('git_sha')} != {git_sha}")
            if int(meta["rows"]) != 2881 or int(meta["days"]) != 120 or int(meta["every"]) != 60:
                raise ValueError(f"{path}: rows / days / every differ from 2881 / 120 / 60")
            if verify and not verify_episode(path):
                raise ValueError(f"{path}: verify_episode failed")
            episodes.append({"scene": scene, "split": scene, "schedule_id": "none", "seed": int(meta["seed"]),
                             "role": role_of(scene, int(meta["seed"])), "path": f"{SUBDIR}/{scene}/{path.name}",
                             "rows": int(meta["rows"]), "days": int(meta["days"]), "schema_version": meta["schema_version"],
                             "git_sha": meta["git_sha"],
                             "files": {k: {"sha256": v["sha256"]} for k, v in sorted(meta["arrays"].items())}})
    episodes.sort(key=lambda e: (list(ROLES).index(e["scene"]), e["seed"]))
    for scene, roles in ROLES.items():
        want = sorted(s for _, seeds in roles for s in seeds)
        got = sorted(e["seed"] for e in episodes if e["scene"] == scene)
        if got != want:
            raise ValueError(f"{scene}: {len(got)} episodes, expected the {len(want)} role seeds")
    counts = {s: {r: sum(e["scene"] == s and e["role"] == r for e in episodes) for r, _ in ROLES[s]} for s in ROLES}
    return {"dataset": "worldsim-wm w13 (WM-1.3)", "episode_count": len(episodes), "git_sha": git_sha,
            "roles": {s: {r: [x.start, x.stop - 1] for r, x in roles} for s, roles in ROLES.items()},
            "counts": counts, "verified": verify, "episodes": episodes}


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--root", default="/home/hyunlord/work/worldsim-wm-data")
    ap.add_argument("--git-sha", required=True)
    ap.add_argument("--out", default=str(Path(__file__).resolve().parents[3] / "docs/worldmodel/wm1/w13_manifest.json"))
    args = ap.parse_args(argv)
    m = build(Path(args.root), args.git_sha)
    Path(args.out).write_text(json.dumps(m, indent=1, sort_keys=True) + "\n")
    print(m["episode_count"], json.dumps(m["counts"]))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
