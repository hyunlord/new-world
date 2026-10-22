# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scikit-image"]
# ///
# How to run: imported by acceptance and provenance builders
"""Immutable acceptance identities and exact-byte provenance checks."""
from __future__ import annotations

import hashlib
from dataclasses import dataclass
from pathlib import Path

from art.pipeline2.judge import ModelScores


@dataclass(frozen=True, slots=True)
class Candidate:
    name: str
    production_family: str
    family: str
    attempt: int
    raw: Path
    raw_sha256: str
    working: Path
    working_sha256: str
    provenance: Path | None = None
    provenance_sha256: str | None = None


@dataclass(frozen=True, slots=True)
class Review:
    raw: Path
    raw_sha256: str
    references: tuple[tuple[Path, str], ...]
    scorecard: Path
    scorecard_sha256: str
    scores: ModelScores
    manifest: Path | None = None


def sha(path: Path) -> str:
    """Hash the exact bytes used for an acceptance decision."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def review_problem(candidate: Candidate, review: Review, allowed: set[str]) -> str | None:
    """Verify candidate, two distinct family references and actual scorecard bytes."""
    if review.raw.resolve() != candidate.raw.resolve() or review.raw_sha256 != candidate.raw_sha256:
        return 'RAW_IDENTITY_MISMATCH'
    if sha(candidate.raw) != candidate.raw_sha256:
        return 'RAW_HASH_MISMATCH'
    references = {digest for _, digest in review.references}
    if len(review.references) != 2 or len(references) != 2 or not references.issubset(allowed):
        return 'SAME_FAMILY_REFERENCES_MISMATCH'
    if any(not path.is_file() or sha(path) != digest for path, digest in review.references):
        return 'REFERENCE_HASH_MISMATCH'
    if not review.scorecard.is_file() or sha(review.scorecard) != review.scorecard_sha256:
        return 'SCORECARD_HASH_MISMATCH'
    return None
