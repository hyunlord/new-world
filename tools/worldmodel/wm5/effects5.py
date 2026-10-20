"""WM-5 effect evaluation (docs/worldmodel/wm5/GOAL.md 4.1, 7.1–7.3): obs2 cases, effect floors, the multi-cast
observational baselines, the direct effect regressions and the M1 models' predictions on obs2. numpy only (a model enters
through `wm3.effects3`).

**obs2 cases** (7.1): (world, ref, arm) for every val / test world, ref in (c1, c2), arm in (`gf240`, `sp240`, `rw120`),
and each recorded replicate k; truth = arm_k − `pre`_k at c + 168 and c + 720 rows for WM-2's four outcomes (the
restricted split shift as WM-2, reported only). A case's `rep` encodes (ref, k) as 2 · ref index + k (c1 r0 → 0, c1 r1
→ 1, c2 r0 → 2, c2 r1 → 3), so case keys and WM-2's world-level bootstrap work unchanged; its arm is rewritten to the
v2 key (`wm4.effects4.to_v2`). Model predictions read the `pre` member of the same (ref, k) (`FamilyData.factual[rep]`).
Test r0 cases are the primary cases; test r1 cases appear only in the floor.

**Effect floor** (7.1): each case is predicted by the other replicate's true effect (arm r1 − `pre` r1 for an r0 case);
on v2 each test case by the other replicate of its family (WM-2's two replicates). Scored with the same cells and scales.

**Scales** (4.1): obs2 uses WM-4's floored scales (`wm4.effects4.scales`); v2 WM-2's.

**Observational baselines** (7.2), from the observed **train** episodes and `policy.json` only. Risk set: every
decision row c of a world (days 8–60, p_tot(c) > 0: not refractory, capped or a non-decision; DECISIONS L2.1) with
c + h <= 2,880. Treated T_a: the world cast a at c and no other cast in (c, c + h); controls C: no cast in [c, c + h).
Weights (history up to c conditioned on; earlier casts allowed): treated 1 / (p_a(c) · Π_{j ∈ (c, c+h)} (1 − p_tot(j))),
controls 1 / Π_{j ∈ [c, c+h)} (1 − p_tot(j)) over decision rows j (refractory rows contribute 1). `naive` = unweighted
difference of means, `ipw` = the Hájek difference, `match5` = WM-4's rule (5 nearest treated of the arm by an5's
24-row distance at their cast rows, 5 nearest controls at row c; obs2 train inv_std), `zero`. Each lag's baseline is
chosen on the obs2 **val** families' r0 cases (12-cell average).

**Direct effect regressions** (7.3): one ridge regression per (arm, outcome, lag) cell on the v2 **train** families
(both replicates, every case), target = the true effect / the cell's scale (WM-2's v2 std), features normalized by the v2
train stds: the 37 state features at c, their means over rows c − 23 … c and c − 167 … c, the 30 calendar features at c,
and the counts of earlier casts per arm and the days since the last cast (0 on v2). λ ∈ {0.01, …, 1,000} chosen per cell
on the v2 val families (scaled MAE); applied unchanged to the v2 and obs2 tests. `ridge_priv` (diagnostic) adds every
hidden field at c (WM-4's leakage list: the hidden global columns, `latent.npy`, the hidden settlement columns summed and
maxed over settlements, the static `world_variety` values).

usage (DGX):
  effects5.py describe --root ROOT --base effects_base.json --out DIR      # DATASET_OBS2 numbers (train + val only)
  effects5.py base --root ROOT --base effects_base.json --out DIR          # cases, baselines, val selection, test preds
  effects5.py ridge --root ROOT --base effects_base.json --obs2-base obs2_base.json --out DIR [--priv]
  effects5.py m1 --run RUN --root ROOT --obs2-base obs2_base.json --out DIR [--name NAME]
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

import wm5  # noqa: F401
import bench_v1 as bv1
from wm_data import load_episode, load_family
from wm1 import common as c
from wm1 import data as d
from wm2 import effects as ef
from wm2 import v2
from wm4 import effects4 as e4
from wm5 import obs2 as o2
from wm5 import policy2 as p2

OBS_ARMS = p2.ARMS
TO_V2 = p2.TO_V2
OBS_OF = {v: k for k, v in TO_V2.items()}
LAGS = v2.LAGS
CELLS = e4.CELLS
BASELINES = e4.BASELINES
REF_INDEX = {"c1": 0, "c2": 1}
LAMBDAS = (0.01, 0.1, 1.0, 10.0, 100.0, 1000.0)
SCENE = "game_v19"
ARRAYS = ef.ARRAYS
ARRAYS_PRIV = ef.ARRAYS + ["latent.npy"]
HIDDEN_GLOBAL = ("winter_severity", "night_burn_per_hour", "winter_fall_today")
HIDDEN_SETTLEMENT = ("wood_target", "fission_pressure", "fission_threshold")
VARIANT_NAME = "sev-blind_action-aware"


def rep_code(ref: str, k: int) -> int:
    return 2 * REF_INDEX[ref] + int(k)


def ref_of(rep: int) -> tuple[str, int]:
    return ("c1", "c2")[rep // 2], rep % 2


def other_rep(rep: int) -> int:
    return rep ^ 1


# ------------------------------------------------------------------ obs2 families -----------------------------------
@dataclasses.dataclass
class WorldFam:
    """One val / test world's families: per ref, its reference row and the members {`pre`, arm: {k: Episode}}."""
    seed: int
    role: str
    refs: dict            # ref -> row
    members: dict         # ref -> {"pre" / arm: {k: wm_data.Episode}}
    pre_casts: dict       # ref -> the casts before c (policy.json)


def load_world_fams(root: Path, role: str, arrays=ARRAYS, verify: bool = True, worlds: list | None = None) -> list:
    root = Path(root)
    policy = json.loads((o2_root(root) / "policy" / "policy.json").read_text())
    out = []
    for w in policy["worlds"]:
        if w["role"] != role or (worlds is not None and w["seed"] not in worlds):
            continue
        mem, refs, pres = {}, {}, {}
        for ref in o2.REFS:
            path = o2.fam_dir(o2_root(root), role, ref) / "fam" / f"{SCENE}_s{w['seed']}"
            fam = load_family(path, verify=verify, arrays=arrays)
            mem[ref] = o2.members(fam)
            refs[ref] = int(w["refs"][ref])
            pres[ref] = p2.pre_casts(w["casts"], refs[ref])
            if fam.fork_tick != refs[ref] * 60:
                raise ValueError(f"{path}: fork {fam.fork_tick} != ref {refs[ref] * 60}")
        out.append(WorldFam(int(w["seed"]), role, refs, mem, pres))
    return out


def o2_root(root: Path) -> Path:
    return Path(root) / "obs2"


def world_cases(wf: WorldFam) -> list[ef.Case]:
    """Every (ref, arm, replicate) case of one world with its truths, shift and an5 window (of `pre`_k)."""
    out = []
    for ref in o2.REFS:
        row = wf.refs[ref]
        pre = wf.members[ref]["pre"]
        for k, ep_p in sorted(pre.items()):
            ps = v2.member_series(ep_p)
            vnone = bv1.episode_arrays(ep_p)["v"]
            dn, cn = v2.restricted_split_day(ps["fissions_total"], row)
            for arm in OBS_ARMS:
                ep_a = wf.members[ref][arm][k]
                a_s = v2.member_series(ep_a)
                truth = {o: {name: float(a_s[o][row + lag] - ps[o][row + lag]) for name, lag in LAGS.items()
                             if row + lag <= v2.LAST_ROW} for o in v2.OUTCOMES}
                da, ca = v2.restricted_split_day(a_s["fissions_total"], row)
                out.append(ef.Case(wf.seed, TO_V2[arm], rep_code(ref, k), wf.role, row * 60, row, truth,
                                   float(da - dn), (ca, cn), vnone[row - 23: row + 1].copy()))
    return out


def family_data(wfs: list[WorldFam], role: str, replicate: int | None = None) -> list:
    """`wm3.effects3.FamilyData` per world with `factual[rep]` = the `pre` member of (ref, k) as `EpisodeData` (the
    history models roll out from) and `truth[rep]` its truth series."""
    from wm3 import effects3 as e3
    out = []
    for wf in wfs:
        fact, truth = {}, {}
        for ref in o2.REFS:
            for k, ep in sorted(wf.members[ref]["pre"].items()):
                if replicate is not None and k != replicate:
                    continue
                r = rep_code(ref, k)
                fact[r] = d.from_episode(ep, "obs2_fam", role, wf.seed, "none", c.V17_EXTRA)
                truth[r] = v2.member_series(ep)
        out.append(e3.FamilyData(wf.seed, fact, truth))
    return out


def obs2_eval_inputs(root: Path, cases_json: str | Path, role: str = "test", replicate: int = 0):
    """(cases of replicate `replicate`, FamilyData) for a model evaluation on obs2 (cases from `obs2_base.json`)."""
    j = json.loads(Path(cases_json).read_text())
    cases = [ef.case_from(x) for x in j["cases"][role] if ref_of(x["rep"])[1] == replicate]
    wfs = load_world_fams(root, role, worlds=sorted({x.seed for x in cases}))
    return cases, family_data(wfs, role, replicate)


# ------------------------------------------------------------------ floors ------------------------------------------
def floor_predictions(cases: list[ef.Case]) -> dict:
    """Each case predicted by the true effect of the other replicate of the same (world, ref, arm) (7.1)."""
    by = {(x.seed, x.arm, x.rep): x for x in cases}
    out = {}
    for x in cases:
        o = by.get((x.seed, x.arm, other_rep(x.rep)))
        if o is None:
            continue
        out[x.key] = {"effects": {og: {lag: o.truth[og][lag] for lag in x.truth[og] if lag in o.truth[og]}
                                  for og in v2.OUTCOMES}, "shift": o.shift}
    return out


# ------------------------------------------------------------------ multi-cast baselines ---------------------------
@dataclasses.dataclass
class World5:
    """One observed train world: per-row truth series, bench V, its casts and per-row propensities."""
    seed: int
    series: dict          # outcome -> [T] (v2.member_series of the observed episode)
    v: np.ndarray         # [T, 8]
    casts: dict           # row -> obs arm
    probs: dict           # decision row -> (p_g, p_s, p_r) (0 for refractory / cap / no-decision rows)


@dataclasses.dataclass
class Unit:
    seed: int
    row: int
    dy: dict
    weight: float
    window: np.ndarray


def dy_of(world: World5, c_row: int, h: int) -> dict:
    out = {o: float(world.series[o][c_row + h] - world.series[o][c_row]) for o in v2.OUTCOMES}
    if "fissions_total" in world.series:
        out["shift_day"] = float(v2.restricted_split_day(world.series["fissions_total"], c_row)[0])
    return out


def stack(worlds: list[World5], h: int, decision_rows=p2.DECISION_ROWS, last_row: int = v2.LAST_ROW,
          survival: bool = True) -> tuple[dict, dict]:
    """(treated {obs arm: [Unit]}, controls {row c: [Unit]}) at horizon h (rows), GOAL.md 7.2's weights. `survival`
    False drops the post-cast survival factor of the treated (a unit test shows that this biases `ipw`)."""
    treated = {a: [] for a in OBS_ARMS}
    controls: dict = {}
    for w in worlds:
        cast_rows = sorted(w.casts)
        for c_row in decision_rows:
            if c_row + h > last_row:
                continue
            p = w.probs.get(c_row, (0.0, 0.0, 0.0))
            if sum(p) <= 0.0:
                continue                                   # not at risk (refractory, capped or no decision)
            win = w.v[max(0, c_row - 23): c_row + 1]
            later = [r for r in cast_rows if c_row < r < c_row + h]
            inside = [j for j in decision_rows if c_row < j < c_row + h]
            if c_row in w.casts and not later:
                a = w.casts[c_row]
                s = np.prod([1.0 - sum(w.probs.get(j, (0.0, 0.0, 0.0))) for j in inside]) if survival else 1.0
                treated[a].append(Unit(w.seed, c_row, dy_of(w, c_row, h), 1.0 / (p[OBS_ARMS.index(a)] * s), win))
            if c_row not in w.casts and not later:
                pi = (1.0 - sum(p)) * np.prod([1.0 - sum(w.probs.get(j, (0.0, 0.0, 0.0))) for j in inside])
                controls.setdefault(c_row, []).append(Unit(w.seed, c_row, dy_of(w, c_row, h), 1.0 / pi, win))
    return treated, controls


wmean, ess, nearest = e4.wmean, e4.ess, e4.nearest


def keys_of(units: list[Unit]) -> tuple:
    return tuple(units[0].dy) if units else v2.OUTCOMES


def pooled_estimates(worlds: list[World5], **kw) -> dict:
    out = {"naive": {}, "ipw": {}, "counts": {}}
    for lag, h in LAGS.items():
        treated, controls = stack(worlds, h, **kw)
        allc = [u for us in controls.values() for u in us]
        for a in OBS_ARMS:
            t = treated[a]
            ks = keys_of(t)
            out["naive"].setdefault(a, {})[lag] = {k: wmean(t, k, False) - wmean(allc, k, False) for k in ks} if t else None
            out["ipw"].setdefault(a, {})[lag] = {k: wmean(t, k, True) - wmean(allc, k, True) for k in ks} if t else None
            out["counts"].setdefault(a, {})[lag] = {
                "treated": len(t), "controls": len(allc), "ess_treated": ess(t), "ess_controls": ess(allc),
                "max_weight_treated": max((u.weight for u in t), default=None),
                "max_weight_controls": max((u.weight for u in allc), default=None),
                "treated_with_earlier_cast": sum(1 for u in t if any(r < u.row for r in
                                                                      next(w for w in worlds if w.seed == u.seed).casts))}
    return out


def load_worlds5(root: Path, role: str = "train") -> list[World5]:
    policy = json.loads((o2_root(root) / "policy" / "policy.json").read_text())
    out = []
    for w in policy["worlds"]:
        if w["role"] != role:
            continue
        ep = load_episode(o2_root(root) / w["observed"], arrays=ARRAYS)
        out.append(World5(int(w["seed"]), v2.member_series(ep), bv1.episode_arrays(ep)["v"],
                          {int(x["row"]): x["arm"] for x in w["casts"]}, p2.prob_table(w["rows"])))
    return out


class Baselines:
    """The four baselines' predictions for a case (keyed by `Case.key`)."""

    def __init__(self, worlds: list[World5], inv_std: np.ndarray):
        self.inv_std = inv_std
        self.est = pooled_estimates(worlds)
        self.stacks = {lag: stack(worlds, h) for lag, h in LAGS.items()}

    def control_row(self, lag: str, row: int) -> int:
        """Controls are matched at row c; a reference row past day 60 (c2 = c1 + 14 days) uses the last decision row
        (DECISIONS L2.2)."""
        rows = self.stacks[lag][1]
        return row if row in rows else max(r for r in rows if r <= row) if any(r <= row for r in rows) else min(rows)

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
            crow = self.control_row(lag, case.cast_row)
            nc = nearest(case.window, controls.get(crow, []), self.inv_std)
            nb[lag] = {"treated": [f"{u.seed}:{u.row}" for u in nt], "controls": [f"{u.seed}:{u.row}" for u in nc],
                       "control_row": crow}
            for k in v2.OUTCOMES + ("shift_day",):
                val = (float(np.mean([u.dy[k] for u in nt])) - float(np.mean([u.dy[k] for u in nc]))) if nt and nc else 0.0
                eff.setdefault(k, {})[lag] = val
        out["match5"] = {"effects": {o: {lag: eff[o][lag] for lag in case.truth[o]} for o in v2.OUTCOMES},
                         "shift": eff["shift_day"]["30d"], "neighbours": nb}
        return out

    def predictions(self, cases: list[ef.Case]) -> dict:
        per = {x.key: self.predict(x) for x in cases}
        return {n: {k: v[n] for k, v in per.items()} for n in BASELINES}


def select(val_cases: list[ef.Case], val_preds: dict, stds: dict) -> dict:
    return e4.select(val_cases, val_preds, stds)


# ------------------------------------------------------------------ direct effect regressions ----------------------
def obs_feature_names(state_names: tuple, calendar_names: tuple) -> list[str]:
    return ([f"{n}@c" for n in state_names] + [f"{n}@mean24" for n in state_names]
            + [f"{n}@mean168" for n in state_names] + [f"{n}@c" for n in calendar_names]
            + [f"casts_{a}" for a in OBS_ARMS] + ["days_since_cast"])


def priv_feature_names(world_variety_keys: tuple) -> list[str]:
    return ([f"hidden.g.{n}@c" for n in HIDDEN_GLOBAL] + [f"hidden.latent{i}@c" for i in range(7)]
            + [f"hidden.s.{n}.{agg}@c" for n in HIDDEN_SETTLEMENT for agg in ("sum", "max")]
            + [f"hidden.world_variety.{k}" for k in world_variety_keys])


def variant_of(epd: d.EpisodeData) -> d.Variant:
    return d.variant_for(VARIANT_NAME, epd.names)


def obs_features(epd: d.EpisodeData, row: int, pre_casts: list | tuple = ()) -> np.ndarray:
    """GOAL.md 7.3's observable features at reference row `row` of one episode (observable columns only: the 37 state
    features of the action-aware list, the calendar)."""
    st = epd.state(variant_of(epd)).astype(np.float64)
    cal = epd.calendar.astype(np.float64)
    counts = [sum(1 for x in pre_casts if x["arm"] == a) for a in OBS_ARMS]
    last = max((int(x["row"]) for x in pre_casts), default=None)
    since = 0.0 if last is None else (row - last) / 24.0
    return np.concatenate([st[row], st[max(0, row - 23): row + 1].mean(0), st[max(0, row - 167): row + 1].mean(0),
                           cal[row], counts, [since]])


def priv_features(ep, row: int) -> tuple[np.ndarray, tuple]:
    """The hidden fields at `row` of a recorded episode (`wm_data.Episode` with global, settlements and latent)."""
    g = [float(ep.column("global.npy", n)[row]) for n in HIDDEN_GLOBAL]
    lat = [float(x) for x in ep.arrays["latent.npy"][row]]
    s = []
    for n in HIDDEN_SETTLEMENT:
        col = np.asarray(ep.column("settlements.npy", n)[row], dtype=np.float64)
        ok = col[~np.isnan(col)]
        s += [float(ok.sum()) if len(ok) else 0.0, float(ok.max()) if len(ok) else 0.0]
    wv = ep.world_variety() or {}
    keys = tuple(sorted(k for k, v in wv.items() if isinstance(v, (int, float))))
    return np.array(g + lat + s + [float(wv[k]) for k in keys]), keys


class Ridge:
    """Closed-form ridge with an unpenalized intercept on standardized features (`mean`, `std` from the training
    rows; a constant feature keeps std 1)."""

    def __init__(self, lam: float):
        self.lam = float(lam)

    def fit(self, X: np.ndarray, y: np.ndarray, mean: np.ndarray, std: np.ndarray) -> "Ridge":
        Z = (X - mean) / std
        zm, ym = Z.mean(0), y.mean()
        Zc = Z - zm
        self.w = np.linalg.solve(Zc.T @ Zc + self.lam * np.eye(Z.shape[1]), Zc.T @ (y - ym))
        self.b = ym - zm @ self.w
        self.mean, self.std = mean, std
        return self

    def predict(self, X: np.ndarray) -> np.ndarray:
        return ((X - self.mean) / self.std) @ self.w + self.b


def norm_stats(X: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    mean, std = X.mean(0), X.std(0)
    return mean, np.where(std > 1e-12, std, 1.0)


def fit_cells(train: list, val: list, Xtr: dict, Xva: dict, stds: dict, cells: list) -> dict:
    """Per cell (v2 arm, outcome, lag): λ by the scaled val MAE, then the model on the train rows. `train` / `val`:
    cases; `X*`: case key -> feature vector."""
    out = {}
    keys = sorted({x.key for x in train})
    Xall = np.stack([Xtr[k] for k in keys])
    mean, std = norm_stats(Xall)
    for cell in cells:
        arm, o, lag = cell
        sd = stds.get(cell) or 1.0
        tr = [x for x in train if x.arm == arm and lag in x.truth[o]]
        va = [x for x in val if x.arm == arm and lag in x.truth[o]]
        X, y = np.stack([Xtr[x.key] for x in tr]), np.array([x.truth[o][lag] / sd for x in tr])
        Xv, yv = np.stack([Xva[x.key] for x in va]), np.array([x.truth[o][lag] / sd for x in va])
        scores = {lam: float(np.mean(np.abs(Ridge(lam).fit(X, y, mean, std).predict(Xv) - yv))) for lam in LAMBDAS}
        lam = min(LAMBDAS, key=lambda v: (scores[v], v))
        out[cell] = {"model": Ridge(lam).fit(X, y, mean, std), "lambda": lam, "val_mae": scores, "scale": sd,
                     "n_train": len(tr), "n_val": len(va)}
    return out


def ridge_predict(fits: dict, cases: list, X: dict) -> dict:
    out = {}
    for x in cases:
        eff = {o: {} for o in v2.OUTCOMES}
        for o in v2.OUTCOMES:
            for lag in x.truth[o]:
                f = fits.get((x.arm, o, lag))
                eff[o][lag] = float(f["model"].predict(X[x.key][None])[0] * f["scale"]) if f else 0.0
        out[x.key] = {"effects": eff, "shift": 0.0}
    return out


FEATURE_NAMES: dict = {}


def v2_features(root: Path, cases: list, role: str, priv: bool) -> dict:
    """Feature vectors of v2 cases: the `none_r<rep>` member of the case's family at its cast row (no earlier cast).
    The feature names are left in `FEATURE_NAMES["names"]`."""
    out = {}
    by: dict = {}
    for x in cases:
        by.setdefault((x.seed, x.rep), []).append(x)
    fam_root = Path(root) / "v2" / "v19" / "fam"
    for (seed, rep), xs in sorted(by.items()):
        p = fam_root / f"{SCENE}_s{seed}" / f"{SCENE}_s{seed}_none_r{rep}"
        ep = load_episode(p, arrays=ARRAYS_PRIV if priv else ARRAYS)
        epd = d.from_episode(ep, "v19_fam", role, seed, "none", c.V17_EXTRA)
        if "names" not in FEATURE_NAMES:
            var = variant_of(epd)
            names = obs_feature_names(var.state_names, tuple(var.input_names()[len(var.state_names):
                                                                                len(var.state_names) + 30]))
            FEATURE_NAMES["names"] = names + (priv_feature_names(priv_features(ep, xs[0].cast_row)[1]) if priv else [])
        for x in xs:
            f = obs_features(epd, x.cast_row)
            out[x.key] = np.concatenate([f, priv_features(ep, x.cast_row)[0]]) if priv else f
    return out


def obs2_features(root: Path, cases: list, role: str, priv: bool) -> dict:
    out = {}
    wfs = {w.seed: w for w in load_world_fams(root, role, ARRAYS_PRIV if priv else ARRAYS,
                                               worlds=sorted({x.seed for x in cases}))}
    cache: dict = {}
    for x in cases:
        ref, k = ref_of(x.rep)
        wf = wfs[x.seed]
        if (x.seed, x.rep) not in cache:
            ep = wf.members[ref]["pre"][k]
            epd = d.from_episode(ep, "obs2_fam", role, x.seed, "none", c.V17_EXTRA)
            f = obs_features(epd, x.cast_row, wf.pre_casts[ref])
            cache[(x.seed, x.rep)] = np.concatenate([f, priv_features(ep, x.cast_row)[0]]) if priv else f
        out[x.key] = cache[(x.seed, x.rep)]
    return out


# ------------------------------------------------------------------ tables / describe ------------------------------
table, distribution = e4.table, e4.distribution


def describe(root: Path, base: dict) -> dict:
    """DATASET_OBS2's numbers from the train worlds and the val families only (the test distribution is added after
    METRICS part b)."""
    policy = json.loads((o2_root(root) / "policy" / "policy.json").read_text())
    worlds = load_worlds5(root, "train")
    val = [x for wf in load_world_fams(root, "val") for x in world_cases(wf)]
    floored, unfloored = e4.scales(base)
    return {"policy_summary": policy["summary"], "rounds": policy["rounds"], "terciles": policy["terciles"],
            "pooled": pooled_estimates(worlds)["counts"], "val_cases": len(val),
            "distribution_val": {ref: distribution([x for x in val if ref_of(x.rep)[0] == ref]) for ref in o2.REFS}
            | {"all": distribution(val)},
            "scales": e4.scales_json(floored), "scales_unfloored": e4.scales_json(unfloored)}


# ------------------------------------------------------------------ commands ----------------------------------------
def cmd_describe(a) -> int:
    t0 = time.time()
    out = describe(Path(a.root), json.loads(Path(a.base).read_text()))
    out |= {"code_commit": c.git_commit(), "argv": sys.argv, "seconds": time.time() - t0}
    c.write_json(Path(a.out) / "describe.json", out)
    print(f"describe done {time.time() - t0:.0f}s val cases {out['val_cases']}")
    return 0


def cmd_base(a) -> int:
    t0 = time.time()
    root = Path(a.root)
    target = Path(a.out) / "obs2_base.json"
    if target.exists():
        raise SystemExit(f"{target} exists: computed once")
    base = json.loads(Path(a.base).read_text())
    floored, unfloored = e4.scales(base)
    worlds = load_worlds5(root, "train")
    inv_std = ef.inv_std_of([w.v for w in worlds])
    bl = Baselines(worlds, inv_std)
    val = [x for wf in load_world_fams(root, "val") for x in world_cases(wf)]
    test = [x for wf in load_world_fams(root, "test") for x in world_cases(wf)]
    test0 = [x for x in test if ref_of(x.rep)[1] == 0]
    val_preds, test_preds = bl.predictions(val), bl.predictions(test0)
    out = {"code_commit": c.git_commit(), "argv": sys.argv, "inv_std": inv_std.tolist(),
           "scales": e4.scales_json(floored), "scales_unfloored": e4.scales_json(unfloored),
           "estimates": {k: bl.est[k] for k in ("naive", "ipw", "counts")},
           "cases": {"val": [ef.case_json(x) for x in val], "test": [ef.case_json(x) for x in test]},
           "rep_codes": {"c1": [0, 1], "c2": [2, 3], "note": "rep = 2 * ref index + replicate"},
           "distribution": {"val": distribution(val), "test": distribution(test0),
                            "test_by_ref": {ref: distribution([x for x in test0 if ref_of(x.rep)[0] == ref])
                                            for ref in o2.REFS}},
           "val_predictions": val_preds, "test_predictions": test_preds,
           "floor_predictions": floor_predictions(test),
           "selection": select(val, val_preds, floored),
           "selection_unfloored": select(val, val_preds, unfloored),
           "val_tables": {n: table(val, p, floored) for n, p in val_preds.items()},
           "policy_sha256": ef._sha(o2_root(root) / "policy" / "policy.json"), "base_sha256": ef._sha(Path(a.base)),
           "seconds": time.time() - t0}
    if target.exists():
        raise SystemExit(f"{target} exists: computed once")
    c.write_json(target, out)
    print(json.dumps(out["selection"]))
    return 0


def load_obs2_base(path: Path) -> tuple[list, list, dict, dict]:
    """(val cases, test cases of every replicate, floored scales, unfloored scales) keyed by v2 arms."""
    j = json.loads(Path(path).read_text())
    cases = {r: [ef.case_from(x) for x in j["cases"][r]] for r in ("val", "test")}
    back = lambda s: {(TO_V2[k.split("|")[0]], k.split("|")[1], k.split("|")[2]): v for k, v in s.items()}
    return cases["val"], cases["test"], back(j["scales"]), back(j["scales_unfloored"])


def cmd_ridge(a) -> int:
    """`ridge_obs` (or `ridge_priv` with --priv): fitted on the v2 train families, λ on the v2 val families, applied to
    the v2 test cases (both replicates) and the obs2 test r0 cases."""
    t0 = time.time()
    root = Path(a.root)
    name = "ridge_priv" if a.priv else "ridge_obs"
    target = Path(a.out) / f"{name}.json"
    if target.exists():
        raise SystemExit(f"{target} exists: computed once")
    base = json.loads(Path(a.base).read_text())
    stds = ef.stds_from(base["stds"])
    tr, va, te = ([ef.case_from(x) for x in base["cases"][r]] for r in ("train", "val", "test"))
    cells = [cell for lag in LAGS for cell in CELLS[lag]]
    Xtr, Xva, Xte = (v2_features(root, cs, r, a.priv) for cs, r in ((tr, "train"), (va, "val"), (te, "test")))
    print(f"{name}: v2 features {time.time() - t0:.0f}s", flush=True)
    fits = fit_cells(tr, va, Xtr, Xva, stds, cells)
    _, o2test, _, _ = load_obs2_base(Path(a.obs2_base))
    o2test0 = [x for x in o2test if ref_of(x.rep)[1] == 0]
    Xo = obs2_features(root, o2test0, "test", a.priv)
    out = {"name": name, "diagnostic": bool(a.priv), "code_commit": c.git_commit(), "argv": sys.argv,
           "lambdas": {"|".join(k): v["lambda"] for k, v in fits.items()},
           "val_mae": {"|".join(k): v["val_mae"] for k, v in fits.items()},
           "n_train": {"|".join(k): v["n_train"] for k, v in fits.items()},
           "coef_norm": {"|".join(k): float(np.linalg.norm(v["model"].w)) for k, v in fits.items()},
           "n_features": int(len(next(iter(Xtr.values())))), "feature_names": FEATURE_NAMES.get("names"),
           "v2_val_predictions": ridge_predict(fits, va, Xva),
           "v2_test_predictions": ridge_predict(fits, te, Xte),
           "obs2_test_predictions": ridge_predict(fits, o2test0, Xo),
           "base_sha256": ef._sha(Path(a.base)), "obs2_base_sha256": ef._sha(Path(a.obs2_base)),
           "seconds": time.time() - t0}
    if target.exists():
        raise SystemExit(f"{target} exists")
    c.write_json(target, out)
    print(f"{name} done {time.time() - t0:.0f}s; lambdas {out['lambdas']}")
    return 0


def cmd_m1(a) -> int:
    """An M1 model (WM-2 M1b-act, WM-3 `wm3_*`, WM-4 M1b-obs, WM-5 M1b-obs2) on the obs2 test r0 cases:
    `wm3.effects3.predictions` with 720-row rollouts (both lags; the split shift is not predicted: DECISIONS L2.4)."""
    from wm1 import m1_gru as g
    import bench_v0 as b
    from wm3 import effects3 as e3
    from wm3 import model  # noqa: F401  (registers m1_tfh)
    from wm4 import m1_obs
    from wm5 import m1_obs2
    m1_obs.register(); m1_obs2.register()
    t0 = time.time()
    target = Path(a.out) / f"{a.name or 'effects'}.json"
    if target.exists():
        raise SystemExit(f"{target} exists: computed once per run")
    pred, config = g.load_predictor(Path(a.run), a.device)
    hz = e3.is_hazard_run(config)
    cases, fams = obs2_eval_inputs(Path(a.root), a.obs2_base, "test", 0)
    tests = c.load_groups(["test"], Path(a.root), "obs2_v19")
    test_std = b.split_std([ep.bench for ep in tests["test"]])
    preds, drift = e3.predictions(pred, fams, cases, horizon=e3.VAL_EFF_HORIZON, test_std=test_std, hazard=False)
    out = {"run": a.run, "model": config["model"], "dataset": config.get("dataset"), "seed": config["seed"],
           "hazard": hz, "selected": config.get("selected"), "round_counts": pred.round_counts,
           "code_commit": c.git_commit(), "train_code_commit": config.get("code_commit"), "argv": sys.argv,
           "predictions": preds, "drift": drift, "test_std": test_std.tolist(), "horizon": e3.VAL_EFF_HORIZON,
           "obs2_base_sha256": ef._sha(Path(a.obs2_base)), "n_cases": len(cases), "seconds": time.time() - t0}
    if target.exists():
        raise SystemExit(f"{target} exists: computed once per run")
    c.write_json(target, out)
    print(f"m1 obs2 effects done {time.time() - t0:.0f}s")
    return 0


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    for name in ("describe", "base", "ridge"):
        p = sub.add_parser(name)
        p.add_argument("--root", default=str(c.DEFAULT_ROOT))
        p.add_argument("--base", required=True, help="WM-2 effects_base.json")
        p.add_argument("--out", required=True)
        if name == "ridge":
            p.add_argument("--obs2-base", dest="obs2_base", required=True)
            p.add_argument("--priv", action="store_true")
    p = sub.add_parser("m1")
    p.add_argument("--run", required=True)
    p.add_argument("--root", default=str(c.DEFAULT_ROOT))
    p.add_argument("--obs2-base", dest="obs2_base", required=True)
    p.add_argument("--device", default="auto")
    p.add_argument("--name", default=None)
    p.add_argument("--out", required=True)
    a = ap.parse_args(argv)
    return {"describe": cmd_describe, "base": cmd_base, "ridge": cmd_ridge, "m1": cmd_m1}[a.cmd](a)


if __name__ == "__main__":
    raise SystemExit(main())
