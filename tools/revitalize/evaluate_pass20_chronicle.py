#!/usr/bin/env python3
# How to run: python3 tools/revitalize/evaluate_pass20_chronicle.py --probe PROBE_DIR --chronicle CHRONICLE_DIR
#     [--twin TWIN_CHRONICLE_DIR] --scene game_v23 --seeds 19200..19232 --out-md OUT.md --out-json OUT.json
"""Pass 20 copy of Pass 19's C1-C5 (the chronicle against the probe's traces),
with the kind names 17-21 appended (Pass 19's tool indexes kinds by position
and would raise on them). Used for T7's "C1-C4 still pass on game_v25".

PROBE_DIR is a `worldsim_probe --agent-trace` run (family events, deaths
observer, fissions, people); CHRONICLE_DIR holds the same run's
`--chronicle-out` files; TWIN_CHRONICLE_DIR a second run of the same seeds
(C4 compares the files byte for byte). C6 (cost) is measured elsewhere.

Tick conventions: family events and the chronicle record the system tick;
the deaths observer records the completed tick (system tick + 1).
"""
from __future__ import annotations

import argparse
import hashlib
import json
import sys
from collections import Counter, defaultdict
from collections.abc import Iterable, Mapping
from pathlib import Path
from typing import Any, Final

KINDS: Final = ("born", "founding_member", "paired", "came_of_age", "became_elder", "departed",
    "departure_cancelled", "settled", "joined", "left_settlement", "widowed", "died", "settlement_formed",
    "fission_started", "daughter_founded", "fission_cancelled", "settlement_dissolved",
    # Pass 20 (`game_v25`): kinds 17-21.
    "porter_departed", "porter_delivered", "quarrel", "villages_estranged", "villages_reconciled")
K: Final = {name: index for index, name in enumerate(KINDS)}
CAUSES: Final = ("ranked", "seed", "household", "elder_partner", "top_up")
DEATH: Final = {"starvation": 0, "dehydration": 1, "combat": 2, "old_age": 3, "exposure": 4}
CHILD_TICKS: Final = 5760
ELDER_AGE_TICKS: Final = 66240
DAYS_PER_YEAR: Final = 28  # four seasons of seven days (`SeasonRule::days_per_season`)


def jsonl(path: Path) -> list[dict[str, Any]]:
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def seed_range(text: str) -> range:
    start, end = text.split("..")
    return range(int(start), int(end))


class Entry:
    __slots__ = ("tick", "kind", "subject", "other", "third", "settlement", "settlement2", "cause", "value")

    def __init__(self, row: list[Any]) -> None:
        (self.tick, self.kind, self.subject, self.other, self.third,
         self.settlement, self.settlement2, self.cause, self.value) = row


def matcher(entries: Iterable[Entry], kind: str, key) -> Counter:
    return Counter(key(e) for e in entries if e.kind == K[kind])


def compare(name: str, expected: Iterable[tuple], found: Counter, both_ways: bool = True) -> dict[str, Any]:
    """Every expected key must appear exactly once in `found`; with `both_ways`
    every found key must also be expected."""
    expected = Counter(expected)
    missing = [list(k) for k, n in expected.items() if found.get(k, 0) != n or n != 1]
    extra = [list(k) for k, n in found.items() if both_ways and expected.get(k, 0) == 0]
    return {"rule": name, "expected": sum(expected.values()), "found": sum(found.values()),
        "mismatches": len(missing) + len(extra), "examples": (missing + extra)[:5]}


def evaluate_seed(probe: Path, chronicle_dir: Path, scene: str, seed: int) -> dict[str, Any]:
    doc = json.loads((chronicle_dir / f"chronicle_{seed}.json").read_text())
    entries = [Entry(row) for row in doc["entries"]]
    people = {int(k): v for k, v in doc["people"].items()}
    events = jsonl(probe / f"{scene}_s{seed}_family_events.jsonl")
    deaths = jsonl(probe / f"{scene}_s{seed}_deaths.jsonl")
    fissions = jsonl(probe / f"{scene}_s{seed}_family_fissions.jsonl")
    traced = jsonl(probe / f"{scene}_s{seed}_people.jsonl")
    c1 = []
    born = [e for e in events if e["event"] == "born"]
    c1.append(compare("born", ((e["tick"], e["child"], e["mother"], e["father"]) for e in born),
        matcher(entries, "born", lambda e: (e.tick, e.subject, e.other, e.third))))
    c1.append(compare("paired", ((e["tick"], e["mother"], e["father"]) for e in events if e["event"] == "paired"),
        matcher(entries, "paired", lambda e: (e.tick, e.subject, e.other))))
    departed = [e for e in events if e["event"] == "departed"]
    c1.append(compare("departed_members",
        ((e["tick"], m, e["parent"], e["fission_id"], e["leader"]) for e in departed for m in e["members"]),
        matcher(entries, "departed", lambda e: (e.tick, e.subject, e.settlement, e.value, e.other))))
    c1.append(compare("deaths", ((d["tick"] - 1, d["agent_id"], DEATH.get(d["death_reason"], -1)) for d in deaths),
        matcher(entries, "died", lambda e: (e.tick, e.subject, e.cause))))
    c1.append(compare("fission_started", ((f["depart_tick"], f["parent"], f["fission_id"]) for f in fissions),
        matcher(entries, "fission_started", lambda e: (e.tick, e.settlement, e.value))))
    c1.append(compare("daughter_founded",
        ((f["founded_tick"], f["daughter"], f["parent"], f["fission_id"]) for f in fissions if f["founded_tick"] is not None),
        matcher(entries, "daughter_founded", lambda e: (e.tick, e.settlement, e.settlement2, e.value))))
    c1.append(compare("fission_cancelled",
        ((f["cancelled_tick"], f["parent"], f["fission_id"]) for f in fissions if f["cancelled_tick"] is not None),
        matcher(entries, "fission_cancelled", lambda e: (e.tick, e.settlement, e.value))))
    stage = {"came_of_age": 0, "became_elder": 0}
    stage_bad: list[list[int]] = []
    for e in entries:
        if e.kind in (K["came_of_age"], K["became_elder"]):
            name = KINDS[e.kind]
            stage[name] += 1
            born_tick = people.get(e.subject, {}).get("born_tick")
            threshold = CHILD_TICKS if name == "came_of_age" else ELDER_AGE_TICKS
            if born_tick is None or not 0 <= e.tick - (born_tick + threshold) < 60:
                stage_bad.append([e.kind, e.subject, e.tick, born_tick])
    c1.append({"rule": "stage_ticks", "expected": sum(stage.values()), "found": sum(stage.values()),
        "mismatches": len(stage_bad), "examples": stage_bad[:5], "counts": stage})
    # C2: causes sum to the departed members; ElderPartner = the trace's elders_departed.
    causes_by_fission: dict[int, Counter] = defaultdict(Counter)
    for e in entries:
        if e.kind == K["departed"]:
            causes_by_fission[e.value][CAUSES[e.cause]] += 1
    c2_bad = []
    elder_rows = []
    for f in departed:
        counts = causes_by_fission.get(f["fission_id"], Counter())
        if sum(counts.values()) != len(f["members"]):
            c2_bad.append(["members", f["fission_id"], sum(counts.values()), len(f["members"])])
        if counts["elder_partner"] != f["elders_departed"]:
            c2_bad.append(["elders", f["fission_id"], counts["elder_partner"], f["elders_departed"]])
        elder_rows.append({"fission_id": f["fission_id"], "members": len(f["members"]),
            "elders_departed": f["elders_departed"], "causes": dict(counts)})
    # C3: person table.
    c3_bad = []
    for person in traced:
        if person["id"] not in people:
            c3_bad.append(["no_record", person["id"]])
    for d in deaths:
        record = people.get(d["agent_id"])
        if record is None or record["died_tick"] != d["tick"] - 1 or record["death_reason"] != DEATH.get(d["death_reason"]):
            c3_bad.append(["death", d["agent_id"], d["tick"] - 1, None if record is None else record["died_tick"]])
    for b in born:
        record = people.get(b["child"])
        if record is None or record["mother"] != b["mother"] or record["father"] != b["father"]:
            c3_bad.append(["parents", b["child"]])
    alive_without_death = [i for i, p in people.items() if p["died_tick"] is not None and p["death_reason"] is None]
    c3_bad.extend(["died_without_reason", i] for i in alive_without_death)
    kinds = Counter(KINDS[e.kind] for e in entries)
    return {"seed": seed, "entries": len(entries), "people": len(people), "traced_people": len(traced),
        "kinds": dict(sorted(kinds.items())), "c1": c1, "c2": {"mismatches": len(c2_bad), "examples": c2_bad[:5],
        "fissions": elder_rows, "causes": dict(sum((c for c in causes_by_fission.values()), Counter()))},
        "c3": {"mismatches": len(c3_bad), "examples": c3_bad[:5], "records": len(people),
        "dead_records": sum(1 for p in people.values() if p["died_tick"] is not None)},
        "approx_bytes": doc["approx_bytes"], "entry_size_bytes": doc["entry_size_bytes"],
        "json_bytes": (chronicle_dir / f"chronicle_{seed}.json").stat().st_size, "ticks": doc["ticks"]}


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--probe", type=Path, required=True)
    parser.add_argument("--chronicle", type=Path, required=True)
    parser.add_argument("--twin", type=Path)
    parser.add_argument("--scene", default="game_v25")
    parser.add_argument("--seeds", type=seed_range, required=True)
    parser.add_argument("--out-md", type=Path, required=True)
    parser.add_argument("--out-json", type=Path, required=True)
    args = parser.parse_args(argv)
    summary = json.loads((args.probe / "summary.json").read_text())
    agent_days = {run["seed"]: run["agent_days"] for run in summary["runs"]}
    seeds = [evaluate_seed(args.probe, args.chronicle, args.scene, seed) for seed in args.seeds]
    c1_rows: dict[str, dict[str, int]] = {}
    for result in seeds:
        for row in result["c1"]:
            agg = c1_rows.setdefault(row["rule"], {"expected": 0, "found": 0, "mismatches": 0})
            for key in ("expected", "found", "mismatches"):
                agg[key] += row[key]
    c1_total = sum(r["mismatches"] for r in c1_rows.values())
    c2_total = sum(r["c2"]["mismatches"] for r in seeds)
    c3_total = sum(r["c3"]["mismatches"] for r in seeds)
    twin = None
    if args.twin is not None:
        differing = [s for s in args.seeds if sha(args.chronicle / f"chronicle_{s}.json") != sha(args.twin / f"chronicle_{s}.json")]
        twin = {"files": len(args.seeds), "differing": differing}
    causes = sum((Counter(r["c2"]["causes"]) for r in seeds), Counter())
    person_days = sum(agent_days[s] for s in args.seeds)
    entries = sum(r["entries"] for r in seeds)
    approx = sum(r["approx_bytes"] for r in seeds)
    json_bytes = sum(r["json_bytes"] for r in seeds)
    per_pd = {"entries": entries / person_days, "approx_bytes": approx / person_days, "json_bytes": json_bytes / person_days}
    projection_pd = 10_000 * 10 * DAYS_PER_YEAR
    c5 = {"person_days": person_days, "entries": entries, "approx_bytes": approx, "json_bytes": json_bytes,
        "entry_size_bytes": seeds[0]["entry_size_bytes"], "per_person_day": per_pd,
        "projection": {"people": 10_000, "years": 10, "days_per_year": DAYS_PER_YEAR, "person_days": projection_pd,
            "entries": per_pd["entries"] * projection_pd, "approx_bytes": per_pd["approx_bytes"] * projection_pd,
            "json_bytes": per_pd["json_bytes"] * projection_pd}}
    kinds = sum((Counter(r["kinds"]) for r in seeds), Counter())
    verdict = {"C1": "PASS" if c1_total == 0 else "FAIL", "C2": "PASS" if c2_total == 0 else "FAIL",
        "C3": "PASS" if c3_total == 0 else "FAIL",
        "C4": None if twin is None else ("PASS" if not twin["differing"] else "FAIL")}
    out = {"scene": args.scene, "seeds": [args.seeds.start, args.seeds.stop], "verdict": verdict,
        "c1": c1_rows, "c1_mismatches": c1_total, "c2_mismatches": c2_total, "c3_mismatches": c3_total,
        "c4": twin, "c5": c5, "depart_causes": dict(causes), "kinds": dict(sorted(kinds.items())), "per_seed": seeds}
    args.out_json.write_text(json.dumps(out, indent=1, sort_keys=True) + "\n")
    lines = [f"# Pass 19 chronicle checks C1-C5 on Pass 20 data ({args.scene}, seeds {args.seeds.start}–{args.seeds.stop - 1})", "",
        "| rule | verdict | detail |", "|---|---|---|"]
    lines.append(f"| C1 completeness | {verdict['C1']} | mismatches {c1_total}; " + "; ".join(
        f"{name} {row['expected']}/{row['found']}" for name, row in c1_rows.items()) + " (expected/found) |")
    lines.append(f"| C2 causes | {verdict['C2']} | mismatches {c2_total}; causes " + ", ".join(
        f"{k} {causes.get(k, 0)}" for k in CAUSES) + " |")
    lines.append(f"| C3 person table | {verdict['C3']} | mismatches {c3_total}; records {sum(r['people'] for r in seeds)}, "
        f"dead {sum(r['c3']['dead_records'] for r in seeds)}, traced people {sum(r['traced_people'] for r in seeds)} |")
    lines.append(f"| C4 determinism | {verdict['C4']} | " + ("not run" if twin is None else
        f"{twin['files'] - len(twin['differing'])}/{twin['files']} files byte-identical") + " |")
    lines.append(f"| C5 size | reported | {entries} entries, {person_days:.0f} person-days: "
        f"{per_pd['entries']:.4f} entries, {per_pd['approx_bytes']:.1f} B (memory est.), {per_pd['json_bytes']:.1f} B (JSON) per person-day; "
        f"10,000 people × 10 years ({projection_pd:,} person-days): {c5['projection']['entries']:,.0f} entries, "
        f"{c5['projection']['approx_bytes'] / 1e6:,.1f} MB memory, {c5['projection']['json_bytes'] / 1e6:,.1f} MB JSON |")
    lines += ["", "Entries by kind: " + ", ".join(f"{k} {v}" for k, v in sorted(kinds.items())), ""]
    args.out_md.write_text("\n".join(lines) + "\n")
    print("\n".join(lines))
    return 0 if c1_total == c2_total == c3_total == 0 and (twin is None or not twin["differing"]) else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
