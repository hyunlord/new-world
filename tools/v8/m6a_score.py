"""Recompute frozen M6a bars from per-tick events and daily physical snapshots."""
from __future__ import annotations

import argparse
from dataclasses import asdict, dataclass
from pathlib import Path

from m6a_protocol import recorded, check_freeze
from m6a_causal import audit as causal_audit
from m6a_evidence import EvidenceError, Node, Reference, reference_json, write_json


@dataclass(frozen=True, slots=True)
class WorldScore:
    seed: int
    alive: bool
    hunger_free_winters: bool
    stores: bool
    cooked_share: bool
    diversity: bool
    spoilage: bool
    ledger: bool
    stockpile_day1: bool
    beds_day10: bool
    wolf_deaths_le1: bool
    cooked_events: int
    meal_events: int
    median_groups_day36: int
    summer_daily_medians: dict[int, int]
    causal: dict
    harvested_units: int
    harvested_edible_units: int
    harvested_convertible_units: int
    spoiled_units: int
    winter42_alive: bool
    winter66_alive: bool
    winter42_store: bool
    winter66_store: bool


def files(world: Node) -> dict[str, Reference]:
    refs = [Reference.parse(row) for row in world.get("files").rows()]
    names = [Path(ref.path).name for ref in refs]
    if len(names) != len(set(names)):
        raise EvidenceError("ambiguous output file names")
    return dict(zip(names, refs, strict=True))


def receipt(path: Path, arm: str, purpose: str) -> tuple[Node, ...]:
    value = Node.load(path)
    if value.get("schema_version").integer() != 1 or value.get("kind").text() != "m6a_cohort_v1":
        raise EvidenceError("unsupported cohort receipt")
    if value.get("arm").text() != arm or value.get("purpose").text() != purpose:
        raise EvidenceError("wrong cohort arm or purpose")
    if value.get("days").integer() != 72 or value.get("exit_code").integer() or not value.get("source_unchanged").boolean():
        raise EvidenceError("incomplete or changed cohort")
    for key in ("binary", "observer", "scorer"):
        Reference.parse(value.get(key))
    if purpose == "heldout":
        frozen = Reference.parse(value.get("freeze")).read()
        if recorded(value).name == "repair_r1":
            check_freeze(frozen, recorded(value))
    cohorts = {"heldout": recorded(value).heldout, "calibration": list(range(8)), "fixture": [0, 1]}
    if purpose not in cohorts:
        raise EvidenceError("unsupported cohort purpose")
    expected = cohorts[purpose]
    if [seed.integer() for seed in value.get("seeds").rows()] != expected:
        raise EvidenceError("wrong cohort seed register")
    worlds = value.get("worlds").rows()
    if [world.get("seed").integer() for world in worlds] != expected:
        raise EvidenceError("missing, duplicate or reordered worlds")
    for world in worlds:
        if world.get("exit_code").integer():
            raise EvidenceError("failed world")
        Reference.parse(world.get("log"))
        artifacts = files(world)
        if recorded(value).name == 'repair_r1':
            required = {'summary.json', 'events.jsonl', 'chronicle.json'} if arm == 'candidate' else {'summary.json'}
            if not required.issubset(artifacts):
                raise EvidenceError('required world artifacts missing')
    return worlds


def within(node: Node, limit: int) -> bool:
    return node.value is not None and node.integer() <= limit


def common(summary: Node, base: bool) -> tuple[bool, bool, bool, bool]:
    mux = summary.get("mux" if base else "common")
    mux2 = summary.get("mux2")
    tpd = mux.get("ticks_per_day").integer()
    clean = not mux2.get("observer_integrity_errors").rows()
    if base:
        clean &= summary.get("ledger_violation_ticks").integer() == 0
        clean &= summary.get("reservation_violation_ticks").integer() == 0
        inputs = mux2.get("inputs")
        if not inputs.get("steward").boolean() or not inputs.get("defence").boolean() or inputs.get("player_script").boolean():
            raise EvidenceError("BASE setup differs")
    return (within(mux.get("stockpile_first_tick"), tpd),
            within(mux2.get("walled_beds_for_all_first_tick"), 10 * tpd),
            mux2.get("wolf_deaths").integer() <= 1, clean)


def store_pass(snapshot: Node) -> bool:
    adults = snapshot.get("adults").rows()
    if snapshot.get("alive").integer() == 0 or not adults:
        return False
    ids = {adult.get("id").integer() for adult in adults}
    memberships = {int(sid): {member.integer() for member in members.rows()} & ids
                   for sid, members in snapshot.get("settlements").fields()}
    homes = snapshot.get("settlement_stores").rows()
    home_ids = [home.get("id").integer() for home in homes]
    if len(set(home_ids)) != len(home_ids) or set(home_ids) != set(memberships):
        raise EvidenceError("settlement ownership evidence mismatch")
    assigned = [agent for members in memberships.values() for agent in members]
    if len(assigned) != len(set(assigned)) or set(assigned) != ids:
        raise EvidenceError("living adults missing or duplicated in settlement membership")
    units = dict.fromkeys(home_ids, 0)
    reachable_units = dict.fromkeys(home_ids, 0)
    for stack in snapshot.get("stored").rows():
        reachable = {value.integer() for value in stack.get("reachable_adults").rows()}
        if not reachable.issubset(ids) or stack.get("remaining_days").integer() == 0:
            raise EvidenceError("invalid stored owner evidence")
        region = stack.get("region").integer()
        tile = [value.integer() for value in stack.get("tile").rows()]
        candidates = [home for home in homes if region != 0 and home.get("region").integer() == region]
        if not candidates:
            continue
        def ownership_order(home: Node) -> tuple[int, int]:
            point = [value.integer() for value in home.get("home").rows()]
            return max(abs(point[0] - tile[0]), abs(point[1] - tile[1])), home.get("id").integer()
        sid = min(candidates, key=ownership_order).get("id").integer()
        amount = stack.get("pieces").integer() * stack.get("units_per_piece").integer()
        units[sid] += amount
        if reachable & memberships[sid]:
            reachable_units[sid] += amount
    for home in homes:
        if units[home.get("id").integer()] != home.get("runtime_units").integer():
            raise EvidenceError("independent ownership allocation differs from runtime reserves")
    return all(reachable_units[sid] >= (72 * len(members) + 4) // 5 for sid, members in memberships.items())


def conservation(snapshot: Node) -> None:
    owners = snapshot.get("physical").rows()
    identities = [repr(owner.get("owner").value) for owner in owners]
    if len(set(identities)) != len(identities):
        raise EvidenceError("duplicate physical food owner")
    present = sum(owner.get("pieces").integer() * owner.get("units_per_piece").integer() for owner in owners)
    if present != snapshot.get("physical_food_units").integer():
        raise EvidenceError("physical owner sum differs")
    tally = snapshot.get("tallies")
    incoming = sum(tally.get(key).integer() for key in ("initial_units", "harvested_units", "craft_output_units", "traded_in_units"))
    outgoing = present + sum(tally.get(key).integer() for key in ("eaten_units", "spoiled_units", "lost_units", "craft_input_units", "traded_out_units"))
    if incoming != outgoing:
        raise EvidenceError("independent food conservation mismatch")


def expiry_pair(before: Node, after: Node) -> None:
    day = before.get("day").integer()
    expired = [owner for owner in before.get("physical").rows()
               if day >= owner.get("born_day").integer() + owner.get("lifetime_days").integer()]
    expected = sum(owner.get("pieces").integer() * owner.get("units_per_piece").integer() for owner in expired)
    actual = after.get("tallies").get("spoiled_units").integer() - before.get("tallies").get("spoiled_units").integer()
    if expected != actual:
        raise EvidenceError("expiry units differ from owner storage lifetimes")
    surviving = {repr(owner.get("owner").value): owner for owner in after.get("physical").rows()}
    for old in expired:
        new = surviving.get(repr(old.get("owner").value))
        if new is not None and new.get("born_day").integer() == old.get("born_day").integer():
            raise EvidenceError("expired owner survived daily expiry")


def audit_flows(flows: list[Node], initial: Node, boundaries: dict[int, Node], final: Node, end_tick: int) -> None:
    """Reconcile chronological flow records with independently captured cumulative tallies."""
    previous_tick = -1
    for flow in flows:
        tick = flow.get("tick").integer()
        if tick <= previous_tick or tick >= end_tick:
            raise EvidenceError("food flow ticks are duplicated, reordered or outside observation")
        previous_tick = tick
    cursor = 0
    tally = initial
    for day in sorted(boundaries):
        boundary = boundaries[day]
        while cursor < len(flows) and flows[cursor].get("tick").integer() < boundary.get("tick").integer():
            tally = flows[cursor].get("tallies")
            cursor += 1
        if tally.value != boundary.get("tallies").value:
            raise EvidenceError("food flows disagree with before-expiry boundary tallies")
    last = flows[-1].get("tallies") if flows else initial
    if last.value != final.value:
        raise EvidenceError("food flows disagree with final observed tallies")


def score_world(world: Node) -> WorldScore:
    refs = files(world)
    summary = refs["summary.json"].read()
    if summary.get("kind").text() != "m6a_food_observation_v1" or summary.get("days").integer() != 72:
        raise EvidenceError("wrong food observer protocol")
    inputs = summary.get("inputs")
    if not inputs.get("steward").boolean() or not inputs.get("defence").boolean() or inputs.get("player_orders").boolean():
        raise EvidenceError("candidate setup differs")
    if inputs.get("story").text() != "on" or inputs.get("neighbours").text() != "on":
        raise EvidenceError("candidate world rules differ")
    tpd = summary.get("ticks_per_day").integer()
    offset = summary.get("clock_offset_ticks").integer()
    if tpd != 1440 or offset != 480 or summary.get("observed_ticks").integer() != 72 * tpd:
        raise EvidenceError("incomplete calendar coverage")
    if summary.get("end_tick").integer() != 72 * tpd or summary.get("invariant_checks").integer() != 144 * tpd:
        raise EvidenceError("incomplete invariant check coverage")
    if summary.get("seed").integer() != world.get("seed").integer():
        raise EvidenceError("world seed mismatch")
    before: dict[int, Node] = {}
    daily: dict[int, Node] = {}
    meals: list[Node] = []
    hunger: list[Node] = []
    flows: list[Node] = []
    violations = 0
    with Path(refs["events.jsonl"].path).open() as stream:
        for line in stream:
            row = Node.parse(line)
            kind = row.get("type").text()
            match kind:
                case "daily":
                    snap = row.get("snapshot")
                    day = snap.get("day").integer()
                    phase = snap.get("phase").text()
                    if phase not in ("before_expiry", "after_expiry"):
                        raise EvidenceError("unknown boundary phase")
                    target = before if phase == "before_expiry" else daily
                    if day in target or snap.get("tick").integer() != max(0, day * tpd - offset) or snap.get("errors").rows():
                        raise EvidenceError("duplicate or invalid daily boundary")
                    conservation(snap)
                    target[day] = snap
                case "meal": meals.append(row)
                case "hunger_death": hunger.append(row)
                case "food_flow": flows.append(row)
                case "violation": violations += 1
                case _: raise EvidenceError(f"unknown observer record {kind}")
    if sorted(daily) != list(range(73)) or sorted(before) != list(range(1, 73)):
        raise EvidenceError("missing daily snapshots")
    for day in range(1, 73):
        expiry_pair(before[day], daily[day])
    for row in meals:
        day = row.get("day").integer()
        if row.get("pieces").integer() == 0 or not row.get("born_day").integer() <= day < row.get("born_day").integer() + row.get("storage_lifetime_days").integer():
            raise EvidenceError("meal consumed expired or invalid food")
        choice = row.get("choice")
        if choice.value is None or choice.get("agent").integer() != row.get("agent").integer() or choice.get("tick").integer() > row.get("tick").integer():
            raise EvidenceError("meal has no actual preceding matching choice")
    if sum(row.get("pieces").integer() for row in meals) != summary.get("observed_meals").integer():
        raise EvidenceError("meal event coverage mismatch")
    for row in meals + hunger:
        if row.get("day").integer() != (row.get("tick").integer() + offset) // tpd:
            raise EvidenceError("event calendar mismatch")
    causal = causal_audit(refs["chronicle.json"].read(), hunger, offset, tpd)
    window = [row for row in meals if 24 <= row.get("day").integer() < 72]
    meal_count = sum(row.get("pieces").integer() for row in window)
    cooked = sum(row.get("pieces").integer() for row in window if row.get("cooked").boolean())
    summer = {}
    for boundary in range(31, 37):
        diversity = []
        for adult in daily[boundary].get("adults").rows():
            agent = adult.get("id").integer()
            groups = {group.text() for row in meals if row.get("agent").integer() == agent
                      and boundary - 3 <= row.get("day").integer() < boundary for group in row.get("groups").rows()}
            groups.discard("meal")
            diversity.append(len(groups))
        diversity.sort()
        summer[boundary] = diversity[(len(diversity) - 1) // 2] if diversity else 0
    median = summer[36]
    audit_flows(flows, daily[0].get("tallies"), before, summary.get("final_tallies"), 72 * tpd)
    # Priority118 captures the exact end of [0,72), before day72 expiry/consumption.
    tally = before[72].get("tallies")
    harvested, spoiled = tally.get("harvested_units").integer(), tally.get("spoiled_units").integer()
    stock, beds, wolves, clean = common(summary, False)
    clean &= violations == summary.get("violation_count").integer() == 0
    w42, w66 = store_pass(daily[42]), store_pass(daily[66])
    alive = daily[72].get("alive").integer() > 0
    deaths = sum(row.get("count").integer() for row in hunger if row.get("day").integer() in (*range(42, 48), *range(66, 72)))
    return WorldScore(world.get("seed").integer(), alive, alive and deaths == 0, w42 and w66,
                      meal_count > 0 and 2 * cooked >= meal_count, median >= 3,
                      harvested > 0 and 100 * spoiled <= 30 * harvested, clean, stock, beds, wolves,
                      cooked, meal_count, median, summer, causal, harvested, before[72].get("harvested_edible_units").integer(), before[72].get("harvested_convertible_units").integer(), spoiled,
                      daily[42].get("alive").integer() > 0, daily[66].get("alive").integer() > 0, w42, w66)


def base_columns(worlds: list[Node], common_rows: list[tuple[bool, bool, bool, bool]]) -> dict:
    """BASE survival is observed; physical-food concepts did not exist in BASE."""
    alive = sum(world.get('alive_end').integer() > 0 for world in worlds)
    result = {'alive': {'base': alive, 'base_status': 'MEASURED',
                        'base_definition': 'world alive_end > 0 after 72 days'}}
    for index, name in enumerate(('stockpile_day1', 'beds_day10', 'wolf_deaths_le1', 'ledger')):
        result[name] = {'base': sum(row[index] for row in common_rows), 'base_status': 'MEASURED'}
    for name in ('hunger_free_winters', 'stores'):
        result[name] = {'base': None, 'base_status': 'NOT_OBSERVED',
                        'base_reason': 'BASE has hunger/stores, but its frozen observer lacks season-aligned death causes and physical reachable-store snapshots for this predicate'}
    for name in ('cooked_share', 'diversity', 'spoilage'):
        result[name] = {'base': None, 'base_status': 'NOT_SUPPORTED',
                        'base_reason': 'Published BASE predates physical meal kinds, dietary groups and age-tracked spoilage'}
    return result


def evaluate(candidate: Path, base: Path) -> dict:
    scores = [score_world(world) for world in receipt(candidate, "candidate", "heldout")]
    baseline = []
    base_worlds = []
    selected = recorded(Node.load(candidate))
    if recorded(Node.load(base)).name != selected.name:
        raise EvidenceError("candidate/BASE protocol differs")
    if selected.name == "repair_r1" and Node.load(candidate).get("freeze") != Node.load(base).get("freeze"):
        raise EvidenceError("candidate/BASE freeze differs")
    for world in receipt(base, "base", "heldout"):
        root = files(world)["summary.json"].read()
        rows = root.get("seeds").rows()
        if len(rows) != 1 or rows[0].get("seed").integer() != world.get("seed").integer() or rows[0].get("days").integer() != 72:
            raise EvidenceError("BASE per-world summary mismatch")
        baseline.append(common(rows[0], True))
        base_worlds.append(rows[0])
    thresholds = {"alive": 30, "hunger_free_winters": 28, "stores": 26, "cooked_share": 26, "diversity": 26, "spoilage": 26, "ledger": 32}
    bars = {name: {"passed_worlds": sum(bool(getattr(score, name)) for score in scores), "required": count} for name, count in thresholds.items()}
    for index, name in enumerate(("stockpile_day1", "beds_day10", "wolf_deaths_le1")):
        base_count = sum(row[index] for row in baseline)
        bars[name] = {"passed_worlds": sum(bool(getattr(score, name)) for score in scores), "required": max(0, base_count - 1), "base": base_count}
    if selected.name == 'repair_r1':
        for name, column in base_columns(base_worlds, baseline).items():
            bars[name].update(column)
    failed = [name for name, bar in bars.items() if bar["passed_worlds"] < bar["required"]]
    if not all(row[3] for row in baseline): failed.append("base_ledger")
    return {"schema_version": 1, "kind": "m6a_score_v1", "evaluation_protocol": selected.name, "status": "FAIL" if failed else "PASS", "failures": failed,
            "candidate": reference_json(candidate), "base": reference_json(base), "bars": bars,
            "winter_conditional": {str(day): {"passed": sum(getattr(s, f"winter{day}_store") for s in scores), "alive": sum(getattr(s, f"winter{day}_alive") for s in scores)} for day in (42, 66)},
            "worlds": [asdict(score) for score in scores]}


def calibration(candidate: Path) -> dict:
    """Eight-world diagnostics using frozen world predicates, never heldout acceptance."""
    scores = [score_world(world) for world in receipt(candidate, "candidate", "calibration")]
    names = ("alive", "hunger_free_winters", "stores", "cooked_share", "diversity", "spoilage", "ledger",
             "stockpile_day1", "beds_day10", "wolf_deaths_le1")
    return {"schema_version": 1, "kind": "m6a_calibration_score_v1", "status": "OBSERVED",
            "acceptance_status": "NOT_EVALUATED", "purpose": "calibration", "denominator": 8,
            "counts": {name: {"passed_worlds": sum(bool(getattr(score, name)) for score in scores),
                               "observed_worlds": 8} for name in names},
            "winter_conditional": {str(day): {"passed": sum(getattr(s, f"winter{day}_store") for s in scores),
                                               "alive": sum(getattr(s, f"winter{day}_alive") for s in scores)}
                                   for day in (42, 66)},
            "worlds": [asdict(score) for score in scores], "receipt": reference_json(candidate)}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="operation", required=True)
    scoring = sub.add_parser("score")
    scoring.add_argument("--candidate", type=Path, required=True)
    scoring.add_argument("--base", type=Path, required=True)
    scoring.add_argument("--out", type=Path, required=True)
    fixture = sub.add_parser("fixture")
    fixture.add_argument("--receipt", type=Path, required=True)
    fixture.add_argument("--out", type=Path)
    diagnostic = sub.add_parser("calibration")
    diagnostic.add_argument("--receipt", type=Path, required=True)
    diagnostic.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    try:
        if args.operation == "score":
            report = evaluate(args.candidate, args.base)
        elif args.operation == "calibration":
            report = calibration(args.receipt)
        else:
            scores = [score_world(world) for world in receipt(args.receipt, "candidate", "fixture")]
            passed = all(s.ledger and s.harvested_units > 0 and s.cooked_events > 0 and s.meal_events > 0 for s in scores)
            report = {"schema_version": 1, "kind": "m6a_fixture_v1", "status": "PASS" if passed else "FAIL", "worlds": [asdict(s) for s in scores], "receipt": reference_json(args.receipt)}
        if args.out: write_json(args.out, report)
        print(report["status"])
        return int(report["status"] not in ("PASS", "OBSERVED"))
    except (EvidenceError, OSError, KeyError) as error:
        parser.exit(2, f"M6A invalid evidence: {error}\n")


if __name__ == "__main__":
    raise SystemExit(main())
