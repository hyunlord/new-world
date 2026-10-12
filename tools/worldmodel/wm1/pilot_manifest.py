"""The p10 pilot manifest (GOAL_1_2.md 6.3): one entry per episode of `p10/pilot/{game_v15,game_v16_neutral,game_v16}`.

Each entry: scene, seed, role (train 20000-20079, val 20080-20095, test 20096-20111), path relative to the data
root, the files with sha256 from the episode's `meta.json`, rows; plus `split` = the scene and `schedule_id` = `none`
(what `data.load_split` reads). Built from the episodes' `meta.json` files only (the data stay read-only).

usage: python3 tools/worldmodel/wm1/pilot_manifest.py [--root WM_DATA_ROOT] [--out docs/worldmodel/wm1/p10_pilot_manifest.json]
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path

SCENES = ("game_v15", "game_v16_neutral", "game_v16")
ROLES = (("train", range(20000, 20080)), ("val", range(20080, 20096)), ("test", range(20096, 20112)))
SUBDIR = "p10/pilot"


def role_of(seed: int) -> str:
    for role, seeds in ROLES:
        if seed in seeds:
            return role
    raise ValueError(f"seed {seed} is outside the pilot's roles")


def build(root: Path) -> dict:
    episodes = []
    for scene in SCENES:
        for path in sorted((root / SUBDIR / scene).iterdir(), key=lambda p: p.name):
            meta = json.loads((path / "meta.json").read_text())
            if meta["scene"] != scene or meta.get("episode_id") != "none" or meta.get("schedule") is not None:
                raise ValueError(f"{path}: unexpected scene / episode id / schedule")
            if path.name != f"{scene}_s{meta['seed']}_none":
                raise ValueError(f"{path}: directory name does not match the seed")
            episodes.append({"scene": scene, "split": scene, "schedule_id": "none", "seed": int(meta["seed"]),
                             "role": role_of(int(meta["seed"])), "path": f"{SUBDIR}/{scene}/{path.name}",
                             "rows": int(meta["rows"]), "days": int(meta["days"]), "schema_version": meta["schema_version"],
                             "git_sha": meta["git_sha"],
                             "files": {k: {"sha256": v["sha256"]} for k, v in sorted(meta["arrays"].items())}})
    episodes.sort(key=lambda e: (SCENES.index(e["scene"]), e["seed"]))
    counts = {s: {r: sum(e["scene"] == s and e["role"] == r for e in episodes) for r, _ in ROLES} for s in SCENES}
    return {"dataset": "worldsim-wm p10 pilot", "episode_count": len(episodes), "roles": {r: [s.start, s.stop - 1] for r, s in ROLES},
            "counts": counts, "episodes": episodes}


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--root", default="/Users/rexxa/work/worldsim-wm-data")
    ap.add_argument("--out", default=str(Path(__file__).resolve().parents[3] / "docs/worldmodel/wm1/p10_pilot_manifest.json"))
    args = ap.parse_args(argv)
    m = build(Path(args.root))
    Path(args.out).write_text(json.dumps(m, indent=1, sort_keys=True) + "\n")
    print(m["episode_count"], json.dumps(m["counts"]))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
