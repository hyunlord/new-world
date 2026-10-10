# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: imported explicitly by merge_p4_selection after its dry-run succeeds.
"""Freeze exactly the selected, freshly reviewed hair masks before installation."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Protocol

import numpy as np
from PIL import Image

from hair_registry_p4 import HairRegistry
from install_v4 import InstallError, InstallPlan, digest, verify_file
from salvage_verdicts import atomic_json, candidate_fingerprint


class HairFreezeRequest(Protocol):
    """The recovery output location, independent of its orchestration module."""

    root: Path
    output: Path


def freeze_selected_hair(request: HairFreezeRequest, plan: InstallPlan) -> int:
    """Validate every proposal before writing; preserve unrelated registry records."""
    root = request.root.resolve()
    output = root / request.output
    selection_path = output / 'selection.json'
    if plan.request.root.resolve() != root or plan.request.selection.resolve() != selection_path.resolve():
        raise InstallError('Hair freeze request differs from validated recovery plan')
    verify_file(selection_path, plan.selection_sha256)
    for path, expected in plan.evidence:
        verify_file(path, expected)
    selection = json.loads(selection_path.read_text())
    report = json.loads((output / 'adjudication.json').read_text())
    if report['selection_sha256'] != plan.selection_sha256:
        raise InstallError('Hair freeze adjudication selection changed')
    registry_path = root / 'tools/art_pipeline/reviewed_hair_maps_p4.json'
    registry = json.loads(registry_path.read_text())
    hair = HairRegistry.load(root)
    sprites = {sprite.slot_id: sprite for sprite in plan.sprites}
    proposals = []
    proposed_masks = {}
    for target, identity in report['selected'].items():
        decision = selection['slots'][target]
        if decision['candidate_id'] != identity or decision['disposition'] not in ('accepted', 'derived'):
            continue
        ledger_path, review_path = root / decision['ledger'], root / decision['verdict']
        verify_file(ledger_path, decision['ledger_sha256'])
        verify_file(review_path, decision['verdict_sha256'])
        record = next(row for row in json.loads(ledger_path.read_text())['candidates'] if row['id'] == identity)
        review = json.loads(review_path.read_text())
        if (review['candidate_fingerprints'].get(record['slot_id']) != candidate_fingerprint(record) or
                not any(row['candidate_id'] == identity and row['verdict'] == 'ok' for row in review['verdicts'])):
            raise InstallError(f'Selected hair lacks exact fresh approval: {target}')
        sprite = sprites[target]
        if not sprite.body or sprite.body == 'child':
            continue
        with Image.open(root / record['images']['sprite']) as opened:
            original = opened.convert('RGBA')
        if original.size != sprite.sprite.size or original.tobytes() != sprite.sprite.tobytes():
            raise InstallError(f'Selected sprite changed after dry-run: {target}')
        source = root / record['images']['sprite_hair_mask']
        with Image.open(source) as opened:
            mask = opened.convert('RGBA')
        pixels = np.asarray(mask)
        rgba = np.asarray(sprite.sprite.convert('RGBA'))
        hair_pixels = pixels[..., 0] == 3
        if (mask.size != sprite.sprite.size or not np.array_equal(pixels[..., 3], rgba[..., 3]) or
                not np.all(np.isin(pixels[..., 0], [0, 3])) or np.any(pixels[..., 1] > 2) or
                np.any(pixels[..., 1][~hair_pixels]) or np.any(pixels[..., 2]) or
                np.any(hair_pixels & (pixels[..., 3] == 0))):
            raise InstallError(f'Invalid selected hair mask: {target}')
        bbox = sprite.sprite.getchannel('A').getbbox()
        excluded = sprite.body == 'child' or sprite.activity in ('asleep', 'lying') or bbox is None or bbox[2]-bbox[0] > bbox[3]-bbox[1]
        if (excluded and np.any(hair_pixels)) or (sprite.body.startswith('adult') and not excluded and not np.any(hair_pixels)):
            raise InstallError(f'Selected hair violates body/pose mask contract: {target}')
        pixel_sha = hashlib.sha256(sprite.sprite.tobytes()).hexdigest()
        key = (sprite.body, sprite.activity, pixel_sha)
        proposed = proposed_masks.get(key)
        if proposed is not None and proposed != mask.tobytes():
            raise InstallError(f'Conflicting proposed same-pixel hair masks: {target}')
        proposed_masks[key] = mask.tobytes()
        previous = hair.maps.get(key)
        if previous is not None and (previous.image.size != mask.size or previous.image.tobytes() != mask.tobytes()):
            raise InstallError(f'Conflicting existing same-pixel hair mask: {target}')
        path = root / f'tools/art_pipeline/hair_masks_p4/recovery/{target}.png'
        row = dict(registry['frames'].get(target, {}), mask_path=str(path.relative_to(root)),
            mask_sha256=digest(source), rgba_sha256=pixel_sha, body=sprite.body, activity=sprite.activity,
            size=list(mask.size), source='hair_maps', verdict='ok', visual_verdict='ok', visual_reason='', candidate_id=identity,
            reviewed_mask_path=str(source), review_path=str(review_path), review_sha256=digest(review_path),
            candidate_fingerprint=candidate_fingerprint(record), hair_texels=int(np.count_nonzero(hair_pixels)))
        proposals.append((path, source.read_bytes(), row, target))
    changed_paths = {registry_path, *(path for path, _, _, _ in proposals)}
    if any(path.resolve() in {changed.resolve() for changed in changed_paths} for path, _ in plan.evidence):
        raise InstallError('Plan pins pre-freeze hair state; regenerate the recovery plan')
    for path, data, row, target in proposals:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        registry['frames'][target] = row
    if proposals:
        atomic_json(registry_path, registry)
    return len(proposals)
