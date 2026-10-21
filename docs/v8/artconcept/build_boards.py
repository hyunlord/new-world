#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = ["Pillow>=10"]
# ///
# How to run: python3 docs/v8/artconcept/build_boards.py --help
# Or: uv run docs/v8/artconcept/build_boards.py --selection /external/selection.json
"""Assemble original concept art without quantization, repainting or outlining.

Selection JSON: {"01_earth_and_ash": {"S1": "/absolute/file.png", ...,
"S8": "relative/to/data/concept.png", "C1": "cutout1.png", "C2": "cutout2.png"}}.
Without --selection, read each concept's manifest.json with that same subject map,
optionally nested under "selected". Values may be paths or {"path": "..."}.
All eight S subjects are required. C1/C2 may explicitly be null when unavailable.
A missing cutout suppresses BOTH blind crops for that concept; its board row says
NOT_VERIFIED and assembly.json records the unavailable inputs, never a pass.
Use a fresh evidence directory: existing crop_*.png files cause a preflight error.
S6 quadrants: top-left river, top-right forest, bottom-left meadow, bottom-right rock.
Readability crops use copies of a 4x4, 64-pixel tile patch across the canvas;
patches repeat to cover 640x360, then the canvas is cropped without rescaling.
Optional --body-heights maps absolute cutout paths to measured crown-to-foot spans.
Without an override the whole bounding box is 56px and body scale is NOT_VERIFIED.
Put _status: {"S5": "REJECTED_CAMERA"} inside a concept selection to label diagnostic art.
Also supported: top-level _status: {"04_diorama": {"S5": "REJECTED_CAMERA"}}.
Only saturated magenta pixels are removed; no despill or recolouring is performed.
"""
from __future__ import annotations

import argparse
import hashlib
import io
import json
import math
from dataclasses import dataclass
from pathlib import Path
from typing import Final
from collections.abc import Mapping

from PIL import Image, ImageDraw, ImageFont

ROOT: Final = Path(__file__).resolve().parents[3]
DATA: Final = Path("/Users/rexxa/work/artconcept-data")
SLUGS: Final = (
    "01_earth_and_ash", "02_folk_woodcut", "03_watercolor_field", "04_diorama",
    "05_ink_and_tone", "06_cave_ochre", "07_storybook_gouache", "08_cold_north",
)
SUBJECTS: Final = tuple(f"S{i}" for i in range(1, 9)) + ("C1", "C2")
LIMIT: Final = 1_500_000


@dataclass(frozen=True, slots=True)
class Concept:
    """Resolved selections and the authored pitch; paths always name originals."""

    slug: str
    pitch: str
    paths: tuple[Path, ...]
    cutouts: tuple[Path | None, ...]
    statuses: tuple[str, ...]
    cutout_statuses: tuple[str, ...]


def selections(data: Path, selection: Path | None) -> list[Concept]:
    """Parse explicit subject choices, failing before output if any file is absent."""
    explicit = json.loads(selection.read_text()) if selection else None
    result = []
    for slug in SLUGS:
        raw = explicit[slug] if explicit is not None else json.loads(
            (data / slug / "manifest.json").read_text())
        statuses = raw.get("_status", {})
        if explicit is not None:
            statuses = explicit.get("_status", {}).get(slug, statuses)
        raw = raw.get("selected", raw)
        paths: list[Path] = []
        cutouts: list[Path | None] = []
        for subject in SUBJECTS:
            value = raw[subject]
            if value is None and subject.startswith("C"):
                cutouts.append(None)
                continue
            match value:
                case str(filename):
                    path = Path(filename)
                case {"path": str(filename)}:
                    path = Path(filename)
                case _:
                    message = f"{slug}/{subject}: expected path string or path object"
                    raise ValueError(message)
            path = path if path.is_absolute() else data / slug / path
            if not path.is_file():
                raise FileNotFoundError(path)
            if subject.startswith("C"):
                cutouts.append(path.resolve())
            else:
                paths.append(path.resolve())
        text = (ROOT / "art/concepts" / slug / "CONCEPT.md").read_text()
        pitch = text.split("## 한 줄 피치\n", 1)[1].strip().split("\n\n", 1)[0]
        result.append(Concept(slug, pitch, tuple(paths), tuple(cutouts),
                              tuple(str(statuses.get(f"S{i}", "")) for i in range(1, 9)),
                              tuple(str(statuses.get(f"C{i}", "")) for i in range(1, 3))))
    return result


def load(path: Path) -> Image.Image:
    """Read pixels without modifying or saving the source file."""
    with Image.open(path) as source:
        return source.convert("RGB")


def reduce_short(image: Image.Image, short: int) -> Image.Image:
    """Preserve aspect ratio; never upscale small originals."""
    factor = min(1.0, short / min(image.size))
    size = tuple(max(1, round(side * factor)) for side in image.size)
    return image.resize(size, Image.Resampling.LANCZOS)


def grid(cells: list[tuple[Image.Image, str]], columns: int,
         font: ImageFont.FreeTypeFont) -> Image.Image:
    """Place intact images in uniform cells with separate dark label boxes."""
    width = max(image.width for image, _ in cells)
    height = max(image.height for image, _ in cells)
    label_height = 64
    board = Image.new("RGB", (width * columns,
        (height + label_height) * ((len(cells) + columns - 1) // columns)), "#171717")
    draw = ImageDraw.Draw(board)
    for index, (image, label) in enumerate(cells):
        x, y = index % columns * width, index // columns * (height + label_height)
        board.paste(image, (x + (width - image.width) // 2, y))
        fitted = font
        while fitted.getlength(label) > width - 20 and fitted.size > 8:
            fitted = fitted.font_variant(size=fitted.size - 1)
        draw.text((x + 10, y + height + 8), label, font=fitted, fill="white")
    return board


def bounded_save(board: Image.Image, path: Path) -> None:
    """Reduce the entire RGB board only until the PNG byte cap is satisfied."""
    while True:
        buffer = io.BytesIO()
        board.save(buffer, format="PNG", optimize=True)
        if buffer.tell() <= LIMIT:
            path.write_bytes(buffer.getvalue())
            return
        board = board.resize((max(1, int(board.width * 0.9)),
                              max(1, int(board.height * 0.9))), Image.Resampling.LANCZOS)


@dataclass(frozen=True, slots=True)
class ScaledCutout:
    """Sprite plus the evidence for its uniform downscale."""

    image: Image.Image
    source_bbox: tuple[int, int, int, int]
    source_body_span: float | None
    effective_scale: float
    rendered_body_span: float | None


def cutout(path: Path, body_height: float | None = None) -> ScaledCutout:
    """Chroma-key only magenta, crop its transparent border, then reduce to 56 px."""
    image = load(path).convert("RGBA")
    pixels = list(image.getdata())
    keyed = [(r, g, b, 0 if r >= 170 and b >= 170 and g <= 140
              and min(r, b) - g >= 70 else 255) for r, g, b, _ in pixels]
    transparent_fraction = sum(pixel[3] == 0 for pixel in keyed) / len(keyed)
    corner_indices = (0, image.width - 1, len(keyed) - image.width, len(keyed) - 1)
    magenta_corners = sum(keyed[index][3] == 0 for index in corner_indices)
    if transparent_fraction < 0.20 or magenta_corners < 3:
        message = (f"Invalid magenta cutout: {path}; keyed fraction={transparent_fraction:.3f}, "
                   f"magenta corners={magenta_corners}/4; mark C1/C2 null if unavailable")
        raise ValueError(message)
    image.putdata(keyed)
    box = image.getchannel("A").getbbox()
    if box is None:
        message = f"No foreground after magenta key: {path}"
        raise ValueError(message)
    image = image.crop(box)
    source_span = image.height if body_height is None else body_height
    if not math.isfinite(source_span) or source_span < 56 or source_span > image.height:
        message = f"Body span must be between 56 and keyed sprite height {image.height}: {path}"
        raise ValueError(message)
    scale = 56 / source_span
    height = max(1, round(image.height * scale))
    effective_scale = height / image.height
    rendered = image.resize((max(1, round(image.width * scale)), height), Image.Resampling.LANCZOS)
    return ScaledCutout(rendered, box, body_height, effective_scale,
                        None if body_height is None else body_height * effective_scale)


def readability(concept: Concept, forest: bool,
                sprites: Mapping[Path, ScaledCutout]) -> Image.Image:
    """Use the registered S6 quadrant, a repeating 4x4 patch and two people."""
    source = load(concept.paths[5])
    w, h = source.size
    box = (w // 2, 0, w, h // 2) if forest else (0, h // 2, w // 2, h)
    quadrant = reduce_short(source.crop(box), 64)
    tile = quadrant.crop((0, 0, 64, 64))
    patch = Image.new("RGB", (256, 256))
    for y in range(4):
        for x in range(4):
            patch.paste(tile, (x * 64, y * 64))
    canvas = Image.new("RGB", (768, 512))
    for y in range(2):
        for x in range(3):
            canvas.paste(patch, (x * 256, y * 256))
    canvas = canvas.crop((0, 0, 640, 360))
    for path, center in zip(concept.cutouts, (220, 420), strict=True):
        if path is None:
            message = f"Readability unavailable: {concept.slug} has a missing cutout"
            raise ValueError(message)
        person = sprites[path].image
        if person.width > 180:
            message = f"Cutout too wide to preserve separated placement: {path}"
            raise ValueError(message)
        canvas.paste(person, (center - person.width // 2, 208 - person.height), person)
    return canvas


def main() -> None:
    """Build the twelve specified boards and external blind-review evidence."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--data", type=Path, default=DATA)
    parser.add_argument("--selection", type=Path)
    parser.add_argument("--body-heights", type=Path, help="JSON: absolute cutout path -> crown-to-foot height px")
    parser.add_argument("--output", type=Path, default=ROOT / "art/concepts/boards")
    parser.add_argument("--evidence", type=Path, default=DATA / "checks")
    parser.add_argument("--font", type=Path,
                        default=Path("/System/Library/Fonts/AppleSDGothicNeo.ttc"))
    args = parser.parse_args()
    concepts = selections(args.data, args.selection)
    if any(args.evidence.glob("crop_*.png")):
        message = "Evidence directory contains prior blind crops; use a fresh --evidence path"
        raise FileExistsError(message)
    raw_heights = json.loads(args.body_heights.read_text()) if args.body_heights else {}
    body_heights = {str(Path(path).resolve()): float(span) for path, span in raw_heights.items()}
    sprites = {}
    for concept in concepts:
        if all(path is not None for path in concept.cutouts):
            for path in concept.cutouts:
                if path is not None:
                    span = body_heights.get(str(path))
                    sprites[path] = cutout(path, None if span is None else float(span))
    font = ImageFont.truetype(str(args.font), 20)
    args.output.mkdir(parents=True, exist_ok=True)
    args.evidence.mkdir(parents=True, exist_ok=True)
    boards: list[Path] = []
    provenance = []
    review_cells = []
    unavailable = []
    for index, concept in enumerate(concepts):
        cells = [(reduce_short(load(path), 480), f"S{i + 1} · {concept.pitch} {concept.statuses[i]}")
                 for i, path in enumerate(concept.paths[:8])]
        target = args.output / f"concept_{concept.slug}.png"
        bounded_save(grid(cells, 2, font), target)
        boards.append(target)
        cutout_flags = " · ".join(f"C{i+1}:{status}" for i, status in enumerate(concept.cutout_statuses) if status)
        missing = [f"C{i + 1}" for i, path in enumerate(concept.cutouts) if path is None]
        if missing:
            unavailable.append({"concept": concept.slug, "inputs": missing,
                                "status": "NOT_VERIFIED", "blind_crops_emitted": 0})
        for terrain, forest in (("forest", True), ("meadow", False)):
            if missing:
                placeholder = Image.new("RGB", (640, 360), "#171717")
                ImageDraw.Draw(placeholder).text((30, 150),
                    f"NOT_VERIFIED — unavailable {', '.join(missing)}", font=font, fill="white")
                review_cells.append((placeholder, f"{concept.slug} · {terrain} · NOT_VERIFIED {cutout_flags}"))
                continue
            crop = readability(concept, forest, sprites)
            blind_id = f"crop_{index * 2 + (0 if forest else 1) + 1:02d}"
            crop.save(args.evidence / f"{blind_id}.png")
            review_cells.append((crop, f"{concept.slug} · {terrain} {cutout_flags}"))
            provenance.append({"blind_id": blind_id, "concept": concept.slug,
                               "terrain": terrain, "status": "READY_FOR_REVIEW",
                               "cutout_statuses": {f"C{i+1}": status for i, status in enumerate(concept.cutout_statuses) if status}})
    for subject, name, columns in ((1, "people", 8), (6, "scene", 2), (3, "hut", 8)):
        cells = [(reduce_short(load(c.paths[subject]), 480), f"{c.slug} {c.statuses[subject]}") for c in concepts]
        target = args.output / f"overview_{name}.png"
        bounded_save(grid(cells, columns, font), target)
        boards.append(target)
    target = args.output / "readability.png"
    bounded_save(grid(review_cells, 2, font), target)
    boards.append(target)
    total = sum(p.stat().st_size for p in args.output.parent.rglob("*") if p.is_file())
    if total > 20_000_000:
        message = f"art/concepts budget exceeded: {total} bytes"
        raise ValueError(message)
    manifest = {"quadrants": {"forest": "top-right", "meadow": "bottom-left"},
        "target_body_height": 56, "baseline_y": 208, "tile_size": 64, "patch_tiles": [4, 4],
        "crop_size": [640, 360], "chroma_key": "r,b>=170; g<=140; min(r,b)-g>=70",
        "chroma_validation": "keyed fraction >=0.20 and >=3 of 4 magenta corners",
        "expected_crop_count": 16, "emitted_crop_count": len(provenance),
        "unavailable_crop_count": len(unavailable) * 2, "unavailable": unavailable,
        "readability_status": "PARTIAL_NOT_VERIFIED" if unavailable else "READY_FOR_REVIEW",
        "body_scaling": [{"path": str(path), "source_bbox": sprite.source_bbox,
            "source_body_span": sprite.source_body_span, "effective_scale": sprite.effective_scale,
            "rendered_body_span": sprite.rendered_body_span, "sprite_size": sprite.image.size,
            "method": "manual_body_span" if sprite.source_body_span is not None else "full_bbox_fallback_BODY_SCALE_NOT_VERIFIED"}
            for path, sprite in sprites.items()],
        "cutout_statuses": {c.slug: {f"C{i+1}": status for i, status in enumerate(c.cutout_statuses) if status} for c in concepts},
        "subject_statuses": {c.slug: {f"S{i+1}": status for i, status in enumerate(c.statuses) if status} for c in concepts},
        "blind_mapping": provenance, "boards": [str(p) for p in boards],
        "sources": [{"path": str(p), "sha256": hashlib.sha256(p.read_bytes()).hexdigest()}
                    for c in concepts for p in (*c.paths, *c.cutouts) if p is not None], "concept_directory_bytes": total}
    (args.evidence / "assembly.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(f"Built {len(boards)} boards; concept directory {total} bytes; evidence {args.evidence}")


if __name__ == "__main__":
    main()
