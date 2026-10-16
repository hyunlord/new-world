# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/apply_masks_p5.py (after independent animation reviews and mask freeze)
"""Restore audited statuses through the sole manifest installer after frozen-mask approval."""
from __future__ import annotations

from collections import Counter
from dataclasses import replace
import hashlib
import json
from pathlib import Path
from PIL import Image

from hair_registry_p4 import HairFrame, HairRegistry
from install_v4 import InstallError, InstallRequest, Target, apply_plan, build_plan, digest, validate_assembled
from merge_p5_selection import current_sprites
from pack_atlas import pack
from regions import build_atlases
from salvage_verdicts import atomic_json, candidate_fingerprint
from salvage_v4 import guarded_candidate
from v4_contracts import WorldScale


def apply(root: Path = Path.cwd()) -> dict:
    """Apply only frozen c masks and restore the exact audit-bound prior status."""
    read = lambda path: json.loads(Path(path).read_text())
    output = root / 'docs/art/production5/masks'
    destination = output / 'mask_installation.json'
    if destination.exists():
        raise InstallError('Mask installation evidence already exists')
    frozen = read(output / 'freeze_report.json')
    frames = [r for r in frozen['frames'] if r['installed']]
    affected = {tuple(r['group']) for r in frames}
    groups = [g for g in frozen['groups'] if tuple(g['group']) in affected]
    if any(g['verdict'] != 'ok' or not g['single_physical_set'] for g in groups):
        raise InstallError('Mask group lacks single-set animation approval')
    # A standard source-bound installer plan authorizes writes; its reconstructed fallbacks
    # are replaced by the exact current drawables so unrelated audited statuses remain intact.
    selection = root / 'docs/art/production5/harvest/selection.json'
    plan = build_plan(InstallRequest(root, selection))
    current = current_sprites(root)
    before = Counter(s.qa_status for s in current.values())
    hair = HairRegistry.load(root)
    evidence = dict(plan.evidence)
    evidence[output / 'freeze_report.json'] = digest(output / 'freeze_report.json')
    registry = root / 'tools/art_pipeline/reviewed_hair_maps_p5.json'
    evidence[registry] = digest(registry)
    applied = []
    for item in frames:
        target = item['target']
        sprite = current[target]
        ledger, index, review = (root / item[key] for key in ('ledger','review_index','review_path'))
        record = next(r for r in read(ledger)['candidates'] if r['id'] == item['review_candidate_id'])
        if candidate_fingerprint(record) != read(index)['candidate_fingerprints'][record['id']]:
            raise InstallError(f'Mask fingerprint changed: {target}')
        if not any(r['candidate_id'] == record['id'] and r['verdict'] == 'ok' for r in read(review)['verdicts']):
            raise InstallError(f'Mask approval missing: {target}')
        resolved = hair.resolve(HairFrame(sprite.body, sprite.activity, sprite.sprite, sprite.frame))
        expected_mask = root / item['mask_path']
        with Image.open(expected_mask) as opened:
            mask = opened.convert('RGBA')
        if resolved.image.size != mask.size or resolved.image.tobytes() != mask.tobytes():
            raise InstallError(f'Frozen registry mask mismatch: {target}')
        rgba = hashlib.sha256(sprite.sprite.tobytes()).hexdigest()
        status = item['prior_status'] if rgba == item['audited_rgba_sha256'] else sprite.qa_status
        current[target] = replace(sprite, qa_status=status)
        applied.append(dict(item, status_before=sprite.qa_status, status_after=status,
                            status_restored=rgba == item['audited_rgba_sha256'], rgba_sha256=rgba))
        for path in (ledger, index, review, expected_mask):
            evidence[path] = digest(path)
    source_records = {}
    selection_paths = list((root / 'docs/art/production5').glob('**/selection.json'))
    selection_paths += [root / path for path in ('docs/art/production3/installation_selection.json',
                       'docs/art/production3/reprocess/selection.json')]
    seen_ledgers = set()
    for selected_path in selection_paths:
        for decision in read(selected_path)['slots'].values():
            ledger_path = root / decision['ledger']
            if ledger_path in seen_ledgers:
                continue
            seen_ledgers.add(ledger_path)
            for record in read(ledger_path)['candidates']:
                source_records.setdefault(record['id'], record)
    candidates = []
    for target, sprite in current.items():
        record = source_records.get(sprite.candidate_id)
        if record:
            candidates.append((Target(target, sprite.body, sprite.view, sprite.activity, sprite.frame),
                               guarded_candidate(record)))
    for row in read(root / 'assets/art/manifest.json')['assets']:
        if row.get('sprite_id') in current:
            for field in ('path', 'native_path'):
                if row.get(field):
                    path = root / row[field]
                    evidence[path] = digest(path)
    diagnostics = validate_assembled(tuple(current.values()), tuple(candidates), WorldScale(),
                                     reviewed_group_keys=frozenset(tuple(g["group"]) for g in groups))
    plan = replace(plan, sprites=tuple(current.values()), evidence=tuple(evidence.items()),
                   suppressed={k:v for k,v in plan.suppressed.items() if k not in current},
                   fallback_guard_diagnostics=diagnostics)
    apply_plan(plan, plan.selection_sha256)
    pack(root)
    build_atlases(root / 'assets/art/atlas/world_atlas.json')
    report = dict(applied=True, frames=applied, groups=groups, before=dict(before),
                  after=dict(Counter(s.qa_status for s in current.values())),
                  blocked=[r for r in frozen['frames'] if not r['installed']])
    atomic_json(destination, report)
    return report


if __name__ == '__main__':
    result = apply()
    print(json.dumps({'frames':len(result['frames']), 'before':result['before'], 'after':result['after']}))
