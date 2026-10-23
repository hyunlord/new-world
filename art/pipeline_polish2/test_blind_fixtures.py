# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pydantic"]
# ///
# How to run: imported by portrait blind protocol tests; synthetic data only.
"""Explicit synthetic launches and candidate inputs for protocol regression tests."""
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from PIL import Image

from art.compositor.face_variants import FacePlan
from art.pipeline_polish2.blind_models import (
    PrivateKey,
    ReviewAgent,
    ReviewerAssignment,
    ReviewRoster,
)
from art.pipeline_polish2.blind_packets import make_reviewer
from art.pipeline_polish2.blind_round import freeze_round
from art.pipeline_polish2.portrait_models import (
    AssetRef,
    Candidate,
    CandidateSet,
    write_evidence,
)


def fake_agent(asset: AssetRef, task: str = "/synthetic/A") -> ReviewAgent:
    return ReviewAgent(agent_task=task, isolated_context=True, spawn_receipt=asset)


@dataclass(frozen=True, slots=True)
class SyntheticRound:
    base: Path
    candidates: CandidateSet
    a: PrivateKey
    b: PrivateKey


def fixture_round(directory: Path) -> SyntheticRound:
    portrait, sprite = directory / "portrait.png", directory / "sprite.png"
    Image.new("RGB", (128, 128), "#aa8866").save(portrait)
    Image.new("RGBA", (80, 112), "#664433").save(sprite)
    receipt = directory / "synthetic_spawn_receipt.txt"
    _ = receipt.write_text("SYNTHETIC FIXTURE ONLY: two isolated test agents; no actual reviewer evidence.\n")
    plan = FacePlan.read()
    candidates = CandidateSet(candidates=tuple(Candidate(look=look.look, variant=variant.id,
                              portrait=AssetRef.of(portrait), sprite=AssetRef.of(sprite), attempt=1)
                              for look in plan.looks for variant in look.variants))
    write_evidence(directory / "candidates.json", candidates)
    base = directory / "blind/round0"
    roster = ReviewRoster(round=0, assignments=(
        ReviewerAssignment(reviewer="A", agent=fake_agent(AssetRef.of(receipt))),
        ReviewerAssignment(reviewer="B", agent=fake_agent(AssetRef.of(receipt), "/synthetic/B"))))
    manifest = freeze_round(base, directory / "candidates.json", roster)
    a = make_reviewer(PrivateKey(round=0, reviewer="A", agent=roster.assignments[0].agent,
                      round_manifest=manifest, seed=124, descriptions=(), identity=()), base)
    b = make_reviewer(PrivateKey(round=0, reviewer="B", agent=roster.assignments[1].agent,
                      round_manifest=manifest, seed=761, descriptions=(), identity=()), base)
    write_evidence(base / "private/A.json", a)
    write_evidence(base / "private/B.json", b)
    return SyntheticRound(base, candidates, a, b)
