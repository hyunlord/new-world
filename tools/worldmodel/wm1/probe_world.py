"""World identification probe (GOAL_1_2.md 9.3, METRICS_1_2.md Z5).

For a trained M1b on a pilot dataset: the model's final layer-normed state (`M1TF.hidden`) at row s = 166 (06:00
of display day 8) after the window of rows s-167 .. s (row -1 masked, as the predictor's warm-up masks rows < 0),
for every train / val / test episode; a ridge from that state to each hidden world parameter of `meta.json`'s
`world_variety` (labels only, read here after training, never an input), lambda chosen on val (MSE), R^2 on test
(1 - SSE / SS around the test mean). `climate` (a 0-2 category): a one-vs-rest ridge (one-hot targets, lambda on
val one-hot MSE), accuracy of the argmax on test. Reference: the same ridges from the raw observable state row at s
(the 35 columns of the observable table). `starting_agents` equals `alive` at row 0, so the reference recovers it
almost trivially from the row-166 population.

usage (DGX, venv): python3 tools/worldmodel/wm1/probe_world.py --run RUN_DIR --out OUT_DIR
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
from wm1 import calcore as cc
from wm1 import common as c
from wm1 import data as d

PARAMS = ("starting_agents", "berry_bushes", "berry_capacity", "deadwood_spots", "deadwood_capacity",
          "forest_permille", "food_regen_permille")
CATEGORY = "climate"
CLIMATES = (0, 1, 2)
LAMBDAS = (0.01, 0.1, 1.0, 10.0, 100.0, 1000.0, 10000.0)
DAY, MINUTE = 8, 360
WINDOW = 168


def probe_row() -> int:
    r = cc.row_of(DAY, MINUTE)
    assert r == 166, r
    return r


def r2(y: np.ndarray, pred: np.ndarray) -> float | None:
    sst = float(((y - y.mean()) ** 2).sum())
    return None if sst == 0 else 1.0 - float(((y - pred) ** 2).sum()) / sst


def fit_select(xtr, ytr, xva, yva):
    """Ridge (bench_v0.Ridge: standardized x, unpenalized intercept) with lambda from LAMBDAS by val MSE."""
    best, scores = None, {}
    for lam in LAMBDAS:
        m = b.Ridge().fit(xtr, ytr, lam)
        mse = float(((m.predict(xva) - yva) ** 2).mean())
        scores[str(lam)] = mse
        if best is None or mse < best[0]:
            best = (mse, lam, m)
    return best[2], best[1], scores


def probe(x: dict[str, np.ndarray], y: dict[str, dict[str, np.ndarray]]) -> dict:
    """x[role] [E, D]; y[role][param] [E]. R^2 per parameter on test, climate accuracy on test."""
    out = {}
    for p in PARAMS:
        m, lam, scores = fit_select(x["train"], y["train"][p][:, None], x["val"], y["val"][p][:, None])
        pred = m.predict(x["test"])[:, 0]
        out[p] = {"r2_test": r2(y["test"][p], pred), "lambda": lam, "val_mse": scores,
                  "train_values": sorted({float(v) for v in y["train"][p]})[:3] + ["..."]}
    hot = {r: b.onehot(y[r][CATEGORY].astype(np.int64), list(CLIMATES)) for r in y}
    m, lam, scores = fit_select(x["train"], hot["train"], x["val"], hot["val"])
    pred = np.array(CLIMATES)[m.predict(x["test"]).argmax(axis=1)]
    truth = y["test"][CATEGORY]
    out[CATEGORY] = {"accuracy_test": float(np.mean(pred == truth)), "lambda": lam, "val_mse": scores,
                     "majority_train_class_accuracy_test": float(np.mean(truth == np.bincount(
                         y["train"][CATEGORY].astype(np.int64), minlength=3).argmax())),
                     "test_counts": np.bincount(truth.astype(np.int64), minlength=3).tolist()}
    return out


def labels(root: Path, ds: str, role: str) -> tuple[list[int], dict[str, np.ndarray]]:
    """world_variety of every episode of a role, in manifest order (meta.json read directly: labels only)."""
    spec = c.dataset(ds)
    split, _ = spec.groups[role]
    seeds, rows = [], []
    for e in c.manifest(ds)["episodes"]:
        if e["split"] == split and e["role"] == role:
            meta = json.loads((root / e["path"] / "meta.json").read_text())
            seeds.append(int(e["seed"]))
            rows.append(meta["world_variety"]["values"])
    return seeds, {k: np.array([r[k] for r in rows], dtype=np.float64) for k in PARAMS + (CATEGORY,)}


def hidden_states(pred, eps: list[d.EpisodeData], s: int) -> np.ndarray:
    import torch
    out = []
    for ep in eps:
        prep = d.prepare(ep, pred.variant, pred.norm)
        rows = np.arange(s - (WINDOW - 1), s + 1)
        valid = rows >= 0
        r = np.clip(rows, 0, None)
        inp = np.concatenate([prep.x_state[r], prep.calendar[r], prep.actions[r], prep.sev[r]], axis=-1)
        tt = lambda a, dt=torch.float32: torch.tensor(np.asarray(a)[None], dtype=dt, device=pred.device)
        pred.model.eval()
        out.append(pred.model.hidden(tt(inp), tt(valid, torch.bool))[0].double().cpu().numpy())
    return np.stack(out)


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--run", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--root", default=str(c.DEFAULT_ROOT))
    ap.add_argument("--device", default="auto")
    args = ap.parse_args(argv)
    from wm1 import m1_gru
    t0 = time.time()
    run, target = Path(args.run), Path(args.out) / "probe.json"
    if target.exists():
        raise SystemExit(f"{target} exists: test numbers are computed once")
    pred, config = m1_gru.load_predictor(run, args.device)
    ds = config["dataset"]
    if config.get("model") != "m1_tf":
        raise SystemExit("the probe reads M1TF.hidden")
    s = probe_row()
    g = c.load_groups(["train", "val", "test"], Path(args.root), ds)
    xs, xr, ys = {}, {}, {}
    for role in ("train", "val", "test"):
        seeds, ys[role] = labels(Path(args.root), ds, role)
        if seeds != [int(ep.seed) for ep in g[role]]:
            raise SystemExit(f"{role}: label order differs from the loaded episodes")
        xs[role] = hidden_states(pred, g[role], s)
        xr[role] = np.stack([ep.state(pred.variant)[s] for ep in g[role]])
    out = {"run": str(run), "dataset": ds, "row": s, "window": [s - WINDOW + 1, s],
           "hidden_dim": int(xs["train"].shape[1]), "reference_columns": list(pred.variant.state_names),
           "model": probe(xs, ys), "reference_raw_row": probe(xr, ys),
           "counts": {r: len(g[r]) for r in g}, "code_commit": c.git_commit(),
           "run_code_commit": config["code_commit"], "manifest_sha256": c.manifest_sha(ds), "argv": sys.argv,
           "seconds": time.time() - t0}
    c.write_json(target, out)
    print(json.dumps({p: (v.get("r2_test", v.get("accuracy_test"))) for p, v in out["model"].items()}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
