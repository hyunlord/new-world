# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# Run: python3 tools/art_pipeline/presentation_p6.py RAW_ART RAW_LONG RAW_GOD
"""Publish actual captured pixels with explicit natural/intervention provenance."""
from __future__ import annotations
import gzip
import hashlib
import json
from pathlib import Path
import sys
from typing import Final
from PIL import Image, ImageDraw
from report_artifacts import animation

ROOT: Final = Path(__file__).resolve().parents[2]
OUTPUT: Final = ROOT / 'docs/art/production6'


def source(directory: Path, stem: str) -> Path:
    """Historical JPEGs and new lossless captures share exact named framings."""
    for suffix in ('.png', '.jpg'):
        path = directory / (stem + suffix)
        if path.exists():
            return path
    raise FileNotFoundError(directory / stem)


def metadata(path: Path):
    """Read historical compressed metadata without altering it."""
    plain = path.with_suffix('.entities.json')
    return json.loads(plain.read_text() if plain.exists() else gzip.decompress(plain.with_suffix('.json.gz').read_bytes()))


def panel(path: Path, width: int = 640) -> Image.Image:
    """Return nearest-neighbour capture thumbnail, never fabricated content."""
    with Image.open(path) as image:
        return image.convert('RGB').resize((width, round(image.height * width / image.width)), Image.Resampling.NEAREST)


def before_after(raw: Path, long_raw: Path):
    """Compare Pass 5 screenshots at matching named framings and record drift."""
    board = Image.new('RGB', (1280, 990), '#191c1c')
    draw = ImageDraw.Draw(board)
    records = []
    for row, (directory, stem) in enumerate(((long_raw, 'l_d29_0200_F2N'), (raw, 'a_d2_1200_F2'))):
        old = source(ROOT / 'docs/art/production5/shots', stem)
        new = source(directory, stem)
        for column, path in enumerate((old, new)):
            board.paste(panel(path), (column * 640, row * 390 + 25))
            draw.text((column * 640 + 8, row * 390 + 5), f'Pass {5 + column}: {stem}', fill='white')
        a, b = metadata(old), metadata(new)
        records.append({'before': str(old), 'after': str(new), 'same_camera': a['camera_position'] == b['camera_position'], 'same_zoom': a['zoom'] == b['zoom']})
    for column, directory in enumerate((ROOT / 'docs/art/production5/shots', raw)):
        path = source(directory, 'a_lineup_z2')
        data = metadata(path)
        actor = next(row for row in data['entities'] if row['kind'] == 'agent' and row.get('activity') == 'carry' and row.get('visible'))
        scale = float(data.get('framebuffer_scale', 1))
        x, y, w, h = actor.get('cue_rect', actor['rect'])
        with Image.open(path) as image:
            original = image.convert('RGB').resize((1920, 1080), Image.Resampling.NEAREST)
            crop = original.crop(tuple(round(value / scale) for value in (x, y, x+w, y+h)))
        board.paste(crop, (column * 640 + 260, 830))
        draw.text((column * 640 + 8, 795), f'Pass {5+column}: actual 2x carry lineup / {actor.get("frame_id")}', fill='white')
        records.append({'carry_source': str(path), 'frame': actor.get('frame_id'), 'scope': 'captured 2x lineup crop'})
    board.save(OUTPUT / 'before_after.png', optimize=True)
    return records


def cold_night(long_raw: Path):
    """No controlled-scene substitute is permitted for the natural close-up."""
    path = source(long_raw, 'l_d28_0200_F2Z')
    data = metadata(path)
    if data.get('intervention_casts') or data.get('scenario') != 'NATURAL SCENARIO' or not any(data.get('agent_snapshot', {}).get('colds', [])):
        raise RuntimeError('Cold close-up lacks natural cold snapshot provenance')
    with Image.open(path) as image:
        image.save(OUTPUT / 'cold_night.png', optimize=True)
    return str(path)


def seasons(long_raw: Path):
    """A labelled strip is assembled only from this pass's actual season frames."""
    board = Image.new('RGB', (1280, 208), '#191c1c')
    draw = ImageDraw.Draw(board)
    paths = []
    for column, (day, season) in enumerate(((10, 'Summer'), (18, 'Autumn'), (24, 'Winter'), (32, 'Spring'))):
        path = source(long_raw, f'l_d{day}_1200_F2')
        board.paste(panel(path, 320), (column * 320, 28))
        draw.text((column * 320 + 8, 8), f'd{day}: {season}', fill='white')
        paths.append(str(path))
    board.save(OUTPUT / 'season_strip.png', optimize=True)
    return paths


def god_board(god_raw: Path):
    """Select the panel and four effect frames by their recorded cast context."""
    manifest = json.loads((god_raw / 'manifest.json').read_text())
    frames = [(god_raw / row['file'], metadata(god_raw / row['file'])) for row in manifest['captures']]
    selected = []
    for kind in ('panel', 'rain', 'cold_snap', 'windfall', 'gift_wood'):
        candidates = [(path, data) for path, data in frames if (kind == 'panel' and 'panel' in path.stem) or (kind != 'panel' and data.get('intervention_casts') and data['intervention_casts'][-1]['kind'] == kind)]
        if not candidates:
            raise FileNotFoundError('Missing god-hand panel/effect: ' + kind)
        selected.append((kind, candidates[0][0]))
    board = Image.new('RGB', (1280, 1170), '#191c1c')
    draw = ImageDraw.Draw(board)
    for index, (kind, path) in enumerate(selected):
        x, y = index % 2 * 640, index // 2 * 390
        board.paste(panel(path), (x, y + 25))
        draw.text((x + 8, y + 5), 'INTERVENTION SCENARIO: ' + kind, fill='white')
    board.save(OUTPUT / 'god_hand.png', optimize=True)
    return [str(path) for _, path in selected]


def carry_board(raw: Path):
    """Compare actual 2x lineup carry/eat pixels, including the attached load."""
    paths = (source(ROOT / 'docs/art/production5/shots', 'a_lineup_z2'), source(raw, 'a_lineup_z2'))
    rows = []
    for column, path in enumerate(paths):
        data = metadata(path)
        scale = float(data.get('framebuffer_scale', 1))
        selected = [row for row in data['entities'] if row['kind'] == 'agent' and row.get('activity') in ('carry', 'carry_wood', 'eat') and row.get('visible')]
        with Image.open(path) as image:
            normalized = image.convert('RGB').resize((1920, 1080), Image.Resampling.NEAREST)
            for row in selected:
                x, y, w, h = row.get('cue_rect', row['rect'])
                crop = normalized.crop(tuple(round(value / scale) for value in (x, y, x + w, y + h)))
                rows.append((column, row, crop))
    board = Image.new('RGB', (1000, 40 + max(1, len(rows)) * 100), '#343934')
    draw = ImageDraw.Draw(board)
    for index, (column, row, crop) in enumerate(rows):
        y = 35 + index * 100
        draw.text((8, y), f'Pass {5+column}: {row.get("body")} {row.get("activity")} {row.get("frame_id")}', fill='white')
        board.paste(crop, (500, y))
    board.save(OUTPUT / 'carry_lineup_2x.png', optimize=True)
    return [str(path) for path in paths]


def main(raw: Path, long_raw: Path, god_raw: Path) -> None:
    """Keep missing artifacts explicit rather than substituting staged pictures."""
    from carry_comparison_p6 import exhaustive_carry
    OUTPUT.mkdir(parents=True, exist_ok=True)
    evidence = {'sources': {}, 'missing': {}}
    jobs = {'before_after': lambda: before_after(raw, long_raw), 'cold_night': lambda: cold_night(long_raw), 'season_strip': lambda: seasons(long_raw), 'god_hand': lambda: god_board(god_raw), 'carry_lineup_2x': lambda: carry_board(raw), 'carry_2x': lambda: exhaustive_carry(OUTPUT / 'carry_2x.png'), 'village_noon': lambda: animation(raw, OUTPUT, 'strip.json', 'village_noon.gif'), 'village_evening': lambda: animation(raw, OUTPUT, 'evening.json', 'village_evening.gif')}
    for name, job in jobs.items():
        try:
            evidence['sources'][name] = job()
        except (FileNotFoundError, RuntimeError) as error:
            evidence['missing'][name] = str(error)
    names = ('before_after.png', 'cold_night.png', 'god_hand.png', 'carry_2x.png', 'carry_lineup_2x.png', 'season_strip.png', 'village_noon.gif', 'village_evening.gif')
    evidence['outputs'] = [{'file': name, 'sha256': hashlib.sha256((OUTPUT / name).read_bytes()).hexdigest(), 'bytes': (OUTPUT / name).stat().st_size} for name in names if (OUTPUT / name).exists()]
    evidence['budget_bytes'] = {str(folder.relative_to(ROOT)): sum(path.stat().st_size for path in folder.rglob('*') if path.is_file() and path.suffix != '.import') for folder in (ROOT / 'assets/art', OUTPUT)}
    evidence['budget_pass'] = evidence['budget_bytes']['assets/art'] <= 25_000_000 and evidence['budget_bytes']['docs/art/production6'] <= 30_000_000
    (OUTPUT / 'presentation_artifacts.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(json.dumps(evidence, indent=2))


if __name__ == '__main__':
    main(*(Path(argument) for argument in sys.argv[1:]))
