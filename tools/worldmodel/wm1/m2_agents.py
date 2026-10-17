"""M2: agent-level next-hour model (GOAL.md 10), evaluated on T5 (GOAL.md 7.6).

Per agent and row: its own observable features (never `lifespan_days`) and their change over the last hour, its
settlement row (the severity-blind M1 state, train-normalized), the calendar, the row's actions, and the mean and
max of its 8 nearest living agents' features. Two architectures, chosen by validation: an MLP over that vector, or a
small set transformer over all living agents of the row (2 layers, d 128). Heads: next state tag, next target kind
(cross-entropy), position change dx, dy as classes -3..3 (clipped), next hunger / thirst / fatigue change
(regression, train-std-normalized).

usage:
  python3 tools/worldmodel/wm1/m2_agents.py train --out RUN_DIR [--arch mlp|set] [--seed 0]
  python3 tools/worldmodel/wm1/m2_agents.py final --run RUN_DIR
"""
from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm1  # noqa: F401
import bench_v0 as b
from wm1 import agents as A
from wm1 import common as c
from wm1 import data as d

try:
    import torch
    import torch.nn as nn
    import torch.nn.functional as F
except ImportError:
    torch = None

K_NEAR = 8
DMAX = 3
DEFAULTS = dict(hidden=256, d_model=128, lr=1e-3, weight_decay=1e-4, batch=4096, row_batch=96, epochs=4,
                max_seconds=3600, threads=16, eval_every=1000)
F_ = A.AGENT_FIELDS.index


def onehot(x: np.ndarray, n: int) -> np.ndarray:
    x = np.nan_to_num(x, nan=0).astype(int).clip(0, n - 1)
    return np.eye(n)[x]


def own_features(ep: A.AgentEpisode) -> np.ndarray:
    """[T, N, F_own] observable per-agent features at each row (zeros where not alive)."""
    a = np.nan_to_num(ep.agents, nan=0.0)
    x, y = a[..., F_("x")], a[..., F_("y")]
    tx, ty = a[..., F_("target_x")], a[..., F_("target_y")]
    has_t = (tx >= 0) & (ty >= 0)
    prev = np.concatenate([a[:1], a[:-1]], axis=0)
    needs = a[..., [F_(k) for k in A.NEEDS]]
    prev_needs = prev[..., [F_(k) for k in A.NEEDS]]
    parts = [x[..., None] / ep.width, y[..., None] / ep.height,
             onehot(a[..., F_("state_tag")], 4), onehot(a[..., F_("seek_kind")], 4), onehot(a[..., F_("target_kind")], 8),
             has_t[..., None].astype(float), (np.where(has_t, tx - x, 0) / 10).clip(-3, 3)[..., None],
             (np.where(has_t, ty - y, 0) / 10).clip(-3, 3)[..., None], needs / 100.0,
             a[..., [F_("carried_food"), F_("carried_wood")]] / 10.0, onehot(a[..., F_("life_stage")], 3),
             a[..., [F_("cold")]], (a[..., [F_("settlement")]] >= 0).astype(float), a[..., [F_("age_days")]] / 100.0,
             ((x - prev[..., F_("x")]).clip(-3, 3))[..., None], ((y - prev[..., F_("y")]).clip(-3, 3))[..., None],
             (needs - prev_needs) / 10.0, onehot(prev[..., F_("state_tag")], 4), onehot(prev[..., F_("target_kind")], 8)]
    out = np.concatenate(parts, axis=-1)
    return out * ep.alive[..., None]


NEAR_COLS = slice(2, 18)       # state tag, seek kind, target kind one-hots of own_features
NEEDS_CARRIED = slice(21, 26)  # hunger, thirst, fatigue, carried food, carried wood


def near_features(ep: A.AgentEpisode, own: np.ndarray) -> np.ndarray:
    """[T, N, 2 * (1 + F_near)] mean and max over the 8 nearest living agents (Chebyshev; distance included)."""
    T, N = ep.alive.shape
    x = np.nan_to_num(ep.col("x"), nan=0.0)
    y = np.nan_to_num(ep.col("y"), nan=0.0)
    dist = np.maximum(np.abs(x[:, :, None] - x[:, None, :]), np.abs(y[:, :, None] - y[:, None, :]))
    valid = ep.alive[:, None, :] & ep.alive[:, :, None] & ~np.eye(N, dtype=bool)[None]
    dist = np.where(valid, dist, np.inf)
    k = min(K_NEAR, max(N - 1, 1))
    idx = np.argsort(dist, axis=2, kind="stable")[:, :, :k]
    dk = np.take_along_axis(dist, idx, axis=2)
    ok = np.isfinite(dk)
    sub = np.concatenate([own[..., NEAR_COLS], own[..., NEEDS_CARRIED]], axis=-1)   # one-hots + needs + carried
    gathered = sub[np.arange(T)[:, None, None], idx]                          # [T, N, k, F]
    feats = np.concatenate([np.where(ok, dk, 0)[..., None] / 10.0, gathered], axis=-1) * ok[..., None]
    cnt = ok.sum(axis=2, keepdims=True).clip(min=1)
    mean = feats.sum(axis=2) / cnt
    mx = np.where(ok[..., None], feats, -1.0).max(axis=2)
    return np.concatenate([mean, mx], axis=-1) * ep.alive[..., None]


class Featurizer:
    def __init__(self, norm: d.Normalizer):
        self.norm = norm

    def episode(self, ep: A.AgentEpisode, actions: np.ndarray | None = None) -> np.ndarray:
        """[T, N, F] features of every agent at every row."""
        own = own_features(ep)
        near = near_features(ep, own)
        s = ep.settlement
        row = np.concatenate([(s.state(d.MAIN) - self.norm.mean) / self.norm.std, s.calendar,
                              (s.bench.actions if actions is None else actions) / d.ACTION_SCALE], axis=1)
        T, N = ep.alive.shape
        return np.concatenate([own, near, np.broadcast_to(row[:, None, :], (T, N, row.shape[1]))], axis=-1).astype(np.float32)


def targets(ep: A.AgentEpisode, dstd: np.ndarray | None = None):
    """Per (t, n) with t+1 valid: next state tag, target kind, dx/dy classes (index 0..6), needs change."""
    a = ep.agents
    nxt = np.concatenate([a[1:], a[-1:]], axis=0)
    st = np.nan_to_num(nxt[..., F_("state_tag")], nan=0).astype(int)
    tk = np.nan_to_num(nxt[..., F_("target_kind")], nan=0).astype(int)
    dx = np.nan_to_num(nxt[..., F_("x")] - a[..., F_("x")], nan=0).clip(-DMAX, DMAX).astype(int) + DMAX
    dy = np.nan_to_num(nxt[..., F_("y")] - a[..., F_("y")], nan=0).clip(-DMAX, DMAX).astype(int) + DMAX
    dn = np.nan_to_num(nxt[..., [F_(k) for k in A.NEEDS]] - a[..., [F_(k) for k in A.NEEDS]], nan=0)
    if dstd is not None:
        dn = dn / dstd
    return st, tk, dx, dy, dn.astype(np.float32)


def train_mask(ep: A.AgentEpisode) -> np.ndarray:
    m = np.zeros_like(ep.alive)
    m[:-1] = ep.alive[:-1] & ep.alive[1:]
    return m


if torch is not None:
    class Heads(nn.Module):
        def __init__(self, h: int):
            super().__init__()
            self.st, self.tk, self.dx, self.dy, self.dn = (nn.Linear(h, 4), nn.Linear(h, 8), nn.Linear(h, 2 * DMAX + 1),
                                                           nn.Linear(h, 2 * DMAX + 1), nn.Linear(h, 3))

        def forward(self, z):
            return self.st(z), self.tk(z), self.dx(z), self.dy(z), self.dn(z)

    class MLP(nn.Module):
        def __init__(self, n_in: int, h: int):
            super().__init__()
            self.body = nn.Sequential(nn.Linear(n_in, h), nn.GELU(), nn.Linear(h, h), nn.GELU(), nn.Linear(h, h), nn.GELU())
            self.heads = Heads(h)

        def forward(self, x, mask=None):
            return self.heads(self.body(x))

    class SetTransformer(nn.Module):
        def __init__(self, n_in: int, dm: int):
            super().__init__()
            self.inp = nn.Sequential(nn.Linear(n_in, dm), nn.GELU())
            layer = nn.TransformerEncoderLayer(dm, 4, 4 * dm, dropout=0.0, batch_first=True)
            self.enc = nn.TransformerEncoder(layer, 2, enable_nested_tensor=False)
            self.heads = Heads(dm)

        def forward(self, x, mask):
            """x [R, N, F], mask [R, N] True where the agent is alive."""
            return self.heads(self.enc(self.inp(x), src_key_padding_mask=~mask))


def loss_fn(out, st, tk, dx, dy, dn):
    o_st, o_tk, o_dx, o_dy, o_dn = out
    return (F.cross_entropy(o_st, st) + F.cross_entropy(o_tk, tk) + F.cross_entropy(o_dx, dx)
            + F.cross_entropy(o_dy, dy) + F.smooth_l1_loss(o_dn, dn))


class M2Predictor:
    def __init__(self, model, arch: str, feat: Featurizer, dstd: np.ndarray, device: str):
        self.model, self.arch, self.feat, self.dstd, self.device = model, arch, feat, dstd, device

    @torch.no_grad() if torch is not None else (lambda f: f)
    def outputs(self, ep: A.AgentEpisode, rows: np.ndarray) -> dict[str, np.ndarray]:
        """Predictions for every agent of the given rows: arrays [len(rows), N]."""
        self.model.eval()
        X = self.feat.episode(ep)[rows]
        mask = ep.alive[rows]
        res = {k: [] for k in ("st", "tk", "dx", "dy", "dn")}
        chunk = 256
        for i in range(0, len(rows), chunk):
            xb = torch.tensor(X[i:i + chunk], device=self.device)
            mb = torch.tensor(mask[i:i + chunk], device=self.device)
            if self.arch == "set":
                out = self.model(xb, mb)
            else:
                R, N, Fd = xb.shape
                out = [o.reshape(R, N, -1) for o in self.model(xb.reshape(R * N, Fd))]
            for k, o in zip(res, out):
                res[k].append((o.argmax(-1) if k != "dn" else o).cpu().numpy())
        return {k: np.concatenate(v) for k, v in res.items()}

    def __call__(self, ep: A.AgentEpisode, t: np.ndarray, n: np.ndarray) -> A.AgentPred:
        rows = np.unique(t)
        out = self.outputs(ep, rows)
        pos = np.searchsorted(rows, t)
        cur = A.state_at(ep, t, n)
        return A.AgentPred(out["st"][pos, n], out["tk"][pos, n], cur.x + out["dx"][pos, n] - DMAX,
                           cur.y + out["dy"][pos, n] - DMAX, cur.needs + out["dn"][pos, n] * self.dstd)


def aggregate_working(pred: M2Predictor, eps: list[A.AgentEpisode]) -> dict:
    """Sum of predicted working adults (alive at t, adult, member, predicted target kind a site or a gathering
    target) vs the recorded `adults_working` at t+1; also the same count on the true agent states at t+1 (how well
    the agent-level definition matches the settlement column) and persistence."""
    pr, tr, proxy, pers = [], [], [], []
    for ep in eps:
        rows = np.arange(23, ep.rows - 1)
        out = pred.outputs(ep, rows)
        tk = np.where(ep.alive[rows], out["tk"], -1)
        pr.append(A.working_count(ep, rows, tk))
        aw = ep.settlement.bench.v[:, b.VARS.index("adults_working")]
        tr.append(aw[rows + 1])
        proxy.append(A.working_count(ep, rows + 1))
        pers.append(aw[rows])
    p, t, x, q = (np.concatenate(z).astype(float) for z in (pr, tr, proxy, pers))
    sd = t.std()
    return {"model_nmae": float(np.abs(p - t).mean() / sd), "true_agents_proxy_nmae": float(np.abs(x - t).mean() / sd),
            "persistence_nmae": float(np.abs(q - t).mean() / sd), "n": int(len(t))}


def score(m: dict, base: dict) -> float:
    """Validation selection score (lower is better)."""
    return ((1 - m["state_tag_accuracy"]) + (1 - m["target_kind_accuracy"])
            + m["position_chebyshev"] / max(base["position_chebyshev"], 1e-9) + m["needs_nmae_mean"])


def train(args) -> int:
    cfg = dict(DEFAULTS)
    cfg.update({k: v for k, v in vars(args).items() if k in DEFAULTS and v is not None})
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    np.random.seed(args.seed)
    torch.manual_seed(args.seed)
    torch.set_num_threads(cfg["threads"])
    torch.use_deterministic_algorithms(True)
    device = "cpu"
    t0 = time.time()
    m = c.manifest()
    root = Path(args.root)
    tr_eps = A.load_agent_group(root, m, "A_train")
    va_eps = A.load_agent_group(root, m, "A_val")
    norm = d.Normalizer.fit([e.settlement for e in tr_eps], d.MAIN)
    feat = Featurizer(norm)
    raw_dn = np.vstack([targets(e)[4][train_mask(e)] for e in tr_eps])
    dstd = raw_dn.std(axis=0)
    dstd[dstd == 0] = 1.0
    Xs, Ms, Ys = [], [], []
    for e in tr_eps:
        Xs.append(feat.episode(e))
        Ms.append(train_mask(e))
        Ys.append(targets(e, dstd))
    n_in = Xs[0].shape[-1]
    rng = np.random.default_rng(args.seed)
    if args.arch == "mlp":
        Xf = np.concatenate([x[mm] for x, mm in zip(Xs, Ms)])
        Yf = [np.concatenate([y[i][mm] for y, mm in zip(Ys, Ms)]) for i in range(5)]
        model = MLP(n_in, cfg["hidden"]).to(device)
        n_items = len(Xf)
        per_epoch = n_items // cfg["batch"]
    else:
        Nmax = max(x.shape[1] for x in Xs)
        pad = lambda a, fill=0: np.concatenate([a, np.full(a.shape[:1] + (Nmax - a.shape[1],) + a.shape[2:], fill, a.dtype)], axis=1)
        rows_ok = [mm.any(axis=1) for mm in Ms]
        Xr = np.concatenate([pad(x)[ok] for x, ok in zip(Xs, rows_ok)])
        Mr = np.concatenate([pad(mm, False)[ok] for mm, ok in zip(Ms, rows_ok)])
        Yr = [np.concatenate([pad(y[i])[ok] for y, ok in zip(Ys, rows_ok)]) for i in range(5)]
        model = SetTransformer(n_in, cfg["d_model"]).to(device)
        n_items = len(Xr)
        per_epoch = n_items // cfg["row_batch"]
    opt = torch.optim.AdamW(model.parameters(), lr=cfg["lr"], weight_decay=cfg["weight_decay"])
    total = cfg["epochs"] * per_epoch
    sched = torch.optim.lr_scheduler.OneCycleLR(opt, max_lr=cfg["lr"], total_steps=max(total, 1))
    pred = M2Predictor(model, args.arch, feat, dstd, device)
    base_val = A.t5(A.persistence, va_eps)
    std_val = A.needs_std(va_eps)
    config = {"model": "m2", "arch": args.arch, "seed": args.seed, "hyper": cfg, "device": device,
              "torch": torch.__version__, "code_commit": c.git_commit(), "manifest_sha256": c.manifest_sha(),
              "train": "A_obs agent episodes 0-31", "val": "A_obs 96-99", "n_inputs": n_in, "n_items": n_items,
              "normalizer": norm.to_json(), "dstd": dstd.tolist(), "hidden_fields_excluded": list(A.HIDDEN)}
    c.write_json(out / "config.json", config)
    log = (out / "train_log.jsonl").open("w")
    best, step, stop = float("inf"), 0, "epochs"
    for epoch in range(cfg["epochs"]):
        perm = rng.permutation(n_items)
        for i in range(per_epoch):
            model.train()
            if args.arch == "mlp":
                idx = perm[i * cfg["batch"]:(i + 1) * cfg["batch"]]
                xb = torch.tensor(Xf[idx])
                o = model(xb)
                ys = [torch.tensor(y[idx]) for y in Yf]
                loss = loss_fn(o, *[ys[j].long() for j in range(4)], ys[4])
            else:
                idx = perm[i * cfg["row_batch"]:(i + 1) * cfg["row_batch"]]
                xb, mb = torch.tensor(Xr[idx]), torch.tensor(Mr[idx])
                o = model(xb, mb)
                sel = lambda z: z[mb]
                ys = [torch.tensor(y[idx]) for y in Yr]
                loss = loss_fn([sel(z) for z in o], *[sel(ys[j]).long() for j in range(4)], sel(ys[4]))
            opt.zero_grad()
            loss.backward()
            nn.utils.clip_grad_norm_(model.parameters(), 1.0)
            opt.step()
            sched.step()
            step += 1
            if step % 100 == 0:
                log.write(json.dumps({"step": step, "epoch": epoch, "loss": loss.item(), "t": time.time() - t0}) + "\n")
                log.flush()
            if step % cfg["eval_every"] == 0 or (epoch == cfg["epochs"] - 1 and i == per_epoch - 1):
                vm = A.t5(pred, va_eps, std_val)
                s = score(vm, base_val)
                if s < best:
                    best = s
                    torch.save(model.state_dict(), out / "best.pt")
                log.write(json.dumps({"step": step, "eval": True, "val": vm, "score": s, "t": time.time() - t0}) + "\n")
                log.flush()
                print(f"step {step} ep {epoch} loss {loss.item():.4f} val score {s:.4f} st {vm['state_tag_accuracy']:.4f} "
                      f"pos {vm['position_chebyshev']:.4f} {time.time() - t0:.0f}s", flush=True)
            if time.time() - t0 > cfg["max_seconds"]:
                stop = "time"
                break
        if stop == "time":
            break
    model.load_state_dict(torch.load(out / "best.pt"))
    vm = A.t5(pred, va_eps, std_val)
    summary = {"val": vm, "val_score": score(vm, base_val), "val_persistence": base_val,
               "val_day_earlier": A.t5(A.day_earlier, va_eps, std_val), "steps": step, "stop": stop,
               "seconds": time.time() - t0, "val_aggregate": aggregate_working(pred, va_eps)}
    c.write_json(out / "val_metrics.json", summary)
    print(f"done val score {summary['val_score']:.4f} {time.time() - t0:.0f}s")
    return 0


def load(run: Path):
    config = json.loads((run / "config.json").read_text())
    torch.set_num_threads(config["hyper"]["threads"])
    norm = d.Normalizer.from_json(config["normalizer"])
    n_in = config["n_inputs"]
    model = MLP(n_in, config["hyper"]["hidden"]) if config["arch"] == "mlp" else SetTransformer(n_in, config["hyper"]["d_model"])
    model.load_state_dict(torch.load(run / "best.pt"))
    return M2Predictor(model, config["arch"], Featurizer(norm), np.array(config["dstd"]), "cpu"), config


def final(args) -> int:
    run = Path(args.run)
    if (run / "metrics.json").exists() and not args.force:
        raise SystemExit("test metrics are computed once per configuration")
    pred, config = load(run)
    m = c.manifest()
    out = {}
    for name in ("A_test", "B_test"):
        eps = A.load_agent_group(Path(args.root), m, name)
        std = A.needs_std(eps)
        out[name] = {"m2": A.t5(pred, eps, std), "persistence": A.t5(A.persistence, eps, std),
                     "day_earlier": A.t5(A.day_earlier, eps, std), "aggregate_adults_working": aggregate_working(pred, eps)}
    out["config"] = {k: config[k] for k in ("arch", "seed", "code_commit")}
    out["eval_code_commit"] = c.git_commit()
    out["argv"], out["forced"] = sys.argv, bool(args.force)   # provenance of the single test evaluation
    c.write_json(run / "metrics.json", out)
    print(json.dumps({k: v["m2"]["state_tag_accuracy"] for k, v in out.items() if k.endswith("test")}))
    return 0


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    tr = sub.add_parser("train")
    tr.add_argument("--out", required=True)
    tr.add_argument("--arch", default="mlp", choices=["mlp", "set"])
    tr.add_argument("--seed", type=int, default=0)
    tr.add_argument("--root", default=str(c.DEFAULT_ROOT))
    for k, v in DEFAULTS.items():
        tr.add_argument(f"--{k.replace('_', '-')}", dest=k, type=type(v), default=None)
    fi = sub.add_parser("final")
    fi.add_argument("--run", required=True)
    fi.add_argument("--root", default=str(c.DEFAULT_ROOT))
    fi.add_argument("--force", action="store_true")
    args = ap.parse_args(argv)
    if torch is None:
        raise SystemExit("torch is required")
    return train(args) if args.cmd == "train" else final(args)


if __name__ == "__main__":
    raise SystemExit(main())
