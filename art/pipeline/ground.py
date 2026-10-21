# /// script
# dependencies = ["numpy", "Pillow"]
# ///
# How to run: python3 -m art.pipeline.ground RAW OUTPUT_STEM TILE_COUNT
"""Measure fixed seam criteria without modifying edges or selecting favorable crops."""
import hashlib
import json
import sys
from collections.abc import Sequence
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Final

import numpy as np
from numpy.typing import NDArray
from PIL import Image

from .color import rgb_to_lab, vector_length
from .models import PipelineError
from .palette import load_palette, map_palette
from .pixelize import pixelize

POSITIONS: Final = ((0, 0), (64, 64), (128, 128), (224, 224))
ROOT: Final = Path(__file__).resolve().parents[2]


@dataclass(frozen=True, slots=True)
class AxisSeam:
    edge_mean: float
    interior_mean: float
    relative_limit: float
    strip_mean_delta: float
    passed: bool


@dataclass(frozen=True, slots=True)
class SeamResult:
    x: AxisSeam
    y: AxisSeam
    passed: bool


def axis_seam(lab: NDArray[np.float64]) -> AxisSeam:
    """Measure wrap in columns; transposed input measures the other axis."""
    edge_deltas = vector_length(lab[:, 0] - lab[:, -1], axis=-1)
    edge = float(np.mean(edge_deltas))
    interior_deltas = vector_length(np.diff(lab, axis=1), axis=-1)
    inside = float(np.mean(interior_deltas))
    first_strip = np.asarray(np.mean(lab[:, :4], axis=(0, 1)), dtype=np.float64)
    last_strip = np.asarray(np.mean(lab[:, -4:], axis=(0, 1)), dtype=np.float64)
    strip = float(np.linalg.norm(first_strip - last_strip))
    limit = max(2.0, 1.5 * inside)
    return AxisSeam(edge, inside, limit, strip, edge <= 6 and edge <= limit and strip <= 4)


def measure_seams(image: Image.Image) -> SeamResult:
    """Apply preregistered CIE76 edge/strip bounds independently on both axes."""
    if min(image.size) < 8:
        raise PipelineError('Seam measurement requires at least 8 pixels per side')
    lab = rgb_to_lab(np.array(image.convert('RGB')))
    x = axis_seam(lab)
    y = axis_seam(lab.transpose(1, 0, 2))
    return SeamResult(x, y, x.passed and y.passed)


def repeat_preview(image: Image.Image) -> Image.Image:
    """Make a 4x4 exact repeat without blending or seam repair."""
    output = Image.new('RGBA', (image.width * 4, image.height * 4))
    for y in range(4):
        for x in range(4):
            output.paste(image, (x * image.width, y * image.height))
    return output


def cut_tiles(image: Image.Image, count: int) -> tuple[Image.Image, ...]:
    """Cut only predeclared 32px squares, never search for better-scoring crops."""
    if image.size != (256, 256) or not 1 <= count <= len(POSITIONS):
        raise PipelineError('Ground crops require a 256 square and one to four tiles')
    return tuple(image.crop((x, y, x + 32, y + 32)) for x, y in POSITIONS[:count])


@dataclass(frozen=True, slots=True)
class GroundRecord:
    source: str
    source_sha256: str
    source_size: tuple[int, int]
    image: str
    output_sha256: str
    palette_sha256: str
    moved_over_8: int
    texture_seams: SeamResult
    tile_seams: tuple[SeamResult, ...]
    tile_positions: Sequence[tuple[int, int]]
    distinct_tiles: bool
    metric_passed: bool


def process_ground(source: Path, stem: Path, count: int) -> GroundRecord:
    """Persist unrepaired mode-reduced texture, prescribed crops and repeat evidence."""
    if stem.resolve().is_relative_to(ROOT):
        raise PipelineError('Ground evidence must remain outside the repository')
    with Image.open(source) as opened:
        rgba = opened.convert('RGBA')
        pixels: NDArray[np.uint8] = np.array(rgba)
        opaque = np.equal(pixels[:, :, 3], 255)
        if not np.all(opaque):
            raise PipelineError('Ground source must be opaque edge to edge')
        reduced = pixelize(rgba, (256, 256))
        source_size = opened.size
    palette = ROOT / 'art/palette/worldsim_b.json'
    mapped = map_palette(reduced, load_palette(palette))
    tiles = cut_tiles(mapped.image, count)
    seam = measure_seams(mapped.image)
    tile_seams = tuple(measure_seams(tile) for tile in tiles)
    distinct = len({tile.tobytes() for tile in tiles}) == count
    stem.parent.mkdir(parents=True, exist_ok=True)
    image_path = stem.with_suffix('.png')
    mapped.image.save(image_path)
    repeat_preview(mapped.image).save(stem.with_name(stem.name + '_repeat.png'))
    for index, tile in enumerate(tiles):
        tile.save(stem.with_name(stem.name + f'_tile_{index}.png'))
        repeat_preview(tile).save(stem.with_name(stem.name + f'_tile_{index}_repeat.png'))
    record = GroundRecord(str(source), hashlib.sha256(source.read_bytes()).hexdigest(),
                          source_size, str(image_path), hashlib.sha256(image_path.read_bytes()).hexdigest(),
                          hashlib.sha256(palette.read_bytes()).hexdigest(), mapped.moved_over_8,
                          seam, tile_seams, POSITIONS[:count], distinct,
                          seam.passed and all(s.passed for s in tile_seams) and distinct)
    _ = stem.with_suffix('.json').write_text(json.dumps(asdict(record), indent=2) + '\n')
    return record


def main() -> int:
    if len(sys.argv) != 4:
        raise PipelineError('Usage: RAW OUTPUT_STEM TILE_COUNT')
    record = process_ground(Path(sys.argv[1]), Path(sys.argv[2]), int(sys.argv[3]))
    print(json.dumps(asdict(record), indent=2))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
