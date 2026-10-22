# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: imported by art.whole_figure_lab.prod2
"""Deterministic certified-cohort selection, independent of review answers."""
from __future__ import annotations

from .samples import Person


def validate(people: tuple[Person, ...]) -> None:
    """Fail closed when any preregistered cohort quota is absent."""
    if (len(people) != 12 or len({p.look for p in people}) != 12
            or sum(p.sex == 'F' and p.age == 'adult' for p in people) < 4
            or sum(p.age == 'child' for p in people) < 2
            or {p.top for p in people} != {'loincloth', 'hide_wrap', 'fur_cloak'}
            or {p.hair for p in people} != {'short', 'long_tied', 'bun'}
            or {p.activity for p in people} != {'idle', 'walk', 'work', 'carry', 'sleep'}):
        message = 'Certified cohort needs 12 unique looks, 4 adult females, 2 children, all tops/hairs/activities'
        raise ValueError(message)


def choose(people: tuple[Person, ...]) -> tuple[Person, ...]:
    """Prefer 5 female adults, 5 male adults and one child of each sex; fall back to minima."""
    tops = ('loincloth', 'hide_wrap', 'fur_cloak')
    hairs = ('short', 'long_tied', 'bun')
    ordered = tuple(sorted({p.look: p for p in people}.values(),
        key=lambda p: (p.age, p.sex, (hairs.index(p.hair) - tops.index(p.top)) % 3, tops.index(p.top))))
    balanced = True
    activities = ('idle', 'walk', 'work', 'carry', 'sleep')

    def feasible(prefix: tuple[int, ...], start: int) -> tuple[int, ...] | None:
        selected = tuple(ordered[i] for i in prefix)
        slots = 12 - len(prefix)
        available = ordered[start:]
        if (len(available) < slots or len({p.top for p in selected + available}) < 3
                or len({p.hair for p in selected + available}) < 3):
            return None
        female_needed = max(0, 4 - sum(p.sex == 'F' and p.age == 'adult' for p in selected))
        child_needed = max(0, 2 - sum(p.age == 'child' for p in selected))
        if (female_needed + child_needed > slots
                or sum(p.sex == 'F' and p.age == 'adult' for p in available) < female_needed
                or sum(p.age == 'child' for p in available) < child_needed):
            return None
        if balanced:
            groups = (('adult', 'F', 5), ('adult', 'M', 5), ('child', 'F', 1), ('child', 'M', 1))
            for age, sex, quota in groups:
                count = sum(p.age == age and p.sex == sex for p in selected)
                supply = sum(p.age == age and p.sex == sex for p in available)
                if count > quota or count + supply < quota:
                    return None
        if slots == 0:
            if len({p.top for p in selected}) == len({p.hair for p in selected}) == 3:
                return prefix
            return None
        for index in range(start, len(ordered)):
            result = feasible((*prefix, index), index + 1)
            if result is not None:
                return result
        return None

    indices = feasible((), 0)
    if indices is None:
        balanced = False
        indices = feasible((), 0)
    if indices is None:
        message = 'Accepted set cannot supply the certified PROD2 cohort'
        raise ValueError(message)
    chosen = tuple(Person(ordered[j].sex, ordered[j].age, ordered[j].top,
                          ordered[j].hair, activities[i % 5]) for i, j in enumerate(indices))
    validate(chosen)
    return chosen
