"""Pairwise summary comparison; raw evidence is never modified.

Only the twelve baseline timing templates and the two historical BASE schema
asymmetries below are masked. Unknown keys, including nested observer names and
source provenance, remain comparable. Binary/source provenance is admitted by
the calling gate, never inferred from a filename or erased recursively here.
"""
from __future__ import annotations

import copy
import json
import math
from pathlib import Path
from typing import Any

BASE_COMMIT = "fbc03bddb8f5b68f43fba1b2dad283982e5cf436"
TIMINGS = ("wall_seconds", "ms_per_agent_tick")
STATS = ("median", "p10", "p90", "min", "max")
OBSERVERS = ("logs_ground_by_day", "cap_target_by_day")
EXCLUDED_PATHS = tuple(f"/runs/*/{key}" for key in TIMINGS) + tuple(
    f"/timing/{key}/{stat}" for key in TIMINGS for stat in STATS
) + tuple(f"/seeds/*/{key} (historical BASE asymmetry only)" for key in OBSERVERS)


def load_json(path: str | Path) -> Any:
    def pairs(items: list[tuple[str, Any]]) -> dict[str, Any]:
        result: dict[str, Any] = {}
        for key, value in items:
            if key in result:
                raise ValueError(f"{path}: duplicate key {key!r}")
            result[key] = value
        return result

    def constant(value: str) -> Any:
        raise ValueError(f"{path}: nonfinite number {value}")

    value = json.loads(Path(path).read_text(), object_pairs_hook=pairs, parse_constant=constant)
    validate_finite(value)
    return value


def validate_finite(value: Any, path: str = "") -> None:
    if isinstance(value, float) and not math.isfinite(value):
        raise ValueError(f"{path or '/'}: nonfinite number")
    if isinstance(value, dict):
        for key, child in value.items():
            validate_finite(child, f"{path}/{key}")
    elif isinstance(value, list):
        for index, child in enumerate(value):
            validate_finite(child, f"{path}/{index}")


def number(value: Any, path: str) -> None:
    if type(value) not in (int, float) or not math.isfinite(value) or value < 0:
        raise ValueError(f"{path}: expected finite nonnegative number")


def first_difference(a: Any, b: Any, path: str = "") -> str | None:
    location = path or "/"
    if type(a) is not type(b):
        return f"{location}: type {type(a).__name__} != {type(b).__name__}"
    if isinstance(a, dict):
        if a.keys() != b.keys():
            return f"{location}: missing {sorted(a.keys() - b.keys())}, extra {sorted(b.keys() - a.keys())}"
        for key in sorted(a):
            escaped = key.replace("~", "~0").replace("/", "~1")
            difference = first_difference(a[key], b[key], f"{path}/{escaped}")
            if difference:
                return difference
    elif isinstance(a, list):
        if len(a) != len(b):
            return f"{location}: length {len(a)} != {len(b)}"
        for index, (left, right) in enumerate(zip(a, b)):
            difference = first_difference(left, right, f"{path}/{index}")
            if difference:
                return difference
    elif a != b:
        return f"{location}: {a!r} != {b!r}"
    return None


def normalise_pair(a: Any, b: Any, base_commit: str | None = None) -> tuple[Any, Any, list[str]]:
    validate_finite(a)
    validate_finite(b)
    left, right = copy.deepcopy(a), copy.deepcopy(b)
    masked: list[str] = []
    if not isinstance(left, dict) or not isinstance(right, dict):
        return left, right, masked
    # Baseline probe summary v13: preserve containers and all unknown fields.
    for label, value in (("BASE", left), ("HEAD", right)):
        if type(value.get("schema_version")) is int and value["schema_version"] == 13:
            if not isinstance(value.get("runs"), list) or not value["runs"]:
                raise ValueError(f"{label}/runs: expected nonempty array")
            try:
                for index, row in enumerate(value["runs"]):
                    for key in TIMINGS:
                        path = f"/runs/{index}/{key}"
                        number(row[key], path)
                        row[key] = 0
                        masked.append(f"{label}{path}")
                for key in TIMINGS:
                    for stat in STATS:
                        path = f"/timing/{key}/{stat}"
                        number(value["timing"][key][stat], path)
                        value["timing"][key][stat] = 0
                        masked.append(f"{label}{path}")
            except (KeyError, TypeError) as error:
                raise ValueError(f"{label}: incomplete baseline timing schema: {error}") from error
    if left.get("scene") == right.get("scene") == "colony":
        for label, value in (("BASE", left), ("HEAD", right)):
            rows, days = value.get("seeds"), value.get("days")
            if type(days) is not int or days < 1 or not isinstance(rows, list) or not rows:
                raise ValueError(f"{label}: invalid colony seed/day coverage")
            seeds: set[int] = set()
            for index, row in enumerate(rows):
                if not isinstance(row, dict) or type(row.get("seed")) is not int or type(row.get("days")) is not int or row["days"] != days or row["seed"] in seeds:
                    raise ValueError(f"{label}/seeds/{index}: invalid or duplicate seed/day coverage")
                seeds.add(row["seed"])
        if base_commit == BASE_COMMIT:
            base_rows, head_rows = left.get("seeds"), right.get("seeds")
            if not isinstance(base_rows, list) or not isinstance(head_rows, list) or not base_rows or len(base_rows) != len(head_rows):
                raise ValueError("/seeds: historical comparison needs matching nonempty arrays")
            for index, (base, head) in enumerate(zip(base_rows, head_rows)):
                path = f"/seeds/{index}"
                if not isinstance(base, dict) or not isinstance(head, dict):
                    raise ValueError(f"{path}: expected seed objects")
                absent = all(key not in base for key in OBSERVERS)
                added = all(key in head for key in OBSERVERS)
                if not (absent and added):
                    continue  # Shared fields and incomplete schemas must compare normally.
                days = base.get("days")
                seed = base.get("seed")
                if type(seed) is not int or type(head.get("seed")) is not int or seed != head["seed"] or type(days) is not int or days < 1 or type(head.get("days")) is not int or head["days"] != days or left.get("days") != days or right.get("days") != days:
                    raise ValueError(f"{path}: mismatched seed/day coverage")
                logs, caps = (head[key] for key in OBSERVERS)
                if not isinstance(logs, list) or len(logs) != days or any(type(item) is not int or item < 0 for item in logs):
                    raise ValueError(f"{path}/logs_ground_by_day: expected {days} nonnegative integer observations")
                if not isinstance(caps, list) or len(caps) != days or any(item is not None for item in caps):
                    raise ValueError(f"{path}/cap_target_by_day: Stage -1 requires {days} null observations")
                for key in OBSERVERS:
                    del head[key]
                    masked.append(f"HEAD{path}/{key}")
    return left, right, masked
