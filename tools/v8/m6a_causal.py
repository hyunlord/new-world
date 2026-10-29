"""Validate actual retained food causal chains without inferring absent history."""
from collections import Counter

from m6a_evidence import EvidenceError, Node


def audit(chronicle: Node, hunger: list[Node], offset: int, tpd: int) -> dict:
    rows = chronicle.get("entries").rows()
    if [row.get("index").integer() for row in rows] != list(range(len(rows))):
        raise EvidenceError("chronicle identity is not consecutive")

    def entry(parent: Node, index: int) -> Node:
        if index >= parent.get("index").integer():
            raise EvidenceError("causal parent does not precede child")
        return rows[index]

    def linked(parent: Node) -> Node:
        return entry(parent, parent.get("cause").get("Entry").integer())

    def stage(parent: Node, child: Node, expected: int) -> None:
        if child.get("kind").text() != "hunger_food_chain" or child.get("code").integer() != expected:
            raise EvidenceError("wrong hunger chain stage")
        for key in ("tick", "subject"):
            if child.get(key).integer() != parent.get(key).integer():
                raise EvidenceError("hunger chain crosses person or tick")
        child.get("value").integer()

    deaths = Counter()
    reasons = Counter()
    winter = 0
    for row in rows:
        kind = row.get("kind").text()
        if kind == "died" and row.get("code").integer() == 0:
            deaths[row.get("tick").integer()] += 1
            need = linked(row)
            stage(row, need, 0)
            need.get("cause").get("Sample").get("sample").integer()
            sample = need.get("sample")
            if sample.get("agent").integer() != row.get("subject").integer() or sample.get("tick").integer() != row.get("tick").integer():
                raise EvidenceError("hunger death lacks actual same-person need sample")
            if sample.get("hunger").integer() != need.get("value").integer() or sample.get("hunger").integer() < 1000:
                raise EvidenceError("hunger death sample below saturation")
            stores = entry(need, sample.get("entry").integer())
            stage(need, stores, 1)
            ripe = linked(stores)
            stage(stores, ripe, 2)
            reason = linked(ripe)
            code = reason.get("code").integer()
            if code not in (3, 4, 5, 6):
                raise EvidenceError("unknown hunger opportunity classification")
            stage(ripe, reason, code)
            all_units, reachable = reason.get("value").integer(), ripe.get("value").integer()
            if reachable > all_units or (code in (3, 4) and all_units != 0) or (code == 5 and (all_units == 0 or reachable != 0)) or (code == 6 and reachable == 0):
                raise EvidenceError("hunger opportunity observation contradicts chain")
            reasons[code] += 1
        elif kind == "winter_stored_survived":
            start = linked(row)
            if start.get("kind").text() != "winter_food_plan" or row.get("settlement").value != start.get("settlement").value:
                raise EvidenceError("winter survival does not reference its actual plan")
            start_day = (start.get("tick").integer() + offset) // tpd
            end_day = (row.get("tick").integer() + offset) // tpd
            if start_day % 24 != 18 or end_day != start_day + 6 or start.get("value").integer() < start.get("code").integer() or row.get("value").integer() == 0:
                raise EvidenceError("winter milestone lacks qualifying boundary observations")
            winter += 1
    observed = Counter()
    for row in hunger:
        observed[row.get("tick").integer()] += row.get("count").integer()
    if deaths != observed:
        raise EvidenceError("hunger death tally lacks complete causal chains")
    return {"hunger_deaths": sum(deaths.values()), "opportunity_reason_counts": dict(reasons),
            "historical_unharvested_reason_unknown": reasons[6], "winter_survival_chains": winter}
