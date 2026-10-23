# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic"]
# ///
# How to run: imported by python3 -m art.pipeline_polish2.blind_packets.
"""Image-independent, frozen sample selection for the two portrait gates."""
from __future__ import annotations

from typing import Literal

from art.compositor.face_variants import FacePlan
from art.pipeline_polish2.portrait_models import Candidate, CandidateSet, Evidence


class Sample(Evidence):
    distinctness_looks: tuple[str, ...]
    identity_keys: tuple[str, ...]
    rule_version: Literal[1] = 1


def cohort(look: str) -> str:
    return "_".join(look.split("_")[:2])


def sample(candidates: CandidateSet, plan: FacePlan) -> Sample:
    """Read the pre-generation frozen plan; never select by image quality."""
    distinct, keys = plan.owner_sample, plan.identity_sample
    if len(distinct) != 8 or len(set(distinct)) != 8 or len(keys) != 36 or len(set(keys)) != 36:
        message = "Blind samples need exactly eight looks and 36 unique identity cases"
        raise ValueError(message)
    if sorted(cohort(look) for look in distinct) != ["adult_F"] * 2 + ["adult_M"] * 2 + ["child_F"] * 2 + ["child_M"] * 2:
        message = "Distinctness requires two looks in each age/sex cohort"
        raise ValueError(message)
    expected = {f"{look}/v{index}" for look in distinct for index in range(1, 7)} | set(keys)
    if not expected <= {row.key for row in candidates.candidates}:
        message = "Blind samples require all six variants for every selected look"
        raise ValueError(message)
    return Sample(distinctness_looks=distinct, identity_keys=keys)


def identity_options(target: Candidate, candidates: CandidateSet) -> tuple[str, ...]:
    """Return truth plus two same-look tints and one same-cohort other look."""
    variants = sorted(row.key for row in candidates.candidates if row.look == target.look)
    if len(variants) != 6:
        message = f"Identity alternatives need six variants: {target.look}"
        raise ValueError(message)
    index = variants.index(target.key)
    looks = sorted({row.look for row in candidates.candidates if cohort(row.look) == cohort(target.look)})
    other = looks[(looks.index(target.look) + 1) % len(looks)]
    if other == target.look:
        message = f"Identity needs another look in cohort: {target.look}"
        raise ValueError(message)
    return target.key, variants[(index + 1) % 6], variants[(index + 3) % 6], f"{other}/{target.variant}"
