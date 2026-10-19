# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/decal_refit.py
from __future__ import annotations

from copy import deepcopy
import hashlib
import json
from pathlib import Path
from typing import Final

from PIL import Image, ImageDraw

from palette_tools import Color
from pixel_normalize_v4 import FitResult, fit_cell_v4, measure_cell_v4
from salvage_v4 import guard_data, guarded_candidate
from v4_contracts import guard_frame

ROOT: Final = Path(__file__).resolve().parents[2]
OUTPUT: Final = ROOT / 'docs/art/production3/decals'


class DecalFitError(ValueError):
    code = 'decal_fit_failed'


def native_limit(name: str) -> int:
    return 6 if name.startswith('leaf_') else 8


def fit_decal(raw: Image.Image, palette: list[Color], limit: int) -> FitResult:
    metadata = measure_cell_v4(raw)
    factor = min(1.0, limit / max(metadata['native_size']))
    for _ in range(3):
        result = fit_cell_v4(raw, palette, factor, measurement=metadata)
        if max(result.native.size) <= limit:
            return result
        factor *= (limit - 0.25) / max(result.native.size)
    raise DecalFitError(f'Native dimensions exceed {limit} after three source-grid fits')


def main() -> None:
    source = ROOT / 'docs/art/production2/candidates_round2.json'
    data = json.loads(source.read_text())
    palette = [tuple(color['rgb']) for color in json.loads(
        (ROOT / 'tools/art_pipeline/production_palette.json').read_text())['colors']]
    OUTPUT.mkdir(parents=True, exist_ok=True)
    records = []
    boards = []
    for old in data['candidates']:
        if old['sheet'] != 'N3':
            continue
        record = deepcopy(old)
        record['id'] += ':p3decal'
        record['review_status'] = 'pending'
        record['source_candidate'] = old['id']
        record['source_raw_sha256'] = hashlib.sha256(Path(old['raw_path']).read_bytes()).hexdigest()
        limit = native_limit(old['name'])
        record['native_limit'] = limit
        folder = OUTPUT / 'cache' / record['id'].replace('@', '_').replace(':', '_')
        folder.mkdir(parents=True, exist_ok=True)
        with Image.open(old['raw_path']) as opened:
            raw = opened.convert('RGBA').crop(tuple(old['raw_box']))
        try:
            fit = fit_decal(raw, palette, limit)
        except (ValueError, IndexError) as error:
            record['conversion_error'] = f'{type(error).__name__}: {error}'
            record['guards'] = {'passed': False, 'errors': ['conversion_failure'], 'warnings': []}
            records.append(record)
            continue
        fit.native.save(folder / 'native.png')
        fit.sprite.save(folder / 'sprite.png')
        record['images'] = {'native': str(folder / 'native.png'), 'sprite': str(folder / 'sprite.png')}
        record['factor'] = fit.metrics['factor']
        record['factor_source'] = 'p3_native_decal_limit'
        record['metrics'] = fit.metrics
        record['conversion_error'] = None
        record['guards'] = guard_data(guard_frame(guarded_candidate(record)))
        records.append(record)
        card = Image.new('RGB', (600, 240), (38, 41, 45))
        draw = ImageDraw.Draw(card)
        draw.text((8, 8), record['id'] + ' ' + old['name'], fill='white')
        draw.text((8, 28), 'source thumbnail / prior 6x / refit 8x / refit 2x', fill='white')
        raw.thumbnail((170, 170), Image.Resampling.NEAREST)
        card.paste(raw, (8, 55))
        with Image.open(old['images']['sprite']) as opened:
            prior = opened.convert('RGBA')
        for sprite, scale, offset in ((prior, 6, (190, 65)), (fit.sprite, 8, (320, 65)), (fit.sprite, 2, (505, 80))):
            enlarged = sprite.resize((sprite.width * scale, sprite.height * scale), Image.Resampling.NEAREST)
            card.paste(enlarged, offset, enlarged)
        boards.append(card)
    for start in range(0, len(boards), 8):
        board = Image.new('RGB', (1200, 960), (38, 41, 45))
        for index, card in enumerate(boards[start:start + 8]):
            board.paste(card, ((index % 2) * 600, (index // 2) * 240))
        board.save(OUTPUT / f'review_{start // 8 + 1:02d}.png')
    (OUTPUT / 'candidates.json').write_text(json.dumps({
        'schema_version': 1, 'scale': data['scale'],
        'source_ledger': str(source.relative_to(ROOT)),
        'source_sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
        'candidates': records + [old for old in data['candidates'] if old['sheet'] == 'N3'],
        'review_status': 'pending'}, indent=2) + '\n')
    print(f'Refitted {len(boards)}/{len(records)} N3 candidates; all require fresh visual review')


if __name__ == '__main__':
    main()
