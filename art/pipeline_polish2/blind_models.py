# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic"]
# ///
# How to run: imported by python3 -m art.pipeline_polish2.blind_packets.
"""Strict response and private answer-key shapes for the two blind gates."""
from __future__ import annotations

from typing import Literal, Self

from pydantic import Field, model_validator

from art.pipeline_polish2.portrait_models import AssetRef, Evidence

Reviewer = Literal["A", "B"]
ImageLabel = Literal["A", "B", "C", "D", "E", "F"]
OptionLabel = Literal["A", "B", "C", "D"]
LineId = Literal["1", "2", "3", "4", "5", "6"]


class Description(Evidence):
    id: str
    lines: dict[ImageLabel, str]


class ReviewAgent(Evidence):
    agent_task: str = Field(pattern=r"^/", min_length=2)
    session_id: str | None = Field(default=None, min_length=1)
    isolated_context: Literal[True]
    spawn_receipt: AssetRef


class ReviewerStamp(Evidence):
    reviewer: Reviewer
    agent_task: str = Field(pattern=r"^/", min_length=2)
    session_id: str | None = Field(default=None, min_length=1)


class ReviewerAssignment(Evidence):
    reviewer: Reviewer
    agent: ReviewAgent


class ReviewRoster(Evidence):
    round: int = Field(ge=0, le=1)
    assignments: tuple[ReviewerAssignment, ReviewerAssignment]

    @model_validator(mode="after")
    def unique_reviewers(self) -> Self:
        if {row.reviewer for row in self.assignments} != {"A", "B"}:
            message = "Roster must assign A and B exactly once"
            raise ValueError(message)
        agents = [row.agent.agent_task for row in self.assignments]
        sessions = [row.agent.session_id for row in self.assignments if row.agent.session_id is not None]
        if len(set(agents)) != 2 or len(set(sessions)) != len(sessions):
            message = "A and B must be distinct actual review agents/sessions"
            raise ValueError(message)
        return self


class RoundManifest(Evidence):
    round: int = Field(ge=0, le=1)
    source_index: AssetRef
    source_snapshot: AssetRef
    frozen_candidates: AssetRef
    source_plan: AssetRef
    frozen_plan: AssetRef
    roster: ReviewRoster
    previous_round: AssetRef | None


class Descriptions(ReviewerStamp):
    descriptions: tuple[Description, ...]


class DescriptionKey(Evidence):
    id: str
    look: str
    options: dict[ImageLabel, str]
    image: AssetRef


class IdentityKey(Evidence):
    id: str
    target: str
    correct: OptionLabel
    options: dict[OptionLabel, str]
    image: AssetRef


class MatchingKey(Evidence):
    id: str
    look: str
    correct: dict[LineId, ImageLabel]
    targets: dict[LineId, str]
    options: dict[ImageLabel, str]
    image: AssetRef
    lines: AssetRef
    description_response: AssetRef


class PrivateKey(Evidence):
    round: int = Field(ge=0, le=1)
    reviewer: Reviewer
    agent: ReviewAgent
    round_manifest: AssetRef
    seed: int
    descriptions: tuple[DescriptionKey, ...]
    identity: tuple[IdentityKey, ...]
    matching: tuple[MatchingKey, ...] = ()


class IdentityAnswer(Evidence):
    id: str
    answer: OptionLabel


class IdentityAnswers(ReviewerStamp):
    answers: tuple[IdentityAnswer, ...]


class MatchAnswer(Evidence):
    id: str
    matches: dict[LineId, ImageLabel]


class MatchAnswers(ReviewerStamp):
    answers: tuple[MatchAnswer, ...]
