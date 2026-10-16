# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# Run: python3 tools/art_pipeline/presentation_p4.py RAW_ART RAW_LONG REFERENCE_P3
"""Publish compact, provenance-labelled Art Pass 4 presentation artifacts."""
from __future__ import annotations

from dataclasses import asdict, replace
import gzip
import hashlib
import json
from pathlib import Path
import sys
from typing import Final

from PIL import Image, ImageDraw, ImageFont

from ingame_qa_types import Entity, sprite_pixels
from presentation_colour_p4 import P3_COMMIT, colour_comparison, reference_hashes
from report_artifacts import animation

ROOT: Final = Path(__file__).resolve().parents[2]
OUTPUT: Final = ROOT / 'docs/art/production4'
BODIES: Final = ('adult_a', 'adult_b', 'child')


def matched_before_after(raw: Path) -> list[str]:
    """Compare identical named capture framings, including noon, night and overview."""
    stems = ('a_d2_1200_F2', 'a_d2_2300_F2N', 'a_d2_1200_F1')
    board = Image.new('RGB', (1280, 1110), '#191c1c')
    draw = ImageDraw.Draw(board)
    sources = []
    for row, stem in enumerate(stems):
        for column, directory in enumerate((ROOT / 'docs/art/production3/shots', raw)):
            paths = [directory / (stem + suffix) for suffix in ('.png', '.jpg')]
            path = next(path for path in paths if path.is_file())
            sources.append(str(path))
            with Image.open(path) as image:
                panel = image.convert('RGB').resize((640, 360), Image.Resampling.NEAREST)
            board.paste(panel, (column * 640, row * 370 + 10))
            draw.text((column * 640 + 8, row * 370), f'P{3+column} {stem}', fill='white')
    board.save(OUTPUT / 'before_after.png', optimize=True)
    return sources


def seasons(raw: Path) -> list[str]:
    """Label the four actual captured seasons, using day 32 for spring."""
    board = Image.new('RGB', (1280, 208), '#191c1c')
    draw = ImageDraw.Draw(board)
    sources = []
    for index, (day, label) in enumerate(((10, 'Summer'), (18, 'Autumn'), (24, 'Winter'), (32, 'Spring'))):
        path = raw / f'l_d{day}_1200_F2.png'
        sources.append(str(path))
        with Image.open(path) as image:
            board.paste(image.convert('RGB').resize((320, 180), Image.Resampling.NEAREST), (index * 320, 28))
        metadata = json.loads(path.with_suffix('.entities.json').read_text())['time_snapshot']
        draw.text((index * 320 + 8, 8), f'd{day} {label} (runtime season {metadata["season"]})', fill='white')
    board.save(OUTPUT / 'season_strip.png', optimize=True)
    return sources


def natural_sheet() -> list[dict[str, str | int]]:
    """Show one deterministic drawable frame per body/activity at exactly 4x."""
    atlas = json.loads((ROOT / 'assets/art/atlas/world_atlas.json').read_text())['sprites']
    selected = {}
    for ident, entry in sorted(atlas.items(), key=lambda item: (item[1].get('view') != 'S', item[0])):
        if entry.get('body') in BODIES and entry.get('drawable', True):
            selected.setdefault((entry['body'], entry['activity']), (ident, entry))
    for body in BODIES:
        for activity, fallback in (('carry_wood', 'walk'), ('gather_wood', 'crouch')):
            key = (body, activity)
            if key not in selected:
                selected[key] = selected.get((body, fallback), selected[(body, 'idle')])
    row_height = max(entry['rect'][3] * 4 for _, entry in selected.values()) + 24
    cell_width = max(entry['rect'][2] * 4 for _, entry in selected.values()) + 12
    rows = sorted(selected)
    board = Image.new('RGBA', (230 + 15 * cell_width, 32 + row_height * len(rows)), '#555a50')
    draw = ImageDraw.Draw(board)
    font = ImageFont.truetype(str(ROOT / 'assets/fonts/galmuri/Galmuri9.ttf'), 10)
    records = []
    for column in range(15):
        draw.text((230 + column * cell_width, 8), f'T{column//5} H{column%5}', font=font, fill='white')
    for index, (body, activity) in enumerate(rows):
        ident, entry = selected[(body, activity)]
        y = 32 + index * row_height
        draw.text((8, y + 8), f'{body} / {activity}\n{ident}\n{entry.get("qa_status", "")}\n{"wood fallback (no real frame)" if activity.endswith("_wood") and entry["activity"] != activity else entry.get("fallback_kind", "")}', font=font, fill='white')
        entity = Entity('agent', 0, (0,0,0,0), True, ident, ident, (0,0), body,
                        (0,0), (0,0), activity, entry.get('view',''), '', -1, False,
                        entry['page'], tuple(entry['rect']), '')
        for tone in range(3):
            for hair in (range(5) if body != 'child' else (0,)):
                pixels = sprite_pixels(replace(entity, variant=(tone, hair)), ROOT, variants=True)
                if pixels is None:
                    raise FileNotFoundError(entity.texture)
                image = Image.fromarray(pixels).resize((pixels.shape[1]*4, pixels.shape[0]*4), Image.Resampling.NEAREST)
                board.alpha_composite(image, (230 + (tone*5+hair)*cell_width, y + 12))
        records.append({'body': body, 'activity': activity, 'frame': ident, 'scale': 4})
    board.convert('RGB').save(OUTPUT / 'villagers_natural.png', optimize=True)
    return records


def main(raw: Path, long_raw: Path, reference: Path) -> None:
    """Build all requested plates after the capture owner releases complete evidence."""
    OUTPUT.mkdir(parents=True, exist_ok=True)
    evidence = {
        'before_after_sources': matched_before_after(raw),
        'season_sources': seasons(long_raw),
        'natural_sheet': natural_sheet(),
        'colour': asdict(colour_comparison(raw, OUTPUT, reference, ROOT)),
        'colour_method': 'Same P4 framebuffer; exact P3 RGBA matches and exact visible P4 shader texels only. Missing mappings stay unchanged, not guessed.',
        'reference_commit': P3_COMMIT,
        'reference_sha256': reference_hashes(reference),
        'animations': [animation(raw, OUTPUT, 'strip.json', 'village_noon.gif'),
                       animation(raw, OUTPUT, 'evening.json', 'village_evening.gif')],
    }
    names = ('before_after.png', 'colour_before_after.png', 'season_strip.png',
             'village_noon.gif', 'village_evening.gif', 'villagers_natural.png')
    evidence['outputs'] = [{
        'file': name, 'bytes': (OUTPUT / name).stat().st_size,
        'sha256': hashlib.sha256((OUTPUT / name).read_bytes()).hexdigest(),
    } for name in names]
    framing = []
    for stem in ('a_d2_1200_F2', 'a_d2_2300_F2N', 'a_d2_1200_F1'):
        old = json.loads(gzip.decompress((ROOT / 'docs/art/production3/shots' / (stem + '.entities.json.gz')).read_bytes()))
        new = json.loads((raw / (stem + '.entities.json')).read_text())
        framing.append({'stem': stem, 'p3_camera': old['camera_position'], 'p4_camera': new['camera_position'], 'p3_zoom': old['zoom'], 'p4_zoom': new['zoom'], 'equal': old['camera_position'] == new['camera_position'] and old['zoom'] == new['zoom']})
    evidence['matched_framings'] = framing
    (OUTPUT / 'presentation_artifacts.json').write_text(json.dumps(evidence, indent=2) + '\n')
    summary = {'animations': evidence['animations'], 'outputs': evidence['outputs'], 'matched_framings': framing, 'colour': evidence['colour'], 'budget_bytes': {str(folder.relative_to(ROOT)): sum(path.stat().st_size for path in folder.rglob('*') if path.is_file()) for folder in (ROOT / 'assets/art', OUTPUT)}, 'assets_limit_bytes': 25_000_000, 'docs_limit_bytes': 40_000_000}
    (OUTPUT / 'report_artifacts.json').write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps(evidence['outputs'], indent=2))


if __name__ == '__main__':
    main(*(Path(argument) for argument in sys.argv[1:]))
