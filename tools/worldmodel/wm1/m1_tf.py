"""M1b: settlement-level causal transformer world model (GOAL.md 9).

4 pre-LN layers, d_model 256, 4 heads, context 168 rows, no positional encoding besides the calendar: the
calendar one-hots enter through the input projection, i.e. as learned hour / season embeddings (a linear map of a
one-hot is an embedding table). Same inputs, heads (Gaussian change + fire logit), training recipe (teacher
forcing + k-step rollout loss, k 1 -> 24), early stopping and predictor contract as M1 (`m1_gru.py`). Rollouts use
a key/value cache trimmed to the last 168 rows, so a step costs one token, not a full re-encode.

usage:
  python3 tools/worldmodel/wm1/m1_tf.py train --out RUN_DIR --variant sev-blind_action-aware --seed 0
  python3 tools/worldmodel/wm1/m1_gru.py final --run RUN_DIR     # the shared test evaluation
"""
from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm1  # noqa: F401
from wm1 import data as d
from wm1 import evaluate as ev
from wm1 import m1_gru as g

torch = g.torch
if torch is not None:
    import torch.nn as nn
    import torch.nn.functional as F

DEFAULTS = dict(d_model=256, layers=4, heads=4, context=168, lr=3e-4, weight_decay=1e-4, batch=64, window=168,
                k_max=24, roll_starts=4, roll_weight=1.0, max_steps=6000, eval_every=300, patience=5, max_seconds=7200,
                warmup=168, grad_clip=1.0, threads=16)


if torch is not None:
    class Block(nn.Module):
        def __init__(self, dm: int, heads: int):
            super().__init__()
            self.h, self.dh = heads, dm // heads
            self.ln1, self.ln2 = nn.LayerNorm(dm), nn.LayerNorm(dm)
            self.qkv = nn.Linear(dm, 3 * dm)
            self.out = nn.Linear(dm, dm)
            self.mlp = nn.Sequential(nn.Linear(dm, 4 * dm), nn.GELU(), nn.Linear(4 * dm, dm))

        def forward(self, x, past_k, past_v, past_valid, valid):
            """x [B, Lq, d]; past_* [B, h, P, dh] / [B, P] (or None); valid [B, Lq]. Returns x and the new k, v."""
            B, Lq, dm = x.shape
            q, k, v = self.qkv(self.ln1(x)).split(dm, dim=-1)
            shape = lambda t: t.view(B, Lq, self.h, self.dh).transpose(1, 2)
            q, k, v = shape(q), shape(k), shape(v)
            causal = torch.tril(torch.ones(Lq, Lq, dtype=torch.bool, device=x.device))
            allowed = causal[None] & valid[:, None, :]                       # [B, Lq, Lq]
            allowed = allowed | torch.eye(Lq, dtype=torch.bool, device=x.device)[None]   # never an empty row
            if past_k is not None:
                K, V = torch.cat([past_k, k], 2), torch.cat([past_v, v], 2)
                allowed = torch.cat([past_valid[:, None, :].expand(B, Lq, past_valid.shape[1]), allowed], 2)
            else:
                K, V = k, v
            y = F.scaled_dot_product_attention(q, K, V, attn_mask=allowed[:, None])
            x = x + self.out(y.transpose(1, 2).reshape(B, Lq, dm))
            return x + self.mlp(self.ln2(x)), k, v

    class M1TF(nn.Module):
        def __init__(self, n_in: int, n_cont: int, dm: int = 256, layers: int = 4, heads: int = 4, context: int = 168):
            super().__init__()
            self.context = context
            self.proj = nn.Linear(n_in, dm)
            self.blocks = nn.ModuleList([Block(dm, heads) for _ in range(layers)])
            self.ln = nn.LayerNorm(dm)
            self.mu, self.logvar, self.fire = nn.Linear(dm, n_cont), nn.Linear(dm, n_cont), nn.Linear(dm, 1)

        def run(self, x, cache=None, valid=None):
            """x [B, L, n_in]; cache {"kv": [(k, v)] per layer, "valid": [B, P]} -> heads and the new tokens' (k, v)."""
            B, L, _ = x.shape
            if valid is None:
                valid = torch.ones(B, L, dtype=torch.bool, device=x.device)
            z = self.proj(x)
            kv = []
            for i, blk in enumerate(self.blocks):
                pk, pv = cache["kv"][i] if cache is not None else (None, None)
                z, k, v = blk(z, pk, pv, cache["valid"] if cache is not None else None, valid)
                kv.append((k, v))
            z = self.ln(z)
            return self.mu(z), self.logvar(z).clamp(-12.0, 8.0), self.fire(z).squeeze(-1), kv

        def forward(self, x, h=None):
            mu, logvar, fl, _ = self.run(x)
            return mu, logvar, fl, None


def append(cache: dict | None, kv: list, valid, keep: int | None = None) -> dict:
    """Append new tokens' keys / values; keep only the last `keep` positions when given."""
    if cache is None:
        new = {"kv": kv, "valid": valid}
    else:
        new = {"kv": [(torch.cat([pk, k], 2), torch.cat([pv, v], 2)) for (pk, pv), (k, v) in zip(cache["kv"], kv)],
               "valid": torch.cat([cache["valid"], valid], 1)}
    if keep is not None and new["valid"].shape[1] > keep:
        new = {"kv": [(k[:, :, -keep:], v[:, :, -keep:]) for k, v in new["kv"]], "valid": new["valid"][:, -keep:]}
    return new


def losses_tf(model, stepper: g.Stepper, bt: dict, k: int, roll_starts: int, rng: np.random.Generator):
    """As `m1_gru.losses`: teacher-forced NLL + BCE, and the k-step rollout loss from `roll_starts` positions."""
    L = bt["dtarget"].shape[1]
    inp = torch.cat([bt["x_state"], bt["calendar"], bt["actions"], bt["sev"]], dim=-1)
    mu, logvar, fl, kv = model.run(inp[:, :L])
    nll = 0.5 * (logvar + (bt["dtarget"] - mu) ** 2 / logvar.exp()).mean()
    bce = F.binary_cross_entropy_with_logits(fl, bt["fire_next"])
    tf = nll + bce
    B = inp.shape[0]
    js = torch.tensor(np.sort(rng.integers(0, L - k + 1, size=roll_starts)), device=inp.device)
    m = len(js)
    flat = lambda a: a.reshape(B * m, *a.shape[2:])
    rep = lambda t: t[:, None].expand(B, m, *t.shape[1:]).reshape(B * m, *t.shape[1:])
    # Each start sees the teacher-forced keys of rows <= j (causality makes them exact for a rollout after j).
    valid = (torch.arange(L, device=inp.device)[None, None, :] <= js[None, :, None]).expand(B, m, L).reshape(B * m, L)
    cache = {"kv": [(rep(kk), rep(vv)) for kk, vv in kv], "valid": valid}
    mu_j, fl_j = flat(mu[:, js]), flat(fl[:, js])
    cur = flat(bt["state"][:, js])
    sev_hold = flat(bt["sev"][:, js])
    cont = stepper.cont
    roll = 0.0
    ones = torch.ones(B * m, 1, dtype=torch.bool, device=inp.device)
    for i in range(k):
        cur = stepper.next_state(cur, mu_j, fl_j, hard=False)
        true = flat(bt["state"][:, js + i + 1])
        err = (cur[:, cont] - true[:, cont]) / stepper.std[cont]
        roll = roll + err.abs().mean() + F.binary_cross_entropy_with_logits(fl_j, true[:, stepper.lay.fire])
        if i == k - 1:
            break
        r = js + i + 1
        nxt = torch.cat([stepper.normalize(cur), flat(bt["calendar"][:, r]), flat(bt["actions"][:, r]), sev_hold], -1)
        mm, _, ff, new_kv = model.run(nxt.unsqueeze(1), cache, ones)
        cache = append(cache, new_kv, ones)
        mu_j, fl_j = mm[:, 0], ff[:, 0]
    roll = roll / k
    return tf, {"nll": nll.item(), "bce": bce.item(), "roll": roll.item()}, roll


class TFPredictor(g.GRUPredictor):
    """M1b as a harness predictor: encode the last `warmup` observed rows, then decode with a trimmed cache."""

    def __init__(self, *a, context: int = 168, **kw):
        kw.setdefault("chunk", 512)
        super().__init__(*a, **kw)
        self.name = "m1_tf"
        self.context = context

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
        n, W = len(starts), self.warmup
        rows = starts_t[:, None] - (W - 1) + torch.arange(W, device=dev)[None]
        valid = rows >= 0
        r = rows.clamp(min=0)
        inp = torch.cat([x_state[r], cal[r], act[r], sev[r]], -1)
        mu, _, fl, kv = self.model.run(inp, valid=valid)
        cache = append(None, kv, valid, keep=self.context - 1)
        mu, fl = mu[:, -1], fl[:, -1]
        cur = state_raw[starts_t]
        sev_hold = sev[starts_t]
        ones = torch.ones(n, 1, dtype=torch.bool, device=dev)
        vs, probs, chs, des = [], [], [], []
        vidx, didx = torch.tensor(lay.v, device=dev), torch.tensor(lay.deaths, device=dev)
        for k in range(horizon):
            cur = self.stepper.next_state(cur, mu, fl)
            probs.append(torch.sigmoid(fl))
            vs.append(cur[:, vidx])
            chs.append(cur[:, lay.cold_hours])
            des.append(cur[:, didx].sum(-1))
            if k == horizon - 1:
                break
            rr = (starts_t + k + 1).clamp(max=T - 1)
            x1 = torch.cat([self.stepper.normalize(cur), cal[rr], act[rr], sev_hold], -1).unsqueeze(1)
            m1, _, f1, new_kv = self.model.run(x1, cache, ones)
            cache = append(cache, new_kv, ones, keep=self.context - 1)
            mu, fl = m1[:, 0], f1[:, 0]
        v = torch.stack(vs, 1).double().cpu().numpy()
        cold = ep.bench.cold_hours[starts][:, None] + np.cumsum(torch.stack(chs, 1).double().cpu().numpy(), axis=1)
        deaths = ep.bench.deaths[starts][:, None] + np.cumsum(torch.stack(des, 1).double().cpu().numpy(), axis=1)
        if self.round_counts:
            v, cold, deaths = np.round(v), np.round(cold), np.round(deaths)
        return ev.Rollout(v, torch.stack(probs, 1).double().cpu().numpy(), cold, deaths)


def _build(lay, cfg):
    return M1TF(lay.n_input, lay.n_cont, cfg["d_model"], cfg["layers"], cfg["heads"], cfg["context"])


def _predictor(model, lay, norm, variant, device, cfg, round_counts=False):
    return TFPredictor(model, lay, norm, variant, device, cfg["warmup"], round_counts=round_counts,
                       context=cfg["context"])


if torch is not None:
    g.KINDS["m1_tf"] = g.Kind("m1_tf", DEFAULTS, _build, losses_tf, _predictor)


def main(argv=None) -> int:
    import argparse
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("cmd", choices=["train"])
    ap.add_argument("--out", required=True)
    ap.add_argument("--variant", default=d.MAIN.name, choices=sorted(d.VARIANTS))
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--root", default=str(g.c.DEFAULT_ROOT))
    ap.add_argument("--device", default="auto")
    for k, v in DEFAULTS.items():
        ap.add_argument(f"--{k.replace('_', '-')}", dest=k, type=type(v), default=None)
    args = ap.parse_args(argv)
    if torch is None:
        raise SystemExit("torch is required")
    return g.train(args, g.KINDS["m1_tf"])


if __name__ == "__main__":
    raise SystemExit(main())
