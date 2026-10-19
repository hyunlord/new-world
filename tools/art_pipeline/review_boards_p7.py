# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# Run: python3 tools/art_pipeline/review_boards_p7.py RAW_ART RAW_LONG RAW_GOD
"""Number observed objects without disclosing expected states to screen reviewers."""
from __future__ import annotations
import hashlib
import json
import os
from pathlib import Path
import sys
from PIL import Image, ImageDraw


def cycle_contacts(cycle_path: Path, destination: Path):
    """Crop visible shelters through sampled time without exposing their state."""
    cycle = json.loads(cycle_path.read_text())
    stem = cycle_path.name.removesuffix('.cold_cycle.json')
    original = cycle_path.parent / (stem + '.png')
    data = json.loads(original.with_suffix('.entities.json').read_text())
    samples = cycle.get('samples', [])
    if not samples:
        raise ValueError(f'No samples in {cycle_path}')
    start = samples[0]['real_time_usec']
    selected = []
    for target in range(0, 4801, 400):
        index = min(range(len(samples)), key=lambda index: abs(samples[index]['real_time_usec'] - start - target * 1000))
        if index not in selected:
            selected.append(index)
    with Image.open(original) as image:
        original_size = image.size
    images = []
    records = []
    for index in selected:
        sample = samples[index]
        path = cycle_path.parent / sample['file']
        with Image.open(path) as image:
            images.append(image.convert('RGB'))
        records.append({'file': str(path), 'sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
                        'relative_seconds': (sample['real_time_usec'] - start) / 1e6})
    scale_x, scale_y = images[0].width / original_size[0], images[0].height / original_size[1]
    shelters = [row for row in data['entities'] if row['kind'] == 'building' and row.get('building_kind') == 1 and row.get('visible')]
    boxes = []
    for row in shelters:
        x, y, w, h = row['rect']
        boxes.append((round(x * scale_x) - 12, round(y * scale_y) - 16,
                      round((x + w) * scale_x) + 12, round((y + h) * scale_y) + 16))
    width = max((right - left for left, top, right, bottom in boxes), default=120)
    height = max((bottom - top for left, top, right, bottom in boxes), default=80)
    contact = Image.new('RGB', (100 + len(images) * (width + 8), 54 + max(1, len(shelters)) * (height + 12)), '#191c1c')
    draw = ImageDraw.Draw(contact)
    draw.text((8, 6), stem + ' | actual sampled pixels / shelter IDs only', fill='white')
    for column, record in enumerate(records):
        draw.text((100 + column * (width + 8), 30), f'{record["relative_seconds"]:.3f}s', fill='white')
    for row_index, (shelter, box) in enumerate(zip(shelters, boxes, strict=True)):
        y = 54 + row_index * (height + 12)
        draw.text((8, y + 8), 'B' + str(shelter['id']), fill='white')
        for column, image in enumerate(images):
            contact.paste(image.crop(box), (100 + column * (width + 8), y))
    destination.mkdir(parents=True, exist_ok=True)
    target = destination / (stem + '.cold_cycle_contact.png')
    contact.save(target, optimize=True)
    return {'stem': stem, 'source': str(original), 'sha256': hashlib.sha256(original.read_bytes()).hexdigest(),
            'cycle_source': str(cycle_path), 'cycle_sha256': hashlib.sha256(cycle_path.read_bytes()).hexdigest(),
            'contact': str(target), 'contact_sha256': hashlib.sha256(target.read_bytes()).hexdigest(),
            'shelter_ids': [row['id'] for row in shelters], 'samples': records}


def main(directories: list[Path]) -> None:
    """Bind each labelled board to the unmodified capture's exact hash."""
    output = Path('docs/art/production7/review/boards')
    output.mkdir(parents=True, exist_ok=True)
    identity_output = Path(os.environ.get('CAPTURE_EVIDENCE', '/Users/rexxa/work/art-production7-evidence')) / 'review/identity'
    identity_output.mkdir(parents=True, exist_ok=True)
    mapping = {}
    hud_rows = []
    for directory in directories:
        manifest = json.loads((directory / 'manifest.json').read_text())
        for entry in manifest['captures']:
            path = directory / entry['file']
            if path.stem.startswith(('a_calib', 'a_lineup')):
                continue
            data = json.loads(path.with_suffix('.entities.json').read_text())
            with Image.open(path) as image:
                board = image.convert('RGB')
            hud_labels = [row for row in data.get('labels', []) if 'HudTopbar' in row.get('name', '')]
            hud_bottom = max((row['rect'][1] + row['rect'][3] for row in hud_labels), default=160)
            hud_rows.append((path.stem, hashlib.sha256(path.read_bytes()).hexdigest(), board.crop((0, 0, board.width, min(board.height, round(hud_bottom) + 8)))))
            draw = ImageDraw.Draw(board)
            objects = []
            for row in data['entities']:
                if not row.get('visible') or row['kind'] not in ('agent', 'building'):
                    continue
                x, y, w, h = row['rect']
                label = ('A' if row['kind'] == 'agent' else 'B') + str(row['id'])
                draw.rectangle((x, y, x + w, y + h), outline='#FFFF00', width=1)
                draw.text((x, max(0, y - 12)), label, fill='#FFFF00', stroke_width=1, stroke_fill='black')
                objects.append({'label': label, 'id': row['id'], 'kind': row['kind']})
            target = identity_output / (path.stem + '.jpg')
            board.save(target, quality=90)
            mapping[path.stem] = {'source': str(path), 'sha256': hashlib.sha256(path.read_bytes()).hexdigest(), 'board': str(target), 'objects': objects}
    hud_index = []
    for offset in range(0, len(hud_rows), 4):
        batch = hud_rows[offset:offset + 4]
        contact = Image.new('RGB', (max(row[2].width for row in batch), sum(row[2].height + 36 for row in batch)), '#191c1c')
        draw = ImageDraw.Draw(contact)
        y = 0
        for stem, digest, crop in batch:
            draw.text((8, y + 3), stem + ' | ORIGINAL SHA256 ' + digest, fill='white')
            contact.paste(crop, (0, y + 30))
            y += crop.height + 36
        target = output / f'hud_batch_{offset // 4 + 1:03d}.png'
        contact.save(target, optimize=True)
        hud_index.append({'board': str(target), 'frames': [{'stem': row[0], 'sha256': row[1]} for row in batch]})
    cycle_output = identity_output.parent / 'cycles'
    cycle_index = [cycle_contacts(path, cycle_output) for directory in directories for path in sorted(directory.glob('*.cold_cycle.json'))]
    (output.parent / 'cycle_contacts.json').write_text(json.dumps(cycle_index, indent=2) + '\n')
    (output.parent / 'hud_batches.json').write_text(json.dumps(hud_index, indent=2) + '\n')
    (output.parent / 'board_sources.json').write_text(json.dumps(mapping, indent=2) + '\n')
    print(f'{len(mapping)} identity-only boards written to {output}')


if __name__ == '__main__':
    main([Path(argument) for argument in sys.argv[1:]])
