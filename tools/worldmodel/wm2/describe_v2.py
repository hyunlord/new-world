"""Dataset v2 description (GOAL.md 6.1, 6.5): what was recorded, counts, sizes, timings, the cast times of the factual
timeline, the share of worlds per arm and the per-arm true effect distribution on the **train** worlds (so the reader
knows what there is to predict; validation and test effects are not read here). numpy only.

usage (DGX): python3 tools/worldmodel/wm2/describe_v2.py --root ROOT --record-logs DIR --out JSON
"""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm2  # noqa: F401
from wm1 import common as c
from wm2 import effects as ef
from wm2 import v2


def du(path: Path) -> int:
    out = subprocess.run(["du", "-sbL", str(path)], capture_output=True, text=True).stdout.split()
    return int(out[0]) if out else 0


def timeline_stats(path: Path) -> dict:
    t = json.loads(path.read_text())
    per = t["per_seed"]
    low = [s for s in per if s["T_low"] is not None]
    low_days = np.array([int(v2.display_day(s["T_low"])) for s in low])
    out = {"seeds": len(per), "share": t["share"], "count": t["count"], "line_check": t["line_check"]}
    for role, (lo, hi) in v2.ROLES["game_v19"].items():
        rs = [s for s in per if lo <= s["seed"] < hi]
        out.setdefault("by_role", {})[role] = {"seeds": len(rs), "T_low": sum(s["T_low"] is not None for s in rs),
                                              "never_depart": sum(s["first_departure_tick"] is None for s in rs)}
    vals, counts = np.unique(low_days, return_counts=True)
    out["T_low_display_day"] = {"min": int(low_days.min()), "median": float(np.median(low_days)),
                                "max": int(low_days.max()), "histogram": {str(int(v)): int(n) for v, n in zip(vals, counts)}}
    fd = np.array([s["first_departure_day"] for s in per if s["first_departure_day"] is not None])
    out["first_departure_day"] = {"n": int(len(fd)), "min": int(fd.min()), "median": float(np.median(fd)),
                                  "max": int(fd.max())}
    return out


def record_timings(logs: Path) -> dict:
    out = {}
    for f in sorted(logs.glob("*.start")):
        name = f.stem
        end = logs / f"{name}.end"
        out[name] = {"start": int(f.read_text()), "end": int(end.read_text()) if end.exists() else None,
                     "exit": (logs / f"{name}.exit").read_text().strip() if (logs / f"{name}.exit").exists() else None}
        if out[name]["end"]:
            out[name]["seconds"] = out[name]["end"] - out[name]["start"]
    return out


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--root", default=str(c.DEFAULT_ROOT))
    ap.add_argument("--record-logs", required=True)
    ap.add_argument("--out", required=True)
    a = ap.parse_args(argv)
    t0 = time.time()
    root = Path(a.root)
    base = root / "v2"
    m = json.loads(c.V2_MANIFEST.read_text())
    out = {"manifest_episodes": m["episode_count"], "counts_by_split_role": m["counts_by_split_role"],
           "families_per_arm_role": m["families_per_arm_role"], "timeline": timeline_stats(base / "v19/timeline_out/timeline.json"),
           "bytes": {"v19_flat": du(base / "v19/flat"), "v16": du(base / "v16"), "v19_timeline_probe": du(base / "v19/timeline")},
           "record": record_timings(Path(a.record_logs))}
    n_flat = sum(1 for e in m["episodes"] if e["scene"] == "game_v19")
    out["mean_episode_bytes_v19"] = out["bytes"]["v19_flat"] / max(1, n_flat)
    applied = {}
    for e in m["episodes"]:
        for cast in e["casts"]:
            applied.setdefault(e["arm"], []).append(cast["applied"] / cast["amount"] if cast["amount"] else None)
    out["applied_share"] = {k: {"mean": float(np.mean(v)), "min": float(np.min(v)), "n": len(v)} for k, v in applied.items()}
    train, _ = ef.load_cases(root, "train")
    out["train_effects"] = ef.distribution(train)
    out["train_cases"] = len(train)
    out["code_commit"] = c.git_commit()
    out["seconds"] = time.time() - t0
    c.write_json(Path(a.out), out)
    print(json.dumps({k: out[k] for k in ("manifest_episodes", "families_per_arm_role", "train_cases")}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
