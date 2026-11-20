"""Load RON action maps and select an approved BVH from the local CMU index.

The supported RON subset uses quoted map keys, arrays, strings, finite numbers
and booleans. It is also valid JSON, so the Blender build needs no RON library.
Optional properties are omitted. Coordinates are metres, +Z up and −Y forward.
"""

from collections.abc import Mapping
from dataclasses import dataclass
import hashlib
import json
import math
from pathlib import Path
import re
from types import MappingProxyType
from typing import TypeAlias

Value: TypeAlias = "str | int | float | bool | None | list[Value] | dict[str, Value]"
XYZ: TypeAlias = tuple[float, float, float]
BODIES = ("p1", "p2", "p3", "p4")
ID = re.compile(r"[A-Za-z0-9][A-Za-z0-9_-]*")


class SpecError(ValueError):
    """An action or a source index entry cannot be used."""


@dataclass(frozen=True, slots=True)
class Hand:
    target: str
    offset: XYZ
    slide: tuple[tuple[float, float], ...]


@dataclass(frozen=True, slots=True)
class Target:
    kind: str
    anchor: XYZ
    radius: float
    depth: float
    tip: XYZ
    orientation: XYZ
    fixture_radius: float
    solid: bool


@dataclass(frozen=True, slots=True)
class Action:
    key: str
    tags: tuple[str, ...]
    takes: tuple[str, ...]
    trim: tuple[float, float]
    loop: bool
    speed: Mapping[str, float]
    stoop: Mapping[str, float]
    prop: str | None
    hands: Mapping[str, Hand]
    target: Target
    foot_lock: bool
    contacts: tuple[float, ...]
    fixture: str | None
    reverse: bool
    heading: float
    posture: XYZ


@dataclass(frozen=True, slots=True)
class Take:
    id: str
    path: Path
    fps: float
    url: str


def _unique(pairs: list[tuple[str, Value]]) -> dict[str, Value]:
    values: dict[str, Value] = {}
    for key, value in pairs:
        if key in values:
            raise SpecError(f"duplicate key {key!r}")
        values[key] = value
    return values


def _read(path: Path) -> dict[str, Value]:
    try:
        return _map(json.loads(path.read_text(), object_pairs_hook=_unique))
    except (OSError, ValueError) as error:
        raise SpecError(f"{path}: {error}") from error


def _map(value: Value, fields: str = "", required: str = "") -> dict[str, Value]:
    if not isinstance(value, dict):
        raise SpecError("expected a map")
    extra = value.keys() - fields.split() if fields else set()
    missing = set(required.split()) - value.keys()
    if extra or missing:
        raise SpecError(f"unexpected fields {sorted(extra)}, missing fields {sorted(missing)}")
    return value


def _array(value: Value, size: int | None = None) -> list[Value]:
    if not isinstance(value, list) or (size is not None and len(value) != size):
        raise SpecError(f"expected an array{'' if size is None else f' of length {size}'}")
    return value


def _text(value: Value) -> str:
    if not isinstance(value, str) or not value.strip():
        raise SpecError("expected nonempty text")
    return value


def _id(value: Value) -> str:
    ident = _text(value)
    if not ID.fullmatch(ident):
        raise SpecError(f"invalid identifier {ident!r}")
    return ident


def _num(value: Value, low: float = -math.inf, high: float = math.inf) -> float:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise SpecError("expected a number")
    try:
        number = float(value)
    except OverflowError as error:
        raise SpecError(f"number outside [{low}, {high}]") from error
    if not math.isfinite(number) or not low <= number <= high:
        raise SpecError(f"number outside [{low}, {high}]")
    return number


def _bool(value: Value) -> bool:
    if not isinstance(value, bool):
        raise SpecError("expected true or false")
    return value


def _xyz(value: Value) -> XYZ:
    x, y, z = _array(value, 3)
    return _num(x), _num(y), _num(z)


def _ordered(phases: tuple[float, ...]) -> None:
    if any(first >= second for first, second in zip(phases, phases[1:])):
        raise SpecError("phases must be strictly increasing")


def _hand(value: Value) -> Hand:
    row = _map(value, "target offset slide", "target")
    target = _text(row["target"])
    if target not in {"prop", "anchor"}:
        raise SpecError(f"unknown hand target {target!r}")
    slide = tuple((_num(phase, 0, 1), _num(offset)) for phase, offset in
                  (_array(point, 2) for point in _array(row.get("slide", []))))
    _ordered(tuple(phase for phase, _ in slide))
    if slide and target != "prop":
        raise SpecError("a sliding grip needs a prop")
    return Hand(target, _xyz(row.get("offset", [0, 0, 0])), slide)


def _target(value: Value) -> Target:
    row = _map(value, "kind anchor radius depth tip orientation fixture_radius solid", "kind")
    kind = _text(row["kind"])
    if kind not in {"swing", "circle", "reach", "free"}:
        raise SpecError(f"unknown target kind {kind!r}")
    if kind != "free" and "anchor" not in row:
        raise SpecError(f"{kind} requires a target anchor")
    radius = _num(row.get("radius", 0), 0)
    if kind == "circle" and radius == 0:
        raise SpecError("circle radius must be positive")
    orientation = _xyz(row.get("orientation", [0, 0, 1]))
    if not any(orientation):
        raise SpecError("target orientation must be nonzero")
    return Target(kind, _xyz(row.get("anchor", [0, 0, 0])), radius,
                  _num(row.get("depth", .065), 0), _xyz(row.get("tip", [0, 0, 0])), orientation,
                  _num(row.get("fixture_radius", .155), 0), _bool(row.get("solid", False)))


def _bodies(value: Value, elder: float, default: float, low: float,
            high: float = math.inf) -> Mapping[str, float]:
    row = _map(value, " ".join(BODIES))
    return MappingProxyType({
        body: _num(row.get(body, elder if body == "p4" else default), low, high) for body in BODIES
    })


def _action(key: str, value: Value) -> Action:
    row = _map(value, "tags takes trim loop speed stoop prop hands target foot_lock contacts fixture "
               "reverse heading posture",
               "trim loop target foot_lock")
    tags = tuple(_text(tag) for tag in _array(row.get("tags", [])))
    takes = tuple(_id(take) for take in _array(row.get("takes", [])))
    if not tags and not takes:
        raise SpecError("expected a take pin or search tags")
    start, finish = (_num(second, 0) for second in _array(row["trim"], 2))
    if start >= finish:
        raise SpecError("trim must start before it ends")
    contacts = tuple(_num(phase, 0, 1) for phase in _array(row.get("contacts", [])))
    _ordered(contacts)
    hands = MappingProxyType({side: _hand(hand) for side, hand in
                              _map(row.get("hands", {}), "R L").items()})
    prop = _id(row["prop"]) if "prop" in row else None
    has_prop_grip = any(hand.target == "prop" for hand in hands.values())
    if has_prop_grip != (prop is not None):
        raise SpecError("prop and at least one prop grip must be specified together")
    target = _target(row["target"])
    loop = _bool(row["loop"])
    if target.kind == "reach" and prop is not None and (not contacts or loop):
        raise SpecError("reaching for a prop requires a one-shot and a contact phase")
    if loop and any(hand.slide and hand.slide[0][1] != hand.slide[-1][1] for hand in hands.values()):
        raise SpecError("a loop's sliding grip must join at the same offset")
    return Action(_id(key), tags, takes, (start, finish), loop,
                  _bodies(row.get("speed", {}), .85, 1., .01),
                  _bodies(row.get("stoop", {}), 7., 0., 0., 45.), prop, hands,
                  target, _bool(row["foot_lock"]), contacts,
                  _id(row["fixture"]) if "fixture" in row else None, _bool(row.get("reverse", False)),
                  _num(row.get("heading", 0), -360, 360), _xyz(row.get("posture", [0, 0, 0])))


def load(path: Path) -> tuple[Action, ...]:
    rows = _read(path)
    if not rows:
        raise SpecError(f"{path}: expected at least one action")
    actions = []
    for key, row in rows.items():
        try:
            actions.append(_action(key, row))
        except SpecError as error:
            raise SpecError(f"{path} ({key}): {error}") from error
    return tuple(actions)


def _words(value: str) -> str:
    return " ".join(re.findall(r"[^\W_]+", value.casefold()))


def _rank(query: tuple[str, ...], value: Value) -> int:
    row = _map(value, required="description")
    description = f" {_words(_text(row['description']))} "
    tags = {_words(_text(tag)) for tag in _array(row.get("tags", []))}
    return sum(needle in tags or f" {needle} " in description for needle in query)


def select(action: Action, index_path: Path) -> Take:
    """Respect ordered take pins, or rank cached takes by the matched search phrases."""
    index = _read(index_path)
    if action.takes:
        missing = set(action.takes) - index.keys()
        if missing:
            raise SpecError(f"{action.key}: take IDs missing from index: {', '.join(sorted(missing))}")
        ranked = list(action.takes)
    else:
        query = tuple(dict.fromkeys(phrase for tag in action.tags if (phrase := _words(tag))))
        scores = [(-score, key) for key, row in index.items() if (score := _rank(query, row))]
        ranked = [key for _, key in sorted(scores)]
    raw = index_path.resolve().parent / "raw"
    urls = []
    for key in ranked:
        path = raw / f"{_id(key)}.bvh"
        row = _map(index[key])
        url = _text(row["url"]) if "url" in row else ""
        if not raw.is_symlink() and not path.is_symlink() and path.is_file():
            if "sha256" in row:
                with path.open("rb") as stream:
                    digest = hashlib.file_digest(stream, "sha256").hexdigest()
                if digest != _text(row["sha256"]):
                    raise SpecError(f"{path}: SHA-256 differs from the indexed download")
            return Take(key, path, _num(row.get("source_fps", 120), 1), url)
        urls.append(f"{key} {url}".strip())
    options = ", ".join(urls[:5]) if urls else "no catalogue match"
    raise SpecError(f"{action.key}: no approved BVH in {raw}; candidates: {options}")
