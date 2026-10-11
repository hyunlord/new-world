# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# Run: python3 tools/art_pipeline/presentation_p5.py RAW_ART RAW_LONG
"""Publish compact, provenance-labelled Art Pass 5 presentation artifacts."""
from __future__ import annotations

from dataclasses import replace
import gzip
import hashlib
import json
from pathlib import Path
import sys
from typing import Final, TypedDict

from PIL import Image, ImageDraw, ImageFont

from ingame_qa_types import Entity, sprite_pixels
from report_artifacts import animation
from natural_boards_p4 import hair_visuals

ROOT: Final = Path(__file__).resolve().parents[2]
OUTPUT: Final = ROOT / 'docs/art/production5'
BODIES: Final = ('adult_a', 'adult_b', 'child')


class CaptureMetadata(TypedDict):
    camera_position: list[float]
    zoom: list[float]


class Framing(TypedDict):
    stem: str
    p4_camera: list[float]
    p5_camera: list[float]
    p4_zoom: list[float]
    p5_zoom: list[float]
    equal: bool


class ColdEvidence(TypedDict):
    source: str
    scope: str
    source_sha256: str



def matched_before_after(raw: Path) -> list[str]:
    """Compare identical named capture framings, including noon, night and overview."""
    stems = ('a_d2_1200_F2', 'a_d2_2300_F2N', 'a_d2_1200_F1')
    board = Image.new('RGB', (1280, 1110), '#191c1c')
    draw = ImageDraw.Draw(board)
    sources = []
    for row, stem in enumerate(stems):
        for column, directory in enumerate((ROOT / 'docs/art/production4/shots', raw)):
            paths = [directory / (stem + suffix) for suffix in ('.png', '.jpg')]
            path = next(path for path in paths if path.is_file())
            sources.append(str(path))
            with Image.open(path) as image:
                panel = image.convert('RGB').resize((640, 360), Image.Resampling.NEAREST)
            board.paste(panel, (column * 640, row * 370 + 10))
            draw.text((column * 640 + 8, row * 370), f'P{4+column} {stem}', fill='white')
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
    board = Image.new('RGBA', (230 + 28 * cell_width, 32 + row_height * len(rows)), '#555a50')
    draw = ImageDraw.Draw(board)
    font = ImageFont.truetype(str(ROOT / 'assets/fonts/galmuri/Galmuri9.ttf'), 10)
    records = []
    for column in range(28):
        draw.text((230 + column * cell_width, 8), f'T{column//7} H{column%7}', font=font, fill='white')
    for index, (body, activity) in enumerate(rows):
        ident, entry = selected[(body, activity)]
        y = 32 + index * row_height
        draw.text((8, y + 8), f'{body} / {activity}\n{ident}\n{entry.get("qa_status", "")}\n{"wood fallback (no real frame)" if activity.endswith("_wood") and entry["activity"] != activity else entry.get("fallback_kind", "")}', font=font, fill='white')
        entity = Entity('agent', 0, (0,0,0,0), True, ident, ident, (0,0), body,
                        (0,0), (0,0), activity, entry.get('view',''), '', -1, False,
                        entry['page'], tuple(entry['rect']), '')
        for tone in range(4):
            for hair in (range(7) if body != 'child' else (0,)):
                pixels = sprite_pixels(replace(entity, variant=(tone, hair)), ROOT, variants=True)
                if pixels is None:
                    raise FileNotFoundError(entity.texture)
                image = Image.fromarray(pixels).resize((pixels.shape[1]*4, pixels.shape[0]*4), Image.Resampling.NEAREST)
                board.alpha_composite(image, (230 + (tone*7+hair)*cell_width, y + 12))
        records.append({'body': body, 'activity': activity, 'frame': ident, 'scale': 4})
    board.convert('RGB').save(OUTPUT / 'villagers_natural.png', optimize=True)
    return records


def metadata(path: Path) -> CaptureMetadata:
    """Read actual capture metadata, retaining compressed historical sidecars."""
    plain = path.with_suffix('.entities.json')
    if plain.is_file():
        return json.loads(plain.read_text())
    return json.loads(gzip.decompress(plain.with_suffix('.json.gz').read_bytes()))


def framing(raw: Path) -> list[Framing]:
    """Report camera equality independently of visually paired image names."""
    rows = []
    for stem in ('a_d2_1200_F2', 'a_d2_2300_F2N', 'a_d2_1200_F1'):
        old = metadata(ROOT / 'docs/art/production4/shots' / (stem + '.jpg'))
        new = metadata(raw / (stem + '.png'))
        rows.append({'stem': stem, 'p4_camera': old['camera_position'],
                     'p5_camera': new['camera_position'], 'p4_zoom': old['zoom'],
                     'p5_zoom': new['zoom'], 'equal': all(old[key] == new[key]
                     for key in ('camera_position', 'zoom'))})
    return rows


def harvest_strip() -> list[str]:
    """Render every final installed P1/P3/P4 group from the runtime atlas."""
    manifest = OUTPUT / 'harvest/installed_groups_final.json'
    groups = json.loads(manifest.read_text())['groups']
    atlas_path = ROOT / 'assets/art/atlas/world_atlas.json'
    atlas = json.loads(atlas_path.read_text())['sprites']
    if not groups:
        raise FileNotFoundError('No final installed groups')
    board = Image.new('RGBA', (1280, len(groups) * 176 + 32), '#555a50')
    draw = ImageDraw.Draw(board)
    draw.text((8, 8), 'FINAL INSTALLED groups at 2x: runtime atlas pixels; copied frame numbers explicitly labelled', fill='white')
    sources = [str(manifest), str(atlas_path)]
    for index, group in enumerate(groups):
        body, view, activity = group['group']
        frames = sorted(((ident, entry) for ident, entry in atlas.items()
            if (entry.get('body'), entry.get('view'), entry.get('activity')) == (body, view, activity)
            and entry.get('drawable', True)), key=lambda pair: pair[1]['frame'])
        if not frames:
            raise FileNotFoundError(f'Final group absent in atlas: {group["group"]}')
        y = 32 + index * 176
        draw.text((8, y + 8), f'{body}/{view}/{activity}\n{group.get("installation", "")}\nsource {group["source"]}', fill='white')
        for column, (ident, entry) in enumerate(frames):
            page = ROOT / entry['page'].removeprefix('res://')
            x, top, width, height = entry['rect']
            with Image.open(page) as image:
                frame = image.convert('RGBA').crop((x, top, x + width, top + height))
                frame = frame.resize((width * 2, height * 2), Image.Resampling.NEAREST)
            copied = entry['frame'] in group['copied_frames']
            status = 'COPY' if copied else ('REAL' if entry.get('qa_status') == 'accepted' else str(entry.get('qa_status', 'unknown')).upper())
            draw.text((260 + column * 240, y + 4), f'{ident} / {status}', fill='white')
            board.alpha_composite(frame, (260 + column * 240, y + 28))
            sources.append(str(page))
    board.convert('RGB').save(OUTPUT / 'harvest_strip.png', optimize=True)
    return list(dict.fromkeys(sources))


def hair_masks_before_after() -> list[str]:
    """Compare the 29 frozen audit inputs to masks currently drawn by the atlas."""
    document = json.loads((OUTPUT / 'masks/candidates.json').read_text())
    rows = [row for row in document['candidates'] if row.get('mask_scope') == 'c']
    atlas_path = ROOT / 'assets/art/atlas/world_atlas.json'
    atlas = json.loads(atlas_path.read_text())['sprites']
    if len(rows) != 29:
        raise FileNotFoundError(f'Expected 29 audited old/new mask pairs; found {len(rows)}')
    board = Image.new('RGB', (1060, len(rows) * 180 + 40), '#555a50')
    draw = ImageDraw.Draw(board)
    draw.text((8, 8), 'Audit masks: OLD overlay / OLD white / FINAL INSTALLED overlay / FINAL INSTALLED white', fill='white')
    sources = [str(atlas_path)]
    for index, row in enumerate(rows):
        slot = row['id'].split(':')[0]
        old_path = OUTPUT / 'masks/inputs' / slot / 'original_mask.png'
        sprite_path = ROOT / row['images']['sprite']
        entry = atlas[slot]
        page = ROOT / entry['page'].removeprefix('res://')
        region_page = page.with_name(page.stem + '_regions.png')
        x, top, width, height = entry['rect']
        rx, ry, rw, rh = entry.get('region_rect', entry['rect'])
        with Image.open(sprite_path) as source, Image.open(old_path) as old_mask, Image.open(page) as final_page, Image.open(region_page) as regions:
            original = source.convert('RGBA')
            old = old_mask.convert('RGBA')
            installed = final_page.convert('RGBA').crop((x, top, x + width, top + height))
            actual_mask = regions.convert('RGBA').crop((rx, ry, rx + rw, ry + rh))
            unchanged = old.size == actual_mask.size and old.tobytes() == actual_mask.tobytes()
            visuals = (*hair_visuals(original, old), *hair_visuals(installed, actual_mask))
            y = index * 180 + 40
            draw.text((8, y + 4), f'{slot}\n{"UNCHANGED mask" if unchanged else "CHANGED mask"}\n{entry.get("qa_status", "")}', fill='white')
            for column, image in enumerate(visuals):
                image = image.resize((image.width * 4, image.height * 4), Image.Resampling.NEAREST)
                board.paste(image, (260 + column * 190, y + 24), image)
        sources.extend(str(path) for path in (sprite_path, old_path, page, region_page))
    board.save(OUTPUT / 'hair_masks_before_after.png', optimize=True)
    return list(dict.fromkeys(sources))


def cold_night(raw: Path, long_raw: Path) -> ColdEvidence:
    """Prefer observed cold-peak capture; record absent visual evidence honestly."""
    paths = sorted(set(raw.glob('*.cold_peak.png')) | set(long_raw.glob('*.cold_peak.png')))
    scope = 'gameplay cold peak'
    if not paths:
        paths = sorted((OUTPUT / 'cold').glob('*scene*.png'))
        scope = 'controlled scene test; not natural gameplay'
    if not paths:
        raise FileNotFoundError('No cold-peak screenshot or controlled cold scene screenshot')
    source = paths[0]
    with Image.open(source) as image:
        image.save(OUTPUT / 'cold_night.png', optimize=True)
    return {'source': str(source), 'scope': scope,
            'source_sha256': hashlib.sha256(source.read_bytes()).hexdigest()}


def main(raw: Path, long_raw: Path) -> None:
    """Publish available real evidence and list every missing required artifact."""
    OUTPUT.mkdir(parents=True, exist_ok=True)
    evidence = {'missing': {}, 'animations': []}
    jobs = {
        'before_after': lambda: matched_before_after(raw),
        'matched_framings': lambda: framing(raw),
        'seasons': lambda: seasons(long_raw),
        'natural_sheet': natural_sheet,
        'harvest': harvest_strip,
        'hair_masks': hair_masks_before_after,
        'cold': lambda: cold_night(raw, long_raw),
    }
    for name, job in jobs.items():
        try:
            evidence[name] = job()
        except (FileNotFoundError, StopIteration) as error:
            evidence['missing'][name] = str(error) or 'Required source image is absent'
    mask_board = OUTPUT / 'hair_masks_before_after.png'
    if not mask_board.is_file():
        evidence['missing']['hair_masks_before_after'] = 'Awaiting mask owner before/after board'
    for manifest, filename in (('strip.json', 'village_noon.gif'), ('evening.json', 'village_evening.gif')):
        try:
            evidence['animations'].append(animation(raw, OUTPUT, manifest, filename))
        except (FileNotFoundError, RuntimeError) as error:
            evidence['missing'][filename] = str(error)
    names = ('before_after.png', 'season_strip.png', 'villagers_natural.png',
             'harvest_strip.png', 'hair_masks_before_after.png', 'cold_night.png',
             'village_noon.gif', 'village_evening.gif')
    evidence['outputs'] = [{'file': name, 'bytes': (OUTPUT / name).stat().st_size,
        'sha256': hashlib.sha256((OUTPUT / name).read_bytes()).hexdigest()}
        for name in names if (OUTPUT / name).is_file()]
    evidence['budget_bytes'] = {str(folder.relative_to(ROOT)): sum(path.stat().st_size
        for path in folder.rglob('*') if path.is_file()) for folder in (ROOT / 'assets/art', OUTPUT)}
    evidence['assets_limit_bytes'] = 25_000_000
    evidence['docs_limit_bytes'] = 40_000_000
    evidence['budget_pass'] = evidence['budget_bytes']['assets/art'] <= 25_000_000 and evidence['budget_bytes']['docs/art/production5'] <= 40_000_000
    text = json.dumps(evidence, indent=2) + '\n'
    (OUTPUT / 'presentation_artifacts.json').write_text(text)
    (OUTPUT / 'report_artifacts.json').write_text(text)
    print(text)


if __name__ == '__main__':
    main(*(Path(argument) for argument in sys.argv[1:]))
