"""WM-4 effect evaluation on the observational split (docs/worldmodel/wm4/GOAL.md 4.1, 7.1, 7.2): cases, effect scales,
the observational baselines and the M1 models' predictions. numpy only (a model enters through `wm3.effects3`).

**Cases** (7.1): for every val / test world and every arm a of {`gf240`, `sp240`, `rw120`}, one case at the world's
reference row c_w (the policy's cast row, or 06:00 of day 30 for a world that never cast); truth = arm member − `none`
member at c_w + 168 and + 720 rows (`wm2.effects.family_cases` on `obs/families/<role>/fam/*`) and the restricted split
day shift. Each `Case`'s arm is rewritten to its v2 key (`gf240` → `gf240E`, `sp240` → `sp240L`, `rw120` → `rw120E`)
so that `wm3.effects3.predictions` (`v2.ARM_SPEC`) and `wm2.effects.cell_errors` (`stds[(arm, o, lag)]`) work
unchanged; tables label arms with the obs names (`OBS_OF`).

**Scales** (4.1): per (outcome, lag), the scale of an obs arm = max(the v2 arm's train std, the v2 train std pooled
over every arm's train cases), both from `effects_base.json` `cases.train`; the unfloored variant = the v2 arm's std
alone (WM-2's `stds`).

**Baselines** (7.2), from the observed **train** episodes and `policy.json` only. Stack every decision row c (days
8–60) with c + h ≤ 2,880. Treated T_a = {(w, c): w cast a at c}; controls C = {(k, c): c a decision row of k and k cast
at no decision row j < c + h}. Δy = y(c + h) − y(c) of the observed episode.
- `zero`;
- `naive` = mean_{T_a} Δy − mean_C Δy;
- `ipw` = Σ_{T_a} Δy / p_a(c) ÷ Σ_{T_a} 1 / p_a(c) − Σ_C Δy / π_kc ÷ Σ_C 1 / π_kc, π_kc = ∏_{decision rows j, c ≤ j < c + h}
  (1 − p_tot,k(j)): the mean over risk-set pairs of E[Δy_a − Δy_none] (Hájek weights);
- `match5`: for a query (world, a, c) the 5 nearest members of T_a (an5's 24-row distance of the 8 V at their cast rows,
  inv_std of the observed train episodes) and the 5 nearest members of C at row c (the same display day); predicted
  effect = mean treated Δy − mean control Δy.
The split shift of every baseline is the same construction on the restricted split day from c (controls of h = 720).
Each lag's baseline is chosen on the **val** families by the 12-cell average (the only use of val counterfactuals).

usage (DGX):
  effects4.py describe --root ROOT --base effects_base.json --policy policy.json --out DIR   # DATASET_OBS numbers (val)
  effects4.py base --root ROOT --base effects_base.json --policy policy.json --out DIR       # baselines, val selection,
                                                                                            # test predictions
  effects4.py m1 --run RUN --root ROOT --base effects_base.json --out DIR                   # an M1 model on obs test
"""
from __future__ import annotations

import argparse
import dataclasses
import json
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm4  # noqa: F401
import bench_v0 as b
import bench_v1 as bv1
from wm_data import load_episode, load_family
from wm1 import common as c
from wm2 import effects as ef
from wm2 import v2
from wm4 import policy as pol

OBS_ARMS = pol.ARMS
TO_V2 = pol.TO_V2
OBS_OF = {v: k for k, v in TO_V2.items()}
V2_ARMS = tuple(TO_V2[a] for a in OBS_ARMS)
LAGS = v2.LAGS
CELLS = {lag: [(TO_V2[a], o, lag) for a in OBS_ARMS for o in v2.OUTCOMES] for lag in LAGS}
BASELINES = ("zero", "naive", "ipw", "match5")
SCENE = "game_v19"


def obs_root(root: Path) -> Path:
    return Path(root) / "obs"


def family_dirs(root: Path, role: str) -> list[Path]:
    fam = obs_root(root) / "families" / role / "fam"
    lo, hi = {"val": (46400, 46480), "test": (46480, 46560)}[role]
    return [fam / f"{SCENE}_s{s}" for s in range(lo, hi) if (fam / f"{SCENE}_s{s}").is_dir()]


def to_v2(case: ef.Case) -> ef.Case:
    """The case with its arm rewritten to the v2 key (on-disk ids stay the obs names)."""
    return dataclasses.replace(case, arm=TO_V2[case.arm])


def load_cases(root: Path, role: str, verify: bool = True) -> list[ef.Case]:
    out = []
    for path in family_dirs(root, role):
        fam = load_family(path, verify=verify, arrays=ef.ARRAYS)
        out += [to_v2(x) for x in ef.family_cases(fam, role)]
    return out


# ------------------------------------------------------------------ scales ------------------------------------------
def scales(base: dict) -> tuple[dict, dict]:
    """(floored, unfloored) scales {(v2 arm, outcome, lag): std} of the obs arms (4.1)."""
    train = [ef.case_from(j) for j in base["cases"]["train"]]
    arm_std = ef.stds_from(base["stds"])
    floored, unfloored = {}, {}
    for o in v2.OUTCOMES:
        for lag in LAGS:
            pooled = float(np.std([x.truth[o][lag] for x in train if lag in x.truth[o]]))
            for arm in V2_ARMS:
                sd = arm_std[(arm, o, lag)]
                floored[(arm, o, lag)] = max(sd if sd is not None else 0.0, pooled)
                unfloored[(arm, o, lag)] = sd
    return floored, unfloored


def scales_json(s: dict) -> dict:
    return {f"{OBS_OF[k[0]]}|{k[1]}|{k[2]}": v for k, v in s.items()}


# ------------------------------------------------------------------ observed train episodes -------------------------
@dataclasses.dataclass
class World:
    """One observed train world: per-row truth series, bench V, and its policy record."""
    seed: int
    series: dict          # v2.member_series of the observed episode
    v: np.ndarray         # [T, 8] bench V
    cast_row: int | None
    cast_arm: str | None
    probs: dict           # decision row -> (p_g, p_s, p_r) (rows up to and including the cast)
    decisions: set        # decision rows (p > 0)


def load_worlds(root: Path, policy: dict, role: str = "train", manifest: dict | None = None) -> list[World]:
    """The observed episodes of a role's worlds (`_obs` of a casting world, `_none` otherwise)."""
    out = []
    for w in policy["worlds"]:
        if w["role"] != role:
            continue
        eid = "obs" if w["cast"] else "none"
        ep = load_episode(obs_root(root) / "observed" / f"{SCENE}_s{w['seed']}_{eid}", arrays=ef.ARRAYS)
        out.append(world_of(w, v2.member_series(ep), bv1.episode_arrays(ep)["v"]))
    return out


def world_of(w: dict, series: dict, v: np.ndarray) -> World:
    probs = pol.prob_table(w)
    dec = {int(r["row"]) for r in w["rows"] if r["decision"]}
    cast = w["cast"]
    return World(int(w["seed"]), series, v, int(cast["row"]) if cast else None, cast["arm"] if cast else None, probs, dec)


def dy(world: World, c_row: int, h: int) -> dict:
    out = {o: float(world.series[o][c_row + h] - world.series[o][c_row]) for o in v2.OUTCOMES}
    out["shift_day"] = float(v2.restricted_split_day(world.series["fissions_total"], c_row)[0])
    return out


@dataclasses.dataclass
class Unit:
    seed: int
    row: int
    dy: dict
    weight: float              # 1 / p_a(c) (treated) or 1 / pi_kc (control)
    window: np.ndarray


def stack(worlds: list[World], h: int) -> tuple[dict, dict]:
    """(treated {obs arm: [Unit]}, controls {row c: [Unit]}) at horizon h (rows)."""
    treated = {a: [] for a in OBS_ARMS}
    controls: dict = {}
    for w in worlds:
        for c_row in pol.DECISION_ROWS:
            if c_row + h > v2.LAST_ROW:
                continue
            if w.cast_row is not None and w.cast_row < c_row:
                break                                   # out of the risk set
            if c_row not in w.decisions:
                continue
            win = w.v[c_row - 23: c_row + 1]
            if w.cast_row == c_row:
                p = w.probs[c_row][OBS_ARMS.index(w.cast_arm)]
                treated[w.cast_arm].append(Unit(w.seed, c_row, dy(w, c_row, h), 1.0 / p, win))
            if w.cast_row is None or w.cast_row >= c_row + h:
                pi = 1.0
                for j in pol.DECISION_ROWS:
                    if c_row <= j < c_row + h:
                        pi *= 1.0 - sum(w.probs.get(j, (0.0, 0.0, 0.0)))
                controls.setdefault(c_row, []).append(Unit(w.seed, c_row, dy(w, c_row, h), 1.0 / pi, win))
    return treated, controls


def wmean(units: list[Unit], key: str, weighted: bool) -> float:
    wts = np.array([u.weight if weighted else 1.0 for u in units])
    vals = np.array([u.dy[key] for u in units])
    return float((wts * vals).sum() / wts.sum())


def ess(units: list[Unit]) -> float:
    w = np.array([u.weight for u in units])
    return float(w.sum() ** 2 / (w ** 2).sum()) if len(w) else 0.0


KEYS = v2.OUTCOMES + ("shift_day",)


def pooled_estimates(worlds: list[World]) -> dict:
    """`naive` and `ipw` per obs arm and lag (and the shift at 30 d), with counts and effective sample sizes."""
    out = {"naive": {}, "ipw": {}, "counts": {}}
    for lag, h in LAGS.items():
        treated, controls = stack(worlds, h)
        allc = [u for us in controls.values() for u in us]
        for a in OBS_ARMS:
            t = treated[a]
            out["naive"].setdefault(a, {})[lag] = {k: wmean(t, k, False) - wmean(allc, k, False) for k in KEYS} if t else None
            out["ipw"].setdefault(a, {})[lag] = {k: wmean(t, k, True) - wmean(allc, k, True) for k in KEYS} if t else None
            out["counts"].setdefault(a, {})[lag] = {"treated": len(t), "controls": len(allc), "ess_treated": ess(t),
                                                   "ess_controls": ess(allc),
                                                   "max_weight_treated": max((u.weight for u in t), default=None),
                                                   "max_weight_controls": max((u.weight for u in allc), default=None)}
    return out


def nearest(window: np.ndarray, units: list[Unit], inv_std: np.ndarray, k: int = 5) -> list[Unit]:
    """The k nearest units by an5's distance, at most one per world; ties by world seed (stable)."""
    best: dict = {}
    for u in units:
        dist = float((((window - u.window) * inv_std) ** 2).sum())
        if u.seed not in best or dist < best[u.seed][0]:
            best[u.seed] = (dist, u)
    ranked = sorted(best.values(), key=lambda t: (t[0], t[1].seed))
    return [t[1] for t in ranked[:k]]


class Baselines:
    """The four baselines' predictions for a case (keyed by `Case.key`)."""

    def __init__(self, worlds: list[World], inv_std: np.ndarray):
        self.inv_std = inv_std
        self.est = pooled_estimates(worlds)
        self.stacks = {lag: stack(worlds, h) for lag, h in LAGS.items()}

    def predict(self, case: ef.Case) -> dict:
        a = OBS_OF[case.arm]
        out = {"zero": ef.predict_zero(case)}
        for name in ("naive", "ipw"):
            eff = {o: {lag: self.est[name][a][lag][o] for lag in case.truth[o]} for o in v2.OUTCOMES}
            out[name] = {"effects": eff, "shift": self.est[name][a]["30d"]["shift_day"]}
        eff, nb = {}, {}
        for lag in LAGS:
            treated, controls = self.stacks[lag]
            nt = nearest(case.window, treated[a], self.inv_std)
            nc = nearest(case.window, controls.get(case.cast_row, []), self.inv_std)
            nb[lag] = {"treated": [f"{u.seed}:{u.row}" for u in nt], "controls": [f"{u.seed}:{u.row}" for u in nc]}
            for k in KEYS:
                val = (float(np.mean([u.dy[k] for u in nt])) - float(np.mean([u.dy[k] for u in nc]))) if nt and nc else 0.0
                eff.setdefault(k, {})[lag] = val
        out["match5"] = {"effects": {o: {lag: eff[o][lag] for lag in case.truth[o]} for o in v2.OUTCOMES},
                         "shift": eff["shift_day"]["30d"], "neighbours": nb}
        return out

    def predictions(self, cases: list[ef.Case]) -> dict:
        per = {x.key: self.predict(x) for x in cases}
        return {n: {k: v[n] for k, v in per.items()} for n in BASELINES}


def select(val_cases: list[ef.Case], val_preds: dict, stds: dict) -> dict:
    """Per lag, the baseline with the lowest 12-cell average on the val families (ties in BASELINES order)."""
    worlds = sorted({x.seed for x in val_cases})
    out = {}
    for lag, cells in CELLS.items():
        vals = {n: ef.cell_stat(ef.cell_errors(val_cases, p, stds), cells, worlds) for n, p in val_preds.items()}
        out[lag] = {"chosen": min(BASELINES, key=lambda n: (vals[n], BASELINES.index(n))), "val": vals}
    return out


def inv_std_of_worlds(worlds: list[World]) -> np.ndarray:
    return ef.inv_std_of([w.v for w in worlds])


# ------------------------------------------------------------------ tables ------------------------------------------
def table(cases: list[ef.Case], preds: dict, stds: dict) -> dict:
    """WM-2's per-cell summary with arms labelled by their obs names, plus mean predicted / true effects."""
    errs = ef.cell_errors(cases, preds, stds)
    out = {}
    for cell, rows in errs.items():
        name = "|".join((OBS_OF.get(cell[0], cell[0]),) + tuple(cell[1:]))
        node = ef.cell_summary(rows)
        node["mean_pred"] = float(np.mean([r[4] for r in rows]))
        node["mean_true"] = float(np.mean([r[3] for r in rows]))
        out[name] = node
    return out


def distribution(cases: list[ef.Case]) -> dict:
    d = ef.distribution(cases)
    return {OBS_OF.get(k, k): v for k, v in d.items()}


# ------------------------------------------------------------------ commands ----------------------------------------
def _policy(path: Path) -> dict:
    return json.loads(Path(path).read_text())


def cmd_describe(a) -> int:
    t0 = time.time()
    policy = _policy(Path(a.policy))
    base = json.loads(Path(a.base).read_text())
    floored, unfloored = scales(base)
    worlds = load_worlds(Path(a.root), policy, "train")
    val = load_cases(Path(a.root), "val")
    out = {"code_commit": c.git_commit(), "argv": sys.argv, "policy_summary": policy["summary"],
           "terciles": policy["terciles"], "scales": scales_json(floored), "scales_unfloored": scales_json(unfloored),
           "pooled": pooled_estimates(worlds)["counts"], "val_cases": len(val), "distribution_val": distribution(val),
           "seconds": time.time() - t0}
    c.write_json(Path(a.out) / "describe.json", out)
    print(f"describe done {time.time() - t0:.0f}s val cases {len(val)}")
    return 0


def cmd_base(a) -> int:
    t0 = time.time()
    policy = _policy(Path(a.policy))
    base = json.loads(Path(a.base).read_text())
    floored, unfloored = scales(base)
    worlds = load_worlds(Path(a.root), policy, "train")
    inv_std = inv_std_of_worlds(worlds)
    bl = Baselines(worlds, inv_std)
    val, test = load_cases(Path(a.root), "val"), load_cases(Path(a.root), "test")
    val_preds, test_preds = bl.predictions(val), bl.predictions(test)
    out = {"code_commit": c.git_commit(), "argv": sys.argv, "inv_std": inv_std.tolist(),
           "scales": scales_json(floored), "scales_unfloored": scales_json(unfloored),
           "estimates": {k: bl.est[k] for k in ("naive", "ipw", "counts")},
           "cases": {"val": [ef.case_json(x) for x in val], "test": [ef.case_json(x) for x in test]},
           "distribution": {"val": distribution(val), "test": distribution(test)},
           "val_predictions": val_preds, "test_predictions": test_preds,
           "selection": select(val, val_preds, floored),
           "selection_unfloored": select(val, val_preds, {k: v for k, v in unfloored.items()}),
           "val_tables": {n: table(val, p, floored) for n, p in val_preds.items()},
           "policy_sha256": ef._sha(Path(a.policy)), "base_sha256": ef._sha(Path(a.base)), "seconds": time.time() - t0}
    target = Path(a.out) / "obs_base.json"
    if target.exists():
        raise SystemExit(f"{target} exists: computed once")
    c.write_json(target, out)
    print(json.dumps(out["selection"]))
    return 0


def load_obs_base(path: Path) -> tuple[list, list, dict, dict]:
    j = json.loads(Path(path).read_text())
    cases = {r: [ef.case_from(x) for x in j["cases"][r]] for r in ("val", "test")}
    back = lambda s: {(TO_V2[k.split("|")[0]], k.split("|")[1], k.split("|")[2]): v for k, v in s.items()}
    return cases["val"], cases["test"], back(j["scales"]), back(j["scales_unfloored"])


def cmd_m1(a) -> int:
    """An M1 model (WM-2 M1b-act, WM-3 `wm3_*`, WM-4 M1b-obs) on the obs test cases: `effects3.predictions` directly
    (`effects3._load` refuses datasets other than v2_v19)."""
    from wm1 import m1_gru as g
    from wm3 import effects3 as e3
    from wm3 import model  # noqa: F401  (registers m1_tfh)
    from wm4 import m1_obs
    m1_obs.register()
    t0 = time.time()
    target = Path(a.out) / "effects.json"
    if target.exists():
        raise SystemExit(f"{target} exists: computed once per run")
    pred, config = g.load_predictor(Path(a.run), a.device)
    hz = e3.is_hazard_run(config)
    _, test, _, _ = load_obs_base(Path(a.obs_base))
    tests = c.load_groups(["test"], Path(a.root), "obs_v19")
    test_std = b.split_std([ep.bench for ep in tests["test"]])
    fams = e3.load_families(family_dirs(Path(a.root), "test"), "test")
    preds, drift = e3.predictions(pred, fams, test, None, test_std, hazard=hz)
    out = {"run": a.run, "model": config["model"], "dataset": config.get("dataset"), "seed": config["seed"],
           "hazard": hz, "selected": config.get("selected"), "round_counts": pred.round_counts,
           "code_commit": c.git_commit(), "train_code_commit": config.get("code_commit"), "argv": sys.argv,
           "predictions": preds, "drift": drift, "test_std": test_std.tolist(),
           "obs_base_sha256": ef._sha(Path(a.obs_base)), "seconds": time.time() - t0}
    if target.exists():
        raise SystemExit(f"{target} exists: computed once per run")
    c.write_json(target, out)
    print(f"m1 obs effects done {time.time() - t0:.0f}s")
    return 0


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    for name in ("describe", "base"):
        p = sub.add_parser(name)
        p.add_argument("--root", default=str(c.DEFAULT_ROOT))
        p.add_argument("--base", required=True, help="WM-2 effects_base.json (scales)")
        p.add_argument("--policy", required=True)
        p.add_argument("--out", required=True)
    p = sub.add_parser("m1")
    p.add_argument("--run", required=True)
    p.add_argument("--root", default=str(c.DEFAULT_ROOT))
    p.add_argument("--obs-base", dest="obs_base", required=True, help="obs_base.json (cases)")
    p.add_argument("--device", default="auto")
    p.add_argument("--out", required=True)
    a = ap.parse_args(argv)
    return {"describe": cmd_describe, "base": cmd_base, "m1": cmd_m1}[a.cmd](a)


if __name__ == "__main__":
    raise SystemExit(main())
