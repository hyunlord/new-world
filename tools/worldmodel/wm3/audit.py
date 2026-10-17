"""WM-3 Part 1 audit (docs/worldmodel/wm3/GOAL.md 6): is WM-2's wrong-signed one-step response to an intervention a bug
(data, harness) or a property of the trained models? Train and validation families only; no test family is opened.

A **case** is one (world, arm, replicate r) of `wm2.effects.load_cases`; its **arm member** and its **partner** (the
factual member `none` of the same world and replicate) are read the way training reads them: the manifest episodes of
dataset `v2_v19` (`wm1.data.load_split`), where `EpisodeData.eid` is the arm name (`schedule_id`) and the replicate comes
from the manifest split (`v19_fact` / `v19_arm_r0` -> 0, `v19_none_r1` / `v19_arm_r1` -> 1).

- **A1 data.** Structural: in the arm member's `d.prepare` output the action matrix is non-zero exactly at the cast row
  and in the arm's kind column, value requested / `ACTION_SCALE`; the partner's action matrix is zero; rows <= the cast
  row of `x_state` are equal between the two; the arm - partner stock difference at the cast row is exactly 0.
  Behavioural: (arm - partner stock at c + 1) / (s_kind x applied) in [0.5, 1.5] (s_kind = +1 for `gift_food`, -1 for
  `spoil_food` and `rot_wood`; the stock is the summed `s.food_stock`, or `s.wood_stock` for `rot_wood`).
- **A2 one-step response.** For each WM-2 M1b-act checkpoint: the teacher-forced predicted mean change at the cast row
  (`model.run` on rows c - 167 .. c, `valid` = row >= 0, exactly as `wm2.rollout_ext` warms the transformer up) of the 7
  continuous V variables, x `norm.dstd` (raw units), and the fire probability, for the arm member and the partner; per
  arm and role the mean true and predicted arm - partner jump, their correlation and the right-sign share.
- **A3 harness consistency.** Validation cases of replicate 0, `round_counts = False`: the lag-1 difference of
  `wm2.rollout_ext.rollout_jobs` (a factual job and an arm job from the partner, horizon 1, actions as
  `wm2.effects.model_predictions` builds them) against `Stepper.next_state(s_c, mu_A, fire_A) -
  Stepper.next_state(s_c, mu_F, fire_F)` from A2's outputs; tolerance 1e-4 x max(1, |s_c|) + 1e-3.

usage (DGX): python3 tools/worldmodel/wm3/audit.py --run RUN [RUN ...] --out DIR
"""
from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm3  # noqa: F401
import bench_v0 as b
from wm1 import common as c
from wm1 import data as d
from wm2 import v2

SPLIT_REP = {c.V19_FACT: 0, c.V19_ARM_R0: 0, c.V19_NONE_R1: 1, c.V19_ARM_R1: 1}
STOCK = {"gift_food": "s.food_stock", "spoil_food": "s.food_stock", "rot_wood": "s.wood_stock"}
SIGN = {"gift_food": 1.0, "spoil_food": -1.0, "rot_wood": -1.0}
CONT_V = tuple(n for n in d.V_STATE if n != d.FIRE)       # the 7 continuous V variables
RATIO_BAND = (0.5, 1.5)
BEHAVIOUR_SHARE = 0.005


def load_members(root: Path, roles: tuple[str, ...]) -> dict:
    """{(seed, eid, rep): EpisodeData} of every `v2_v19` member of the given roles, read as training reads them."""
    m = c.manifest("v2_v19")
    out = {}
    for split in c.V19_ALL:
        for ep in d.load_split(Path(root), m, split, roles=roles, extra=c.V17_EXTRA):
            key = (int(ep.seed), ep.eid, SPLIT_REP[ep.split])
            if key in out:
                raise ValueError(f"member {key} is not unique")
            out[key] = ep
    return out


def pair(case, members: dict):
    """(arm member, partner) of a case."""
    return members[(case.seed, case.arm, case.rep)], members[(case.seed, "none", case.rep)]


def a1_case(case, arm, partner, variant: d.Variant, norm: d.Normalizer) -> dict:
    """The structural and behavioural checks of one case (GOAL.md 6.1)."""
    kind, amount = v2.ARM_SPEC[case.arm]
    k = b.KINDS.index(kind)
    row = case.cast_row
    pa, pp = d.prepare(arm, variant, norm), d.prepare(partner, variant, norm)
    (rec,) = arm.interventions
    nz = {(int(r), int(j)) for r, j in zip(*np.nonzero(pa.actions))}
    want = float(rec["amount"]) / d.ACTION_SCALE[k]
    stock = arm.names.index(STOCK[kind])
    sa, sp = arm.table[:, stock], partner.table[:, stock]
    applied = float(rec["applied"])
    denom = SIGN[kind] * applied
    ratio = float((sa[row + 1] - sp[row + 1]) / denom) if denom else None
    ratio2 = float((sa[row + 2] - sp[row + 2]) / denom) if denom else None
    checks = {
        "action_position": nz == {(row, k)},
        "action_value": bool(pa.actions[row, k] == want) and float(rec["amount"]) == float(amount)
                        and rec["kind"] == kind and int(rec["tick"]) == case.cast_tick,
        "partner_zero_actions": not pp.actions.any() and not partner.interventions,
        "prefix_equal": bool(np.array_equal(pa.x_state[: row + 1], pp.x_state[: row + 1])
                             and np.array_equal(pa.calendar[: row + 1], pp.calendar[: row + 1])),
        "cast_row_diff_zero": bool(sa[row] - sp[row] == 0.0),
    }
    return {"key": case.key, "seed": case.seed, "arm": case.arm, "rep": case.rep, "role": case.role, "cast_row": row,
            "kind": kind, "requested": float(rec["amount"]), "applied": applied, "checks": checks,
            "ratio_c1": ratio, "ratio_c2": ratio2, "jump_c1": float(sa[row + 1] - sp[row + 1]),
            "behaviour_ok": ratio is not None and RATIO_BAND[0] <= ratio <= RATIO_BAND[1]}


def quantiles(x) -> dict:
    x = np.asarray(x, dtype=np.float64)
    return {"n": int(len(x)), "mean": float(x.mean()), "min": float(x.min()), "p01": float(np.percentile(x, 1)),
            "p10": float(np.percentile(x, 10)), "median": float(np.median(x)), "p90": float(np.percentile(x, 90)),
            "p99": float(np.percentile(x, 99)), "max": float(x.max())}


def a1_summary(rows: list[dict]) -> dict:
    names = list(rows[0]["checks"])
    out = {"cases": len(rows), "structural_mismatches": {n: int(sum(not r["checks"][n] for r in rows)) for n in names},
           "behaviour_outside": int(sum(not r["behaviour_ok"] for r in rows)), "per_arm": {}}
    out["structural_total"] = int(sum(out["structural_mismatches"].values()))
    out["behaviour_outside_share"] = out["behaviour_outside"] / len(rows)
    for arm in v2.ARMS:
        for role in ("train", "val"):
            rs = [r for r in rows if r["arm"] == arm and r["role"] == role]
            if not rs:
                continue
            out["per_arm"][f"{arm}|{role}"] = {
                "n": len(rs), "ratio_c1": quantiles([r["ratio_c1"] for r in rs]),
                "ratio_c2": quantiles([r["ratio_c2"] for r in rs]),
                "jump_c1_mean": float(np.mean([r["jump_c1"] for r in rs])),
                "outside": int(sum(not r["behaviour_ok"] for r in rs)),
                "structural": int(sum(not all(r["checks"].values()) for r in rs))}
    out["outside_cases"] = [{k: r[k] for k in ("key", "role", "cast_row", "applied", "ratio_c1", "ratio_c2", "jump_c1")}
                            for r in rows if not r["behaviour_ok"]]
    return out


# ------------------------------------------------------------------ A2 ---------------------------------------------
def window_input(prep: d.Prepared, row: int, warmup: int) -> tuple[np.ndarray, np.ndarray]:
    """The transformer warm-up input of a rollout from `row` (rows row - warmup + 1 .. row, clamped at 0) and its
    validity mask, as `wm2.rollout_ext` builds it."""
    rows = row - (warmup - 1) + np.arange(warmup)
    valid = rows >= 0
    r = np.clip(rows, 0, None)
    return d.model_input(prep.x_state[r], prep.calendar[r], prep.actions[r], prep.sev[r]), valid


def teacher_forced(pred, inputs: np.ndarray, valid: np.ndarray, chunk: int = 512) -> tuple[np.ndarray, np.ndarray]:
    """The model's normalized mean change and fire logit at the last position of every window ([n, n_cont], [n])."""
    from wm1 import m1_gru as g
    torch = g.torch
    pred.model.eval()
    mus, fls = [], []
    with torch.no_grad():
        for i in range(0, len(inputs), chunk):
            x = torch.tensor(inputs[i: i + chunk], dtype=torch.float32, device=pred.device)
            v = torch.tensor(valid[i: i + chunk], device=pred.device)
            mu, _, fl, _ = pred.model.run(x, valid=v)
            mus.append(mu[:, -1].double().cpu().numpy())
            fls.append(fl[:, -1].double().cpu().numpy())
    return np.concatenate(mus), np.concatenate(fls)


def v_cont_positions(lay: d.Layout) -> list[int]:
    """Positions of the 7 continuous V variables in the continuous feature list (the model's mean-change head)."""
    cont = list(lay.cont)
    return [cont.index(lay.names.index(n)) for n in CONT_V]


def a2_run(pred, cases: list, members: dict) -> dict:
    """One checkpoint's teacher-forced one-step response at every case (arm member and partner)."""
    lay, norm = pred.lay, pred.norm
    pos = v_cont_positions(lay)
    vidx = [lay.names.index(n) for n in CONT_V]
    preps: dict = {}

    def prep(ep):
        key = id(ep)
        if key not in preps:
            preps[key] = d.prepare(ep, pred.variant, norm)
        return preps[key]

    xs, vs = [], []
    for x in cases:
        arm, partner = pair(x, members)
        for ep in (arm, partner):
            a, v = window_input(prep(ep), x.cast_row, pred.warmup)
            xs.append(a)
            vs.append(v)
    mu, fl = teacher_forced(pred, np.stack(xs).astype(np.float32), np.stack(vs))
    raw = mu[:, pos] * np.asarray(norm.dstd)[pos]
    prob = 1.0 / (1.0 + np.exp(-fl))
    rows = []
    for i, x in enumerate(cases):
        arm, partner = pair(x, members)
        r = x.cast_row
        true_a = arm.table[r + 1, vidx] - arm.table[r, vidx]
        true_p = partner.table[r + 1, vidx] - partner.table[r, vidx]
        rows.append({"key": x.key, "arm": x.arm, "role": x.role, "rep": x.rep, "cast_row": r,
                     "mu_arm": mu[2 * i].tolist(), "mu_partner": mu[2 * i + 1].tolist(),
                     "fl_arm": float(fl[2 * i]), "fl_partner": float(fl[2 * i + 1]),
                     "pred_arm": raw[2 * i].tolist(), "pred_partner": raw[2 * i + 1].tolist(),
                     "true_arm": true_a.tolist(), "true_partner": true_p.tolist(),
                     "fire_arm": float(prob[2 * i]), "fire_partner": float(prob[2 * i + 1])})
    return rows


def a2_summary(rows: list[dict]) -> dict:
    out = {}
    for arm in v2.ARMS:
        for role in ("train", "val"):
            rs = [r for r in rows if r["arm"] == arm and r["role"] == role]
            if not rs:
                continue
            node = {"n": len(rs), "vars": {}}
            for j, name in enumerate(CONT_V):
                t = np.array([r["true_arm"][j] - r["true_partner"][j] for r in rs])
                p = np.array([r["pred_arm"][j] - r["pred_partner"][j] for r in rs])
                nz = t != 0
                corr = float(np.corrcoef(t, p)[0, 1]) if t.std() > 0 and p.std() > 0 else None
                node["vars"][name] = {"true_mean": float(t.mean()), "pred_mean": float(p.mean()), "corr": corr,
                                      "right_sign": float(np.mean(np.sign(p[nz]) == np.sign(t[nz]))) if nz.any() else None,
                                      "n_true_nonzero": int(nz.sum()), "pred_abs_mean": float(np.abs(p).mean()),
                                      "true_abs_mean": float(np.abs(t).mean())}
            node["fire"] = {"arm": float(np.mean([r["fire_arm"] for r in rs])),
                            "partner": float(np.mean([r["fire_partner"] for r in rs]))}
            out[f"{arm}|{role}"] = node
    return out


# ------------------------------------------------------------------ A3 ---------------------------------------------
def arm_actions(ep: d.EpisodeData, arm: str, row: int) -> tuple[np.ndarray, np.ndarray]:
    """The factual (zero) and the arm action arrays of a case, as `wm2.effects.model_predictions` builds them."""
    zeros = np.zeros_like(ep.bench.actions)
    act = zeros.copy()
    kind, amount = v2.ARM_SPEC[arm]
    act[row, b.KINDS.index(kind)] = float(amount)
    return zeros, act


def harness_lag1(pred, jobs_in: list[tuple], chunk: int = 256) -> list[tuple[np.ndarray, np.ndarray]]:
    """For each (partner EpisodeData, arm, cast row): the lag-1 V of the factual and the arm rollout
    (`wm2.rollout_ext.rollout_jobs`, horizon 1)."""
    from wm2 import rollout_ext as rx
    jobs = []
    for ep, arm, row in jobs_in:
        zeros, act = arm_actions(ep, arm, row)
        jobs += [rx.Job(ep, row, zeros, 1), rx.Job(ep, row, act, 1)]
    ros = rx.rollout_jobs(pred, jobs, chunk)
    return [(ros[2 * i].v[0, 0], ros[2 * i + 1].v[0, 0]) for i in range(len(jobs_in))]


def stepped_lag1(pred, s_c: np.ndarray, mu: np.ndarray, fl: float) -> np.ndarray:
    """`Stepper.next_state` of the raw state s_c with a normalized mean change and fire logit (the harness's clamp)."""
    from wm1 import m1_gru as g
    torch = g.torch
    st = pred.stepper
    t = lambda a: torch.tensor(np.asarray(a), dtype=torch.float32, device=pred.device)
    return st.next_state(t(s_c)[None], t(mu)[None], t([fl])).double().cpu().numpy()[0]


def a3_run(pred, cases: list, members: dict, a2rows: dict) -> dict:
    """Harness lag-1 differences vs the stepped A2 outputs on the val cases of replicate 0."""
    pred.round_counts = False
    lay = pred.lay
    vpos = [list(d.V_STATE).index(n) for n in CONT_V]       # positions in the rollout's 8-V output
    sidx = [lay.names.index(n) for n in CONT_V]
    sel = [x for x in cases if x.role == "val" and x.rep == 0]
    lag1 = harness_lag1(pred, [(pair(x, members)[1], x.arm, x.cast_row) for x in sel])
    viol, worst, rows = 0, 0.0, []
    for x, (fv, av) in zip(sel, lag1):
        partner = pair(x, members)[1]
        s_c = partner.table[x.cast_row]
        r2 = a2rows[x.key]
        nxt_a = stepped_lag1(pred, s_c, r2["mu_arm"], r2["fl_arm"])
        nxt_f = stepped_lag1(pred, s_c, r2["mu_partner"], r2["fl_partner"])
        harness = av[vpos] - fv[vpos]
        stepped = nxt_a[sidx] - nxt_f[sidx]
        tol = 1e-4 * np.maximum(1.0, np.abs(s_c[sidx])) + 1e-3
        bad = np.abs(harness - stepped) > tol
        viol += int(bad.any())
        worst = max(worst, float((np.abs(harness - stepped) / tol).max()))
        rows.append({"key": x.key, "harness": harness.tolist(), "stepped": stepped.tolist(), "violation": bool(bad.any())})
    return {"cases": len(sel), "violations": viol, "worst_ratio_to_tolerance": worst, "rows": rows}


# ------------------------------------------------------------------ decision ---------------------------------------
def decide(a1: dict, a3: dict) -> dict:
    """GOAL.md 6.4's rule from the counts (the hand inspection of >0.5 % behavioural outliers is recorded beside it)."""
    structural = a1["structural_total"]
    a3v = sum(r["violations"] for r in a3.values())
    behav = a1["behaviour_outside_share"]
    if structural or a3v:
        verdict = "bug"
    elif behav > BEHAVIOUR_SHARE:
        verdict = "inspect"
    else:
        verdict = "clean"
    return {"structural_mismatches": structural, "a3_violations": a3v, "behaviour_outside_share": behav,
            "verdict": verdict}


def write_compact(path: Path, obj) -> None:
    """A large per-case file: one line, numbers as `json` writes them (the summaries use `common.write_json`)."""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(obj, sort_keys=True, default=c._default, separators=(",", ":")) + "\n")


def main(argv=None) -> int:
    from wm1 import m1_gru as g
    from wm2 import effects as ef
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--run", nargs="+", required=True)
    ap.add_argument("--root", default=str(c.DEFAULT_ROOT))
    ap.add_argument("--device", default="auto")
    ap.add_argument("--out", required=True)
    a = ap.parse_args(argv)
    out_dir = Path(a.out)
    t0 = time.time()
    root = Path(a.root)
    cases = []
    for role in ("train", "val"):
        cases += ef.load_cases(root, role)[0]
    print(f"cases {len(cases)} {time.time() - t0:.0f}s", flush=True)
    members = load_members(root, ("train", "val"))
    print(f"members {len(members)} {time.time() - t0:.0f}s", flush=True)
    pred0, cfg0 = g.load_predictor(Path(a.run[0]), a.device)
    a1rows = [a1_case(x, *pair(x, members), pred0.variant, pred0.norm) for x in cases]
    a1 = a1_summary(a1rows)
    print(f"A1 structural {a1['structural_total']} behaviour outside {a1['behaviour_outside']} "
          f"{time.time() - t0:.0f}s", flush=True)
    meta = {"code_commit": c.git_commit(), "argv": sys.argv, "cases": {r: sum(x.role == r for x in cases)
                                                                       for r in ("train", "val")},
            "members": len(members)}
    c.write_json(out_dir / "a1.json", meta | {"summary": a1})
    write_compact(out_dir / "a1_cases.json", meta | {"rows": a1rows})
    a2, a3, a2_case = {}, {}, {}
    for run in a.run:
        name = Path(run).name
        pred, cfg = g.load_predictor(Path(run), a.device)
        if cfg["normalizer"] != cfg0["normalizer"] or cfg["input_names"] != cfg0["input_names"]:
            raise SystemExit(f"{name}: normalizer or input list differs from {Path(a.run[0]).name}")
        rows = a2_run(pred, cases, members)
        a2[name] = a2_summary(rows)
        a2_case[name] = [r for r in rows if r["role"] == "val"]
        a3[name] = a3_run(pred, cases, members, {r["key"]: r for r in rows if r["role"] == "val" and r["rep"] == 0})
        print(f"{name}: A2 {time.time() - t0:.0f}s; A3 violations {a3[name]['violations']} / {a3[name]['cases']} "
              f"worst {a3[name]['worst_ratio_to_tolerance']:.3g}", flush=True)
    c.write_json(out_dir / "a2.json", meta | {"vars": list(CONT_V), "summary": a2})
    write_compact(out_dir / "a2_cases_val.json", meta | {"vars": list(CONT_V), "rows": a2_case})
    c.write_json(out_dir / "a3.json", meta | {"vars": list(CONT_V), "summary": {n: {k: v for k, v in r.items() if k != "rows"}
                                                                               for n, r in a3.items()}})
    write_compact(out_dir / "a3_cases.json", meta | {"vars": list(CONT_V), "rows": {n: r["rows"] for n, r in a3.items()}})
    dec = decide(a1, a3) | {"seconds": time.time() - t0}
    c.write_json(out_dir / "audit.json", meta | {"decision": dec})
    print(json.dumps(dec))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
