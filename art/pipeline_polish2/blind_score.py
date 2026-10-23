# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic", "typer"]
# ///
# How to run: python3 -m art.pipeline_polish2.blind_score A_IDENTITY B_IDENTITY A_MATCH B_MATCH
"""Score each blind gate independently, rejecting partial or malformed evidence."""
from __future__ import annotations

from pathlib import Path
from typing import Literal

import typer

from art.pipeline_polish2.blind_matching import LINES
from art.pipeline_polish2.blind_models import (
    IdentityAnswers,
    MatchAnswers,
    PrivateKey,
    ReviewAgent,
    Reviewer,
    RoundManifest,
)
from art.pipeline_polish2.blind_round import check_response_agent, load_round
from art.pipeline_polish2.portrait_evidence import preserve_input, record_revision
from art.pipeline_polish2.portrait_models import (
    DATA,
    ROOT,
    AssetRef,
    Evidence,
    write_evidence,
)


class LookScore(Evidence):
    look: str
    correct: int
    total: Literal[6] = 6
    passed: bool
    failed_variants: tuple[str, ...]


class ReviewerScore(Evidence):
    reviewer: Reviewer
    agent: ReviewAgent
    round_manifest: AssetRef
    round: int
    identity_correct: int
    identity_total: Literal[36] = 36
    identity_passed: bool
    identity_failed_variants: tuple[str, ...]
    distinctness: tuple[LookScore, ...]
    distinctness_passed: bool
    answer_key: AssetRef
    identity_response: AssetRef
    matching_response: AssetRef


class GateScores(Evidence):
    schema_version: Literal[1] = 1
    round: int
    reviewers: tuple[ReviewerScore, ReviewerScore]
    passed: bool
    failed_variants: tuple[str, ...]
    owner_adopted: Literal[False] = False


def identity_misses(key: PrivateKey, response: IdentityAnswers) -> tuple[str, ...]:
    """Require exactly36 unique answers and retain each erroneous target key."""
    ids = [row.id for row in response.answers]
    check_response_agent(key, response)
    if response.reviewer != key.reviewer or len(key.identity) != 36:
        message = "Identity requires matching reviewer and exactly36 keyed questions"
        raise ValueError(message)
    if len(ids) != len(set(ids)) or set(ids) != {row.id for row in key.identity}:
        message = "Identity response IDs must match every question exactly once"
        raise ValueError(message)
    answers = {row.id: row.answer for row in response.answers}
    return tuple(row.target for row in key.identity if answers[row.id] != row.correct)


def distinctness_scores(key: PrivateKey, response: MatchAnswers) -> tuple[LookScore, ...]:
    """Keep the per-look 5/6 bar: good rows cannot compensate for one miss."""
    ids = [row.id for row in response.answers]
    check_response_agent(key, response)
    if response.reviewer != key.reviewer or len(key.matching) != 8 or len({row.look for row in key.matching}) != 8:
        message = "Distinctness requires matching reviewer and eight unique keyed looks"
        raise ValueError(message)
    if len(ids) != len(set(ids)) or set(ids) != {row.id for row in key.matching}:
        message = "Matching response IDs must match every question exactly once"
        raise ValueError(message)
    answers = {row.id: row.matches for row in response.answers}
    result: list[LookScore] = []
    for row in key.matching:
        answer = answers[row.id]
        if set(answer) != set(LINES):
            message = f"Matching response must answer all six line IDs: {row.id}"
            raise ValueError(message)
        failed = tuple(row.targets[line] for line in LINES if answer[line] != row.correct[line])
        result.append(LookScore(look=row.look, correct=6 - len(failed), passed=len(failed) <= 1, failed_variants=failed))
    return tuple(sorted(result, key=lambda row: row.look))


def score_reviewer(key_path: Path, identity_path: Path, matching_path: Path) -> ReviewerScore:
    """Bind scores to all packet and response bytes before publishing a summary."""
    key_ref = preserve_input(key_path, key_path.parent / "keys")
    key = PrivateKey.model_validate_json(key_ref.path.read_bytes())
    _ = load_round(key)
    identity_ref = preserve_input(identity_path, key_path.parent / "responses/identity")
    matching_ref = preserve_input(matching_path, key_path.parent / "responses/matching")
    for question in key.identity:
        question.image.verify()
    for question in key.matching:
        question.image.verify()
        question.lines.verify()
        question.description_response.verify()
    missing = identity_misses(key, IdentityAnswers.model_validate_json(identity_ref.path.read_bytes()))
    distinctness = distinctness_scores(key, MatchAnswers.model_validate_json(matching_ref.path.read_bytes()))
    return ReviewerScore(reviewer=key.reviewer, agent=key.agent, round_manifest=key.round_manifest, round=key.round,
                         identity_correct=36 - len(missing), identity_passed=len(missing) <= 4,
                         identity_failed_variants=missing, distinctness=distinctness,
                         distinctness_passed=all(row.passed for row in distinctness), answer_key=key_ref,
                         identity_response=identity_ref, matching_response=matching_ref)


def main(a_identity: Path, b_identity: Path, a_match: Path, b_match: Path, round_number: int = 0) -> None:
    base = DATA / "blind" / f"round{round_number}"
    scores = (score_reviewer(base / "private/A.json", a_identity, a_match),
              score_reviewer(base / "private/B.json", b_identity, b_match))
    if any(row.round != round_number for row in scores) or scores[0].round_manifest != scores[1].round_manifest:
        message = "Scores must bind both reviewers to the requested frozen round"
        raise ValueError(message)
    if tuple(row.reviewer for row in scores) != ("A", "B"):
        message = "Aggregate scoring requires exactly A and B in their assigned slots"
        raise ValueError(message)
    scores[0].round_manifest.verify()
    manifest = RoundManifest.model_validate_json(scores[0].round_manifest.path.read_bytes())
    assigned = {row.reviewer: row.agent for row in manifest.roster.assignments}
    sessions = [row.agent.session_id for row in scores if row.agent.session_id is not None]
    if (any(row.agent != assigned[row.reviewer] for row in scores)
            or len({row.agent.agent_task for row in scores}) != 2 or len(set(sessions)) != len(sessions)):
        message = "Scored agents must be the distinct actual agents in the frozen roster"
        raise ValueError(message)
    failed = sorted({variant for row in scores for variant in row.identity_failed_variants}
                    | {variant for row in scores for look in row.distinctness for variant in look.failed_variants})
    result = GateScores(round=round_number, reviewers=scores,
                        passed=all(row.identity_passed and row.distinctness_passed for row in scores),
                        failed_variants=tuple(failed))
    revision = record_revision(base / "scores.json", result)
    _ = record_revision(ROOT / "docs/v8/artpolish2" / f"blind_scores_round{round_number}.json", result)
    write_evidence(base / "latest_score_revision.json", revision)
    print(result.model_dump_json(indent=2))


if __name__ == "__main__":
    typer.run(main)
