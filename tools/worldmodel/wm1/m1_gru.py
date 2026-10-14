"""M1: settlement-level recurrent world model (GOAL.md 8).

input projection -> 2-layer GRU (hidden 256) -> heads: Gaussian mean and log-variance of every continuous
feature's normalized hourly change, Bernoulli logit of the next `fire_lit`. Trained with teacher forcing plus a
rollout loss (unroll k steps on its own predictions, k growing 1 -> 24 over the first half of training);
early-stopped on the validation mean nMAE of T2 24 h (bench starts, A val and B val averaged).

usage:
  python3 tools/worldmodel/wm1/m1_gru.py train --out RUN_DIR --variant sev-blind_action-aware --seed 0 [...]
  python3 tools/worldmodel/wm1/m1_gru.py final --run RUN_DIR      # test evaluation, once per configuration

`--dataset` (GOAL_1_2.md 6.3; default `v0`): the training, validation and test groups come from `common.DATASETS`;
`config.json` stores the dataset and `final` reads it from there (a dataset without a counterfactual group skips
T3 / T4 and runs the dataset's task list with the bias-removed T2 scores and their per-world scores).

WM-2 (docs/worldmodel/wm2/GOAL.md 7.6): `--burn-in full` (M1-long) gives the GRU the teacher-forced state over every row
from row 0: in training the rows before each window run as one batched no-grad pass (`burn_in_states`, packed
sequences), then the window runs with gradient from those per-layer states (truncated BPTT); in evaluation the
teacher-forced states of every row are computed once per episode and indexed by start. `--burn-in window` (default)
is the behaviour of WM-1.3. `--val-frac F` (< 1) scores early stopping on a fixed subsample of the validation starts
(`numpy.random.default_rng(0)`, the same for every run; 1 = every start, WM-1.3).

GOAL_1_3.md 7.2-7.4: `--ramp-steps` decouples the k curriculum and the patience start from `--max-steps` (0 = half of
it); every run evaluated on WM-1.2's grid (every 150 steps) also replays WM-1.2's stopping rule online and keeps that
checkpoint as `best_wm12rule.pt` (`val_metrics.json` `wm12rule`). `final --ckpt best_wm12rule.pt --out DIR` scores it,
`final --eval-dataset NAME --out DIR` scores a run on another dataset's test split (same input list required), each
once per output directory.

Rollouts (evaluation and the predictor): warm the GRU up from a zero state on the last `warmup` observed rows
(<= the start row; 168 = the training window), then feed back its own predictions: the continuous features add the
predicted mean change, `fire_lit` is thresholded at 0.5; the calendar follows the episode, the actions are the given
ones, a severity-on model holds the severity at the start row's value.
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

import wm1  # noqa: F401
import bench_v0 as b
from wm1 import common as c
from wm1 import data as d
from wm1 import evaluate as ev

try:
    import torch
    import torch.nn as nn
    import torch.nn.functional as F
except ImportError:  # the Mac gate runs without torch; the torch tests skip
    torch = None

DEFAULTS = dict(hidden=256, layers=2, lr=3e-4, weight_decay=1e-4, batch=64, window=168, k_max=24, roll_starts=8,
                roll_weight=1.0, max_steps=12000, eval_every=400, patience=5, max_seconds=7200, warmup=168,
                grad_clip=1.0, threads=16, roll_min_context=0, tf_l1=0.0, clamp_nonneg=0, v_weight=1.0, ramp_steps=0,
                burn_in="window", val_frac=1.0)
BURN_IN = ("window", "full")
# `ramp_steps` (GOAL_1_3.md 7.2): the steps of the k curriculum and the start of patience; 0 = `max_steps // 2` (the
# WM-1 / WM-1.2 behaviour). An int, so the parser builds `--ramp-steps` with `type=int`.
# Recipe r1 = DEFAULTS. Recipe r2 (after r1 drifted at 168 h, DECISIONS D2.2): window 336, k_max 168,
# roll_min_context 167 (rollouts start after a full 168-row context, as in evaluation), roll_starts 2, tf_l1 1.0,
# clamp_nonneg 1, max_steps 4000, eval_every 200 (so a run ends on steps or patience inside the 2-hour cap on a
# shared CPU, which keeps the same-seed rerun deterministic; DECISIONS D2.3-D2.4).
R2 = dict(window=336, k_max=168, roll_min_context=167, roll_starts=2, tf_l1=1.0, clamp_nonneg=1, max_steps=4000,
          eval_every=200)
# Recipe r3 (after r2 lost T1 and T2 24 h, DECISIONS D2.5): r2 + v_weight 5 — the eight V variables weigh 5x the
# other state features in the L1 terms (teacher-forced and rollout) and the rollout fire BCE; weights rescaled to mean 1.
R3 = dict(R2, v_weight=5.0)
# WM-1.2's stopping rule (GOAL_1_2.md 9.1), tracked online in every run evaluated on its grid (GOAL_1_3.md 7.2):
# cap 3,000 steps, patience 5 evaluations counted from step 1,500, evaluations every 150 steps.
WM12_RULE = dict(max_steps=3000, patience=5, patience_from=1500, eval_every=150)
WM12_CKPT = "best_wm12rule.pt"


def pick_device(prefer: str = "auto") -> str:
    if prefer == "cpu" or torch is None:
        return "cpu"
    try:
        if torch.cuda.is_available():
            torch.zeros(1, device="cuda")
            return "cuda"
    except Exception:  # noqa: BLE001  (CUDA context creation fails on the DGX: DECISIONS D0.3)
        pass
    return "cpu"


if torch is not None:
    class M1GRU(nn.Module):
        """Two stacked single-layer GRUs (= a 2-layer GRU) so every layer's state at every position is available."""

        def __init__(self, n_in: int, n_cont: int, hidden: int = 256, layers: int = 2):
            super().__init__()
            self.proj = nn.Sequential(nn.Linear(n_in, hidden), nn.GELU())
            self.grus = nn.ModuleList([nn.GRU(hidden, hidden, 1, batch_first=True) for _ in range(layers)])
            self.mu = nn.Linear(hidden, n_cont)
            self.logvar = nn.Linear(hidden, n_cont)
            self.fire = nn.Linear(hidden, 1)

        def run(self, x, h=None):
            """x [B, L, n_in], h [layers, B, H] or None -> heads [B, L, ...], per-layer outputs [layers, B, L, H]."""
            z = self.proj(x)
            outs = []
            for i, g in enumerate(self.grus):
                z, _ = g(z, None if h is None else h[i: i + 1].contiguous())
                outs.append(z)
            return self.mu(z), self.logvar(z).clamp(-12.0, 8.0), self.fire(z).squeeze(-1), torch.stack(outs)

        def forward(self, x, h=None):
            mu, logvar, fl, outs = self.run(x, h)
            return mu, logvar, fl, outs[:, :, -1]


class Stepper:
    """Turns predicted normalized changes into the next state and the next model input (torch)."""

    def __init__(self, lay: d.Layout, norm: d.Normalizer, device: str, nonneg: bool = False):
        t = lambda a: torch.tensor(np.asarray(a), dtype=torch.float32, device=device)
        self.lay = lay
        self.cont = torch.tensor(lay.cont, device=device)
        self.mean, self.std, self.dstd = t(norm.mean), t(norm.std), t(norm.dstd)
        span = norm.hi - norm.lo
        lo = norm.lo - 0.25 * span
        if nonneg:   # features never negative in train (stocks, counts, flags) are clamped at 0 (recipe r2)
            lo = np.where(norm.lo >= 0, np.maximum(lo, 0.0), lo)
        self.lo, self.hi = t(lo), t(norm.hi + 0.25 * span)

    def feature_weights(self, v_weight: float):
        """Per continuous feature: v_weight on the V variables, 1 elsewhere, rescaled to mean 1."""
        w = np.ones(len(self.lay.cont))
        if v_weight != 1.0:
            for i in self.lay.v:
                if i != self.lay.fire:
                    w[list(self.lay.cont).index(i)] = v_weight
            w = w / w.mean()
        return torch.tensor(w, dtype=torch.float32, device=self.mean.device)

    def next_state(self, state, mu, fire_logit, hard: bool = True):
        """state [..., Ds] raw -> next raw state; fire thresholded (straight-through when not `hard`)."""
        nxt = state.clone()
        nxt[..., self.cont] = state[..., self.cont] + mu * self.dstd
        p = torch.sigmoid(fire_logit)
        h = (p >= 0.5).float()
        nxt[..., self.lay.fire] = h if hard else h + p - p.detach()
        return torch.maximum(torch.minimum(nxt, self.hi), self.lo)

    def normalize(self, state):
        return (state - self.mean) / self.std


class GRUPredictor(ev.Predictor):
    """A trained M1 as a harness predictor (`burn_in` "full": the teacher-forced state over rows 0 .. start)."""
    provides_t4 = True

    def __init__(self, model, lay: d.Layout, norm: d.Normalizer, variant: d.Variant, device: str, warmup: int = 168,
                 name: str = "m1_gru", round_counts: bool = False, chunk: int = 4096, nonneg: bool = False,
                 burn_in: str = "window"):
        if burn_in not in BURN_IN:
            raise ValueError(f"burn_in {burn_in}")
        self.model, self.lay, self.norm, self.variant = model, lay, norm, variant
        self.device, self.warmup, self.name, self.round_counts, self.chunk = device, warmup, name, round_counts, chunk
        self.burn_in = burn_in
        self.stepper = Stepper(lay, norm, device, nonneg)

    def rollout(self, ep, starts, horizon, actions):
        outs = [self._rollout(ep, starts[i:i + self.chunk], horizon, actions) for i in range(0, len(starts), self.chunk)]
        cat = lambda k: np.concatenate([getattr(o, k) for o in outs])
        return ev.Rollout(cat("v"), cat("fire_prob"), cat("cold_hours"), cat("deaths"))

    @torch.no_grad() if torch is not None else (lambda f: f)
    def _rollout(self, ep, starts, horizon, actions):
        self.model.eval()
        dev, lay = self.device, self.lay
        prep = d.prepare(ep, self.variant, self.norm, actions=actions)
        T = ep.rows
        tt = lambda a: torch.tensor(np.asarray(a), dtype=torch.float32, device=dev)
        x_state, cal, act, sev = tt(prep.x_state), tt(prep.calendar), tt(prep.actions), tt(prep.sev)
        state_raw = tt(prep.state)
        starts_t = torch.tensor(starts, device=dev)
        n = len(starts)
        hidden = None
        W = self.warmup
        if self.burn_in == "full":
            # The teacher-forced states of every row, once per episode (one pass), indexed by start.
            mu_all, _, fl_all, outs = self.model.run(torch.cat([x_state, cal, act, sev], dim=-1).unsqueeze(0))
            hidden = outs[:, 0, starts_t].contiguous()
            mu, fl = mu_all[0, starts_t], fl_all[0, starts_t]
        else:
            # Warm-up: rows start-W+1 .. start, from a zero state; windows cut at row 0 start later (masked).
            for tau in range(W):
                rows = starts_t - (W - 1) + tau
                active = rows >= 0
                r = rows.clamp(min=0)
                inp = torch.cat([x_state[r], cal[r], act[r], sev[r]], dim=-1).unsqueeze(1)
                mu, logvar, fl, h_new = self.model(inp, hidden)
                if hidden is None:
                    hidden = torch.zeros_like(h_new)
                hidden = torch.where(active.view(1, n, 1), h_new, hidden)
            mu, fl = mu[:, 0], fl[:, 0]
        cur = state_raw[starts_t]
        sev_hold = sev[starts_t]
        vs, probs, chs, des = [], [], [], []
        for k in range(horizon):
            cur = self.stepper.next_state(cur, mu, fl)
            probs.append(torch.sigmoid(fl))
            vs.append(cur[:, torch.tensor(lay.v, device=dev)])
            chs.append(cur[:, lay.cold_hours])
            des.append(cur[:, torch.tensor(lay.deaths, device=dev)].sum(-1))
            if k == horizon - 1:
                break
            r = (starts_t + k + 1).clamp(max=T - 1)
            inp = torch.cat([self.stepper.normalize(cur), cal[r], act[r], sev_hold], dim=-1).unsqueeze(1)
            mu, logvar, fl, hidden = self.model(inp, hidden)
            mu, fl = mu[:, 0], fl[:, 0]
        v = torch.stack(vs, 1).double().cpu().numpy()
        cold = ep.bench.cold_hours[starts][:, None] + np.cumsum(torch.stack(chs, 1).double().cpu().numpy(), axis=1)
        deaths = ep.bench.deaths[starts][:, None] + np.cumsum(torch.stack(des, 1).double().cpu().numpy(), axis=1)
        if self.round_counts:
            v, cold, deaths = np.round(v), np.round(cold), np.round(deaths)
        return ev.Rollout(v, torch.stack(probs, 1).double().cpu().numpy(), cold, deaths)


# ------------------------------------------------------------------ training ---------------------------------------
def set_seed(seed: int, threads: int) -> None:
    np.random.seed(seed)
    torch.manual_seed(seed)
    torch.set_num_threads(threads)
    torch.use_deterministic_algorithms(True)
    if torch.cuda.is_available():
        torch.backends.cudnn.deterministic = True
        torch.backends.cudnn.benchmark = False


def batch_tensors(win: dict, device: str) -> dict:
    return {k: torch.tensor(v, device=device) for k, v in win.items()}


def burn_in_states(model, preps: list, which: np.ndarray, starts: np.ndarray, device: str):
    """Per-layer teacher-forced GRU states [layers, B, H] after rows 0 .. start - 1 of each sampled window's episode
    (zeros where start = 0): one batched no-grad pass through `proj` and every layer's `nn.GRU`, the different
    lengths packed (`pack_padded_sequence`; zero padding is not neutral for a GRU, so it is never fed)."""
    from torch.nn.utils.rnn import pack_padded_sequence
    nl, hid = len(model.grus), model.grus[0].hidden_size
    h = torch.zeros(nl, len(starts), hid, device=device)
    nz = np.nonzero(np.asarray(starts) > 0)[0]
    if len(nz) == 0:
        return h
    lengths = np.asarray(starts)[nz]
    n_in = model.proj[0].in_features
    x = np.zeros((len(nz), int(lengths.max()), n_in), np.float32)
    for row, i in enumerate(nz):
        p, ln = preps[which[i]], int(starts[i])
        x[row, :ln] = d.model_input(p.x_state[:ln], p.calendar[:ln], p.actions[:ln], p.sev[:ln])
    with torch.no_grad():
        z = model.proj(torch.tensor(x, device=device))
        packed = pack_padded_sequence(z, torch.tensor(lengths), batch_first=True, enforce_sorted=False)
        for i, g in enumerate(model.grus):
            packed, hn = g(packed)
            h[i, torch.tensor(nz, device=device)] = hn[0]
    return h


def make_batch(preps: list, cfg: dict, rng: np.random.Generator, device: str, model=None, batch: int | None = None,
               window: int | None = None) -> dict:
    """One training batch: `d.sample_windows` tensors (`burn_in` "window": exactly WM-1.3's batch), plus for
    `burn_in` "full" the per-layer burn-in state `h0` of every window (the same draws)."""
    batch, window = batch or cfg["batch"], window or cfg["window"]
    if cfg.get("burn_in", "window") == "window":
        return batch_tensors(d.sample_windows(preps, window, batch, rng), device)
    win, which, starts = d.sample_windows(preps, window, batch, rng, return_starts=True)
    bt = batch_tensors(win, device)
    bt["h0"] = burn_in_states(model, preps, which, starts, device)
    return bt


def roll_positions(L: int, k: int, cfg: dict, rng: np.random.Generator) -> np.ndarray:
    """Rollout start positions j (the last teacher-forced row) with j + k <= L and j >= roll_min_context when it fits."""
    lo = min(cfg.get("roll_min_context", 0), L - k)
    return np.sort(rng.integers(lo, L - k + 1, size=cfg["roll_starts"]))


def losses(model, stepper: Stepper, bt: dict, k: int, cfg: dict, rng: np.random.Generator):
    """Teacher-forced Gaussian NLL + BCE (+ `tf_l1` x L1 of the mean), and the k-step rollout loss (L1 of the
    train-std-normalized continuous state + fire BCE, averaged over the k steps) from `roll_starts` positions."""
    L = bt["dtarget"].shape[1]
    inp = torch.cat([bt["x_state"], bt["calendar"], bt["actions"], bt["sev"]], dim=-1)
    mu, logvar, fl, outs = model.run(inp[:, :L], bt.get("h0"))
    y = bt["dtarget"]
    nll = 0.5 * (logvar + (y - mu) ** 2 / logvar.exp()).mean()
    bce = F.binary_cross_entropy_with_logits(fl, bt["fire_next"])
    tf = nll + bce
    vw = cfg.get("v_weight", 1.0)
    w = stepper.feature_weights(vw)
    if cfg.get("tf_l1", 0.0):
        tf = tf + cfg["tf_l1"] * ((y - mu).abs() * w).mean()
    B = inp.shape[0]
    js = torch.tensor(roll_positions(L, k, cfg, rng), device=inp.device)
    m = len(js)
    flat = lambda a: a.reshape(B * m, *a.shape[2:])
    # The teacher-forced state after consuming row j's true input, for every layer: [layers, B*m, H].
    h = outs[:, :, js].reshape(outs.shape[0], B * m, outs.shape[-1])
    mu_j, fl_j = flat(mu[:, js]), flat(fl[:, js])
    cur = flat(bt["state"][:, js])
    sev_hold = flat(bt["sev"][:, js])
    cont = stepper.cont
    fire_w = float(w.max().item())   # the fire is a V variable: same (normalized) weight as the other V features
    roll = 0.0
    for i in range(k):
        cur = stepper.next_state(cur, mu_j, fl_j, hard=False)
        true = flat(bt["state"][:, js + i + 1])
        err = (cur[:, cont] - true[:, cont]) / stepper.std[cont]
        roll = roll + (err.abs() * w).mean() + fire_w * F.binary_cross_entropy_with_logits(fl_j, true[:, stepper.lay.fire])
        if i == k - 1:
            break
        r = js + i + 1
        nxt = torch.cat([stepper.normalize(cur), flat(bt["calendar"][:, r]), flat(bt["actions"][:, r]), sev_hold], -1)
        mm, _, ff, h = model(nxt.unsqueeze(1), h)
        mu_j, fl_j = mm[:, 0], ff[:, 0]
    roll = roll / k
    return tf, {"nll": nll.item(), "bce": bce.item(), "roll": roll.item()}, roll


def val_starts(val: dict[str, list], frac: float, horizon: int = 24) -> dict | None:
    """A fixed subsample of the validation T2 starts (06:00 bench starts) per group and episode, or None (every
    start) when `frac` >= 1: per episode in group order, ceil(frac x n) of its n starts chosen by one
    `numpy.random.default_rng(0)` stream (the same for every run on the same validation set)."""
    if frac >= 1.0:
        return None
    rng = np.random.default_rng(0)
    out = {}
    for n, eps in val.items():
        for ep in eps:
            s = ev.bench_starts(ep, horizon)
            keep = np.sort(rng.permutation(len(s))[: int(np.ceil(frac * len(s)))])
            key = (n, int(ep.seed), ep.eid)
            if key in out:
                raise ValueError(f"validation episode {key} is not unique: no start subsample")
            out[key] = s[keep]
    return out


def val_score(pred: ev.Predictor, val: dict[str, list], stds: dict, keep: dict | None = None) -> tuple[float, dict]:
    """Validation T2 24 h (06:00 starts) per group and their mean; `keep` (`val_starts`) restricts the starts."""
    if keep is None:
        per = {n: ev.t2(pred, eps, stds[n], 24)["mean"] for n, eps in val.items()}
        return float(np.mean(list(per.values()))), per
    per = {}
    for n, eps in val.items():
        preds, trues = [], []
        for ep in eps:
            s = keep[(n, int(ep.seed), ep.eid)]
            preds.append(ev.run_rollouts(pred, ep, s, 24).v[:, -1])
            trues.append(ep.bench.v[s + 24])
        per[n] = float(np.nanmean(b.nmae(np.vstack(preds), np.vstack(trues), stds[n])))
    return float(np.mean(list(per.values()))), per


def calibration(model, stepper, preps: list, device: str, window: int, cfg: dict | None = None) -> dict:
    """Share of true normalized changes within +-2 sigma of the predicted mean (teacher forcing, val windows)."""
    rng = np.random.default_rng(123)
    win = make_batch(preps, cfg or {}, rng, device, model, batch=64, window=window)
    with torch.no_grad():
        L = win["dtarget"].shape[1]
        inp = torch.cat([win["x_state"], win["calendar"], win["actions"], win["sev"]], -1)[:, :L]
        mu, logvar, fl, _ = model(inp, win.get("h0"))
        inside = ((win["dtarget"] - mu).abs() <= 2 * (0.5 * logvar).exp()).float()
    per = inside.mean(dim=(0, 1)).cpu().numpy()
    return {"within_2sigma_mean": float(per.mean()), "within_2sigma": dict(zip(stepper.lay.variant.continuous_names, per.tolist()))}


class Kind:
    """What differs between M1 (GRU) and M1b (transformer): the network, its loss and its predictor."""

    def __init__(self, name, defaults, build, loss, predictor):
        self.name, self.defaults, self.build, self.loss, self.predictor = name, defaults, build, loss, predictor


def _gru_build(lay, cfg):
    return M1GRU(lay.n_input, lay.n_cont, cfg["hidden"], cfg["layers"])


def _gru_predictor(model, lay, norm, variant, device, cfg, round_counts=False):
    return GRUPredictor(model, lay, norm, variant, device, cfg["warmup"], round_counts=round_counts,
                        nonneg=bool(cfg.get("clamp_nonneg", 0)), burn_in=cfg.get("burn_in", "window"))


KINDS = {"m1_gru": Kind("m1_gru", DEFAULTS, _gru_build, lambda *a: losses(*a), _gru_predictor)}


def kind_of(name: str) -> Kind:
    if name not in KINDS:
        __import__("wm1." + name)  # importing wm1.m1_tf registers its kind in the module `wm1.m1_gru`
        # When this file runs as a script it is `__main__`, a different module object from `wm1.m1_gru`.
        KINDS.update(sys.modules["wm1.m1_gru"].KINDS)
    return KINDS[name]


def dataset_variant(ds: c.Dataset, name: str) -> d.Variant:
    """The variant `name` on dataset `ds` (observable datasets: the schema-4 table plus the dataset's `extra_state`
    columns, severity-on refused)."""
    return d.variant_for(name, d.ALL_STATE_OBS + tuple(ds.extra_state) if ds.observable else None)


def ramp_half(cfg: dict) -> int:
    """The step at which k reaches k_max and patience starts counting: `ramp_steps`, or `max_steps // 2` when 0."""
    return int(cfg.get("ramp_steps", 0) or cfg["max_steps"] // 2)


def k_at(step: int, cfg: dict) -> int:
    """The rollout horizon of a training step (k 1 -> k_max over the ramp)."""
    return 1 + int((cfg["k_max"] - 1) * min(1.0, step / max(1, ramp_half(cfg))))


class Wm12Rule:
    """WM-1.2's early stopping replayed on a run's own evaluations (`WM12_RULE`); `active` only on its grid."""

    def __init__(self, cfg: dict):
        self.active = cfg["eval_every"] == WM12_RULE["eval_every"]
        self.best, self.best_step, self.bad, self.stop, self.stop_step = float("inf"), -1, 0, None, None

    def update(self, step: int, score: float) -> bool:
        """Feed one evaluation; True when it is a new best under the rule (save the checkpoint)."""
        if not self.active or self.stop is not None:
            return False
        improved = score < self.best - 1e-6
        if improved:
            self.best, self.best_step, self.bad = score, step, 0
        else:
            self.bad += 1
        if self.bad >= WM12_RULE["patience"] and step >= WM12_RULE["patience_from"]:
            self.stop, self.stop_step = "patience", step
        elif step >= WM12_RULE["max_steps"]:
            self.stop, self.stop_step = "max_steps", step
        return improved

    def summary(self, last_step: int, run_stop: str) -> dict | None:
        if not self.active:
            return None
        stop = self.stop or ("run_ended_" + run_stop)   # the run itself ended before the rule did
        return {"rule": WM12_RULE, "best_step": self.best_step, "best_val_t2_24": self.best, "stop": stop,
                "stop_step": self.stop_step if self.stop else last_step, "checkpoint": WM12_CKPT}


def choose_rounding(pred, val: dict, stds: dict, keep: dict | None = None) -> tuple[bool, dict]:
    """Output rounding of count variables chosen on val (T2 24 h + 168 h, 06:00 starts)."""
    choice = {}
    for rc in (False, True):
        pred.round_counts = rc
        s24, _ = val_score(pred, val, stds, keep)
        s168 = float(np.mean([ev.t2(pred, eps, stds[n], 168)["mean"] for n, eps in val.items()]))
        choice[str(rc)] = {"t2_24": s24, "t2_168": s168}
    rc = choice["True"]["t2_24"] + choice["True"]["t2_168"] < choice["False"]["t2_24"] + choice["False"]["t2_168"]
    pred.round_counts = rc
    return rc, choice


def train(args, kind: Kind | None = None) -> int:
    kind = kind or KINDS["m1_gru"]
    cfg = dict(kind.defaults)
    cfg.update({k: v for k, v in vars(args).items() if k in kind.defaults and v is not None})
    if cfg.get("burn_in", "window") not in BURN_IN:
        raise SystemExit(f"--burn-in must be one of {BURN_IN}")
    ds = c.dataset(getattr(args, "dataset", "v0"))
    variant = dataset_variant(ds, args.variant)
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    device = pick_device(args.device)
    set_seed(args.seed, cfg["threads"])
    t0 = time.time()
    train_groups = ds.train_groups(variant.action_aware)
    report = [n for n in ds.val_report if n not in ds.val]
    g = c.load_groups(train_groups + list(ds.val) + report, Path(args.root), ds.name)
    train_eps = [ep for n in train_groups for ep in g[n]]
    want = variant.state_names if ds.observable else d.ALL_STATE
    if any(ep.names != want for ep in train_eps + [ep for n in ds.val for ep in g[n]]):
        raise SystemExit(f"{ds.name}: an episode's feature table differs from the dataset's column list")
    val = {n: g[n] for n in ds.val}
    stds = c.split_stds(val)
    keep = val_starts(val, float(cfg.get("val_frac", 1.0)))
    norm = d.Normalizer.fit(train_eps, variant)
    lay = d.Layout(variant)
    preps = [d.prepare(ep, variant, norm) for ep in train_eps]
    model = kind.build(lay, cfg).to(device)
    opt = torch.optim.AdamW(model.parameters(), lr=cfg["lr"], weight_decay=cfg["weight_decay"])
    stepper = Stepper(lay, norm, device, bool(cfg.get("clamp_nonneg", 0)))
    pred = kind.predictor(model, lay, norm, variant, device, cfg)
    config = {"model": kind.name, "variant": variant.name, "seed": args.seed, "hyper": cfg, "device": device,
              "torch": torch.__version__, "numpy": np.__version__, "python": sys.version.split()[0],
              "code_commit": c.git_commit(), "manifest_sha256": c.manifest_sha(ds.name), "train_groups": train_groups,
              "dataset": ds.name, "val_groups": list(ds.val), "train_episodes": len(train_eps),
              "input_names": list(variant.input_names()),
              "normalizer": norm.to_json(), "data_root": str(args.root), "threads": torch.get_num_threads(),
              "val_episodes": sum(len(v) for v in val.values()),
              "val_starts": None if keep is None else int(sum(len(x) for x in keep.values()))}
    c.write_json(out / "config.json", config)
    rng = np.random.default_rng(args.seed)
    log = (out / "train_log.jsonl").open("w")
    best, best_step, bad, step = float("inf"), -1, 0, 0
    half = ramp_half(cfg)
    wm12 = Wm12Rule(cfg)
    stop = "max_steps"
    for step in range(1, cfg["max_steps"] + 1):
        model.train()
        k = k_at(step, cfg)
        bt = make_batch(preps, cfg, rng, device, model)
        tf, parts, roll = kind.loss(model, stepper, bt, k, cfg, rng)
        loss = tf + cfg["roll_weight"] * roll
        opt.zero_grad()
        loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), cfg["grad_clip"])
        opt.step()
        if step % 50 == 0:
            log.write(json.dumps({"step": step, "k": k, "loss": loss.item(), **parts, "t": time.time() - t0}) + "\n")
            log.flush()
        if step % cfg["eval_every"] == 0:
            te = time.time()
            score, per = val_score(pred, val, stds, keep)
            eval_s = time.time() - te
            improved = score < best - 1e-6
            if improved:
                best, best_step, bad = score, step, 0
                torch.save(model.state_dict(), out / "best.pt")
            else:
                bad += 1
            if wm12.update(step, score):
                torch.save(model.state_dict(), out / WM12_CKPT)
            log.write(json.dumps({"step": step, "eval": True, "val_t2_24": score, "per_split": per, "best": best,
                                  "k": k, "t": time.time() - t0, "eval_s": eval_s}) + "\n")
            log.flush()
            print(f"step {step} k {k} loss {loss.item():.4f} val {score:.4f} best {best:.4f}@{best_step} "
                  f"{time.time() - t0:.0f}s", flush=True)
            # Patience counts only after the rollout horizon has reached k_max (the curriculum's end).
            if bad >= cfg["patience"] and step >= half:
                stop = "patience"
                break
        if time.time() - t0 > cfg["max_seconds"]:
            stop = "time"
            break
    log.close()
    wm12_summary = wm12.summary(step, stop)
    if wm12_summary is not None and (out / WM12_CKPT).exists():
        # The checkpoint WM-1.2's rule would have kept, with its own rounding choice on val.
        model.load_state_dict(torch.load(out / WM12_CKPT, map_location=device))
        wm12_summary["round_counts"], wm12_summary["rounding_val"] = choose_rounding(pred, val, stds, keep)
    model.load_state_dict(torch.load(out / "best.pt", map_location=device))
    # Choose the output rounding of count variables on val (T2 24 h and 168 h).
    round_counts, choice = choose_rounding(pred, val, stds, keep)
    if ds.tasks is None:
        val_metrics = ev.evaluate(pred, val, stds, cf_test=None, t3_split="none", any_hour=False, t1_on=True)
    elif ds.val_report:
        rep = {n: g[n] for n in ds.val_report}
        val_metrics = ev.evaluate(pred, rep, c.split_stds(rep), cf_test=None, t3_split=None, tasks=ds.tasks,
                                  bias_removed=True)
    else:
        val_metrics = ev.evaluate(pred, val, stds, cf_test=None, t3_split=None, tasks=ds.tasks, bias_removed=True)
    cal_preps = [d.prepare(ep, variant, norm) for n in ds.val for ep in val[n]]
    summary = {"best_val_t2_24": best, "best_step": best_step, "last_step": step, "stop": stop,
               "seconds": time.time() - t0, "round_counts": round_counts, "rounding_val": choice,
               "calibration_val": calibration(model, stepper, cal_preps, device, cfg["window"], cfg), "val": val_metrics,
               "ramp_steps": half, "wm12rule": wm12_summary}
    c.write_json(out / "val_metrics.json", summary)
    config.update({"round_counts": round_counts, "best_step": best_step, "stop": stop})
    c.write_json(out / "config.json", config)
    print(f"done: best {best:.4f} at {best_step}, stop {stop}, round {round_counts}, {time.time() - t0:.0f}s")
    return 0


def checkpoint_rounding(run: Path, config: dict, ckpt: str) -> bool:
    """The val-chosen `round_counts` of a run's checkpoint: `best.pt` from config.json, the WM-1.2-rule checkpoint
    from `val_metrics.json`'s `wm12rule`; any other name is refused."""
    if ckpt == "best.pt":
        return bool(config.get("round_counts", False))
    if ckpt == WM12_CKPT:
        wm12 = json.loads((run / "val_metrics.json").read_text()).get("wm12rule") or {}
        if "round_counts" not in wm12:
            raise SystemExit(f"{run}: no WM-1.2-rule rounding in val_metrics.json")
        return bool(wm12["round_counts"])
    raise SystemExit(f"unknown checkpoint {ckpt}")


def load_predictor(run: Path, device: str = "auto", ckpt: str = "best.pt") -> tuple[ev.Predictor, dict]:
    config = json.loads((run / "config.json").read_text())
    kind = kind_of(config.get("model", "m1_gru"))
    variant = dataset_variant(c.dataset(config.get("dataset", "v0")), config["variant"])
    norm = d.Normalizer.from_json(config["normalizer"])
    lay = d.Layout(variant)
    dev = pick_device(device)
    torch.set_num_threads(config["hyper"].get("threads", 16))
    model = kind.build(lay, config["hyper"]).to(dev)
    model.load_state_dict(torch.load(run / ckpt, map_location=dev))
    return kind.predictor(model, lay, norm, variant, dev, config["hyper"],
                          round_counts=checkpoint_rounding(run, config, ckpt)), config


def test_plan(config: dict, eval_dataset: str | None = None) -> tuple[c.Dataset, list[str], dict]:
    """What `final` loads and runs for a run's config.json: the dataset (WM-1 runs have no `dataset` key: v0; or
    `eval_dataset`, GOAL_1_3.md 7.4), the groups to load, and the `evaluate` keywords (v0: every task, T3 on A_test,
    T4 on C_test; a dataset without a counterfactual group: its task list, no T3 / T4, bias-removed T2 and the
    per-world scores)."""
    ds = c.dataset(eval_dataset or config.get("dataset", "v0"))
    if ds.cf is None:
        return ds, list(ds.test), {"t3_split": None, "tasks": ds.tasks, "bias_removed": True, "per_world": True}
    return ds, list(ds.test) + [ds.cf], {"t3_split": ds.t3_test}


def check_eval_dataset(config: dict, eval_dataset: str) -> None:
    """A run is evaluated on another dataset only when its input list equals that dataset's (GOAL_1_3.md 7.4)."""
    want = list(dataset_variant(c.dataset(eval_dataset), config["variant"]).input_names())
    if config.get("input_names") != want:
        raise SystemExit(f"the run's input list ({len(config.get('input_names') or [])} names) differs from "
                         f"{eval_dataset}'s ({len(want)}): refused")


def final(args) -> int:
    """Test evaluation of a trained run (GOAL.md 4.2: once per configuration)."""
    run = Path(args.run)
    out_dir = Path(args.out) if getattr(args, "out", None) else run
    target = out_dir / "metrics.json"
    if target.exists() and not args.force:
        raise SystemExit(f"{target} exists: test metrics are computed once per configuration")
    ckpt = getattr(args, "ckpt", None) or "best.pt"
    config = json.loads((run / "config.json").read_text())
    eval_ds = getattr(args, "eval_dataset", None)
    if eval_ds:
        check_eval_dataset(config, eval_ds)
    pred, config = load_predictor(run, args.device, ckpt)
    t0 = time.time()
    ds, groups, kwargs = test_plan(config, eval_ds)
    g = c.load_groups(groups, Path(args.root), ds.name)
    tests = {n: g[n] for n in ds.test}
    out = ev.evaluate(pred, tests, c.split_stds(tests), cf_test=g[ds.cf] if ds.cf else None, **kwargs)
    out["dataset"] = ds.name
    out["run_dataset"], out["eval_dataset"] = config.get("dataset", "v0"), ds.name
    out["run"], out["checkpoint"], out["round_counts"] = str(run), ckpt, pred.round_counts
    out["config"] = {k: config[k] for k in ("variant", "seed", "device", "code_commit", "round_counts")}
    out["eval_code_commit"] = c.git_commit()
    out["argv"], out["forced"] = sys.argv, bool(args.force)   # provenance of the single test evaluation
    out["seconds"] = time.time() - t0
    c.write_json(target, out)
    print(f"final done {time.time() - t0:.0f}s")
    return 0


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    tr = sub.add_parser("train")
    tr.add_argument("--out", required=True)
    tr.add_argument("--variant", default=d.MAIN.name, choices=sorted(d.VARIANTS))
    tr.add_argument("--seed", type=int, default=0)
    tr.add_argument("--root", default=str(c.DEFAULT_ROOT))
    tr.add_argument("--device", default="auto")
    tr.add_argument("--dataset", default="v0", choices=sorted(c.DATASETS))
    for k, v in DEFAULTS.items():
        tr.add_argument(f"--{k.replace('_', '-')}", dest=k, type=type(v), default=None)
    fi = sub.add_parser("final")
    fi.add_argument("--run", required=True)
    fi.add_argument("--root", default=str(c.DEFAULT_ROOT))
    fi.add_argument("--device", default="auto")
    fi.add_argument("--force", action="store_true")
    fi.add_argument("--ckpt", default=None, help=f"best.pt (default) or {WM12_CKPT}")
    fi.add_argument("--eval-dataset", dest="eval_dataset", default=None, choices=sorted(c.DATASETS),
                    help="evaluate on this dataset's test split (input lists must match)")
    fi.add_argument("--out", default=None, help="output directory (default: the run directory)")
    args = ap.parse_args(argv)
    if torch is None:
        raise SystemExit("torch is required")
    return train(args) if args.cmd == "train" else final(args)


if __name__ == "__main__":
    raise SystemExit(main())
