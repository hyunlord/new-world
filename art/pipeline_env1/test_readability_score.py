# /// script
# requires-python = ">=3.12"
# dependencies = ["pydantic", "pytest"]
# ///
# How to run: uv run --with pydantic --with pytest python -m pytest art/pipeline_env1/test_readability_score.py
from __future__ import annotations

import json
from hashlib import sha256
from itertools import product
from pathlib import Path

import pytest
from pydantic import JsonValue, ValidationError

from art.pipeline_env1.readability_models import (
    ActivityMapping,
    Answer,
    Key,
    Mappings,
    Mark,
    Packet,
    Reference,
    Response,
    RuleError,
    Truth,
)
from art.pipeline_env1.readability_score import (
    CONDITIONS,
    TASKS,
    ZOOMS,
    assess,
    load_reviews,
    points_match,
    require_unique,
    score,
)


def payload() -> dict[str, JsonValue]:
    return {
        "image_id": "P12345", "started_at": "2026-10-23T10:00:00Z",
        "ended_at": "2026-10-23T10:01:00Z", "people_count": 10,
        "storage": {"present": True, "point": [5, 5]}, "carriers": 2,
        "wolves": {"present": True, "points": [[1, 1], [8, 8]]},
        "time_class": "day", "activities": {"A": "idle", "B": "carrying", "C": "asleep"},
        "confidence": "medium", "notes": "Uncertain but one primary answer.",
    }


def test_sixty_seconds_is_on_time() -> None:
    answer = Answer.model_validate_json(json.dumps(payload()))
    assert answer.elapsed == 60


@pytest.mark.parametrize("change", [
    {"started_at": "2026-10-23T10:00:00"},
    {"ended_at": "2026-10-23T11:01:00+01:00"},
    {"ended_at": "2026-10-23T09:59:59Z"},
    {"people_count": True}, {"people_count": -1},
    {"storage": {"present": False, "point": [5, 5]}},
    {"wolves": {"present": False, "points": [[1, 1]]}},
    {"activities": {"A": "idle", "B": "carry"}},
])
def test_invalid_answers_fail_closed(change: dict[str, JsonValue]) -> None:
    with pytest.raises((ValidationError, RuleError)):
        _ = Answer.model_validate_json(json.dumps(payload() | change))


def test_wolf_points_require_distinct_boxes() -> None:
    assert not points_match(((1.0, 1.0), (2.0, 2.0)), ((0.0, 0.0, 3.0, 3.0), (7.0, 7.0, 9.0, 9.0)))


def test_overlapping_boxes_use_complete_matching() -> None:
    assert points_match(((2.0, 2.0), (1.0, 1.0)), ((0.0, 0.0, 3.0, 3.0), (2.0, 2.0, 4.0, 4.0)))


def test_extra_wolf_point_fails() -> None:
    assert not points_match(((1.0, 1.0), (2.0, 2.0)), ((0.0, 0.0, 3.0, 3.0),))


def test_duplicate_identifiers_fail() -> None:
    with pytest.raises(RuleError):
        require_unique(("P1", "P1"), "image IDs")


def dataset(tmp_path: Path) -> tuple[Path, Path]:
    repo, external = tmp_path / "repo", tmp_path / "reviews"
    directory = external / "round0"
    (directory / "responses").mkdir(parents=True)
    docs = repo / "docs/v8/artenv1"
    docs.mkdir(parents=True)
    for name in ("READABILITY_PROTOCOL.md", "ENV1_PLAN.md"):
        _ = (docs / name).write_text("Synthetic fixture, not actual review evidence.\n")
    image_path = repo / "fixture.png"
    _ = image_path.write_bytes(b"synthetic image hash fixture")
    reference = Reference(path="fixture.png", sha256=sha256(image_path.read_bytes()).hexdigest())
    images: list[Truth] = []
    for condition, zoom, renderer in product(CONDITIONS, ZOOMS, ("before", "after")):
        images.append(Truth(image_id=f"P{len(images):02}", renderer=renderer, condition=condition, zoom=zoom,
                            source=reference, frozen=reference, visible_people=10, carriers=2,
                            storage_bboxes=((0.0, 0.0, 10.0, 10.0),),
                            wolves=((0.0, 0.0, 3.0, 3.0), (7.0, 7.0, 9.0, 9.0)) if renderer == "after" else (),
                            time_class="night" if condition == "night" else "day",
                            marks={"A": Mark(agent_id=1, activity="awake"), "B": Mark(agent_id=2, activity="carry"),
                                   "C": Mark(agent_id=3, activity="sleep")}, limitations=("Synthetic",),
                            comparability={task: "NOT_COMPARABLE: synthetic fixture" for task in TASKS}))
    packets: list[Packet] = []
    mappings: list[ActivityMapping] = []
    for condition, reviewer in product(CONDITIONS, ("a", "b")):
        group = [i for i in images if i.condition == condition]
        packet_id = f"G{len(packets):02}"
        packet_path = directory / "packets" / packet_id
        packet_path.mkdir(parents=True)
        packet = Packet(packet_id=packet_id, reviewer=reviewer, condition=condition,
                        path=str(packet_path), order=tuple(i.image_id for i in group))
        _ = (packet_path / "order.json").write_text(json.dumps(packet.order))
        packets.append(packet)
        answers: list[Answer] = []
        for n, truth in enumerate(group):
            _ = (packet_path / f"{truth.image_id}.png").write_bytes(image_path.read_bytes())
            value = payload() | {"image_id": truth.image_id, "started_at": f"2026-10-23T10:0{n}:00Z",
                                 "ended_at": f"2026-10-23T10:0{n}:59Z", "time_class": truth.time_class}
            if not truth.wolves:
                value["wolves"] = {"present": False, "points": []}
            answer = Answer.model_validate_json(json.dumps(value))
            answers.append(answer)
            mappings.extend(ActivityMapping(packet_id=packet_id, image_id=truth.image_id, mark=mark,
                                            answer=answer.activities[mark], expected=truth.marks[mark].activity,
                                            score=1, reason="Synthetic reviewed mapping.") for mark in truth.marks)
        response = Response(reviewer=reviewer, packet_id=packet_id, answers=tuple(answers))
        _ = (directory / "responses" / f"{packet_id}.json").write_text(response.model_dump_json())
    _ = (directory / "key.json").write_text(Key(round=0, images=tuple(images), packets=tuple(packets)).model_dump_json())
    _ = (directory / "activity_mappings.json").write_text(Mappings(mappings=tuple(mappings)).model_dump_json())
    return repo, external


def test_full_matrix_and_reproducible_copies(tmp_path: Path) -> None:
    repo, external = dataset(tmp_path)
    result = score(0, external, repo)
    assert (len(result.observations), len(result.activity_subpoints), len(result.cells), len(result.activity_cells)) == (144, 72, 36, 18)
    assert result.raw_noninferiority and result.half_counts and result.timing_pass
    assert result.controlled_noninferiority == "NOT_VERIFIED"
    assert all(sha256(Path(copy.path).read_bytes()).hexdigest() == copy.sha256 for copy in result.raw_copies)
    assert result.model_dump_json() == score(0, external, repo).model_dump_json()


@pytest.mark.parametrize("mutation", ["missing_response", "missing_mapping", "duplicate_mapping", "stale_mapping", "packet_image", "duplicate_answer"])
def test_incomplete_or_changed_inputs_refuse_output(tmp_path: Path, mutation: str) -> None:
    repo, external = dataset(tmp_path)
    directory = external / "round0"
    mapping_path = directory / "activity_mappings.json"
    mappings = Mappings.model_validate_json(mapping_path.read_bytes())
    response_path = directory / "responses/G00.json"
    response = Response.model_validate_json(response_path.read_bytes())
    changed = mappings.mappings[0].model_copy(update={"answer": "Changed after review"})
    replacements: dict[str, tuple[Path, str | None]] = {
        "missing_response": (response_path, None),
        "missing_mapping": (mapping_path, Mappings(mappings=mappings.mappings[1:]).model_dump_json()),
        "duplicate_mapping": (mapping_path, Mappings(mappings=mappings.mappings + mappings.mappings[:1]).model_dump_json()),
        "stale_mapping": (mapping_path, Mappings(mappings=(changed,) + mappings.mappings[1:]).model_dump_json()),
        "packet_image": (directory / "packets/G00/P00.png", "changed"),
        "duplicate_answer": (response_path, response.model_copy(update={"answers": response.answers + response.answers[:1]}).model_dump_json()),
    }
    path, content = replacements[mutation]
    if content is None:
        path.unlink()
    else:
        _ = path.write_text(content)
    with pytest.raises(RuleError):
        _ = score(0, external, repo)
    assert not (repo / "docs/v8/artenv1/reviews/readability/round0/scores.json").exists()


@pytest.mark.parametrize(("count", "expected"), [(9, 1), (11, 1), (8, 0), (12, 0)])
def test_people_tolerance_boundaries(tmp_path: Path, count: int, expected: int) -> None:
    repo, external = dataset(tmp_path)
    path = external / "round0/responses/G00.json"
    response = Response.model_validate_json(path.read_bytes())
    changed = response.answers[0].model_copy(update={"people_count": count})
    _ = path.write_text(response.model_copy(update={"answers": (changed,) + response.answers[1:]}).model_dump_json())
    result = score(0, external, repo)
    row = next(o for o in result.observations if o.packet_id == "G00" and o.image_id == "P00" and o.task == "people")
    assert row.score == expected
    assert result.half_counts


def test_fractional_late_answer_is_retained(tmp_path: Path) -> None:
    repo, external = dataset(tmp_path)
    path = external / "round0/responses/G00.json"
    raw = path.read_text().replace("2026-10-23T10:03:59Z", "2026-10-23T10:04:00.000001Z")
    _ = path.write_text(raw)
    result = score(0, external, repo)
    assert result.late_images == ("G00/P03",)
    assert not result.timing_pass and result.raw_noninferiority
    assert next(o.elapsed_seconds for o in result.observations if o.packet_id == "G00" and o.image_id == "P03") == 60.000001


def test_activity_sum_cannot_hide_mark_regression(tmp_path: Path) -> None:
    repo, external = dataset(tmp_path)
    path = external / "round0/activity_mappings.json"
    original = Mappings.model_validate_json(path.read_bytes())
    mappings = tuple(m.model_copy(update={"score": 0}) if (m.packet_id, m.image_id, m.mark) in
                     {("G00", "P00", "B"), ("G00", "P01", "A")} else m for m in original.mappings)
    _ = path.write_text(Mappings(mappings=mappings).model_dump_json())
    result = score(0, external, repo)
    total = next(c for c in result.cells if (c.condition, c.zoom, c.task) == ("day", "1x", "activities"))
    mark = next(c for c in result.activity_cells if (c.condition, c.zoom, c.task) == ("day", "1x", "A"))
    assert total.before_total == total.after_total
    assert not mark.raw_noninferior and not result.raw_noninferiority


def test_mapping_boolean_cannot_be_a_binary_score() -> None:
    with pytest.raises(ValidationError):
        _ = ActivityMapping.model_validate_json('{"packet_id":"G1","image_id":"P1","mark":"A","answer":"idle","expected":"awake","score":true,"reason":"reviewed"}')


def test_duplicate_wolf_clicks_do_not_locate_two_wolves() -> None:
    assert not points_match(((2.0, 2.0), (2.0, 2.0)), ((0.0, 0.0, 3.0, 3.0), (2.0, 2.0, 4.0, 4.0)))


@pytest.mark.parametrize(("count", "expected"), [(0, 1), (1, 0)])
def test_zero_population_requires_exact_zero(tmp_path: Path, count: int, expected: int) -> None:
    repo, external = dataset(tmp_path)
    original = load_reviews(0, external / "round0", repo).answers[0]
    review = original.model_copy(update={"truth": original.truth.model_copy(update={"visible_people": 0}),
                                         "answer": original.answer.model_copy(update={"people_count": count})})
    assert next(o.score for o in assess(review) if o.task == "people") == expected
