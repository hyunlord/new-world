#!/usr/bin/env python3
"""Independent, stdlib-only B4 evidence scorer (D39).

Run --manifest evaluation-index.json --out score.json; --calibration permits
explicit subsets of seeds 0..7 and arms, never an evaluation acceptance claim.
Input hashes refer to raw bytes. No Rust helper, simulation, or subprocess runs.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import re
from collections import Counter
from fractions import Fraction
from pathlib import Path
from typing import TypeAlias

JSON: TypeAlias = None | bool | int | str | list["JSON"] | dict[str, "JSON"]
Record: TypeAlias = dict[str, JSON]
MASK = (1 << 64) - 1
ARMS = ("B0", "BP", "BPT", "BPTH")
CAPS = [0, 20, 35, 45, 55, 65, 75, 85, 92, 97, 100]
COLD_TICKS = [day * 1440 + hour * 60 for day in (19, 20) for hour in range(6)]


class Invalid(ValueError):
    """Malformed, unbound, or internally inconsistent evidence."""


def require(condition: bool, detail: str) -> None:
    if not condition:
        raise Invalid(detail)


def parse_json(text: str, *, legacy_floats: bool = False) -> JSON:
    def pairs(entries):
        result = {}
        for key, value in entries:
            require(key not in result, f"duplicate JSON key: {key}")
            result[key] = value
        return result

    def non_integer(token):
        raise Invalid(f"non-integer JSON number: {token}")

    def legacy_number(token):
        number = float(token)
        require(math.isfinite(number), "nonfinite legacy number")
        return number

    try:
        return json.loads(text, object_pairs_hook=pairs, parse_float=legacy_number if legacy_floats else non_integer,
                          parse_constant=non_integer)
    except json.JSONDecodeError as error:
        raise Invalid(f"invalid JSON: {error}") from error


def validate(value, schema, where="record") -> None:
    """Small closed-schema parser: every key and numeric width is explicit."""
    if isinstance(schema, dict):
        require(type(value) is dict, f"{where}: expected object")
        require(set(value) == set(schema), f"{where}: fields differ: {sorted(set(value) ^ set(schema))}")
        for key, field in schema.items():
            validate(value[key], field, f"{where}.{key}")
    elif isinstance(schema, tuple):
        tag, inner = schema
        if tag == "nullable":
            if value is not None:
                validate(value, inner, where)
        elif tag == "enum":
            require(type(value) is str and value in inner, f"{where}: invalid enum")
        elif tag == "list":
            require(type(value) is list, f"{where}: expected array")
            for index, element in enumerate(value):
                validate(element, inner, f"{where}[{index}]")
        else:
            raise Invalid(f"unknown validator schema: {tag}")
    elif schema == "command":
        check_command(value, where)
    elif schema == "bill_mode":
        require(type(value) is dict and len(value) == 1 and next(iter(value)) in ("Make", "Maintain"), f"{where}: bill mode")
        validate(next(iter(value.values())), "u16", where)
    elif schema == "bool":
        require(type(value) is bool, f"{where}: expected bool")
    elif schema == "str":
        require(type(value) is str, f"{where}: expected string")
    elif schema == "hash":
        require(type(value) is str and re.fullmatch(r"[0-9a-f]{64}", value) is not None,
                f"{where}: expected SHA256")
    elif schema == "material_term":
        require(type(value) is dict, f"{where}: material descriptor")
        if value.get("kind") == "none":
            validate(value, {"kind": ("enum", ("none",))}, where)
        else:
            validate(value, {"kind": ("enum", ("hardness",)), "offset": "i32", "divisor": "i32"}, where)
            require(value["divisor"] > 0, f"{where}: nonpositive divisor")
    elif schema.startswith(("u", "i")) and schema[1:].isdigit():
        bits = int(schema[1:])
        lo, hi = (0, (1 << bits) - 1) if schema[0] == "u" else (-(1 << (bits - 1)), (1 << (bits - 1)) - 1)
        require(type(value) is int and lo <= value <= hi, f"{where}: expected {schema}")
    else:
        raise Invalid(f"unknown schema {schema}")


def array(schema):
    return ("list", schema)


def optional(schema):
    return ("nullable", schema)


REF = {"path": "str", "sha256": "hash"}
REASON = {"code": "str", "executed_tick": optional("u64"), "agent_id": optional("u64"), "detail": "str"}
STATUS = ("enum", ("VALID", "OBSERVER_INVALID", "NOT_MEASURED", "NOT_RUN"))
TALLY = {"agent_id": "u64", "recipe_code": "u32", "completed_count": "u64", "quality_sum": "u64"}
GATE = {"node_id": "u8", "entry_present": "bool", "proficiency": "u16",
        "last_used": optional("u16"), "admission_threshold": "u16"}
CRAFT = {
    **dict.fromkeys(("seed", "agent_id", "executed_tick"), "u64"),
    **dict.fromkeys(("workshop_id", "bill_id", "recipe_code", "output_item_code"), "u32"),
    "recipe_identity": "str", "catalog_revision": "str", "output_count": "u16",
    "at": array("u32"), "job_stage": ("enum", ("working",)), "ticks_left": "u32",
    "skill_code": "u8", "skill_present": "bool", "xp_before": "u32", "level_before": "u8",
    "material_id": optional("u32"), "material_hardness": optional("u16"),
    "hardness_percent": optional("i32"), "temperament_present": "bool", "persistence": optional("u16"),
    "knowledge_enabled": "bool", "gate_mask": "u32", "gating_nodes": array(GATE),
    "recipe_quality_add": "i32", "material_term": "material_term",
    **dict.fromkeys(("tally_before_count", "tally_after_count", "tally_before_quality_sum",
                    "tally_after_quality_sum", "completion_count_delta"), "u64"),
    "actual_quality": "u8", "alive_after_tick": "bool", "arithmetic_manifest_sha256": "hash",
}
COLD_COUNTS = dict.fromkeys(("clothed_cold", "clothed_eligible", "unclothed_cold", "unclothed_eligible"), "u64")
MEAT = dict.fromkeys(("dropped", "eaten", "spoiled", "hauls"), "u64")
DAY = {"day": "u32", "executed_tick": "u64", "population": "u64", "insulating_wearers": "u64",
       "garment_counts": array({"item_code": "u32", "count": "u64", "cold_permille": "u32"}),
       "craft_tallies": array(TALLY), "cold": COLD_COUNTS, "deaths_total": "u64",
       "jobs_done_total": "u64", "cancel_totals": array({"reason": "str", "count": "u64"}),
       "meat": MEAT, "ledger_violation_ticks": "u64", "reservation_violation_ticks": "u64"}
CONSERVATION = dict.fromkeys(("ledger_violation_ticks", "reservation_violation_ticks",
                            "unresolved_start_count", "unresolved_terminal_count"), "u64")
META_SCHEMA = {"schema_version": "u32", "arm": ("enum", ARMS), "arithmetic_manifest_sha256": "hash",
               "catalog_revision": "str", "policy": ("enum", ("old_3_1", "scarce_30_10_1"))}


def raw_ref(base: Path, ref: Record) -> bytes:
    validate(ref, REF, "file reference")
    try:
        data = (base / ref["path"]).read_bytes()
    except OSError as error:
        raise Invalid(f"unreadable evidence {ref['path']}: {error}") from error
    require(hashlib.sha256(data).hexdigest() == ref["sha256"], f"hash mismatch: {ref['path']}")
    return data


def read_ref(base: Path, ref: Record) -> JSON:
    try:
        return parse_json(raw_ref(base, ref).decode("utf-8"))
    except UnicodeDecodeError as error:
        raise Invalid(f"non-UTF8 JSON: {ref['path']}") from error


def mix(value: int) -> int:
    z = (value + 0x9E3779B97F4A7C15) & MASK
    z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & MASK
    z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & MASK
    return z ^ (z >> 31)


def keyed_u64(seed: int, stream: int, subject: int, tick: int, slot: int, replicate: int) -> int:
    value = mix(seed ^ stream)
    for field in (subject, tick, slot, replicate):
        value = mix(value ^ field)
    return value


def keyed_below(seed: int, stream: int, subject: int, tick: int, slot: int, replicate: int, n: int) -> int:
    require(n > 0, "draw bound must be positive")
    return (keyed_u64(seed, stream, subject, tick, slot, replicate) * n) >> 64


def weighted_index(draw: int, weights: list[int]) -> int:
    for index, weight in enumerate(weights):
        if draw < weight:
            return index
        draw -= weight
    raise Invalid("weighted draw outside table")


def trunc_div(numerator: int, denominator: int) -> int:
    require(denominator > 0, "nonpositive divisor")
    return (1 if numerator >= 0 else -1) * (abs(numerator) // denominator)


def craft_formula(row: Record, arithmetic: Record) -> Record:
    weights = arithmetic["quality_bonus_table"]
    draw = keyed_below(row["seed"], arithmetic["stream_quality"], row["agent_id"],
                       row["executed_tick"], row["bill_id"], 0, sum(weights))
    bonus = weighted_index(draw, weights) - 10
    term = row["material_term"]
    material = 0
    if term["kind"] == "hardness" and row["hardness_percent"] is not None:
        material = trunc_div(row["hardness_percent"] - term["offset"], term["divisor"])
    temperament = 2 if row["temperament_present"] and row["persistence"] >= 600 else 0
    raw = arithmetic["quality_base"] + arithmetic["quality_per_level"] * row["level_before"]
    base = max(0, min(100, raw + bonus + row["recipe_quality_add"] + material + temperament))
    cap = 100
    if row["knowledge_enabled"] and row["gate_mask"]:
        proficiency = min(node["proficiency"] for node in row["gating_nodes"])
        cap = arithmetic["qual_cap"][min(proficiency // 100, 10)]
    return {"quality": min(base, cap), "base": base, "cap": cap,
            "bonus": bonus, "material": material, "temperament": temperament}


def check_craft(row: Record, arithmetic: Record, metadata: Record, *, controlled: bool = False) -> Record:
    validate(row, CRAFT, "craft")
    require(row["arithmetic_manifest_sha256"] == metadata["arithmetic_manifest_sha256"], "craft arithmetic hash")
    require(row["catalog_revision"] == metadata["catalog_revision"], "craft catalogue revision")
    require(len(row["at"]) == 2 and (controlled or row["ticks_left"] <= 1), "craft completion boundary")
    require(row["completion_count_delta"] == row["tally_after_count"] - row["tally_before_count"] == 1,
            "craft count delta must equal one")
    require(row["actual_quality"] == row["tally_after_quality_sum"] - row["tally_before_quality_sum"] <= 100,
            "craft quality delta mismatch")
    level = max(i for i, xp in enumerate(arithmetic["xp_table"]) if row["xp_before"] >= xp)
    require(row["level_before"] == level, "craft XP/level mismatch")
    require(row["skill_present"] or (row["xp_before"] == 0 and level == 0), "missing skill has nonzero value")
    require(row["temperament_present"] == (row["persistence"] is not None), "temperament presence mismatch")
    if row["persistence"] is not None:
        require(row["persistence"] <= 1000, "temperament outside permille")
    require((row["material_hardness"] is None) == (row["hardness_percent"] is None), "hardness presence mismatch")
    if row["material_hardness"] is not None:
        require(row["material_id"] is not None and row["hardness_percent"] == row["material_hardness"] // 10,
                "material raw hardness mismatch")
    bits = [i for i in range(32) if row["gate_mask"] & (1 << i)]
    require([n["node_id"] for n in row["gating_nodes"]] == bits, "gating node coverage/order")
    nodes = {n["node_id"]: n for n in arithmetic["knowledge_nodes"]}
    for node in row["gating_nodes"]:
        require(node["node_id"] in nodes and node["admission_threshold"] == nodes[node["node_id"]]["threshold"], "gate threshold differs from manifest")
        require(node["proficiency"] <= 1000, "proficiency outside permille")
        require(node["entry_present"] == (node["last_used"] is not None), "knowledge entry presence mismatch")
        require(node["entry_present"] or node["proficiency"] == 0, "missing knowledge has proficiency")
    primary = arithmetic["primary_recipe"]
    if row["recipe_code"] == primary["code"]:
        for event_key, manifest_key in [("recipe_identity", "identity"), ("output_item_code", "output_item_code"),
                ("output_count", "output_count"), ("skill_code", "skill_code"),
                ("recipe_quality_add", "quality_add"), ("material_term", "material_term"), ("gate_mask", "gate_mask")]:
            require(row[event_key] == primary[manifest_key], f"primary recipe {event_key} mismatch")
    result = craft_formula(row, arithmetic)
    require(row["actual_quality"] == result["quality"], "craft arithmetic mismatch")
    return result


def check_coverage(observed: list, expected: list, label: str) -> None:
    require(len(observed) == len(set(observed)), f"duplicate {label}")
    require(set(observed) == set(expected), f"missing/unexpected {label}")


def rate(numerator: int, denominator: int) -> Record:
    require(0 <= numerator <= denominator, "rate numerator exceeds denominator")
    return {"numerator": numerator, "denominator": denominator,
            "status": "VALID" if denominator else "NOT_MEASURED",
            "reason": None if denominator else "zero denominator"}


def rational(value: Fraction) -> Record:
    return {"numerator": value.numerator, "denominator": value.denominator}


def tally_map(rows: list[Record]) -> dict:
    validate(rows, array(TALLY), "tallies")
    keys = [(r["agent_id"], r["recipe_code"]) for r in rows]
    require(keys == sorted(set(keys)), "tally keys duplicate or unordered")
    for row in rows:
        require(row["quality_sum"] <= row["completed_count"] * 100, "impossible tally quality")
    return {key: (row["completed_count"], row["quality_sum"]) for key, row in zip(keys, rows)}


GOOD = {"item_code": "u32", "count": "u16", "quality": "u8"}
RESULT = {"accepted": "bool", "count": "u32", "reason": optional("str")}
WAVE = {"state": ("enum", ("arriving", "waiting", "leaving", "gone")),
        "waiting_since": optional("u64"),
        "answer": optional(("enum", ("accepted", "declined", "unanswered"))), "joined_ids": array("u64")}
COMMAND_RECEIPT = {"ordinal": "u64", "upcoming_tick": "u64", "command": "command", "result": RESULT,
                   "population_before": "u64", "population_after": "u64", "wave_id": optional("u32"),
                   "wave_size": optional("u64"), "wave_before": optional(WAVE), "wave_after": optional(WAVE)}
WAVE_TRANSITION = {"wave_id": "u32", "executed_tick": "u64", "before": optional(WAVE), "after": WAVE,
                   "population_before": "u64", "population_after": "u64", "configured_cap": optional("u32"),
                   "outcome": ("enum", ("joined", "acceptance_cleared_waiting", "state_changed")),
                   "population_observation": ("enum", ("tick_boundary_not_arrival_instant",))}
PURCHASE = {"command_ordinal": "u64", "caravan_id": "u32", "executed_tick": "u64",
            "goods_before": array(GOOD), "goods_after": array(GOOD),
            **dict.fromkeys(("unknown_bought_before", "unknown_bought_after", "eurekas_before", "eurekas_after"), "u64")}
ELIGIBLE = {"authored_index": "u32", "item_code": "u32", "made_by": optional("u8"),
            "classification": ("enum", ("staple", "lacked", "scarce", "rare")), "weight": "u32",
            "count_min": "u16", "count_max": "u16"}
CARAVAN = {"caravan_id": "u32", "neighbour_id": "u8", "executed_tick": "u64", "policy": META_SCHEMA["policy"],
           "neighbour_known_mask": "u32", "old_all_knowledge_practice_mask": "u32",
           "tech_progress_permille": "u16", "practice_days": "u64",
           "practitioner_rows": array({"node_id": "u8", "count": "u64"}),
           "practitioner_inputs": array({"agent_id": "u64", "is_child": "bool",
               "knowledge": array({"node_id": "u8", "proficiency": "u16", "last_used": "u16"})}),
           "eligible_rows": array(ELIGIBLE), "initial_goods": array(GOOD)}
HAUL_START = {"agent_id": "u64", "executed_tick": "u64", "source": array("u32"),
              "destination": array("u32"), "item_code": "u32",
              "destination_class": ("enum", ("stockpile", "hearth", "other")),
              "observation": ("enum", ("hunt_to_haul",)), "taken_at": "u64"}
HAUL_TERMINAL = {"agent_id": "u64", "taken_at": "u64", "executed_tick": "u64",
                 "outcome": ("enum", ("PROVEN_FINISH", "PROVEN_CANCEL", "UNRESOLVED_TERMINAL")),
                 "predicate_id": "str", "source": array("u32"), "destination": array("u32"),
                 "alive_after_tick": "bool", "job_present_after_tick": "bool", "carry_present_after_tick": "bool"}
NATURAL = {"schema_version": "u32", "manifest_sha256": "hash", "arm": ("enum", ARMS), "seed": "u64",
           "ticks_per_day": "u64", "horizon_ticks": "u64", "status": STATUS, "invalid_reasons": array(REASON),
           "days": array(DAY), "craft_events": array(CRAFT), "tally_initial": array(TALLY), "tally_final": array(TALLY),
           "command_receipts": array(COMMAND_RECEIPT), "wave_transitions": array(WAVE_TRANSITION),
           "caravan_generations": array(CARAVAN), "purchase_receipts": array(PURCHASE),
           "haul_starts": array(HAUL_START), "haul_terminals": array(HAUL_TERMINAL), "conservation": CONSERVATION}
RECT = dict.fromkeys(("x0", "y0", "x1", "y1"), "u32")
COMMANDS = {name: RECT for name in ("DesignateChop", "CancelArea", "AddStockpile", "RemoveZoneArea",
    "DesignateQuarry", "DesignateDig", "DesignateDeconstruct", "DesignateClay", "AddHuntZone", "CancelHunt")}
COMMANDS.update({
    "DesignateChopTiles": {"tiles": array(array("u32"))},
    "PlaceBlueprint": {"kind": "str", "x": "u32", "y": "u32"},
    "PlaceLine": {**RECT, "kind": "str"}, "PlaceRect": {**RECT, "kind": "str", "hollow": "bool"},
    "AddBill": {"workshop": "u32", "recipe": "str", "mode": "bill_mode"},
    "RemoveBill": {"workshop": "u32", "bill": "u32"},
    "SetBillMode": {"workshop": "u32", "bill": "u32", "mode": "bill_mode"},
    "MoveBill": {"workshop": "u32", "bill": "u32", "up": "bool"},
    "SetBillSuspended": {"workshop": "u32", "bill": "u32", "suspended": "bool"},
    "SetWorkPriority": {"agent": "u64", "work": ("enum", ("Craft", "Build", "Quarry", "Chop", "Split", "Haul", "Mine", "Hunt")), "level": "u8"},
    "SetRoomRole": {"room": "u32", "role": optional(("enum", ("Unassigned", "Bedroom", "Storehouse", "Workshop", "Hall")))},
    "OrderTeach": {"teacher": "u64", "learner": "u64", "knowledge": "u8"}, "CancelTeach": {"order": "u32"},
    "DebugKill": {"agent": "u64"}, "DebugGive": {"kind": "str", "count": "u16", "x": "u32", "y": "u32"},
    "SetDrafted": {"agent": "u64", "drafted": "bool"}, "DraftMove": {"agent": "u64", "x": "u32", "y": "u32"},
    "DraftAttack": {"agent": "u64", "animal": "u32"}, "DraftAttackPerson": {"agent": "u64", "target": "u64"},
    "DesignateHunt": {"animal": "u32"}, "SetDressOrder": {"on": "bool"},
    "AcceptMigrants": {"wave": "u32"}, "DeclineMigrants": {"wave": "u32"},
    "Trade": {"caravan": "u32", "give": array(array("u32")), "take": array(array("u16"))},
    "SetSteward": {"on": "bool", "scope": ("enum", ("All", "Housing", "Food"))},
    "Steward": {"rule": "u16", "command": "command"},
})


def check_command(value, where):
    require(type(value) is dict and len(value) == 1, f"{where}: externally tagged command required")
    name = next(iter(value))
    require(name in COMMANDS, f"{where}: unsupported command {name}")
    fields = value[name]
    validate(fields, COMMANDS[name], where)
    if name == "DesignateChopTiles":
        require(all(len(tile) == 2 for tile in fields["tiles"]), "command tile width")
    if name == "Trade":
        require(all(len(row) == 3 and row[2] <= 65535 for row in fields["give"]), "trade give width")
        require(all(len(row) == 2 for row in fields["take"]), "trade take width")


ARITHMETIC = {
    "schema_version": "u32", "xp_table": array("u32"), "quality_bonus_table": array("u32"),
    "qual_cap": array("u8"), "quality_base": "i32", "quality_per_level": "i32",
    "stream_quality": "u64", "stream_apparel": "u64", "stream_trade": "u64",
    "rng": {"algorithm": "str", "below": "str", "add": "u64", "mul1": "u64", "mul2": "u64"},
    "primary_recipe": {"code": "u32", "identity": "str", "output_item_code": "u32", "output_count": "u16",
        "input_item_code": "u32", "input_count": "u16", "work_ticks": "u32", "skill_code": "u8",
        "quality_add": "i32", "material_term": "material_term", "gate_mask": "u32",
        "material_id": "u32", "material_hardness": "u16"},
    "garments": array({"garment_arm": ("enum", ("hide_wrap", "fur_cloak")), "item_code": "u32", "cold_permille": "u32"}),
    "knowledge_nodes": array({"node_id": "u8", "threshold": "u16", "era": ("enum", ("paleolithic", "neolithic"))}),
    "trade": {"goods": array({"authored_index": "u32", "item_code": "u32", "made_by": optional("u8"),
        "tag": ("enum", ("lacked", "staple")), "count_min": "u16", "count_max": "u16"}),
        "stacks_min": "u16", "stacks_max": "u16", "quality_min": "u8", "quality_max": "u8", "practice_days": "u64"},
    "sources": array({"commit": "str", "repo_path": "str", "line_start": "u32", "line_end": "u32",
                      "sha256": "hash", "artifact_path": "str"}),
}


def check_arithmetic(data: Record, base: Path) -> None:
    validate(data, ARITHMETIC, "arithmetic manifest")
    constants = {"schema_version": 1, "xp_table": [15 * n * (n + 1) for n in range(21)],
        "quality_bonus_table": [27 - abs(2 * n - 5) for n in range(-10, 16)], "qual_cap": CAPS,
        "quality_base": 30, "quality_per_level": 3, "stream_quality": 0x5155414C4954595F,
        "stream_apparel": 0x4150504152454C21, "stream_trade": 0x5452414445525321,
        "rng": {"algorithm": "splitmix64-chain-v1", "below": "multiply-high-u64", "add": 0x9E3779B97F4A7C15,
                "mul1": 0xBF58476D1CE4E5B9, "mul2": 0x94D049BB133111EB}}
    for key, value in constants.items():
        require(data[key] == value, f"unregistered arithmetic constant: {key}")
    p = data["primary_recipe"]
    for key, expected in {"code": 0, "identity": "knap_blade", "output_count": 1, "input_count": 1,
        "work_ticks": 45, "quality_add": 0, "material_hardness": 700,
        "material_term": {"kind": "hardness", "offset": 50, "divisor": 5}}.items():
        require(p[key] == expected, f"unregistered primary recipe {key}")
    require(p["gate_mask"] != 0, "primary gate missing")
    check_coverage([g["garment_arm"] for g in data["garments"]], ["hide_wrap", "fur_cloak"], "garment arm")
    require(len({g["item_code"] for g in data["garments"]}) == 2, "duplicate garment code")
    for g in data["garments"]:
        require(g["cold_permille"] == {"hide_wrap": 700, "fur_cloak": 500}[g["garment_arm"]], "unregistered garment multiplier")
    nodes = data["knowledge_nodes"]
    ids = [n["node_id"] for n in nodes]
    require(ids == sorted(set(ids)) and ids and max(ids) < 32, "knowledge node order/width")
    require(all(0 < n["threshold"] <= 1000 for n in nodes), "knowledge threshold range")
    trade = data["trade"]
    for prefix in ("stacks", "quality"):
        require(0 < trade[prefix + "_min"] <= trade[prefix + "_max"], f"trade {prefix} bounds")
    require(trade["quality_max"] <= 100, "trade quality range")
    require([g["authored_index"] for g in trade["goods"]] == list(range(len(trade["goods"]))), "authored goods order")
    for g in trade["goods"]:
        require(0 < g["count_min"] <= g["count_max"], "goods count bounds")
        require(g["made_by"] is None or g["made_by"] in ids, "unknown producing node")
    sources = data["sources"]
    required_suffixes = ("/rng.rs", "/colony/skills.rs", "/colony/knowledge.rs", "/colony/recipes.ron",
                         "/colony/items.ron", "/colony/knowledge.ron", "/colony/neighbours.ron", "/stone/flint.ron",
                         "/jobs/craft.rs", "/settlement/fire.rs", "/neighbours/trade.rs")
    paths = [s["repo_path"] for s in sources]
    require(len(paths) == len(set(paths)), "duplicate source path")
    require(all(any(p.endswith(suffix) for p in paths) for suffix in required_suffixes), "required source artifact missing")
    for source in sources:
        require(re.fullmatch(r"[0-9a-f]{40}", source["commit"]) is not None, "source commit syntax")
        raw = raw_ref(base, {"path": source["artifact_path"], "sha256": source["sha256"]})
        require(1 <= source["line_start"] <= source["line_end"] <= len(raw.splitlines()), "source line range")


def check_caravan(row: Record, seed: int, arithmetic: Record, metadata: Record) -> Record:
    validate(row, CARAVAN, "caravan")
    require(row["policy"] == metadata["policy"], "caravan policy mismatch")
    require(row["executed_tick"] % 1440 == 0 and row["executed_tick"] < 69120, "caravan generation schedule")
    require(row["tech_progress_permille"] <= 1000, "tech progress range")
    require(row["practice_days"] == arithmetic["trade"]["practice_days"], "practice window mismatch")
    nodes = arithmetic["knowledge_nodes"]
    ids = [n["node_id"] for n in nodes]
    inputs = row["practitioner_inputs"]
    agent_ids = [p["agent_id"] for p in inputs]
    require(agent_ids == sorted(set(agent_ids)), "practitioner agent order")
    for person in inputs:
        keys = [k["node_id"] for k in person["knowledge"]]
        require(keys == sorted(set(keys)) and set(keys) <= set(ids), "practitioner knowledge order/identity")
        require(all(k["proficiency"] <= 1000 for k in person["knowledge"]), "practitioner proficiency")
    today = min(row["executed_tick"] // 1440, 65535)
    counts = {n["node_id"]: sum(not p["is_child"] and any(k["node_id"] == n["node_id"]
        and k["proficiency"] >= n["threshold"] and max(0, today - k["last_used"]) <= row["practice_days"]
        for k in p["knowledge"]) for p in inputs) for n in nodes}
    require(row["practitioner_rows"] == [{"node_id": n, "count": counts[n]} for n in ids], "fresh practitioner counts mismatch")
    eras = {n["node_id"]: n["era"] for n in nodes}
    eligible = []
    for good in arithmetic["trade"]["goods"]:
        node = good["made_by"]
        classification = None
        if good["tag"] == "staple":
            classification = "staple"
        elif node is not None and row["neighbour_known_mask"] & (1 << node):
            if row["policy"] == "old_3_1":
                if not row["old_all_knowledge_practice_mask"] & (1 << node):
                    classification = "lacked"
            elif eras[node] == "neolithic" and row["tech_progress_permille"] < 750 and counts[node] == 0:
                classification = "rare"
            elif counts[node] <= 2:
                classification = "scarce"
        if classification is not None:
            weight = (1 if classification == "staple" else 3) if row["policy"] == "old_3_1" else {"staple": 10, "rare": 1, "scarce": 30}[classification]
            eligible.append({**{k: v for k, v in good.items() if k != "tag"}, "classification": classification, "weight": weight})
    require(row["eligible_rows"] == eligible, "eligible goods table mismatch")
    weights = [r["weight"] for r in eligible]
    total = sum(weights)
    require(total > 0, "empty trade denominator")
    trade = arithmetic["trade"]
    def draw(slot, bound):
        return keyed_below(seed, arithmetic["stream_trade"], row["caravan_id"], 0, slot, 0, bound)
    stacks = trade["stacks_min"] + draw(0, trade["stacks_max"] - trade["stacks_min"] + 1)
    expected = []
    classes = Counter()
    for i in range(stacks):
        good = eligible[weighted_index(draw(0x10 + i, total), weights)]
        expected.append({"item_code": good["item_code"], "count": good["count_min"] + draw(0x40 + i, good["count_max"] - good["count_min"] + 1),
                         "quality": trade["quality_min"] + draw(0x80 + i, trade["quality_max"] - trade["quality_min"] + 1)})
        classes[good["classification"]] += 1
    require(row["initial_goods"] == expected, "initial goods keyed reconstruction mismatch")
    return {"caravan_id": row["caravan_id"], "total_weight": total, "table": eligible,
            "rare_probability": rational(Fraction(sum(r["weight"] for r in eligible if r["classification"] == "rare"), total)),
            "offered_stack_classes": dict(classes), "old_practice_mask_scope": "all Knowledge components; separately observed"}


def envelope(data, schema, metadata, manifest_hash):
    validate(data, schema)
    require(data["schema_version"] == 1, "schema version")
    require(data["manifest_sha256"] == manifest_hash and data["arm"] == metadata["arm"], "envelope manifest/arm mismatch")
    require(data["status"] == "VALID" and not data["invalid_reasons"],
            f"producer status {data['status']}: {data['invalid_reasons']}")


def monotonic_tallies(before, after):
    require(set(before) <= set(after), "persistent tally key disappeared (including deceased crafter)")
    for key, (count, quality) in after.items():
        old_count, old_quality = before.get(key, (0, 0))
        require(count >= old_count and quality >= old_quality, "negative tally delta")
        require(quality - old_quality <= 100 * (count - old_count), "quality change without matching craft count")


def check_natural(data: Record, arithmetic: Record, metadata: Record, manifest_hash: str,
                  *, require_steward: bool = True) -> Record:
    envelope(data, NATURAL, metadata, manifest_hash)
    require(data["ticks_per_day"] == 1440 and data["horizon_ticks"] == 69120, "natural calendar/horizon")
    days = data["days"]
    require([d["day"] for d in days] == list(range(1, 49)), "missing/duplicate/unordered day")
    events = data["craft_events"]
    keys = [(r["executed_tick"], r["agent_id"], r["recipe_code"]) for r in events]
    require(keys == sorted(set(keys)), "duplicate/unordered craft event")
    primary = arithmetic["primary_recipe"]["code"]
    results = []
    for row in events:
        require(row["seed"] == data["seed"] and row["executed_tick"] < 69120, "craft seed/tick")
        require(row["recipe_code"] == primary, "producer contract emits only primary crafts")
        results.append(check_craft(row, arithmetic, metadata))
    running = tally_map(data["tally_initial"])
    require(not running, "fresh natural world has nonempty initial craft tally")
    event_index = 0
    previous = running.copy()
    previous_day = None
    for day in days:
        require(day["executed_tick"] + 1 == day["day"] * 1440, "day timestamp")
        require(day["insulating_wearers"] <= day["population"], "wearers exceed population")
        garments = day["garment_counts"]
        require([g["item_code"] for g in garments] == sorted({g["item_code"] for g in garments}), "garment duplicate/order")
        require(sum(g["count"] for g in garments if g["cold_permille"] < 1000) == day["insulating_wearers"], "garment wearer total")
        require(sum(g["count"] for g in garments) <= day["population"], "garment count exceeds population")
        for group in ("clothed", "unclothed"):
            rate(day["cold"][group + "_cold"], day["cold"][group + "_eligible"])
        require(not day["ledger_violation_ticks"] and not day["reservation_violation_ticks"], "daily conservation violation")
        cancel_keys = [r["reason"] for r in day["cancel_totals"]]
        require(len(cancel_keys) == len(set(cancel_keys)), "duplicate cancellation counter")
        if previous_day is not None:
            for key in ("deaths_total", "jobs_done_total"):
                require(day[key] >= previous_day[key], "cumulative counter decreased")
            for group in ("cold", "meat"):
                require(all(day[group][key] >= value for key, value in previous_day[group].items()), "cumulative grouped counter decreased")
            old_cancels = {r["reason"]: r["count"] for r in previous_day["cancel_totals"]}
            cancels = {r["reason"]: r["count"] for r in day["cancel_totals"]}
            require(all(cancels.get(k, 0) >= v for k, v in old_cancels.items()), "cumulative cancellation decreased")
        actual = tally_map(day["craft_tallies"])
        monotonic_tallies(previous, actual)
        while event_index < len(events) and events[event_index]["executed_tick"] <= day["executed_tick"]:
            event = events[event_index]
            key = (event["agent_id"], primary)
            require(running.get(key, (0, 0)) == (event["tally_before_count"], event["tally_before_quality_sum"]), "event tally chain mismatch")
            running[key] = (event["tally_after_count"], event["tally_after_quality_sum"])
            event_index += 1
        expected_primary = {key: value for key, value in running.items() if key[1] == primary}
        actual_primary = {key: value for key, value in actual.items() if key[1] == primary}
        require(actual_primary == expected_primary, "primary day event/tally reconciliation failed")
        previous, previous_day = actual, day
    require(event_index == len(events) and tally_map(data["tally_final"]) == previous, "final tally reconciliation failed")
    receipts = data["command_receipts"]
    ordinals = [r["ordinal"] for r in receipts]
    require(ordinals == sorted(set(ordinals)), "command ordinal order/duplicate")
    require(all(r["upcoming_tick"] < 69120 for r in receipts), "command outside horizon")
    if require_steward:
        require(any(r["upcoming_tick"] == 0 and r["command"].get("SetSteward", {}).get("on") is False
                    and r["result"]["accepted"] for r in receipts), "missing accepted tick0 SetSteward OFF")
        require(not any(r["command"].get("SetSteward", {}).get("on") is True or "Steward" in r["command"] for r in receipts), "steward enabled/issued during fixed OFF run")
    generations = data["caravan_generations"]
    caravan_ids = [r["caravan_id"] for r in generations]
    require(len(caravan_ids) == len(set(caravan_ids)), "duplicate caravan generation")
    caravans = [check_caravan(row, data["seed"], arithmetic, metadata) for row in generations]
    receipt_by_id = {r["ordinal"]: r for r in receipts}
    purchase_keys = [p["command_ordinal"] for p in data["purchase_receipts"]]
    require(len(purchase_keys) == len(set(purchase_keys)), "duplicate purchase receipt")
    require(set(purchase_keys) == {r["ordinal"] for r in receipts if "Trade" in r["command"]}, "trade receipt coverage")
    purchases = []
    for purchase in data["purchase_receipts"]:
        receipt = receipt_by_id[purchase["command_ordinal"]]
        require(purchase["executed_tick"] == receipt["upcoming_tick"] and purchase["caravan_id"] == receipt["command"]["Trade"]["caravan"], "purchase command identity")
        for counter in ("unknown_bought", "eurekas"):
            require(purchase[counter + "_after"] >= purchase[counter + "_before"], "purchase counter decreased")
        before, after = purchase["goods_before"], purchase["goods_after"]
        if not receipt["result"]["accepted"]:
            require(before == after and purchase["unknown_bought_before"] == purchase["unknown_bought_after"]
                    and purchase["eurekas_before"] == purchase["eurekas_after"], "rejected trade mutation")
        classified = Counter()
        if receipt["result"]["accepted"]:
            expected_after = [dict(g) for g in before]
            take = receipt["command"]["Trade"]["take"]
            for index, count in take:
                require(index < len(expected_after) and 0 < count <= expected_after[index]["count"], "accepted trade take exceeds goods")
                expected_after[index]["count"] -= count
                generation = next((g for g in caravans if g["caravan_id"] == purchase["caravan_id"]), None)
                require(generation is not None, "purchase missing initial caravan generation")
                classes = {r["classification"] for r in generation["table"] if r["item_code"] == before[index]["item_code"]}
                label = next(iter(classes)) if len(classes) == 1 else "UNRESOLVED_CLASSIFICATION"
                classified[label] += count
            require(after == [g for g in expected_after if g["count"] > 0], "accepted trade goods delta mismatch")
        purchases.append({**purchase, "purchased_units_by_generation_class": dict(classified),
                          "eureka_attribution": "aggregate observed delta; no unique asynchronous causal attribution"})
    starts = data["haul_starts"]
    start_map = {(s["agent_id"], s["taken_at"]): s for s in starts}
    require(len(start_map) == len(starts), "duplicate haul start")
    for start in starts:
        require(start["executed_tick"] == start["taken_at"] < 69120 and len(start["source"]) == len(start["destination"]) == 2, "haul start identity")
    terminal_keys = []
    for terminal in data["haul_terminals"]:
        key = (terminal["agent_id"], terminal["taken_at"])
        terminal_keys.append(key)
        require(key in start_map, "haul terminal without observed start")
        start = start_map[key]
        require(start["executed_tick"] <= terminal["executed_tick"] < 69120 and start["source"] == terminal["source"] and start["destination"] == terminal["destination"], "haul terminal identity")
        require(terminal["outcome"] == "UNRESOLVED_TERMINAL" and terminal["predicate_id"] == "job_disappeared_or_replaced_no_unique_terminal_proof", "no unique natural terminal proof registered")
    require(len(terminal_keys) == len(set(terminal_keys)), "duplicate haul terminal")
    conservation = data["conservation"]
    require(not conservation["ledger_violation_ticks"] and not conservation["reservation_violation_ticks"], "conservation violation")
    require(days[-1]["meat"]["hauls"] >= len(starts), "observed starts exceed authoritative meat hauls")
    require(conservation["unresolved_start_count"] == days[-1]["meat"]["hauls"] - len(starts), "unresolved starts mismatch")
    require(conservation["unresolved_terminal_count"] == len(data["haul_terminals"]), "unresolved terminals mismatch")
    def window(lo, hi):
        rows = [e for e in events if lo <= e["executed_tick"] < hi]
        total = sum(r["actual_quality"] for r in rows)
        return {"count": len(rows), "quality_sum": total, "mean": rational(Fraction(total, len(rows))) if rows else None,
                "status": "VALID" if rows else "NOT_MEASURED"}
    early, late = window(0, 17280), window(51840, 69120)
    delta = Fraction(late["quality_sum"], late["count"]) - Fraction(early["quality_sum"], early["count"]) if early["count"] and late["count"] else None
    cold = days[-1]["cold"]
    rates = {group: rate(cold[group + "_cold"], cold[group + "_eligible"]) for group in ("clothed", "unclothed")}
    ratio = None
    if cold["clothed_eligible"] and cold["unclothed_eligible"] and cold["unclothed_cold"]:
        ratio = rational(Fraction(cold["clothed_cold"] * cold["unclothed_eligible"], cold["clothed_eligible"] * cold["unclothed_cold"]))
    return {"seed": data["seed"], "status": "VALID", "f4a": days[29]["insulating_wearers"] >= 3 and days[47]["insulating_wearers"] >= 5,
            "wearers_day30": days[29]["insulating_wearers"], "wearers_day48": days[47]["insulating_wearers"],
            "f9a": late["count"] >= 5, "craft_count": len(events), "arithmetic_pass": bool(events),
            "early": early, "late": late, "quality_delta": rational(delta) if delta is not None else None,
            "legacy_plus8_primary_only": delta >= 8 if delta is not None else None,
            "level_distribution": dict(sorted(Counter(str(e["level_before"]) for e in events).items())),
            "xp_distribution": dict(sorted(Counter(str(e["xp_before"]) for e in events).items())),
            "capped_share": rate(sum(r["quality"] < r["base"] for r in results), len(events)),
            "natural_cold": {"rates": rates, "ratio": ratio, "status": "VALID" if ratio is not None else "NOT_MEASURED",
                             "reason": None if ratio is not None else "zero group denominator or zero unclothed reference", "causal": False},
            "population_end": days[-1]["population"], "population_daily_max": max(d["population"] for d in days),
            "deaths": days[-1]["deaths_total"], "meat": days[-1]["meat"], "cancel_totals": days[-1]["cancel_totals"],
            "command_receipts": receipts, "wave_transitions": data["wave_transitions"], "caravans": caravans,
            "population_observations": {
                "accept_command_attempts": sum("AcceptMigrants" in r["command"] for r in receipts),
                "accept_command_rejected": sum("AcceptMigrants" in r["command"] and not r["result"]["accepted"] for r in receipts),
                "capacity_blocked": sum("AcceptMigrants" in r["command"] and not r["result"]["accepted"]
                                        and r["result"]["reason"] == "blocked" for r in receipts),
                "arrival_acceptance_cleared": sum(r["outcome"] == "acceptance_cleared_waiting" for r in data["wave_transitions"]),
                "arrival_population_limit": "tick boundary only; exact intra-tick admission belongs to controlled P tests"},
            "purchase_receipts": purchases, "observed_haul_starts": len(starts),
            "haul_destinations": dict(Counter(s["destination_class"] for s in starts)),
            "conservation": conservation, "ongoing_observed_hauls": len(starts) - len(terminal_keys),
            "optional_full_m55_controls": "NOT_RUN"}


COLD_ROW = {"seed": "u64", "agent_id": "u64", "lineage_key": "u64", "executed_tick": "u64",
            "garment_arm": ("enum", ("none", "hide_wrap", "fur_cloak")), "item_code": optional("u32"),
            "cold_permille": "u32", "observed_cold": "bool", "member": "bool", "season": "u8",
            "in_cold_window": "bool", "burn_rate": "u32", "no_roof": "bool", "wood_zero": "bool",
            "excluded_markers_absent": "bool", "damage_disabled": "bool"}
LEDGER = {"item_code": "u32", **dict.fromkeys(("created", "consumed", "traded_in", "traded_out", "ground", "carried", "held", "delivered", "shelved"), "u64")}
CRAFT_EXTRA = {"fixture_id": "str", "cap_arm": ("enum", ("unrestricted", "restricted")),
               "target_level": "u8", "target_proficiency": "u16", "admission_eligible": "bool", "inputs_consumed": "bool",
               "ledger_before": array(LEDGER), "ledger_after": array(LEDGER), "output_delta": "u64",
               "bill_done_before": "u32", "bill_done_after": "u32", "setup_finished_tick": "u64",
               "add_bill_result": RESULT, "paid_working_observed": "bool", "ticks_left_at_paid_boundary": "u32",
               "primary_tally_before_payment": TALLY, "primary_tally_at_paid_boundary": TALLY,
               "tally_key_present_before_payment": "bool", "cleanup_complete": "bool", "ledger_balanced": "bool", "reservation_check_passed": "bool"}


def fixture_schema(row):
    return {"schema_version": "u32", "manifest_sha256": "hash", "arm": ("enum", ARMS),
            "fixture_kind": ("enum", ("cold_filter", "craft_finish")), "status": STATUS,
            "invalid_reasons": array(REASON), "rows": array(row)}


def check_cold_fixture(data, arithmetic, metadata, manifest_hash, seeds):
    envelope(data, fixture_schema(COLD_ROW), metadata, manifest_hash)
    require(data["fixture_kind"] == "cold_filter", "wrong cold fixture kind")
    rows = data["rows"]
    keys = [(r["seed"], r["agent_id"], r["executed_tick"], r["garment_arm"]) for r in rows]
    expected = [(seed, agent, tick, garment) for seed in seeds for agent in range(1, 9)
                for tick in COLD_TICKS for garment in ("none", "hide_wrap", "fur_cloak")]
    check_coverage(keys, expected, "cold seed/agent/tick/garment")
    garments = {g["garment_arm"]: g for g in arithmetic["garments"]}
    for row in rows:
        require(row["agent_id"] == row["lineage_key"], "cold lineage identity")
        require(all(row[key] for key in ("member", "in_cold_window", "no_roof", "wood_zero", "excluded_markers_absent", "damage_disabled")), "cold eligibility assertion failed")
        require(row["season"] == 3 and row["burn_rate"] > 0, "cold season/burn precondition")
        garment = garments.get(row["garment_arm"], {"item_code": None, "cold_permille": 1000})
        require(row["item_code"] == garment["item_code"] and row["cold_permille"] == garment["cold_permille"], "cold garment metadata")
        expected_cold = keyed_below(row["seed"], arithmetic["stream_apparel"], row["lineage_key"], row["executed_tick"], 0, 0, 1000) < row["cold_permille"]
        require(row["observed_cold"] == expected_cold, "cold keyed mismatch")
    results = {}
    for garment in ("none", "hide_wrap", "fur_cloak"):
        group = [r for r in rows if r["garment_arm"] == garment]
        n, observed = len(group), sum(r["observed_cold"] for r in group)
        p_sum = sum(r["cold_permille"] for r in group)
        variance_sum = sum(r["cold_permille"] * (1000 - r["cold_permille"]) for r in group)
        sigma = (1000 * observed - p_sum) ** 2 <= 9 * variance_sum
        protection = 5 * observed <= 4 * n if garment != "none" else observed == n
        results[garment] = {"N": n, "O": observed, "E": rational(Fraction(p_sum, 1000)),
                            "V": rational(Fraction(variance_sum, 1000000)), "keyed_mismatches": 0,
                            "three_sigma_pass": sigma, "retention_bar_pass": protection, "pass": sigma and protection}
    return {"status": "VALID", "pass": all(r["pass"] for r in results.values()), "garments": results}


def check_ledgers(rows):
    validate(rows, array(LEDGER), "ledger")
    keys = [r["item_code"] for r in rows]
    require(keys == sorted(set(keys)), "ledger item ordering")
    for row in rows:
        supply = row["created"] + row["traded_in"]
        accounted = sum(row[k] for k in ("consumed", "traded_out", "ground", "carried", "held", "delivered", "shelved"))
        require(supply == accounted, "raw inventory/ledger conservation mismatch")
    return {r["item_code"]: r for r in rows}


def check_craft_fixture(data, arithmetic, metadata, manifest_hash, seeds):
    envelope(data, fixture_schema({**CRAFT, **CRAFT_EXTRA}), metadata, manifest_hash)
    require(data["fixture_kind"] == "craft_finish", "wrong craft fixture kind")
    rows = data["rows"]
    keys = [(r["seed"], r["target_level"], r["cap_arm"]) for r in rows]
    check_coverage(keys, [(s, level, cap) for s in seeds for level in (1, 6) for cap in ("unrestricted", "restricted")], "craft seed/level/cap")
    require(len({r["fixture_id"] for r in rows}) == len(rows), "duplicate fixture id")
    primary = arithmetic["primary_recipe"]
    pairs = {}
    for row in rows:
        raw = {k: row[k] for k in CRAFT}
        result = check_craft(raw, arithmetic, metadata, controlled=True)
        require(row["agent_id"] == row["bill_id"] == 1 and row["executed_tick"] == 27360, "controlled craft keyed identity")
        require(row["recipe_code"] == primary["code"] and row["material_id"] == primary["material_id"]
                and row["material_hardness"] == 700 and row["persistence"] is not None and row["persistence"] < 600,
                "controlled craft material/temperament")
        require(row["knowledge_enabled"] and row["skill_present"] and row["level_before"] == row["target_level"]
                and row["xp_before"] == {1: 30, 6: 630}[row["target_level"]], "controlled craft level/XP")
        proficiency = 1000 if row["cap_arm"] == "unrestricted" else 300
        require(row["target_proficiency"] == proficiency and all(n["entry_present"] and n["proficiency"] == proficiency
            and n["proficiency"] >= n["admission_threshold"] for n in row["gating_nodes"]), "controlled admission proficiency")
        require(result["cap"] == (100 if proficiency == 1000 else 45), "controlled cap")
        require(all(row[k] for k in ("admission_eligible", "inputs_consumed", "paid_working_observed", "cleanup_complete", "ledger_balanced", "reservation_check_passed")), "controlled ordinary payment/cleanup invariant")
        require(row["setup_finished_tick"] < 27360 and row["add_bill_result"]["accepted"]
                and row["add_bill_result"]["count"] == 1 and row["add_bill_result"]["reason"] is None, "controlled setup command")
        require(row["ticks_left"] == row["ticks_left_at_paid_boundary"] > 0, "controlled real Working boundary")
        before_payment = row["primary_tally_before_payment"]
        require(before_payment == row["primary_tally_at_paid_boundary"], "craft completed before explicit finish")
        require((before_payment["agent_id"], before_payment["recipe_code"], before_payment["completed_count"], before_payment["quality_sum"])
                == (1, primary["code"], row["tally_before_count"], row["tally_before_quality_sum"]), "paid tally identity")
        require(row["tally_key_present_before_payment"] or (before_payment["completed_count"] == before_payment["quality_sum"] == 0), "absent tally must be zero")
        before, after = check_ledgers(row["ledger_before"]), check_ledgers(row["ledger_after"])
        require(set(before) == set(after) == {primary["input_item_code"], primary["output_item_code"]}, "controlled ledger item coverage")
        for code in before:
            for counter in ("created", "consumed", "traded_in", "traded_out"):
                expected = primary["input_count"] if code == primary["input_item_code"] and counter == "consumed" else (
                    primary["output_count"] if code == primary["output_item_code"] and counter == "created" else 0)
                require(after[code][counter] - before[code][counter] == expected, "controlled ledger counter delta")
        require(row["output_delta"] == 1 and row["bill_done_after"] - row["bill_done_before"] == 1, "controlled output/bill delta")
        pairs[(row["seed"], row["cap_arm"], row["target_level"])] = row
    for seed in seeds:
        for cap in ("unrestricted", "restricted"):
            low, high = pairs[(seed, cap, 1)], pairs[(seed, cap, 6)]
            for key in ("workshop_id", "bill_id", "at", "material_id", "persistence", "gating_nodes"):
                # last_used is a setup artifact; proficiency and admission are already checked.
                if key != "gating_nodes":
                    require(low[key] == high[key], "paired controlled fixture inputs differ")
            if cap == "unrestricted":
                require(high["actual_quality"] - low["actual_quality"] == 15, "unrestricted level difference")
    return {"status": "VALID", "pass": True, "completions": len(rows), "arithmetic_mismatches": 0,
            "claim": "finish-boundary quality; not work timing or skill speed"}


INDEX_ARM = {"arm": ("enum", ARMS), "metadata": REF, "arithmetic": REF, "provenance": REF,
             "natural": array(REF), "cold_fixture": REF, "craft_fixture": REF}
INDEX = {"schema_version": "u32", "mode": ("enum", ("evaluation", "calibration")),
         "seeds": array("u64"), "arms": array(INDEX_ARM)}


def unpack_natural(value):
    """Only B4 has this closed schema; legacy summaries are retained verbatim."""
    require(type(value) is dict, "natural artifact must be object")
    if "seeds" in value:
        require(type(value["seeds"]) is list, "summary seeds must be list")
        entries = value["seeds"]
    else:
        entries = [value]
    result = []
    for entry in entries:
        require(type(entry) is dict, "seed summary must be object")
        if "b4" in entry:
            block = entry["b4"]
            require(type(block) is dict and entry.get("seed") == block.get("seed"), "legacy/B4 seed mismatch")
            result.append((block, {k: entry[k] for k in ("m5", "m55", "deaths", "alive_end") if k in entry}))
        else:
            result.append((entry, {}))
    return result


def mean_median(values):
    require(bool(values), "empty paired metric")
    ordered = sorted(values)
    n = len(values)
    midpoint = Fraction(ordered[n // 2]) if n % 2 else Fraction(ordered[n // 2 - 1] + ordered[n // 2], 2)
    return {"mean": rational(Fraction(sum(values), n)), "median": rational(midpoint)}


def paired_effects(arms, seeds):
    metrics = ("wearers_day30", "wearers_day48", "craft_count", "population_end", "population_daily_max",
               "deaths", "observed_haul_starts")
    pairs = [("B0", "BP", "P conditional on baseline"), ("BP", "BPT", "T conditional on P"),
             ("BPT", "BPTH", "H conditional on P and T"), ("B0", "BPTH", "total sequential intervention")]
    output = []
    for before, after, label in pairs:
        if before not in arms or after not in arms:
            output.append({"before": before, "after": after, "status": "NOT_RUN", "interpretation": label})
            continue
        a = {r["seed"]: r for r in arms[before]["natural"] if r.get("status") == "VALID"}
        b = {r["seed"]: r for r in arms[after]["natural"] if r.get("status") == "VALID"}
        if set(a) != set(seeds) or set(b) != set(seeds):
            output.append({"before": before, "after": after, "status": "OBSERVER_INVALID", "reason": "incomplete paired seed coverage"})
            continue
        per_seed = []
        for seed in seeds:
            delta = {metric: b[seed][metric] - a[seed][metric] for metric in metrics}
            delta["late_crafts"] = b[seed]["late"]["count"] - a[seed]["late"]["count"]
            for key in MEAT:
                delta["meat_" + key] = b[seed]["meat"][key] - a[seed]["meat"][key]
            for key in ("unresolved_start_count", "unresolved_terminal_count"):
                delta[key] = b[seed]["conservation"][key] - a[seed]["conservation"][key]
            per_seed.append({"seed": seed, "delta": delta})
        output.append({"before": before, "after": after, "status": "VALID", "interpretation": label,
                       "per_seed": per_seed, "summary": {key: mean_median([r["delta"][key] for r in per_seed]) for key in per_seed[0]["delta"]},
                       "claim": "conditional sequential differences, not independent factorial effects"})
    return output


def score_index(index, base, *, calibration=False):
    validate(index, INDEX, "evaluation index")
    require(index["schema_version"] == 1, "index version")
    require((index["mode"] == "calibration") == calibration, "explicit calibration flag/index label mismatch")
    seeds = index["seeds"]
    require(seeds == sorted(set(seeds)) and seeds, "index seed uniqueness/order")
    if calibration:
        require(set(seeds) <= set(range(8)), "calibration seeds must be subset of 0..7")
    else:
        require(seeds == list(range(1100, 1132)), "evaluation requires all 32 fixed seeds")
    arm_names = [a["arm"] for a in index["arms"]]
    require(arm_names and len(arm_names) == len(set(arm_names)), "missing/duplicate index arms")
    if not calibration:
        check_coverage(arm_names, list(ARMS), "paired arm")
    report = {"schema_version": 1, "mode": index["mode"], "status": "VALID", "acceptance": "NOT_CLAIMED" if calibration else "PENDING",
              "seeds": seeds, "arms": {}, "invalid": [], "not_run": ["F2", "F5", "F8-control", "F10-demography", "complete-M55 controls"],
              "scope": "B4 only; no new natural outcome improvement bar"}
    fixtures = {}
    def invalid(arm, artifact, error):
        row = {"arm": arm, "artifact": artifact, "status": "OBSERVER_INVALID", "reason": str(error)}
        report["invalid"].append(row)
        return row
    for entry in index["arms"]:
        name = entry["arm"]
        arm = {"natural": [], "input_refs": entry}
        report["arms"][name] = arm
        try:
            metadata = read_ref(base, entry["metadata"])
            validate(metadata, META_SCHEMA, "metadata")
            require(metadata["schema_version"] == 1 and metadata["arm"] == name, "metadata arm/version")
            policy = "old_3_1" if name in ("B0", "BP") else "scarce_30_10_1"
            require(metadata["policy"] == policy, "arm policy mismatch")
            require(metadata["arithmetic_manifest_sha256"] == entry["arithmetic"]["sha256"], "metadata arithmetic reference mismatch")
            arithmetic = read_ref(base, entry["arithmetic"])
            check_arithmetic(arithmetic, (base / entry["arithmetic"]["path"]).parent)
            # Provenance is a separate, hash-bound root-owned document; its inner
            # extensible build/toolchain fields are not simulation evidence DTOs.
            arm["provenance"] = read_ref(base, entry["provenance"])
        except (Invalid, UnicodeError, OSError) as error:
            invalid(name, "manifests", error)
            continue
        observed_seeds = []
        for ref in entry["natural"]:
            try:
                raw = parse_json(raw_ref(base, ref).decode("utf-8"), legacy_floats=True)
                for data, legacy in unpack_natural(raw):
                    validate(data, NATURAL, "natural")
                    observed_seeds.append(data["seed"])
                    try:
                        result = check_natural(data, arithmetic, metadata, entry["metadata"]["sha256"])
                        result["legacy_primary_summary"] = legacy
                        result["input_ref"] = ref
                        arm["natural"].append(result)
                    except Invalid as error:
                        failure = invalid(name, {"ref": ref, "seed": data["seed"], "producer_reasons": data["invalid_reasons"]}, error)
                        arm["natural"].append({**failure, "seed": data["seed"], "legacy_primary_summary": legacy})
            except (Invalid, UnicodeError, OSError) as error:
                invalid(name, ref, error)
        try:
            check_coverage(observed_seeds, seeds, f"{name} natural seeds")
        except Invalid as error:
            invalid(name, "natural coverage", error)
        fixture_data = {}
        for label, checker in (("cold_fixture", check_cold_fixture), ("craft_fixture", check_craft_fixture)):
            try:
                data = read_ref(base, entry[label])
                arm[label] = checker(data, arithmetic, metadata, entry["metadata"]["sha256"], seeds)
                fixture_data[label] = data
            except (Invalid, UnicodeError, OSError) as error:
                arm[label] = invalid(name, entry[label], error)
        fixtures[name] = fixture_data
        valid = [r for r in arm["natural"] if r["status"] == "VALID"]
        complete = len(valid) == len(seeds) and set(observed_seeds) == set(seeds) and len(observed_seeds) == len(seeds)
        arm["f4a_count"] = sum(r["f4a"] for r in valid)
        arm["f9a_count"] = sum(r["f9a"] for r in valid)
        arm["failing_f4a_seeds"] = [r["seed"] for r in valid if not r["f4a"]]
        arm["failing_f9a_seeds"] = [r["seed"] for r in valid if not r["f9a"]]
        arm["zero_craft_seeds"] = [r["seed"] for r in valid if not r["arithmetic_pass"]]
        arm["not_measured_natural_cold_seeds"] = [r["seed"] for r in valid if r["natural_cold"]["status"] == "NOT_MEASURED"]
        totals = {key: sum(r["natural_cold"]["rates"][group][field] for r in valid)
                  for key, group, field in (("clothed_cold", "clothed", "numerator"), ("clothed_eligible", "clothed", "denominator"),
                    ("unclothed_cold", "unclothed", "numerator"), ("unclothed_eligible", "unclothed", "denominator"))}
        arm["pooled_natural_cold"] = {"status": "VALID" if complete else "OBSERVER_INVALID", "counts": totals,
                                     "missing_worlds": len(seeds) - len(valid), "causal": False}
        if calibration:
            arm["F4"] = arm["F9"] = "NOT_CLAIMED_CALIBRATION"
        elif not complete or any(r["arm"] == name for r in report["invalid"]):
            arm["F4"] = arm["F9"] = "OBSERVER_INVALID"
        else:
            arm["F4"] = "PASS" if arm["f4a_count"] >= 24 and arm["cold_fixture"]["pass"] else "FAIL"
            arm["F9"] = "PASS" if arm["f9a_count"] >= 24 and not arm["zero_craft_seeds"] and arm["craft_fixture"]["pass"] else "FAIL"
    # Mechanics are unchanged across paired builds. Compare actual observations
    # with the same fixed keys, excluding catalogue/build identity metadata.
    reference = None
    for name in arm_names:
        data = fixtures.get(name, {})
        if len(data) != 2:
            continue
        signature = {
            "cold": sorted((r["seed"], r["agent_id"], r["executed_tick"], r["garment_arm"], r["observed_cold"])
                           for r in data["cold_fixture"]["rows"]),
            "craft": sorted((r["seed"], r["cap_arm"], r["target_level"], r["actual_quality"])
                            for r in data["craft_fixture"]["rows"])}
        if reference is None:
            reference = signature
        elif signature != reference:
            invalid(name, "cross-arm controlled observations", "unchanged mechanism observations differ")
    report["paired_effects"] = paired_effects(report["arms"], seeds)
    if report["invalid"]:
        report["status"] = "OBSERVER_INVALID"
        report["acceptance"] = "OBSERVER_INVALID"
    elif not calibration:
        report["acceptance"] = "PASS" if all(a.get("F4") == a.get("F9") == "PASS" for a in report["arms"].values()) else "FAIL"
    return report


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True, help="hash-bound external evaluation index")
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--calibration", action="store_true")
    args = parser.parse_args(argv)
    try:
        raw = args.manifest.read_bytes()
        index = parse_json(raw.decode("utf-8"))
        report = score_index(index, args.manifest.parent, calibration=args.calibration)
        report["index_sha256"] = hashlib.sha256(raw).hexdigest()
    except (Invalid, OSError, UnicodeError) as error:
        report = {"schema_version": 1, "status": "OBSERVER_INVALID", "acceptance": "OBSERVER_INVALID",
                  "invalid": [{"artifact": str(args.manifest), "reason": str(error)}]}
    report["scorer_sha256"] = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(report, ensure_ascii=False, indent=2, allow_nan=False) + "\n", encoding="utf-8")
    return 2 if report["status"] == "OBSERVER_INVALID" else (1 if report["acceptance"] == "FAIL" else 0)


if __name__ == "__main__":
    raise SystemExit(main())
