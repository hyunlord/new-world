"""WM-3 training loss (docs/worldmodel/wm3/GOAL.md 7.3): `wm1.m1_tf.losses_tf`'s arithmetic with the rollout start
positions given per window, plus the pair loss and the hazard loss.

A batch's rows are `n_plain` plain windows, then `n_cast` cast windows, then (with the pair loss) their `n_cast`
partners (`wm3.sampling`). Rollout starts: the plain windows share the positions WM-2 draws (`m1_gru.roll_positions`
from the training stream, the same call as `losses_tf`), every cast and partner window starts at p (`CAST_POS`).

- WM-2's terms over every window: teacher-forced Gaussian NLL + fire BCE + `tf_l1` x feature-weighted L1 of the mean,
  and the k-step rollout loss (feature-weighted L1 of the train-std-normalized continuous state + fire BCE, averaged
  over the steps) — unchanged in form and weights.
- **pair** (weight `pair`): for each cast window and its partner, whose rollouts start from the same state (asserted),
  the mean over steps i = 1 .. k and continuous features of |(cur_A,i - cur_F,i) - (true_A,i - true_F,i)| / std x the
  feature weights (`losses_tf`'s indexing); the fire is excluded.
- **hazard** (weight `hazard`): BCE with logits of the hazard head at every teacher-forced row t of every window against
  y_t = 1 if `g.fissions_total`[t + 1] - `g.fissions_total`[t] >= 0.5 (no positive weight).
"""
from __future__ import annotations

import numpy as np

import wm3  # noqa: F401
from wm1 import m1_gru as g
from wm1 import m1_tf
from wm3.sampling import CAST_POS

torch = g.torch
if torch is not None:
    import torch.nn.functional as F

FISSIONS = "g.fissions_total"


def positions(L: int, k: int, cfg: dict, rng: np.random.Generator, n_plain: int, n_rest: int, pos: int) -> np.ndarray:
    """[B, m] rollout start positions: the plain windows share `roll_positions`' draw (drawn only when there are plain
    windows), the cast and partner windows start at `pos`."""
    m = int(cfg["roll_starts"])
    if n_rest and m != 1:
        raise ValueError("cast windows need roll_starts = 1")
    if n_rest and pos + k > L:
        raise ValueError(f"cast position {pos} + k {k} exceeds the window length {L}")
    rows = []
    if n_plain:
        js = g.roll_positions(L, k, cfg, rng)
        rows += [js] * n_plain
    rows += [np.full(m, pos)] * n_rest
    return np.stack(rows)


def hazard_target(state, fi: int):
    """y_t = 1 where the fission count rises by >= 0.5 from row t to t + 1 ([B, L] from the [B, L + 1] raw states)."""
    return (state[:, 1:, fi] - state[:, :-1, fi] >= 0.5).float()


def wm3_losses(model, stepper: g.Stepper, bt: dict, k: int, cfg: dict, rng: np.random.Generator, n_plain: int,
               n_cast: int = 0, partners: bool = False, pos: int = CAST_POS):
    """(tf, parts, roll, pair, haz): `losses_tf`'s tf and roll terms with per-window rollout starts, the pair loss
    (zero tensor without partners) and the hazard loss (zero tensor when `cfg["hazard"]` is 0)."""
    L = bt["dtarget"].shape[1]
    inp = torch.cat([bt["x_state"], bt["calendar"], bt["actions"], bt["sev"]], dim=-1)
    B = inp.shape[0]
    n_part = n_cast if partners else 0
    if B != n_plain + n_cast + n_part:
        raise ValueError(f"batch {B} != {n_plain} plain + {n_cast} cast + {n_part} partners")
    use_h = hasattr(model, "run_h")
    if use_h:
        mu, logvar, fl, hz, kv = model.run_h(inp[:, :L])
    else:
        mu, logvar, fl, kv = model.run(inp[:, :L])
        hz = None
    nll = 0.5 * (logvar + (bt["dtarget"] - mu) ** 2 / logvar.exp()).mean()
    bce = F.binary_cross_entropy_with_logits(fl, bt["fire_next"])
    tf = nll + bce
    w = stepper.feature_weights(cfg.get("v_weight", 1.0))
    if cfg.get("tf_l1", 0.0):
        tf = tf + cfg["tf_l1"] * ((bt["dtarget"] - mu).abs() * w).mean()
    J = torch.tensor(positions(L, k, cfg, rng, n_plain, n_cast + n_part, pos), device=inp.device)
    m = J.shape[1]
    bidx = torch.arange(B, device=inp.device)[:, None].expand(B, m)
    flat = lambda a: a.reshape(B * m, *a.shape[2:])
    rep = lambda t: t[:, None].expand(B, m, *t.shape[1:]).reshape(B * m, *t.shape[1:])
    # Each start sees the teacher-forced keys of rows <= its own position (as `losses_tf`).
    valid = (torch.arange(L, device=inp.device)[None, None, :] <= J[:, :, None]).reshape(B * m, L)
    cache = {"kv": [(rep(kk), rep(vv)) for kk, vv in kv], "valid": valid}
    mu_j, fl_j = flat(mu[bidx, J]), flat(fl[bidx, J])
    cur = flat(bt["state"][bidx, J])
    sev_hold = flat(bt["sev"][bidx, J])
    cont = stepper.cont
    fire_w = float(w.max().item())
    ca = torch.arange(n_plain, n_plain + n_cast, device=inp.device)
    pa = ca + n_cast
    if n_part and not torch.equal(cur[ca], cur[pa]):
        raise ValueError("a cast window and its partner do not start from the same state")
    roll = 0.0
    pair = torch.zeros((), device=inp.device)
    ones = torch.ones(B * m, 1, dtype=torch.bool, device=inp.device)
    for i in range(k):
        cur = stepper.next_state(cur, mu_j, fl_j, hard=False)
        true = flat(bt["state"][bidx, J + i + 1])
        err = (cur[:, cont] - true[:, cont]) / stepper.std[cont]
        roll = roll + (err.abs() * w).mean() + fire_w * F.binary_cross_entropy_with_logits(fl_j, true[:, stepper.lay.fire])
        if n_part:
            dd = ((cur[ca][:, cont] - cur[pa][:, cont]) - (true[ca][:, cont] - true[pa][:, cont])) / stepper.std[cont]
            pair = pair + (dd.abs() * w).mean()
        if i == k - 1:
            break
        r = J + i + 1
        nxt = torch.cat([stepper.normalize(cur), flat(bt["calendar"][bidx, r]), flat(bt["actions"][bidx, r]), sev_hold], -1)
        mm, _, ff, new_kv = model.run(nxt.unsqueeze(1), cache, ones)
        cache = m1_tf.append(cache, new_kv, ones)
        mu_j, fl_j = mm[:, 0], ff[:, 0]
    roll = roll / k
    pair = pair / k
    if cfg.get("hazard", 0.0) and hz is not None:
        fi = stepper.lay.names.index(FISSIONS)
        y = hazard_target(bt["state"][:, : L + 1], fi)
        haz = F.binary_cross_entropy_with_logits(hz, y)
        pos_rate = float(y.mean().item())
    else:
        haz, pos_rate = torch.zeros((), device=inp.device), None
    parts = {"nll": nll.item(), "bce": bce.item(), "roll": roll.item(), "pair": pair.item(), "haz": haz.item(),
             "haz_pos_rate": pos_rate}
    return tf, parts, roll, pair, haz


def kind_loss(model, stepper, bt, k, cfg, rng):
    """The `g.Kind` loss of `m1_tfh` (plain windows only: `losses_tf`); WM-3 training uses `wm3_losses`."""
    return m1_tf.losses_tf(model, stepper, bt, k, cfg, rng)
