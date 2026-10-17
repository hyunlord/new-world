"""CRN (Bica et al., ICLR 2020) and the Causal Transformer (Melnychuk et al., ICML 2022) at daily resolution
(docs/worldmodel/wm4/GOAL.md 7.4; deviations from the papers in docs/worldmodel/wm4/METHODS.md).

Building blocks are the authors' plain-PyTorch modules (`third_party/causal_transformer`: `VariationalLSTM`,
`grad_reverse`, `BRTreatmentOutcomeHead`, `TransformerMultiInputBlock`, `RelativePositionalEncoding`); the training
loops, data and rollouts are written here (no Lightning, Hydra or MLflow). Tokens: `wm4.tokens` (outcomes y = the 37
normalized daily state features, covariates v = the calendar at the token, treatments a = {none, gift, spoil, rot}
one-hot + amount). At token t the representation BR_t sees a_{<t}, y_{<=t}, v_{<=t}; the outcome head reads
[BR_t, a_t] and predicts the normalized change y_{t+1} − y_t; the treatment head predicts a_t's class from BR_t.

- **CRN**: encoder = `VariationalLSTM` over [a_{t−1}, v_t, y_t] → `BRTreatmentOutcomeHead` with gradient reversal,
  α = 2 / (1 + e^(−10 p)) − 1 (p = epoch / epochs); decoder = a second `VariationalLSTM` over [a_{t−1}, y_t] whose
  state is initialized from the encoder's BR at the last observed token, trained after the encoder (frozen) with
  teacher forcing on windows of τ = 29 steps; a rollout uses the encoder's head for the first step and the decoder for
  the next 29.
- **CT**: three input streams (treatments a_{t−1}, outcomes y_t, covariates v_t) with separate linear projections,
  `TransformerMultiInputBlock`s with trainable relative positional encodings (max distance 15), BR = ELU(linear(mean of
  the streams)); counterfactual domain confusion: the representation (with every non-treatment-head parameter) is
  updated on MSE + α · CE(treatment head, uniform), the treatment head on α · CE(treatment head on detached BR, a_t),
  α = 0.01 × the same exponential rise, two Adam optimizers, EMA of both parameter groups (β 0.99) used for evaluation;
  training batches are augmented with a copy whose covariates are masked after a random split, as at prediction time.
  Rollouts are autoregressive (the predicted y fed back, covariates of future tokens masked).

Checkpoint selection on validation **factual** loss only: CRN encoder = one-step teacher-forced MSE; CRN decoder and CT =
`val_roll`, the mean squared error (in train std units) of 30-token factual rollouts with the observed treatments from
tokens 6, 18, 30, 42, 54 (days 8, 20, 32, 44, 56) of every validation episode (both 06:00 and 18:00 phases on v2).
Evaluations are at most every max_seconds / 15 seconds of training (and at the end), so they cost a bounded share.

usage (DGX):
  python3 tools/worldmodel/wm4/causal.py train --method crn|ct --dataset obs_v19|v2_v19 --seed S --out RUN
        [--config JSON] --max-seconds N
  python3 tools/worldmodel/wm4/causal.py grid --method crn|ct --seed 0 --out RUN --max-seconds-each 1200
  python3 tools/worldmodel/wm4/causal.py effects --run RUN --split obs|v2 --cases JSON --out DIR
"""
from __future__ import annotations

import argparse
import copy
import faulthandler
import json
import shutil
import signal
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm4  # noqa: F401
from wm1 import common as c
from wm1 import data as d
from wm1 import m1_gru as g
from wm2 import effects as ef
from wm2 import v2
from wm4 import tokens as tk

torch = g.torch
if torch is not None:
    import torch.nn as nn
    import torch.nn.functional as F
    from wm4.third_party.causal_transformer.utils import BRTreatmentOutcomeHead, grad_reverse  # noqa: F401
    from wm4.third_party.causal_transformer.utils_lstm import VariationalLSTM
    from wm4.third_party.causal_transformer.utils_transformer import (RelativePositionalEncoding,
                                                                      TransformerMultiInputBlock)

HORIZON = 30
TAU = HORIZON - 1
VAL_STARTS = (6, 18, 30, 42, 54)        # 06:00 of days 8, 20, 32, 44, 56
N_OUT = 37
N_COV = 30

# The authors' cancer simulator trains on 10,000 sequences (batch 64 / 128; the CRN decoder on ~55 windows per sequence,
# batch 1,024): epochs here are scaled so that the number of optimizer steps equals theirs (METHODS.md D7).
PAPER_TRAIN = 10_000
PAPER_WINDOWS = 55


def epochs_for(paper_epochs: int, n_items: int, paper_items: int, batch: int, scale: bool = True) -> int:
    """Epochs over `n_items` with the paper's number of optimizer steps (`paper_epochs` over `paper_items`);
    `scale` False (unit tests): `paper_epochs` itself."""
    if not scale:
        return int(paper_epochs)
    steps = paper_epochs * int(np.ceil(paper_items / batch))
    return int(np.ceil(steps / max(1, n_items // batch)))


# GOAL.md 7.4: the paper's defaults for its synthetic benchmark (the authors' cancer_sim γ = 4 configs) with hidden
# size 128 (CRN) / d_model 128, 2 layers, 4 heads (CT).
DEFAULTS = {
    "crn": {"enc_hidden": 128, "enc_br": 24, "enc_fc": 96, "enc_dropout": 0.2, "enc_lr": 1e-3, "enc_batch": 64,
            "enc_epochs": 100, "dec_br": 24, "dec_fc": 12, "dec_dropout": 0.1, "dec_lr": 1e-3, "dec_batch": 1024,
            "dec_epochs": 100, "alpha": 1.0, "enc_time_share": 0.5},
    "ct": {"d_model": 128, "layers": 2, "heads": 4, "br": 16, "fc": 48, "dropout": 0.1, "lr": 0.01, "batch": 128,
           "epochs": 150, "alpha": 0.01, "beta": 0.99, "max_rel": 15},
}
# GOAL.md 7.4: one tuning grid per method on obs_v19, at most 4 configurations (METRICS.md part b).
GRID = {
    "crn": [{}, {"enc_br": 64, "enc_fc": 128, "dec_br": 64, "dec_fc": 32}, {"enc_lr": 1e-4, "dec_lr": 1e-4},
            {"enc_br": 64, "enc_fc": 128, "dec_br": 64, "dec_fc": 32, "enc_dropout": 0.1}],
    "ct": [{}, {"lr": 1e-3}, {"lr": 1e-3, "br": 64, "fc": 128}, {"lr": 1e-4, "br": 64, "fc": 128}],
}


class Gate:
    """Evaluate at most every `gap` seconds (and always when `force`)."""

    def __init__(self, gap: float):
        self.gap, self.last = gap, -float("inf")

    def __call__(self, due: bool, force: bool = False) -> bool:
        if force or (due and time.time() - self.last >= self.gap):
            self.last = time.time()
            return True
        return False


def alpha_at(p: float, alpha_max: float) -> float:
    """The authors' `AlphaRise` 'exp' schedule."""
    return float((2.0 / (1.0 + np.exp(-10.0 * p)) - 1.0) * alpha_max)


# ------------------------------------------------------------------ data -------------------------------------------
class Data:
    """Hourly arrays of a set of episodes (float32), tokenized on demand at any phase."""

    def __init__(self, eps: list, variant: d.Variant):
        self.names = variant.state_names
        self.raws = [tk.raw_of(ep, variant) for ep in eps]
        self.seeds = [ep.seed for ep in eps]

    def seqs(self, phases: list[int], norm: tk.Norm) -> list[tk.Seq]:
        return [tk.tokenize(r, p, norm) for r, p in zip(self.raws, phases)]


def stack(seqs: list[tk.Seq], device) -> dict:
    t = lambda a: torch.tensor(np.stack(a), dtype=torch.float32, device=device)
    y, v, a = t([s.y for s in seqs]), t([s.v for s in seqs]), t([s.a for s in seqs])
    prev_a = torch.cat([torch.zeros_like(a[:, :1]), a[:, :-1]], 1)
    return {"y": y, "v": v, "a": a, "prev_a": prev_a}


def targets(bt: dict, norm_t: dict) -> torch.Tensor:
    """Normalized change of the raw daily state, y_{t+1} − y_t in dstd units: [B, K − 1, 37]."""
    raw = bt["y"] * norm_t["std"] + norm_t["mean"]
    return (raw[:, 1:] - raw[:, :-1]) / norm_t["dstd"]


def norm_tensors(norm: tk.Norm, device) -> dict:
    t = lambda x: torch.tensor(x, dtype=torch.float32, device=device)
    return {"mean": t(norm.mean), "std": t(norm.std), "dstd": t(norm.dstd)}


def classes(a: torch.Tensor) -> torch.Tensor:
    return a[..., : tk.N_CLASSES].argmax(-1)


def ce(logits: torch.Tensor, target: torch.Tensor) -> torch.Tensor:
    """Per-position cross entropy [B, K] (target: class indices, or probabilities for the confusion loss); the authors'
    `bce(..., 'multiclass')` with `reduction='none'`."""
    return F.cross_entropy(logits.permute(0, 2, 1), target if target.dtype == torch.long else target.permute(0, 2, 1),
                           reduction="none")


# ------------------------------------------------------------------ models -----------------------------------------
if torch is not None:
    def head(seq_hidden: int, br: int, fc: int, alpha: float, balancing: str) -> nn.Module:
        """`BRTreatmentOutcomeHead` with a 4-class treatment classifier and the 5-value treatment as the outcome
        head's input (METHODS.md D5)."""
        h = BRTreatmentOutcomeHead(seq_hidden, br, fc, tk.N_TREAT, N_OUT, alpha, update_alpha=False,
                                   balancing=balancing)
        h.linear3 = nn.Linear(fc, tk.N_CLASSES)
        return h

    class CRNEncoder(nn.Module):
        def __init__(self, hp: dict):
            super().__init__()
            self.lstm = VariationalLSTM(tk.N_TREAT + N_COV + N_OUT, hp["enc_hidden"], 1, hp["enc_dropout"])
            self.head = head(hp["enc_hidden"], hp["enc_br"], hp["enc_fc"], 0.0, "grad_reverse")
            self.out_dropout = hp["enc_dropout"]

        def br(self, bt: dict) -> torch.Tensor:
            x = torch.cat([bt["prev_a"], bt["v"], bt["y"]], -1)
            return self.head.build_br(self.lstm(x))

        def forward(self, bt: dict):
            br = self.br(bt)
            return self.head.build_treatment(br), self.head.build_outcome(br, bt["a"]), br

    class CRNDecoder(nn.Module):
        def __init__(self, hp: dict):
            super().__init__()
            self.lstm = VariationalLSTM(tk.N_TREAT + N_OUT, hp["enc_br"], 1, hp["dec_dropout"])
            self.head = head(hp["enc_br"], hp["dec_br"], hp["dec_fc"], 0.0, "grad_reverse")

        def forward(self, prev_a, y, a, init):
            br = self.head.build_br(self.lstm(torch.cat([prev_a, y], -1), init_states=init))
            return self.head.build_treatment(br), self.head.build_outcome(br, a), br

        def step(self, prev_a, y, a, state):
            """One decoder step (eval, no dropout): inputs [B, 5], [B, 37], [B, 5]; state (h, c)."""
            cell = self.lstm.lstm_layers[0]
            h, cc = cell(torch.cat([prev_a, y], -1), state)
            br = self.head.build_br(h)
            return self.head.build_outcome(br, a), (h, cc)

    class CT(nn.Module):
        def __init__(self, hp: dict):
            super().__init__()
            dm, heads = hp["d_model"], hp["heads"]
            self.t_in = nn.Linear(tk.N_TREAT, dm)
            self.o_in = nn.Linear(N_OUT, dm)
            self.v_in = nn.Linear(N_COV, dm)
            self.pe_k = RelativePositionalEncoding(hp["max_rel"], dm // heads, True)
            self.pe_v = RelativePositionalEncoding(hp["max_rel"], dm // heads, True)
            self.blocks = nn.ModuleList([TransformerMultiInputBlock(dm, heads, dm // heads, dm * 4, hp["dropout"],
                                                                    hp["dropout"], self_positional_encoding_k=self.pe_k,
                                                                    self_positional_encoding_v=self.pe_v, n_inputs=3)
                                         for _ in range(hp["layers"])])
            self.drop = nn.Dropout(hp["dropout"])
            self.head = head(dm, hp["br"], hp["fc"], 0.0, "domain_confusion")

        def br(self, bt: dict, split: torch.Tensor | None) -> torch.Tensor:
            """split [B]: covariates of positions >= split are masked (zeroed and excluded); None: none masked."""
            B, K, _ = bt["y"].shape
            active = torch.ones(B, K, 1, device=bt["y"].device)
            v, act_v = bt["v"], active.clone()
            if split is not None:
                keep = (torch.arange(K, device=v.device)[None, :] < split[:, None]).float()[..., None]
                v, act_v = v * keep, active * keep
            x_t, x_o, x_v = self.t_in(bt["prev_a"]), self.o_in(bt["y"]), self.v_in(v)
            x_s = torch.zeros(B, 1, x_t.shape[-1], device=x_t.device)
            for blk in self.blocks:
                x_t, x_o, x_v = blk((x_t, x_o, x_v), x_s, active, act_v)
            if split is None:
                x = (x_o + x_t + x_v) / 3
            else:
                x = torch.where(act_v.bool(), (x_o + x_t + x_v) / 3, (x_o + x_t) / 2)
            return self.head.build_br(self.drop(x))

        def forward(self, bt: dict, split=None, detach_treatment: bool = False):
            br = self.br(bt, split)
            return self.head.build_treatment(br, detach_treatment), self.head.build_outcome(br, bt["a"]), br

    class EMA:
        """Exponential moving average of a parameter list (`torch_ema.ExponentialMovingAverage`'s update rule)."""

        def __init__(self, params: list, beta: float):
            self.params, self.beta = list(params), beta
            self.shadow = [p.detach().clone() for p in self.params]
            self.n = 0

        def update(self) -> None:
            self.n += 1
            decay = min(self.beta, (1 + self.n) / (10 + self.n))
            with torch.no_grad():
                for s, p in zip(self.shadow, self.params):
                    s.mul_(decay).add_(p.detach(), alpha=1 - decay)

        def swap(self) -> None:
            with torch.no_grad():
                for s, p in zip(self.shadow, self.params):
                    tmp = p.detach().clone()
                    p.copy_(s)
                    s.copy_(tmp)


# ------------------------------------------------------------------ rollouts ---------------------------------------
class Job:
    """A rollout: history `seq` (tokens 0 … k true), planned treatments `plan` [H, 5] for tokens k … k + H − 1."""

    def __init__(self, seq: tk.Seq, k: int, plan: np.ndarray):
        self.seq, self.k, self.plan = seq, int(k), np.asarray(plan, dtype=np.float64)


def _clamp_t(raw: torch.Tensor, names) -> torch.Tensor:
    out = raw.clamp(min=0.0)
    for n in tk.UNIT:
        if n in names:
            i = names.index(n)
            out[..., i] = out[..., i].clamp(max=1.0)
    return out


def rollout(model: dict, jobs: list[Job], device, chunk: int = 512) -> list[np.ndarray]:
    """Raw daily states [H, 37] of tokens k + 1 … k + H for every job."""
    out = []
    for i in range(0, len(jobs), chunk):
        out += _rollout(model, jobs[i: i + chunk], device)
    return out


def _rollout(m: dict, jobs: list[Job], device) -> list[np.ndarray]:
    H = len(jobs[0].plan)
    assert all(len(j.plan) == H for j in jobs)
    nt, names = m["norm_t"], m["names"]
    L = max(j.k + 1 + H for j in jobs)
    B = len(jobs)
    y = np.zeros((B, L, N_OUT)); v = np.zeros((B, L, N_COV)); a = np.zeros((B, L, tk.N_TREAT))
    a[:, :, 0] = 1.0
    for b, j in enumerate(jobs):
        n = j.k + 1
        y[b, :n], v[b, :n], a[b, :j.k] = j.seq.y[:n], j.seq.v[:n], j.seq.a[:j.k]
        a[b, j.k: j.k + H] = j.plan
    T = lambda x: torch.tensor(x, dtype=torch.float32, device=device)
    y, v, a = T(y), T(v), T(a)
    ks = torch.tensor([j.k for j in jobs], device=device)
    bidx = torch.arange(B, device=device)
    preds = []
    with torch.no_grad():
        if m["method"] == "ct":
            net = m["net"]
            net.eval()
            for step in range(H):
                prev_a = torch.cat([torch.zeros_like(a[:, :1]), a[:, :-1]], 1)
                bt = {"y": y, "v": v, "a": a, "prev_a": prev_a}
                _, out, _ = net(bt, split=ks + 1)
                pos = ks + step
                raw = y[bidx, pos] * nt["std"] + nt["mean"] + out[bidx, pos] * nt["dstd"]
                raw = _clamp_t(raw, names)
                preds.append(raw)
                if step < H - 1:
                    y[bidx, pos + 1] = (raw - nt["mean"]) / nt["std"]
        else:
            enc, dec = m["enc"], m["dec"]
            enc.eval(); dec.eval()
            prev_a = torch.cat([torch.zeros_like(a[:, :1]), a[:, :-1]], 1)
            bt = {"y": y, "v": v, "a": a, "prev_a": prev_a}
            br = enc.br(bt)[bidx, ks]
            out = enc.head.build_outcome(br, a[bidx, ks])
            cur = y[bidx, ks] * nt["std"] + nt["mean"]
            raw = _clamp_t(cur + out * nt["dstd"], names)
            preds.append(raw)
            state = (br, br)
            for step in range(1, H):
                yn = (raw - nt["mean"]) / nt["std"]
                out, state = dec.step(a[bidx, ks + step - 1], yn, a[bidx, ks + step], state)
                raw = _clamp_t(raw + out * nt["dstd"], names)
                preds.append(raw)
    st = torch.stack(preds, 1).double().cpu().numpy()
    return [st[b] for b in range(B)]


def val_jobs(seqs: list[tk.Seq]) -> list[Job]:
    """`val_roll`'s jobs: 30-token factual rollouts with the observed treatments from the `VAL_STARTS` tokens."""
    return [Job(s, k, s.a[k: k + HORIZON]) for s in seqs for k in VAL_STARTS if k + HORIZON < len(s.rows)]


def roll_loss(model: dict, jobs: list[Job], device) -> float:
    """Mean squared error of the rolled-out raw daily state, in train std units, over jobs, tokens and features."""
    preds = rollout(model, jobs, device)
    std = model["norm"].std
    err = [((p - j.seq.raw[j.k + 1: j.k + 1 + HORIZON]) / std) ** 2 for p, j in zip(preds, jobs)]
    return float(np.mean(err))


# ------------------------------------------------------------------ training ---------------------------------------
def load_data(dataset: str, root: Path):
    """(train Data, val Data, val phases, train phase mode): obs_v19 — observed train / val episodes, phase 06:00;
    v2_v19 — every member of the train families (phase drawn per sequence), val = `v19_fact` val (06:00 and 18:00)."""
    from wm4 import m1_obs
    m1_obs.register()
    if dataset == "obs_v19":
        gr = c.load_groups(["train", "val"], root, "obs_v19")
        tr, va = gr["train"], gr["val"]
        val_phases = [tk.PHASE_06]
        mode = "fixed_06"
    elif dataset == "v2_v19":
        tr = c.load_groups(["train"], root, "v2_v19")["train"]
        va = c.load_groups(["val"], root, "v2_v19_fact")["val"]
        val_phases = [tk.PHASE_06, tk.PHASE_18]
        mode = "uniform"
    else:
        raise SystemExit(f"unknown dataset {dataset}")
    variant = g.dataset_variant(c.dataset(dataset), "sev-blind_action-aware")
    return Data(tr, variant), Data(va, variant), val_phases, mode, variant


def train_phases(data: Data, mode: str, rng: np.random.Generator) -> list[int]:
    if mode == "fixed_06":
        return [tk.PHASE_06] * len(data.raws)
    return [int(x) for x in rng.integers(0, tk.TOKEN_ROWS, len(data.raws))]


def val_seqs(data: Data, phases: list[int], norm: tk.Norm) -> list[tk.Seq]:
    return [s for p in phases for s in data.seqs([p] * len(data.raws), norm)]


def batches(n: int, size: int, rng: np.random.Generator):
    order = rng.permutation(n)
    for i in range(0, n - size + 1 if n >= size else 1, size):
        yield order[i: i + size]


def train_crn(hp: dict, tr: Data, val: list[tk.Seq], mode: str, norm: tk.Norm, seed: int, device, max_seconds: float,
              log, t0: float) -> tuple[dict, dict]:
    rng = np.random.default_rng([seed, 0])
    nt = norm_tensors(norm, device)
    enc = CRNEncoder(hp).to(device)
    opt = torch.optim.Adam(enc.parameters(), lr=hp["enc_lr"])
    vb = stack(val, device)
    best, best_ep, best_state, stop = float("inf"), -1, None, "epochs"
    enc_deadline = t0 + max_seconds * hp["enc_time_share"]
    n_ep = epochs_for(hp["enc_epochs"], len(tr.raws), PAPER_TRAIN, hp["enc_batch"], hp.get("paper_steps", True))
    every = max(1, n_ep // 100)
    gate = Gate(max_seconds / 15)
    steps = 0
    for ep in range(n_ep):
        enc.head.alpha = alpha_at(ep / n_ep, hp["alpha"])
        seqs = tr.seqs(train_phases(tr, mode, rng), norm)
        enc.train()
        losses = []
        for idx in batches(len(seqs), hp["enc_batch"], rng):
            bt = stack([seqs[i] for i in idx], device)
            tpred, opred, _ = enc(bt)
            mse = ((opred[:, :-1] - targets(bt, nt)) ** 2).mean()
            cls = ce(tpred[:, :-1], classes(bt["a"][:, :-1])).mean()
            loss = mse + cls
            opt.zero_grad(); loss.backward(); opt.step()
            losses.append((mse.item(), cls.item()))
            steps += 1
        last = ep == n_ep - 1 or time.time() > enc_deadline
        if gate(ep % every == 0, force=last or ep == 0):
            enc.eval()
            with torch.no_grad():
                _, opred, _ = enc(vb)
                vloss = float(((opred[:, :-1] - targets(vb, nt)) ** 2).mean())
            log({"stage": "encoder", "epoch": ep, "epochs": n_ep, "steps": steps, "alpha": enc.head.alpha,
                 "mse": float(np.mean([x[0] for x in losses])), "ce": float(np.mean([x[1] for x in losses])),
                 "val_one_step": vloss, "t": time.time() - t0})
            if vloss < best:
                best, best_ep, best_state = vloss, ep, copy.deepcopy(enc.state_dict())
        if time.time() > enc_deadline:
            stop = "time"
            break
    enc.load_state_dict(best_state)
    enc.eval()
    for p in enc.parameters():
        p.requires_grad_(False)
    info = {"encoder": {"best_epoch": best_ep, "best_val_one_step": best, "epochs_run": ep + 1, "epochs": n_ep,
                        "stop": stop}}
    dec = CRNDecoder(hp).to(device)
    opt = torch.optim.Adam(dec.parameters(), lr=hp["dec_lr"])
    model = {"method": "crn", "enc": enc, "dec": dec, "norm": norm, "norm_t": nt, "names": tr.names}
    vjobs = val_jobs(val)
    best, best_ep, best_state, stop = float("inf"), -1, None, "epochs"
    K = len(tr.seqs([tk.PHASE_06], norm)[0].rows) if tr.raws else 0
    starts = K - 2 - TAU                       # windows k = 0 … K − 2 − τ
    n_ep = epochs_for(hp["dec_epochs"], len(tr.raws) * (starts + 1), PAPER_TRAIN * PAPER_WINDOWS, hp["dec_batch"],
                      hp.get("paper_steps", True))
    every = max(1, n_ep // 50)
    gate = Gate(max_seconds / 15)
    steps = 0
    for ep in range(n_ep):
        dec.head.alpha = alpha_at(ep / n_ep, hp["alpha"])
        seqs = tr.seqs(train_phases(tr, mode, rng), norm)
        full = stack(seqs, device)
        with torch.no_grad():
            enc_br = enc.br(full)
        win = np.array([(i, k) for i in range(len(seqs)) for k in range(starts + 1)])
        win = win[rng.permutation(len(win))]
        dec.train()
        losses = []
        tgt_all = targets(full, nt)
        for i in range(0, len(win) - hp["dec_batch"] + 1, hp["dec_batch"]):
            w = win[i: i + hp["dec_batch"]]
            si, ki = torch.tensor(w[:, 0], device=device), torch.tensor(w[:, 1], device=device)
            pos = ki[:, None] + 1 + torch.arange(TAU, device=device)[None]         # decoder positions k+1 … k+τ
            init = enc_br[si, ki]
            prev_a = full["a"][si[:, None], pos - 1]
            yy = full["y"][si[:, None], pos]
            aa = full["a"][si[:, None], pos]
            tpred, opred, _ = dec(prev_a, yy, aa, init)
            mse = ((opred - tgt_all[si[:, None], pos]) ** 2).mean()
            cls = ce(tpred, classes(aa)).mean()
            loss = mse + cls
            opt.zero_grad(); loss.backward(); opt.step()
            losses.append((mse.item(), cls.item()))
            steps += 1
        if gate(ep % every == 0, force=ep == n_ep - 1 or time.time() - t0 > max_seconds or ep == 0):
            te = time.time()
            vloss = roll_loss(model, vjobs, device)
            log({"stage": "decoder", "epoch": ep, "epochs": n_ep, "steps": steps, "eval_s": time.time() - te,
                 "alpha": dec.head.alpha,
                 "mse": float(np.mean([x[0] for x in losses])), "ce": float(np.mean([x[1] for x in losses])),
                 "val_roll": vloss, "t": time.time() - t0})
            if vloss < best:
                best, best_ep, best_state = vloss, ep, copy.deepcopy(dec.state_dict())
        if time.time() - t0 > max_seconds:
            stop = "time"
            break
    dec.load_state_dict(best_state)
    info["decoder"] = {"best_epoch": best_ep, "best_val_roll": best, "epochs_run": ep + 1, "epochs": n_ep, "stop": stop}
    info["val_roll"] = best
    return model, info


def train_ct(hp: dict, tr: Data, val: list[tk.Seq], mode: str, norm: tk.Norm, seed: int, device, max_seconds: float,
             log, t0: float) -> tuple[dict, dict]:
    rng = np.random.default_rng([seed, 0])
    nt = norm_tensors(norm, device)
    net = CT(hp).to(device)
    tnames = ("head.linear2", "head.linear3")
    t_params = [p for n, p in net.named_parameters() if n.startswith(tnames)]
    o_params = [p for n, p in net.named_parameters() if not n.startswith(tnames)]
    opt0 = torch.optim.Adam(o_params, lr=hp["lr"])
    opt1 = torch.optim.Adam(t_params, lr=hp["lr"])
    ema_t, ema_o = EMA(t_params, hp["beta"]), EMA(o_params, hp["beta"])
    model = {"method": "ct", "net": net, "norm": norm, "norm_t": nt, "names": tr.names}
    vjobs = val_jobs(val)
    best, best_ep, best_state, stop = float("inf"), -1, None, "epochs"
    n_ep = epochs_for(hp["epochs"], len(tr.raws), PAPER_TRAIN, hp["batch"], hp.get("paper_steps", True))
    every = max(1, n_ep // 50)
    gate = Gate(max_seconds / 15)
    steps = 0
    for ep in range(n_ep):
        alpha = alpha_at(ep / n_ep, hp["alpha"])
        seqs = tr.seqs(train_phases(tr, mode, rng), norm)
        net.train()
        losses = []
        for idx in batches(len(seqs), hp["batch"], rng):
            bt = stack([seqs[i] for i in idx], device)
            K = bt["y"].shape[1]
            # augment with a copy whose covariates are masked from a random split on (the authors' masked vitals)
            split = torch.cat([torch.full((len(idx),), K, device=device),
                               torch.tensor(rng.integers(0, K + 1, len(idx)), device=device)])
            bt2 = {k: torch.cat([v, v], 0) for k, v in bt.items()}
            tgt, cls_t = targets(bt2, nt), classes(bt2["a"][:, :-1])
            tpred, opred, _ = net(bt2, split)
            mse = ((opred[:, :-1] - tgt) ** 2).mean()
            uniform = torch.full_like(tpred[:, :-1], 1.0 / tk.N_CLASSES)
            conf = alpha * ce(tpred[:, :-1], uniform).mean()
            loss0 = mse + conf
            opt0.zero_grad(); opt1.zero_grad(); loss0.backward(); opt0.step(); ema_o.update()
            tpred, _, _ = net(bt2, split, detach_treatment=True)
            loss1 = alpha * ce(tpred[:, :-1], cls_t).mean()
            opt0.zero_grad(); opt1.zero_grad(); loss1.backward(); opt1.step(); ema_t.update()
            losses.append((mse.item(), conf.item(), loss1.item()))
            steps += 1
        if gate(ep % every == 0, force=ep == n_ep - 1 or time.time() - t0 > max_seconds or ep == 0):
            ema_t.swap(); ema_o.swap()
            te = time.time()
            vloss = roll_loss(model, vjobs, device)
            state = copy.deepcopy(net.state_dict())
            ema_t.swap(); ema_o.swap()
            log({"stage": "ct", "epoch": ep, "epochs": n_ep, "steps": steps, "eval_s": time.time() - te, "alpha": alpha, "mse": float(np.mean([x[0] for x in losses])),
                 "conf": float(np.mean([x[1] for x in losses])), "cls": float(np.mean([x[2] for x in losses])),
                 "val_roll": vloss, "t": time.time() - t0})
            if vloss < best:
                best, best_ep, best_state = vloss, ep, state
        if time.time() - t0 > max_seconds:
            stop = "time"
            break
    net.load_state_dict(best_state)                 # the EMA weights of the best evaluation
    return model, {"best_epoch": best_ep, "val_roll": best, "epochs_run": ep + 1, "epochs": n_ep, "stop": stop}


def save(model: dict, out: Path) -> None:
    if model["method"] == "ct":
        torch.save(model["net"].state_dict(), out / "model.pt")
    else:
        torch.save({"enc": model["enc"].state_dict(), "dec": model["dec"].state_dict()}, out / "model.pt")


def load(run: Path, device) -> tuple[dict, dict]:
    cfg = json.loads((Path(run) / "config.json").read_text())
    hp, norm = cfg["hyper"], tk.Norm.from_json(cfg["norm"])
    sd = torch.load(Path(run) / "model.pt", map_location=device)
    if cfg["method"] == "ct":
        net = CT(hp).to(device); net.load_state_dict(sd); net.eval()
        m = {"method": "ct", "net": net}
    else:
        enc, dec = CRNEncoder(hp).to(device), CRNDecoder(hp).to(device)
        enc.load_state_dict(sd["enc"]); dec.load_state_dict(sd["dec"]); enc.eval(); dec.eval()
        m = {"method": "crn", "enc": enc, "dec": dec}
    m |= {"norm": norm, "norm_t": norm_tensors(norm, device), "names": tuple(cfg["state_names"])}
    return m, cfg


def run_train(method: str, dataset: str, seed: int, out: Path, overrides: dict, max_seconds: float, root: Path,
              device_name: str = "auto", data=None, heartbeat: Path | None = None) -> dict:
    t0 = time.time()
    out.mkdir(parents=True, exist_ok=True)
    device = g.pick_device(device_name)
    torch.manual_seed(seed)
    tr, va, vphases, mode, variant = data if data is not None else load_data(dataset, root)
    norm = tk.Norm.fit(tr.raws)
    val = val_seqs(va, vphases, norm)
    hp = dict(DEFAULTS[method]) | overrides
    logf = (out / "train_log.jsonl").open("w")

    def log(rec):
        logf.write(json.dumps(rec) + "\n"); logf.flush()
        if heartbeat is not None:          # the grid's own log: the queue's watchdog reads OUT/train_log.jsonl
            with open(heartbeat, "a") as hb:
                hb.write(json.dumps({"grid_member": str(out.name)} | rec) + "\n")
        print(json.dumps(rec), flush=True)

    cfg = {"method": method, "dataset": dataset, "seed": seed, "hyper": hp, "overrides": overrides,
           "max_seconds": max_seconds, "phase_mode": mode, "val_phases": vphases, "norm": norm.to_json(),
           "state_names": list(variant.state_names), "input_names": list(variant.input_names()),
           "train_episodes": len(tr.raws), "val_episodes": len(va.raws), "code_commit": c.git_commit(),
           "manifest_sha256": c.manifest_sha(dataset), "device": device, "torch": torch.__version__,
           "numpy": np.__version__, "argv": sys.argv}
    c.write_json(out / "config.json", cfg)
    fn = train_ct if method == "ct" else train_crn
    model, info = fn(hp, tr, val, mode, norm, seed, device, max_seconds, log, t0)
    logf.close()
    save(model, out)
    info["seconds"] = time.time() - t0
    cfg["result"] = info
    c.write_json(out / "config.json", cfg)
    c.write_json(out / "val_metrics.json", info)
    print(f"done {method} {dataset} s{seed}: val_roll {info['val_roll']:.5f} {info['seconds']:.0f}s", flush=True)
    return info


def cmd_grid(a) -> int:
    """The tuning grid (GOAL.md 7.4, METRICS.md part b) on obs_v19, seed 0, val factual loss only: each configuration in
    `OUT/grid/<i>`; the best is copied to OUT (it is the method's `_obs_s0` run)."""
    t0 = time.time()
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    hb = out / "train_log.jsonl"            # every member's records, live (the queue's watchdog reads it)
    data = load_data("obs_v19", Path(a.root))
    res = []
    for i, ov in enumerate(GRID[a.method]):
        info = run_train(a.method, "obs_v19", a.seed, out / "grid" / str(i), ov, a.max_seconds_each, Path(a.root),
                         a.device, data, heartbeat=hb)
        res.append({"config": i, "overrides": ov, "val_roll": info["val_roll"], "seconds": info["seconds"]})
    best = min(res, key=lambda r: (r["val_roll"], r["config"]))
    hb.rename(out / "grid_log.jsonl")
    for f in ("config.json", "model.pt", "train_log.jsonl", "val_metrics.json"):
        shutil.copy(out / "grid" / str(best["config"]) / f, out / f)
    c.write_json(out / "grid.json", {"method": a.method, "results": res, "chosen": best["config"],
                                     "chosen_overrides": best["overrides"], "seconds": time.time() - t0,
                                     "code_commit": c.git_commit()})
    print(f"grid {a.method}: chosen {best['config']} {best['overrides']} val_roll {best['val_roll']:.5f}")
    return 0


# ------------------------------------------------------------------ prediction contract ----------------------------
def predict_cases(model: dict, fams: list, cases: list, device) -> dict:
    """For each case (world, arm, c, replicate r): predicted effect = rollout with the arm − rollout with none, both
    from the same observed history (`none_r` of the family, tokens at the cast row's phase), read at tokens + 7 and
    + 30 (rows c + 168, c + 720). Deaths are the summed daily deaths of the five causes."""
    names = model["names"]
    idx = {k: names.index(k) for k in ("s.food_stock", "g.alive", "g.settlements_alive")}
    didx = [names.index(n) for n in tk.DEATHS]
    by = {(f.seed): f for f in fams}
    jobs, keys = [], []
    for x in cases:
        ep = by[x.seed].factual[x.rep]
        if tuple(ep.names) != tuple(names):
            raise ValueError(f"{x.key}: state columns differ from the model's")
        raw = tk.raw_of(ep, d.variant_for("sev-blind_action-aware", ep.names))
        seq = tk.tokenize(raw, tk.phase_of_row(x.cast_row), model["norm"])
        k = tk.token_of_row(x.cast_row)
        assert seq.rows[k] == x.cast_row
        kind, amount = v2.ARM_SPEC[x.arm]
        arm_plan = np.vstack([tk.arm_treatment(kind, amount)] + [tk.NONE] * (HORIZON - 1))
        jobs += [Job(seq, k, arm_plan), Job(seq, k, np.vstack([tk.NONE] * HORIZON))]
        keys.append(x.key)
    ros = rollout(model, jobs, device)
    out = {}
    for i, (x, key) in enumerate(zip(cases, keys)):
        ar, fa = ros[2 * i], ros[2 * i + 1]
        eff = {}
        for o in v2.OUTCOMES:
            eff[o] = {}
            for lag in x.truth[o]:
                n = v2.LAGS[lag] // tk.TOKEN_ROWS
                if o == "deaths":
                    eff[o][lag] = float(ar[:n, didx].sum() - fa[:n, didx].sum())
                else:
                    j = idx[{"food_stock": "s.food_stock", "alive": "g.alive", "settlements_alive": "g.settlements_alive"}[o]]
                    eff[o][lag] = float(ar[n - 1, j] - fa[n - 1, j])
        out[key] = {"effects": eff, "shift": 0.0, "shift_predicted": False}
    return out


def cmd_effects(a) -> int:
    from wm3 import effects3 as e3
    from wm4 import effects4 as e4
    from wm4 import m1_obs
    m1_obs.register()
    t0 = time.time()
    role = a.role
    target = Path(a.out) / ("effects.json" if role == "test" else "effects_val.json")
    if target.exists():
        raise SystemExit(f"{target} exists: computed once per run")
    device = g.pick_device(a.device)
    model, cfg = load(Path(a.run), device)
    root = Path(a.root)
    if role == "val" and a.split == "obs":           # a dry run on the val families (before part b)
        cases = e4.load_cases(root, "val")
    else:
        cases = [ef.case_from(x) for x in json.loads(Path(a.cases).read_text())["cases"][role]]
    paths = e4.family_dirs(root, role) if a.split == "obs" else ef.family_dirs(root, role)
    fams = e3.load_families(paths, role)
    preds = predict_cases(model, fams, cases, device)
    # factual test loss (reported): val_roll's construction on the test factual episodes
    ds = "obs_v19" if a.split == "obs" else "v2_v19_fact"
    variant = g.dataset_variant(c.dataset(ds), "sev-blind_action-aware")
    te = Data(c.load_groups([role], root, ds)[role], variant)
    phases = [tk.PHASE_06] if a.split == "obs" else [tk.PHASE_06, tk.PHASE_18]
    fact = roll_loss(model, val_jobs(val_seqs(te, phases, model["norm"])), device)
    out = {"run": a.run, "method": cfg["method"], "dataset": cfg["dataset"], "seed": cfg["seed"], "split": a.split,
           "code_commit": c.git_commit(), "train_code_commit": cfg["code_commit"], "argv": sys.argv,
           "role": role, "predictions": preds, "test_fact_roll": fact,
           "cases_sha256": ef._sha(Path(a.cases)) if a.cases else None,
           "seconds": time.time() - t0}
    if target.exists():
        raise SystemExit(f"{target} exists")
    c.write_json(target, out)
    print(f"effects {cfg['method']} {a.split} done {time.time() - t0:.0f}s fact {fact:.5f}")
    return 0


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("train")
    p.add_argument("--method", required=True, choices=["crn", "ct"])
    p.add_argument("--dataset", required=True, choices=["obs_v19", "v2_v19"])
    p.add_argument("--seed", type=int, default=0)
    p.add_argument("--config", default="{}", help="hyper-parameter overrides (JSON), or @grid.json (its chosen ones)")
    p.add_argument("--max-seconds", dest="max_seconds", type=float, required=True)
    p = sub.add_parser("grid")
    p.add_argument("--method", required=True, choices=["crn", "ct"])
    p.add_argument("--seed", type=int, default=0)
    p.add_argument("--max-seconds-each", dest="max_seconds_each", type=float, default=1200)
    p = sub.add_parser("effects")
    p.add_argument("--run", required=True)
    p.add_argument("--split", required=True, choices=["obs", "v2"])
    p.add_argument("--cases", default=None, help="obs_base.json (obs) or effects_base.json (v2)")
    p.add_argument("--role", default="test", choices=["test", "val"])
    for p in sub.choices.values():
        p.add_argument("--root", default=str(c.DEFAULT_ROOT))
        p.add_argument("--device", default="auto")
        p.add_argument("--out", required=True)
    a = ap.parse_args(argv)
    if torch is None:
        raise SystemExit("torch is required")
    faulthandler.register(signal.SIGUSR1, file=sys.stderr, all_threads=True)
    if a.cmd == "train":
        ov = json.loads(Path(a.config[1:]).read_text())["chosen_overrides"] if a.config.startswith("@") else json.loads(a.config)
        run_train(a.method, a.dataset, a.seed, Path(a.out), ov, a.max_seconds, Path(a.root), a.device)
        return 0
    return cmd_grid(a) if a.cmd == "grid" else cmd_effects(a)


if __name__ == "__main__":
    raise SystemExit(main())
