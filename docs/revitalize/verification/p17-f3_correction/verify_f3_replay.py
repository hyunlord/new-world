#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 verify_f3_replay.py ORIGINAL_EVAL CORRECTED_EVAL NEW_RECEIPT
"""Verify the permitted F3 replay delta without recomputing cohort statistics."""
from __future__ import annotations

from collections import Counter
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
import hashlib
import json
import math
from pathlib import Path
import sys
from typing import Final, TypeAlias, assert_never

Json: TypeAlias = None | bool | int | float | str | list["Json"] | dict[str, "Json"]
Document: TypeAlias = dict[str, Json]
ROOT: Final = set("schema_version stage mode verdict rules metrics candidate baseline identity performance family methods".split())
METRIC_FIELDS: Final = set("value ci95 verdict bar unit candidate baseline bootstrap_valid bootstrap_replicates details error".split())
METRICS: Final = {"F1", "F2", "F3", "F4", "F5", *(f"F6-{x}" for x in ("alive", "deaths", "food", "shelters", "social", "wood")), *(f"F3-{x}{axis}" for x in ("siblings", "unrelated") for axis in ("", "-ha", "-ns", "-p", "-rd"))}
RULES: Final = {f"F{x}" for x in range(1, 9)}
PROTECTED_METRIC_FIELDS: Final = {"bar", "unit", "bootstrap_replicates"}
COHORT_FIELDS: Final = set("scene source days seeds worlds hourly_states activity_minutes health_by_world inputs_sha256".split())
WORLD_FIELDS: Final = set("seed day60_partnered_adults day60_adults births_from_day20 couple_births_from_day20 all_births all_fallback_births same_shelter_couple_nights bedded_couple_nights all_living_couple_nights bedless_person_nights completed_fissions cancelled_fissions pending_fissions departure_couples split_couples completed_departure_households_kept completed_departure_households_split sibling_unordered_pairs unrelated_unordered_pairs excluded_close_kin_pairs stage_pair_categories f3_unassigned_person_hours f3_unassigned_people pair_eligibility".split())


@dataclass(frozen=True, slots=True)
class ContractError(Exception):
    reason: str

    def __str__(self) -> str:
        return self.reason


def require(condition: bool, reason: str) -> None:
    if not condition:
        raise ContractError(reason)


def mapping(value: Json) -> Document:
    require(type(value) is dict, "expected JSON object")
    return value


def sequence(value: Json) -> list[Json]:
    require(type(value) is list, "expected JSON array")
    return value


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical(value: Json) -> bytes:
    return json.dumps(value, sort_keys=True, ensure_ascii=False, allow_nan=False, separators=(",", ":")).encode()


def unique_object(pairs: list[tuple[str, Json]]) -> Document:
    value = dict(pairs)
    require(len(value) == len(pairs), "duplicate JSON object key")
    return value


def finite_float(value: str) -> float:
    number = float(value)
    require(math.isfinite(number), "nonfinite JSON number")
    return number


def finite(value: Json) -> bool:
    return type(value) is int or (type(value) is float and math.isfinite(value))


def aggregate(verdicts: list[Json]) -> str:
    for verdict in ("ERROR", "FAIL", "PENDING"):
        if verdict in verdicts:
            return verdict
    return "PASS"


def validate(report: Document) -> None:
    require(set(report) == ROOT, "report root keys differ from complete schema17")
    require(type(report["schema_version"]) is int and report["schema_version"] == 17 and report["stage"] == "b", "requires schema17 Stage B")
    require(report["mode"] in ("evaluation", "tuning"), "invalid report mode")
    rules, metrics = mapping(report["rules"]), mapping(report["metrics"])
    require(set(rules) == RULES and set(metrics) == METRICS, "missing or extra rule/metric keys")
    for name, raw in metrics.items():
        metric = mapping(raw)
        require(set(metric) == METRIC_FIELDS, f"{name}: incomplete metric")
        interval = sequence(metric["ci95"])
        require(metric["verdict"] in ("PASS", "FAIL") and metric["error"] is None, f"{name}: invalid metric result")
        require(finite(metric["value"]) and len(interval) == 2 and all(finite(x) for x in interval), f"{name}: undefined/nonfinite estimate")
        require(interval[0] <= interval[1], f"{name}: reversed interval")
        require(type(metric["bootstrap_replicates"]) is int and metric["bootstrap_replicates"] == 2000, f"{name}: bootstrap count changed")
        require(type(metric["bootstrap_valid"]) is int and 0 < metric["bootstrap_valid"] <= 2000, f"{name}: invalid bootstrap result")
    for key in ("F1", "F2", "F3", "F4", "F5"):
        require(rules[key] == mapping(metrics[key])["verdict"], f"{key}: rule/metric verdict mismatch")
    require(rules["F6"] == aggregate([mapping(v)["verdict"] for k, v in metrics.items() if k.startswith("F6-")]), "F6: aggregate mismatch")
    for key, name in (("F7", "identity"), ("F8", "performance")):
        evidence = mapping(report[name])
        require(set(evidence) == {"verdict", "source", "sha256", "payload", "error"}, f"{name}: incomplete evidence")
        pending = report["mode"] == "tuning" and evidence["verdict"] == "PENDING"
        require(pending or (evidence["verdict"] in ("PASS", "FAIL") and evidence["error"] is None), f"{name}: invalid evidence status")
        require(rules[key] == evidence["verdict"], f"{key}: evidence verdict mismatch")
    require(report["verdict"] == aggregate(list(rules.values())) and report["verdict"] != "ERROR", "invalid aggregate report verdict")
    eligibility = mapping(report["methods"]).get("F3_eligibility")
    require(type(eligibility) is str and bool(eligibility), "missing/invalid F3 eligibility method")
    for name in ("candidate", "baseline"):
        cohort = mapping(report[name])
        require(set(cohort) == COHORT_FIELDS, f"{name}: incomplete cohort")
        require(type(cohort["days"]) is int and cohort["days"] > 0, f"{name}: invalid days")
        require(type(cohort["worlds"]) is int and cohort["worlds"] == len(sequence(cohort["seeds"])) > 0, f"{name}: invalid world count")
        require(bool(mapping(cohort["inputs_sha256"])), f"{name}: missing input hashes")
    family = mapping(report["family"])
    require(set(family) == {"worlds", "births_by_day", "inputs_sha256"}, "incomplete family block")
    worlds = sequence(family["worlds"])
    require(bool(worlds), "missing family worlds")
    for raw in worlds:
        world = mapping(raw)
        require(set(world) == WORLD_FIELDS, "missing or extra family world fields")
        pairs = mapping(world["pair_eligibility"])
        require(set(pairs) == {"siblings", "unrelated"}, "missing pair eligibility lists")
        for name in ("siblings", "unrelated"):
            sequence(pairs[name])
        for key in ("sibling_unordered_pairs", "unrelated_unordered_pairs"):
            require(type(world[key]) is int and world[key] >= 0, f"invalid {key}")


def category(path: tuple[str, ...]) -> str | None:
    if len(path) >= 3 and path[0] == "metrics" and path[1] in METRICS and path[1].startswith("F3") and path[2] not in PROTECTED_METRIC_FIELDS:
        return "F3_metrics"
    if path == ("rules", "F3"):
        return "F3_rule"
    if path == ("verdict",):
        return "derived_aggregate_verdict"
    if path == ("methods", "F3_eligibility"):
        return "F3_eligibility_method"
    if len(path) >= 4 and path[:2] == ("family", "worlds") and path[2].isdigit():
        if path[3] in ("sibling_unordered_pairs", "unrelated_unordered_pairs") and len(path) == 4:
            return "F3_world_pair_counts"
        if path[3] == "pair_eligibility" and len(path) >= 5:
            return "F3_pair_eligibility"
    return None


def describe(value: Json) -> Document:
    encoded = canonical(value)
    return {"sha256": sha(encoded), "bytes": len(encoded), "preview": encoded[:140].decode(errors="replace"), "type": type(value).__name__}


def differences(before: Json, after: Json, path: tuple[str, ...] = ()) -> list[Document]:
    scope = category(path)
    pointer = "/" + "/".join(x.replace("~", "~0").replace("/", "~1") for x in path)
    change = {"path": pointer, "category": scope, "allowed": scope is not None, "before": describe(before), "after": describe(after)}
    if type(before) is not type(after):
        return [change]
    if scope == "F3_pair_eligibility":
        return [] if canonical(before) == canonical(after) else [change]
    match before, after:
        case dict() as left, dict() as right:
            result = []
            for key in sorted(left.keys() | right.keys()):
                if key not in left or key not in right:
                    result.append({"path": pointer.rstrip("/") + "/" + key.replace("~", "~0").replace("/", "~1"), "category": None, "allowed": False, "error": "missing/extra key", "before_present": key in left, "after_present": key in right})
                else:
                    result.extend(differences(left[key], right[key], (*path, key)))
            return result
        case list() as left, list() as right:
            if len(left) != len(right):
                return [change]
            return [item for index, (a, b) in enumerate(zip(left, right)) for item in differences(a, b, (*path, str(index)))]
        case (None | bool() | int() | float() | str()), (None | bool() | int() | float() | str()):
            return [] if canonical(before) == canonical(after) else [change]
        case unreachable:
            assert_never(unreachable)


def compare(before: Document, after: Document) -> Document:
    errors = []
    for label, report in (("original", before), ("corrected", after)):
        try:
            validate(report)
        except ContractError as error:
            errors.append(f"{label}: {error}")
    changes = differences(before, after)
    forbidden = [change for change in changes if not change["allowed"]]
    passed = not errors and not forbidden
    return {"schema_version": 1, "verdict": "PASS" if passed else "FAIL", "passed": passed,
        "original_verdict": before.get("verdict"), "corrected_verdict": after.get("verdict"), "errors": errors,
        "changed_path_count": len(changes), "allowed_changed_path_count": len(changes) - len(forbidden), "forbidden_changed_path_count": len(forbidden),
        "counts_by_category": dict(Counter(str(change["category"]) for change in changes)), "changes": changes,
        "limits": "Structural isolation/status check only; no raw-file validation, pair membership reconstruction, numeric F3 recomputation, or claim that unchanged F3 proves a correction. Pair eligibility arrays are compared as canonical JSON blocks, not semantically matched."}


def main(argv: list[str]) -> int:
    if len(argv) != 3:
        print(__doc__ + "\nUsage: verify_f3_replay.py ORIGINAL_EVAL CORRECTED_EVAL NEW_RECEIPT", file=sys.stderr)
        return 2
    original, corrected, receipt = (Path(value).resolve() for value in argv)
    try:
        with receipt.open("x", encoding="utf-8") as output:
            result: Document = {"schema_version": 1, "verdict": "FAIL", "passed": False, "errors": []}
            inputs = []
            try:
                documents = []
                for path in (original, corrected):
                    data = path.read_bytes()
                    inputs.append({"path": str(path), "sha256": sha(data), "bytes": len(data)})
                    documents.append(mapping(json.loads(data, object_pairs_hook=unique_object, parse_float=finite_float, parse_constant=finite_float)))
                result = compare(*documents)
                require(all(sha(Path(item["path"]).read_bytes()) == item["sha256"] for item in inputs), "input changed while comparing")
            except (ContractError, OSError, UnicodeError, json.JSONDecodeError, RecursionError) as error:
                result.update(verdict="FAIL", passed=False, errors=[str(error)])
            result.update(created_utc=datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"), inputs=inputs,
                comparator={"path": str(Path(__file__).resolve()), "sha256": sha(Path(__file__).read_bytes())})
            json.dump(result, output, ensure_ascii=False, indent=2, allow_nan=False)
            output.write("\n")
        print(f'{result["verdict"]}: {receipt}; SHA256 {sha(receipt.read_bytes())}')
        return 0 if result["passed"] else 1
    except OSError as error:
        print(f"Cannot create exclusive receipt: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
