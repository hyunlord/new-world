"""M1b-act with a split-hazard head (docs/worldmodel/wm3/GOAL.md 7.1).

`M1TFH` is `wm1.m1_tf.M1TF` plus `haz = nn.Linear(d_model, 1)` on the same final layer-normed state as the other heads:
`run_h(x, cache, valid)` returns (mu, logvar, fire_logit, haz_logit, kv); `run` returns `run_h` without `haz_logit`, so
every WM-1 / WM-2 caller (`TFPredictor`, `wm2.rollout_ext`, `m1_tf.losses_tf`) works unchanged and, with a WM-2 state
dict loaded, gives bit-identical mu, logvar and fire.

Importing this module registers the kind `m1_tfh` in the module object `wm1.m1_gru` (`g.KINDS`), so `g.load_predictor`
builds a WM-3 run directory (`config.json` `model: "m1_tfh"`); every WM-3 evaluation script imports it before
`g.load_predictor`.
"""
from __future__ import annotations

import wm3  # noqa: F401
from wm1 import m1_gru as g
from wm1 import m1_tf

torch = g.torch
HAZ_KEYS = ("haz.weight", "haz.bias")
KIND = "m1_tfh"

if torch is not None:
    import torch.nn as nn

    class M1TFH(m1_tf.M1TF):
        def __init__(self, n_in: int, n_cont: int, dm: int = 256, layers: int = 4, heads: int = 4, context: int = 168,
                     seed: int | None = None):
            super().__init__(n_in, n_cont, dm, layers, heads, context)
            if seed is not None:
                torch.manual_seed(seed)
            self.haz = nn.Linear(dm, 1)

        def run_h(self, x, cache=None, valid=None):
            """`M1TF.run`'s arithmetic, plus the hazard logit [B, L] from the same final state."""
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
            return (self.mu(z), self.logvar(z).clamp(-12.0, 8.0), self.fire(z).squeeze(-1), self.haz(z).squeeze(-1),
                    kv)

        def run(self, x, cache=None, valid=None):
            mu, logvar, fl, _, kv = self.run_h(x, cache, valid)
            return mu, logvar, fl, kv


def build(lay, cfg: dict):
    """The network of a WM-3 run (the hazard layer seeded by `haz_seed` when the config carries one)."""
    return M1TFH(lay.n_input, lay.n_cont, cfg["d_model"], cfg["layers"], cfg["heads"], cfg["context"],
                 seed=cfg.get("haz_seed"))


def load_init(model, state: dict) -> None:
    """Load a WM-2 `m1_tf` state dict into an `M1TFH`: strict except the hazard layer, which must be the only missing
    key set (raises otherwise)."""
    res = model.load_state_dict(state, strict=False)
    if sorted(res.missing_keys) != sorted(HAZ_KEYS) or res.unexpected_keys:
        raise ValueError(f"init state dict: missing {res.missing_keys}, unexpected {res.unexpected_keys}")


def _loss(*a, **kw):
    from wm3 import losses
    return losses.kind_loss(*a, **kw)


if torch is not None:
    g.KINDS[KIND] = g.Kind(KIND, m1_tf.DEFAULTS, build, _loss, m1_tf._predictor)
