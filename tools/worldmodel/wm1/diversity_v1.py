"""WM-1.1 Part 5: seed diversity and calendar-family baselines on dataset v1 (GOAL_1_1.md 11). numpy only.

Runs against Pass 8's code in a separate read-only clone (`--v1-tools <v1src>/tools/worldmodel`): this file puts that
directory first on `sys.path` before anything else and imports `bench_v1` / `wm_data` (recorder schema 2) from it.
It never imports the `wm1` package (whose `__init__` puts this branch's schema-1 `wm_data` on the path); the array
math comes from `calcore.py`, loaded by file path. Never import this module from a unittest process.

usage (on the DGX):
  python3 tools/worldmodel/wm1/diversity_v1.py --v1-tools V1SRC/tools/worldmodel --out RUN_DIR --val-only  # first
  python3 tools/worldmodel/wm1/diversity_v1.py --v1-tools V1SRC/tools/worldmodel --out RUN_DIR   # after METRICS_1_1
  python3 tools/worldmodel/wm1/diversity_v1.py --self-test        # calcore on synthetic arrays, no data needed
"""
import sys
from pathlib import Path


def _arg(argv, name, default):
    for i, a in enumerate(argv):
        if a == name and i + 1 < len(argv):
            return argv[i + 1]
        if a.startswith(name + "="):
            return a.split("=", 1)[1]
    return default


V1_TOOLS = str(Path(_arg(sys.argv[1:], "--v1-tools", "/home/hyunlord/work/wm1-runs/_build/v1src/tools/worldmodel")).resolve())
sys.path.insert(0, V1_TOOLS)
SELF_TEST = "--self-test" in sys.argv[1:]
if not SELF_TEST:
    import bench_v1 as bv   # noqa: E402
    import wm_data          # noqa: E402
    assert Path(wm_data.__file__).resolve().parent == Path(V1_TOOLS), wm_data.__file__
    assert Path(bv.__file__).resolve().parent == Path(V1_TOOLS), bv.__file__

import argparse  # noqa: E402
import hashlib  # noqa: E402
import importlib.util  # noqa: E402
import json  # noqa: E402
import os  # noqa: E402
import subprocess  # noqa: E402
import time  # noqa: E402

import numpy as np  # noqa: E402

_spec = importlib.util.spec_from_file_location("wm1_calcore", Path(__file__).resolve().parent / "calcore.py")
cc = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(cc)

REPO = Path(__file__).resolve().parents[3]
GROUPS = {"A_train": ("A_obs", "train"), "A_val": ("A_obs", "val"), "A_test": ("A_obs", "test"),
          "B_val": ("B_act", "val"), "B_test": ("B_act", "test"), "OOD_v13": ("OOD_v13", "test"),
          "OOD_long": ("OOD_long", "test")}
DIAG = ("A_train", "A_val", "A_test", "B_test", "OOD_v13", "OOD_long")
VAL = ("A_val", "B_val")
TEST = ("A_test", "B_test", "OOD_v13", "OOD_long")
K_MAX, T_TR, LAST_WINTER_DAY = 168, 2881, 218
# Task -> (horizon, every_hour); T1 is the next hour from every row 23 .. T-2 (bench_v1.t1_scores).
TASKS = {"T1": (1, None), "T2_24_every_hour": (24, True), "T2_24_0600": (24, False),
         "T2_168_every_hour": (168, True), "T2_168_0600": (168, False)}


class Fit:
    """Train-only calendar statistics from v1 A_train (arrays [E, T, 8])."""

    def __init__(self, lib: np.ndarray, std: np.ndarray):
        self.lib = lib
        self.m_abs, self.m_year = cc.calendar_means(lib)
        rows = np.arange(lib.shape[1])
        self.alpha = {k: cc.fit_alpha(lib, cc.mean_at(k, rows, self.m_abs, self.m_year), K_MAX) for k in ("abs", "year")}
        self.std = std
        self.inv_std = np.where(std > 0, 1.0 / np.where(std > 0, std, 1.0), 1.0)

    def predict(self, member: str, v: np.ndarray, starts: np.ndarray, horizon: int) -> np.ndarray:
        """Predicted V at row s + horizon for every start."""
        return cc.predict_member(member, v, starts, horizon, self.lib, self.m_abs, self.m_year, self.alpha,
                                 self.inv_std)[:, -1]


def task_nmae(fit: Fit, member: str, eps: list, std: np.ndarray, task: str, nmae) -> np.ndarray:
    horizon, every = TASKS[task]
    preds, trues = [], []
    for ep in eps:
        n = len(ep.tick)
        starts = (np.arange(23, n - 1) if every is None
                  else np.array(bv.rollout_starts(n, horizon, every), dtype=np.int64))
        if not len(starts):
            continue
        preds.append(fit.predict(member, ep.v, starts, horizon))
        trues.append(ep.v[starts + horizon])
    return nmae(np.vstack(preds), np.vstack(trues), std)


def severity_sequence(ep) -> tuple:
    out = []
    for day in cc.winter_days(len(ep.tick), LAST_WINTER_DAY):
        r = cc.row_of(day, 1080)
        if int(ep.season[r]) != 3 or int(ep.hidden_severity[r]) < 0:
            raise ValueError(f"seed {ep.seed} day {day}: season {ep.season[r]}, severity {ep.hidden_severity[r]}")
        out.append(int(ep.hidden_severity[r]))
    return tuple(out)


def git_commit() -> str:
    if os.environ.get("WM1_CODE_COMMIT"):
        return os.environ["WM1_CODE_COMMIT"]
    try:
        return subprocess.run(["git", "-C", str(REPO), "rev-parse", "HEAD"], capture_output=True, text=True,
                              check=True).stdout.strip()
    except (OSError, subprocess.CalledProcessError):
        return "unknown"


def per(arr, names) -> dict:
    return {n: (None if not np.isfinite(x) else float(x)) for n, x in zip(names, arr)}


def run(args) -> int:
    t0 = time.time()
    src = Path(V1_TOOLS).parents[1]
    manifest_path = src / "docs/worldmodel/dataset_v1_manifest.json"
    manifest = json.loads(manifest_path.read_text())
    data = {s: bv.load_split(Path(args.root), manifest, s) for s in ("A_obs", "B_act", "OOD_v13", "OOD_long")}
    groups = {name: [e for e in data[split] if e.role == role] for name, (split, role) in GROUPS.items()}
    names = tuple(bv.VARS)
    train = groups["A_train"]
    if {len(e.tick) for e in train} != {T_TR}:
        raise SystemExit(f"v1 A_train rows {sorted({len(e.tick) for e in train})} != {T_TR}")
    for e in train:
        if not np.array_equal(e.tick, 60 * np.arange(T_TR)):
            raise SystemExit(f"seed {e.seed}: tick != 60 * arange(rows)")
    lib = np.stack([e.v for e in train]).astype(np.float64)
    fit = Fit(lib, bv.split_std(train))
    if not np.isfinite(fit.m_year).all():
        raise SystemExit("m_year leaves a year position uncovered")
    out = {"dataset": "v1", "vars": list(names), "v1_tools": V1_TOOLS,
           "v1_commit": subprocess.run(["git", "-C", str(src), "rev-parse", "HEAD"], capture_output=True,
                                       text=True).stdout.strip(),
           "manifest_sha256": hashlib.sha256(manifest_path.read_bytes()).hexdigest(),
           "counts": {k: len(v) for k, v in groups.items()}, "groups": {}, "code_commit": git_commit(), "argv": sys.argv,
           "alpha": {k: {str(i): per(a[i], names) for i in (1, 24, 168)} for k, a in fit.alpha.items()}}
    for name in DIAG:
        eps = groups[name]
        rows = {len(e.tick) for e in eps}
        if len(rows) != 1:
            raise SystemExit(f"{name}: episodes differ in length {sorted(rows)}")
        v = np.stack([e.v for e in eps]).astype(np.float64)
        d = cc.split_diagnostics(v, fit.m_abs, fit.m_year, bv.split_std(eps), [severity_sequence(e) for e in eps],
                                 names)
        d["in_sample"], d["seeds"] = name == "A_train", [int(e.seed) for e in eps]
        out["groups"][name] = d
        print(name, len(eps), f"{time.time() - t0:.0f}s", flush=True)
    stds = {n: bv.split_std(groups[n]) for n in VAL + TEST}
    val = {m: {t: {s: per(task_nmae(fit, m, groups[s], stds[s], t, bv.nmae), names) for s in VAL} for t in TASKS}
           for m in cc.MEMBERS}
    mean = lambda d: float(np.nanmean([x for x in d.values() if x is not None]))
    sel = {}
    for t in TASKS:
        score = {m: float(np.mean([mean(val[m][t][s]) for s in VAL])) for m in cc.MEMBERS}
        sel[t] = {"cal_best": min(cc.MEMBERS, key=lambda m: (score[m], cc.MEMBERS.index(m))),
                  "cal_best_fair": min(cc.FAIR, key=lambda m: (score[m], cc.MEMBERS.index(m))), "val_scores": score}
    out["val"], out["selection"] = val, sel
    print("selection", {t: (s["cal_best"], s["cal_best_fair"]) for t, s in sel.items()}, flush=True)
    if args.val_only:
        return _write(out, Path(args.out) / "diversity_v1_val.json", t0)
    out["test"] = {m: {t: {s: per(task_nmae(fit, m, groups[s], stds[s], t, bv.nmae), names) for s in TEST}
                       for t in TASKS} for m in cc.MEMBERS}
    bench = json.loads((src / "docs/worldmodel/bench_v1.json").read_text())
    ref = {"T1": {s: {k: float(np.nanmean(v)) for k, v in bench["T1"][s].items()} for s in bench["T1"]}}
    for h in ("24", "168"):
        for mode, tag in (("every_hour", "every_hour"), ("0600_only", "0600")):
            ref[f"T2_{h}_{tag}"] = {s: {k: float(np.nanmean(v)) for k, v in d.items()}
                                   for s, d in bench["T2"][h][mode].items()}
    out["bench_v1_reference"] = ref
    return _write(out, Path(args.out) / "diversity_v1.json", t0)


def _write(out: dict, target: Path, t0: float) -> int:
    out["seconds"] = time.time() - t0
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(json.dumps(out, indent=1, sort_keys=True, default=lambda x: x.item() if hasattr(x, "item")
                                 else x.tolist()) + "\n")
    print(f"done {time.time() - t0:.0f}s -> {target}")
    return 0


def self_test() -> int:
    """calcore on synthetic arrays (no data): eta^2, the wrap rule and an analog lookup."""
    rng = np.random.default_rng(0)
    v = np.repeat(rng.standard_normal((1, 800, 8)), 4, axis=0)
    assert np.allclose(cc.eta2(v), 0.0)
    noise = 3.0 + rng.standard_normal((50, 1000, 8))
    assert abs(float(cc.eta2(noise).mean()) - 49 / 50) < 0.02
    for r in (2881, 3552, 3553, 5760):
        w = int(cc.wrap_row(r, 2881))
        assert cc.year_pos(w) == cc.year_pos(r) and 2881 - 672 <= w < 2881
    lib = rng.standard_normal((10, 800, 8))
    near, delta = cc.analog_neighbours(lib[7] + 1e-3, np.array([100, 500]), 24, lib, np.ones(8), 5)
    assert (near[:, 0] == 7).all() and (delta == 0).all()
    print("self-test OK")
    return 0


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--v1-tools", default=V1_TOOLS)
    ap.add_argument("--root", default="/home/hyunlord/work/worldsim-wm-data")
    ap.add_argument("--out")
    ap.add_argument("--self-test", action="store_true")
    ap.add_argument("--val-only", action="store_true", help="diagnostics, val scores and the selection; no test roles")
    args = ap.parse_args(argv)
    if args.self_test:
        return self_test()
    if not args.out:
        ap.error("--out is required")
    return run(args)


if __name__ == "__main__":
    raise SystemExit(main())
