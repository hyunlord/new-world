#!/usr/bin/env python3
"""Reduce one tools/v8/perf.sh output directory to a small JSON summary.

natural: per repeat, the mean tick over the four worlds (sum of steady tick
time / steady ticks) and the mean of their p99 and p99.9; then the median over
repeats. quick / stress_v26: mean, p99, p99.9, max and the end fingerprint.

usage: perf_summary.py OUT_DIR                 print the summary JSON
       perf_summary.py --compare BASE.json CAND.json   print the comparison
           (candidate natural vs base natural; candidate quick vs base
           stress_v26, the same world under the same rules) and WARN when a
           mean rises more than 10 %.
"""
import json
import statistics
import sys
from pathlib import Path


def load(path):
    return json.loads(Path(path).read_text())


def runs(directory):
    return [load(p) for p in sorted(Path(directory).glob("*.json")) if p.name != "index.json" and "tick_summary" in load(p)]


def natural_repeat(directory):
    files = runs(directory)
    ticks = sum(f["tick_summary"]["ticks"] for f in files)
    total = sum(f["steady_tick_ms_total"] for f in files)
    return {
        "worlds": len(files),
        "mean_ms": total / ticks if ticks else 0.0,
        "p99_ms": statistics.mean(f["tick_summary"]["p99_ms"] for f in files),
        "p999_ms": statistics.mean(f["tick_summary"]["p999_ms"] for f in files),
        "fingerprints": [f["fingerprint"] for f in files],
    }


def single(directory):
    files = runs(directory)
    if len(files) != 1:
        return None
    f = files[0]
    t = f["tick_summary"]
    return {"file": next(p.name for p in sorted(Path(directory).glob("*.json")) if "tick_summary" in load(p)),
            "mean_ms": t["mean_ms"], "p99_ms": t["p99_ms"], "p999_ms": t["p999_ms"], "max_ms": t["max_ms"],
            "fingerprint": f["fingerprint"], "founders": f["founders"]}


def summarize(out):
    out = Path(out)
    repeats = [natural_repeat(d) for d in sorted(out.glob("natural_r*")) if d.is_dir()]
    natural = {"repeats": repeats}
    if repeats:
        for key in ("mean_ms", "p99_ms", "p999_ms"):
            natural[f"median_{key}"] = statistics.median(r[key] for r in repeats)
    summary = {"natural": natural}
    for name in ("quick", "stress_v26"):
        if (out / name).is_dir():
            summary[name] = single(out / name)
    runlog = out / "RUNLOG.txt"
    summary["runlog"] = runlog.read_text().splitlines() if runlog.exists() else []
    return summary


def compare(base, cand):
    lines, warn = [], False
    pairs = [("natural mean", base["natural"]["median_mean_ms"], cand["natural"]["median_mean_ms"]),
             ("natural p99", base["natural"]["median_p99_ms"], cand["natural"]["median_p99_ms"]),
             ("natural p99.9", base["natural"]["median_p999_ms"], cand["natural"]["median_p999_ms"]),
             ("quick (baseline stress) mean", base["stress_v26"]["mean_ms"], cand["quick"]["mean_ms"]),
             ("quick (baseline stress) p99", base["stress_v26"]["p99_ms"], cand["quick"]["p99_ms"])]
    for label, b, c in pairs:
        ratio = c / b if b else float("nan")
        flag = ""
        if "mean" in label and ratio > 1.10:
            flag, warn = "  WARN (> +10 %)", True
        lines.append(f"{label}: base {b:.4f} ms, candidate {c:.4f} ms, x{ratio:.3f}{flag}")
    same = base["stress_v26"]["fingerprint"] == cand["quick"]["fingerprint"]
    lines.append(f"quick fingerprint equal to base stress_v26: {same}")
    nat_same = [r["fingerprints"] for r in base["natural"]["repeats"]][:1] == [r["fingerprints"] for r in cand["natural"]["repeats"]][:1]
    lines.append(f"natural fingerprints equal (repeat 1): {nat_same}")
    lines.append("perf: WARN" if warn else "perf: OK (no mean above +10 %)")
    return "\n".join(lines)


def main(argv):
    if len(argv) == 4 and argv[1] == "--compare":
        print(compare(load(argv[2]), load(argv[3])))
        return 0
    if len(argv) != 2:
        print(__doc__.strip(), file=sys.stderr)
        return 2
    print(json.dumps(summarize(argv[1]), indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
