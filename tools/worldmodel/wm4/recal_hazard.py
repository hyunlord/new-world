"""Platt recalibration of WM-3's hazard heads (docs/worldmodel/wm4/GOAL.md 9.4).

WM-3 never selected for its hazard head (checkpoint selection read `val_eff` and `val_t2`), and the head's
miscalibration shows on 720-step rollouts, whose states differ from the teacher-forced ones it was trained on; so the
trunk stays frozen and only the head's logit is rescaled: h' = σ(a · logit h + b). (a, b) maximize the likelihood of
"split within 30 days", P = 1 − ∏_{k=1..720} (1 − h'_k), against the truth `v2.split_event_row(none_r0, c) <= c + 720`,
on 720-step zero-action rollouts from every **train**-family `none_r0` W3 cast row (`effects3.rollout_jobs_h`,
`round_counts` off). The result is written into the head — `haz.weight ← a · W`, `haz.bias ← a · b0 + b`
(`wm3.model.HAZ_KEYS`) — of a new run directory holding a copy of the run's `config.json` (plus a `recal` block) and
`val_metrics.json`, so `effects3` split / calib / model work unchanged. It is kept only if its val Brier
(`effects3.calibration` on the val families) is below the original's; recorded either way (`recal.json`).

usage (DGX):
  python3 tools/worldmodel/wm4/recal_hazard.py fit --run RUN --base effects_base.json --out DIR
  python3 tools/worldmodel/wm4/recal_hazard.py calib --run RUN --base effects_base.json --role val|test --out DIR
"""
from __future__ import annotations

import argparse
import json
import shutil
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm4  # noqa: F401
from wm1 import common as c
from wm2 import effects as ef
from wm2 import rollout_ext as rx
from wm2 import v2
from wm3 import effects3 as e3

HORIZON = 720
EPS = 1e-7


def logit(h: np.ndarray) -> np.ndarray:
    h = np.clip(np.asarray(h, dtype=np.float64), EPS, 1.0 - EPS)
    return np.log(h) - np.log1p(-h)


def p30(logits: np.ndarray, a: float, b: float) -> np.ndarray:
    """P(split within the horizon) = 1 − ∏_k σ(−(a L_k + b)) for logits [n, K]."""
    z = a * logits + b
    log_surv = -np.logaddexp(0.0, z).sum(axis=1)          # Σ log σ(−z)
    return -np.expm1(log_surv)


def fit_platt(logits: np.ndarray, y: np.ndarray, iters: int = 100) -> dict:
    """Maximum-likelihood (a, b) of `p30` (mean Bernoulli NLL): damped Newton in float64 from (1, 0) with exact
    gradient and Hessian (`torch.autograd.functional`), step halving until the NLL decreases."""
    from wm1 import m1_gru as g
    torch = g.torch
    L = torch.tensor(logits, dtype=torch.float64)
    t = torch.tensor(y, dtype=torch.float64)

    def nll(ab):
        log_surv = -torch.nn.functional.softplus(ab[0] * L + ab[1]).sum(dim=1)     # log P(no split)
        log_p = torch.log(-torch.expm1(log_surv).clamp(max=-1e-300))
        return -(t * log_p + (1 - t) * log_surv).mean()

    ab = torch.tensor([1.0, 0.0], dtype=torch.float64)
    before = cur = float(nll(ab))
    steps = 0
    for steps in range(1, iters + 1):
        grad = torch.autograd.functional.jacobian(nll, ab)
        hess = torch.autograd.functional.hessian(nll, ab)
        try:
            step = torch.linalg.solve(hess + 1e-9 * torch.eye(2, dtype=torch.float64), grad)
        except RuntimeError:
            step = grad
        if float(grad @ step) <= 0:            # not a descent direction: gradient step
            step = grad
        scale, moved = 1.0, False
        for _ in range(40):
            cand = ab - scale * step
            val = float(nll(cand))
            if np.isfinite(val) and val < cur:
                ab, moved = cand, True
                break
            scale /= 2
        if not moved or cur - val < 1e-13:
            if moved:
                cur = val
            break
        cur = val
    a, b = (float(x) for x in ab)
    return {"a": a, "b": b, "nll_before": before, "nll_after": cur, "newton_steps": steps, "n": int(len(y)),
            "pos_rate": float(np.mean(y))}


def train_rollouts(pred, base: dict, root: Path) -> tuple[np.ndarray, np.ndarray, list]:
    """Hazard logits [n, 720] of the zero-action rollouts from every train-family `none_r0` W3 cast row and the truth."""
    cases = [ef.case_from(j) for j in base["cases"]["train"] if j["rep"] == 0 and j["arm"] in v2.W3_ARMS]
    rows = sorted({(x.seed, x.cast_row) for x in cases})
    fams = {f.seed: f for f in e3.load_families(ef.family_dirs(root, "train"), "train")}
    jobs, y = [], []
    for seed, row in rows:
        ep = fams[seed].factual[0]
        jobs.append(rx.Job(ep, row, np.zeros_like(ep.bench.actions), HORIZON))
        ev = v2.split_event_row(fams[seed].truth[0]["fissions_total"], row)
        y.append(float(ev is not None and ev <= row + HORIZON))
    _, hs = e3.rollout_jobs_h(pred, jobs)
    return logit(np.stack(hs)), np.array(y), [f"{s}:{r}" for s, r in rows]


def calib(pred, base: dict, root: Path, role: str) -> dict:
    cases = [ef.case_from(j) for j in base["cases"][role]]
    fams = e3.load_families(ef.family_dirs(root, role), role)
    return e3.calibration(pred, fams, cases)


def write_head(model, a: float, b: float) -> None:
    from wm1 import m1_gru as g
    with g.torch.no_grad():
        model.haz.bias.copy_(a * model.haz.bias + b)
        model.haz.weight.mul_(a)


def cmd_fit(args) -> int:
    from wm1 import m1_gru as g
    t0 = time.time()
    run, out = Path(args.run), Path(args.out)
    if (out / "recal.json").exists():
        raise SystemExit(f"{out}/recal.json exists")
    pred, config = e3._load(run, args.device)
    if not e3.is_hazard_run(config):
        raise SystemExit(f"{run}: not a hazard run")
    pred.round_counts = False
    base = json.loads(Path(args.base).read_text())
    root = Path(args.root)
    L, y, keys = train_rollouts(pred, base, root)
    t_roll = time.time() - t0
    fit = fit_platt(L, y)
    p_before, p_after = p30(L, 1.0, 0.0), p30(L, fit["a"], fit["b"])
    train_brier = {"before": float(((p_before - y) ** 2).mean()), "after": float(((p_after - y) ** 2).mean()),
                   "climatology": float(((y.mean() - y) ** 2).mean())}
    val_orig = calib(pred, base, root, "val")
    write_head(pred.model, fit["a"], fit["b"])
    val_new = calib(pred, base, root, "val")
    kept = val_new["brier"] < val_orig["brier"]
    out.mkdir(parents=True, exist_ok=True)
    g.torch.save(pred.model.state_dict(), out / "best.pt")
    shutil.copy(run / "val_metrics.json", out / "val_metrics.json")
    cfg = json.loads((run / "config.json").read_text())
    cfg["recal"] = {"from_run": str(run), "a": fit["a"], "b": fit["b"], "code_commit": c.git_commit(), "kept": kept,
                    "init_checkpoint_sha256": ef._sha(run / "best.pt")}
    c.write_json(out / "config.json", cfg)
    res = {"run": str(run), "out": str(out), "fit": fit, "train_brier": train_brier, "train_rows": keys,
           "train_pred_mean": {"before": float(p_before.mean()), "after": float(p_after.mean())},
           "val_original": {k: v for k, v in val_orig.items() if k != "per_row"},
           "val_recal": {k: v for k, v in val_new.items() if k != "per_row"}, "kept": kept,
           "code_commit": c.git_commit(), "argv": sys.argv, "rollout_seconds": t_roll, "seconds": time.time() - t0}
    c.write_json(out / "recal.json", res)
    print(f"recal {run.name}: a {fit['a']:.4f} b {fit['b']:.4f}; val brier {val_orig['brier']:.4f} -> "
          f"{val_new['brier']:.4f} (kept {kept}); {time.time() - t0:.0f}s")
    return 0


def cmd_calib(args) -> int:
    t0 = time.time()
    target = Path(args.out) / f"calib_{args.role}.json"
    if target.exists():
        raise SystemExit(f"{target} exists")
    pred, config = e3._load(Path(args.run), args.device)
    pred.round_counts = False
    base = json.loads(Path(args.base).read_text())
    res = calib(pred, base, Path(args.root), args.role) | {"run": args.run, "role": args.role,
                                                            "code_commit": c.git_commit(), "argv": sys.argv,
                                                            "recal": config.get("recal"), "seconds": time.time() - t0}
    c.write_json(target, res)
    print(f"calib {args.role} {Path(args.run).name}: brier {res['brier']:.4f} clim {res['brier_climatology']:.4f}")
    return 0


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    for name in ("fit", "calib"):
        p = sub.add_parser(name)
        p.add_argument("--run", required=True)
        p.add_argument("--base", required=True)
        p.add_argument("--root", default=str(c.DEFAULT_ROOT))
        p.add_argument("--device", default="auto")
        p.add_argument("--out", required=True)
        if name == "calib":
            p.add_argument("--role", required=True, choices=["val", "test"])
    a = ap.parse_args(argv)
    return cmd_fit(a) if a.cmd == "fit" else cmd_calib(a)


if __name__ == "__main__":
    raise SystemExit(main())
