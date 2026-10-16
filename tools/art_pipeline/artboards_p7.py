"""Compose actual captured pixels at 1x, with SHA-bound source manifests.

Run: python3 tools/art_pipeline/artboards_p7.py RAW_ART RAW_SOCIETY CALIBRATION_PNG
The optional calibration must be an actual captured production renderer fixture.
"""
from __future__ import annotations
import hashlib
import json
from pathlib import Path
import sys
from PIL import Image, ImageDraw

OUTPUT = Path('docs/art/production7')


def board(name: str, paths: list[Path], caption: str) -> dict:
    """Retain all screenshot pixels without resizing or modifying evidence."""
    images = [Image.open(path).convert('RGB') for path in paths]
    width = max(image.width for image in images)
    height = sum(image.height + 28 for image in images) + 32
    canvas = Image.new('RGB', (width, height), '#172026')
    draw = ImageDraw.Draw(canvas)
    draw.text((8, 8), caption, fill='white')
    y = 32
    sources = []
    for path, image in zip(paths, images, strict=True):
        draw.text((8, y + 6), path.name, fill='white')
        canvas.paste(image, (0, y + 28))
        sources.append({'path': str(path), 'sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
                        'board_rect': [0, y + 28, image.width, image.height], 'scale': 1})
        y += image.height + 28
    target = OUTPUT / name
    canvas.save(target, optimize=True)
    return {'file': str(target), 'sha256': hashlib.sha256(target.read_bytes()).hexdigest(),
            'sources': sources, 'caption': caption}


def baseline(kind: str, filename: str) -> Path:
    """Resolve archived original PNG paths rather than recompressed publications."""
    manifest = OUTPUT / 'baseline' / kind / filename
    row = json.loads(manifest.read_text())['captures'][0]
    return Path(row['raw_file'])


def main(art: Path, society: Path, calibration: Path) -> None:
    records = []
    records.append(board('villages.png', [society / f'{stem}.png' for stem in (
        'soc_d29_1200_FV_all', 'soc_d29_1200_FV_all_selected_orig',
        'soc_d29_1200_FV_all_selected_daughter', 'soc_d85_0842_FV_all')],
        'Natural society seed 20261013; unselected / selected original / selected daughter / three villages. Pixels 1x.'))
    records.append(board('founding.png', [society / f'{stem}.png' for stem in (
        'soc_d29_0630_FD_leader', 'soc_d29_0800_FD_leader',
        'soc_d29_0630_FT_site', 'soc_d29_0819_FS_daughter')],
        'Natural departure, walking, proposed site, founded daughter. Off-screen markers remain unobserved. Pixels 1x.'))
    records.append(board('before-after-live.png', [baseline('live_bound', 'manifest.json'), art / 'a_d2_1200_F2.png'],
        'Merged baseline / final live seed, day 2 noon F2. Pixels 1x.'))
    records.append(board('before-after-society.png', [baseline('society', 'manifest_art_society.json'), society / 'soc_d29_0909_FV_all.png'],
        'Merged baseline / final society seed, day 29 09:09 FV. Pixels 1x.'))
    if calibration.exists():
        records.append(board('deaths.png', [calibration, society / 'soc_d85_0101_DE_daughter.png'],
            'Five-reason controlled production-renderer fixture / natural daughter exposure and feed. Pixels 1x.'))
    else:
        raise FileNotFoundError(f'Actual five-reason renderer capture is required: {calibration}')
    (OUTPUT / 'artboard_sources.json').write_text(json.dumps(records, indent=2) + '\n')
    print(json.dumps([{'file': row['file'], 'bytes': Path(row['file']).stat().st_size} for row in records]))


if __name__ == '__main__':
    main(*(Path(argument) for argument in sys.argv[1:]))
