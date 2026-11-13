# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy"]
# ///
# How to run: python -m art.batch_r1.process RAW LOOK OUT --reference SHIPPED_LOOK
"""Normalize one ten-panel candidate and preserve structural comparison evidence."""
from __future__ import annotations

import getopt
import json
import re
import sys
from dataclasses import asdict, dataclass
from hashlib import sha256
from pathlib import Path
from typing import Final

from PIL import Image, ImageDraw

from art.pipeline5.attempt_integrity import digest
from art.pipeline5.sheet_geometry import Normalization, normalize_panel, split_panels
from art.pipeline5.whole_checks import FRAMES, WholeResult, check_whole

FACINGS: Final = ('S', 'E')
REPOSITORY: Final = Path(__file__).resolve().parents[2]


@dataclass(frozen=True, slots=True)
class Request:
    raw: Path
    look: str
    output: Path
    reference: Path

    def __post_init__(self) -> None:
        if re.fullmatch(r'(adult|child)_[MF]_[a-z0-9_]+', self.look) is None:
            message = 'look must start adult_M_, adult_F_, child_M_, or child_F_'
            raise ValueError(message)
        if self.output.resolve().is_relative_to(REPOSITORY):
            message = 'candidate evidence output must be outside the repository'
            raise ValueError(message)

    @property
    def age(self) -> str:
        return self.look.split('_')[0]


@dataclass(frozen=True, slots=True)
class Frame:
    facing: str
    frame: str
    raw_sha256: str
    reference_sha256: str
    image_sha256: str
    pixel_sha256: str
    standing_scale: float
    source_box: tuple[int, int, int, int]
    normalization: Normalization | None
    checks: WholeResult
    error: str | None = None


@dataclass(frozen=True, slots=True)
class Summary:
    look: str
    reference: str
    raw: str
    raw_sha256: str
    structural_pass: bool
    frames: tuple[Frame, ...]
    error: str | None = None
    acceptance: str = 'structural only; style metrics and visual review required'


def contact_sheet(output: Path, reference: Path, label: str) -> None:
    """Place shipped/candidate pairs together at native and doubled game scale."""
    sheet = Image.new('RGB', (10 * 116, 2 * 132), '#dfd6c4')
    draw = ImageDraw.Draw(sheet)
    for row, facing in enumerate(FACINGS):
        y = row * 132
        draw.text((4, y), f'{label} {facing}: reference / candidate; 1x then 2x', fill='black')
        for column, frame in enumerate(FRAMES):
            for side, root in enumerate((reference, output)):
                with Image.open(root / facing / (frame + '.png')) as image:
                    x = (column * 2 + side) * 116
                    sheet.paste(image, (x, y + 20), image)
    sheet.save(output / 'comparison_1x.png')
    doubled_sheet = Image.new('RGB', (5 * 448, 2 * 250), '#dfd6c4')
    draw = ImageDraw.Draw(doubled_sheet)
    for row, facing in enumerate(FACINGS):
        draw.text((4, row * 250), f'{label} {facing}: reference / candidate 2x', fill='black')
        for column, frame in enumerate(FRAMES):
            for side, root in enumerate((reference, output)):
                with Image.open(root / facing / (frame + '.png')) as image:
                    enlarged = image.resize((image.width * 2, image.height * 2), Image.Resampling.NEAREST)
                    doubled_sheet.paste(enlarged, (column * 448 + side * 224, row * 250 + 20), enlarged)
    doubled_sheet.save(output / 'comparison_2x.png')


def process(request: Request) -> Summary:
    """Require an unused evidence directory and leave failed checks visible."""
    raw_hash = digest(request.raw)
    references = {f'{facing}/{frame}': digest(request.reference / facing / (frame + '.png'))
                  for facing in FACINGS for frame in FRAMES}
    request.output.mkdir(parents=True, exist_ok=False)
    frames: list[Frame] = []
    error: str | None = None
    try:
        with Image.open(request.raw) as source:
            if source.mode != 'RGBA' or source.size != (1536, 1024):
                message = f'raw_sheet: expected 1536x1024 RGBA, got {source.size} {source.mode}'
                raise ValueError(message)
            panels = split_panels(source, rows=2)
        target = 40 if request.age == 'child' else 56
        scale = target / panels.idle_height
        for index, panel in enumerate(panels.images):
            facing, frame = FACINGS[index // 5], FRAMES[index % 5]
            directory = request.output / facing
            directory.mkdir(exist_ok=True)
            path = directory / (frame + '.png')
            try:
                normalized = normalize_panel(panel, scale, frame == 'sleep', sleep_length=target)
                normalized.image.save(path)
                result = Frame(facing, frame, raw_hash, references[f'{facing}/{frame}'],
                               digest(path), sha256(normalized.image.tobytes()).hexdigest(), scale,
                               panels.boxes[index], normalized.transform,
                               check_whole(normalized.image, request.age, frame))
            except ValueError as failure:
                reason = str(failure)
                result = Frame(facing, frame, raw_hash, references[f'{facing}/{frame}'], '', '',
                               scale, panels.boxes[index], None,
                               WholeResult(False, (reason,), 0, 0, 0, False), reason)
            _ = path.with_suffix('.json').write_text(json.dumps(asdict(result), indent=2) + '\n')
            frames.append(result)
        if all(frame.image_sha256 for frame in frames):
            contact_sheet(request.output, request.reference, request.look)
    except (ValueError, OSError) as failure:
        error = str(failure)
    summary = Summary(request.look, str(request.reference.resolve()), str(request.raw.resolve()),
                      raw_hash, len(frames) == 10 and all(frame.checks.passed for frame in frames),
                      tuple(frames), error)
    _ = (request.output / 'summary.json').write_text(json.dumps(asdict(summary), indent=2) + '\n')
    return summary


def main() -> int:
    """Parse the requested positional command without adding dependencies."""
    try:
        if len(sys.argv) != 6:
            message = 'usage: RAW LOOK OUT --reference SHIPPED_LOOK'
            raise ValueError(message)
        options, rest = getopt.getopt(sys.argv[4:], '', ['reference='])
        if rest or len(options) != 1 or options[0][0] != '--reference':
            message = 'require --reference SHIPPED_LOOK'
            raise ValueError(message)
        request = Request(Path(sys.argv[1]), sys.argv[2], Path(sys.argv[3]), Path(options[0][1]))
        result = process(request)
        print(json.dumps(asdict(result)))
        return 0 if result.structural_pass else 1
    except (ValueError, OSError, getopt.GetoptError) as failure:
        print(json.dumps({'structural_pass': False, 'error': str(failure)}))
        return 2


if __name__ == '__main__':
    raise SystemExit(main())
