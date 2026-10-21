# /// script
# dependencies = ["numpy", "Pillow"]
# ///
# How to run: python3 -m art.pipeline.run_candidate SOURCE WIDTH HEIGHT OUTPUT_STEM
"""Process an object candidate; save external evidence without accepting the art."""
from __future__ import annotations

import hashlib
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Final

from PIL import Image

from .check import CheckResult, check_asset
from .ingest import ingest
from .models import Asset, PipelineError, Size
from .outline import enforce_outline
from .palette import load_palette, map_palette
from .pixelize import pixelize
from .snap import snap

ROOT: Final = Path(__file__).resolve().parents[2]
PALETTE: Final = ROOT / "art/palette/worldsim_b.json"


@dataclass(frozen=True, slots=True)
class CandidateRecord:
    """Reproduction inputs and measurements; acceptance is deliberately separate."""

    source: str
    source_sha256: str
    source_size: Size
    crop: tuple[int, int, int, int]
    reduction: Size
    box: Size
    offset: Size
    palette_sha256: str
    moved_over_8: int
    maximum_delta_e: float
    output_sha256: str
    check: CheckResult
    accepted: bool = False


def fit_size(source: Size, box: Size) -> Size:
    """Fit aspect ratio within a one-pixel margin, without any upscaling."""
    if min(box) < 3:
        raise PipelineError("Object boxes require at least three pixels per side")
    scale = min((box[0] - 2) / source[0], (box[1] - 2) / source[1], 1.0)
    return max(1, int(source[0] * scale)), max(1, int(source[1] * scale))


def process_candidate(source: Path, box: Size, output_stem: Path) -> CandidateRecord:
    """Run ingest→mode→palette→outline→snap→check on one unaccepted object."""
    if output_stem.resolve().is_relative_to(ROOT):
        raise PipelineError("Candidate intermediates must remain outside the repository")
    with Image.open(source) as opened:
        keyed = ingest(opened)
    target = fit_size(keyed.image.size, box)
    reduced = pixelize(keyed.image, target)
    colors = load_palette(PALETTE)
    mapped = map_palette(reduced, colors)
    padded = snap(enforce_outline(mapped.image), box)
    asset = Asset(source.stem, padded.image, box, (box[0] // 2, box[1] - 1))
    report = check_asset(asset, colors)
    output_stem.parent.mkdir(parents=True, exist_ok=True)
    png = output_stem.with_suffix(".png")
    padded.image.save(png, optimize=True)
    padded.image.resize((box[0] * 8, box[1] * 8), Image.Resampling.NEAREST).save(
        output_stem.with_name(output_stem.name + "_8x.png"), optimize=True,
    )
    record = CandidateRecord(
        str(source.resolve()), hashlib.sha256(source.read_bytes()).hexdigest(),
        keyed.source_size, keyed.bbox, target, box, padded.offset,
        hashlib.sha256(PALETTE.read_bytes()).hexdigest(), mapped.moved_over_8,
        mapped.max_delta_e, hashlib.sha256(png.read_bytes()).hexdigest(), report,
    )
    _ = output_stem.with_suffix(".json").write_text(
        json.dumps(asdict(record), indent=2) + "\n", encoding="utf-8",
    )
    return record


def main() -> int:
    """Minimal dependency-free CLI; nonzero status preserves rejected outputs."""
    if len(sys.argv) != 5:
        raise PipelineError("Usage: SOURCE WIDTH HEIGHT OUTPUT_STEM")
    record = process_candidate(
        Path(sys.argv[1]), (int(sys.argv[2]), int(sys.argv[3])), Path(sys.argv[4]),
    )
    print(json.dumps(asdict(record), indent=2))
    return 0 if record.check.passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
