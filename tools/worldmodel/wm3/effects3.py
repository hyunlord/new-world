"""WM-3 effect and split evaluation (docs/worldmodel/wm3/GOAL.md 7.5, 7.6) on pre-loaded families of any role.

`predictions(pred, families, cases, horizon=None)` is `wm2.effects.model_predictions` on families loaded once
(`load_families`), with `horizon=None` = LAST_ROW - row (WM-2) or a fixed rollout length (the fine-tuning `val_eff`:
720 rows, both lags read from one rollout; no split shift, no drift past the horizon). For a model without a hazard head
it reproduces `model_predictions` number for number (`wm2.rollout_ext.rollout_jobs`, the same jobs in the same order).

For a hazard model (`hazard=True`: an `M1TFH` trained with a hazard loss) the rollout also returns the hazard probability
h_k of every step k — the head's output at the position whose mean change produces row c + k — and the **expected
restricted split day** from reference row c: S_0 = 1, P_k = S_{k-1} h_k, S_k = S_{k-1} (1 - h_k),
E = sum_{k=1..K} P_k day(c + k) + S_K day(LAST_ROW), K = LAST_ROW - c, day = `v2.day_of_row`. Split shift = E_arm -
E_factual (`shift_expected`); WM-2's event-based shift stays in `shift`.

usage (DGX; every command imports `wm3.model` before `g.load_predictor`):
  python3 tools/worldmodel/wm3/effects3.py model --run RUN --base effects_base.json --out DIR   # test effects (once)
  python3 tools/worldmodel/wm3/effects3.py split --run RUN --out DIR                            # test split timing (once)
  python3 tools/worldmodel/wm3/effects3.py calib --run RUN --base effects_base.json --out DIR   # val hazard calibration
"""
from __future__ import annotations

import argparse
import json
import sys
import time
from dataclasses import dataclass
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm3  # noqa: F401
import bench_v0 as b
from wm_data import load_family
from wm1 import common as c
from wm1 import data as d
from wm2 import effects as ef
from wm2 import rollout_ext as rx
from wm2 import v2

VAL_EFF_HORIZON = 720
VAL_EFF_LAGS = ("7d", "30d")


@dataclass
class FamilyData:
    """One loaded family: per replicate the factual member as `EpisodeData` and its truth series."""
    seed: int
    factual: dict          # r -> EpisodeData
    truth: dict            # r -> v2.member_series


def load_families(paths, role: str, verify: bool = True) -> list[FamilyData]:
    """Load family directories once (sha256-verified as `model_predictions` does)."""
    out = []
    for path in paths:
        fam = load_family(path, verify=verify, arrays=ef.ARRAYS)
        out.append(FamilyData(fam.seed, {r: d.from_episode(raw, "v19_fam", role, fam.seed, "none", c.V17_EXTRA)
                                         for r, raw in sorted(fam.factual.items())},
                              {r: v2.member_series(raw) for r, raw in sorted(fam.factual.items())}))
    return out


def val_eff_cells(stds: dict) -> list:
    """The 24 cells of `val_eff`: 4 outcomes x the 3 W3 arms x {7 d, 30 d} (cells without a scale dropped)."""
    return [(arm, o, lag) for arm in v2.W3_ARMS for o in v2.OUTCOMES for lag in VAL_EFF_LAGS
            if stds[(arm, o, lag)] is not None]


# ------------------------------------------------------------------ hazard rollouts --------------------------------
def rollout_jobs_h(pred, jobs: list, chunk: int = 128) -> tuple[list, list]:
    """`rollout_jobs` for an `M1TFH` predictor, also returning each job's hazard probabilities [horizon] (the step-k
    value is the head's output at the position that produces row start + k)."""
    order = sorted(range(len(jobs)), key=lambda i: -jobs[i].horizon)
    ros: list = [None] * len(jobs)
    hs: list = [None] * len(jobs)
    for i in range(0, len(order), chunk):
        idx = order[i: i + chunk]
        for j, (ro, h) in zip(idx, _chunk_h(pred, [jobs[k] for k in idx])):
            ros[j], hs[j] = ro, h
    return ros, hs


def _chunk_h(pred, jobs: list) -> list:
    """`wm2.rollout_ext._chunk`'s transformer path with `run_h` (the same arithmetic) plus the hazard series."""
    from wm1 import m1_gru as g
    from wm1 import m1_tf
    torch = g.torch
    with torch.no_grad():
        model, lay, dev = pred.model, pred.lay, pred.device
        model.eval()
        T = {j.ep.rows for j in jobs}
        if len(T) != 1:
            raise ValueError("jobs of one chunk must have episodes of equal length")
        T = T.pop()
        uniq: dict = {}
        eu_list = [uniq.setdefault((id(j.ep), id(j.actions)), len(uniq)) for j in jobs]
        first = {}
        for j, u in zip(jobs, eu_list):
            first.setdefault(u, j)
        preps = [d.prepare(first[u].ep, pred.variant, pred.norm, actions=first[u].actions) for u in range(len(uniq))]
        tt = lambda a: torch.tensor(np.stack(a), dtype=torch.float32, device=dev)
        x_state, cal = tt([p.x_state for p in preps]), tt([p.calendar for p in preps])
        act, sev = tt([p.actions for p in preps]), tt([p.sev for p in preps])
        state_raw = tt([p.state for p in preps])
        n = len(jobs)
        ar = torch.tensor(eu_list, device=dev)
        starts = torch.tensor([j.start for j in jobs], device=dev)
        H = max(j.horizon for j in jobs)
        W = pred.warmup
        rows = starts[:, None] - (W - 1) + torch.arange(W, device=dev)[None]
        valid = rows >= 0
        r = rows.clamp(min=0)
        inp = torch.cat([x_state[ar[:, None], r], cal[ar[:, None], r], act[ar[:, None], r], sev[ar[:, None], r]], -1)
        mu, _, fl, hz, kv = model.run_h(inp, valid=valid)
        cache = m1_tf.append(None, kv, valid, keep=pred.context - 1)
        mu, fl, hz = mu[:, -1], fl[:, -1], hz[:, -1]
        ones = torch.ones(n, 1, dtype=torch.bool, device=dev)
        cur = state_raw[ar, starts]
        sev_hold = sev[ar, starts]
        vidx, didx = torch.tensor(lay.v, device=dev), torch.tensor(lay.deaths, device=dev)
        sa_i, ft_i = lay.names.index(rx.EXTRA[0]), lay.names.index(rx.EXTRA[1])
        vs, probs, chs, des, sas, fts, hzs = [], [], [], [], [], [], []
        for k in range(H):
            cur = pred.stepper.next_state(cur, mu, fl)
            probs.append(torch.sigmoid(fl))
            hzs.append(torch.sigmoid(hz))
            vs.append(cur[:, vidx])
            chs.append(cur[:, lay.cold_hours])
            des.append(cur[:, didx].sum(-1))
            sas.append(cur[:, sa_i])
            fts.append(cur[:, ft_i])
            if k == H - 1:
                break
            rr = (starts + k + 1).clamp(max=T - 1)
            x1 = torch.cat([pred.stepper.normalize(cur), cal[ar, rr], act[ar, rr], sev_hold], -1).unsqueeze(1)
            m1, _, f1, h1, new_kv = model.run_h(x1, cache, ones)
            cache = m1_tf.append(cache, new_kv, ones, keep=pred.context - 1)
            mu, fl, hz = m1[:, 0], f1[:, 0], h1[:, 0]
        st = lambda xs: torch.stack(xs, 1).double().cpu().numpy()
        v, prob, ch, de, sa, ft, hh = st(vs), st(probs), st(chs), st(des), st(sas), st(fts), st(hzs)
    out = []
    for i, j in enumerate(jobs):
        h = j.horizon
        cold = j.ep.bench.cold_hours[j.start] + np.cumsum(ch[i, :h])
        deaths = j.ep.bench.deaths[j.start] + np.cumsum(de[i, :h])
        vv = v[i, :h]
        if pred.round_counts:
            vv, cold, deaths = np.round(vv), np.round(cold), np.round(deaths)
        out.append((rx.RolloutX(vv[None], prob[i, :h][None], cold[None], deaths[None], sa[i, :h][None], ft[i, :h][None]),
                    hh[i, :h]))
    return out


def expected_day(h: np.ndarray, ref_row: int, last_row: int = v2.LAST_ROW) -> dict:
    """Expected restricted split day from the hazards h_1 .. h_K of rows ref + 1 .. ref + K (K = last_row - ref):
    E = sum_k P_k day(ref + k) + S_K day(last_row); also S_K and 1 - S_720 (split within 30 days, when K >= 720)."""
    h = np.asarray(h, dtype=np.float64)
    K = last_row - ref_row
    if len(h) != K:
        raise ValueError(f"{len(h)} hazards for {K} steps")
    surv = np.cumprod(1.0 - h)
    prev = np.concatenate([[1.0], surv[:-1]])
    p = prev * h
    days = v2.day_of_row(ref_row + np.arange(1, K + 1)).astype(np.float64)
    e = float((p * days).sum() + surv[-1] * float(v2.day_of_row(last_row)))
    return {"E": e, "S_K": float(surv[-1]), "p30": float(1.0 - surv[719]) if K >= 720 else None}


# ------------------------------------------------------------------ predictions ------------------------------------
def predictions(pred, families: list[FamilyData], cases: list, horizon: int | None = None, test_std=None,
                hazard: bool = False, chunk: int = 128) -> tuple[dict, dict]:
    """A model's predicted effects (and, for the full horizon, shifts) for every case, and the factual-rollout drift
    (`wm2.effects.model_predictions` on pre-loaded families; drift needs `test_std`)."""
    by_fam: dict[int, list] = {}
    for x in cases:
        by_fam.setdefault(x.seed, []).append(x)
    preds, drift = {}, {k: [] for k in ef.DRIFT_LAGS}
    for fam in families:
        mine = by_fam.get(fam.seed, [])
        for r, ep in sorted(fam.factual.items()):
            cs = [x for x in mine if x.rep == r]
            if not cs:
                continue
            truth = fam.truth[r]
            hz_of = lambda row: v2.LAST_ROW - row if horizon is None else min(horizon, v2.LAST_ROW - row)
            zeros = np.zeros_like(ep.bench.actions)
            jobs, keys = [], []
            for row in sorted({x.cast_row for x in cs}):
                jobs.append(rx.Job(ep, row, zeros, hz_of(row)))
                keys.append(("factual", row))
            for x in cs:
                act = zeros.copy()
                kind, amount = v2.ARM_SPEC[x.arm]
                act[x.cast_row, b.KINDS.index(kind)] = float(amount)
                jobs.append(rx.Job(ep, x.cast_row, act, hz_of(x.cast_row)))
                keys.append(("arm", x.key))
            if hazard:
                ro_list, h_list = rollout_jobs_h(pred, jobs, chunk)
                hzd = dict(zip(keys, h_list))
            else:
                ro_list, hzd = rx.rollout_jobs(pred, jobs, chunk), None
            ros = dict(zip(keys, ro_list))
            if test_std is not None:
                for row in sorted({x.cast_row for x in cs}):
                    ro = ros[("factual", row)]
                    for name, lag in ef.DRIFT_LAGS.items():
                        if row + lag <= v2.LAST_ROW and lag <= ro.v.shape[1]:
                            drift[name].append(np.abs(ro.v[0, lag - 1] - ep.bench.v[row + lag])
                                               / np.where(test_std > 0, test_std, np.nan))
            for x in cs:
                fa, ar = ros[("factual", x.cast_row)], ros[("arm", x.key)]
                eff = {}
                for o in v2.OUTCOMES:
                    eff[o] = {}
                    for lag_name in x.truth[o]:
                        i = v2.LAGS[lag_name] - 1
                        if i >= ar.v.shape[1]:
                            continue
                        get = {"food_stock": lambda ro: ro.v[0, i, ef.V_FOOD], "alive": lambda ro: ro.v[0, i, ef.V_ALIVE],
                               "deaths": lambda ro: ro.deaths[0, i],
                               "settlements_alive": lambda ro: ro.settlements_alive[0, i]}[o]
                        eff[o][lag_name] = float(get(ar) - get(fa))
                if horizon is None:
                    f_true = truth["fissions_total"][: x.cast_row + 1]
                    days = {}
                    for name, ro in (("arm", ar), ("factual", fa)):
                        days[name] = v2.restricted_split_day(np.concatenate([f_true, ro.fissions_total[0]]), x.cast_row)
                    p = {"effects": eff, "shift": float(days["arm"][0] - days["factual"][0]),
                         "censored": [days["arm"][1], days["factual"][1]]}
                    if hazard:
                        ea = expected_day(hzd[("arm", x.key)], x.cast_row)
                        efa = expected_day(hzd[("factual", x.cast_row)], x.cast_row)
                        p |= {"shift_expected": ea["E"] - efa["E"], "E_arm": ea["E"], "E_factual": efa["E"],
                              "S_K_arm": ea["S_K"], "S_K_factual": efa["S_K"]}
                else:
                    p = {"effects": eff, "shift": float("nan"), "censored": None}
                preds[x.key] = p
    drift_out = {k: {"mean": float(np.nanmean(np.vstack(v))) if v else None,
                     "per_var": np.nanmean(np.vstack(v), axis=0).tolist() if v else None,
                     "n": len(v)} for k, v in drift.items()}
    return preds, drift_out


def val_eff(pred, families: list[FamilyData], cases: list, stds: dict) -> tuple[float, dict]:
    """The fine-tuning selection score: the 24-cell mean scaled effect MAE on the given (val, replicate 0) cases from
    720-row rollouts (`round_counts` as set on the predictor)."""
    cells = val_eff_cells(stds)
    preds, _ = predictions(pred, families, cases, horizon=VAL_EFF_HORIZON)
    errs = ef.cell_errors(cases, preds, stds)
    worlds = sorted({x.seed for x in cases})
    per = {"|".join(cell): ef.cell_stat(errs, [cell], worlds) for cell in cells}
    return ef.cell_stat(errs, cells, worlds), per


# ------------------------------------------------------------------ split timing / calibration ---------------------
def split_timing(pred, eps: list[d.EpisodeData], hazard: bool) -> dict:
    """WM-2's split-timing rollouts (`wm2.split_task.cmd_model`) on the eligible worlds: the event-based day and, for a
    hazard model, the expected day from the start row."""
    from wm2 import split_task as st
    el = st.eligible(eps)
    jobs = [rx.Job(ep, st.START_ROW, ep.bench.actions, v2.LAST_ROW - st.START_ROW) for ep in el]
    if hazard:
        ros, hs = rollout_jobs_h(pred, jobs)
    else:
        ros, hs = rx.rollout_jobs(pred, jobs), [None] * len(jobs)
    out = {"event": {}, "censored": {}, "truth": {}, "expected": {}, "S_K": {}}
    for ep, ro, h in zip(el, ros, hs):
        f = st.fissions(ep)
        day, cz = v2.restricted_split_day(np.concatenate([f[: st.START_ROW + 1], ro.fissions_total[0]]), st.START_ROW)
        out["event"][ep.seed], out["censored"][ep.seed] = float(day), cz
        out["truth"][ep.seed] = st.target(ep)[0]
        if h is not None:
            e = expected_day(h, st.START_ROW)
            out["expected"][ep.seed], out["S_K"][ep.seed] = e["E"], e["S_K"]
    return out


def calibration(pred, families: list[FamilyData], cases: list, bins: int = 5) -> dict:
    """Hazard calibration (GOAL.md 10) on every distinct (world, replicate-0 cast row) of the given cases: the factual
    rollout's predicted P(split within 30 days) = 1 - S_720 vs the observed `split_event_row(none_r0, c) <= c + 720`,
    in `bins` quantile bins of the prediction."""
    rows = sorted({(x.seed, x.cast_row) for x in cases if x.rep == 0})
    fams = {f.seed: f for f in families}
    jobs, obs, keys = [], [], []
    for seed, row in rows:
        ep = fams[seed].factual[0]
        jobs.append(rx.Job(ep, row, np.zeros_like(ep.bench.actions), 720))
        ev = v2.split_event_row(fams[seed].truth[0]["fissions_total"], row)
        obs.append(float(ev is not None and ev <= row + 720))
        keys.append(f"{seed}:{row}")
    _, hs = rollout_jobs_h(pred, jobs)
    p = np.array([1.0 - float(np.prod(1.0 - np.asarray(h, dtype=np.float64))) for h in hs])
    o = np.array(obs)
    edges = np.quantile(p, np.linspace(0, 1, bins + 1))
    which = np.clip(np.searchsorted(edges, p, side="right") - 1, 0, bins - 1)
    table = [{"bin": i, "n": int((which == i).sum()), "pred_mean": float(p[which == i].mean()) if (which == i).any() else None,
              "obs_rate": float(o[which == i].mean()) if (which == i).any() else None,
              "lo": float(edges[i]), "hi": float(edges[i + 1])} for i in range(bins)]
    brier = float(((p - o) ** 2).mean())
    return {"n": len(p), "bins": table, "pred_mean": float(p.mean()), "obs_rate": float(o.mean()), "brier": brier,
            "brier_climatology": float(((o.mean() - o) ** 2).mean()), "per_row": dict(zip(keys, zip(p.tolist(), obs)))}


# ------------------------------------------------------------------ commands ---------------------------------------
def is_hazard_run(config: dict) -> bool:
    """A run whose hazard head was trained (hazard weight > 0); `wm3_ctrl` / `wm3_os` are scored as non-hazard models."""
    return float(config.get("hyper", {}).get("hazard", 0.0)) > 0


def _load(run: Path, device: str):
    from wm1 import m1_gru as g
    from wm3 import model  # noqa: F401  (registers m1_tfh)
    pred, config = g.load_predictor(run, device)
    if config.get("dataset") != "v2_v19":
        raise SystemExit(f"{run}: trained on {config.get('dataset')}, not v2_v19")
    return pred, config


def _refuse(target: Path) -> None:
    if target.exists():
        raise SystemExit(f"{target} exists: computed once per run")


def cmd_model(a) -> int:
    t0 = time.time()
    target = Path(a.out) / "effects.json"
    _refuse(target)
    run = Path(a.run)
    pred, config = _load(run, a.device)
    hz = is_hazard_run(config)
    base = json.loads(Path(a.base).read_text())
    cases = [ef.case_from(j) for j in base["cases"]["test"]]
    tests = c.load_groups(["test"], Path(a.root), "v2_v19_fact")
    test_std = b.split_std([ep.bench for ep in tests["test"]])
    fams = load_families(ef.family_dirs(Path(a.root), "test"), "test")
    preds, drift = predictions(pred, fams, cases, None, test_std, hazard=hz)
    out = {"run": str(run), "model": config["model"], "variant": config["variant"], "wm3_config": config.get("wm3_config"),
           "seed": config["seed"], "hyper": config["hyper"], "selected": config.get("selected"), "hazard": hz,
           "round_counts": pred.round_counts, "code_commit": c.git_commit(), "train_code_commit": config.get("code_commit"),
           "argv": sys.argv, "predictions": preds, "drift": drift, "test_std": test_std.tolist(),
           "base_sha": ef._sha(Path(a.base)), "seconds": time.time() - t0}
    _refuse(target)
    c.write_json(target, out)
    print(f"effects done {time.time() - t0:.0f}s hazard {hz}")
    return 0


def cmd_split(a) -> int:
    from wm2 import split_task as st
    t0 = time.time()
    target = Path(a.out) / "split.json"
    _refuse(target)
    pred, config = _load(Path(a.run), a.device)
    hz = is_hazard_run(config)
    gr = c.load_groups(["test"], Path(a.root), "v2_v19_fact")
    res = split_timing(pred, gr["test"], hz)
    out = {"run": a.run, "model": config["model"], "wm3_config": config.get("wm3_config"), "seed": config["seed"],
           "selected": config.get("selected"), "hazard": hz, "hyper": config["hyper"], "code_commit": c.git_commit(),
           "argv": sys.argv, "round_counts": pred.round_counts,
           "event": st.summarize(res["event"], res["truth"]) | {"predictions": {str(s): v for s, v in res["event"].items()},
                                                                 "censored_pred_share": float(np.mean(list(res["censored"].values())))},
           "seconds": time.time() - t0}
    if hz:
        out["expected"] = st.summarize(res["expected"], res["truth"]) | {
            "predictions": {str(s): v for s, v in res["expected"].items()},
            "S_K": {str(s): v for s, v in res["S_K"].items()}}
    _refuse(target)
    c.write_json(target, out)
    print(f"split done {time.time() - t0:.0f}s event mae {out['event']['mae']:.2f}"
          + (f" expected mae {out['expected']['mae']:.2f}" if hz else ""))
    return 0


def cmd_calib(a) -> int:
    t0 = time.time()
    pred, config = _load(Path(a.run), a.device)
    pred.round_counts = False
    base = json.loads(Path(a.base).read_text())
    cases = [ef.case_from(j) for j in base["cases"]["val"]]
    fams = load_families(ef.family_dirs(Path(a.root), "val"), "val")
    out = calibration(pred, fams, cases) | {"run": a.run, "code_commit": c.git_commit(), "argv": sys.argv,
                                            "selected": config.get("selected"), "seconds": time.time() - t0}
    c.write_json(Path(a.out) / "calib.json", out)
    print(f"calib done {time.time() - t0:.0f}s n {out['n']} pred {out['pred_mean']:.3f} obs {out['obs_rate']:.3f}")
    return 0


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    for name in ("model", "split", "calib"):
        p = sub.add_parser(name)
        p.add_argument("--run", required=True)
        p.add_argument("--root", default=str(c.DEFAULT_ROOT))
        p.add_argument("--device", default="auto")
        p.add_argument("--out", required=True)
        if name != "split":
            p.add_argument("--base", required=True)
    a = ap.parse_args(argv)
    return {"model": cmd_model, "split": cmd_split, "calib": cmd_calib}[a.cmd](a)


if __name__ == "__main__":
    raise SystemExit(main())
