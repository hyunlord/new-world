"""M1: settlement-level recurrent world model (GOAL.md 8).

input projection -> 2-layer GRU (hidden 256) -> heads: Gaussian mean and log-variance of every continuous
feature's normalized hourly change, Bernoulli logit of the next `fire_lit`. Trained with teacher forcing plus a
rollout loss (unroll k steps on its own predictions, k growing 1 -> 24 over the first half of training);
early-stopped on the validation mean nMAE of T2 24 h (bench starts, A val and B val averaged).

usage:
  python3 tools/worldmodel/wm1/m1_gru.py train --out RUN_DIR --variant sev-blind_action-aware --seed 0 [...]
  python3 tools/worldmodel/wm1/m1_gru.py final --run RUN_DIR      # test evaluation, once per configuration

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
                grad_clip=1.0, threads=16)


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

    def __init__(self, lay: d.Layout, norm: d.Normalizer, device: str):
        t = lambda a: torch.tensor(np.asarray(a), dtype=torch.float32, device=device)
        self.lay = lay
        self.cont = torch.tensor(lay.cont, device=device)
        self.mean, self.std, self.dstd = t(norm.mean), t(norm.std), t(norm.dstd)
        span = norm.hi - norm.lo
        self.lo, self.hi = t(norm.lo - 0.25 * span), t(norm.hi + 0.25 * span)

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
    """A trained M1 as a harness predictor."""
    provides_t4 = True

    def __init__(self, model, lay: d.Layout, norm: d.Normalizer, variant: d.Variant, device: str, warmup: int = 168,
                 name: str = "m1_gru", round_counts: bool = False, chunk: int = 4096):
        self.model, self.lay, self.norm, self.variant = model, lay, norm, variant
        self.device, self.warmup, self.name, self.round_counts, self.chunk = device, warmup, name, round_counts, chunk
        self.stepper = Stepper(lay, norm, device)

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


def losses(model, stepper: Stepper, bt: dict, k: int, roll_starts: int, rng: np.random.Generator):
    """Teacher-forced Gaussian NLL + BCE, and the k-step rollout loss (L1 of the train-std-normalized continuous
    state + fire BCE, averaged over the k steps) from `roll_starts` positions per window."""
    L = bt["dtarget"].shape[1]
    inp = torch.cat([bt["x_state"], bt["calendar"], bt["actions"], bt["sev"]], dim=-1)
    mu, logvar, fl, outs = model.run(inp[:, :L])
    y = bt["dtarget"]
    nll = 0.5 * (logvar + (y - mu) ** 2 / logvar.exp()).mean()
    bce = F.binary_cross_entropy_with_logits(fl, bt["fire_next"])
    tf = nll + bce
    B = inp.shape[0]
    js = torch.tensor(np.sort(rng.integers(0, L - k + 1, size=roll_starts)), device=inp.device)
    m = len(js)
    flat = lambda a: a.reshape(B * m, *a.shape[2:])
    # The teacher-forced state after consuming row j's true input, for every layer: [layers, B*m, H].
    h = outs[:, :, js].reshape(outs.shape[0], B * m, outs.shape[-1])
    mu_j, fl_j = flat(mu[:, js]), flat(fl[:, js])
    cur = flat(bt["state"][:, js])
    sev_hold = flat(bt["sev"][:, js])
    cont = stepper.cont
    roll = 0.0
    for i in range(k):
        cur = stepper.next_state(cur, mu_j, fl_j, hard=False)
        true = flat(bt["state"][:, js + i + 1])
        err = (cur[:, cont] - true[:, cont]) / stepper.std[cont]
        roll = roll + err.abs().mean() + F.binary_cross_entropy_with_logits(fl_j, true[:, stepper.lay.fire])
        if i == k - 1:
            break
        r = js + i + 1
        nxt = torch.cat([stepper.normalize(cur), flat(bt["calendar"][:, r]), flat(bt["actions"][:, r]), sev_hold], -1)
        mm, _, ff, h = model(nxt.unsqueeze(1), h)
        mu_j, fl_j = mm[:, 0], ff[:, 0]
    roll = roll / k
    return tf, {"nll": float(nll), "bce": float(bce), "roll": float(roll)}, roll


def val_score(pred: ev.Predictor, val: dict[str, list], stds: dict) -> tuple[float, dict]:
    per = {n: ev.t2(pred, eps, stds[n], 24)["mean"] for n, eps in val.items()}
    return float(np.mean(list(per.values()))), per


def calibration(model, stepper, preps: list, device: str, window: int) -> dict:
    """Share of true normalized changes within +-2 sigma of the predicted mean (teacher forcing, val windows)."""
    rng = np.random.default_rng(123)
    win = batch_tensors(d.sample_windows(preps, window, 64, rng), device)
    with torch.no_grad():
        L = win["dtarget"].shape[1]
        inp = torch.cat([win["x_state"], win["calendar"], win["actions"], win["sev"]], -1)[:, :L]
        mu, logvar, fl, _ = model(inp)
        inside = ((win["dtarget"] - mu).abs() <= 2 * (0.5 * logvar).exp()).float()
    per = inside.mean(dim=(0, 1)).cpu().numpy()
    return {"within_2sigma_mean": float(per.mean()), "within_2sigma": dict(zip(stepper.lay.variant.continuous_names, per.tolist()))}


def train(args) -> int:
    cfg = dict(DEFAULTS)
    cfg.update({k: v for k, v in vars(args).items() if k in DEFAULTS and v is not None})
    variant = d.VARIANTS[args.variant]
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    device = pick_device(args.device)
    set_seed(args.seed, cfg["threads"])
    t0 = time.time()
    train_groups = ["A_train", "B_train"] if variant.action_aware else ["A_train"]
    g = c.load_groups(train_groups + list(c.VAL_SPLITS), Path(args.root))
    train_eps = [ep for n in train_groups for ep in g[n]]
    val = {n: g[n] for n in c.VAL_SPLITS}
    stds = c.split_stds(val)
    norm = d.Normalizer.fit(train_eps, variant)
    lay = d.Layout(variant)
    preps = [d.prepare(ep, variant, norm) for ep in train_eps]
    model = M1GRU(lay.n_input, lay.n_cont, cfg["hidden"], cfg["layers"]).to(device)
    opt = torch.optim.AdamW(model.parameters(), lr=cfg["lr"], weight_decay=cfg["weight_decay"])
    stepper = Stepper(lay, norm, device)
    pred = GRUPredictor(model, lay, norm, variant, device, cfg["warmup"])
    config = {"model": "m1_gru", "variant": variant.name, "seed": args.seed, "hyper": cfg, "device": device,
              "torch": torch.__version__, "numpy": np.__version__, "python": sys.version.split()[0],
              "code_commit": c.git_commit(), "manifest_sha256": c.manifest_sha(), "train_groups": train_groups,
              "val_groups": list(c.VAL_SPLITS), "train_episodes": len(train_eps), "input_names": list(variant.input_names()),
              "normalizer": norm.to_json(), "data_root": str(args.root), "threads": torch.get_num_threads()}
    c.write_json(out / "config.json", config)
    rng = np.random.default_rng(args.seed)
    log = (out / "train_log.jsonl").open("w")
    best, best_step, bad, step = float("inf"), -1, 0, 0
    half = cfg["max_steps"] // 2
    stop = "max_steps"
    for step in range(1, cfg["max_steps"] + 1):
        model.train()
        k = 1 + int((cfg["k_max"] - 1) * min(1.0, step / max(1, half)))
        bt = batch_tensors(d.sample_windows(preps, cfg["window"], cfg["batch"], rng), device)
        tf, parts, roll = losses(model, stepper, bt, k, cfg["roll_starts"], rng)
        loss = tf + cfg["roll_weight"] * roll
        opt.zero_grad()
        loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), cfg["grad_clip"])
        opt.step()
        if step % 50 == 0:
            log.write(json.dumps({"step": step, "k": k, "loss": float(loss), **parts, "t": time.time() - t0}) + "\n")
            log.flush()
        if step % cfg["eval_every"] == 0:
            score, per = val_score(pred, val, stds)
            improved = score < best - 1e-6
            if improved:
                best, best_step, bad = score, step, 0
                torch.save(model.state_dict(), out / "best.pt")
            else:
                bad += 1
            log.write(json.dumps({"step": step, "eval": True, "val_t2_24": score, "per_split": per, "best": best,
                                  "k": k, "t": time.time() - t0}) + "\n")
            log.flush()
            print(f"step {step} k {k} loss {float(loss):.4f} val {score:.4f} best {best:.4f}@{best_step} "
                  f"{time.time() - t0:.0f}s", flush=True)
            # Patience counts only after the rollout horizon has reached k_max (the curriculum's end).
            if bad >= cfg["patience"] and step >= half:
                stop = "patience"
                break
        if time.time() - t0 > cfg["max_seconds"]:
            stop = "time"
            break
    log.close()
    model.load_state_dict(torch.load(out / "best.pt", map_location=device))
    # Choose the output rounding of count variables on val (T2 24 h and 168 h).
    choice = {}
    for rc in (False, True):
        pred.round_counts = rc
        s24, _ = val_score(pred, val, stds)
        s168 = float(np.mean([ev.t2(pred, eps, stds[n], 168)["mean"] for n, eps in val.items()]))
        choice[str(rc)] = {"t2_24": s24, "t2_168": s168}
    round_counts = choice["True"]["t2_24"] + choice["True"]["t2_168"] < choice["False"]["t2_24"] + choice["False"]["t2_168"]
    pred.round_counts = round_counts
    val_metrics = ev.evaluate(pred, val, stds, cf_test=None, t3_split="none", any_hour=False, t1_on=True)
    cal_preps = [d.prepare(ep, variant, norm) for n in c.VAL_SPLITS for ep in val[n]]
    summary = {"best_val_t2_24": best, "best_step": best_step, "last_step": step, "stop": stop,
               "seconds": time.time() - t0, "round_counts": round_counts, "rounding_val": choice,
               "calibration_val": calibration(model, stepper, cal_preps, device, cfg["window"]), "val": val_metrics}
    c.write_json(out / "val_metrics.json", summary)
    config.update({"round_counts": round_counts, "best_step": best_step, "stop": stop})
    c.write_json(out / "config.json", config)
    print(f"done: best {best:.4f} at {best_step}, stop {stop}, round {round_counts}, {time.time() - t0:.0f}s")
    return 0


def load_predictor(run: Path, device: str = "auto") -> tuple[GRUPredictor, dict]:
    config = json.loads((run / "config.json").read_text())
    variant = d.VARIANTS[config["variant"]]
    norm = d.Normalizer.from_json(config["normalizer"])
    lay = d.Layout(variant)
    dev = pick_device(device)
    torch.set_num_threads(config["hyper"].get("threads", 16))
    model = M1GRU(lay.n_input, lay.n_cont, config["hyper"]["hidden"], config["hyper"]["layers"]).to(dev)
    model.load_state_dict(torch.load(run / "best.pt", map_location=dev))
    return GRUPredictor(model, lay, norm, variant, dev, config["hyper"]["warmup"],
                        round_counts=config.get("round_counts", False)), config


def final(args) -> int:
    """Test evaluation of a trained run (GOAL.md 4.2: once per configuration)."""
    run = Path(args.run)
    target = run / "metrics.json"
    if target.exists() and not args.force:
        raise SystemExit(f"{target} exists: test metrics are computed once per configuration")
    pred, config = load_predictor(run, args.device)
    t0 = time.time()
    g = c.load_groups(list(c.TEST_SPLITS) + ["C_test"], Path(args.root))
    tests = {n: g[n] for n in c.TEST_SPLITS}
    out = ev.evaluate(pred, tests, c.split_stds(tests), cf_test=g["C_test"], t3_split="A_test")
    out["config"] = {k: config[k] for k in ("variant", "seed", "device", "code_commit", "round_counts")}
    out["eval_code_commit"] = c.git_commit()
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
    for k, v in DEFAULTS.items():
        tr.add_argument(f"--{k.replace('_', '-')}", dest=k, type=type(v), default=None)
    fi = sub.add_parser("final")
    fi.add_argument("--run", required=True)
    fi.add_argument("--root", default=str(c.DEFAULT_ROOT))
    fi.add_argument("--device", default="auto")
    fi.add_argument("--force", action="store_true")
    args = ap.parse_args(argv)
    if torch is None:
        raise SystemExit("torch is required")
    return train(args) if args.cmd == "train" else final(args)


if __name__ == "__main__":
    raise SystemExit(main())
