"""WM-6 direct effect models (docs/worldmodel/wm6/GOAL.md 7, 9): nested training sets over v2 + v2x, WM-5's features,
ridge / MLP / GRU / GBM, observable and privileged, their training, the v2 test evaluation and the obs2 transfer.

**Cases** (GOAL.md 7.1): WM-2's (world, arm, replicate) cases of the v2 families (`effects_base.json`: train, val, test)
and the same cases of the v2x families (`wm2.effects.family_cases` on `v2x/fam`, replicate 0). Only the three W3 arms
(`sp240L`, `gf240E`, `rw120E`) have target cells; `sp120L` cases are never a training row of the neural / GBM models
and are predicted 0 at evaluation (their cells are never scored; the ridge keeps WM-5's code, DECISIONS L2.2).

**Training sets** (GOAL.md 4.1, 4.2): replicate 0 only; N = 400 → WM-2's train worlds 40000–40399; 800 → + 50000–50399;
1,600 → + 50000–51199; 2,000 → + 50000–51599 (`NESTED`). Validation: v2 val (40400–40479), both replicates, unchanged.
The WM-5 reproduction (`ridge --repro`) uses WM-2's train cases with both replicates and every arm, as WM-5 did.

**Targets**: per case the 8 values truth[o][lag] / std(arm, o, lag), WM-2's scales (`effects_base.json` `stds`, the
scoring scales, the same at every N); a lag past the episode's last row is masked (censoring as WM-2).

**Features** (WM-5 `effects5`): `obs` = `obs_features` of the case's `none_r<rep>` member at the cast row (145:
the 37 state features at c, their means over c − 23 … c and c − 167 … c, the 30 calendar features, the earlier-cast
terms (0 on v2)); `priv` adds `priv_features` (27 hidden values). `seq` = WM-2's 74 action-aware inputs (37 state, 30
calendar, 7 scaled actions; `wm1.data.Variant.input_names`) of rows c − 167 … c (168 rows ending at the pre-act row c);
`seqh` = the 27 hidden values at the same rows (for `seq_priv`).

**Models**: `mlp` = 3 hidden layers × width, GELU, dropout, on the normalized features + the arm one-hot → 8 outputs;
`seq` = a 2-layer GRU (hidden 128) over the normalized window (+ hidden values for `priv`), its last hidden state
concatenated with the normalized `obs` features and the arm one-hot → the same MLP head; loss = the masked mean absolute
error over the case's 8 targets. AdamW (lr 1e-3, weight decay 1e-4), batch 256, at most 400 epochs, early stopping on
the v2 val cell error (the 24-cell average of the scaled MAE, both lags; patience 30), the best epoch kept;
`--max-seconds` stops a run (stop reason recorded). Inputs are normalized by the training rows' mean / std; a feature
constant on the training rows is set to 0 (it carries no information and, as in the ridge, no weight; DECISIONS L2.3).
`gbm` = LightGBM per (arm, outcome, lag) cell (L1, 500 rounds, early stopping on val after 50 rounds without
improvement, 31 leaves, lr 0.05, min_data_in_leaf 20). `ridge` = WM-5's `fit_cells` (λ per cell on the v2 val MAE).

usage (DGX):
  effects6.py prepare --part v2|v2x|test --root ROOT --base effects_base.json --out DIR
  effects6.py train --model mlp|seq --feat obs|priv --n N --seed S [--width W --dropout P | --config-from JSON]
                    --prep DIR --base effects_base.json --max-seconds S --out RUN
  effects6.py ridge --feat obs|priv (--n N | --repro) --prep DIR --base effects_base.json --out RUN
  effects6.py gbm --feat obs|priv --n N --prep DIR --base effects_base.json --out RUN         (side venv, lightgbm)
  effects6.py choose --runs RUN... --out JSON                                                   (the N = 400 grid)
  effects6.py evaluate --run RUN --prep DIR --base effects_base.json [--obs2-base obs2_base.json --root ROOT]
"""
from __future__ import annotations

import argparse
import dataclasses
import hashlib
import json
import os
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm6  # noqa: F401
from wm_data import load_episode, load_family
from wm1 import common as c
from wm1 import data as d
from wm2 import effects as ef
from wm2 import v2
from wm5 import effects5 as e5

ARMS = v2.W3_ARMS                                   # the arms with target cells (sp240L, gf240E, rw120E)
LAGS = tuple(v2.LAGS)                               # 7d, 30d
TARGETS = [(o, lag) for lag in LAGS for o in v2.OUTCOMES]     # 8 outputs per case, lag-major
CELLS = {lag: [(a, o, lag) for a in ARMS for o in v2.OUTCOMES] for lag in LAGS}
ALL_CELLS = [cell for lag in LAGS for cell in CELLS[lag]]
V2_TRAIN = (40000, 40400)
NESTED = {400: (), 800: ((50000, 50400),), 1600: ((50000, 51200),), 2000: ((50000, 51600),)}
NS = tuple(NESTED)
SEQ_LEN = 168
SCENE = "game_v19"
# GOAL.md 7.3 / 7.5 training rule
LR, WEIGHT_DECAY, BATCH, MAX_EPOCHS, PATIENCE = 1e-3, 1e-4, 256, 400, 30
GRID = [(w, p) for w in (256, 512) for p in (0.1, 0.3)]
DEFAULT_WIDTH, DEFAULT_DROPOUT, DEPTH, GRU_HIDDEN, GRU_LAYERS = 256, 0.1, 3, 128, 2
# GOAL.md 7.4
GBM_PARAMS = {"objective": "l1", "num_leaves": 31, "learning_rate": 0.05, "min_data_in_leaf": 20, "verbose": -1,
              "seed": 0, "deterministic": True, "force_row_wise": True, "num_threads": 4}
GBM_ROUNDS, GBM_EARLY = 500, 50


# ------------------------------------------------------------------ sets --------------------------------------------
def train_seeds(n: int) -> list[int]:
    """The nested training worlds of N (GOAL.md 4.2)."""
    if n not in NESTED:
        raise ValueError(f"N {n} not in {NS}")
    out = list(range(*V2_TRAIN))
    for lo, hi in NESTED[n]:
        out += list(range(lo, hi))
    return out


def select_cases(cases: list, seeds, reps=(0,), arms=ARMS) -> list:
    keep = set(seeds)
    return [x for x in cases if x.seed in keep and x.rep in reps and (arms is None or x.arm in arms)]


def targets(cases: list, stds: dict) -> tuple[np.ndarray, np.ndarray]:
    """[n, 8] scaled truths and the mask (lag present)."""
    y, m = np.zeros((len(cases), len(TARGETS))), np.zeros((len(cases), len(TARGETS)))
    for i, x in enumerate(cases):
        for j, (o, lag) in enumerate(TARGETS):
            if lag in x.truth[o]:
                y[i, j] = x.truth[o][lag] / stds[(x.arm, o, lag)]
                m[i, j] = 1.0
    return y, m


def arm_onehot(cases: list) -> np.ndarray:
    out = np.zeros((len(cases), len(ARMS)))
    for i, x in enumerate(cases):
        out[i, ARMS.index(x.arm)] = 1.0
    return out


# ------------------------------------------------------------------ features ----------------------------------------
def hidden_rows(ep) -> tuple[np.ndarray, tuple]:
    """[T, 27] the `effects5.priv_features` values at every row (vectorized; equal to them row by row, unit test)."""
    g = [np.asarray(ep.column("global.npy", n), dtype=np.float64) for n in e5.HIDDEN_GLOBAL]
    lat = np.asarray(ep.arrays["latent.npy"], dtype=np.float64)
    cols = g + [lat[:, i] for i in range(lat.shape[1])]
    for n in e5.HIDDEN_SETTLEMENT:
        col = np.asarray(ep.column("settlements.npy", n), dtype=np.float64)
        if col.ndim == 1 or col.shape[1] == 0:
            cols += [np.zeros(col.shape[0]), np.zeros(col.shape[0])]
            continue
        ok = ~np.isnan(col)
        any_ = ok.any(axis=1)
        cols.append(np.where(any_, np.nansum(np.where(ok, col, 0.0), axis=1), 0.0))
        cols.append(np.where(any_, np.max(np.where(ok, col, -np.inf), axis=1), 0.0))
    wv = ep.world_variety() or {}
    keys = tuple(sorted(k for k, v in wv.items() if isinstance(v, (int, float))))
    t = len(cols[0])
    cols += [np.full(t, float(wv[k])) for k in keys]
    return np.stack(cols, axis=1), keys


def seq_inputs(epd: d.EpisodeData) -> np.ndarray:
    """[T, 74] WM-2's action-aware inputs (raw state, calendar, actions / ACTION_SCALE) in `input_names` order."""
    var = e5.variant_of(epd)
    return np.concatenate([epd.state(var).astype(np.float64), epd.calendar.astype(np.float64),
                           epd.bench.actions / d.ACTION_SCALE], axis=1)


def episode_rows(ep, epd: d.EpisodeData, rows: list[int], priv: bool, pre_casts=()) -> dict:
    """{row: {"obs", "priv", "seq", "seqh"}} of one member at its cast rows."""
    out = {}
    s = seq_inputs(epd)
    h = hidden_rows(ep)[0] if priv else None
    for r in rows:
        node = {"obs": e5.obs_features(epd, r, pre_casts), "seq": window(s, r)}
        if priv:
            node["priv"] = e5.priv_features(ep, r)[0]
            node["seqh"] = window(h, r)
        out[r] = node
    return out


def window(a: np.ndarray, r: int) -> np.ndarray:
    """Rows r − 167 … r of `a`; rows before 0 repeat row 0 (only obs2 reference rows < 167, DECISIONS L2.4)."""
    lo = r - SEQ_LEN + 1
    if lo >= 0:
        return a[lo: r + 1]
    return np.concatenate([np.repeat(a[:1], -lo, axis=0), a[: r + 1]])


def member_path(root: Path, kind: str, seed: int, rep: int) -> Path:
    fam = Path(root) / ("v2/v19/fam" if kind == "v2" else "v2x/fam")
    return fam / f"{SCENE}_s{seed}" / f"{SCENE}_s{seed}_none_r{rep}"


@dataclasses.dataclass
class Part:
    """Prepared cases and features of one part (`prepare`): `rows[key]` -> index into the arrays, key = seed:rep:row."""
    cases: list
    keys: list
    obs: np.ndarray
    priv: np.ndarray
    seq: np.ndarray
    seqh: np.ndarray
    meta: dict
    _pos: dict | None = dataclasses.field(default=None, repr=False)


def row_key(x) -> str:
    return f"{x.seed}:{x.rep}:{x.cast_row}"


def positions(p: Part) -> dict:
    """{row key: index} of a part, built once."""
    if p._pos is None:
        p._pos = {k: i for i, k in enumerate(p.keys)}
    return p._pos


def build_part(root: Path, kind: str, cases: list, role: str) -> Part:
    """Features of every (seed, rep, cast row) of `cases` (obs, priv, seq, seqh; priv arrays always built)."""
    by: dict = {}
    for x in cases:
        by.setdefault((x.seed, x.rep), set()).add(x.cast_row)
    keys, obs, priv, seq, seqh = [], [], [], [], []
    wv_keys = None
    for (seed, rep), rows in sorted(by.items()):
        ep = load_episode(member_path(root, kind, seed, rep), arrays=e5.ARRAYS_PRIV)
        epd = d.from_episode(ep, "v19_fam", role, seed, "none", c.V17_EXTRA)
        k = hidden_rows(ep)[1]
        if wv_keys is None:
            wv_keys = k
        elif k != wv_keys:
            raise ValueError(f"{seed}: world_variety keys differ")
        for r, node in sorted(episode_rows(ep, epd, sorted(rows), True).items()):
            keys.append(f"{seed}:{rep}:{r}")
            obs.append(node["obs"]); priv.append(node["priv"]); seq.append(node["seq"]); seqh.append(node["seqh"])
    epd0 = d.from_episode(load_episode(member_path(root, kind, cases[0].seed, cases[0].rep), arrays=e5.ARRAYS),
                          "v19_fam", role, cases[0].seed, "none", c.V17_EXTRA)
    var = e5.variant_of(epd0)
    names = e5.obs_feature_names(var.state_names, tuple(var.input_names()[len(var.state_names): len(var.state_names) + 30]))
    meta = {"kind": kind, "role": role, "obs_names": names, "priv_names": e5.priv_feature_names(wv_keys),
            "seq_names": list(var.input_names()), "n_rows": len(keys), "n_cases": len(cases)}
    return Part(cases, keys, np.array(obs), np.array(priv), np.array(seq, dtype=np.float32),
                np.array(seqh, dtype=np.float32), meta)


def save_part(part: Part, out: Path, name: str) -> None:
    out.mkdir(parents=True, exist_ok=True)
    np.savez(out / f"{name}.npz", obs=part.obs, priv=part.priv, seq=part.seq, seqh=part.seqh)
    c.write_json(out / f"{name}.json", {"meta": part.meta, "keys": part.keys,
                                         "cases": [ef.case_json(x) for x in part.cases]})


def load_part(prep: Path, name: str) -> Part:
    j = json.loads((Path(prep) / f"{name}.json").read_text())
    z = np.load(Path(prep) / f"{name}.npz")
    return Part([ef.case_from(x) for x in j["cases"]], j["keys"], z["obs"], z["priv"], z["seq"], z["seqh"], j["meta"])


def v2x_cases(root: Path) -> tuple[list, list]:
    """Every case of the v2x families (replicate 0) and the per-world member lists."""
    cases, worlds = [], []
    fam = Path(root) / "v2x" / "fam"
    for p in sorted(fam.iterdir()):
        if not p.is_dir():
            continue
        f = load_family(p, verify=True, arrays=ef.ARRAYS)
        if f.replicates != 1:
            raise ValueError(f"{p}: R = {f.replicates}, expected 1")
        cases += ef.family_cases(f, "train")
        worlds.append({"seed": f.seed, "arms": sorted(f.variants)})
    return cases, worlds


# ------------------------------------------------------------------ data for a run ----------------------------------
@dataclasses.dataclass
class Data:
    cases: list
    x: np.ndarray            # [n, F] features (obs or obs + priv), raw
    seq: np.ndarray | None   # [n, 168, D] raw window
    arm: np.ndarray          # [n, 3]
    y: np.ndarray
    m: np.ndarray


def gather(parts: list[Part], cases: list, feat: str, stds: dict, with_seq: bool) -> Data:
    xs, ss = [], []
    for x in cases:
        for p in parts:
            i = positions(p).get(row_key(x))
            if i is not None:
                break
        else:
            raise KeyError(f"no features for {row_key(x)}")
        xs.append(np.concatenate([p.obs[i], p.priv[i]]) if feat == "priv" else p.obs[i])
        if with_seq:
            ss.append(np.concatenate([p.seq[i], p.seqh[i]], axis=1) if feat == "priv" else p.seq[i])
    y, m = targets(cases, stds)
    return Data(cases, np.array(xs), np.array(ss, dtype=np.float32) if with_seq else None, arm_onehot(cases), y, m)


def norm_of(a: np.ndarray, axis) -> dict:
    mean, std = a.mean(axis=axis), a.std(axis=axis)
    keep = std > 1e-12
    return {"mean": mean, "std": np.where(keep, std, 1.0), "keep": keep.astype(np.float64)}


def apply_norm(a: np.ndarray, nm: dict) -> np.ndarray:
    return ((a - nm["mean"]) / nm["std"]) * nm["keep"]


def load_sets(prep: Path, base: dict, n: int, feat: str, with_seq: bool) -> tuple[Data, Data, dict]:
    stds = ef.stds_from(base["stds"])
    v2p = load_part(prep, "v2")
    parts = [v2p]
    tr = select_cases(v2p.cases, train_seeds(n))
    if n != 400:
        xp = load_part(prep, "v2x")
        parts.append(xp)
        tr += select_cases(xp.cases, train_seeds(n))
    va = [x for x in v2p.cases if x.role == "val" and x.arm in ARMS]
    return gather(parts, tr, feat, stds, with_seq), gather(parts, va, feat, stds, with_seq), stds


# ------------------------------------------------------------------ scoring -----------------------------------------
def to_predictions(cases: list, out: np.ndarray, stds: dict) -> dict:
    """{case key: {"effects": {o: {lag: raw}}, "shift": 0}} from scaled outputs [n, 8]; non-W3 cases → 0."""
    preds = {}
    for i, x in enumerate(cases):
        eff = {o: {} for o in v2.OUTCOMES}
        for j, (o, lag) in enumerate(TARGETS):
            if lag in x.truth[o]:
                eff[o][lag] = float(out[i, j] * stds[(x.arm, o, lag)]) if x.arm in ARMS else 0.0
        preds[x.key] = {"effects": eff, "shift": 0.0}
    return preds


def cell_scores(cases: list, preds: dict, stds: dict) -> dict:
    """The v2 val / test cell averages per lag and both lags (`wm2.effects.cell_stat`, every world)."""
    errs = ef.cell_errors(cases, preds, stds)
    worlds = sorted({x.seed for x in cases})
    out = {lag: ef.cell_stat(errs, CELLS[lag], worlds) for lag in LAGS}
    out["both"] = float(np.mean([out[lag] for lag in LAGS]))
    return out


# ------------------------------------------------------------------ neural models -----------------------------------
def build_net(kind: str, n_x: int, n_seq: int, width: int, dropout: float):
    import torch
    from torch import nn

    def head(n_in):
        layers, w = [], n_in
        for _ in range(DEPTH):
            layers += [nn.Linear(w, width), nn.GELU(), nn.Dropout(dropout)]
            w = width
        layers.append(nn.Linear(w, len(TARGETS)))
        return nn.Sequential(*layers)

    class MLP(nn.Module):
        def __init__(self):
            super().__init__()
            self.head = head(n_x + len(ARMS))

        def forward(self, x, arm, seq=None):
            return self.head(torch.cat([x, arm], 1))

    class Seq(nn.Module):
        def __init__(self):
            super().__init__()
            self.gru = nn.GRU(n_seq, GRU_HIDDEN, num_layers=GRU_LAYERS, batch_first=True)
            self.head = head(GRU_HIDDEN + n_x + len(ARMS))

        def forward(self, x, arm, seq=None):
            _, h = self.gru(seq)
            return self.head(torch.cat([h[-1], x, arm], 1))

    return MLP() if kind == "mlp" else Seq()


def masked_l1(pred, y, m):
    return ((pred - y).abs() * m).sum() / m.sum().clamp_min(1.0)


def predict_net(net, x, arm, seq, device, batch: int = 1024) -> np.ndarray:
    import torch
    net.eval()
    outs = []
    with torch.no_grad():
        for i in range(0, len(x), batch):
            sl = slice(i, i + batch)
            s = None if seq is None else torch.as_tensor(seq[sl], device=device)
            outs.append(net(torch.as_tensor(x[sl], device=device), torch.as_tensor(arm[sl], device=device), s).cpu().numpy())
    return np.concatenate(outs).astype(np.float64)


def sha(path: Path) -> str:
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run_meta(a, extra: dict) -> dict:
    import torch
    prep = Path(a.prep)
    return {"code_commit": c.git_commit(), "argv": sys.argv, "host": os.uname().nodename, "torch": torch.__version__,
            "numpy": np.__version__, "prep_sha256": {p.name: sha(p) for p in sorted(prep.glob("*.json"))},
            "base_sha256": sha(Path(a.base)), "v2_manifest_sha256": sha(c.REPO / "docs/worldmodel/wm2/v2_manifest.json"),
            "v2x_manifest_sha256": (sha(c.REPO / "docs/worldmodel/wm6/v2x_manifest.json")
                                    if (c.REPO / "docs/worldmodel/wm6/v2x_manifest.json").exists() else None)} | extra


def cmd_train(a) -> int:
    import torch
    t0 = time.time()
    out = Path(a.out)
    if (out / "model.pt").exists():
        raise SystemExit(f"{out}/model.pt exists")
    out.mkdir(parents=True, exist_ok=True)
    width, dropout = a.width, a.dropout
    if a.config_from:
        ch = json.loads(Path(a.config_from).read_text())[f"mlp_{a.feat}"]
        width, dropout = ch["width"], ch["dropout"]
    base = json.loads(Path(a.base).read_text())
    seq = a.model == "seq"
    tr, va, stds = load_sets(Path(a.prep), base, a.n, a.feat, seq)
    nx = norm_of(tr.x, 0)
    ns = norm_of(tr.seq.reshape(-1, tr.seq.shape[-1]).astype(np.float64), 0) if seq else None
    dev = torch.device("cuda" if torch.cuda.is_available() and a.device != "cpu" else "cpu")
    torch.manual_seed(a.seed)
    np.random.seed(a.seed)
    T = lambda z: torch.as_tensor(np.asarray(z, dtype=np.float32), device=dev)
    xtr, xva = T(apply_norm(tr.x, nx)), apply_norm(va.x, nx).astype(np.float32)
    str_ = T(apply_norm(tr.seq, ns)) if seq else None
    sva = apply_norm(va.seq, ns).astype(np.float32) if seq else None
    atr, ytr, mtr = T(tr.arm), T(tr.y), T(tr.m)
    net = build_net(a.model, tr.x.shape[1], tr.seq.shape[-1] if seq else 0, width, dropout).to(dev)
    opt = torch.optim.AdamW(net.parameters(), lr=LR, weight_decay=WEIGHT_DECAY)
    gen = torch.Generator(device="cpu").manual_seed(a.seed)
    config = {"model": a.model, "feat": a.feat, "name": f"{a.model}_{a.feat}", "n": a.n, "seed": a.seed,
              "width": width, "dropout": dropout, "depth": DEPTH, "gru_hidden": GRU_HIDDEN if seq else None,
              "gru_layers": GRU_LAYERS if seq else None, "seq_len": SEQ_LEN if seq else None, "lr": LR,
              "weight_decay": WEIGHT_DECAY, "batch": BATCH, "max_epochs": MAX_EPOCHS, "patience": PATIENCE,
              "max_seconds": a.max_seconds, "device": str(dev), "n_train_cases": len(tr.cases),
              "n_train_worlds": len({x.seed for x in tr.cases}), "train_reps": sorted({x.rep for x in tr.cases}),
              "n_val_cases": len(va.cases), "n_features": int(tr.x.shape[1]),
              "n_seq_inputs": int(tr.seq.shape[-1]) if seq else None,
              "constant_features_zeroed": int((nx["keep"] == 0).sum()),
              "diagnostic": a.feat == "priv", "config_from": a.config_from,
              "n_params": int(sum(p.numel() for p in net.parameters()))}
    config |= run_meta(a, {})
    c.write_json(out / "config.json", config)
    log = open(out / "train_log.jsonl", "w")
    best, best_ep, best_state, stop = np.inf, -1, None, "max_epochs"
    n = len(tr.cases)
    for ep in range(MAX_EPOCHS):
        net.train()
        perm = torch.randperm(n, generator=gen).to(dev)
        tot, cnt = 0.0, 0
        for i in range(0, n, BATCH):
            idx = perm[i: i + BATCH]
            pred = net(xtr[idx], atr[idx], str_[idx] if seq else None)
            loss = masked_l1(pred, ytr[idx], mtr[idx])
            opt.zero_grad()
            loss.backward()
            opt.step()
            tot += loss.item() * len(idx)
            cnt += len(idx)
        vp = to_predictions(va.cases, predict_net(net, xva, va.arm.astype(np.float32), sva, dev), stds)
        vs = cell_scores(va.cases, vp, stds)
        improved = vs["both"] < best
        if improved:
            best, best_ep = vs["both"], ep
            best_state = {k: v.detach().cpu().clone() for k, v in net.state_dict().items()}
        log.write(json.dumps({"epoch": ep, "train_loss": tot / cnt, "val": vs, "best": improved,
                              "seconds": time.time() - t0}) + "\n")
        log.flush()
        if ep - best_ep >= PATIENCE:
            stop = "patience"
            break
        if time.time() - t0 > a.max_seconds:
            stop = "max_seconds"
            break
    log.close()
    net.load_state_dict(best_state)
    torch.save({"state": best_state, "config": config, "norm_x": {k: v.tolist() for k, v in nx.items()},
                "norm_seq": None if ns is None else {k: v.tolist() for k, v in ns.items()}}, out / "model.pt")
    vp = to_predictions(va.cases, predict_net(net, xva, va.arm.astype(np.float32), sva, dev), stds)
    c.write_json(out / "val_predictions.json", vp)
    c.write_json(out / "val_metrics.json", {"stop": stop, "best_epoch": best_ep, "epochs": ep + 1, "val": cell_scores(va.cases, vp, stds),
                                            "best_val_both": best, "seconds": time.time() - t0,
                                            "model_sha256": sha(out / "model.pt")})
    print(f"{a.model}_{a.feat} n{a.n} s{a.seed}: stop {stop} best epoch {best_ep} val {best:.4f} {time.time() - t0:.0f}s")
    return 0


def load_net(run: Path, device: str = "auto"):
    import torch
    blob = torch.load(Path(run) / "model.pt", map_location="cpu", weights_only=False)
    cfg = blob["config"]
    dev = torch.device("cuda" if torch.cuda.is_available() and device != "cpu" else "cpu")
    net = build_net(cfg["model"], cfg["n_features"], cfg["n_seq_inputs"] or 0, cfg["width"], cfg["dropout"])
    net.load_state_dict(blob["state"])
    net.to(dev)
    arr = lambda nm: None if nm is None else {k: np.array(v) for k, v in nm.items()}
    return net, cfg, arr(blob["norm_x"]), arr(blob["norm_seq"]), dev


# ------------------------------------------------------------------ ridge / gbm -------------------------------------
def ridge_fits_json(fits: dict) -> dict:
    return {"|".join(k): {"lambda": v["lambda"], "val_mae": v["val_mae"], "scale": v["scale"], "n_train": v["n_train"],
                          "n_val": v["n_val"], "w": v["model"].w.tolist(), "b": float(v["model"].b),
                          "mean": v["model"].mean.tolist(), "std": v["model"].std.tolist()} for k, v in fits.items()}


def ridge_from_json(j: dict) -> dict:
    out = {}
    for k, v in j.items():
        r = e5.Ridge(v["lambda"])
        r.w, r.b, r.mean, r.std = np.array(v["w"]), v["b"], np.array(v["mean"]), np.array(v["std"])
        out[tuple(k.split("|"))] = {"model": r, "scale": v["scale"], "lambda": v["lambda"]}
    return out


def feature_dict(parts: list[Part], cases: list, feat: str) -> dict:
    out = {}
    for x in cases:
        for p in parts:
            i = positions(p).get(row_key(x))
            if i is not None:
                out[x.key] = np.concatenate([p.obs[i], p.priv[i]]) if feat == "priv" else p.obs[i]
                break
        else:
            raise KeyError(row_key(x))
    return out


def cmd_ridge(a) -> int:
    """WM-5's ridge at N (replicate 0, every arm's case in the normalization as WM-5) or, with --repro, WM-5's own fit
    (both replicates of WM-2's train worlds); λ on v2 val; val predictions compared with WM-5's committed file."""
    t0 = time.time()
    out = Path(a.out)
    if (out / "fits.json").exists():
        raise SystemExit(f"{out}/fits.json exists")
    base = json.loads(Path(a.base).read_text())
    stds = ef.stds_from(base["stds"])
    v2p = load_part(Path(a.prep), "v2")
    parts = [v2p]
    if a.repro:
        tr = [x for x in v2p.cases if x.role == "train"]
        n = 400
    else:
        n = a.n
        tr = select_cases(v2p.cases, train_seeds(n), arms=None)
        if n != 400:
            xp = load_part(Path(a.prep), "v2x")
            parts.append(xp)
            tr += select_cases(xp.cases, train_seeds(n), arms=None)
    va = [x for x in v2p.cases if x.role == "val"]
    Xtr, Xva = feature_dict(parts, tr, a.feat), feature_dict(parts, va, a.feat)
    fits = e5.fit_cells(tr, va, Xtr, Xva, stds, ALL_CELLS)
    vp = e5.ridge_predict(fits, va, Xva)
    name = f"ridge_{a.feat}"
    res = {"name": name, "feat": a.feat, "n": n, "repro": bool(a.repro), "diagnostic": a.feat == "priv",
           "n_train_cases": len(tr), "n_train_worlds": len({x.seed for x in tr}), "train_reps": sorted({x.rep for x in tr}),
           "lambdas": {"|".join(k): v["lambda"] for k, v in fits.items()},
           "coef_norm": {"|".join(k): float(np.linalg.norm(v["model"].w)) for k, v in fits.items()},
           "val": cell_scores([x for x in va if x.arm in ARMS], vp, stds), "seconds": time.time() - t0}
    if a.repro:
        wm5 = json.loads((c.REPO / f"docs/worldmodel/wm5/runs/{name}/{name}.json").read_text())
        res["wm5_check"] = compare_preds(vp, wm5["v2_val_predictions"]) | {
            "lambdas_equal": res["lambdas"] == wm5["lambdas"],
            "coef_norm_max_abs_diff": max(abs(res["coef_norm"][k] - wm5["coef_norm"][k]) for k in wm5["coef_norm"])}
    out.mkdir(parents=True, exist_ok=True)
    c.write_json(out / "config.json", {"model": "ridge", "feat": a.feat, "name": name, "n": n, "repro": bool(a.repro),
                                       "diagnostic": a.feat == "priv", "lambdas_grid": list(e5.LAMBDAS)} | run_meta_np(a))
    c.write_json(out / "fits.json", ridge_fits_json(fits))
    c.write_json(out / "val_predictions.json", vp)
    c.write_json(out / "val_metrics.json", res | {"stop": "closed_form"})
    print(json.dumps({k: v for k, v in res.items() if k not in ("coef_norm", "lambdas")}))
    return 0


def run_meta_np(a) -> dict:
    prep = Path(a.prep)
    return {"code_commit": c.git_commit(), "argv": sys.argv, "host": os.uname().nodename, "numpy": np.__version__,
            "prep_sha256": {p.name: sha(p) for p in sorted(prep.glob("*.json"))}, "base_sha256": sha(Path(a.base))}


def compare_preds(a: dict, b: dict) -> dict:
    diffs = [abs(a[k]["effects"][o][lag] - b[k]["effects"][o][lag]) for k in b for o in b[k]["effects"]
             for lag in b[k]["effects"][o]]
    return {"keys_equal": sorted(a) == sorted(b), "n_values": len(diffs), "max_abs_diff": max(diffs) if diffs else None}


def cmd_gbm(a) -> int:
    import lightgbm as lgb
    t0 = time.time()
    out = Path(a.out)
    if (out / "fits.json").exists():
        raise SystemExit(f"{out}/fits.json exists")
    base = json.loads(Path(a.base).read_text())
    tr, va, stds = load_sets(Path(a.prep), base, a.n, a.feat, False)
    out.mkdir(parents=True, exist_ok=True)
    fits, info = {}, {}
    for arm in ARMS:
        it = np.array([x.arm == arm for x in tr.cases])
        iv = np.array([x.arm == arm for x in va.cases])
        for j, (o, lag) in enumerate(TARGETS):
            mt, mv = it & (tr.m[:, j] > 0), iv & (va.m[:, j] > 0)
            dtr = lgb.Dataset(tr.x[mt], tr.y[mt, j], free_raw_data=False)
            dva = lgb.Dataset(va.x[mv], va.y[mv, j], reference=dtr, free_raw_data=False)
            bst = lgb.train(GBM_PARAMS, dtr, num_boost_round=GBM_ROUNDS, valid_sets=[dva],
                            callbacks=[lgb.early_stopping(GBM_EARLY, verbose=False)])
            key = f"{arm}|{o}|{lag}"
            bst.save_model(str(out / f"gbm_{key.replace('|', '_')}.txt"), num_iteration=bst.best_iteration)
            fits[key] = f"gbm_{key.replace('|', '_')}.txt"
            info[key] = {"best_iteration": int(bst.best_iteration), "n_train": int(mt.sum()), "n_val": int(mv.sum())}
    vp = gbm_predict(out, fits, va)
    res = {"name": f"gbm_{a.feat}", "n": a.n, "info": info, "val": cell_scores(va.cases, to_predictions(va.cases, vp, stds), stds),
           "seconds": time.time() - t0, "stop": "early_stopping_per_cell", "lightgbm": lgb.__version__}
    c.write_json(out / "config.json", {"model": "gbm", "feat": a.feat, "name": f"gbm_{a.feat}", "n": a.n,
                                       "diagnostic": a.feat == "priv", "params": GBM_PARAMS, "rounds": GBM_ROUNDS,
                                       "early_stopping_rounds": GBM_EARLY, "n_train_cases": len(tr.cases),
                                       "lightgbm": lgb.__version__, "python": sys.executable} | run_meta_np(a))
    c.write_json(out / "fits.json", fits)
    c.write_json(out / "val_predictions.json", to_predictions(va.cases, vp, stds))
    c.write_json(out / "val_metrics.json", res)
    print(json.dumps({k: v for k, v in res.items() if k != "info"}))
    return 0


def gbm_predict(run: Path, fits: dict, data: Data) -> np.ndarray:
    import lightgbm as lgb
    out = np.zeros((len(data.cases), len(TARGETS)))
    arms = np.array([x.arm for x in data.cases])
    for key, fname in fits.items():
        arm, o, lag = key.split("|")
        j = TARGETS.index((o, lag))
        sel = arms == arm
        if sel.any():
            out[sel, j] = lgb.Booster(model_file=str(Path(run) / fname)).predict(data.x[sel])
    return out


def cmd_choose(a) -> int:
    """The N = 400 grid (GOAL.md 7.3): per feature set, the (width, dropout) with the lowest best val score."""
    rows = []
    for r in a.runs:
        cfg = json.loads((Path(r) / "config.json").read_text())
        val = json.loads((Path(r) / "val_metrics.json").read_text())
        rows.append({"run": r, "feat": cfg["feat"], "width": cfg["width"], "dropout": cfg["dropout"],
                     "val_both": val["val"]["both"], "val_7d": val["val"]["7d"], "val_30d": val["val"]["30d"]})
    out = {"rows": rows, "rule": "lowest v2 val 24-cell average (both lags) of the seed-0 run at N = 400; ties: the "
                                 "smaller width, then the smaller dropout"}
    for feat in ("obs", "priv"):
        rs = [x for x in rows if x["feat"] == feat]
        if rs:
            best = min(rs, key=lambda x: (x["val_both"], x["width"], x["dropout"]))
            out[f"mlp_{feat}"] = {"width": best["width"], "dropout": best["dropout"], "val_both": best["val_both"]}
    c.write_json(Path(a.out), out)
    print(json.dumps(out))
    return 0


# ------------------------------------------------------------------ prepare / evaluate -------------------------------
def cmd_prepare(a) -> int:
    t0 = time.time()
    out = Path(a.out)
    base = json.loads(Path(a.base).read_text())
    if (out / f"{a.part}.npz").exists():
        raise SystemExit(f"{out}/{a.part}.npz exists")
    if a.part == "v2":
        cases = [ef.case_from(x) for r in ("train", "val") for x in base["cases"][r]]
        part = build_part(Path(a.root), "v2", cases, "train")
    elif a.part == "test":
        cases = [ef.case_from(x) for x in base["cases"]["test"]]
        part = build_part(Path(a.root), "v2", cases, "test")
    else:
        cases, worlds = v2x_cases(Path(a.root))
        part = build_part(Path(a.root), "v2x", cases, "train")
        part.meta["worlds"] = len(worlds)
        part.meta["arms_per_world"] = {k: sum(1 for w in worlds if k in w["arms"]) for k in v2.ARMS}
    part.meta |= {"seconds": time.time() - t0, "code_commit": c.git_commit()}
    save_part(part, out, a.part)
    print(json.dumps({k: v for k, v in part.meta.items() if not k.endswith("_names")}))
    return 0


def predict_run(run: Path, data_by_feat, stds: dict, cases: list) -> np.ndarray | dict:
    cfg = json.loads((Path(run) / "config.json").read_text())
    if cfg["model"] == "ridge":
        fits = ridge_from_json(json.loads((Path(run) / "fits.json").read_text()))
        X = data_by_feat("dict", cfg["feat"], False)
        return e5.ridge_predict(fits, cases, X)
    if cfg["model"] == "gbm":
        data = data_by_feat("data", cfg["feat"], False)
        return to_predictions(cases, gbm_predict(run, json.loads((Path(run) / "fits.json").read_text()), data), stds)
    net, cfg, nx, ns, dev = load_net(run)
    data = data_by_feat("data", cfg["feat"], cfg["model"] == "seq")
    x = apply_norm(data.x, nx).astype(np.float32)
    s = apply_norm(data.seq, ns).astype(np.float32) if cfg["model"] == "seq" else None
    return to_predictions(cases, predict_net(net, x, data.arm.astype(np.float32), s, dev), stds)


def cmd_evaluate(a) -> int:
    """One run's v2 test predictions (every test case of both replicates; GOAL.md 9.2) or, with --obs2-base, its obs2
    test r0 predictions (9.3, WM-5's cases and features, the earlier-cast terms included). Computed once."""
    t0 = time.time()
    run = Path(a.run)
    target = run / ("effects_obs2.json" if a.obs2_base else "effects.json")
    if target.exists():
        raise SystemExit(f"{target} exists: computed once")
    base = json.loads(Path(a.base).read_text())
    stds = ef.stds_from(base["stds"])
    if a.obs2_base:
        _, o2test, _, _ = e5.load_obs2_base(Path(a.obs2_base))
        cases = [x for x in o2test if e5.ref_of(x.rep)[1] == 0]
        part = obs2_part(Path(a.root), cases)
    else:
        part = load_part(Path(a.prep), "test")
        cases = part.cases
    cache: dict = {}

    def data_by_feat(kind, feat, seq):
        key = (kind, feat, seq)
        if key not in cache:
            cache[key] = feature_dict([part], cases, feat) if kind == "dict" else gather_any(part, cases, feat, stds, seq)
        return cache[key]

    preds = predict_run(run, data_by_feat, stds, cases)
    res = {"run": str(run), "split": "obs2_test_r0" if a.obs2_base else "v2_test", "n_cases": len(cases),
           "code_commit": c.git_commit(), "argv": sys.argv, "predictions": preds,
           "base_sha256": sha(Path(a.base)), "seconds": time.time() - t0}
    if a.obs2_base:
        res["obs2_base_sha256"] = sha(Path(a.obs2_base))
    else:
        res["test_part_sha256"] = sha(Path(a.prep) / "test.json")
    if target.exists():
        raise SystemExit(f"{target} exists")
    c.write_json(target, res)
    print(f"evaluate {run.name} {res['split']} {time.time() - t0:.0f}s")
    return 0


def gather_any(part: Part, cases: list, feat: str, stds: dict, seq: bool) -> Data:
    """`gather` for evaluation: every case (any arm; non-W3 rows predicted and then set to 0) of one part."""
    xs, ss = [], []
    pos = positions(part)
    for x in cases:
        i = pos[row_key(x)]
        xs.append(np.concatenate([part.obs[i], part.priv[i]]) if feat == "priv" else part.obs[i])
        if seq:
            ss.append(np.concatenate([part.seq[i], part.seqh[i]], axis=1) if feat == "priv" else part.seq[i])
    arm = np.zeros((len(cases), len(ARMS)))
    for i, x in enumerate(cases):
        if x.arm in ARMS:
            arm[i, ARMS.index(x.arm)] = 1.0
    y = np.zeros((len(cases), len(TARGETS)))
    return Data(cases, np.array(xs), np.array(ss, dtype=np.float32) if seq else None, arm, y, y.copy())


def obs2_part(root: Path, cases: list) -> Part:
    """Features of the obs2 test r0 cases: the `pre` member of (ref, replicate) at c (WM-5 `obs2_features`, the
    earlier-cast terms included) and its 168-row window (WM-5 obs2 roles: test families read only in evaluation)."""
    wfs = {w.seed: w for w in e5.load_world_fams(root, "test", e5.ARRAYS_PRIV, worlds=sorted({x.seed for x in cases}))}
    keys, obs, priv, seq, seqh, done = [], [], [], [], [], {}
    for x in cases:
        k = row_key(x)
        if k in done:
            continue
        ref, rep = e5.ref_of(x.rep)
        wf = wfs[x.seed]
        ep = wf.members[ref]["pre"][rep]
        epd = d.from_episode(ep, "obs2_fam", "test", x.seed, "none", c.V17_EXTRA)
        node = episode_rows(ep, epd, [x.cast_row], True, wf.pre_casts[ref])[x.cast_row]
        done[k] = True
        keys.append(k); obs.append(node["obs"]); priv.append(node["priv"]); seq.append(node["seq"]); seqh.append(node["seqh"])
    return Part(cases, keys, np.array(obs), np.array(priv), np.array(seq, dtype=np.float32), np.array(seqh, dtype=np.float32),
                {"kind": "obs2", "role": "test"})


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("prepare")
    p.add_argument("--part", required=True, choices=["v2", "v2x", "test"])
    p.add_argument("--root", default=str(c.DEFAULT_ROOT)); p.add_argument("--base", required=True); p.add_argument("--out", required=True)
    p = sub.add_parser("train")
    p.add_argument("--model", required=True, choices=["mlp", "seq"]); p.add_argument("--feat", required=True, choices=["obs", "priv"])
    p.add_argument("--n", type=int, required=True); p.add_argument("--seed", type=int, required=True)
    p.add_argument("--width", type=int, default=DEFAULT_WIDTH); p.add_argument("--dropout", type=float, default=DEFAULT_DROPOUT)
    p.add_argument("--config-from", dest="config_from", default=None)
    p.add_argument("--prep", required=True); p.add_argument("--base", required=True)
    p.add_argument("--max-seconds", dest="max_seconds", type=float, default=3600); p.add_argument("--device", default="auto")
    p.add_argument("--out", required=True)
    p = sub.add_parser("ridge")
    p.add_argument("--feat", required=True, choices=["obs", "priv"]); p.add_argument("--n", type=int, default=400)
    p.add_argument("--repro", action="store_true"); p.add_argument("--prep", required=True); p.add_argument("--base", required=True)
    p.add_argument("--out", required=True)
    p = sub.add_parser("gbm")
    p.add_argument("--feat", required=True, choices=["obs", "priv"]); p.add_argument("--n", type=int, required=True)
    p.add_argument("--prep", required=True); p.add_argument("--base", required=True); p.add_argument("--out", required=True)
    p = sub.add_parser("choose")
    p.add_argument("--runs", nargs="+", required=True); p.add_argument("--out", required=True)
    p = sub.add_parser("evaluate")
    p.add_argument("--run", required=True); p.add_argument("--prep", required=True); p.add_argument("--base", required=True)
    p.add_argument("--obs2-base", dest="obs2_base", default=None); p.add_argument("--root", default=str(c.DEFAULT_ROOT))
    a = ap.parse_args(argv)
    return {"prepare": cmd_prepare, "train": cmd_train, "ridge": cmd_ridge, "gbm": cmd_gbm, "choose": cmd_choose,
            "evaluate": cmd_evaluate}[a.cmd](a)


if __name__ == "__main__":
    raise SystemExit(main())
