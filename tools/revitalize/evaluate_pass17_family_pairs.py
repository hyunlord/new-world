from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass
from itertools import combinations
from statistics import fmean

from evaluate_pass17_family_data import FamilyPair, FamilyPairs, FamilyPerson, FamilyWorld, StagePair
from evaluate_pass17_statistics import UndefinedStatistic


def full_siblings(first: FamilyPerson, second: FamilyPerson) -> bool:
    return first.mother is not None and first.father is not None and (first.mother, first.father) == (second.mother, second.father)


def close_kin(first: FamilyPerson, second: FamilyPerson) -> bool:
    first_parents = {parent for parent in (first.mother, first.father) if parent is not None}
    second_parents = {parent for parent in (second.mother, second.father) if parent is not None}
    return second.agent_id in first_parents or first.agent_id in second_parents or bool(first_parents & second_parents)


def collect_pairs(world: FamilyWorld) -> FamilyPairs:
    people = {person.agent_id: person for person in world.people}
    siblings: dict[tuple[int, int], FamilyPair] = {}
    unrelated: dict[tuple[int, int], FamilyPair] = {}
    kin: set[tuple[int, int]] = set()
    categories: set[StagePair] = set()
    for sibling_pass in (True, False):
        for snapshot in world.snapshots:
            settlements: dict[int, list[int]] = {}
            members = {member.agent_id: member for member in snapshot.members}
            for member in snapshot.members:
                if member.settlement_id is not None:
                    settlements.setdefault(member.settlement_id, []).append(member.agent_id)
            for settlement_id, residents in settlements.items():
                for first_id, second_id in combinations(sorted(residents), 2):
                    key = first_id, second_id
                    if key in siblings or key in unrelated or (not sibling_pass and key in kin):
                        continue
                    first, second = people[first_id], people[second_id]
                    stages = tuple(sorted((members[first_id].stage, members[second_id].stage)))
                    stage_pair: StagePair = stages[0], stages[1]
                    pair = FamilyPair(first, second, snapshot.tick, settlement_id, stage_pair)
                    if sibling_pass:
                        if full_siblings(first, second):
                            siblings[key] = pair
                            categories.add(stage_pair)
                        elif close_kin(first, second):
                            kin.add(key)
                    elif stage_pair in categories:
                        unrelated[key] = pair
    return FamilyPairs(world.seed, tuple(siblings.values()), tuple(unrelated.values()), tuple(sorted(categories)), len(kin))


@dataclass(frozen=True, slots=True)
class Moments:
    count: int
    total: int
    squares: int
    products: int


def moments(pairs: Sequence[tuple[int, int]]) -> Moments:
    return Moments(2 * len(pairs), sum(a + b for a, b in pairs),
        sum(a * a + b * b for a, b in pairs), sum(2 * a * b for a, b in pairs))


def axis_correlation(worlds: Sequence[Moments]) -> float:
    count = sum(world.count for world in worlds)
    if count < 3:
        raise UndefinedStatistic("correlation requires at least three oriented pairs")
    total = sum(world.total for world in worlds)
    squares = sum(world.squares for world in worlds)
    products = sum(world.products for world in worlds)
    denominator = count * squares - total * total
    if denominator <= 0:
        raise UndefinedStatistic("family pair correlation has a constant temperament axis")
    return (count * products - total * total) / denominator


@dataclass(frozen=True, slots=True)
class PairMoments:
    siblings: tuple[Moments, ...]
    unrelated: tuple[Moments, ...]


def pair_moments(pairs: FamilyPairs) -> PairMoments:
    return PairMoments(
        tuple(moments([(p.first.temperament[axis], p.second.temperament[axis]) for p in pairs.siblings]) for axis in range(4)),
        tuple(moments([(p.first.temperament[axis], p.second.temperament[axis]) for p in pairs.unrelated]) for axis in range(4)))


def sibling_mean(worlds: Sequence[PairMoments]) -> float:
    return fmean(axis_correlation([world.siblings[axis] for world in worlds]) for axis in range(4))


def unrelated_mean(worlds: Sequence[PairMoments]) -> float:
    return fmean(axis_correlation([world.unrelated[axis] for world in worlds]) for axis in range(4))


def resemblance_difference(worlds: Sequence[PairMoments]) -> float:
    return sibling_mean(worlds) - unrelated_mean(worlds)
