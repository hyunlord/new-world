# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
"""Freeze installed P3 hair before repacking; train from exact reviewed RGB partners.

Run from any directory: python3 tools/art_pipeline/hair_inventory_p4.py extract
Then: python3 tools/art_pipeline/hair_inventory_p4.py complete
"""
from __future__ import annotations

import hashlib
import io
import json
from pathlib import Path
import subprocess
import sys
from typing import Final, TypedDict

import numpy as np
from PIL import Image

ROOT: Final = Path(__file__).resolve().parents[2]
REGISTRY: Final = Path('tools/art_pipeline/reviewed_hair_maps_p4.json')
REPORT: Final = Path('docs/art/production4/hair_inventory.json')


class InventoryError(ValueError):
    """Frozen source or RGB/map pairing is inconsistent."""


class Frame(TypedDict):
    mask_path: str
    mask_sha256: str
    rgba_sha256: str
    body: str
    activity: str
    source: str
    verdict: str
    size: list[int]
    original_hair_texels: int
    hair_texels: int


class Registry(TypedDict):
    version: int
    source_atlas_sha256: str
    frames: dict[str, Frame]


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_image(path: Path) -> Image.Image:
    with Image.open(path) as opened:
        return opened.convert('RGBA')


def validate_partner(sprite: Image.Image, regions: Image.Image, expected: str, size: list[int]) -> None:
    """Fail closed on wrong pixels, geometry or alpha, before using reviewed labels."""
    if digest(sprite.tobytes()) != expected or list(sprite.size) != size or regions.size != sprite.size:
        raise InventoryError('Reviewed RGB hash or dimensions mismatch')
    if sprite.getchannel('A').tobytes() != regions.getchannel('A').tobytes():
        raise InventoryError('Reviewed RGB/region alpha mismatch')


def installed_mask(sprite: Image.Image, regions: Image.Image, body: str, activity: str) -> Image.Image:
    """Keep existing R3/G texels, except excluded child, asleep and lying frames."""
    pixels = np.asarray(sprite)
    labels = np.asarray(regions)
    output = np.zeros_like(pixels)
    output[..., 3] = pixels[..., 3]
    bbox = sprite.getchannel('A').getbbox()
    if body == 'child' or activity == 'asleep' or not bbox or bbox[2] - bbox[0] > bbox[3] - bbox[1]:
        return Image.fromarray(output)
    hair = (labels[..., 0] == 3) & (pixels[..., 3] > 0)
    output[..., 0][hair] = 3
    output[..., 1][hair] = labels[..., 1][hair]
    return Image.fromarray(output)


def save_json(path: Path, data: Registry) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2, sort_keys=True) + '\n')


def validate_frozen(root: Path, registry: Registry) -> None:
    """A rerun may reuse frozen extraction but never replace its provenance."""
    for name, row in registry['frames'].items():
        path = root / row['mask_path']
        if digest(path.read_bytes()) != row['mask_sha256'] or list(load_image(path).size) != row['size']:
            raise InventoryError(f'Frozen mask changed: {name}')


def extract(root: Path = ROOT) -> Registry:
    """Freeze all current drawable villagers; refuse already hair-only input pages."""
    if (root / REGISTRY).exists():
        frozen: Registry = json.loads((root / REGISTRY).read_text())
        validate_frozen(root, frozen)
        return frozen
    atlas_path = root / 'assets/art/atlas/world_atlas.json'
    atlas = json.loads(atlas_path.read_text())
    registry: Registry = {'version': 1, 'source_atlas_sha256': digest(atlas_path.read_bytes()), 'frames': {}}
    pages: dict[str, Image.Image] = {}
    regions: dict[str, Image.Image] = {}
    for name, entry in atlas['sprites'].items():
        body = entry.get('body', '')
        if body not in ('adult_a', 'adult_b', 'child') or not entry.get('drawable', True):
            continue
        page = entry['page']
        if isinstance(page, int):
            page = atlas['pages'][page]
        path = root / page.removeprefix('res://')
        if page not in pages:
            pages[page] = load_image(path)
            regions[page] = load_image(path.with_name(path.stem + '_regions.png'))
            if not np.isin(np.asarray(regions[page])[..., 0], [1, 2, 4, 5]).any():
                raise InventoryError('Cannot extract P3 provenance from hair-only region pages')
        x, y, w, h = entry['rect']
        box = (x, y, x + w, y + h)
        sprite = pages[page].crop(box)
        old = regions[page].crop(box)
        mask = installed_mask(sprite, old, body, entry['activity'])
        target = Path('tools/art_pipeline/hair_masks_p4') / f'{name}.png'
        (root / target).parent.mkdir(parents=True, exist_ok=True)
        mask.save(root / target, optimize=True)
        registry['frames'][name] = {'mask_path': str(target), 'mask_sha256': digest((root / target).read_bytes()),
            'rgba_sha256': digest(sprite.tobytes()), 'body': body, 'activity': entry['activity'],
            'source': 'installed_map', 'verdict': 'pending', 'size': [w, h],
            'original_hair_texels': int((np.asarray(old)[..., 0] == 3).sum()),
            'hair_texels': int((np.asarray(mask)[..., 0] == 3).sum())}
    save_json(root / REGISTRY, registry)
    return registry


def reviewed_pairs(root: Path = ROOT) -> list[tuple[str, Image.Image, Image.Image, str]]:
    """Recover each reviewed image once from immutable provenance, including replaced frames."""
    rows = json.loads((root / 'tools/art_pipeline/reviewed_region_maps_p3.json').read_text())['maps']
    candidates = {row['id']: row for row in json.loads((root / 'docs/art/production3/reprocess/candidates.json').read_text())['candidates']}
    old_atlases = {}
    old_pages: dict[tuple[str, str], Image.Image] = {}
    result = []
    seen: set[tuple[str, str]] = set()
    for row in rows:
        source = row['source']
        if source.startswith('prior_approved:'):
            _, revision, name = source.split(':')
            if revision not in old_atlases:
                old_atlases[revision] = json.loads(subprocess.check_output(['git', 'show', f'{revision}:assets/art/atlas/world_atlas.json'], cwd=root))
            atlas = old_atlases[revision]
            frame = atlas['sprites'][name]
            page = frame['page']
            if isinstance(page, int):
                page = atlas['pages'][page]
            key = (revision, page)
            if key not in old_pages:
                content = subprocess.check_output(['git', 'show', f"{revision}:{page.removeprefix('res://')}"], cwd=root)
                with Image.open(io.BytesIO(content)) as opened:
                    old_pages[key] = opened.convert('RGBA')
            x, y, w, h = frame['rect']
            sprite = old_pages[key].crop((x, y, x + w, y + h))
        elif source.startswith('fresh_p3_review:'):
            identity = source.removeprefix('fresh_p3_review:')
            sprite = load_image(root / candidates[identity]['images']['sprite'])
        else:
            raise InventoryError(f'Unsupported reviewed source: {source}')
        path = root / row['regions_path']
        if digest(path.read_bytes()) != row['regions_sha256']:
            raise InventoryError(f'Reviewed map changed: {path}')
        region = load_image(path)
        validate_partner(sprite, region, row['rgba_sha256'], row['size'])
        identity = (row['body'], row['rgba_sha256'])
        if identity in seen:
            raise InventoryError(f'Duplicate reviewed image: {identity}')
        seen.add(identity)
        result.append((row['body'], sprite, region, source))
    return result


def complete(root: Path = ROOT) -> None:
    """Train the full reviewed corpus and repair low-hair installed adults once."""
    from hair_maps import build_palette, classify

    registry = extract(root)
    pairs = reviewed_pairs(root)
    palettes = {body: build_palette([(sprite, region) for b, sprite, region, _ in pairs if b == body], body)
                for body in ('adult_a', 'adult_b', 'child')}
    palette_path = root / 'tools/art_pipeline/hair_palette_p4.json'
    palette_data = {'version': 1, 'bodies': {body: {'seen_rgb': sorted(palette.seen_rgb),
        'hair_rgb': sorted(palette.hair_rgb)} for body, palette in palettes.items()}}
    palette_bytes = (json.dumps(palette_data, indent=2, sort_keys=True) + '\n').encode()
    report_path = root / REPORT
    if report_path.exists():
        report = json.loads(report_path.read_text())
        if report.get('completed'):
            if report['palette_sha256'] != digest(palette_bytes) or not palette_path.is_file() or digest(palette_path.read_bytes()) != digest(palette_bytes):
                raise InventoryError('Frozen training palette changed')
            print('Validated frozen completed hair inventory')
            return
    palette_path.write_bytes(palette_bytes)
    atlas_path = root / 'assets/art/atlas/world_atlas.json'
    if digest(atlas_path.read_bytes()) != registry['source_atlas_sha256']:
        raise InventoryError('Atlas changed before installed low-hair repairs completed')
    atlas = json.loads(atlas_path.read_text())
    flagged = []
    for name, row in registry['frames'].items():
        if row['body'] == 'child' or row['activity'] == 'asleep' or row['original_hair_texels'] >= 3:
            continue
        entry = atlas['sprites'][name]
        page = entry['page']
        if isinstance(page, int):
            page = atlas['pages'][page]
        x, y, w, h = entry['rect']
        sprite = load_image(root / page.removeprefix('res://')).crop((x, y, x + w, y + h))
        if digest(sprite.tobytes()) != row['rgba_sha256']:
            raise InventoryError(f'Installed RGB changed: {name}')
        mask = classify(sprite, palettes[row['body']], row['activity'])
        path = root / row['mask_path']
        mask.save(path, optimize=True)
        row['mask_sha256'] = digest(path.read_bytes())
        row['hair_texels'] = int((np.asarray(mask)[..., 0] == 3).sum())
        row['source'] = 'hair_maps'
        flagged.append({'frame_id': name, 'before': row['original_hair_texels'], 'after': row['hair_texels']})
    save_json(root / REGISTRY, registry)
    report = {'completed': True, 'source_atlas_sha256': registry['source_atlas_sha256'],
        'drawable_villager_frames': len(registry['frames']), 'reviewed_corpus_images': len(pairs),
        'training_count_policy': 'Each unique reviewed body/RGBA image once; empty-body entries verified but not trained',
        'palette_sha256': digest(palette_path.read_bytes()), 'low_hair_adults_for_audit': flagged,
        'reviewed_sources': [source for _, _, _, source in pairs],
        'body_counts': {body: sum(row['body'] == body for row in registry['frames'].values()) for body in palettes},
        'palette_counts': {body: {'seen': len(p.seen_rgb), 'hair': len(p.hair_rgb)} for body, p in palettes.items()}}
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')
    print(f"Completed {len(registry['frames'])} masks, {len(pairs)} reviewed partners, {len(flagged)} low-hair adults")


if __name__ == '__main__':
    if sys.argv[1:] == ['extract']:
        print(f"Frozen {len(extract()['frames'])} villager frames")
    elif sys.argv[1:] == ['complete']:
        complete()
    else:
        raise SystemExit('Usage: hair_inventory_p4.py extract|complete')
