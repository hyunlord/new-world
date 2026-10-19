#!/usr/bin/env python3
# How to run: python3 tools/revitalize/grid_p21.py --grid-dir GRID_DIR --out-md OUT.md --out-json OUT.json
"""Pass 21 grid (brief 7.2): the twelve pre-registered configurations and the
selection rule, applied exactly.

Configurations in the brief's order: quarrel_loss in {40, 60} (outer), drift
in {30, 20} permille (middle), (aid giver, aid receiver) in {(3000, 2000),
(2500, 2000), (3000, 2500)} (inner); keep 2,000 permille, the v26 flags and
caps, dispute_loss 5 always. `GRID_DIR/G<n>` is each configuration's probe
run and `GRID_DIR/chronicle_G<n>` its `--chronicle-out` directory.

A multi-village world is one with >= 2 villages alive at once in that
configuration's run. Selection: among configurations meeting all of
(estranged share of pairs at day 120 <= 0.25; quarrels in >= 0.15 of
multi-village worlds; if >= 3 estrangements, >= 0.5 of them reconciled by day
120), the highest share of multi-village worlds with >= 1 delivered trip,
ties to the earlier configuration. If none qualifies: the most delivering
worlds among configurations with estranged share <= 0.5 (ties earlier).
"""
from __future__ import annotations

import argparse
import json
import random
import sys
from pathlib import Path
from typing import Any, Final

sys.path.insert(0, str(Path(__file__).resolve().parent))
from diag_p21 import num, read_run, summarize  # noqa: E402

CONFIGS: Final = [(q, drift, giver, receiver)
                  for q in (40, 60) for drift in (30, 20) for giver, receiver in ((3000, 2000), (2500, 2000), (3000, 2500))]


def qualifies(r: dict[str, Any]) -> tuple[bool, list[str]]:
    why = []
    share = r["estranged_share_at_end"]
    if share is not None and share > 0.25:
        why.append(f"estranged share {share:.3f} > 0.25")
    if (r["share_quarrel"] or 0) < 0.15:
        why.append(f"quarrel share {num(r['share_quarrel'])} < 0.15")
    if r["estrangements"] >= 3 and (r["reconciled_share"] or 0) < 0.5:
        why.append(f"reconciled {r['reconciliations']}/{r['estrangements']} < 0.5")
    return not why, why


def select(reports: list[dict[str, Any]]) -> dict[str, Any]:
    ok = [r for r in reports if r["qualifies"]]
    if ok:
        best = max(ok, key=lambda r: (r["share_delivering"] or 0, -r["index"]))
        return {"label": best["label"], "rule": "all three bars met; highest delivering share, ties earlier", "fallback": False}
    pool = [r for r in reports if r["estranged_share_at_end"] is None or r["estranged_share_at_end"] <= 0.5]
    if not pool:
        return {"label": None, "rule": "no configuration has estranged share <= 0.5", "fallback": True}
    best = max(pool, key=lambda r: (sum(1 for w in r["worlds"] if w["multi_village"] and w["delivered_trips"] > 0), -r["index"]))
    return {"label": best["label"], "rule": "fallback: most delivering worlds among estranged share <= 0.5, ties earlier", "fallback": True}


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--grid-dir", type=Path, required=True)
    parser.add_argument("--out-md", type=Path, required=True)
    parser.add_argument("--out-json", type=Path, required=True)
    parser.add_argument("--model-seed", type=int, default=21)
    args = parser.parse_args(argv)
    rng = random.Random(args.model_seed)
    reports = []
    for index, (q, drift, giver, receiver) in enumerate(CONFIGS, start=1):
        label = f"G{index}"
        data = read_run(args.grid_dir / label, args.grid_dir / f"chronicle_{label}")
        r = summarize(label, data, {"drift": drift, "dispute_loss": 5, "quarrel_loss": q, "dispute_cap": 3, "quarrel_cap": 1,
                                    "estrange_at": -300, "reconcile_at": -150}, rng, 120)
        r.update({"index": index, "config": {"quarrel_loss": q, "drift_permille": drift, "aid_giver_days_permille": giver,
                                              "aid_receiver_days_permille": receiver, "aid_keep_days_permille": 2000}})
        r["qualifies"], r["why_not"] = qualifies(r)
        r["delivering_worlds"] = sum(1 for w in r["worlds"] if w["multi_village"] and w["delivered_trips"] > 0)
        r.pop("model")
        reports.append(r)
    choice = select(reports)
    lines = ["# Pass 21 grid (7.2): G1–G12 on 19500–19515, 120 days", "",
        "Configurations in the brief's pre-registered order; each run is `game_v26` with the configuration's quarrel loss, drift and "
        "aid thresholds (keep 2,000 ‰, dispute loss 5, caps 3 / 1, avoid and exchange first always). Multi-village = ≥ 2 villages alive "
        "at once in that run. Bars: estranged share at day 120 ≤ 0.25; quarrels in ≥ 0.15 of multi-village worlds; with ≥ 3 "
        "estrangements, ≥ 0.5 reconciled.", "",
        "| config | quarrel loss | drift ‰ | giver / receiver ‰ | multi-village | delivering share (worlds) | dispute share | quarrel share | estranged at d120 | estrangements | reconciled | quarrels while estranged | porters | exchange porters | aid-food pair-days (not estranged) | daughter deaths | qualifies |",
        "|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|"]
    for r in reports:
        c = r["config"]
        lines.append(f"| {r['label']} | {c['quarrel_loss']} | {c['drift_permille']} | {c['aid_giver_days_permille']} / {c['aid_receiver_days_permille']} | "
            f"{r['multi_village_worlds']} | {num(r['share_delivering'])} ({r['delivering_worlds']}) | {num(r['share_dispute'])} | {num(r['share_quarrel'])} | "
            f"{r['estranged_at_end']} of {r['pairs_at_end']} ({num(r['estranged_share_at_end'])}) | {r['estrangements']} | {r['reconciliations']} ({num(r['reconciled_share'])}) | "
            f"{r['quarrels_while_estranged']} of {r['quarrels']} | {r['trips']} | {r['exchange_porters']} | {r['aid_food_in_reach']} ({r['aid_food_in_reach_not_estranged']}) | "
            f"{r['daughter_deaths']} | {'yes' if r['qualifies'] else 'no: ' + '; '.join(r['why_not'])} |")
    lines += ["", f"**Selected: {choice['label']}** ({choice['rule']}).", ""]
    for r in reports:
        r.pop("worlds")
    args.out_json.write_text(json.dumps({"configs": reports, "selection": choice}, indent=1, sort_keys=True) + "\n")
    args.out_md.write_text("\n".join(lines) + "\n")
    print("\n".join(lines))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
