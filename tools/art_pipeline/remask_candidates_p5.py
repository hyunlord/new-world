# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/remask_candidates_p5.py
"""Materialise immutable 105-scope position proposals for independent blind review."""
from __future__ import annotations

from collections import Counter
import hashlib
import json
from pathlib import Path

import numpy as np
from PIL import Image
from hair_masks_v5 import colour_counts, remask
from mask_inputs_p5 import ROOT, OUTPUT, collect, image, save_image
from mask_boards_p5 import card
from salvage_verdicts import candidate_fingerprint


def write(path: Path, payload: dict) -> None:
    """Write a new pass-five artifact; historical reviews are never opened for writing."""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2) + '\n')


def main() -> None:
    """Snapshot references, remask each RGBA once, and bind every blind card."""
    if (OUTPUT / 'review_index.json').exists():
        raise FileExistsError('Frozen mask review index already exists')
    rows, references, evidence = collect()
    reference_counts = Counter((ref.body, ref.view) for ref in references)
    body_counts = {body: colour_counts(references, body) for body in ('adult_a', 'adult_b')}
    write(OUTPUT / 'references.json', {'references': evidence,
          'counts': {f'{body}/{view}': count for (body, view), count in sorted(reference_counts.items())},
          'body_colour_counts': {body: [{'rgb': list(rgb), 'count': count} for rgb, count in sorted(counts.items())]
                                 for body, counts in body_counts.items()}})
    idles = {body: image(OUTPUT / 'inputs' / ('V_a1_01' if body == 'adult_a' else 'V_b1_01') / 'sprite.png')
             for body in ('adult_a', 'adult_b')}
    grouped: dict[tuple[str, str], list[dict]] = {}
    for row in rows:
        sprite = image(row['images']['sprite'])
        key = (row['body'], hashlib.sha256(sprite.tobytes()).hexdigest())
        grouped.setdefault(key, []).append(row)
    results = []
    sheets: list[dict] = []
    for group in grouped.values():
        canonical = min(group, key=lambda row: (row.get('canonical_slot', row['slot_id']) != row['slot_id'], row['id']))
        sprite = image(canonical['images']['sprite'])
        view = canonical.get('canonical_view', canonical['view'])
        activity = canonical.get('canonical_activity', canonical['activity'])
        outcome = remask(sprite, image(canonical['images']['native']), canonical['body'], view, activity, references)
        for row in group:
            ident = row['id']
            folder = OUTPUT / 'candidates' / ident.replace(':', '_').replace('@', '_')
            old = image(row['images']['sprite_hair_mask'])
            mask = outcome.mask or old
            row['images']['sprite_hair_mask'] = save_image(mask, folder / 'sprite_hair_mask.png')
            native = image(row['images']['native'])
            mx, my = (sprite.width-native.width)//2, (sprite.height-native.height)//2
            native_mask = np.array(mask.crop((mx,my,mx+native.width,my+native.height)))
            native_mask[..., 3] = np.asarray(native)[..., 3]
            row['images']['hair_mask'] = save_image(Image.fromarray(native_mask), folder / 'hair_mask.png')
            row['mask_v5'] = {'reference_id': outcome.reference_id, 'iou': outcome.iou, 'shift': list(outcome.shift),
                              'new_mask': outcome.mask is not None, 'canonical_id': canonical['id'], 'canonical_view': view,
                              'copies': [other['id'] for other in group], 'hair_texels': int(np.count_nonzero(np.asarray(mask)[..., 0] == 3))}
            row['review_status'] = 'pending_independent_visual_review'
            results.append(row)
            board = card(ident, row['activity'], row['view'], sprite, mask, idles[row['body']])
            path = OUTPUT / 'boards' / (ident.replace(':','_').replace('@','_') + '.png')
            save_image(board, path)
            sheets.append({'candidate_id': ident, 'scope': row['mask_scope'], 'path': str(path.relative_to(ROOT)),
                           'sha256': hashlib.sha256(path.read_bytes()).hexdigest()})
    results.sort(key=lambda row: (row['mask_scope'], row['id']))
    write(OUTPUT / 'candidates.json', {'schema_version': 1, 'candidates': results})
    write(OUTPUT / 'review_index.json', {'candidate_fingerprints': {row['id']: candidate_fingerprint(row) for row in results},
                                        'boards': sheets, 'rubric': 'docs/art/production4/REVIEW_RUBRIC.md'})
    summary = {'scope_counts': dict(Counter(row['mask_scope'] for row in results)), 'unique_rgba': len(grouped),
               'remasked': dict(Counter(row['mask_scope'] for row in results if row['mask_v5']['new_mask'])),
               'no_new_mask': [row['id'] for row in results if not row['mask_v5']['new_mask']],
               'reference_counts': {f'{body}/{view}': count for (body,view),count in sorted(reference_counts.items())}}
    write(OUTPUT / 'summary.json', summary)
    print(json.dumps(summary, indent=2))


if __name__ == '__main__':
    main()
