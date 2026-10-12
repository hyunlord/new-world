"""Golden values of the v0 data path on the committed sample (GOAL_1_2.md 4.1). numpy only.

`python3 tools/worldmodel/wm1/golden_v0.py --write` computes them (run once, at the merge commit b20a991f, before any
`data.py` edit) into `golden_v0.json`; `test_golden_v0.py` recomputes them and asserts bit-identity, so extending the
feature table for recorder schemas 2-4 cannot change a v0 number.
"""
from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm1  # noqa: F401
from wm_data import load_episode
from wm1 import data as d
from wm1 import evaluate as ev

SAMPLE = Path(__file__).resolve().parents[3] / "docs/worldmodel/sample"
TARGET = Path(__file__).resolve().parent / "golden_v0.json"


def _sha(a: np.ndarray) -> str:
    a = np.ascontiguousarray(a)
    return hashlib.sha256(str(a.dtype).encode() + str(a.shape).encode() + a.tobytes()).hexdigest()


def compute() -> dict:
    eps = [d.from_episode(load_episode(SAMPLE / f"game_v13_s{s}_none", arrays=["global.npy", "settlements.npy"]),
                          "A_obs", role, s, "none") for s, role in ((0, "train"), (1, "val"))]
    out: dict = {"episodes": {}}
    for ep in eps:
        bench = {k: _sha(getattr(ep.bench, k)) for k in ("v", "tick", "hour", "season", "severity", "actions",
                                                          "wood_stock", "members", "cold_hours", "deaths")}
        out["episodes"][str(ep.seed)] = {"table": _sha(ep.table), "calendar": _sha(ep.calendar), "bench": bench,
                                         "rows": ep.rows}
    for v in d.VARIANTS.values():
        norm = d.Normalizer.fit(eps[:1], v)
        p = d.prepare(eps[1], v, norm)
        out.setdefault("variants", {})[v.name] = {
            "input_names": list(v.input_names()), "x_state": _sha(p.x_state), "dtarget": _sha(p.dtarget),
            "fire_next": _sha(p.fire_next), "norm_mean": _sha(norm.mean), "norm_dstd": _sha(norm.dstd)}
    std = ev.b.split_std([eps[1].bench])
    pers = ev.Persistence()
    out["harness"] = {"persistence_T1_val": repr(ev.t1(pers, eps[1:], std)["mean"]),
                      "persistence_T2_24_any_val": repr(ev.t2(pers, eps[1:], std, 24, any_hour=True)["mean"]),
                      "persistence_T2_168_0600_val": repr(ev.t2(pers, eps[1:], std, 168)["mean"])}
    return out


def compute_more() -> dict:
    """Review round 1 (WM-1.2): harness numbers of the severity-free ridge and three calendar members on the sample
    (fit on seed 0, scored on seed 1; T2 24 h at every-hour starts, whose targets stay inside the 10-day library).
    Recorded from the merge commit's code (`golden_v0.json` key `harness_more`)."""
    from wm1 import climatology as cl
    eps = [d.from_episode(load_episode(SAMPLE / f"game_v13_s{s}_none", arrays=["global.npy", "settlements.npy"]),
                          "A_obs", role, s, "none") for s, role in ((0, "train"), (1, "val"))]
    std = ev.b.split_std([eps[1].bench])
    ridge, lam, _ = ev.fit_ridge_without_severity(eps[:1], eps[1:], action_aware=False)
    fit = cl.CalendarFit(eps[:1])
    out = {"ridge_nosev_lambda": repr(lam),
           "ridge_nosev_T1_val": repr(ev.t1(ridge, eps[1:], std)["mean"]),
           "ridge_nosev_T2_24_any_val": repr(ev.t2(ridge, eps[1:], std, 24, any_hour=True)["mean"])}
    for m in ("cal_abs", "apfit_abs", "an1"):
        out[f"{m}_T2_24_any_val"] = repr(ev.t2(cl.CalendarPredictor(fit, m), eps[1:], std, 24, any_hour=True)["mean"])
    return out


if __name__ == "__main__":
    if "--write" not in sys.argv[1:]:
        raise SystemExit("usage: golden_v0.py --write")
    TARGET.write_text(json.dumps(compute(), indent=1, sort_keys=True) + "\n")
    print(f"wrote {TARGET}")
