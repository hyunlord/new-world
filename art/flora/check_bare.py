# /// script
# requires-python = ">=3.11"
# dependencies = ["numpy", "pillow", "pydantic"]
# ///
# How to run: python3 -m art.flora.check_bare
"""Read-only paired winter silhouette evidence; never infer exact preservation."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import ClassVar, Final

import numpy as np
from numpy.typing import NDArray
from PIL import Image, ImageDraw
from pydantic import BaseModel, ConfigDict

from art.flora.normalize import NormalizedRecord, apply_transform
from art.flora.summer import NormalizedInputs

ROOT: Final = Path(__file__).resolve().parents[2]
DATA: Final = Path('/Users/rexxa/work/arttrees-data')


class SilhouetteMetrics(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    lower_box_y: int
    lower_alpha128_intersection: int
    lower_alpha128_union: int
    lower_alpha128_iou: float
    lower_alpha_bytes_exact_fraction: float
    lower_alpha_bytes_exact_on_visible_union_fraction: float
    summer_base_row: int | None
    bare_base_row: int | None
    base_delta_px: int | None
    summer_root_span_at_base: tuple[int, int] | None
    bare_root_span_at_base: tuple[int, int] | None
    exact_lower_silhouette: bool


class VisualReading(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    bare_sha256: str
    summer_sha256: str
    reading: str
    exact_preservation_claimed: bool = False


class VisualReadings(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    readings: tuple[VisualReading, ...] = ()


class BareEvidence(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    bare_id: str
    attempt: int
    summer_path: str
    bare_path: str
    summer_sha256: str
    bare_sha256: str
    summer_raw_sha256: str
    bare_raw_sha256: str
    raw_hashes_match_ledger: bool
    source_canvas_matches: bool
    identical_registered_transform: bool
    summer_alpha_reproduced_from_raw: bool
    bare_alpha_reproduced_from_raw: bool
    declared_anchor: tuple[int, int]
    summer_base_anchor_error: int | None
    bare_base_anchor_error: int | None
    silhouette: SilhouetteMetrics
    visual: VisualReading | None


def compare_silhouettes(summer: Image.Image, bare: Image.Image) -> SilhouetteMetrics:
    """Measure lower30% of the full canvas, reporting active and full-box alpha identity."""
    if summer.size != bare.size:
        message = f'Paired winter canvases differ: {summer.size}, {bare.size}'
        raise ValueError(message)
    a: NDArray[np.uint8] = np.asarray(summer.convert('RGBA'), dtype=np.uint8)[..., 3]
    b: NDArray[np.uint8] = np.asarray(bare.convert('RGBA'), dtype=np.uint8)[..., 3]
    lower = int(summer.height * .7)
    ap = a[lower:] >= 128
    bp = b[lower:] >= 128
    intersection = int(np.count_nonzero(ap & bp))
    union = int(np.count_nonzero(ap | bp))
    visible = (a[lower:] > 0) | (b[lower:] > 0)
    exact = a[lower:] == b[lower:]
    visible_count = int(np.count_nonzero(visible))
    summer_rows = np.nonzero(a >= 128)[0]
    bare_rows = np.nonzero(b >= 128)[0]
    sa = int(summer_rows.max()) if summer_rows.size else None
    ba = int(bare_rows.max()) if bare_rows.size else None
    sx = np.nonzero(a[sa] >= 128)[0] if sa is not None else np.array([], dtype=np.int64)
    bx = np.nonzero(b[ba] >= 128)[0] if ba is not None else np.array([], dtype=np.int64)
    return SilhouetteMetrics(lower_box_y=lower, lower_alpha128_intersection=intersection,
                             lower_alpha128_union=union, lower_alpha128_iou=intersection / union if union else 1,
                             lower_alpha_bytes_exact_fraction=int(np.count_nonzero(exact)) / exact.size,
                             lower_alpha_bytes_exact_on_visible_union_fraction=int(np.count_nonzero(exact & visible)) / visible_count if visible_count else 1,
                             summer_base_row=sa, bare_base_row=ba,
                             base_delta_px=ba-sa if sa is not None and ba is not None else None,
                             summer_root_span_at_base=(int(sx.min()), int(sx.max())) if sx.size else None,
                             bare_root_span_at_base=(int(bx.min()), int(bx.max())) if bx.size else None,
                             exact_lower_silhouette=bool(np.array_equal(ap, bp)))


def evidence(pair: tuple[NormalizedRecord, NormalizedRecord], readings: VisualReadings) -> BareEvidence:
    """Verify raw hashes and replay the registered geometry in memory only."""
    summer_record, bare_record = pair
    with Image.open(ROOT / summer_record.output) as opened:
        summer = opened.convert('RGBA')
    with Image.open(ROOT / bare_record.output) as opened:
        bare = opened.convert('RGBA')
    with Image.open(summer_record.preserved_original) as opened:
        raw_summer = opened.convert('RGBA')
    with Image.open(bare_record.preserved_original) as opened:
        raw_bare = opened.convert('RGBA')
    summer_hash = hashlib.sha256((ROOT / summer_record.output).read_bytes()).hexdigest()
    bare_hash = hashlib.sha256((ROOT / bare_record.output).read_bytes()).hexdigest()
    measurements = compare_silhouettes(summer, bare)
    visual = next((row for row in readings.readings if row.bare_sha256 == bare_hash and row.summer_sha256 == summer_hash), None)
    reproduced_summer = apply_transform(raw_summer, summer_record.transform)
    reproduced_bare = apply_transform(raw_bare, bare_record.transform)
    return BareEvidence(
        bare_id=bare_record.id, attempt=bare_record.attempt, summer_path=summer_record.output,
        bare_path=bare_record.output, summer_sha256=summer_hash, bare_sha256=bare_hash,
        summer_raw_sha256=summer_record.source_sha256, bare_raw_sha256=bare_record.source_sha256,
        raw_hashes_match_ledger=all(hashlib.sha256(Path(row.preserved_original).read_bytes()).hexdigest() == row.source_sha256 for row in pair),
        source_canvas_matches=raw_summer.size == raw_bare.size,
        identical_registered_transform=summer_record.transform == bare_record.transform,
        summer_alpha_reproduced_from_raw=summer.getchannel('A').tobytes() == reproduced_summer.getchannel('A').tobytes(),
        bare_alpha_reproduced_from_raw=bare.getchannel('A').tobytes() == reproduced_bare.getchannel('A').tobytes(),
        declared_anchor=bare_record.transform.anchor,
        summer_base_anchor_error=measurements.summer_base_row - summer_record.transform.anchor[1] if measurements.summer_base_row is not None else None,
        bare_base_anchor_error=measurements.bare_base_row - bare_record.transform.anchor[1] if measurements.bare_base_row is not None else None,
        silhouette=measurements, visual=visual)


def review_board(rows: tuple[BareEvidence, ...]) -> Path:
    """External review-only pairs at native1x and nearest2x; never modify sprites."""
    board = Image.new('RGB', (1216, len(rows) * 420), (191, 201, 166))
    drawing = ImageDraw.Draw(board)
    for index, row in enumerate(rows):
        y = index * 420
        drawing.text((10, y+6), f'{row.bare_id} attempt{row.attempt}: summer/bare 1x | summer/bare 2x', fill=(30, 40, 25))
        x = 10
        for zoom in (1, 2):
            for path in (row.summer_path, row.bare_path):
                with Image.open(ROOT / path) as opened:
                    source = opened.convert('RGBA')
                displayed = source.resize((source.width * zoom, source.height * zoom), Image.Resampling.NEAREST)
                board.paste(displayed, (x, y + 410 - displayed.height), displayed)
                x += displayed.width + 12
    path = DATA / 'bare_pairs_1x2x.png'
    board.save(path, optimize=True)
    frozen = DATA / f'bare_pairs_{hashlib.sha256(path.read_bytes()).hexdigest()[:16]}.png'
    if not frozen.exists():
        _ = frozen.write_bytes(path.read_bytes())
    return frozen


def main() -> int:
    """Emit current paired measurements and preserve immutable external snapshots."""
    generation = NormalizedInputs.model_validate_json((ROOT / 'docs/v8/arttrees/GENERATION.json').read_bytes())
    lookup = {row.id: row for row in generation.assets}
    visual_path = DATA / 'bare_readings.json'
    readings = VisualReadings.model_validate_json(visual_path.read_bytes()) if visual_path.exists() else VisualReadings()
    rows = tuple(evidence((lookup[row.transform_reference], row), readings) for row in generation.assets if row.id.endswith('_bare'))
    board = review_board(rows)
    report = {'method': 'Lower30% full canvas silhouette at alpha128; exact alpha both full box and visible union. No invented IoU acceptance threshold; exact preservation is not implied by sharing a transform.',
              'diagnostic_board': str(board), 'diagnostic_board_sha256': hashlib.sha256(board.read_bytes()).hexdigest(),
              'pairs': [row.model_dump(mode='json') for row in rows]}
    encoded = json.dumps(report, indent=2) + '\n'
    _ = (ROOT / 'docs/v8/arttrees/BARE_CHECKS.json').write_text(encoded)
    snapshot = DATA / f'bare_checks_{hashlib.sha256(encoded.encode()).hexdigest()[:16]}.json'
    if not snapshot.exists():
        _ = snapshot.write_text(encoded)
    for row in rows:
        print(row.bare_id, 'IoU', round(row.silhouette.lower_alpha128_iou, 4), 'base delta', row.silhouette.base_delta_px,
              'same transform', row.identical_registered_transform, 'exact', row.silhouette.exact_lower_silhouette)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
