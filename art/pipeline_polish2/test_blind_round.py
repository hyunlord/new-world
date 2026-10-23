# /// script
# requires-python = ">=3.11"
# dependencies = ["pytest", "pillow", "pydantic", "typer"]
# ///
# How to run: python3 -m pytest art/pipeline_polish2/test_blind_round.py -q
"""Protect immutable round inputs and actual independent reviewer provenance."""
from __future__ import annotations

from pathlib import Path

import pytest
from pydantic import ValidationError

from art.pipeline_polish2.blind_models import (
    IdentityAnswers,
    ReviewAgent,
    ReviewerAssignment,
    ReviewRoster,
)
from art.pipeline_polish2.blind_round import (
    check_new_reviewers,
    check_response_agent,
    load_round,
)
from art.pipeline_polish2.portrait_evidence import preserve_input, record_revision
from art.pipeline_polish2.portrait_models import AssetRef
from art.pipeline_polish2.test_blind_fixtures import fake_agent, fixture_round


@pytest.fixture
def receipt(tmp_path: Path) -> AssetRef:
    path = tmp_path / "synthetic_spawn_receipt.txt"
    _ = path.write_text("Synthetic launch evidence only.\n")
    return AssetRef.of(path)


def test_retry_rejects_reused_canonical_agent_even_without_session_uuid(receipt: AssetRef) -> None:
    # Given: native API supplied canonical tasks but no separate session UUID.
    a, b = fake_agent(receipt, "/synthetic/A"), fake_agent(receipt, "/synthetic/B")
    previous = ReviewRoster(round=0, assignments=(ReviewerAssignment(reviewer="A", agent=a), ReviewerAssignment(reviewer="B", agent=b)))
    reused = ReviewRoster(round=1, assignments=(ReviewerAssignment(reviewer="A", agent=b),
                        ReviewerAssignment(reviewer="B", agent=fake_agent(receipt, "/synthetic/C"))))
    # When/Then: relabelling the old B as new A cannot satisfy the new-reviewer rule.
    with pytest.raises(ValueError, match="NEW review agents"):
        check_new_reviewers(reused, previous)


def test_retry_rejects_reused_session_under_different_agent_task(receipt: AssetRef) -> None:
    # Given: two apparent new task names reuse an old underlying session.
    old = ReviewAgent(agent_task="/synthetic/A", session_id="synthetic-session-1", isolated_context=True, spawn_receipt=receipt)
    previous = ReviewRoster(round=0, assignments=(ReviewerAssignment(reviewer="A", agent=old),
                       ReviewerAssignment(reviewer="B", agent=fake_agent(receipt, "/synthetic/B"))))
    renamed = old.model_copy(update={"agent_task": "/synthetic/C"})
    current = ReviewRoster(round=1, assignments=(ReviewerAssignment(reviewer="A", agent=renamed),
                      ReviewerAssignment(reviewer="B", agent=fake_agent(receipt, "/synthetic/D"))))
    # When/Then: actual session overlap is rejected as well as task overlap.
    with pytest.raises(ValueError, match="NEW review agents"):
        check_new_reviewers(current, previous)


def test_two_new_canonical_agents_are_sufficient_when_uuid_is_unavailable(receipt: AssetRef) -> None:
    # Given: actual canonical tasks and hash-bound launch receipts identify all sessions.
    previous = ReviewRoster(round=0, assignments=(ReviewerAssignment(reviewer="A", agent=fake_agent(receipt, "/synthetic/A")),
                       ReviewerAssignment(reviewer="B", agent=fake_agent(receipt, "/synthetic/B"))))
    current = ReviewRoster(round=1, assignments=(ReviewerAssignment(reviewer="A", agent=fake_agent(receipt, "/synthetic/C")),
                      ReviewerAssignment(reviewer="B", agent=fake_agent(receipt, "/synthetic/D"))))
    # When/Then: no fabricated UUID is needed for distinct new recorded tasks.
    check_new_reviewers(current, previous)


def test_roster_rejects_same_agent_for_both_labels(receipt: AssetRef) -> None:
    # Given/When/Then: A/B labels cannot turn one agent into two reviewers.
    agent = fake_agent(receipt)
    with pytest.raises(ValidationError, match="distinct actual review agents"):
        _ = ReviewRoster(round=0, assignments=(ReviewerAssignment(reviewer="A", agent=agent), ReviewerAssignment(reviewer="B", agent=agent)))


def test_response_rejects_unassigned_agent_task(tmp_path: Path) -> None:
    # Given: a real packet assignment and a response bearing another task identity.
    key = fixture_round(tmp_path).a
    response = IdentityAnswers(reviewer="A", agent_task="/synthetic/unassigned", answers=())
    # When/Then: reject before scoring numeric answers.
    with pytest.raises(ValueError, match="assigned actual review agent"):
        check_response_agent(key, response)


def test_private_key_cannot_relabel_the_frozen_round(tmp_path: Path) -> None:
    # Given: round0 evidence under a key relabelled round1.
    key = fixture_round(tmp_path).a.model_copy(update={"round": 1})
    # When/Then: key and round evidence must agree.
    with pytest.raises(ValueError, match="round differs"):
        _ = load_round(key)


def test_frozen_candidate_pixels_remain_bound_to_round(tmp_path: Path) -> None:
    # Given: round copies were made and then one frozen file was corrupted.
    key = fixture_round(tmp_path).a
    _, frozen = load_round(key)
    _ = frozen.candidates[0].portrait.path.write_bytes(b"corrupted synthetic fixture")
    # When/Then: the frozen candidate manifest's pixel digest fails closed.
    with pytest.raises(ValueError, match="changed after request"):
        _ = load_round(key)


def test_score_revisions_and_source_responses_preserve_original_failure(tmp_path: Path) -> None:
    # Given: an original score and source response, followed by a corrected revision.
    source = tmp_path / "response.json"
    _ = source.write_text('{"original": "FAIL"}\n')
    frozen = preserve_input(source, tmp_path / "responses")
    first = IdentityAnswers(reviewer="A", agent_task="/synthetic/A", answers=())
    original = record_revision(tmp_path / "scores.json", first)
    updated = first.model_copy(update={"agent_task": "/synthetic/corrected"})
    _ = source.write_text('{"original": "PASS"}\n')
    # When: a different recheck result is published to the same logical destination.
    revision = record_revision(tmp_path / "scores.json", updated)
    # Then: immutable original bytes stay available, and the new result has a new filename.
    original.verify()
    assert revision.path.name == "scores.revision002.json"
    assert frozen.path.read_text() == '{"original": "FAIL"}\n'
    assert record_revision(tmp_path / "scores.json", updated) == revision
