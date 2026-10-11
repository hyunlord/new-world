# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/key_references_p5.py REFERENCES_JSON [REGISTRY_JSON]
"""Build review-pending key references only from approved exact installed masks."""
from __future__ import annotations

from collections import Counter
import hashlib
import json
from pathlib import Path
import sys

import numpy as np
from PIL import Image, ImageDraw

from key_hair_p5 import KEYS
from key_plan_p5 import ROOT, OUTPUT
from palette_tools import rgb_to_lab


def prepare(pool_path: Path, registry_path: Path) -> Path:
    """Freeze idle identity and top-three approved hair browns; no review implied."""
    pool = json.loads(pool_path.read_text())
    registry = json.loads(registry_path.read_text())['frames']
    atlas = json.loads((ROOT / 'assets/art/atlas/world_atlas.json').read_text())
    directory = OUTPUT / 'references'
    directory.mkdir(parents=True, exist_ok=True)
    bodies = {}
    for body, slot in [('adult_a', 'V_a1_01'), ('adult_b', 'V_b1_01'), ('child', 'V_c1_01')]:
        entry = atlas['sprites'][slot]
        page = entry['page']
        if isinstance(page, int):
            page = atlas['pages'][page]
        x, y, w, h = entry['rect']
        with Image.open(ROOT / page.removeprefix('res://')) as opened:
            natural = opened.convert('RGBA').crop((x, y, x + w, y + h))
        natural_path = directory / f'{body}_idle_natural.png'
        natural.save(natural_path)
        natural.resize((w * 8, h * 8), Image.Resampling.NEAREST).save(directory / f'{body}_idle_natural_8x.png')
        output = {'slot': slot, 'natural_path': str(natural_path.relative_to(ROOT)),
                  'natural_rgba_sha256': hashlib.sha256(natural.tobytes()).hexdigest(), 'key_approved': False}
        bodies[body] = output
        if body == 'child':
            continue
        frozen = registry.get(slot)
        if frozen is None or frozen['verdict'] != 'ok' or frozen['rgba_sha256'] != output['natural_rgba_sha256']:
            output['key_unavailable_reason'] = 'installed_idle_mask_not_approved_or_not_same_rgba'
            continue
        if body == 'adult_b' and frozen['source'] != 'template_v5':
            output['key_unavailable_reason'] = 'adult_b_requires_part3_new_mask_approval'
            continue
        mask_path = ROOT / frozen['mask_path']
        if hashlib.sha256(mask_path.read_bytes()).hexdigest() != frozen['mask_sha256']:
            message = 'Approved mask hash mismatch'
            raise ValueError(message)
        with Image.open(mask_path) as opened:
            mask = np.asarray(opened.convert('RGBA'))
        pixels = np.array(natural)
        hair = mask[..., 0] == 3
        for shade, key in enumerate(KEYS):
            pixels[hair & (mask[..., 1] == shade), :3] = key
        key_image = Image.fromarray(pixels)
        key_path = directory / f'{body}_idle_key_8x.png'
        key_image.resize((w * 8, h * 8), Image.Resampling.NEAREST).save(key_path)
        board = Image.new('RGB', (720, max(320, h * 8 + 70)), (161, 158, 152))
        draw = ImageDraw.Draw(board)
        overlay = np.array(natural); overlay[hair, :3] = (70, 140, 255)
        for column, (label, image) in enumerate([('Natural idle 8x', natural), ('Approved mask 8x', Image.fromarray(overlay)), ('Key reference 8x', key_image)]):
            draw.text((column * 240 + 10, 10), label, fill='black')
            enlarged = image.resize((w * 8, h * 8), Image.Resampling.NEAREST)
            board.paste(enlarged, (column * 240 + 10, 45), enlarged)
        board_path = directory / f'{body}_key_reference_review.png'
        board.save(board_path)
        output['review_board'] = str(board_path.relative_to(ROOT))
        output['non_hair_pixels_unchanged'] = bool(np.array_equal(np.asarray(natural)[~hair], pixels[~hair]))

        counts: Counter[tuple[int, int, int]] = Counter()
        references = pool.get('references', pool.get('rows', []))
        for row in references:
            if row['body'] != body:
                continue
            with Image.open(ROOT / row['sprite_path']) as sprite, Image.open(ROOT / row['mask_path']) as source:
                rgba, regions = np.asarray(sprite.convert('RGBA')), np.asarray(source.convert('RGBA'))
            counts.update(tuple(int(c) for c in pixel) for pixel in rgba[(regions[..., 0] == 3) & (rgba[..., 3] > 0), :3])
        counts.pop((3, 0, 0), None)
        frequent = [color for color, _ in sorted(counts.items(), key=lambda pair: (-pair[1], pair[0]))[:3]]
        if len(frequent) != 3:
            message = f'Need three approved hair colours for {body}'
            raise ValueError(message)
        order = np.argsort(-rgb_to_lab(np.asarray(frequent, dtype=np.uint8))[:, 0])
        output.update(key_path=str(key_path.relative_to(ROOT)), key_sha256=hashlib.sha256(key_path.read_bytes()).hexdigest(),
                      mask_path=frozen['mask_path'], mask_sha256=frozen['mask_sha256'],
                      browns=[list(frequent[int(i)]) for i in order], approval='pending_fresh_reference_review')
    path = directory / 'manifest.json'
    path.write_text(json.dumps({'schema_version': 1, 'pool_path': str(pool_path),
                               'pool_sha256': hashlib.sha256(pool_path.read_bytes()).hexdigest(), 'bodies': bodies}, indent=2) + '\n')
    return path


def approve(manifest_path: Path, verdict_path: Path) -> Path:
    """Bind a fresh observer verdict to both exact key PNG and natural RGBA."""
    manifest = json.loads(manifest_path.read_text())
    review = json.loads(verdict_path.read_text())
    for row in review['verdicts']:
        entry = manifest['bodies'][row['body']]
        if row['key_sha256'] != entry['key_sha256'] or row['natural_rgba_sha256'] != entry['natural_rgba_sha256']:
            message = 'Key reference review fingerprint mismatch'
            raise ValueError(message)
        key_path = ROOT / entry['key_path']
        if hashlib.sha256(key_path.read_bytes()).hexdigest() != row['key_sha256']:
            message = 'Reviewed key reference PNG changed'
            raise ValueError(message)
        entry['key_approved'] = row['verdict'] == 'ok'
        entry['approval'] = row['verdict']
        entry['review_path'] = str(verdict_path)
        entry['review_sha256'] = hashlib.sha256(verdict_path.read_bytes()).hexdigest()
    output = manifest_path.with_name('approved_manifest.json')
    if output.exists():
        message = 'Refusing to overwrite bound reference approval'
        raise ValueError(message)
    output.write_text(json.dumps(manifest, indent=2) + '\n')
    return output


if __name__ == '__main__':
    if sys.argv[1] == 'approve':
        print(approve(Path(sys.argv[2]), Path(sys.argv[3])))
    else:
        registry = Path(sys.argv[2]) if len(sys.argv) > 2 else ROOT / 'tools/art_pipeline/reviewed_hair_maps_p4.json'
        print(prepare(Path(sys.argv[1]), registry))
