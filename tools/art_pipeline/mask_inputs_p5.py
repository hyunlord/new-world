# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: imported by remask_candidates_p5.py.
"""Snapshot the three mask scopes and approved reference provenance without mutation."""
from __future__ import annotations

from copy import deepcopy
import hashlib
import json
from pathlib import Path

import numpy as np
from PIL import Image
from hair_masks_v5 import Reference
from natural_boards_p4 import atlas_frames, ROOT

OUTPUT = ROOT / 'docs/art/production5/masks'


def load(path: Path) -> dict:
    """Read an existing JSON ledger."""
    return json.loads(path.read_text())


def image(path: str | Path) -> Image.Image:
    """Detach decoded pixels from the underlying file."""
    with Image.open(ROOT / path) as opened:
        return opened.convert('RGBA')


def save_image(value: Image.Image, path: Path) -> str:
    """Persist a pass-five snapshot and return a root-relative path."""
    path.parent.mkdir(parents=True, exist_ok=True)
    value.save(path)
    return str(path.relative_to(ROOT))


def collect() -> tuple[list[dict], list[Reference], list[dict]]:
    """Recompute 45+31+29, and reference counts from approved stored masks."""
    p4 = ROOT / 'docs/art/production4'
    recovery = load(p4 / 'recovery/candidates.json')['candidates']
    wood = load(p4 / 'wood/processed/candidates.json')['candidates']
    recovery_reviews = load(p4 / 'recovery/reviews/final_reviews.json')['verdicts']
    wood_reviews = load(p4 / 'wood/reviews/final_reviews.json')['verdicts']
    audit = load(p4 / 'natural_audit/final_reviews.json')['verdicts']
    frames = {frame.ident: frame for frame in atlas_frames()}
    manifest = load(ROOT / 'assets/art/atlas/world_atlas.json')['sprites']
    registry = load(ROOT / 'tools/art_pipeline/reviewed_hair_maps_p4.json')['frames']
    references: list[Reference] = []
    evidence: list[dict] = []
    inputs: list[dict] = []
    for scope, candidates, verdicts in [('a', recovery, recovery_reviews), ('b', wood, wood_reviews)]:
        lookup = {row['id']: row for row in candidates}
        for verdict in verdicts:
            original = lookup[verdict['candidate_id']]
            if verdict['reason'] == 'hair_mask_wrong':
                record = deepcopy(original)
                record.update(mask_scope=scope, original_candidate_id=record['id'], id=record['id'] + ':mask5', hair_mask_source='template_v5')
                inputs.append(record)
            if scope == 'a' and verdict['verdict'] == 'ok':
                ref = Reference(original['id'], original['body'], original['view'], original['activity'],
                                image(original['images']['sprite']), image(original['images']['sprite_hair_mask']))
                references.append(ref)
                evidence.append({'id': ref.ident, 'body': ref.body, 'view': ref.view, 'activity': ref.activity,
                                 'sprite_path': original['images']['sprite'], 'mask_path': original['images']['sprite_hair_mask'],
                                 'source': 'recovery', 'review_path': 'docs/art/production4/recovery/reviews/final_reviews.json'})
    for verdict in audit:
        ident = verdict['sprite_id']
        frame = frames[ident]
        if verdict['verdict'] == 'ok':
            ref = Reference(ident, frame.body, frame.view, frame.activity, frame.sprite, frame.mask)
            references.append(ref)
            sprite_path = save_image(frame.sprite, OUTPUT / 'inputs' / ident / 'sprite.png')
            evidence.append({'id': ident, 'body': ref.body, 'view': ref.view, 'activity': ref.activity,
                             'sprite_path': sprite_path, 'mask_path': registry[ident]['mask_path'],
                             'source': 'natural_audit', 'review_path': 'docs/art/production4/natural_audit/final_reviews.json'})
        if verdict['reason'] != 'hair_mask_wrong':
            continue
        sprite_path = save_image(frame.sprite, OUTPUT / 'inputs' / ident / 'sprite.png')
        native = np.array(frame.sprite)
        native[np.all(native[..., :3] == (3, 0, 0), axis=-1)] = 0
        native_path = save_image(Image.fromarray(native), OUTPUT / 'inputs' / ident / 'native.png')
        mask_path = save_image(frame.mask, OUTPUT / 'inputs' / ident / 'original_mask.png')
        source_slot = manifest[ident].get('source_sprite') or ident
        source = frames.get(source_slot, frame)
        inputs.append({'id': ident + ':mask5', 'slot_id': ident, 'target_slot': ident, 'original_candidate_id': ident,
                       'body': frame.body, 'view': frame.view, 'activity': frame.activity, 'frame': frame.frame,
                       'mask_scope': 'c', 'hair_mask_source': 'template_v5', 'canonical_view': source.view,
                       'canonical_activity': source.activity, 'canonical_slot': source_slot,
                       'images': {'sprite': sprite_path, 'native': native_path, 'sprite_hair_mask': mask_path},
                       'audited_rgba_sha256': hashlib.sha256(frame.sprite.tobytes()).hexdigest(),
                       'pre_p5_status': manifest[ident]['qa_status']})
    eligible = {ref.ident for ref in references if ref.body.startswith('adult') and
                ref.activity not in ('asleep', 'lying', 'carry_wood', 'gather_wood') and
                int(np.count_nonzero(np.asarray(ref.mask)[..., 0] == 3)) >= 6}
    return inputs, [ref for ref in references if ref.ident in eligible], [row for row in evidence if row['id'] in eligible]
