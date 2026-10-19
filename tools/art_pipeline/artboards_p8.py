# /// script
# requires-python = ">=3.11"
# dependencies = ["Pillow"]
# ///
# Run: python3 tools/art_pipeline/artboards_p8.py BOARD_CONFIG_JSON
"""Compose seven SHA-bound pixel boards from explicitly selected real captures."""
from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path
from typing import Final, TypedDict

from PIL import Image, ImageDraw

OUTPUT: Final = Path('docs/art/production8')
NAMES: Final = ('news', 'way', 'restless', 'crowds', 'effects', 'before-after-live', 'before-after-society')


class BoardSpec(TypedDict):
    caption: str
    paths: list[str]


class Source(TypedDict):
    path: str
    sha256: str
    board_rect: list[int]
    scale: int


class BoardRecord(TypedDict):
    file: str
    sha256: str
    sources: list[Source]
    caption: str


def board(name: str, spec: BoardSpec) -> BoardRecord:
    """Preserve every source pixel at native resolution with captions outside it."""
    images = []
    paths = [Path(value) for value in spec['paths']]
    if not paths:
        raise FileNotFoundError(name + ': no source screenshots selected')
    for path in paths:
        with Image.open(path) as source:
            images.append(source.convert('RGB'))
    canvas = Image.new('RGB', (max(image.width for image in images), sum(image.height + 28 for image in images) + 32), '#172026')
    draw = ImageDraw.Draw(canvas)
    draw.text((8, 8), spec['caption'], fill='white')
    y = 32
    sources: list[Source] = []
    for path, image in zip(paths, images, strict=True):
        draw.text((8, y + 6), path.name, fill='white')
        canvas.paste(image, (0, y + 28))
        sources.append({'path': str(path.resolve()), 'sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
                        'board_rect': [0, y + 28, image.width, image.height], 'scale': 1})
        y += image.height + 28
    target = OUTPUT / (name + '.png')
    if target.resolve() in {path.resolve() for path in paths}:
        raise FileExistsError('Board destination would overwrite source: ' + str(target))
    canvas.save(target, optimize=True)
    return {'file': str(target), 'sha256': hashlib.sha256(target.read_bytes()).hexdigest(),
            'sources': sources, 'caption': spec['caption']}


def main(config_path: Path) -> None:
    """Require all seven named boards; never substitute a placeholder image."""
    specs = json.loads(config_path.read_text())
    if set(specs) != set(NAMES):
        raise KeyError('Board configuration must contain exactly: ' + ', '.join(NAMES))
    OUTPUT.mkdir(parents=True, exist_ok=True)
    records = [board(name, specs[name]) for name in NAMES]
    (OUTPUT / 'artboard_sources.json').write_text(json.dumps(records, indent=2) + '\n')
    print(json.dumps([{'file': row['file'], 'bytes': Path(row['file']).stat().st_size} for row in records]))


if __name__ == '__main__':
    main(Path(sys.argv[1]))
