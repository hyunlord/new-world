# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m unittest discover -s tools/v8 -p test_generic_gate_contract.py
"""Pure gate contracts composed with authenticated source and runtime producers.

These validators do not authenticate caller-supplied booleans or digests. The
runner must derive those facts from Git, actual artifacts and completed children.
"""
import json
import math
import re
from collections.abc import Callable
from functools import wraps
from pathlib import PurePosixPath
from typing import ParamSpec, Protocol, TypeGuard, TypeVar

from mux2_gate_inputs import EvidenceError, Json, Node

P = ParamSpec("P")
T = TypeVar("T")


class GateContractError(ValueError):
    """A rejected contract with an actionable location and stable category."""

    def __init__(self, code: str, path: str, reason: str) -> None:
        self.code: str = code
        self.path: str = path
        super().__init__(f"{code} {path}: {reason}")


def boundary(function: Callable[P, T]) -> Callable[P, T]:
    @wraps(function)
    def checked(*args: P.args, **kwargs: P.kwargs) -> T:
        try:
            return function(*args, **kwargs)
        except EvidenceError as error:
            location, _, reason = str(error).partition(":")
            raise GateContractError("INVALID_EVIDENCE", location or "/", reason or str(error)) from error
    return checked


def require(condition: bool, path: str, reason: str) -> None:
    if not condition:
        raise GateContractError("CONTRACT_REJECTED", path, reason)


def _list_value(value: object) -> TypeGuard[list[object]]:
    return isinstance(value, list)


def _dict_value(value: object) -> TypeGuard[dict[object, object]]:
    return isinstance(value, dict)


def _json_shape(value: object) -> Json:
    """Narrow a decoder result without imposing a numeric or duplicate-key policy."""
    if _list_value(value):
        return [_json_shape(child) for child in value]
    if _dict_value(value):
        result: dict[str, Json] = {}
        for key, child in value.items():
            if not isinstance(key, str):
                raise GateContractError("INVALID_JSON", "/", "non-string object key")
            result[key] = _json_shape(child)
        return result
    match value:
        case None | bool() | int() | float() | str():
            return value
        case _:
            raise GateContractError("INVALID_JSON", "/", "unsupported JSON value")


class _JsonDecoder(Protocol):
    def __call__(
        self, s: str | bytes, *,
        object_pairs_hook: Callable[[list[tuple[str, Json]]], Json] | None,
        parse_constant: Callable[[str], Json] | None,
    ) -> object: ...


def decode_json(
    raw: str | bytes, *,
    pairs_hook: Callable[[list[tuple[str, Json]]], Json] | None = None,
    constant_hook: Callable[[str], Json] | None = None,
    decoder: _JsonDecoder = json.loads,
) -> Json:
    """Decode and narrow JSON, retaining the caller's duplicate and numeric policy."""
    return _json_shape(decoder(raw, object_pairs_hook=pairs_hook, parse_constant=constant_hook))


def typed_json(value: object) -> Json:
    """Narrow an already decoded JSON value and reject every nonfinite number."""
    def finite(item: Json, path: str) -> None:
        if isinstance(item, float):
            require(math.isfinite(item), path, "nonfinite number")
        elif isinstance(item, list):
            for index, child in enumerate(item):
                finite(child, f"{path}/{index}")
        elif isinstance(item, dict):
            for key, child in item.items():
                finite(child, f"{path}/{key}")
    result = _json_shape(value)
    finite(result, "")
    return result


def document(raw: str) -> Node:
    def pairs(rows: list[tuple[str, Json]]) -> dict[str, Json]:
        result: dict[str, Json] = {}
        for key, value in rows:
            require(key not in result, "/", f"duplicate key {key!r}")
            result[key] = value
        return result

    def constant(value: str) -> Json:
        raise GateContractError("INVALID_JSON", "/", f"nonfinite {value}")

    try:
        value = decode_json(raw, pairs_hook=pairs, constant_hook=constant)
    except (ValueError, TypeError) as error:
        if isinstance(error, GateContractError):
            raise
        raise GateContractError("INVALID_JSON", "/", str(error)) from error
    node = Node(typed_json(value), "/")
    _ = node.mapping()
    return node


def integer(node: Node, maximum: int = (1 << 64) - 1) -> int:
    value = node.value
    if type(value) is int and 0 <= value <= maximum:
        return value
    raise GateContractError("INVALID_INTEGER", node.label, "expected nonnegative integer in range")


def text(node: Node) -> str:
    value = node.text()
    require(bool(value.strip()), node.label, "empty string")
    return value


def digest(node: Node, length: int = 40) -> str:
    value = text(node)
    require(re.fullmatch(f"[0-9a-f]{{{length}}}", value) is not None, node.label, "invalid digest")
    return value


def strings(node: Node) -> list[str]:
    return [text(row) for row in node.array()]


def purpose(node: Node) -> str:
    value = text(node.field("purpose"))
    require(value in {"publication", "self_proof", "defect_proof"}, node.label, "unknown proof purpose")
    return value


@boundary
def validate_binding(raw: str) -> None:
    node = document(raw)
    mode = purpose(node)
    node.field("clean").equals(True)
    node.field("frozen_source_matches").equals(True)
    harness = digest(node.field("harness"))
    base = digest(node.field("products").field("base"))
    candidate = digest(node.field("products").field("candidate"))
    node.field("frozen_main").equals(base)
    node.field("merge_bases").equals([base])
    declarations = node.field("declarations").array()
    if mode != "publication":
        require(not declarations, "/declarations", "proofs require zero declarations")
    if mode == "self_proof":
        require(candidate == base, "/products/candidate", "self proof requires independent BASE products")
    elif mode == "publication":
        require(harness == candidate, "/harness", "publication harness must be the candidate")
    else:
        require(candidate != base, "/products/candidate", "defect proof requires a different candidate")
    inputs: list[str] = []
    outputs: list[PurePosixPath] = []
    for name, source in (("base", base), ("candidate", candidate)):
        row = node.field("receipts").field(name)
        row.field("source").equals(source)
        row.field("build_source").equals(source)
        expected = digest(node.field("expected_binary_digests").field(name), 64)
        require(digest(row.field("binary_digest"), 64) == expected, row.label, "binary differs from authenticated build")
        inputs.append(digest(row.field("input_digest"), 64))
        output = PurePosixPath(text(row.field("output_root")))
        require(output.is_absolute() and ".." not in output.parts, row.label, "unsafe output root")
        outputs.append(output)
    require(inputs[0] == inputs[1], "/receipts", "fixture inputs differ between arms")
    require(not outputs[0].is_relative_to(outputs[1]) and not outputs[1].is_relative_to(outputs[0]),
            "/receipts", "independent arms require disjoint output roots")


def loss_rows(node: Node, end_tick: int) -> list[Node]:
    rows = node.array()
    for row in rows:
        fields = row.array()
        require(len(fields) == 3, row.label, "expected [tick, knowledge, optional holder]")
        require(integer(fields[0]) <= end_tick, fields[0].label, "loss after terminal tick")
        _ = text(fields[1])
        if fields[2].value is not None:
            _ = integer(fields[2])
    return rows


def ordinary_safety(node: Node) -> None:
    allowed = {"classification", "complete", "initial_tick", "end_tick", "planned_end_tick",
               "founders", "founder_census_tick", "base_alive", "candidate_alive", "lost",
               "base_lost", "prior_loss_history_complete", "base_prior_loss_history_complete"}
    require(set(node.mapping()) == allowed, node.label, "missing or unregistered safety fields")
    node.field("classification").equals("ordinary")
    for field in ("complete", "prior_loss_history_complete", "base_prior_loss_history_complete"):
        node.field(field).equals(True)
    start = integer(node.field("initial_tick"))
    end = integer(node.field("end_tick"))
    require(integer(node.field("founder_census_tick")) == start, node.label, "founder census is not initial")
    require(end >= start and end == integer(node.field("planned_end_tick")), node.label, "incomplete planned horizon")
    founders = integer(node.field("founders"))
    base_alive = integer(node.field("base_alive"))
    candidate_alive = integer(node.field("candidate_alive"))
    floor = max(0, (founders if base_alive >= founders else base_alive) - 2)
    require(candidate_alive >= floor, node.label, f"survival {candidate_alive} below BASE-relative floor {floor}")
    base_losses = loss_rows(node.field("base_lost"), end)
    losses = loss_rows(node.field("lost"), end)
    require(len(losses) <= len(base_losses), node.field("lost").label,
            f"candidate KnowledgeLost {len(losses)} exceeds BASE {len(base_losses)}")


@boundary
def validate_ordinary_safety(raw: str) -> None:
    ordinary_safety(document(raw))


@boundary
def validate_declared_fixture(raw: str) -> None:
    node = document(raw)
    mode = purpose(node)
    base = digest(node.field("base"))
    _ = digest(node.field("candidate"))
    ancestors = {digest(row) for row in node.field("candidate_ancestors_after_base").array()}
    known = set(strings(node.field("known_fixtures")))
    fixture = text(node.field("fixture_id"))
    require(fixture in known, "/fixture_id", "unregistered fixture")
    declared: set[str] = set()
    entries = node.field("declarations").array()
    if entries:
        node.field("file_committed").equals(True)
        require(digest(node.field("file_commit")) in ancestors, "/file_commit", "declaration file not committed after BASE")
        require(mode == "publication", "/purpose", "proofs require zero declarations")
    for row in entries:
        require(set(row.mapping()) == {"fixture_id", "base", "behavior_commit", "reason"},
                row.label, "missing or unknown declaration field")
        name = text(row.field("fixture_id"))
        require(name in known and name not in declared, row.label, "unknown or duplicate fixture")
        require(name not in {"baseline_identity", "f10_expected_control", "identity.baseline_samples",
                             "identity.baseline_twins", "determinism.baseline", "core.baseline10",
                             "knowledge.f10_injected"},
                row.label, "fixture cannot be exempted")
        row.field("base").equals(base)
        require(digest(row.field("behavior_commit")) in ancestors, row.label, "behavior commit is not a post-BASE ancestor")
        _ = text(row.field("reason"))
        declared.add(name)
    equal = node.field("semantic_equal").value
    require(type(equal) is bool, "/semantic_equal", "expected boolean")
    require(equal is True or fixture in declared, "/semantic_equal", "undeclared fixture difference")
    ordinary_safety(node.field("safety"))


def occurrences(node: Node) -> dict[tuple[str, str, int, int, str, str], Json]:
    result: dict[tuple[str, str, int, int, str, str], Json] = {}
    for row in node.array():
        arm = text(row.field("arm"))
        require(arm in {"base", "candidate"}, row.label, "unknown arm")
        key = (text(row.field("fixture_id")), arm, integer(row.field("seed")),
               integer(row.field("repeat")), text(row.field("role")), text(row.field("instance")))
        require(key not in result, row.label, "duplicate world occurrence")
        _ = digest(row.field("input_digest"), 64)
        require(integer(row.field("end_tick")) >= integer(row.field("initial_tick")), row.label, "reversed horizon")
        row.field("terminal_status").equals("COMPLETE")
        row.field("exit_code").equals(0)
        result[key] = row.value
    require(bool(result), node.label, "empty world coverage")
    return result


@boundary
def validate_coverage(raw: str) -> None:
    node = document(raw)
    node.field("constructor_audit_complete").equals(True)
    planned = occurrences(node.field("planned"))
    observed = occurrences(node.field("observed"))
    require(planned == observed, "/observed", "world occurrence coverage or input/horizon mismatch")


@boundary
def finalize_publication(raw: str) -> str:
    node = document(raw)
    require(purpose(node) == "publication", "/purpose", "a proof cannot authorize publication")
    candidate = digest(node.field("candidate"))
    node.field("harness").equals(candidate)
    node.field("frozen_source_matches").equals(True)
    required_rows = [integer(row) for row in node.field("required_steps").array()]
    require(bool(required_rows) and all(required_rows) and len(set(required_rows)) == len(required_rows),
            "/required_steps", "empty or duplicate required step registry")
    required = set(required_rows)
    seen: set[int] = set()
    for row in node.field("steps").array():
        step = integer(row.field("step"))
        require(step in required and step not in seen, row.label, "unexpected or duplicate step")
        row.field("source").equals(candidate)
        row.field("status").equals("PASS")
        seen.add(step)
    waiver = node.field("stall")
    if waiver.value is not None:
        waiver.field("step").equals(41)
        require(41 in required and 41 not in seen, waiver.label, "duplicate or unregistered stall")
        waiver.field("source").equals(candidate)
        waiver.field("status").equals("WAIVER_ELIGIBLE")
        waiver.field("authenticated_complete_capture").equals(True)
        waiver.field("ordinary_safety_complete").equals(True)
        seen.add(41)
    require(seen == required, "/steps", "missing required steps")
    return "PASS" if waiver.value is None else "PASS_WITH_WAIVER"
