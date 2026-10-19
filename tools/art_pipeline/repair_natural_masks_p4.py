"""One bounded hair-classifier repair of independently rejected installed masks."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import shutil

import numpy as np
from PIL import Image

from hair_maps import HairPalette, classify
from natural_boards_p4 import ROOT, atlas_frames, compose, save_board


def main() -> None:
    audit = ROOT / 'docs/art/production4/natural_audit'
    registry_path = ROOT / 'tools/art_pipeline/reviewed_hair_maps_p4.json'
    registry = json.loads(registry_path.read_text())
    palettes = json.loads((ROOT / 'tools/art_pipeline/hair_palette_p4.json').read_text())['bodies']
    rejected = set()
    for path in (audit / 'reviews').glob('*.json'):
        rejected.update(row['sprite_id'] for row in json.loads(path.read_text())['verdicts'] if row['reason'] == 'hair_mask_wrong')
    keys = {(r["body"], r["activity"], r["rgba_sha256"]) for ident, r in registry["frames"].items() if ident in rejected}
    affected = {ident for ident, r in registry["frames"].items() if (r["body"], r["activity"], r["rgba_sha256"]) in keys}
    frames = atlas_frames()
    changes = []
    for frame in frames:
        if frame.ident not in affected:
            continue
        record = registry['frames'][frame.ident]
        palette_data = palettes[frame.body]
        palette = HairPalette(frame.body, frozenset(tuple(rgb) for rgb in palette_data['seen_rgb']), frozenset(tuple(rgb) for rgb in palette_data['hair_rgb']))
        mask = classify(frame.sprite, palette, frame.activity)
        path = ROOT / record['mask_path']
        backup = audit / 'original_masks' / path.name
        backup.parent.mkdir(parents=True, exist_ok=True)
        if backup.exists():
            raise ValueError('Repair already run: ' + str(backup))
        shutil.copyfile(path, backup)
        old = np.asarray(frame.mask)
        new = np.asarray(mask)
        mask.save(path)
        changes.append({'frame': frame.ident, 'before': int((old[..., 0] == 3).sum()), 'after': int((new[..., 0] == 3).sum()), 'changed': int(np.any(old != new, axis=-1).sum()), 'old_mask_sha256': record['mask_sha256']})
        record.update(mask_sha256=hashlib.sha256(path.read_bytes()).hexdigest(), source='hair_maps', verdict='pending', hair_texels=int((new[..., 0] == 3).sum()))
    registry_path.write_text(json.dumps(registry, indent=2) + '\n')
    (audit / 'repair.json').write_text(json.dumps({'method': 'Replace rejected installed maps with exact section5.3 head-connected classifier; no algorithm adjustment.', 'frames': changes}, indent=2) + '\n')
    frames = atlas_frames()
    index: dict[str, list[dict[str, object]]] = {}
    for body in ('adult_a', 'adult_b'):
        rows = [row for row in frames if row.ident in rejected and row.body == body]
        with Image.open(ROOT / f'docs/art/production3/reprocess/references/{body}.png') as image:
            reference = image.convert('RGBA')
        index[body] = []
        for start in range(0, len(rows), 12):
            chunk = rows[start:start + 12]
            path = save_board(compose(chunk, reference), audit / f'round2/{body}_{start // 12 + 1:02}.png')
            index[body].append({'path': str(path.relative_to(ROOT)), 'sha256': hashlib.sha256(path.read_bytes()).hexdigest(), 'frames': [r.ident for r in chunk]})
    (audit / 'round2/index.json').write_text(json.dumps(index, indent=2) + '\n')
    print('Repaired', len(changes), 'hair masks; original evidence preserved')


if __name__ == '__main__':
    main()
