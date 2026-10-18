"""The obs2 split of WM-5 (docs/worldmodel/wm5/GOAL.md 6.2): bookkeeping around the frozen recorder. numpy only.

Roles (GOAL.md 4.2): train 47000–48199, val 48200–48399, test 48400–48639 (`game_v19`). Data root `obs2/`:
`rounds/r<k>/` (round 0 = factual; round k >= 1 = the worlds with a k-th cast, recorded with all their casts so far,
episode id `obs`), `schedules/r<k>/s<seed>_obs.json`, `policy/round_<k>.json` (the rows each round's decisions read),
`policy/policy.json` (the final propensity record), `observed/` (symlinks to each world's last-round episode),
`schedules/fam/<role>/c<ref>/s<seed>/` and `families/<role>/c<ref>/{flat,fam}/` (GOAL.md 6.2.5).

Family members (one recorder call per (world, ref), `--fork-tick tick(c) --replicates R --schedule-dir … --include-factual`,
which the recorder requires with a fork): `none_r<k>` (no schedule; the recorder always adds it), `gf240_r<k>`,
`sp240_r<k>`, `rw120_r<k>` (`pre` + the arm at c) and `pre_r<k>` when the world cast before c. The `pre` member of a family
is `pre_r<k>` when it exists and `none_r<k>` otherwise (an empty `pre` schedule is the factual run: DECISIONS L1.2).

usage (DGX):
  obs2.py verify --dir DIR --expect N --out JSON          every episode passes wm_data.verify_episode, 2,881 rows
  obs2.py round-schedules --root ROOT --round K --out JSON  round K's new casts and schedule files (K >= 1)
  obs2.py round-check --root ROOT --round K --out JSON      prefix checks of round K against round K − 1
  obs2.py finalize --root ROOT --out JSON                   observed/ symlinks, policy.json (replay = rounds)
  obs2.py fam-schedules --root ROOT --out JSON              val / test family schedules at c1 and c2
  obs2.py fam-jobs --root ROOT --role R                     one line per (ref, seed, fork tick, replicates)
  obs2.py fam-check --root ROOT --out JSON                  prefix, casts, fork, pre = unforked recording
  obs2.py manifest --root ROOT --out obs2_manifest.json
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm5  # noqa: F401
from wm_data import load_episode, load_family, verify_episode
from wm2 import families_v2 as fv
from wm4 import obs_split as os4
from wm5 import policy2 as p2

SCENE = "game_v19"
ROLES = {"train": (47000, 48200), "val": (48200, 48400), "test": (48400, 48640)}
GIT_SHA = os4.GIT_SHA
ROWS = os4.ROWS
MAX_ROUNDS = p2.MAX_CASTS
REFS = ("c1", "c2")
REPLICATES = {"val": 1, "test": 2}
FAM_ARMS = p2.ARMS


def role_of(seed: int) -> str:
    for role, (lo, hi) in ROLES.items():
        if lo <= seed < hi:
            return role
    raise ValueError(f"seed {seed} outside the obs2 roles")


def seeds(role: str | None = None) -> list[int]:
    if role is None:
        return list(range(ROLES["train"][0], ROLES["test"][1]))
    return list(range(*ROLES[role]))


def r0_dir(root: Path, seed: int) -> Path:
    return Path(root) / "rounds" / "r0" / f"{SCENE}_s{seed}_none"


def round_dir(root: Path, k: int, seed: int) -> Path:
    return r0_dir(root, seed) if k == 0 else Path(root) / "rounds" / f"r{k}" / f"{SCENE}_s{seed}_obs"


def state_of_dir(path: Path) -> dict:
    return os4.state_of(load_episode(path, arrays=["global.npy", "settlements.npy"]))


def array_shas(path: Path) -> dict[str, str]:
    return os4.array_shas(path)


def _write_json(path: Path, obj) -> None:
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(obj, indent=1, sort_keys=True) + "\n")


# ------------------------------------------------------------------ rounds ------------------------------------------
def load_round(root: Path, k: int) -> dict:
    return json.loads((Path(root) / "policy" / f"round_{k}.json").read_text())


def casts_before(root: Path, k: int) -> dict[int, list[dict]]:
    """Every world's casts found in rounds 0 … k − 1 (i.e. recorded in rounds 1 … k)."""
    out = {s: [] for s in seeds()}
    for j in range(k):
        for w in load_round(root, j + 1)["worlds"]:
            if w["cast"] is not None:
                out[w["seed"]].append(w["cast"])
    return out


def round_schedules(root: Path, k: int, q: dict) -> dict:
    """Round k >= 1 (GOAL.md 6.2.3): every world recorded in round k − 1 (all worlds for k = 1) runs the policy on that
    episode from the row after its last cast (or day 8) to its first new cast or day 60; a world with a new cast gets
    the schedule of all its casts so far in `schedules/r<k>/s<seed>_obs.json`."""
    root = Path(root)
    prior = casts_before(root, k - 1)
    if k == 1:
        todo = seeds()
    else:
        todo = [w["seed"] for w in load_round(root, k - 1)["worlds"] if w["cast"] is not None]
    sched = root / "schedules" / f"r{k}"
    sched.mkdir(parents=True, exist_ok=True)
    worlds, new_seeds = [], []
    for seed in todo:
        src = round_dir(root, k - 1, seed)
        res = p2.step(seed, state_of_dir(src), q, prior[seed])
        rec = {"seed": seed, "role": role_of(seed), "read_round": k - 1, "read_path": str(src.relative_to(root)),
               "prior_casts": len(prior[seed]), "rows": res["rows"], "cast": None}
        if res["cast"] is not None:
            cast = res["cast"] | {"found_in_round": k - 1, "recorded_in_round": k}
            rec["cast"] = cast
            text = json.dumps(p2.schedule_rows(prior[seed] + [cast])) + "\n"
            (sched / f"s{seed}_obs.json").write_text(text)
            rec["schedule_sha256"] = hashlib.sha256(text.encode()).hexdigest()
            new_seeds.append(seed)
        worlds.append(rec)
    out = {"round": k, "terciles": q, "read": len(todo), "new_casts": len(new_seeds), "worlds": worlds,
           "arm_counts": {a: sum(1 for w in worlds if w["cast"] and w["cast"]["arm"] == a) for a in p2.ARMS}}
    _write_json(root / "policy" / f"round_{k}.json", out)
    (root / "policy" / f"round_{k}_seeds.txt").write_text("".join(f"{s}\n" for s in new_seeds))
    return {k2: v for k2, v in out.items() if k2 != "worlds"}


def check_casts(recs: list[dict], casts: list[dict]) -> bool:
    """The episode's intervention records are exactly the casts (tick, kind, requested amount), in order."""
    if len(recs) != len(casts):
        return False
    for r, cst in zip(recs, sorted(casts, key=lambda x: int(x["tick"]))):
        kind, amount, _ = p2.ARM_SPEC[cst["arm"]]
        if int(r["tick"]) != int(cst["tick"]) or r["kind"] != kind or int(r["amount"]) != amount:
            return False
    return True


def round_check(root: Path, k: int) -> dict:
    """GOAL.md 6.2.3's prefix check: each round-k episode equals the world's round k − 1 episode on every row <= the new
    cast row (`global.npy` in full, settlements per WM-2 6.2's rule), carries every cast so far, verifies, and differs
    from the previous episode after the cast (reported)."""
    root = Path(root)
    rnd = load_round(root, k)
    prior = casts_before(root, k - 1)
    rows, bad = [], []
    for w in rnd["worlds"]:
        if w["cast"] is None:
            continue
        seed = w["seed"]
        new, old = round_dir(root, k, seed), round_dir(root, k - 1, seed)
        r = {"seed": seed, "verify": verify_episode(new)}
        ep_n = load_episode(new, arrays=fv.ARRAYS)
        ep_o = load_episode(old, arrays=fv.ARRAYS)
        diffs = fv.compare_prefix(ep_n, ep_o, int(w["cast"]["row"]))
        r["prefix_ok"], r["diffs"] = not diffs, diffs
        r["casts_ok"] = check_casts(ep_n.interventions, prior[seed] + [w["cast"]])
        r["applied"] = [x.get("applied") for x in ep_n.interventions]
        r["after_differs"] = array_shas(new)["global.npy"] != array_shas(old)["global.npy"]
        r["ok"] = bool(r["verify"] and r["prefix_ok"] and r["casts_ok"])
        if not r["ok"]:
            bad.append(seed)
        rows.append(r)
    return {"round": k, "worlds": len(rows), "bad": bad, "all_ok": not bad and len(rows) == rnd["new_casts"],
            "after_differs": sum(r["after_differs"] for r in rows), "rows": rows}


def last_round(root: Path) -> int:
    k = 0
    while (Path(root) / "policy" / f"round_{k + 1}.json").exists():
        k += 1
    return k


def finalize(root: Path, q: dict) -> dict:
    """GOAL.md 6.2.4: the observed episode of a world = its last round's episode (`observed/` symlinks). The policy
    replayed on the observed episode (`policy2.replay`) must reproduce every round's casts and rows (the observed episode
    equals each earlier round's on rows <= each later cast row); `policy/policy.json` is that replay with each cast's
    round."""
    root = Path(root)
    K = last_round(root)
    if load_round(root, K)["new_casts"] != 0 and K < MAX_ROUNDS + 1:
        raise SystemExit(f"round {K} still added casts: record round {K + 1} first")
    casts = casts_before(root, K)
    rows_by_round: dict[int, dict] = {}
    for k in range(1, K + 1):
        for w in load_round(root, k)["worlds"]:
            rows_by_round.setdefault(w["seed"], {})[k] = w["rows"]
    obs = root / "observed"
    obs.mkdir(parents=True, exist_ok=True)
    worlds, bad = [], []
    for seed in seeds():
        n = len(casts[seed])
        src = round_dir(root, n, seed)              # the round that recorded the n-th cast (round 0 when none)
        link = obs / f"{SCENE}_s{seed}_obs"
        if link.is_symlink() or link.exists():
            if os.path.realpath(link) != os.path.realpath(src):
                raise ValueError(f"{link} points elsewhere")
        else:
            link.symlink_to(src.resolve())
        rep = p2.replay(seed, state_of_dir(src), q)
        same_casts = [(c["arm"], c["row"]) for c in rep["casts"]] == [(c["arm"], c["row"]) for c in casts[seed]]
        round_rows = [r for k in sorted(rows_by_round.get(seed, {})) for r in rows_by_round[seed][k]]
        key = lambda r: (r["row"], r["status"], r["p_g"], r["p_s"], r["p_r"], r["cast"])
        same_rows = [key(r) for r in rep["rows"][: len(round_rows)]] == [key(r) for r in round_rows]
        tail_ok = all(r["status"] in ("refractory", "cap", "no_decision") or r["cast"] is None
                      for r in rep["rows"][len(round_rows):])
        ok = same_casts and same_rows and tail_ok
        if not ok:
            bad.append(seed)
        for c_rep, c_rec in zip(rep["casts"], casts[seed]):
            c_rep["recorded_in_round"] = c_rec["recorded_in_round"]
            c_rep["found_in_round"] = c_rec["found_in_round"]
        worlds.append({"seed": seed, "role": role_of(seed), "observed": str(src.relative_to(root)),
                       "observed_round": n, "casts": rep["casts"], "rows": rep["rows"],
                       "refs": p2.ref_rows(rep["casts"]), "replay_ok": ok})
    policy = {"terciles": q, "probabilities": {"gf240": p2.P_GIFT, "sp240": p2.P_SPOIL,
                                               "rw120": [p2.P_ROT_LOW, p2.P_ROT_HIGH]},
              "refractory_days": p2.REFRACTORY_DAYS, "max_casts": p2.MAX_CASTS, "stream": p2.STREAM,
              "days": [p2.DAYS[0], p2.DAYS[-1]], "rounds": K, "worlds": worlds, "summary": summarize(worlds)}
    _write_json(root / "policy" / "policy.json", policy)
    return {"rounds": K, "worlds": len(worlds), "bad": bad, "all_ok": not bad, "summary": policy["summary"]}


def summarize(worlds: list[dict]) -> dict:
    """Casts per world, never-cast share, arm shares overall / per role / per f-tercile of the cast row, the share of
    casts with an earlier cast in the history, and the realized daily cast rate per tercile (GOAL.md 6.2.6)."""
    def block(ws):
        n = len(ws)
        cs = [c for w in ws for c in w["casts"]]
        per = [len(w["casts"]) for w in ws]
        arms = {a: sum(1 for c in cs if c["arm"] == a) for a in p2.ARMS}
        return {"worlds": n, "never": sum(1 for k in per if k == 0), "never_share": sum(1 for k in per if k == 0) / max(1, n),
                "casts": len(cs), "casts_per_world_mean": float(np.mean(per)) if per else 0.0,
                "casts_per_world_hist": {str(k): sum(1 for x in per if x == k) for k in range(p2.MAX_CASTS + 1)},
                "arm_counts": arms, "arm_shares": {a: v / max(1, len(cs)) for a, v in arms.items()},
                "casts_with_earlier_cast": sum(1 for w in ws for i, _ in enumerate(w["casts"]) if i > 0),
                "share_with_earlier_cast": sum(1 for w in ws for i, _ in enumerate(w["casts"]) if i > 0) / max(1, len(cs))}
    out = {"all": block(worlds)}
    for role in ROLES:
        out[role] = block([w for w in worlds if w["role"] == role])
    by_t = {}
    for t in range(3):
        rows = [r for w in worlds for r in w["rows"] if r["status"] == "decision" and r["tercile"] == t]
        casts = [r for r in rows if r["cast"] is not None]
        arms = {a: sum(1 for r in casts if r["cast"] == a) for a in p2.ARMS}
        by_t[str(t)] = {"decision_rows": len(rows), "casts": len(casts), "arm_counts": arms,
                        "arm_shares": {a: v / max(1, len(casts)) for a, v in arms.items()},
                        "cast_rate": len(casts) / max(1, len(rows))}
    out["by_f_tercile"] = by_t
    out["status_counts"] = {s: sum(1 for w in worlds for r in w["rows"] if r["status"] == s)
                            for s in ("decision", "refractory", "cap", "no_decision")}
    return out


# ------------------------------------------------------------------ families ----------------------------------------
def fam_dir(root: Path, role: str, ref: str) -> Path:
    return Path(root) / "families" / role / ref


def fam_schedules(root: Path) -> dict:
    """GOAL.md 6.2.5: for every val / test world and reference row c in (c1, c2), `schedules/fam/<role>/<ref>/s<seed>/`
    holds `s<seed>_<arm>.json` (`pre` + the arm at c) for the three arms, `s<seed>_pre.json` when `pre` is not empty,
    and `ref.json` (not read by the recorder: it reads only `s*.json`)."""
    root = Path(root)
    policy = json.loads((root / "policy" / "policy.json").read_text())
    n = {"val": 0, "test": 0}
    pre_nonempty = {r: 0 for r in REFS}
    for w in policy["worlds"]:
        if w["role"] == "train":
            continue
        for ref in REFS:
            c = int(w["refs"][ref])
            d = root / "schedules" / "fam" / w["role"] / ref / f"s{w['seed']}"
            d.mkdir(parents=True, exist_ok=True)
            sch = p2.family_schedules(w["casts"], c)
            for name, rows in sch.items():
                if name == "pre" and not rows:
                    continue
                (d / f"s{w['seed']}_{name}.json").write_text(json.dumps(rows) + "\n")
            pre_nonempty[ref] += bool(sch["pre"])
            _write_json(d / "ref.json", {"seed": w["seed"], "role": w["role"], "ref": ref, "row": c, "tick": c * 60,
                                         "pre": p2.pre_casts(w["casts"], c), "replicates": REPLICATES[w["role"]]})
            n[w["role"]] += 1
    return {"families": n, "pre_nonempty": pre_nonempty}


def fam_jobs(root: Path, role: str) -> list[str]:
    """`<ref> <seed> <fork tick> <replicates>` per family of a role (the recorder calls of GOAL.md 6.2.5)."""
    out = []
    for ref in REFS:
        for seed in seeds(role):
            j = json.loads((Path(root) / "schedules" / "fam" / role / ref / f"s{seed}" / "ref.json").read_text())
            out.append(f"{ref} {seed} {j['tick']} {j['replicates']}")
    return out


def pre_id(fam) -> str:
    return "pre" if "pre" in fam.variants else "none"


def members(fam) -> dict[str, dict[int, object]]:
    """{`pre`, arm: {replicate: Episode}} of one loaded family (`pre` = `pre_r` or `none_r`, DECISIONS L1.2)."""
    out = {"pre": fam.variants["pre"] if "pre" in fam.variants else fam.factual}
    for arm in FAM_ARMS:
        out[arm] = fam.variants[arm]
    return out


def fam_check(root: Path, unforked: Path | None = None) -> dict:
    """GOAL.md 6.2.5's checks per family: every member equals the observed episode on rows < c; every member carries
    the `pre` casts (an arm member: `pre` + the arm at c, nothing else); the fork is at c with the role's replicates; the
    members are exactly `none`, the three arms and `pre` when it is not empty. Also (L1.3): `none_r0` of a family whose
    `pre` is empty equals the world's round-0 factual episode; `pre_r0` equals a fresh unforked recording of the same
    schedule for the sampled worlds under `unforked/` (`<role>/<ref>/<scene>_s<seed>_pre`)."""
    root = Path(root)
    policy = json.loads((root / "policy" / "policy.json").read_text())
    rows, bad = [], []
    for w in policy["worlds"]:
        if w["role"] == "train":
            continue
        seed, role = w["seed"], w["role"]
        obs = load_episode(root / w["observed"], arrays=fv.ARRAYS)
        for ref in REFS:
            c = int(w["refs"][ref])
            pre = p2.pre_casts(w["casts"], c)
            path = fam_dir(root, role, ref) / "fam" / f"{SCENE}_s{seed}"
            fam = load_family(path, verify=True, arrays=fv.ARRAYS)
            r = {"seed": seed, "role": role, "ref": ref, "row": c, "pre": len(pre)}
            want = sorted(FAM_ARMS + (("pre",) if pre else ()))
            r["members_ok"] = sorted(fam.variants) == want
            r["fork_ok"] = fam.fork_tick == c * 60 and fam.replicates == REPLICATES[role]
            diffs, casts_ok, applied = [], True, {}
            for eid, reps in [("none", fam.factual)] + sorted(fam.variants.items()):
                for k, ep in reps.items():
                    diffs += [f"{eid}_r{k}: {x}" for x in fv.compare_prefix(ep, obs, c - 1)]
                    want_casts = ([] if eid == "none" else pre if eid == "pre"
                                  else pre + [{"arm": eid, "tick": c * 60}])
                    ok = check_casts(ep.interventions, want_casts)
                    casts_ok &= ok
                    applied[f"{eid}_r{k}"] = [x.get("applied") for x in ep.interventions]
            r["prefix_ok"], r["diffs"], r["casts_ok"], r["applied"] = not diffs, diffs, casts_ok, applied
            if not pre:
                r["none_r0_equal_r0"] = array_shas(path / f"{SCENE}_s{seed}_none_r0") == array_shas(r0_dir(root, seed))
            if unforked is not None and (Path(unforked) / role / ref / f"{SCENE}_s{seed}_pre").exists():
                r["pre_r0_equal_unforked"] = (array_shas(path / f"{SCENE}_s{seed}_{pre_id(fam)}_r0")
                                              == array_shas(Path(unforked) / role / ref / f"{SCENE}_s{seed}_pre"))
            r["ok"] = bool(r["members_ok"] and r["fork_ok"] and r["prefix_ok"] and r["casts_ok"]
                           and r.get("none_r0_equal_r0", True) and r.get("pre_r0_equal_unforked", True))
            if not r["ok"]:
                bad.append(f"{seed}:{ref}")
            rows.append(r)
    want = sum((hi - lo) * len(REFS) for role, (lo, hi) in ROLES.items() if role != "train")
    return {"families": len(rows), "expected": want, "bad": bad, "all_ok": not bad and len(rows) == want,
            "unforked_checked": sum("pre_r0_equal_unforked" in r for r in rows),
            "none_r0_equal_r0_checked": sum("none_r0_equal_r0" in r for r in rows), "rows": rows}


# ------------------------------------------------------------------ manifest ----------------------------------------
def entry(root: Path, path: Path, split: str, role: str, schedule_id: str, policy_sha: str, **extra) -> dict:
    meta = json.loads((path / "meta.json").read_text())
    if meta.get("git_sha") != GIT_SHA or int(meta["rows"]) != ROWS:
        raise ValueError(f"{path}: git_sha / rows differ")
    if not verify_episode(path):
        raise ValueError(f"{path}: verify_episode failed")
    recs = json.loads((path / "interventions.json").read_text())
    casts = [{"tick": int(x["tick"]), "kind": x["kind"], "amount": x["amount"], "applied": x.get("applied")} for x in recs]
    fork = meta.get("noise_fork")
    real = Path(os.path.realpath(path))
    return {"scene": meta["scene"], "seed": int(meta["seed"]), "role": role, "split": split, "schedule_id": schedule_id,
            "episode_id": meta["episode_id"], "fork_tick": None if not fork else int(fork["tick"]),
            "replicate": None if not fork else int(fork["replicate"]),
            "path": str(real.relative_to(Path(root).resolve().parent)), "rows": int(meta["rows"]),
            "casts": casts, "propensity_sha256": policy_sha,
            "files": {k: {"sha256": v["sha256"]} for k, v in sorted(meta["arrays"].items())}} | extra


def build_manifest(root: Path) -> dict:
    """`obs2_train` / `obs2_val` / `obs2_test`: the observed episode of every world (its last round's); `obs2fam_<id>`
    (`pre` and the three arms; the `pre` member is the recorder's `none` member, `member_id` `none`, when `pre` is
    empty): the family members of the val and test worlds with `ref` (c1 / c2) and `replicate`. Paths relative to the parent of
    the data root (`WM_DATA_ROOT`)."""
    root = Path(root)
    ppath = root / "policy" / "policy.json"
    policy = json.loads(ppath.read_text())
    psha = hashlib.sha256(ppath.read_bytes()).hexdigest()
    eps = []
    for w in policy["worlds"]:
        seed, role = w["seed"], w["role"]
        sched = p2.schedule_rows(w["casts"])
        eps.append(entry(root, root / "observed" / f"{SCENE}_s{seed}_obs", f"obs2_{role}", role,
                         "obs" if w["casts"] else "none", psha, round=w["observed_round"], schedule=sched, ref=None))
        if role == "train":
            continue
        for ref in REFS:
            c = int(w["refs"][ref])
            sch = p2.family_schedules(w["casts"], c)
            path = fam_dir(root, role, ref) / "fam" / f"{SCENE}_s{seed}"
            pid = "pre" if sch["pre"] else "none"
            for eid, mid in [("pre", pid)] + [(a, a) for a in FAM_ARMS]:
                for k in range(REPLICATES[role]):
                    p = path / f"{SCENE}_s{seed}_{mid}_r{k}"
                    e = entry(root, p, f"obs2fam_{eid}", role, eid, psha, ref=ref, ref_row=c, round=None,
                              schedule=sch[eid], member_id=mid)
                    if e["fork_tick"] != c * 60 or e["replicate"] != k:
                        raise ValueError(f"{p}: fork {e['fork_tick']} / replicate {e['replicate']} != {c * 60} / {k}")
                    eps.append(e)
    counts: dict = {}
    for e in eps:
        cc = counts.setdefault(e["split"], {})
        cc[e["role"]] = cc.get(e["role"], 0) + 1
    return {"dataset": "worldsim-wm obs2 (WM-5): the sequential multi-cast observational split of game_v19",
            "episode_count": len(eps), "git_sha": GIT_SHA, "policy_sha256": psha,
            "roles": {r: [lo, hi - 1] for r, (lo, hi) in ROLES.items()}, "counts_by_split_role": counts,
            "root_note": "paths relative to the parent of WM_DATA_ROOT's obs2/ directory, i.e. WM_DATA_ROOT",
            "episodes": eps}


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("verify"); p.add_argument("--dir", required=True); p.add_argument("--expect", type=int, required=True)
    p.add_argument("--out", required=True)
    for name in ("round-schedules", "round-check"):
        p = sub.add_parser(name); p.add_argument("--root", required=True); p.add_argument("--round", type=int, required=True)
        p.add_argument("--out", required=True)
    for name in ("finalize", "fam-schedules", "manifest"):
        p = sub.add_parser(name); p.add_argument("--root", required=True); p.add_argument("--out", required=True)
    p = sub.add_parser("fam-check"); p.add_argument("--root", required=True); p.add_argument("--out", required=True)
    p.add_argument("--unforked", default=None)
    p = sub.add_parser("fam-jobs"); p.add_argument("--root", required=True); p.add_argument("--role", required=True)
    a = ap.parse_args(argv)
    if a.cmd == "fam-jobs":
        print("\n".join(fam_jobs(Path(a.root), a.role)))
        return 0
    t0 = time.time()
    if a.cmd == "verify":
        out = os4.verify(Path(a.dir), a.expect)
    elif a.cmd == "round-schedules":
        out = round_schedules(Path(a.root), a.round, p2.frozen_terciles())
    elif a.cmd == "round-check":
        out = round_check(Path(a.root), a.round)
    elif a.cmd == "finalize":
        out = finalize(Path(a.root), p2.frozen_terciles())
    elif a.cmd == "fam-schedules":
        out = fam_schedules(Path(a.root))
    elif a.cmd == "fam-check":
        out = fam_check(Path(a.root), Path(a.unforked) if a.unforked else None)
    else:
        out = build_manifest(Path(a.root))
    out["seconds"] = time.time() - t0
    _write_json(Path(a.out), out)
    print(json.dumps({k: v for k, v in out.items() if k not in ("rows", "worlds", "episodes")})[:3000])
    return 0 if out.get("all_ok", True) else 1


if __name__ == "__main__":
    raise SystemExit(main())
