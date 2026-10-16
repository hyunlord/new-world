# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# Run through presentation_p4.py with the frozen P3 reference directory.
"""Recolour exact surviving P3 sprites in a real P4 camp without inventing maps."""
from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass, replace
from pathlib import Path
from typing import Final

import numpy as np
from PIL import Image, ImageDraw

from ingame_qa_types import Array, Entity, box, load_capture, placed, sprite_pixels

P3_COMMIT: Final = 'ece4e85e5e0b48945dddef61ba601a144db1397a'


@dataclass(frozen=True, slots=True)
class ColourEvidence:
    source: str
    mapped_ids: tuple[int, ...]
    missing_ids: tuple[int, ...]
    replaced_pixels: int


def old_pixels(entity: Entity, reference: Path, repo: Path) -> Array | None:
    """Use the frozen P3 UVs only when the underlying RGBA is still identical."""
    atlas = json.loads((reference / 'assets/art/atlas/world_atlas.json').read_text())['sprites']
    entry = atlas.get(entity.frame or entity.sprite)
    if entry is None:
        return None
    old = replace(entity, texture=entry['page'], region=tuple(entry['rect']))
    pixels = sprite_pixels(old, reference)
    current = sprite_pixels(entity, repo)
    if pixels is None or current is None or not np.array_equal(pixels, current):
        return None
    page = reference / entry['page'].removeprefix('res://')
    x, y, w, h = old.region
    with Image.open(page.with_name(page.stem + '_regions.png')) as image:
        region = np.array(image.convert('RGBA').crop((x, y, x + w, y + h)))
    if entity.flip:
        region = region[:, ::-1]
    with Image.open(reference / 'assets/art/villager_ramps.png') as image:
        palette = np.asarray(image.convert('RGB'))
    hashed = (entity.ident ^ 0x9E3779B9) & 0xFFFFFFFF
    for _ in range(2):
        hashed = ((hashed ^ (hashed >> 16)) * 0x45D9F3B) & 0xFFFFFFFF
    hashed ^= hashed >> 16
    skin, hair, clothing = entity.legacy_variant or ((hashed >> 9) % 4, entity.variant[1], (hashed >> 15) % 6)
    shade = np.clip(region[..., 1], 0, 2)
    for code, row in ((2, min(skin, 3)), (3, 4 + min(hair, 4)), (4, 9 + min(clothing, 5))):
        mask = region[..., 0] == code
        pixels[mask, :3] = palette[row, shade[mask]]
    return pixels


def colour_comparison(raw: Path, output: Path, reference: Path, repo: Path) -> ColourEvidence:
    """Keep framebuffer, geometry, occlusion and lighting fixed; replace known texels only."""
    capture = load_capture(raw / 'a_d2_1200_F2.png')
    natural = capture.image()
    recoloured = natural.copy()
    mapped, missing = [], []
    replaced = 0
    owners = np.full((1080, 1920), -1, dtype=np.int32)
    ordered = sorted(enumerate(capture.entities), key=lambda pair: (pair[1].draw_order[0], pair[1].world_anchor[1], pair[1].draw_order[1]))
    for index, drawn in ordered:
        if drawn.visible and drawn.alpha > 0 and 'shadow' not in drawn.kind and drawn.kind != 'panel':
            _, opaque = placed(drawn, capture, repo)
            owners[opaque] = index
    for panel in capture.entities:
        if panel.kind == 'panel' and panel.visible:
            x0, y0, x1, y1 = box(panel, capture.scale)
            owners[max(0,y0):min(1080,y1), max(0,x0):min(1920,x1)] = -1
    for entity_index, entity in enumerate(capture.entities):
        if entity.kind != 'agent' or not entity.visible:
            continue
        pixels = old_pixels(entity, reference, repo)
        if pixels is None:
            missing.append(entity.ident)
            continue
        expected, mask = placed(entity, capture, repo, variants=True)
        # Only exact displayed texels are changed: front occluders and lit/blended pixels stay real.
        mask &= owners == entity_index
        mask &= np.max(np.abs(natural.astype(int) - expected.astype(int)), axis=2) <= 1
        x0, y0, x1, y1 = box(entity, capture.scale)
        scaled = np.asarray(Image.fromarray(pixels).resize((x1-x0, y1-y0), Image.Resampling.NEAREST))
        left, top, right, bottom = max(0, x0), max(0, y0), min(1920, x1), min(1080, y1)
        if right <= left or bottom <= top:
            continue
        visible = mask[top:bottom, left:right]
        recoloured[top:bottom, left:right][visible] = scaled[top-y0:bottom-y0, left-x0:right-x0, :3][visible]
        replaced += int(visible.sum())
        mapped.append(entity.ident)
    board = Image.new('RGB', (1280, 410), '#191c1c')
    draw = ImageDraw.Draw(board)
    for index, (pixels, label) in enumerate(((recoloured, 'P3 ramp emulation on same P4 camp'), (natural, 'P4 natural colours - actual capture'))):
        panel = Image.fromarray(pixels).crop((320, 160, 1600, 900)).resize((640, 370), Image.Resampling.NEAREST)
        board.paste(panel, (index * 640, 40))
        draw.text((index * 640 + 8, 8), label, fill='white')
    draw.text((8, 23), f'Exact P3 maps: {len(mapped)}/{len(mapped)+len(missing)} actors; missing IDs {missing} unchanged', fill='white')
    board.save(output / 'colour_before_after.png', optimize=True)
    return ColourEvidence(str(capture.path), tuple(mapped), tuple(missing), replaced)


def reference_hashes(reference: Path) -> dict[str, str]:
    """Fingerprint every extracted immutable P3 input for the artifact ledger."""
    return {str(path.relative_to(reference)): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in reference.rglob('*') if path.is_file()}
