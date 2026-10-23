# /// script
# requires-python = ">=3.11"
# dependencies = ["pytest", "pillow", "pydantic", "typer"]
# ///
# How to run: python3 -m pytest art/pipeline_polish2/test_blind_score.py -q
"""Non-vacuous scoring and packet integrity regressions for the blind gates."""
from __future__ import annotations

from pathlib import Path

import pytest
from PIL import Image

from art.compositor.face_variants import FacePlan
from art.pipeline_polish2.blind_models import (
    IdentityAnswer,
    IdentityAnswers,
    IdentityKey,
    MatchAnswer,
    MatchAnswers,
    MatchingKey,
    PrivateKey,
)
from art.pipeline_polish2.blind_packets import LABELS
from art.pipeline_polish2.blind_sample import sample
from art.pipeline_polish2.blind_score import distinctness_scores, identity_misses
from art.pipeline_polish2.portrait_models import AssetRef
from art.pipeline_polish2.test_blind_fixtures import fake_agent, fixture_round


@pytest.fixture
def asset(tmp_path: Path) -> AssetRef:
    path = tmp_path / "fixture.png"
    Image.new("RGB", (128, 128), "#a09080").save(path)
    return AssetRef.of(path)


def identity_key(asset: AssetRef) -> PrivateKey:
    return PrivateKey(round=0, reviewer="A", agent=fake_agent(asset), round_manifest=asset, seed=11, descriptions=(), identity=tuple(
        IdentityKey(id=f"I{index:02d}", target=f"look{index}/v1", correct="A",
                    options={"A": f"look{index}/v1", "B": f"look{index}/v2", "C": f"look{index}/v3", "D": "other/v1"}, image=asset)
        for index in range(1, 37)))


def test_identity_records_five_misses_instead_of_passing_average(asset: AssetRef) -> None:
    # Given: one reviewer misses five of36.
    key = identity_key(asset)
    response = IdentityAnswers(reviewer="A", agent_task=key.agent.agent_task, answers=tuple(
        IdentityAnswer(id=row.id, answer="B" if index < 5 else "A") for index, row in enumerate(key.identity)))
    # When: score that reviewer alone.
    misses = identity_misses(key, response)
    # Then: preserve exactly the five failed variant keys; 31/36 is below32.
    assert len(misses) == 5
    assert misses == tuple(row.target for row in key.identity[:5])


def test_identity_rejects_duplicate_or_missing_question(asset: AssetRef) -> None:
    # Given: a full-length response repeats I01 and omits I36.
    key = identity_key(asset)
    responses = tuple(IdentityAnswer(id=row.id, answer="A") for row in key.identity[:-1])
    response = IdentityAnswers(reviewer="A", agent_task=key.agent.agent_task, answers=(*responses, responses[0]))
    # When/Then: a complete-looking duplicate response is rejected.
    with pytest.raises(ValueError, match="exactly once"):
        _ = identity_misses(key, response)


def matching_key(asset: AssetRef) -> PrivateKey:
    return PrivateKey(round=0, reviewer="A", agent=fake_agent(asset), round_manifest=asset, seed=11, descriptions=(), identity=(), matching=tuple(
        MatchingKey(id=f"M{index:02d}", look=f"look{index}",
                    correct={"1": "A", "2": "B", "3": "C", "4": "D", "5": "E", "6": "F"},
                    targets={"1": "v1", "2": "v2", "3": "v3", "4": "v4", "5": "v5", "6": "v6"},
                    options={label: f"look{index}/{label}" for label in LABELS}, image=asset, lines=asset,
                    description_response=asset) for index in range(1, 9)))


def test_distinctness_keeps_one_failed_look_even_when_other_seven_are_perfect(asset: AssetRef) -> None:
    # Given: one swapped pair in one row, otherwise all48 matches correct.
    key = matching_key(asset)
    answers = tuple(MatchAnswer(id=row.id, matches=row.correct) for row in key.matching)
    changed = answers[0].model_copy(update={"matches": {"1": "B", "2": "A", "3": "C", "4": "D", "5": "E", "6": "F"}})
    response = MatchAnswers(reviewer="A", agent_task=key.agent.agent_task, answers=(changed, *answers[1:]))
    # When: score every row independently.
    scores = distinctness_scores(key, response)
    # Then: 46/48 overall cannot erase4/6 for the first look.
    assert sum(row.correct for row in scores) == 46
    assert scores[0].correct == 4 and not scores[0].passed
    assert all(row.passed for row in scores[1:])


def test_distinctness_accepts_exactly_five_matches_with_repeated_label(asset: AssetRef) -> None:
    # Given: a row has one wrong answer and reuses labelA twice.
    key = matching_key(asset)
    answers = tuple(MatchAnswer(id=row.id, matches=row.correct) for row in key.matching)
    changed = answers[0].model_copy(update={"matches": {"1": "A", "2": "A", "3": "C", "4": "D", "5": "E", "6": "F"}})
    # When: independently score every line, without a permutation constraint.
    scores = distinctness_scores(key, MatchAnswers(reviewer="A", agent_task=key.agent.agent_task, answers=(changed, *answers[1:])))
    # Then: exactly5/6 passes the user's original bar.
    assert scores[0].correct == 5 and scores[0].passed


def test_distinctness_rejects_missing_line(asset: AssetRef) -> None:
    # Given: a response omits line6 from one row.
    key = matching_key(asset)
    answers = tuple(MatchAnswer(id=row.id, matches=row.correct) for row in key.matching)
    changed = answers[0].model_copy(update={"matches": {"1": "A", "2": "B", "3": "C", "4": "D", "5": "E"}})
    # When/Then: incomplete line coverage fails closed.
    with pytest.raises(ValueError, match="all six line IDs"):
        _ = distinctness_scores(key, MatchAnswers(reviewer="A", agent_task=key.agent.agent_task, answers=(changed, *answers[1:])))


def test_packet_preserves_fixed_sample_and_hides_source_names(tmp_path: Path) -> None:
    # Given: synthetic fixture images for every actual planned look/variant.
    plan = FacePlan.read()
    fixture = fixture_round(tmp_path)
    selection = sample(fixture.candidates, plan)
    # When: drive the real packet renderer.
    result = fixture.a
    # Then: sample identities remain fixed while displayed orders shuffle and filenames stay opaque.
    assert len(result.descriptions) == 8 and len(result.identity) == 36
    assert {row.look for row in result.descriptions} == set(selection.distinctness_looks)
    assert {row.target for row in result.identity} == set(selection.identity_keys)
    for path in (fixture.base / "packets/A").rglob("*"):
        assert "adult_" not in path.name and "child_" not in path.name
        if path.suffix == ".png":
            with Image.open(path) as image:
                assert not image.info
    assert any(list(row.options.values()) != sorted(row.options.values()) for row in result.descriptions)
