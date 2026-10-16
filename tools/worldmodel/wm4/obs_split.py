"""The observational split of WM-4 (docs/worldmodel/wm4/GOAL.md 6.2): bookkeeping around the frozen recorder. numpy only.

Roles (GOAL.md 4.2): train 46000–46399, val 46400–46479, test 46480–46559 (`game_v19`).

usage (DGX):
  obs_split.py verify --dir DIR --expect N --out JSON              every episode passes wm_data.verify_episode
  obs_split.py terciles --factual DIR --out JSON                   q_f1, q_f2, q_w2 over the train worlds (6.1)
  obs_split.py policy --factual DIR --metrics METRICS.md --out policy.json --schedules DIR
                                                                   the policy on every world (terciles read from
                                                                   METRICS.md part a and re-derived: must agree)
  obs_split.py observed-check --factual DIR --observed DIR --policy JSON --out JSON
  obs_split.py fam-schedules --policy JSON --out DIR               per val / test world: the three arms at c_w
  obs_split.py seeds --role val|test
  obs_split.py fam-check --root ROOT --policy JSON --out JSON      prefix checks, observed = family member of its arm
  obs_split.py manifest --root ROOT --policy JSON --out obs_manifest.json
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm4  # noqa: F401
import bench_v1 as bv1
from wm_data import load_episode, load_family, verify_episode
from wm2 import families_v2 as fv
from wm4 import policy as pol

SCENE = "game_v19"
ROLES = {"train": (46000, 46400), "val": (46400, 46480), "test": (46480, 46560)}
GIT_SHA = "f5373ab2b7679c4a0d3a5c86b256971c86f93d41"
ROWS = 2881
TERCILE_TAG = "TERCILES_JSON:"


def role_of(seed: int) -> str:
    for role, (lo, hi) in ROLES.items():
        if lo <= seed < hi:
            return role
    raise ValueError(f"seed {seed} outside the obs roles")


def seeds(role: str | None = None) -> list[int]:
    if role is None:
        return list(range(ROLES["train"][0], ROLES["test"][1]))
    return list(range(*ROLES[role]))


def episode_dirs(d: Path) -> list[Path]:
    return sorted((p for p in Path(d).iterdir() if (p / "meta.json").exists()), key=lambda p: p.name)


def state_of(ep) -> dict:
    """The policy's per-row state of a loaded episode (observable columns only)."""
    g = lambda n: bv1.observable_column(ep, "global.npy", n).astype(np.float64)
    s = ep.arrays["settlements.npy"]
    col = lambda n: (np.nansum(bv1.observable_column(ep, "settlements.npy", n).astype(np.float64), axis=1)
                     if s.shape[1] else np.zeros(s.shape[0]))
    return pol.state_series(col("food_stock"), col("wood_stock"), g("alive"), g("settlements_alive"))


def load_states(factual: Path, which: list[int]) -> dict[int, dict]:
    out = {}
    for seed in which:
        ep = load_episode(Path(factual) / f"{SCENE}_s{seed}_none", arrays=["global.npy", "settlements.npy"])
        out[seed] = state_of(ep)
    return out


def array_shas(path: Path) -> dict[str, str]:
    meta = json.loads((Path(path) / "meta.json").read_text())
    return {k: v["sha256"] for k, v in meta["arrays"].items()}


# ------------------------------------------------------------------ verify / terciles / policy ----------------------
def verify(d: Path, expect: int) -> dict:
    eps = episode_dirs(d)
    bad = [p.name for p in eps if not verify_episode(p)]
    rows = sorted({int(json.loads((p / "meta.json").read_text())["rows"]) for p in eps})
    return {"dir": str(d), "episodes": len(eps), "expected": expect, "bad": bad, "rows": rows,
            "all_ok": len(eps) == expect and not bad and rows == [ROWS]}


def terciles_from(factual: Path) -> dict:
    return pol.terciles(list(load_states(factual, seeds("train")).values()))


def read_terciles(metrics: Path) -> dict:
    """The frozen terciles: the JSON after `TERCILES_JSON:` in METRICS.md part a."""
    for line in Path(metrics).read_text().splitlines():
        if TERCILE_TAG in line:
            return json.loads(line.split(TERCILE_TAG, 1)[1].strip().strip("`"))
    raise ValueError(f"{metrics}: no {TERCILE_TAG} line")


def run_policy(factual: Path, q: dict, schedules: Path) -> dict:
    states = load_states(factual, seeds())
    schedules.mkdir(parents=True, exist_ok=True)
    worlds = []
    for seed in seeds():
        w = pol.run(seed, states[seed], q)
        w["role"] = role_of(seed)
        w["ref_tick"] = pol.ref_tick(w)
        if w["cast"]:
            rows = pol.schedule_rows(w["cast"]["arm"], w["cast"]["tick"])
            text = json.dumps(rows) + "\n"
            (schedules / f"s{seed}_obs.json").write_text(text)
            w["cast"]["schedule_sha256"] = hashlib.sha256(text.encode()).hexdigest()
        worlds.append(w)
    return {"terciles": q, "probabilities": {"gf240": pol.P_GIFT, "sp240": pol.P_SPOIL,
                                             "rw120": [pol.P_ROT_LOW, pol.P_ROT_HIGH]},
            "days": [pol.DAYS[0], pol.DAYS[-1]], "worlds": worlds, "summary": summarize(worlds)}


def summarize(worlds: list[dict]) -> dict:
    """Never-cast share and arm shares, overall, per role and per f-tercile of the cast row (6.2.6)."""
    def block(ws):
        n = len(ws)
        casts = [w["cast"] for w in ws if w["cast"]]
        arms = {a: sum(1 for c in casts if c["arm"] == a) for a in pol.ARMS}
        return {"worlds": n, "never": n - len(casts), "never_share": (n - len(casts)) / max(1, n),
                "casts": len(casts), "arm_counts": arms,
                "arm_shares": {a: k / max(1, len(casts)) for a, k in arms.items()}}
    out = {"all": block(worlds)}
    for role in ROLES:
        out[role] = block([w for w in worlds if w["role"] == role])
    by_t = {}
    for t in range(3):
        casts = [w["rows"][-1] for w in worlds if w["cast"] and w["rows"][-1]["tercile"] == t]
        arms = {a: sum(1 for r in casts if r["cast"] == a) for a in pol.ARMS}
        # decision rows spent in this tercile (every world, up to its cast) and the realized daily cast rate
        rows = [r for w in worlds for r in w["rows"] if r["decision"] and r["tercile"] == t]
        by_t[str(t)] = {"casts": len(casts), "arm_counts": arms,
                        "arm_shares": {a: k / max(1, len(casts)) for a, k in arms.items()},
                        "decision_rows": len(rows), "cast_rate": len(casts) / max(1, len(rows))}
    out["by_f_tercile"] = by_t
    out["non_decision_rows"] = sum(1 for w in worlds for r in w["rows"] if not r["decision"])
    return out


# ------------------------------------------------------------------ observed check ----------------------------------
def observed_check(factual: Path, observed: Path, policy: dict) -> dict:
    rows, bad = [], []
    for w in policy["worlds"]:
        seed = w["seed"]
        fac = Path(factual) / f"{SCENE}_s{seed}_none"
        none = Path(observed) / f"{SCENE}_s{seed}_none"
        r = {"seed": seed, "none_equal_factual": array_shas(none) == array_shas(fac)}
        obs = Path(observed) / f"{SCENE}_s{seed}_obs"
        if w["cast"]:
            ep_o = load_episode(obs, arrays=fv.ARRAYS)
            ep_f = load_episode(fac, arrays=fv.ARRAYS)
            diffs = fv.compare_prefix(ep_o, ep_f, int(w["cast"]["row"]))
            recs = ep_o.interventions
            kind, amount, _ = pol.ARM_SPEC[w["cast"]["arm"]]
            r.update({"prefix_ok": not diffs, "diffs": diffs, "records": len(recs),
                      "record_ok": len(recs) == 1 and int(recs[0]["tick"]) == int(w["cast"]["tick"])
                      and recs[0]["kind"] == kind and int(recs[0]["amount"]) == amount,
                      "applied": recs[0].get("applied") if recs else None,
                      "after_differs": array_shas(obs)["global.npy"] != array_shas(fac)["global.npy"]})
        else:
            r.update({"obs_absent": not obs.exists()})
        ok = r["none_equal_factual"] and (r.get("prefix_ok", True) and r.get("record_ok", True)
                                          and r.get("obs_absent", True))
        r["ok"] = bool(ok)
        if not ok:
            bad.append(seed)
        rows.append(r)
    applied = [r["applied"] for r in rows if r.get("applied") is not None]
    return {"worlds": len(rows), "bad": bad, "all_ok": not bad, "casting": sum(1 for w in policy["worlds"] if w["cast"]),
            "after_differs": sum(1 for r in rows if r.get("after_differs")),
            "applied_min": min(applied) if applied else None, "rows": rows}


# ------------------------------------------------------------------ families ----------------------------------------
def fam_schedules(policy: dict, out: Path) -> dict:
    n = 0
    for w in policy["worlds"]:
        if w["role"] == "train":
            continue
        d = Path(out) / f"s{w['seed']}"
        d.mkdir(parents=True, exist_ok=True)
        for arm in pol.ARMS:
            (d / f"s{w['seed']}_{arm}.json").write_text(json.dumps(pol.schedule_rows(arm, w["ref_tick"])) + "\n")
        (d / "ref.json").write_text(json.dumps({"seed": w["seed"], "ref_tick": w["ref_tick"],
                                                "cast": w["cast"]}) + "\n")
        n += 1
    return {"worlds": n}


def fam_check(root: Path, policy: dict) -> dict:
    rows, bad = [], []
    for w in policy["worlds"]:
        if w["role"] == "train":
            continue
        seed, role = w["seed"], w["role"]
        fam_dir = Path(root) / "families" / role / "fam" / f"{SCENE}_s{seed}"
        fam = load_family(fam_dir, verify=True, arrays=fv.ARRAYS)
        r = fv.check_family(fam)
        r["fork_ok"] = fam.fork_tick == w["ref_tick"] and fam.replicates == 1 and sorted(fam.variants) == sorted(pol.ARMS)
        casts = {}
        for arm, reps in fam.variants.items():
            rec = reps[0].interventions
            kind, amount, _ = pol.ARM_SPEC[arm]
            casts[arm] = {"ok": len(rec) == 1 and int(rec[0]["tick"]) == w["ref_tick"] and rec[0]["kind"] == kind
                          and int(rec[0]["amount"]) == amount, "applied": rec[0].get("applied") if rec else None}
        r["casts"] = casts
        member = lambda eid: fam_dir / f"{SCENE}_s{seed}_{eid}_r0"
        r["none_equal_factual"] = array_shas(member("none")) == array_shas(Path(root) / "factual" / f"{SCENE}_s{seed}_none")
        if w["cast"]:
            r["observed_equal_member"] = (array_shas(member(w["cast"]["arm"]))
                                          == array_shas(Path(root) / "observed" / f"{SCENE}_s{seed}_obs"))
        r["all"] = bool(r["ok"] and r["fork_ok"] and all(c["ok"] for c in casts.values()) and r["none_equal_factual"]
                        and r.get("observed_equal_member", True))
        if not r["all"]:
            bad.append(seed)
        rows.append(r)
    return {"families": len(rows), "bad": bad, "all_ok": not bad and len(rows) == 160, "rows": rows}


# ------------------------------------------------------------------ manifest ----------------------------------------
def entry(root: Path, path: Path, split: str, role: str, schedule_id: str, policy_sha: str) -> dict:
    meta = json.loads((path / "meta.json").read_text())
    if meta.get("git_sha") != GIT_SHA or int(meta["rows"]) != ROWS:
        raise ValueError(f"{path}: git_sha / rows differ")
    if not verify_episode(path):
        raise ValueError(f"{path}: verify_episode failed")
    recs = json.loads((path / "interventions.json").read_text())
    casts = [{"tick": int(x["tick"]), "kind": x["kind"], "amount": x["amount"], "applied": x.get("applied")} for x in recs]
    fork = meta.get("noise_fork")
    return {"scene": meta["scene"], "seed": int(meta["seed"]), "role": role, "split": split, "schedule_id": schedule_id,
            "episode_id": meta["episode_id"], "fork_tick": None if not fork else int(fork["tick"]),
            "path": str(Path(path).resolve().relative_to(Path(root).resolve().parent)), "rows": int(meta["rows"]),
            "cast_tick": casts[0]["tick"] if casts else None, "applied": casts[0]["applied"] if casts else None,
            "casts": casts, "propensity_sha256": policy_sha,
            "files": {k: {"sha256": v["sha256"]} for k, v in sorted(meta["arrays"].items())}}


def build_manifest(root: Path, policy_path: Path) -> dict:
    """`obs_train` / `obs_val`: the observed episode of every train / val world (`_obs` of a casting world, the factual
    of a non-casting one); `obs_test`: every test world's factual episode; `obsfam_<arm>` (`none` included): the family
    members of the val and test worlds. Paths are relative to the data root's parent (`WM_DATA_ROOT`)."""
    root = Path(root)
    policy = json.loads(Path(policy_path).read_text())
    psha = hashlib.sha256(Path(policy_path).read_bytes()).hexdigest()
    eps = []
    for w in policy["worlds"]:
        seed, role = w["seed"], w["role"]
        if role in ("train", "val"):
            if w["cast"]:
                eps.append(entry(root, root / "observed" / f"{SCENE}_s{seed}_obs", f"obs_{role}", role, "obs", psha))
            else:
                eps.append(entry(root, root / "observed" / f"{SCENE}_s{seed}_none", f"obs_{role}", role, "none", psha))
        else:
            eps.append(entry(root, root / "factual" / f"{SCENE}_s{seed}_none", "obs_test", role, "none", psha))
        if role != "train":
            fam_dir = root / "families" / role / "flat"
            for eid in ("none",) + pol.ARMS:
                p = fam_dir / f"{SCENE}_s{seed}_{eid}_r0"
                e = entry(root, p, f"obsfam_{eid}", role, eid, psha)
                if e["fork_tick"] != w["ref_tick"]:
                    raise ValueError(f"{p}: fork tick {e['fork_tick']} != ref tick {w['ref_tick']}")
                eps.append(e)
    counts: dict = {}
    for e in eps:
        c = counts.setdefault(e["split"], {})
        c[e["role"]] = c.get(e["role"], 0) + 1
    return {"dataset": "worldsim-wm obs (WM-4): the confounded observational split of game_v19",
            "episode_count": len(eps), "git_sha": GIT_SHA, "policy_sha256": psha,
            "roles": {r: [lo, hi - 1] for r, (lo, hi) in ROLES.items()}, "counts_by_split_role": counts,
            "root_note": "paths relative to the parent of WM_DATA_ROOT's obs/ directory, i.e. WM_DATA_ROOT",
            "episodes": eps}


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("verify"); p.add_argument("--dir", required=True); p.add_argument("--expect", type=int, required=True)
    p.add_argument("--out", required=True)
    p = sub.add_parser("terciles"); p.add_argument("--factual", required=True); p.add_argument("--out", required=True)
    p = sub.add_parser("policy"); p.add_argument("--factual", required=True); p.add_argument("--metrics", required=True)
    p.add_argument("--out", required=True); p.add_argument("--schedules", required=True)
    p = sub.add_parser("observed-check"); p.add_argument("--factual", required=True); p.add_argument("--observed", required=True)
    p.add_argument("--policy", required=True); p.add_argument("--out", required=True)
    p = sub.add_parser("fam-schedules"); p.add_argument("--policy", required=True); p.add_argument("--out", required=True)
    p = sub.add_parser("seeds"); p.add_argument("--role", required=True)
    p = sub.add_parser("fam-check"); p.add_argument("--root", required=True); p.add_argument("--policy", required=True)
    p.add_argument("--out", required=True)
    p = sub.add_parser("manifest"); p.add_argument("--root", required=True); p.add_argument("--policy", required=True)
    p.add_argument("--out", required=True)
    a = ap.parse_args(argv)
    if a.cmd == "seeds":
        print(" ".join(str(s) for s in seeds(a.role)))
        return 0
    t0 = time.time()
    if a.cmd == "verify":
        out = verify(Path(a.dir), a.expect)
    elif a.cmd == "terciles":
        out = terciles_from(Path(a.factual))
    elif a.cmd == "policy":
        q = read_terciles(Path(a.metrics))
        again = terciles_from(Path(a.factual))
        if any(abs(q[k] - again[k]) > 1e-9 for k in ("q_f1", "q_f2", "q_w2")):
            raise SystemExit(f"terciles in METRICS.md {q} differ from the recomputed {again}")
        out = run_policy(Path(a.factual), q, Path(a.schedules))
    elif a.cmd == "observed-check":
        out = observed_check(Path(a.factual), Path(a.observed), json.loads(Path(a.policy).read_text()))
    elif a.cmd == "fam-schedules":
        out = fam_schedules(json.loads(Path(a.policy).read_text()), Path(a.out))
    elif a.cmd == "fam-check":
        out = fam_check(Path(a.root), json.loads(Path(a.policy).read_text()))
    else:
        out = build_manifest(Path(a.root), Path(a.policy))
    out["seconds"] = time.time() - t0
    Path(a.out).parent.mkdir(parents=True, exist_ok=True)
    Path(a.out).write_text(json.dumps(out, indent=1, sort_keys=True) + "\n")
    print(json.dumps({k: v for k, v in out.items() if k not in ("rows", "worlds", "episodes")})[:3000])
    return 0 if out.get("all_ok", True) else 1


if __name__ == "__main__":
    raise SystemExit(main())
