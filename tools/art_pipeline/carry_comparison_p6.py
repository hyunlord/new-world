# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# Run: imported by presentation_p6.py after captures.
"""Exhaustive staged native comparisons from actual installed texture pixels."""
from __future__ import annotations
import json
from pathlib import Path
from PIL import Image, ImageDraw
from carry_boards_p6 import ROOT, installed_frames


def exhaustive_carry(destination: Path) -> list[str]:
    """Every carry frame has source, installed composite, and nearest eat reference."""
    frames = installed_frames()
    carries = [frame for frame in frames if frame.activity in ('carry', 'carry_wood')]
    eats = [frame for frame in frames if frame.activity == 'eat']
    binding_path = ROOT / 'assets/art/atlas/carry_p6.json'
    binding = json.loads(binding_path.read_text())
    rows = []
    pictures = []
    for frame in carries:
        original = frame.image()
        installed = original.copy()
        entry = binding['sprites'].get(frame.sprite_id)
        if entry:
            with Image.open(ROOT / binding['body_page'].removeprefix('res://')) as page:
                body = page.convert('RGBA').crop(frame.box)
            with Image.open(ROOT / binding['prop_page'].removeprefix('res://')) as page:
                x, y, w, h = entry['rect']
                prop = page.convert('RGBA').crop((x, y, x + w, y + h))
            installed = Image.new('RGBA', (body.width + 9, body.height))
            installed.alpha_composite(body)
            installed.alpha_composite(prop, tuple(entry['anchor']))
        candidates = [eat for eat in eats if eat.body == frame.body]
        reference = min(candidates, key=lambda eat: (eat.view != frame.view, eat.sprite_id)) if candidates else None
        rows.append({'frame': frame.sprite_id, 'group': f'{frame.body}_{frame.activity}_{frame.view}',
                     'installed_layer': bool(entry), 'eat_reference': reference.sprite_id if reference else None,
                     'scope': 'STAGED NATIVE TEXTURE COMPARISON; not a gameplay screenshot'})
        pictures.append((frame, original, installed, reference.image() if reference else None))
    height = max((max(a.height, b.height, c.height if c else 0) * 2 + 30 for _, a, b, c in pictures), default=120)
    board = Image.new('RGB', (1000, 70 + height * len(pictures)), '#73816e')
    draw = ImageDraw.Draw(board)
    draw.text((8, 8), 'STAGED NATIVE TEXTURES AT 2x | all carry groups/frames | NOT gameplay evidence', fill='white')
    draw.text((260, 30), 'BEFORE: source pixels', fill='white')
    draw.text((500, 30), 'AFTER: installed composition', fill='white')
    draw.text((780, 30), 'EAT reference', fill='white')
    for index, (frame, original, installed, eat) in enumerate(pictures):
        y = 70 + index * height
        draw.text((8, y), f'{frame.sprite_id}\n{frame.body}/{frame.activity}/{frame.view}\n' + ('approved attached layer' if rows[index]['installed_layer'] else 'original retained'), fill='white')
        for x, picture in ((290, original), (550, installed), (820, eat)):
            if picture is not None:
                enlarged = picture.resize((picture.width * 2, picture.height * 2), Image.Resampling.NEAREST)
                board.paste(enlarged, (x, y), enlarged)
    board.save(destination, optimize=True)
    destination.with_suffix('.json').write_text(json.dumps({'scope': 'staged-native', 'carry_groups': len({row['group'] for row in rows}), 'frames': rows}, indent=2) + '\n')
    return [str(binding_path), str(ROOT / 'assets/art/atlas/world_atlas.json')]
