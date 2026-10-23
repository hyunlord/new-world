# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pydantic"]
# ///
# How to run: uv run art/pipeline_env1/readability_packets.py --round 0
"""Freeze opaque review packets and a separate source-backed scene answer key."""
from __future__ import annotations

import argparse
import json
import random
import shutil
from hashlib import sha256
from pathlib import Path
from typing import ClassVar, Literal

from PIL import Image, ImageDraw, ImageFont
from pydantic import BaseModel, ConfigDict

ROOT = Path(__file__).resolve().parents[2]
DOCS = ROOT / "docs/v8/artenv1"
RAW = Path("/Users/rexxa/work/artenv1-data")
Point = tuple[float, float]
Box = tuple[float, float, float, float]


class Input(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(extra="ignore", frozen=True)


class ImageSource(Input):
    path: str
    sha256: str


class BaselinePerson(Input):
    agent_id: int
    screen: Point
    in_map_view: bool
    carry: int
    asleep: bool
    review_action: str


class Storage(Input):
    planned_perimeter_map_visible_bbox: Box
    comparison_limit: str


class Capture(Input):
    id: str
    image: ImageSource
    people_in_map_view: int
    people: tuple[BaselinePerson, ...]
    storage_readability_target: Storage


class Baselines(Input):
    captures: tuple[Capture, ...]


class Person(Input):
    id: int
    mark: str
    activity: str
    pixel_visible: bool
    foreground_bbox: Box | None = None


class Object(Input):
    id: str
    bbox: Box


class Render(Input):
    visible_population: int
    visible_carriers: int
    people: tuple[Person, ...]
    objects: tuple[Object, ...]
    storehouse_bbox: Box


class RenderRecord(Input):
    path: str
    sha256: str
    evidence: str
    evidence_sha256: str


class RenderManifest(Input):
    renders: tuple[RenderRecord, ...]


class SelectedRender(Input):
    source_path: str
    sha256: str
    status: Literal["FROZEN"]
    wolf_bboxes: tuple[Box, ...]


class Selected(Input):
    renders: tuple[SelectedRender, ...]


class Mark(Input):
    agent_id: int
    activity: str
    source_activity: str | None = None
    point: Point | None = None
    label_bbox: Box | None = None


class Truth(Input):
    image_id: str
    renderer: Literal["before", "after"]
    condition: str
    zoom: str
    source: ImageSource
    frozen: ImageSource
    visible_people: int
    carriers: int
    storage_bboxes: tuple[Box, ...]
    wolves: tuple[Box, ...]
    time_class: Literal["day", "night"]
    marks: dict[str, Mark]
    limitations: tuple[str, ...]
    comparability: dict[str, str]


def digest(path: Path) -> str:
    return sha256(path.read_bytes()).hexdigest()


def source(path: Path) -> ImageSource:
    return ImageSource(path=str(path.relative_to(ROOT)), sha256=digest(path))


def overlaps(first: Box, second: Box) -> bool:
    return first[0] < second[2] and second[0] < first[2] and first[1] < second[3] and second[1] < first[3]


def crosses(start: Point, end: Point, box: Box) -> bool:
    low, high = 0.0, 1.0
    for origin, delta, minimum, maximum in ((start[0], end[0] - start[0], box[0], box[2]),
                                           (start[1], end[1] - start[1], box[1], box[3])):
        if delta == 0:
            if not minimum <= origin <= maximum:
                return False
            continue
        enter, leave = sorted(((minimum - origin) / delta, (maximum - origin) / delta))
        low, high = max(low, enter), min(high, leave)
        if low > high:
            return False
    return True


def person_box(person: BaselinePerson, zoom: str) -> Box:
    scale = 1.0 if zoom == "1x" else .5
    x, y = person.screen
    width, height = (32 * scale, (12 if person.asleep else 32) * scale)
    return (x - width, y - height, x + width, y + height)


def place_label(draw: ImageDraw.ImageDraw, font: ImageFont.FreeTypeFont, letter: str,
                person: BaselinePerson, bodies: dict[int, Box], occupied: list[Box]) -> tuple[Point, Box]:
    x, y = person.screen
    for distance in (24, 36, 48, 64, 88, 112):
        for dx, dy in ((0, -1), (-1, 0), (1, 0), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)):
            anchor = (x + dx * distance - 5, y + dy * distance - 8)
            box = draw.textbbox(anchor, letter, font=font, stroke_width=1)
            centre = ((box[0] + box[2]) / 2, (box[1] + box[3]) / 2)
            if box[0] < 0 or box[1] < 114 or box[2] > 1600 or box[3] > 876:
                continue
            if any(overlaps(box, body) for body in bodies.values()) or any(overlaps(box, other) for other in occupied):
                continue
            if any(crosses(person.screen, centre, body) for identifier, body in bodies.items() if identifier != person.agent_id):
                continue
            return anchor, box
    raise ValueError(f"No non-obscuring label placement for actor {person.agent_id}")


def annotate(capture: Capture, marks: dict[str, Mark], zoom: str, target: Path) -> dict[str, Mark]:
    image = Image.open(ROOT / capture.image.path).convert("RGB")
    draw = ImageDraw.Draw(image)
    font = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 14)
    indexed = {person.agent_id: person for person in capture.people}
    bodies = {person.agent_id: person_box(person, zoom) for person in capture.people if person.in_map_view}
    occupied: list[Box] = []
    annotated: dict[str, Mark] = {}
    for letter, mark in marks.items():
        person = indexed[mark.agent_id]
        if not person.in_map_view:
            raise ValueError(f"Marked baseline actor outside map: {capture.id}/{letter}")
        anchor, box = place_label(draw, font, letter, person, bodies, occupied)
        centre = ((box[0] + box[2]) / 2, (box[1] + box[3]) / 2)
        draw.line((person.screen, centre), fill=(239, 232, 210), width=1)
        draw.text(anchor, letter, font=font, fill=(239, 232, 210),
                  stroke_width=1, stroke_fill=(35, 29, 23))
        occupied.append(box)
        activity = "sleep" if person.asleep else "carry" if person.carry > 0 else "awake"
        annotated[letter] = Mark(agent_id=person.agent_id, activity=activity,
                                 source_activity=person.review_action, point=person.screen, label_bbox=box)
    image.save(target, optimize=True, compress_level=9)
    return annotated


def build(round_number: int) -> None:
    destination = DOCS / "reviews/readability" / f"round{round_number}"
    external = RAW / "reviews/readability" / f"round{round_number}"
    if (external / "key.json").exists():
        raise ValueError("Frozen round exists; do not overwrite a reviewer packet")
    destination.joinpath("images").mkdir(parents=True, exist_ok=True)
    external.mkdir(parents=True, exist_ok=True)
    baseline = Baselines.model_validate_json((DOCS / "reference/baseline_manifest.json").read_text())
    captures = {capture.id: capture for capture in baseline.captures}
    manifest = RenderManifest.model_validate_json((DOCS / "render_manifest.json").read_text())
    records = {Path(record.path).stem: record for record in manifest.renders}
    selected = Selected.model_validate_json((RAW / "compositor/selected_readability_truth.json").read_text())
    selections = {Path(record.source_path).stem: record for record in selected.renders}
    truth: list[Truth] = []
    packet_index: list[dict[str, str | list[str]]] = []
    generator = random.Random(80651 + round_number)
    ids = iter(generator.sample(range(10000, 99999), 12))
    for condition, scene in (("day", "day6_noon"), ("night", "day6_night"), ("winter", "winter_noon")):
        group: list[Truth] = []
        for zoom in ("1x", "half"):
            suffix = "figures" if zoom == "1x" else "lod"
            stem = f"scene_{scene}_{condition}_{zoom}_{suffix}"
            record = records[stem]
            selection = selections[stem]
            if digest(ROOT / record.path) != record.sha256 or digest(Path(record.evidence)) != record.evidence_sha256:
                raise ValueError(f"Image/evidence pair changed: {stem}")
            if selection.sha256 != record.sha256:
                raise ValueError(f"Compact key image changed: {stem}")
            rendered = Render.model_validate_json(Path(record.evidence).read_text())
            marks = {person.mark: Mark(agent_id=person.id, activity=person.activity)
                     for person in rendered.people if person.mark and person.pixel_visible}
            if set(marks) != {"A", "B", "C"}:
                raise ValueError(f"Missing visible marks in {stem}")
            capture = captures[f"{scene}_{zoom}"]
            if digest(ROOT / capture.image.path) != capture.image.sha256:
                raise ValueError(f"Baseline digest changed: {capture.id}")
            for renderer in ("before", "after"):
                image_id = f"P{next(ids)}"
                target = destination / "images" / f"{image_id}.png"
                if renderer == "before":
                    current_marks = annotate(capture, marks, zoom, target)
                    origin = capture.image
                    count = capture.people_in_map_view
                    carriers = sum(person.carry > 0 and person.in_map_view for person in capture.people)
                    storage = (capture.storage_readability_target.planned_perimeter_map_visible_bbox,)
                    wolves: tuple[Box, ...] = ()
                else:
                    origin = source(DOCS / "renders" / f"{stem}.png")
                    _ = shutil.copyfile(ROOT / origin.path, target)
                    current_marks = marks
                    count, carriers = rendered.visible_population, rendered.visible_carriers
                    storage = (rendered.storehouse_bbox, capture.storage_readability_target.planned_perimeter_map_visible_bbox)
                    wolves = selection.wolf_bboxes
                    if len(wolves) != 2:
                        raise ValueError(f"Expected two declared visible wolf bounds: {stem}")
                if target.stat().st_size > 1_500_000:
                    raise ValueError(f"File cap exceeded: {target}")
                entry = Truth(image_id=image_id, renderer=renderer, condition=condition, zoom=zoom,
                              source=origin, frozen=source(target), visible_people=count, carriers=carriers,
                              storage_bboxes=storage, wolves=wolves,
                              time_class="night" if condition == "night" else "day", marks=current_marks,
                              limitations=(capture.storage_readability_target.comparison_limit,
                                           "Before wolves absent; after two declared wolves.",
                                           "Seven existing actors have declared positions, tops and actions after."),
                              comparability={"people": "MISMATCH: declared actor positions; raw count score remains valid",
                                             "storage": "NOT_COMPARABLE: unfinished native versus finished declared storage",
                                             "carriers": "NOT_COMPARABLE: native zero versus two declared carriers",
                                             "wolves": "NOT_COMPARABLE: native absence versus two declared wolves",
                                             "time": "COMPARABLE: same native clock class",
                                             "activities": "NOT_COMPARABLE: declared action overrides; displayed poses scored"})
                group.append(entry)
                truth.append(entry)
        for reviewer in ("a", "b"):
            packet_id = f"G{generator.randrange(1000, 9999)}"
            packet = external / "packets" / packet_id
            packet.mkdir(parents=True)
            ordered = generator.sample(group, len(group))
            for entry in ordered:
                _ = shutil.copyfile(ROOT / entry.frozen.path, packet / f"{entry.image_id}.png")
            order = [entry.image_id for entry in ordered]
            _ = (packet / "order.json").write_text(json.dumps(order, indent=2) + "\n")
            packet_index.append({"packet_id": packet_id, "condition": condition,
                                 "reviewer": reviewer, "path": str(packet), "order": order})
    key = {"round": round_number, "images": [entry.model_dump(mode="json") for entry in truth],
           "packets": packet_index}
    _ = (external / "key.json").write_text(json.dumps(key, indent=2) + "\n")
    print(json.dumps({"round": round_number, "images": len(truth), "packets": packet_index}, indent=2))


class Arguments(argparse.Namespace):
    round: int = 0


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    _ = parser.add_argument("--round", type=int, required=True)
    args = parser.parse_args(namespace=Arguments())
    build(args.round)


if __name__ == "__main__":
    main()
