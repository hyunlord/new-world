#!/usr/bin/env python3
"""Z6 check (PASS7_GOAL.md section 11): prefix identity of the god's hand.

usage: check_z6.py --data DIR --out DIR [--tick 31920]

DIR holds `<scene>_s<seed>_<kind>` episodes and their factual `<scene>_s<seed>_none`
(recorded by `make_dataset.py z6`). For every seed and kind: `row_digest` must equal
the factual one at every row with tick <= --tick, and differ at some later row
whenever the intervention applied something; a kind that applied 0 in a seed is
listed, not failed. Writes z6.json and z6.md. Exit 0 when every verdict passes.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

import numpy as np

from wm_data import load_episode

KINDS = ("gift_food", "spoil_food", "gift_wood", "rot_wood", "rain", "cold_snap", "windfall")


def compare(factual_digest: np.ndarray, digest: np.ndarray, ticks: np.ndarray, tick: int, applied: int) -> dict:
    """Verdict of one (seed, kind) pair."""
    if factual_digest.shape != digest.shape:
        return {"verdict": "FAIL", "reason": "row counts differ", "first_differing_row": None}
    differs = np.nonzero(factual_digest != digest)[0]
    first = int(differs[0]) if differs.size else None
    prefix_ok = first is None or int(ticks[first]) > tick
    if not prefix_ok:
        return {"verdict": "FAIL", "reason": f"differs at tick {int(ticks[first])} <= {tick}", "first_differing_row": first,
                "first_differing_tick": int(ticks[first])}
    if applied == 0:
        verdict = "APPLIED_0" if first is None else "FAIL"
        return {"verdict": verdict, "reason": "applied 0" + ("" if first is None else " but the run differs"),
                "first_differing_row": first}
    if first is None:
        return {"verdict": "FAIL", "reason": "applied > 0 but no later row differs", "first_differing_row": None}
    return {"verdict": "PASS", "reason": "", "first_differing_row": first, "first_differing_tick": int(ticks[first])}


def check(data: Path, tick: int) -> dict:
    pattern = re.compile(r"^(?P<scene>[a-z0-9_]+?)_s(?P<seed>\d+)_(?P<id>[A-Za-z0-9_-]+)$")
    episodes: dict[int, dict[str, Path]] = {}
    for path in sorted(data.iterdir()):
        match = pattern.match(path.name)
        if match and (path / "meta.json").exists():
            episodes.setdefault(int(match["seed"]), {})[match["id"]] = path
    rows = []
    for seed in sorted(episodes):
        factual_path = episodes[seed].get("none")
        if factual_path is None:
            raise ValueError(f"seed {seed} has no factual episode")
        factual = load_episode(factual_path, arrays=["row_digest.npy", "global.npy"])
        ticks = factual.column("global.npy", "tick")
        for kind in KINDS:
            path = episodes[seed].get(kind)
            if path is None:
                rows.append({"seed": seed, "kind": kind, "applied": None, "verdict": "FAIL", "reason": "episode missing",
                             "first_differing_row": None})
                continue
            ep = load_episode(path, arrays=["row_digest.npy"])
            applied = sum(int(r["applied"]) for r in ep.interventions)
            result = compare(factual.arrays["row_digest.npy"], ep.arrays["row_digest.npy"], ticks, tick, applied)
            rows.append({"seed": seed, "kind": kind, "applied": applied, **result})
    passed = bool(rows) and all(r["verdict"] in ("PASS", "APPLIED_0") for r in rows)
    return {"tick": tick, "passed": passed, "rows": rows,
            "applied_zero": [f"s{r['seed']}:{r['kind']}" for r in rows if r["verdict"] == "APPLIED_0"],
            "counts": {v: sum(1 for r in rows if r["verdict"] == v) for v in ("PASS", "APPLIED_0", "FAIL")}}


def markdown(report: dict) -> str:
    lines = [f"# Z6 — the god's hand (intervention at tick {report['tick']})", "",
             f"Verdict: **{'PASS' if report['passed'] else 'FAIL'}** — {report['counts']}.",
             "Rows equal to the factual `row_digest` at every tick <= the intervention tick, and different at some later row "
             "whenever `applied > 0`. Kinds that applied 0 are listed, not failed.", "",
             "| Seed | Kind | Applied | First differing row (tick) | Verdict |", "|---:|---|---:|---|---|"]
    for r in report["rows"]:
        first = "–" if r["first_differing_row"] is None else f"{r['first_differing_row']} ({r.get('first_differing_tick', '?')})"
        lines.append(f"| {r['seed']} | {r['kind']} | {r['applied']} | {first} | {r['verdict']}{' — ' + r['reason'] if r['reason'] else ''} |")
    if report["applied_zero"]:
        lines += ["", "Applied 0: " + ", ".join(report["applied_zero"])]
    return "\n".join(lines) + "\n"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--data", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("--tick", type=int, default=31920)
    args = parser.parse_args(argv)
    try:
        report = check(args.data, args.tick)
    except (OSError, ValueError, KeyError) as exc:
        print(f"INVALID EVIDENCE: {exc}", file=sys.stderr)
        return 2
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "z6.json").write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    (args.out / "z6.md").write_text(markdown(report))
    print("PASS" if report["passed"] else "FAIL", report["counts"])
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
