# /// script
# requires-python = ">=3.12"
# dependencies = ["pydantic"]
# ///
# How to run: imported by python -m art.pipeline_env1.readability_score
from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timedelta
from typing import Annotated, ClassVar, Literal, Self, override

from pydantic import BaseModel, ConfigDict, Field, model_validator

Count = Annotated[int, Field(ge=0)]
Token = Annotated[str, Field(pattern=r"^[A-Za-z0-9_-]+$")]
Number = Annotated[float, Field(allow_inf_nan=False)]
Point = tuple[Number, Number]
Box = tuple[Number, Number, Number, Number]
MarkName = Literal["A", "B", "C"]
Task = Literal["people", "storage", "carriers", "wolves", "time", "activities", "A", "B", "C"]
Condition = Literal["day", "night", "winter"]
Zoom = Literal["1x", "half"]
Renderer = Literal["before", "after"]


@dataclass(frozen=True, slots=True)
class RuleError(ValueError):
    detail: str

    @override
    def __str__(self) -> str:
        return self.detail


class Model(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(extra="forbid", frozen=True, strict=True)


class Reference(Model):
    path: str
    sha256: Annotated[str, Field(pattern=r"^[0-9a-f]{64}$")]


class Mark(Model):
    agent_id: int
    activity: str
    source_activity: str | None = None
    point: Point | None = None
    label_bbox: Box | None = None


class Truth(Model):
    image_id: Token
    renderer: Renderer
    condition: Condition
    zoom: Zoom
    source: Reference
    frozen: Reference
    visible_people: Count
    carriers: Count
    storage_bboxes: tuple[Box, ...]
    wolves: tuple[Box, ...]
    time_class: Literal["day", "night"]
    marks: dict[MarkName, Mark]
    limitations: tuple[str, ...]
    comparability: dict[str, str]


class Packet(Model):
    packet_id: Token
    reviewer: Literal["a", "b"]
    condition: Condition
    path: str
    order: tuple[Token, ...]


class Key(Model):
    round: Count
    images: tuple[Truth, ...]
    packets: tuple[Packet, ...]


class StorageAnswer(Model):
    present: bool
    point: Point | None

    @model_validator(mode="after")
    def presence_matches_point(self) -> Self:
        if self.present != (self.point is not None):
            raise RuleError("Storage presence and point disagree")
        return self


class WolfAnswer(Model):
    present: bool
    points: tuple[Point, ...]

    @model_validator(mode="after")
    def presence_matches_points(self) -> Self:
        if self.present != bool(self.points):
            raise RuleError("Wolf presence and points disagree")
        return self


class Answer(Model):
    image_id: Token
    started_at: datetime
    ended_at: datetime
    people_count: Count
    storage: StorageAnswer
    carriers: Count
    wolves: WolfAnswer
    time_class: Literal["day", "night", "uncertain"]
    activities: dict[MarkName, str]
    confidence: str | Number
    notes: str

    @property
    def elapsed(self) -> float:
        return (self.ended_at - self.started_at).total_seconds()

    @model_validator(mode="after")
    def complete_timed_answer(self) -> Self:
        if any(t.utcoffset() != timedelta(0) for t in (self.started_at, self.ended_at)) or self.elapsed <= 0:
            raise RuleError("Answer requires increasing UTC timestamps")
        if set(self.activities) != {"A", "B", "C"} or any(not text.strip() for text in self.activities.values()):
            raise RuleError("Answer requires all three activity observations")
        return self


class Response(Model):
    reviewer: Literal["a", "b"]
    packet_id: Token
    answers: tuple[Answer, ...]


class ActivityMapping(Model):
    packet_id: Token
    image_id: Token
    mark: MarkName
    answer: str
    expected: str
    score: Annotated[int, Field(ge=0, le=1)]
    reason: Annotated[str, Field(min_length=1)]


class Mappings(Model):
    mappings: tuple[ActivityMapping, ...]


class MappedAnswer(Model):
    packet: Packet
    truth: Truth
    answer: Answer
    mappings: tuple[ActivityMapping, ...]


class Observation(Model):
    packet_id: str
    image_id: str
    reviewer: str
    renderer: Renderer
    condition: Condition
    zoom: Zoom
    task: Task
    score: int
    maximum: int
    elapsed_seconds: float
    late: bool
    rubric: str
    expected: str
    observed: str
    comparability: str
    mapping: ActivityMapping | None = None


class Cell(Model):
    condition: Condition
    zoom: Zoom
    task: Task
    before: dict[str, int]
    after: dict[str, int]
    before_total: int
    after_total: int
    maximum: int
    raw_noninferior: bool
    comparability: tuple[str, ...]


class Report(Model):
    round: int
    evidence: tuple[Reference, ...]
    raw_copies: tuple[Reference, ...]
    observations: tuple[Observation, ...]
    activity_subpoints: tuple[Observation, ...]
    cells: tuple[Cell, ...]
    activity_cells: tuple[Cell, ...]
    image_answers: int
    late_images: tuple[str, ...]
    raw_noninferiority: bool
    half_counts: bool
    half_count_checks: tuple[Observation, ...]
    timing_pass: bool
    controlled_noninferiority: Literal["NOT_VERIFIED"] = "NOT_VERIFIED"
    owner_approval: Literal["PENDING"] = "PENDING"
