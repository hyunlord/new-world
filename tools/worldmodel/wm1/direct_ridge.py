"""usage: python3 tools/worldmodel/wm1/direct_ridge.py OUT.json   (DGX; numpy only)

Analysis-only reference for M3: a direct 24-hour ridge on M3's inputs (the 06:00 row's V, the season one-hot and
day of season), fit on A train day pairs, lambda chosen on A val, scored at A test's bench starts (bench std)."""
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import numpy as np
import bench_v0 as b
from wm1 import common as c, evaluate as ev
from wm1.m3_text import days_of
g = c.load_groups(["A_train", "A_val", "A_test"])
def xy(eps, rows_fn):
    X, Y, N = [], [], []
    for ep in eps:
        for r in rows_fn(ep):
            day = (r * b.EVERY + b.OFFSET) // b.TPD + 1
            x = np.concatenate([ep.bench.v[r], b.onehot(ep.bench.season[[r]], [-1, 0, 1, 2, 3])[0], [((day - 1) % 7 + 1) / 7.0]])
            X.append(x); Y.append(ep.bench.v[r + 24] - ep.bench.v[r]); N.append(ep.bench.v[r])
    return np.array(X), np.array(Y), np.array(N)
pairs = lambda ep: [ev.row_of(D, 360) for D in days_of(ep)]
starts = lambda ep: list(ev.bench_starts(ep, 24))
xtr, ytr, _ = xy(g["A_train"], pairs)
xva, yva, nva = xy(g["A_val"], starts)
xte, yte, nte = xy(g["A_test"], starts)
std_val = b.split_std([e.bench for e in g["A_val"]]); std = b.split_std([e.bench for e in g["A_test"]])
best = None
for lam in b.LAMBDAS:
    m = b.Ridge().fit(xtr, ytr, lam)
    s = float(np.nanmean(b.nmae(nva + m.predict(xva), nva + yva, std_val)))
    if best is None or s < best[0]: best = (s, lam, m)
pred = nte + best[2].predict(xte)
out = {"lambda": best[1], "val": best[0], "test_per_var": b.nmae(pred, nte + yte, std).tolist(),
       "test_mean": float(np.nanmean(b.nmae(pred, nte + yte, std))), "test_rounded_mean": float(np.nanmean(b.nmae(np.round(pred), nte + yte, std))),
       "n_train_pairs": int(len(xtr)), "n_test": int(len(xte))}
Path(sys.argv[1]).write_text(json.dumps(out, indent=1)) if len(sys.argv) > 1 else None
print(json.dumps(out))
