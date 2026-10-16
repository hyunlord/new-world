# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/key_targets_p5.py (after Part 3 installation)
"""Freeze the section 8.1 target list from the actual post-mask atlas."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path

from PIL import Image

from key_plan_p5 import OUTPUT, ROOT, TARGETS


def freeze() -> Path:
    """Require a missing/non-real frame or a pixel-bound wrong-pose/view rejection."""
    atlas_path = ROOT / 'assets/art/atlas/world_atlas.json'
    atlas = json.loads(atlas_path.read_text())
    registry = json.loads((ROOT / 'tools/art_pipeline/reviewed_hair_maps_p4.json').read_text())['frames']
    audit = json.loads((ROOT / 'docs/art/production4/natural_audit/final_reviews.json').read_text())['verdicts']
    rejected = {r['sprite_id']: r['reason'] for r in audit if r['verdict'] == 'reject' and r['reason'] in ('wrong_pose', 'wrong_view')}
    catalog = json.loads((OUTPUT / 'fill_catalog.json').read_text())
    rows = []
    for sheet in catalog['sheets']:
        groups = {}
        for cell in sheet['cells']:
            target = cell['target_slot']
            installed = atlas['sprites'].get(target)
            reasons = []
            if installed is None or not installed.get('drawable', True):
                reasons.append('no_installed_frame')
            else:
                for field in ('qa_status', 'fallback_kind'):
                    if installed.get(field) in ('procedural', 'derived', 'fallback'):
                        reasons.append(f'{field}:{installed[field]}')
                if installed.get('source', '').startswith(('procedural:', 'derived:', 'fallback:')):
                    reasons.append('non_generated_source')
                if target in rejected and target in registry:
                    page = installed['page']
                    if isinstance(page, int):
                        page = atlas['pages'][page]
                    x, y, w, h = installed['rect']
                    with Image.open(ROOT / page.removeprefix('res://')) as image:
                        rgba = image.convert('RGBA').crop((x, y, x + w, y + h)).tobytes()
                    if hashlib.sha256(rgba).hexdigest() == registry[target]['rgba_sha256']:
                        reasons.append(f'unchanged_audited_{rejected[target]}')
            key = (sheet['body'], cell['view'], cell['activity'])
            group = groups.setdefault(key, {'body': key[0], 'view': key[1], 'activity': key[2], 'targets': [], 'needs_fill': False})
            group['targets'].append({'slot': target, 'reasons': reasons})
            group['needs_fill'] |= bool(reasons)
        rows.append({'sheet': sheet['id'], 'generate': any(g['needs_fill'] for g in groups.values()), 'groups': list(groups.values())})
    path = OUTPUT / 'keys/targets_after_part3.json'
    if path.exists():
        message = 'Refusing to reinterpret an earlier Part 4 target checkpoint'
        raise ValueError(message)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps({'schema_version': 1, 'atlas_sha256': hashlib.sha256(atlas_path.read_bytes()).hexdigest(),
                               'sheet_order': list(TARGETS), 'sheets': rows}, indent=2) + '\n')
    return path


if __name__ == '__main__':
    print(freeze())
