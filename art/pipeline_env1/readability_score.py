# /// script
# requires-python = ">=3.12"
# dependencies = ["pydantic"]
# ///
# How to run: uv run --with pydantic python -m art.pipeline_env1.readability_score --round 0
from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from hashlib import sha256
from itertools import product
from pathlib import Path
from typing import Final

from pydantic import TypeAdapter

from art.pipeline_env1.readability_models import (
    ActivityMapping,
    Box,
    Cell,
    Condition,
    Key,
    MappedAnswer,
    Mappings,
    Observation,
    Point,
    Reference,
    Report,
    Response,
    RuleError,
    Task,
    Zoom,
)

ROOT: Final = Path(__file__).resolve().parents[2]
RAW: Final = Path("/Users/rexxa/work/artenv1-data/reviews/readability")
TASKS: Final[tuple[Task, ...]] = ("people", "storage", "carriers", "wolves", "time", "activities")
MARKS: Final[tuple[Task, ...]] = ("A", "B", "C")
CONDITIONS: Final[tuple[Condition, ...]] = ("day", "night", "winter")
ZOOMS: Final[tuple[Zoom, ...]] = ("1x", "half")
RUBRICS: Final = {
    "people": "abs(answer-key)*10 <= key visible people; no rounding",
    "storage": "Present point lies within any registered storage bbox, inclusive edges",
    "carriers": "Exact carrier count", "wolves": "Exact presence and one-to-one point/bbox matching; no extras",
    "time": "Exact day/night class", "activities": "Sum of three explicitly reviewed binary mappings",
    "A": "Exact-answer reviewed mapping", "B": "Exact-answer reviewed mapping", "C": "Exact-answer reviewed mapping",
}


@dataclass(frozen=True, slots=True)
class Snapshot:
    path: Path
    content: bytes

    @property
    def reference(self) -> Reference:
        return Reference(path=str(self.path), sha256=sha256(self.content).hexdigest())


@dataclass(frozen=True, slots=True)
class Loaded:
    answers: tuple[MappedAnswer, ...]
    snapshots: tuple[Snapshot, ...]


def require_unique(values: tuple[str, ...], context: str) -> None:
    if len(values) != len(set(values)):
        raise RuleError(f"Duplicate {context}")


def contains(point: Point, box: Box) -> bool:
    return box[0] <= point[0] <= box[2] and box[1] <= point[1] <= box[3]


def points_match(points: tuple[Point, ...], boxes: tuple[Box, ...]) -> bool:
    if len(points) != len(boxes) or len(set(points)) != len(points):
        return False
    if not points:
        return True
    return any(contains(points[0], box) and points_match(points[1:], boxes[:i] + boxes[i + 1:])
               for i, box in enumerate(boxes))


def load_reviews(round_number: int, external: Path, repo: Path) -> Loaded:
    key_file = Snapshot(external / "key.json", (external / "key.json").read_bytes())
    key = Key.model_validate_json(key_file.content)
    if key.round != round_number or len(key.images) != 12 or len(key.packets) != 6:
        raise RuleError("Expected this round, 12 distinct images and six reviewer packets")
    require_unique(tuple(i.image_id for i in key.images), "key image IDs")
    require_unique(tuple(p.packet_id for p in key.packets), "packet IDs")
    require_unique(tuple(f"{i.renderer}/{i.condition}/{i.zoom}" for i in key.images), "scene cells")
    lookup = {image.image_id: image for image in key.images}
    for truth in key.images:
        if set(truth.marks) != {"A", "B", "C"} or set(truth.comparability) != set(TASKS) or any(not m.activity.strip() for m in truth.marks.values()):
            raise RuleError(f"Incomplete marks/comparability: {truth.image_id}")
        if not truth.storage_bboxes or any(b[0] >= b[2] or b[1] >= b[3] for b in truth.storage_bboxes + truth.wolves):
            raise RuleError(f"Invalid truth footprints: {truth.image_id}")
        for image in (truth.source, truth.frozen):
            if sha256((repo / image.path).read_bytes()).hexdigest() != image.sha256:
                raise RuleError(f"Image hash changed: {image.path}")
    for condition in CONDITIONS:
        packets = [p for p in key.packets if p.condition == condition]
        if len(packets) != 2 or {p.reviewer for p in packets} != {"a", "b"}:
            raise RuleError(f"Expected independent a/b packets: {condition}")
    response_paths = {external / "responses" / f"{p.packet_id}.json" for p in key.packets}
    if set((external / "responses").glob("*.json")) != response_paths:
        raise RuleError("Missing or unexpected response files")
    mapping_file = Snapshot(external / "activity_mappings.json", (external / "activity_mappings.json").read_bytes())
    mappings = Mappings.model_validate_json(mapping_file.content).mappings
    mapping_keys = tuple(f"{m.packet_id}/{m.image_id}/{m.mark}" for m in mappings)
    require_unique(mapping_keys, "activity mapping keys")
    expected_keys = {f"{p.packet_id}/{i}/{mark}" for p in key.packets for i in p.order for mark in MARKS}
    if len(mappings) != 72 or set(mapping_keys) != expected_keys:
        raise RuleError("Exactly 72 activity mappings are required, with no missing or extra keys")
    mapped = {(m.packet_id, m.image_id, m.mark): m for m in mappings}
    snapshots = [key_file, mapping_file]
    answers: list[MappedAnswer] = []
    for packet in key.packets:
        require_unique(packet.order, f"image IDs in {packet.packet_id}")
        expected_images = {i.image_id for i in key.images if i.condition == packet.condition}
        if len(packet.order) != 4 or set(packet.order) != expected_images:
            raise RuleError(f"Incomplete packet images: {packet.packet_id}")
        packet_path = Path(packet.path)
        order_file = Snapshot(packet_path / "order.json", (packet_path / "order.json").read_bytes())
        if TypeAdapter(tuple[str, ...]).validate_json(order_file.content) != packet.order:
            raise RuleError(f"Packet order changed: {packet.packet_id}")
        if {p.stem for p in packet_path.glob("*.png")} != expected_images:
            raise RuleError(f"Missing or unexpected packet images: {packet.packet_id}")
        for image_id in packet.order:
            if sha256((packet_path / f"{image_id}.png").read_bytes()).hexdigest() != lookup[image_id].frozen.sha256:
                raise RuleError(f"Packet image changed: {packet.packet_id}/{image_id}")
        snapshots.append(order_file)
        path = external / "responses" / f"{packet.packet_id}.json"
        snapshot = Snapshot(path, path.read_bytes())
        response = Response.model_validate_json(snapshot.content)
        if response.packet_id != packet.packet_id or response.reviewer != packet.reviewer:
            raise RuleError(f"Reviewer/packet mismatch: {path}")
        if tuple(a.image_id for a in response.answers) != packet.order:
            raise RuleError(f"Missing, duplicate or reordered response images: {path}")
        if any(a.ended_at > b.started_at for a, b in zip(response.answers, response.answers[1:])):
            raise RuleError(f"Overlapping or reversed timestamps: {path}")
        snapshots.append(snapshot)
        for answer in response.answers:
            truth = lookup[answer.image_id]
            current: list[ActivityMapping] = []
            for mark in truth.marks:
                mapping = mapped[(packet.packet_id, answer.image_id, mark)]
                if mapping.answer != answer.activities[mark] or mapping.expected != truth.marks[mark].activity or not mapping.reason.strip():
                    raise RuleError(f"Unreviewed or stale activity mapping: {packet.packet_id}/{answer.image_id}/{mark}")
                current.append(mapping)
            answers.append(MappedAnswer(packet=packet, truth=truth, answer=answer, mappings=tuple(current)))
    return Loaded(tuple(answers), tuple(snapshots))


def assess(review: MappedAnswer) -> tuple[Observation, ...]:
    truth, answer = review.truth, review.answer
    point = answer.storage.point
    activities = {m.mark: m.score for m in review.mappings}
    scores: dict[Task, int] = {
        "people": int(abs(answer.people_count - truth.visible_people) * 10 <= truth.visible_people),
        "storage": int(answer.storage.present and point is not None and any(contains(point, b) for b in truth.storage_bboxes)),
        "carriers": int(answer.carriers == truth.carriers),
        "wolves": int(answer.wolves.present == bool(truth.wolves) and points_match(answer.wolves.points, truth.wolves)),
        "time": int(answer.time_class == truth.time_class), "activities": sum(activities.values()),
        "A": activities["A"], "B": activities["B"], "C": activities["C"],
    }
    mappings = {m.mark: m for m in review.mappings}
    details: dict[Task, tuple[str, str]] = {
        "people": (str(truth.visible_people), str(answer.people_count)),
        "storage": (json.dumps(truth.storage_bboxes), answer.storage.model_dump_json()),
        "carriers": (str(truth.carriers), str(answer.carriers)),
        "wolves": (json.dumps(truth.wolves), answer.wolves.model_dump_json()),
        "time": (truth.time_class, answer.time_class),
        "activities": (json.dumps({k: m.activity for k, m in truth.marks.items()}), json.dumps(answer.activities)),
        "A": (mappings["A"].expected, mappings["A"].answer),
        "B": (mappings["B"].expected, mappings["B"].answer),
        "C": (mappings["C"].expected, mappings["C"].answer),
    }
    return tuple(Observation(packet_id=review.packet.packet_id, image_id=truth.image_id, reviewer=review.packet.reviewer,
                             renderer=truth.renderer, condition=truth.condition, zoom=truth.zoom, task=task,
                             score=scores[task], maximum=3 if task == "activities" else 1,
                             elapsed_seconds=answer.elapsed, late=answer.elapsed > 60,
                             rubric=RUBRICS[task], mapping=mappings.get(task),
                             expected=details[task][0], observed=details[task][1],
                             comparability=truth.comparability["activities" if task in MARKS else task]) for task in TASKS + MARKS)


def compare(observations: tuple[Observation, ...], reviews: tuple[MappedAnswer, ...]) -> tuple[Cell, ...]:
    cells: list[Cell] = []
    for condition, zoom, task in product(CONDITIONS, ZOOMS, TASKS + MARKS):
        selected = [o for o in observations if (o.condition, o.zoom, o.task) == (condition, zoom, task)]
        before = {o.reviewer: o.score for o in selected if o.renderer == "before"}
        after = {o.reviewer: o.score for o in selected if o.renderer == "after"}
        if set(before) != {"a", "b"} or set(after) != {"a", "b"} or len(selected) != 4:
            raise RuleError(f"Incomplete comparison cell: {condition}/{zoom}/{task}")
        key_task = "activities" if task in MARKS else task
        limits = tuple(sorted({r.truth.comparability[key_task] for r in reviews
                               if (r.truth.condition, r.truth.zoom) == (condition, zoom)}))
        cells.append(Cell(condition=condition, zoom=zoom, task=task, before=before, after=after,
                          before_total=sum(before.values()), after_total=sum(after.values()),
                          maximum=2 * selected[0].maximum, raw_noninferior=sum(after.values()) >= sum(before.values()),
                          comparability=limits))
    return tuple(cells)


def score(round_number: int, external: Path = RAW, repo: Path = ROOT) -> Report:
    directory = external / f"round{round_number}"
    loaded = load_reviews(round_number, directory, repo)
    observations = tuple(o for answer in loaded.answers for o in assess(answer))
    cells = compare(observations, loaded.answers)
    evidence_paths = (repo / "docs/v8/artenv1/READABILITY_PROTOCOL.md", repo / "docs/v8/artenv1/ENV1_PLAN.md",
                      Path(__file__), Path(__file__).with_name("readability_models.py"))
    evidence = tuple(Snapshot(path, path.read_bytes()).reference for path in evidence_paths) + tuple(s.reference for s in loaded.snapshots)
    destination = repo / "docs/v8/artenv1/reviews/readability" / f"round{round_number}"
    copies = tuple(Reference(path=str(destination / "raw" / s.path.relative_to(directory)), sha256=s.reference.sha256)
                   for s in loaded.snapshots)
    half_checks = tuple(o for o in observations if o.task == "people" and o.zoom == "half" and o.renderer == "after")
    report = Report(round=round_number, evidence=evidence, raw_copies=copies,
                    observations=tuple(o for o in observations if o.task in TASKS),
                    activity_subpoints=tuple(o for o in observations if o.task in MARKS),
                    cells=tuple(c for c in cells if c.task in TASKS), activity_cells=tuple(c for c in cells if c.task in MARKS),
                    image_answers=len(loaded.answers),
                    late_images=tuple(f"{r.packet.packet_id}/{r.answer.image_id}" for r in loaded.answers if r.answer.elapsed > 60),
                    raw_noninferiority=all(c.raw_noninferior for c in cells),
                    half_counts=all(o.score == 1 for o in half_checks), half_count_checks=half_checks,
                    timing_pass=all(r.answer.elapsed <= 60 for r in loaded.answers))
    for snapshot, copy in zip(loaded.snapshots, copies, strict=True):
        path = Path(copy.path)
        path.parent.mkdir(parents=True, exist_ok=True)
        _ = path.write_bytes(snapshot.content)
    _ = destination.joinpath("scores.json").write_text(report.model_dump_json(indent=2) + "\n")
    lines = [f"# Scene readability round {round_number}", "",
             f"Raw every-cell noninferiority: **{report.raw_noninferiority}**. After ½× LOD count gate (six answers): **{report.half_counts}**.",
             f"Controlled noninferiority: **NOT_VERIFIED**. Owner approval: **PENDING**. Late images: {len(report.late_images)}/24.",
             "", "144 task observations and 72 activity subpoints remain in scores.json. Late observations are retained; elapsed times are not rounded.",
             "Two reviewer sums per renderer; activities use /6, each mark and other tasks /2. Timing is model/tool elapsed time, not human reaction time.",
             "", "| Condition | Zoom | Task | Before a+b | After a+b | Before | After | Raw ≥ | Comparability |",
             "|---|---|---|---|---|---:|---:|---|---|"]
    for cell in cells:
        lines.append(f"| {cell.condition} | {cell.zoom} | {cell.task} | {cell.before['a']}+{cell.before['b']} | "
                     + f"{cell.after['a']}+{cell.after['b']} | {cell.before_total}/{cell.maximum} | {cell.after_total}/{cell.maximum} | "
                     + f"{cell.raw_noninferior} | {'; '.join(cell.comparability).replace('|', '/')} |")
    _ = destination.joinpath("comparison.md").write_text("\n".join(lines) + "\n")
    return report


class Arguments(argparse.Namespace):
    round: int = 0


def main() -> None:
    parser = argparse.ArgumentParser(description="Score frozen scene reviews with explicit activity mappings")
    _ = parser.add_argument("--round", type=int, required=True)
    args = parser.parse_args(namespace=Arguments())
    report = score(args.round)
    print(f"{len(report.observations)} task observations, {len(report.activity_subpoints)} activity subpoints; controlled NOT_VERIFIED")


if __name__ == "__main__":
    main()
