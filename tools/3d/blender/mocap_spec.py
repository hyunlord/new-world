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
from typing import Final, TypeAlias

Value: TypeAlias = "str | int | float | bool | None | list[Value] | dict[str, Value]"
XYZ: TypeAlias = tuple[float, float, float]
BODIES = ("p1", "p2", "p3", "p4")
ID = re.compile(r"[A-Za-z0-9][A-Za-z0-9_-]*")
LAYER_MASK_FIELDS: Final = (
    "root hips spine chest neck head "
    "upper_arm_R lower_arm_R hand_R upper_leg_R lower_leg_R foot_R "
    "upper_arm_L lower_arm_L hand_L upper_leg_L lower_leg_L foot_L"
)


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
    release_offset: XYZ
    release_pitch: float
    release: bool
    windup: tuple[float, XYZ] | None
    face_clearance: float
    rim_clearance: float
    rebound_clearance: float


@dataclass(frozen=True, slots=True)
class BaseLayer:
    """A moving or static capture with optional per-bone influence."""

    key: str
    tags: tuple[str, ...]
    takes: tuple[str, ...]
    trim: tuple[float, float]
    heading: float
    weight: float
    mask: Mapping[str, float]


@dataclass(frozen=True, slots=True)
class ArmLimits:
    """Maximum arm angles in degrees and clavicle lift in metres."""

    upper_roll: float
    lower_roll: float
    elevation: float
    clavicle_lift: float
    spine_extension: float = 0.
    elbow_flexion: float = 150.


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
    base_layer: BaseLayer | None
    arm_limits: Mapping[str, ArmLimits]
    retime: tuple[tuple[float, float], ...]
    duration: float | None
    impact_window: tuple[float, float]
    in_place: bool


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
    row = _map(value, "kind anchor radius depth tip orientation fixture_radius solid release_offset release_pitch "
               "release windup face_clearance rim_clearance rebound_clearance", "kind")
    kind = _text(row["kind"])
    if kind not in {"swing", "circle", "reach", "carry", "free"}:
        raise SpecError(f"unknown target kind {kind!r}")
    if kind not in {"carry", "free"} and "anchor" not in row:
        raise SpecError(f"{kind} requires a target anchor")
    radius = _num(row.get("radius", 0), 0)
    if kind == "circle" and radius == 0:
        raise SpecError("circle radius must be positive")
    orientation = _xyz(row.get("orientation", [0, 0, 1]))
    if not any(orientation):
        raise SpecError("target orientation must be nonzero")
    windup = None
    if "windup" in row:
        phase, location = _array(row["windup"], 2)
        windup = _num(phase, .01, .99), _xyz(location)
    return Target(kind, _xyz(row.get("anchor", [0, 0, 0])), radius,
                  _num(row.get("depth", .065), 0), _xyz(row.get("tip", [0, 0, 0])), orientation,
                  _num(row.get("fixture_radius", .155), 0), _bool(row.get("solid", False)),
                  _xyz(row.get("release_offset", [0, 0, 0])), _num(row.get("release_pitch", 0), -90, 90),
                  _bool(row.get("release", False)), windup, _num(row.get("face_clearance", 0), 0, 1.),
                  _num(row.get("rim_clearance", 0), 0, 1.), _num(row.get("rebound_clearance", 0), 0, 1.))


def _bodies(value: Value, elder: float, default: float, low: float,
            high: float = math.inf) -> Mapping[str, float]:
    row = _map(value, " ".join(BODIES))
    return MappingProxyType({
        body: _num(row.get(body, elder if body == "p4" else default), low, high) for body in BODIES
    })


def _source(row: Mapping[str, Value]) -> tuple[tuple[str, ...], tuple[str, ...]]:
    tags = tuple(_text(tag) for tag in _array(row.get("tags", [])))
    takes = tuple(_id(take) for take in _array(row.get("takes", [])))
    if not tags and not takes:
        raise SpecError("expected a take pin or search tags")
    return tags, takes


def _base_layer(key: str, value: Value) -> BaseLayer:
    row = _map(value, "tags takes trim heading weight mask", "trim mask")
    tags, takes = _source(row)
    start, finish = (_num(second, 0) for second in _array(row["trim"], 2))
    if start > finish:
        raise SpecError("base layer trim must start at or before it ends")
    mask = MappingProxyType({bone: _num(weight, 0, 1) for bone, weight in
                             _map(row["mask"], LAYER_MASK_FIELDS).items()})
    if not mask:
        raise SpecError("base layer mask needs at least one bone or root translation")
    return BaseLayer(f"{key} lower body", tags, takes, (start, finish),
                     _num(row.get("heading", 0), -360, 360), _num(row.get("weight", 1), 0, 1), mask)


def _arm_limits(body: str, value: Value) -> ArmLimits:
    angles = "upper_roll lower_roll elevation"
    try:
        row = _map(value, f"{angles} clavicle_lift spine_extension elbow_flexion", angles)
        return ArmLimits(_num(row["upper_roll"], 0, 180), _num(row["lower_roll"], 0, 180),
                         _num(row["elevation"], 0, 90), _num(row.get("clavicle_lift", 0), 0, .10),
                         _num(row.get("spine_extension", 0), 0, 25),
                         _num(row.get("elbow_flexion", 150), 60, 170))
    except SpecError as error:
        raise SpecError(f"arm_limits ({body}): {error}") from error


def _retime(value: Value) -> tuple[tuple[float, float], ...]:
    """Read normalized output/source anchors for monotonic playback."""
    try:
        points = tuple((_num(output, 0, 1), _num(source, 0, 1)) for output, source in
                       (_array(point, 2) for point in _array(value)))
        if not points or points[0] != (0, 0) or points[-1] != (1, 1):
            raise SpecError("expected endpoints [0, 0] and [1, 1]")
        _ordered(tuple(output for output, _ in points))
        _ordered(tuple(source for _, source in points))
    except SpecError as error:
        raise SpecError(f"retime: {error}") from error
    return points


def _positive(value: Value, field: str, high: float = math.inf) -> float:
    try:
        number = _num(value, 0, high)
        if number == 0:
            raise SpecError("expected a positive number")
    except SpecError as error:
        raise SpecError(f"{field}: {error}") from error
    return number


def _impact_window(value: Value) -> tuple[float, float]:
    """Read the approach and release lengths in output phase units."""
    try:
        approach, release = _array(value, 2)
    except SpecError as error:
        raise SpecError(f"impact_window: {error}") from error
    return _positive(approach, "impact_window approach", 0.5), _positive(release, "impact_window release", 0.5)


def _action(key: str, value: Value) -> Action:
    row = _map(value, "tags takes trim loop speed stoop prop hands target foot_lock contacts fixture "
               "reverse heading posture base_layer arm_limits retime duration impact_window in_place",
               "trim loop target foot_lock")
    tags, takes = _source(row)
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
    foot_lock = _bool(row["foot_lock"])
    in_place = _bool(row.get("in_place", False))
    impact_window = _impact_window(row["impact_window"]) if "impact_window" in row else (.14, .14)
    if target.kind == "reach" and prop is not None and (not contacts or loop):
        raise SpecError("reaching for a prop requires a one-shot and a contact phase")
    if target.release and (target.kind != "reach" or prop is None):
        raise SpecError("release requires a reach with a prop")
    if target.kind == "carry" and sum(hand.target == "prop" for hand in hands.values()) != 2:
        raise SpecError("carry requires two prop grips")
    if in_place and foot_lock:
        raise SpecError("in-place gait needs moving feet")
    if target.windup is not None and (target.kind != "swing" or not contacts or target.windup[0] >= contacts[0]):
        raise SpecError("windup requires a swing and must peak before impact")
    if target.face_clearance and (target.windup is None or prop is None):
        raise SpecError("face clearance needs a swing prop and a windup")
    if target.rim_clearance and target.kind != "circle":
        raise SpecError("rim clearance needs a circular target")
    if target.rebound_clearance and (not any(target.release_offset) or prop is None):
        raise SpecError("rebound clearance needs a swing prop and a release offset")
    if (any(target.release_offset) or target.release_pitch) and (
        target.kind != "swing" or not contacts or contacts[0] + impact_window[1] >= 1.
    ):
        raise SpecError("a release target needs a swing contact and room to release before the end")
    if loop and any(hand.slide and hand.slide[0][1] != hand.slide[-1][1] for hand in hands.values()):
        raise SpecError("a loop's sliding grip must join at the same offset")
    arm_limits = MappingProxyType({body: _arm_limits(body, limits) for body, limits in
                                  _map(row.get("arm_limits", {}), " ".join((*BODIES, "default"))).items()})
    return Action(_id(key), tags, takes, (start, finish), loop,
                  _bodies(row.get("speed", {}), .85, 1., .01),
                  _bodies(row.get("stoop", {}), 7., 0., 0., 45.), prop, hands,
                  target, foot_lock, contacts,
                  _id(row["fixture"]) if "fixture" in row else None, _bool(row.get("reverse", False)),
                  _num(row.get("heading", 0), -360, 360), _xyz(row.get("posture", [0, 0, 0])),
                  _base_layer(key, row["base_layer"]) if "base_layer" in row else None, arm_limits,
                  _retime(row["retime"]) if "retime" in row else (),
                  _positive(row["duration"], "duration") if "duration" in row else None,
                  impact_window, in_place)


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


def load_many(paths: tuple[Path, ...]) -> tuple[Action, ...]:
    """Read each authored map once and reject accidental clip overrides."""
    actions = tuple(action for path in paths for action in load(path))
    if len({action.key for action in actions}) != len(actions):
        raise SpecError("each visual key must be unique across the selected specs")
    return actions


def _words(value: str) -> str:
    return " ".join(re.findall(r"[^\W_]+", value.casefold()))


def _rank(query: tuple[str, ...], value: Value) -> int:
    row = _map(value, required="description")
    description = f" {_words(_text(row['description']))} "
    tags = {_words(_text(tag)) for tag in _array(row.get("tags", []))}
    return sum(needle in tags or f" {needle} " in description for needle in query)


def select(action: Action | BaseLayer, index_path: Path) -> Take:
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
