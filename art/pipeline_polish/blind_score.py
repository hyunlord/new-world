"""Score complete blinded packets without averaging away a failed group.

Semantic mapping from verbatim first answers is a separate, recorded review step.
These functions only aggregate those explicit judgements against frozen bars.
"""
from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass
from typing import Final, Literal


@dataclass(frozen=True, slots=True)
class AnimalTrial:
    id: str
    scale: Literal[1, 2]
    species_correct: bool
    pose_correct: bool


@dataclass(frozen=True, slots=True)
class AnimalGroup:
    scale: int
    species_correct: int
    pose_correct: int
    passed: bool


@dataclass(frozen=True, slots=True)
class AnimalResult:
    groups: tuple[AnimalGroup, ...]
    passed: bool


@dataclass(frozen=True, slots=True)
class IdentityTrial:
    id: str
    correct: bool


@dataclass(frozen=True, slots=True)
class IdentityResult:
    correct: int
    total: int
    passed: bool


@dataclass(frozen=True, slots=True)
class ItemTrial:
    id: str
    asset: str
    size: Literal[32, 64]
    exact: bool
    garment: bool


@dataclass(frozen=True, slots=True)
class ItemGroup:
    size: int
    exact_correct: int
    count_pass: bool


@dataclass(frozen=True, slots=True)
class ItemResult:
    groups: tuple[ItemGroup, ...]
    failed_pairs: tuple[str, ...]
    pairs_pass: bool
    passed: bool


ITEM_IDS: Final = frozenset((
    "firewood", "ground_axe", "spear", "cured_hide", "hide_wrap", "fur_cloak",
    "wolf_hide", "clay", "raw_pot", "clay_pot_empty", "clay_pot_filled", "meat",
    "flint_nodule", "log_pile_small", "log_pile_medium", "log_pile_large",
    "stone_pile", "flint_pile", "flint_blade", "stone_axe", "hide", "berry_basket",
))
REQUIRED_PAIRS: Final = (
    ("log/firewood", ("log_pile_small", "log_pile_medium", "log_pile_large", "firewood")),
    ("stone/ground axe", ("stone_axe", "ground_axe")),
    ("hide/cured hide", ("hide", "cured_hide")),
    ("raw/fired pot", ("raw_pot", "clay_pot_empty", "clay_pot_filled")),
)


def _unique(ids: Sequence[str], expected: int) -> None:
    """Reject incomplete or repeated trials before computing any percentage."""
    if len(ids) != expected:
        message = f"Expected exactly {expected} trials, got {len(ids)}"
        raise ValueError(message)
    if len(set(ids)) != expected:
        message = "Packet contains duplicate question IDs"
        raise ValueError(message)


def score_animals(rows: Sequence[AnimalTrial]) -> AnimalResult:
    """Apply each scale's species and pose bars to one complete reviewer."""
    groups: list[AnimalGroup] = []
    for scale, pose_bar in ((1, 14), (2, 16)):
        trials = tuple(row for row in rows if row.scale == scale)
        _unique(tuple(row.id for row in trials), 20)
        species = sum(row.species_correct for row in trials)
        poses = sum(row.pose_correct for row in trials)
        groups.append(AnimalGroup(scale, species, poses, species >= 19 and poses >= pose_bar))
    if len(rows) != 40:
        message = "Animal packet must contain only the two declared scales"
        raise ValueError(message)
    return AnimalResult(tuple(groups), all(group.passed for group in groups))


def score_identity(rows: Sequence[IdentityTrial]) -> IdentityResult:
    """Apply the 30/34 bar without accepting a partial identity packet."""
    _unique(tuple(row.id for row in rows), 34)
    correct = sum(row.correct for row in rows)
    return IdentityResult(correct, 34, correct >= 30)


def score_items(rows: Sequence[ItemTrial]) -> ItemResult:
    """Apply exact-kind totals and the separate mandatory 32px pair gate."""
    groups: list[ItemGroup] = []
    for size, bar in ((32, 17), (64, 20)):
        trials = tuple(row for row in rows if row.size == size)
        _unique(tuple(row.id for row in trials), 22)
        if frozenset(row.asset for row in trials) != ITEM_IDS:
            message = "Item packet must contain each of the 22 registered assets"
            raise ValueError(message)
        correct = sum(row.exact for row in trials)
        groups.append(ItemGroup(size, correct, correct >= bar))
    if len(rows) != 44:
        message = "Item packet must contain only the two declared sizes"
        raise ValueError(message)
    small = {row.asset: row for row in rows if row.size == 32}
    failed = [name for name, assets in REQUIRED_PAIRS if not all(small[asset].exact for asset in assets)]
    if not all(small[asset].garment for asset in ("hide_wrap", "fur_cloak")):
        failed.append("wrap/cloak garments")
    return ItemResult(tuple(groups), tuple(failed), not failed,
                      not failed and all(group.count_pass for group in groups))
