"""Dataset v2 bookkeeping (GOAL.md 6.2-6.5): per-world family directories, the per-replicate prefix check, the r0
identity check of the noise-floor families, and the manifest. numpy only.

- `link`: one directory per world of symlinks to the recorder's flat output (`wm_data.load_family` expects one seed
  per directory): `<fam>/<scene>_s<seed>/<member>` -> `<flat>/<member>`.
- `check`: per family, (a) rows <= row(c) of arm member r equal those of `none` member r (c = the arm's cast tick from
  `interventions.json`), (b) rows <= row(fork) of every member equal `none_r0`'s. `global.npy` is compared in full over
  those rows; `settlements.npy` ([T, S, F], S = every settlement the episode ever had) over the slots both episodes
  share, after checking that the shorter `settlement_ids.npy` is a prefix of the longer; the extra slots of the longer
  must be all-NaN in those rows. Not `families.check_prefix` (it requires equal settlement ids).
- `r0`: a floor family's `none_r0` against an unforked recording of the same world: every `.npy` file bit-identical.
- `manifest`: `v2_manifest.json` (families and the game_v16 control; frozen before training) or, with `--part floor`,
  `v2_floor_manifest.json` (the noise-floor families, recorded beside the first training): one entry per episode (scene, seed, role, `split`, `schedule_id`, family id, arm, replicate, fork tick,
  path, files with sha256, rows, cast ticks with requested and applied amounts); every episode verified with
  `wm_data.verify_episode`; every cast tick a multiple of 60 (asserted).

usage (DGX):
  python3 tools/worldmodel/wm2/families_v2.py link --flat DIR --fam DIR
  python3 tools/worldmodel/wm2/families_v2.py check --fam DIR --out JSON
  python3 tools/worldmodel/wm2/families_v2.py r0 --floor-fam DIR --ref DIR --seeds 40480 40481 --out JSON
  python3 tools/worldmodel/wm2/families_v2.py manifest --root ROOT --git-sha SHA --out JSON [--part main|floor]
"""
from __future__ import annotations

import argparse
import json
import os
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm2  # noqa: F401
from wm_data import load_episode, load_family, sha256_file, verify_episode
from wm2 import v2

ARRAYS = ["global.npy", "settlements.npy", "settlement_ids.npy"]
V2 = "v2"


# ------------------------------------------------------------------ link --------------------------------------------
def parse_member(name: str) -> tuple[str, int, str, int]:
    """`<scene>_s<seed>_<id>_r<k>` -> (scene, seed, id, k)."""
    head, rep = name.rsplit("_r", 1)
    scene_seed, eid = head.rsplit("_", 1)
    scene, seed = scene_seed.rsplit("_s", 1)
    return scene, int(seed), eid, int(rep)


def link(flat: Path, fam: Path) -> dict:
    """Symlink every member of `flat` into `fam/<scene>_s<seed>/`; returns the per-world member counts."""
    counts: dict[str, int] = {}
    for member in sorted(p for p in flat.iterdir() if (p / "meta.json").exists()):
        scene, seed, _, _ = parse_member(member.name)
        world = fam / f"{scene}_s{seed}"
        world.mkdir(parents=True, exist_ok=True)
        dst = world / member.name
        if dst.is_symlink() or dst.exists():
            if os.path.realpath(dst) != os.path.realpath(member):
                raise ValueError(f"{dst} exists and points elsewhere")
        else:
            dst.symlink_to(member.resolve())
        counts[world.name] = counts.get(world.name, 0) + 1
    return counts


# ------------------------------------------------------------------ prefix check ------------------------------------
def compare_prefix(a, b, row: int) -> list[str]:
    """Differences of rows <= `row` between two loaded episodes (`global.npy` in full; `settlements.npy` over the
    shared slots, the shorter id list a prefix of the longer, the longer's extra slots all-NaN)."""
    out = []
    ga, gb = a.arrays["global.npy"], b.arrays["global.npy"]
    if ga.shape[1] != gb.shape[1] or not np.array_equal(ga[: row + 1], gb[: row + 1], equal_nan=True):
        out.append(f"global.npy rows <= {row} differ")
    ia, ib = a.arrays["settlement_ids.npy"], b.arrays["settlement_ids.npy"]
    short, long_ = (ia, ib) if len(ia) <= len(ib) else (ib, ia)
    if not np.array_equal(long_[: len(short)], short):
        out.append(f"settlement ids {ia.tolist()} / {ib.tolist()}: the shorter is not a prefix of the longer")
        return out
    n = len(short)
    sa, sb = a.arrays["settlements.npy"], b.arrays["settlements.npy"]
    if not np.array_equal(sa[: row + 1, :n], sb[: row + 1, :n], equal_nan=True):
        out.append(f"settlements.npy rows <= {row}, shared slots differ")
    longer = sa if len(ia) > len(ib) else sb
    if longer.shape[1] > n and not np.isnan(longer[: row + 1, n:]).all():
        out.append(f"settlements.npy rows <= {row}: extra slots are not all-NaN")
    return out


def cast_tick(ep) -> int:
    """The single cast tick of an arm member (`interventions.json`), a multiple of 60 (asserted)."""
    recs = ep.interventions
    if len(recs) != 1:
        raise ValueError(f"{ep.path}: {len(recs)} intervention records, expected 1")
    tick = int(recs[0]["tick"])
    v2.row_of_tick(tick)
    return tick


def check_family(fam) -> dict:
    """GOAL.md 6.2's per-replicate prefix check of one loaded family (`wm_data.Family`)."""
    fork_row = v2.row_of_tick(fam.fork_tick)
    ref = fam.factual[0]
    diffs = []
    members = [("none", k, ep) for k, ep in fam.factual.items()]
    members += [(arm, k, ep) for arm, reps in fam.variants.items() for k, ep in reps.items()]
    for eid, k, ep in members:
        diffs += [f"{eid}_r{k} vs none_r0: {x}" for x in compare_prefix(ep, ref, fork_row)]
        if eid != "none":
            row = v2.row_of_tick(cast_tick(ep))
            if row < fork_row:
                diffs.append(f"{eid}_r{k}: cast row {row} before the fork row {fork_row}")
            diffs += [f"{eid}_r{k} vs none_r{k}: {x}" for x in compare_prefix(ep, fam.factual[k], row)]
        elif ep.interventions:
            diffs.append(f"none_r{k}: has interventions")
    return {"seed": fam.seed, "scene": fam.scene, "fork_tick": fam.fork_tick, "replicates": fam.replicates,
            "arms": sorted(fam.variants), "members": len(members), "ok": not diffs, "diffs": diffs}


def check(fam_root: Path) -> dict:
    rows = []
    for world in sorted(p for p in fam_root.iterdir() if p.is_dir()):
        rows.append(check_family(load_family(world, verify=False, arrays=ARRAYS)))
    per_arm = {}
    for r in rows:
        for a in r["arms"]:
            per_arm[a] = per_arm.get(a, 0) + 1
    return {"fam": str(fam_root), "families": len(rows), "ok": sum(r["ok"] for r in rows),
            "all_ok": bool(rows) and all(r["ok"] for r in rows), "members": sum(r["members"] for r in rows),
            "families_per_arm": per_arm, "rows": rows}


def r0_identity(floor_fam: Path, ref: Path, seeds: list[int]) -> dict:
    """A floor family's `none_r0` against an unforked recording (`ref/<scene>_s<seed>_none`): every `.npy` file equal
    by sha256."""
    rows = []
    for seed in seeds:
        world = next(floor_fam.glob(f"*_s{seed}"))
        member = next(world.glob(f"*_s{seed}_none_r0"))
        unforked = next(ref.glob(f"*_s{seed}_none"))
        names = sorted(p.name for p in Path(os.path.realpath(member)).glob("*.npy"))
        ref_names = sorted(p.name for p in unforked.glob("*.npy"))
        diff = [n for n in names if n not in ref_names or sha256_file(member / n) != sha256_file(unforked / n)]
        diff += [n for n in ref_names if n not in names]
        rows.append({"seed": seed, "member": str(member), "unforked": str(unforked), "npy": len(names),
                     "identical": not diff, "diffs": diff})
    return {"rows": rows, "all_identical": all(r["identical"] for r in rows)}


# ------------------------------------------------------------------ manifest ----------------------------------------
def split_of(scene: str, eid: str, rep: int | None, floor_tick: int | None) -> str:
    """The manifest split of a member (`common.V19_*`, `v16`, `v19_floor_s<s>`, `v16_floor_s<s>`)."""
    short = {"game_v19": "v19", "game_v16": "v16"}[scene]
    if floor_tick is not None:
        return f"{short}_floor_s{floor_tick}"
    if scene == "game_v16":
        return "v16"
    if eid == "none":
        return "v19_fact" if rep == 0 else "v19_none_r1"
    return "v19_arm_r0" if rep == 0 else "v19_arm_r1"


def entry(root: Path, path: Path, git_sha: str, floor_tick: int | None = None, verify: bool = True) -> dict:
    meta = json.loads((path / "meta.json").read_text())
    scene, seed, eid = meta["scene"], int(meta["seed"]), meta["episode_id"]
    fork = meta.get("noise_fork")
    rep = None if not fork else int(fork["replicate"])
    if meta.get("git_sha") != git_sha:
        raise ValueError(f"{path}: git_sha {meta.get('git_sha')} != {git_sha}")
    if int(meta["rows"]) != 2881 or int(meta["days"]) != 120 or int(meta["every"]) != 60:
        raise ValueError(f"{path}: rows / days / every differ from 2881 / 120 / 60")
    if verify and not verify_episode(path):
        raise ValueError(f"{path}: verify_episode failed")
    want = f"{scene}_s{seed}_{eid}" + ("" if rep is None else f"_r{rep}")
    if path.name != want:
        raise ValueError(f"{path}: directory name != {want}")
    if floor_tick is not None:
        if not fork or int(fork["tick"]) != floor_tick or eid != "none":
            raise ValueError(f"{path}: not a factual floor member forked at {floor_tick}")
    elif scene == "game_v19":
        if not fork or int(fork["tick"]) != v2.FORK_TICK or rep not in (0, 1):
            raise ValueError(f"{path}: not a family member forked at {v2.FORK_TICK} with R = 2")
        if eid not in ("none",) + v2.ARMS:
            raise ValueError(f"{path}: unknown arm {eid}")
    elif fork or eid != "none":
        raise ValueError(f"{path}: the game_v16 control is factual and unforked")
    casts = []
    recs = json.loads((path / "interventions.json").read_text())
    for rec in recs:
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
    rel = path.resolve().relative_to(root.resolve())
    return {"scene": scene, "seed": seed, "role": v2.role_of(scene, seed), "split": split_of(scene, eid, rep, floor_tick),
            "schedule_id": eid, "family": f"{scene}_s{seed}" + ("" if floor_tick is None else f"@floor_s{floor_tick}"),
            "arm": eid, "replicate": rep, "fork_tick": None if not fork else int(fork["tick"]), "path": str(rel),
            "rows": int(meta["rows"]), "days": int(meta["days"]), "schema_version": meta["schema_version"],
            "git_sha": meta["git_sha"], "casts": casts,
            "files": {k: {"sha256": v["sha256"]} for k, v in sorted(meta["arrays"].items())}}


def episode_dirs(d: Path) -> list[Path]:
    return sorted((p for p in d.iterdir() if (p / "meta.json").exists()), key=lambda p: p.name)


def build_manifest(root: Path, git_sha: str, verify: bool = True, part: str = "main") -> dict:
    """`part` "main": the game_v19 families and the game_v16 control (`v2_manifest.json`, frozen before training);
    "floor": the noise-floor families of both scenes (`v2_floor_manifest.json`, recorded beside the first training)."""
    base = root / V2
    eps = []
    if part == "main":
        eps += [entry(root, p, git_sha, verify=verify) for p in episode_dirs(base / "v19" / "flat")]
        eps += [entry(root, p, git_sha, verify=verify) for p in episode_dirs(base / "v16")]
    elif part == "floor":
        for scene_dir, scene in (("v19", "game_v19"), ("v16", "game_v16")):
            for s in v2.FLOOR_STARTS:
                got = [entry(root, p, git_sha, s, verify) for p in episode_dirs(base / scene_dir / f"floor_s{s}" / "flat")]
                lo, hi = v2.ROLES[scene]["test"]
                want = sorted((seed, k) for seed in range(lo, hi) for k in range(v2.FLOOR_R))
                if sorted((e["seed"], e["replicate"]) for e in got) != want:
                    raise ValueError(f"{scene} floor_s{s}: {len(got)} members, expected {len(want)}")
                eps += got
    else:
        raise ValueError(part)
    keys = [(e["scene"], e["seed"], e["schedule_id"], e["replicate"], e["fork_tick"]) for e in eps]
    if len(set(keys)) != len(keys):
        raise ValueError("duplicate episodes")
    for scene, roles in v2.ROLES.items():
        if part != "main":
            break
        want = sorted(range(roles["train"][0], roles["test"][1]))
        got = sorted({e["seed"] for e in eps if e["scene"] == scene and e["split"] in ("v19_fact", "v16")})
        if got != want:
            raise ValueError(f"{scene}: factual seeds {len(got)}, expected {len(want)}")
    counts: dict = {}
    for e in eps:
        c = counts.setdefault(e["split"], {})
        c[e["role"]] = c.get(e["role"], 0) + 1
    arms: dict = {}
    for e in eps:
        if e["scene"] == "game_v19" and e["fork_tick"] == v2.FORK_TICK and e["replicate"] == 0:
            a = arms.setdefault(e["arm"], {})
            a[e["role"]] = a.get(e["role"], 0) + 1
    return {"dataset": f"worldsim-wm v2 (WM-2), {part}", "episode_count": len(eps), "git_sha": git_sha,
            "roles": {s: {r: [lo, hi - 1] for r, (lo, hi) in roles.items()} for s, roles in v2.ROLES.items()},
            "counts_by_split_role": counts, "families_per_arm_role": arms, "verified": verify, "episodes": eps}


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("link"); p.add_argument("--flat", required=True); p.add_argument("--fam", required=True)
    p = sub.add_parser("check"); p.add_argument("--fam", required=True); p.add_argument("--out", required=True)
    p = sub.add_parser("r0"); p.add_argument("--floor-fam", required=True); p.add_argument("--ref", required=True)
    p.add_argument("--seeds", type=int, nargs="+", required=True); p.add_argument("--out", required=True)
    p = sub.add_parser("manifest"); p.add_argument("--root", default="/home/hyunlord/work/worldsim-wm-data")
    p.add_argument("--git-sha", required=True); p.add_argument("--out", required=True)
    p.add_argument("--part", default="main", choices=["main", "floor"])
    a = ap.parse_args(argv)
    t0 = time.time()
    if a.cmd == "link":
        counts = link(Path(a.flat), Path(a.fam))
        print(json.dumps({"worlds": len(counts), "members": sum(counts.values()),
                          "members_per_world": sorted(set(counts.values()))}))
        return 0
    if a.cmd == "check":
        out = check(Path(a.fam))
    elif a.cmd == "r0":
        out = r0_identity(Path(a.floor_fam), Path(a.ref), a.seeds)
    else:
        out = build_manifest(Path(a.root), a.git_sha, part=a.part)
    out["seconds"] = time.time() - t0
    Path(a.out).parent.mkdir(parents=True, exist_ok=True)
    Path(a.out).write_text(json.dumps(out, indent=1, sort_keys=True) + "\n")
    summary = {k: v for k, v in out.items() if k not in ("rows", "episodes")}
    print(json.dumps(summary)[:2000])
    ok = out.get("all_ok", out.get("all_identical", True))
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
