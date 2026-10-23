# /// script
# requires-python = ">=3.11"
# dependencies = ["pytest", "pillow", "pydantic", "typer"]
# ///
# How to run: python3 -m pytest art/pipeline_polish2/test_blind_packets.py -q
"""Drive description swap through real packet files using explicit synthetic fixtures."""
from __future__ import annotations

from pathlib import Path

import pytest
from PIL import Image
from pydantic import ValidationError

from art.pipeline_polish2 import blind_score
from art.pipeline_polish2.blind_matching import build_matching
from art.pipeline_polish2.blind_models import (
    Description,
    Descriptions,
    IdentityAnswer,
    IdentityAnswers,
    MatchAnswer,
    MatchAnswers,
    PrivateKey,
)
from art.pipeline_polish2.blind_score import distinctness_scores, score_reviewer
from art.pipeline_polish2.portrait_models import (
    AssetRef,
    CandidateSet,
    write_evidence,
)
from art.pipeline_polish2.test_blind_fixtures import fake_agent, fixture_round


def test_description_swap_preserves_answers_under_independent_shuffles(tmp_path: Path) -> None:
    # Given: synthetic fixture art and a complete two-reviewer packet pair.
    fixture = fixture_round(tmp_path)
    a, b, base = fixture.a, fixture.b, fixture.base
    descriptions = Descriptions(reviewer="B", agent_task=b.agent.agent_task, descriptions=tuple(
        Description(id=row.id, lines={label: f"SYNTHETIC fixture description {key}" for label, key in row.options.items()})
        for row in b.descriptions))
    source = tmp_path / "synthetic_B_descriptions.json"
    write_evidence(source, descriptions)
    # When: A receives B's descriptions with fresh line and image orders.
    matched = build_matching(base, a, source)
    # Then: textual identity still corresponds exactly to each keyed image.
    for row in matched.matching:
        lines = row.lines.path.read_text().splitlines()
        for line_id, label in row.correct.items():
            assert lines[int(line_id) - 1].endswith(row.options[label])
        assert row.description_response.sha256 == AssetRef.of(source).sha256
    answers = MatchAnswers(reviewer="A", agent_task=a.agent.agent_task,
                          answers=tuple(MatchAnswer(id=row.id, matches=row.correct) for row in matched.matching))
    assert all(row.correct == 6 and row.passed for row in distinctness_scores(matched, answers))
    write_evidence(base / "private/A.json", matched)
    identity_path, match_path = tmp_path / "A_identity.json", tmp_path / "A_match.json"
    write_evidence(identity_path, IdentityAnswers(reviewer="A", agent_task=a.agent.agent_task,
                   answers=tuple(IdentityAnswer(id=row.id, answer=row.correct) for row in a.identity)))
    write_evidence(match_path, answers)
    score = score_reviewer(base / "private/A.json", identity_path, match_path)
    assert score.identity_correct == 36 and score.distinctness_passed
    assert score.agent.agent_task == "/synthetic/A" and score.round == 0
    assert score.answer_key.path != base / "private/A.json"
    assert score.identity_response.path != identity_path


def test_cannot_match_own_descriptions(tmp_path: Path) -> None:
    # Given: an A-authored response supplied for A to match.
    path = tmp_path / "descriptions.json"
    write_evidence(path, Descriptions(reviewer="A", agent_task="/synthetic/A", descriptions=()))
    ref = AssetRef.of(path)
    key = PrivateKey(round=0, reviewer="A", agent=fake_agent(ref), round_manifest=ref, seed=12, descriptions=(), identity=())
    # When/Then: reject the invalid protocol before reading packet images.
    with pytest.raises(ValueError, match="own descriptions"):
        _ = build_matching(tmp_path, key, path)


def test_matching_rejects_candidate_mutation_after_description(tmp_path: Path) -> None:
    # Given: two sealed description packets and their source candidate index.
    fixture = fixture_round(tmp_path)
    a, b, base, candidates = fixture.a, fixture.b, fixture.base, fixture.candidates
    portrait_path = candidates.candidates[0].portrait.path
    response = tmp_path / "B.json"
    write_evidence(response, Descriptions(reviewer="B", agent_task=b.agent.agent_task, descriptions=tuple(
        Description(id=row.id, lines={label: f"Fixture {key}" for label, key in row.options.items()})
        for row in b.descriptions)))
    Image.new("RGB", (128, 128), "#202020").save(portrait_path)
    changed = CandidateSet(candidates=tuple(row.model_copy(update={"portrait": AssetRef.of(portrait_path)})
                                           for row in candidates.candidates))
    write_evidence(tmp_path / "candidates.json", changed)
    # When/Then: matching must reject replacement even though new index hashes are internally valid.
    with pytest.raises(ValueError, match="changed|frozen"):
        _ = build_matching(base, a, response)


def test_private_key_rejects_round_without_bound_reviewer_evidence() -> None:
    # Given/When/Then: an A/B label alone cannot prove fresh review agents.
    with pytest.raises(ValidationError):
        _ = PrivateKey.model_validate({"round": 1, "reviewer": "A", "seed": 12,
                                       "descriptions": [], "identity": []})


def test_aggregate_rejects_a_key_and_responses_copied_into_b_slot(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    # Given: valid distinct frozen reviewers, but A's completed key/answers are supplied twice.
    fixture = fixture_round(tmp_path)
    description_path = tmp_path / "B_descriptions.json"
    write_evidence(description_path, Descriptions(reviewer="B", agent_task=fixture.b.agent.agent_task,
        descriptions=tuple(Description(id=row.id, lines={label: f"Synthetic fixture {key}" for label, key in row.options.items()})
                           for row in fixture.b.descriptions)))
    matched = build_matching(fixture.base, fixture.a, description_path)
    write_evidence(fixture.base / "private/A.json", matched)
    write_evidence(fixture.base / "private/B.json", matched)
    identity_path, matching_path = tmp_path / "A_identity.json", tmp_path / "A_match.json"
    write_evidence(identity_path, IdentityAnswers(reviewer="A", agent_task=matched.agent.agent_task,
        answers=tuple(IdentityAnswer(id=row.id, answer=row.correct) for row in matched.identity)))
    write_evidence(matching_path, MatchAnswers(reviewer="A", agent_task=matched.agent.agent_task,
        answers=tuple(MatchAnswer(id=row.id, matches=row.correct) for row in matched.matching)))
    monkeypatch.setattr(blind_score, "DATA", tmp_path)
    monkeypatch.setattr(blind_score, "ROOT", tmp_path / "synthetic_repo")
    # When/Then: the actual aggregate CLI function fails before publishing a misleading PASS.
    with pytest.raises(ValueError, match="exactly A and B"):
        blind_score.main(identity_path, identity_path, matching_path, matching_path)
    assert not (fixture.base / "scores.json").exists()


def test_aggregate_accepts_both_distinct_frozen_assignments(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    # Given: both distinct frozen reviewers complete their own packets.
    fixture = fixture_round(tmp_path)
    for key in (fixture.a, fixture.b):
        write_evidence(tmp_path / f"{key.reviewer}_describe.json", Descriptions(reviewer=key.reviewer,
            agent_task=key.agent.agent_task, descriptions=tuple(Description(id=row.id,
            lines={label: f"Synthetic fixture {candidate}" for label, candidate in row.options.items()})
            for row in key.descriptions)))
    for key, other in ((fixture.a, "B"), (fixture.b, "A")):
        matched = build_matching(fixture.base, key, tmp_path / f"{other}_describe.json")
        write_evidence(fixture.base / "private" / f"{key.reviewer}.json", matched)
        write_evidence(tmp_path / f"{key.reviewer}_identity.json", IdentityAnswers(reviewer=key.reviewer,
            agent_task=key.agent.agent_task, answers=tuple(IdentityAnswer(id=row.id, answer=row.correct) for row in matched.identity)))
        write_evidence(tmp_path / f"{key.reviewer}_match.json", MatchAnswers(reviewer=key.reviewer,
            agent_task=key.agent.agent_task, answers=tuple(MatchAnswer(id=row.id, matches=row.correct) for row in matched.matching)))
    monkeypatch.setattr(blind_score, "DATA", tmp_path)
    monkeypatch.setattr(blind_score, "ROOT", tmp_path / "synthetic_repo")
    # When: run the same aggregate CLI path with the correct four actual response files.
    blind_score.main(tmp_path / "A_identity.json", tmp_path / "B_identity.json", tmp_path / "A_match.json", tmp_path / "B_match.json")
    # Then: both distinct identities remain visible in a correctly accepted synthetic result.
    result = blind_score.GateScores.model_validate_json((fixture.base / "scores.json").read_bytes())
    assert result.passed and tuple(row.reviewer for row in result.reviewers) == ("A", "B")
    assert len({row.agent.agent_task for row in result.reviewers}) == 2
