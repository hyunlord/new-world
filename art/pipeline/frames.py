"""Registered 4x4 generated sheets; processing never constitutes acceptance.

Cells differing by one pixel are majority-reduced to the smallest cell. The
four-pose union PER DIRECTION determines one crop/scale/offset, with two output
pixels of reserved context around it. Larger clothing/tool extensions reject;
regenerate with the same declared context rather than recentering each pose.
"""
from __future__ import annotations

import hashlib
import json
import math
from collections.abc import Sequence
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Final

import numpy as np
from PIL import Image

from .ingest import ingest
from .layer_extract import extract_layer
from .models import (
    RGB,
    Box,
    Json,
    PipelineError,
    Size,
    decode_json,
    json_array,
    json_integer,
    json_object,
)
from .outline import enforce_outline
from .palette import map_palette
from .pixelize import pixelize

NAMES: Final = tuple(f'{direction}_{pose}' for direction in ('south', 'east')
                    for pose in ('idle', 'walk_a', 'walk_b', 'work')) + ('south_sleep',)
BODY_COLORS: Final = ((20, 18, 16), (154, 154, 154), (110, 110, 110))
ROOT: Final = Path(__file__).resolve().parents[2]


@dataclass(frozen=True, slots=True)
class FrameTransform:
    """A shared crop in normalized cell coordinates and exact destination offset."""
    crop: Box
    target: Size
    offset: Size
    box: Size


@dataclass(frozen=True, slots=True)
class SheetTransform:
    """Serializable immutable sheet geometry; cell bounds include unused cells."""
    source_size: Size
    cells: tuple[Box, ...]
    normalized_cell: Size
    south: FrameTransform
    east: FrameTransform
    sleep: FrameTransform

    def to_json(self) -> str:
        return json.dumps(asdict(self), indent=2) + '\n'

    @classmethod
    def from_json(cls, text: str) -> SheetTransform:
        """Parse saved geometry and reject malformed or noncanonical grids."""
        data = json_object(decode_json(text))
        try:
            def pair(value: Json) -> Size:
                values = json_array(value)
                if len(values) != 2:
                    raise PipelineError('Expected integer coordinate pair')
                return json_integer(values[0]), json_integer(values[1])

            def rect(value: Json) -> Box:
                values = json_array(value)
                if len(values) != 4:
                    raise PipelineError('Expected integer rectangle')
                return json_integer(values[0]), json_integer(values[1]), json_integer(values[2]), json_integer(values[3])

            transforms: list[FrameTransform] = []
            for key in ('south', 'east', 'sleep'):
                item = json_object(data[key])
                transforms.append(FrameTransform(rect(item['crop']), pair(item['target']),
                                                 pair(item['offset']), pair(item['box'])))
            result = cls(pair(data['source_size']), tuple(rect(cell) for cell in json_array(data['cells'])),
                         pair(data['normalized_cell']), transforms[0], transforms[1], transforms[2])
        except (KeyError, TypeError, IndexError) as error:
            raise PipelineError('Malformed sheet transform') from error
        result.validate()
        return result

    def validate(self) -> None:
        if min(self.source_size) < 4 or self.cells != grid_cells(self.source_size):
            raise PipelineError('Invalid 4x4 source grid')
        if self.normalized_cell != (self.source_size[0] // 4, self.source_size[1] // 4):
            raise PipelineError('Invalid normalized cell size')
        for transform, box in ((self.south, (32, 32)), (self.east, (32, 32)), (self.sleep, (32, 16))):
            x0, y0, x1, y1 = transform.crop
            tw, th = transform.target
            ox, oy = transform.offset
            if not (0 <= x0 < x1 <= self.normalized_cell[0] and
                    0 <= y0 < y1 <= self.normalized_cell[1] and transform.box == box and
                    0 < tw <= x1 - x0 and 0 < th <= y1 - y0 and
                    0 <= ox <= box[0] - tw and 0 <= oy <= box[1] - th):
                raise PipelineError('Invalid crop, target or offset')


@dataclass(frozen=True, slots=True)
class Frame:
    name: str
    image: Image.Image


def grid_cells(size: Size) -> tuple[Box, ...]:
    return tuple((col * size[0] // 4, row * size[1] // 4,
                  (col + 1) * size[0] // 4, (row + 1) * size[1] // 4)
                 for row in range(4) for col in range(4))


def normalize_cell(cell: Image.Image, size: Size) -> Image.Image:
    """Exact majority for +/-1 grid cells; only expanded edge blocks need votes."""
    if cell.size == size:
        return cell
    source = np.array(cell)
    xs = np.arange(size[0] + 1) * cell.width // size[0]
    ys = np.arange(size[1] + 1) * cell.height // size[1]
    output = source[ys[:-1, None], xs[None, :-1]].copy()
    expanded = {(y, int(x)) for x in np.flatnonzero(np.diff(xs) > 1) for y in range(size[1])}
    expanded.update((int(y), x) for y in np.flatnonzero(np.diff(ys) > 1) for x in range(size[0]))
    for y, x in sorted(expanded):
        block = source[ys[y]:ys[y + 1], xs[x]:xs[x + 1]]
        colors, counts = np.unique(block.reshape(-1, 4), axis=0, return_counts=True)
        output[y, x] = colors[int(counts.argmax())]
    return Image.fromarray(output)


def normalized_frames(image: Image.Image, cells: tuple[Box, ...], size: Size) -> tuple[Image.Image, ...]:
    """Validate source occupancy before reduction, including every unused cell."""
    keyed = ingest(image, (0, 0, image.width, image.height)).image
    frames: list[Image.Image] = []
    for index, bounds in enumerate(cells):
        cell = keyed.crop(bounds)
        occupied = cell.getbbox() is not None
        if occupied != (index < 9):
            raise PipelineError(f'Expected nine frames and seven empty cells; cell {index}')
        if index < 9:
            frames.append(normalize_cell(cell, size))
    return tuple(frames)


def fit_union(frames: Sequence[Image.Image], box: Size, desired_height: int) -> FrameTransform:
    """Reserve two output pixels per side, maintaining one common pose scale."""
    bboxes = [frame.getbbox() for frame in frames]
    if any(bbox is None for bbox in bboxes):
        raise PipelineError('Frame vanished during normalization')
    present = [bbox for bbox in bboxes if bbox is not None]
    left, top = min(b[0] for b in present), min(b[1] for b in present)
    right, bottom = max(b[2] for b in present), max(b[3] for b in present)
    scale = min(desired_height / (bottom - top), (box[0] - 4) / (right - left), 1.0)
    margin = max(1, math.ceil(2 / scale))
    crop = (left - margin, top - margin, right + margin, bottom + margin)
    if crop[0] < 0 or crop[1] < 0 or crop[2] > frames[0].width or crop[3] > frames[0].height:
        raise PipelineError('Insufficient source margin for shared clothing context')
    target = (min(box[0], round((crop[2] - crop[0]) * scale)),
              min(box[1], round((crop[3] - crop[1]) * scale)))
    return FrameTransform(crop, target, ((box[0] - target[0]) // 2,
                                       (box[1] - target[1]) // 2), box)


def build_transform(image: Image.Image, upright_height: int = 28) -> SheetTransform:
    """Derive registered transforms once from the original mannequin sheet."""
    if upright_height not in (24, 28) or min(image.size) < 4:
        raise PipelineError('Expected adult/elder height 28 or child height 24')
    cells = grid_cells(image.size)
    size = (image.width // 4, image.height // 4)
    frames = normalized_frames(image, cells, size)
    result = SheetTransform(image.size, cells, size, fit_union(frames[:4], (32, 32), upright_height),
                            fit_union(frames[4:8], (32, 32), upright_height),
                            fit_union(frames[8:], (32, 16), 12))
    result.validate()
    return result


def calibrate_transform(image: Image.Image, desired_idle_height: int = 24) -> SheetTransform:
    """Opt in to exact idle occupancy while retaining one transform per direction.

    Keep the nominal child crop (including its reserved context) and sleep
    geometry. Search at most 32 integer target heights, ordered by distance
    from the nominal height then by smaller height. Width follows the fixed
    crop aspect ratio, rounded to the nearest pixel. All four poses and later
    variants reuse that direction's selected target and centered offset.
    No source pixels, individual pose anchors, or default adult sizes change.
    """
    if type(desired_idle_height) is not int or not 1 <= desired_idle_height <= 32:
        raise PipelineError('Idle height must be an integer inside the 32px box')
    nominal = build_transform(image, upright_height=24)
    cells = normalized_frames(image, nominal.cells, nominal.normalized_cell)
    calibrated: list[FrameTransform] = []
    for start, geometry in ((0, nominal.south), (4, nominal.east)):
        crop_width = geometry.crop[2] - geometry.crop[0]
        crop_height = geometry.crop[3] - geometry.crop[1]
        idle = ingest(cells[start], geometry.crop).image
        candidates = sorted(range(1, min(32, crop_height) + 1),
                            key=lambda height: (abs(height - geometry.target[1]), height))
        for height in candidates:
            width = round(crop_width * height / crop_height)
            if not 1 <= width <= min(32, crop_width):
                continue
            bbox = pixelize(idle, (width, height)).getbbox()
            if bbox is not None and bbox[3] - bbox[1] == desired_idle_height:
                calibrated.append(FrameTransform(geometry.crop, (width, height),
                                                  ((32 - width) // 2, (32 - height) // 2), geometry.box))
                break
        else:
            raise PipelineError(f'Cannot calibrate {NAMES[start]} to {desired_idle_height}px inside 32px box')
    result = SheetTransform(nominal.source_size, nominal.cells, nominal.normalized_cell,
                            calibrated[0], calibrated[1], nominal.sleep)
    result.validate()
    return result


def apply_transform(image: Image.Image, transform: SheetTransform,
                    colors: Sequence[RGB] = BODY_COLORS) -> tuple[Frame, ...]:
    """Apply saved geometry exactly; three body greys or supplied master palette."""
    transform.validate()
    if image.size != transform.source_size:
        raise PipelineError('Variant source dimensions differ from registered sheet')
    cells = normalized_frames(image, transform.cells, transform.normalized_cell)
    results: list[Frame] = []
    for index, (name, cell) in enumerate(zip(NAMES, cells, strict=True)):
        geometry = (transform.south, transform.east, transform.sleep)[index // 4]
        cropped = ingest(cell, geometry.crop).image
        reduced = pixelize(cropped, geometry.target)
        mapped = enforce_outline(map_palette(reduced, colors).image)
        canvas = Image.new('RGBA', geometry.box)
        canvas.paste(mapped, geometry.offset)
        results.append(Frame(name, canvas))
    return tuple(results)


def extract_variants(base: Image.Image, variant: Image.Image, transform: SheetTransform,
                     colors: Sequence[RGB]) -> tuple[Frame, ...]:
    """Use identical palette and geometry on both inputs; reject body erasure.

    Rendering can use BODY_COLORS, but subtraction must not mistake a different
    quantizer for a changed garment. Alpha-erasure rejection remains unchanged.
    """
    bodies = apply_transform(base, transform, colors)
    worn = apply_transform(variant, transform, colors)
    return tuple(Frame(body.name, extract_layer(body.image, dressed.image).image)
                 for body, dressed in zip(bodies, worn, strict=True))


def save_frames(frames: Sequence[Frame], transform: SheetTransform, source: Path,
                destination: Path) -> None:
    """Write external unaccepted frames with hashes and a reconstructable transform."""
    if destination.resolve().is_relative_to(ROOT):
        raise PipelineError('Raw frame outputs must remain outside the repository')
    if tuple(frame.name for frame in frames) != NAMES:
        raise PipelineError('Expected complete ordered frame set')
    destination.mkdir(parents=True, exist_ok=True)
    records: list[dict[str, str]] = []
    for frame in frames:
        path = destination / f'{frame.name}.png'
        frame.image.save(path)
        records.append({'name': frame.name, 'path': str(path.resolve()),
                        'sha256': hashlib.sha256(path.read_bytes()).hexdigest()})
    payload = {'source': str(source.resolve()), 'source_sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
               'transform': asdict(transform), 'frames': records, 'accepted': False,
               'limits': ('Direction normalization: south and east each use one shared four-pose transform; '
                          'sleep has its own transform. This corrects source direction scale differences '
                          'without per-frame recentering or shape edits. Variants reuse these exact transforms. '
                          'Internal pose drift and semantic channels still require visual review.')}
    _ = (destination / 'frames.json').write_text(json.dumps(payload, indent=2) + '\n')
