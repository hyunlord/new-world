"""Rollout extension for the M1 predictors (GOAL.md 7.2): batched rollouts of many (episode, start, actions) jobs at
once that also return `g.settlements_alive` and `g.fissions_total` from the stepped state.

`evaluate.Rollout` carries only the 8 V variables, fire, cold hours and deaths. `RolloutX` adds two optional fields;
`rollout_jobs` computes them for the two M1 predictors (`m1_tf.TFPredictor`, `m1_gru.GRUPredictor` with either
burn-in) with exactly their rollout arithmetic (warm-up / burn-in, `Stepper.next_state`, calendar from the episode,
the job's actions, the severity held), batched over jobs of different episodes instead of starts of one episode. The
predictors themselves are not changed, so every existing output and the WM-1.2 / WM-1.3 goldens are untouched; a
test checks that `rollout_jobs` reproduces `pred.rollout` on the shared fields. Rows past an episode's last row repeat
the last row's calendar and actions (`clamp`, as the predictors do); each job's result is cut to its own horizon.

The two extra series are the continuous stepped state (never rounded): the split event (`v2.split_event_row`) is
defined on it; a model without those state columns (the `game_v16` control) gets None. Jobs that share an episode and
an actions array share one prepared input (and, for the full burn-in, one teacher-forced pass). The calendar family, `an5`, persistence and the ridge do not predict these columns.
"""
from __future__ import annotations

from dataclasses import dataclass

import numpy as np

import wm2  # noqa: F401
from wm1 import data as d
from wm1 import evaluate as ev
from wm1 import m1_gru as g

torch = g.torch
EXTRA = ("g.settlements_alive", "g.fissions_total")


@dataclass
class RolloutX(ev.Rollout):
    settlements_alive: np.ndarray | None = None   # [n, H] continuous stepped state
    fissions_total: np.ndarray | None = None      # [n, H] continuous stepped state


@dataclass
class Job:
    """One rollout: from row `start` of `ep` (rows <= start observed) for `horizon` steps with `actions` [T, 7]
    (requested amounts per row; the action at row r enters the transition r -> r + 1)."""
    ep: d.EpisodeData
    start: int
    actions: np.ndarray
    horizon: int


def _is_tf(pred) -> bool:
    """The transformer predictor (duck-typed: when `m1_gru.py` / `m1_tf.py` run as scripts their classes live in
    `__main__`, not in the `wm1` modules, so `isinstance` against the `wm1` classes would fail)."""
    return hasattr(pred.model, "blocks")


def _is_m1(pred) -> bool:
    return all(hasattr(pred, a) for a in ("model", "stepper", "lay", "norm", "variant", "warmup"))


def rollout_jobs(pred, jobs: list[Job], chunk: int = 128) -> list[RolloutX]:
    """Every job's rollout (in job order), batched `chunk` jobs at a time (jobs sorted by horizon inside)."""
    if not _is_m1(pred):
        raise TypeError("rollout_jobs supports the M1 predictors (m1_gru.GRUPredictor, m1_tf.TFPredictor)")
    order = sorted(range(len(jobs)), key=lambda i: -jobs[i].horizon)
    out: list[RolloutX | None] = [None] * len(jobs)
    for i in range(0, len(order), chunk):
        idx = order[i: i + chunk]
        for j, ro in zip(idx, _chunk(pred, [jobs[k] for k in idx])):
            out[j] = ro
    return out


@torch.no_grad() if torch is not None else (lambda f: f)
def _chunk(pred, jobs: list[Job]) -> list[RolloutX]:
    model, lay, dev = pred.model, pred.lay, pred.device
    model.eval()
    T = {j.ep.rows for j in jobs}
    if len(T) != 1:
        raise ValueError("jobs of one chunk must have episodes of equal length")
    T = T.pop()
    # One prepared input per distinct (episode, actions) pair; every job indexes its pair (`eu`).
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
    tf = _is_tf(pred)
    if tf:
        from wm1 import m1_tf
        rows = starts[:, None] - (W - 1) + torch.arange(W, device=dev)[None]
        valid = rows >= 0
        r = rows.clamp(min=0)
        inp = torch.cat([x_state[ar[:, None], r], cal[ar[:, None], r], act[ar[:, None], r], sev[ar[:, None], r]], -1)
        mu, _, fl, kv = model.run(inp, valid=valid)
        cache = m1_tf.append(None, kv, valid, keep=pred.context - 1)
        mu, fl = mu[:, -1], fl[:, -1]
        ones = torch.ones(n, 1, dtype=torch.bool, device=dev)
    elif getattr(pred, "burn_in", "window") == "full":
        mu_all, _, fl_all, outs = model.run(torch.cat([x_state, cal, act, sev], -1))   # one pass per distinct input
        hidden = outs[:, ar, starts].contiguous()
        mu, fl = mu_all[ar, starts], fl_all[ar, starts]
    else:
        hidden = None
        for tau in range(W):
            rows = starts - (W - 1) + tau
            active = rows >= 0
            r = rows.clamp(min=0)
            inp = torch.cat([x_state[ar, r], cal[ar, r], act[ar, r], sev[ar, r]], dim=-1).unsqueeze(1)
            mu, _, fl, h_new = model(inp, hidden)
            if hidden is None:
                hidden = torch.zeros_like(h_new)
            hidden = torch.where(active.view(1, n, 1), h_new, hidden)
        mu, fl = mu[:, 0], fl[:, 0]
    cur = state_raw[ar, starts]
    sev_hold = sev[ar, starts]
    vidx, didx = torch.tensor(lay.v, device=dev), torch.tensor(lay.deaths, device=dev)
    has_extra = all(x in lay.names for x in EXTRA)
    sa_i, ft_i = (lay.names.index(EXTRA[0]), lay.names.index(EXTRA[1])) if has_extra else (lay.fire, lay.fire)
    vs, probs, chs, des, sas, fts = [], [], [], [], [], []
    for k in range(H):
        cur = pred.stepper.next_state(cur, mu, fl)
        probs.append(torch.sigmoid(fl))
        vs.append(cur[:, vidx])
        chs.append(cur[:, lay.cold_hours])
        des.append(cur[:, didx].sum(-1))
        sas.append(cur[:, sa_i])
        fts.append(cur[:, ft_i])
        if k == H - 1:
            break
        rr = (starts + k + 1).clamp(max=T - 1)
        x1 = torch.cat([pred.stepper.normalize(cur), cal[ar, rr], act[ar, rr], sev_hold], -1).unsqueeze(1)
        if tf:
            m1, _, f1, new_kv = model.run(x1, cache, ones)
            cache = m1_tf.append(cache, new_kv, ones, keep=pred.context - 1)
            mu, fl = m1[:, 0], f1[:, 0]
        else:
            mu, _, fl, hidden = model(x1, hidden)
            mu, fl = mu[:, 0], fl[:, 0]
    st = lambda xs: torch.stack(xs, 1).double().cpu().numpy()
    v, prob, ch, de, sa, ft = st(vs), st(probs), st(chs), st(des), st(sas), st(fts)
    out = []
    for i, j in enumerate(jobs):
        h = j.horizon
        cold = j.ep.bench.cold_hours[j.start] + np.cumsum(ch[i, :h])
        deaths = j.ep.bench.deaths[j.start] + np.cumsum(de[i, :h])
        vv = v[i, :h]
        if pred.round_counts:
            vv, cold, deaths = np.round(vv), np.round(cold), np.round(deaths)
        out.append(RolloutX(vv[None], prob[i, :h][None], cold[None], deaths[None],
                            sa[i, :h][None] if has_extra else None, ft[i, :h][None] if has_extra else None))
    return out
