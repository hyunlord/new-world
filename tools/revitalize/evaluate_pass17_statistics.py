from __future__ import annotations

from collections.abc import Callable, Sequence
from dataclasses import dataclass
from math import isfinite
from typing import Final, TypeVar

import numpy as np
from numpy.typing import NDArray

BOOTSTRAP: Final = 2000
T = TypeVar("T")
Pair = tuple[float, float]


class UndefinedStatistic(ValueError):
    def __init__(self, reason: str) -> None:
        self.reason = reason
        super().__init__(reason)


@dataclass(frozen=True, slots=True)
class Estimate:
    value: float
    interval: tuple[float, float]
    valid_draws: int
    replicates: int


def rank_average(values: Sequence[float]) -> NDArray[np.float64]:
    numbers = np.asarray(values, dtype=np.float64)
    order = np.argsort(numbers, kind="stable")
    ranks = np.empty(len(numbers), dtype=np.float64)
    ordered = numbers[order]
    start = 0
    while start < len(numbers):
        end = start + 1
        while end < len(numbers) and ordered[end] == ordered[start]:
            end += 1
        ranks[order[start:end]] = (start + end - 1) / 2.0 + 1.0
        start = end
    return ranks


def pearson(pairs: Sequence[Pair]) -> float:
    if len(pairs) < 3:
        raise UndefinedStatistic("correlation requires at least three pairs")
    x = np.asarray([p[0] for p in pairs], dtype=np.float64)
    y = np.asarray([p[1] for p in pairs], dtype=np.float64)
    dx, dy = x - x.mean(), y - y.mean()
    denominator = float(np.sqrt(np.dot(dx, dx) * np.dot(dy, dy)))
    if denominator <= 0.0:
        raise UndefinedStatistic("correlation has a constant axis or behavior")
    return float(np.dot(dx, dy) / denominator)


def spearman(pairs: Sequence[Pair]) -> float:
    x = rank_average([p[0] for p in pairs])
    y = rank_average([p[1] for p in pairs])
    return pearson(list(zip(x.tolist(), y.tolist(), strict=True)))


def pooled_ratio(worlds: Sequence[Pair]) -> float:
    denominator = sum(pair[1] for pair in worlds)
    if denominator <= 0.0:
        raise UndefinedStatistic("pooled exposure denominator is zero")
    return sum(pair[0] for pair in worlds) / denominator


def pooled_spearman(worlds: Sequence[Sequence[Pair]]) -> float:
    return spearman([pair for world in worlds for pair in world])


def bootstrap(
    worlds: Sequence[T], statistic: Callable[[Sequence[T]], float], replicates: int = BOOTSTRAP,
) -> Estimate:
    if not worlds or replicates < 1:
        raise UndefinedStatistic("bootstrap requires worlds and positive replicates")
    point = statistic(worlds)
    if not isfinite(point):
        raise UndefinedStatistic("point estimate is not finite")
    rng = np.random.default_rng(0)
    values: list[float] = []
    for _ in range(replicates):
        indices = rng.integers(0, len(worlds), size=len(worlds))
        try:
            value = statistic([worlds[int(index)] for index in indices])
        except UndefinedStatistic:
            continue
        if isfinite(value):
            values.append(value)
    if not values:
        raise UndefinedStatistic("all bootstrap replicates are undefined")
    lower, upper = np.percentile(values, [2.5, 97.5])
    return Estimate(point, (float(lower), float(upper)), len(values), replicates)
