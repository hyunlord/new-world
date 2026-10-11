# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/harvest_integrity_p5.py
"""A20: audit installed pixels against frozen original approvals and animation reviews."""
from __future__ import annotations

import json
from pathlib import Path
from PIL import Image
from salvage_verdicts import atomic_json, candidate_fingerprint


def pixels(path: str) -> tuple[tuple[int, int], bytes]:
    with Image.open(path) as opened:
        image = opened.convert('RGBA')
    return image.size, image.tobytes()


def audit(output: Path = Path('docs/art/production5/harvest')) -> dict:
    """Recompute integrity; never equate a source label with verified provenance."""
    read = lambda path: json.loads(Path(path).read_text())
    selection = read(output / 'selection.json')
    report = read(output / 'adjudication.json')
    envelope = read(output / 'envelope.json')
    records = {r['id']: r for r in read(output / 'installer_candidates.json')['candidates']}
    manifest = {r['sprite_id']: r for r in read('assets/art/manifest.json')['assets'] if r.get('sprite_id') and r.get('drawable', True)}
    baseline = {r['sprite_id']: r for r in read(output / 'baseline_manifest.json')['assets'] if r.get('sprite_id') and r.get('drawable', True)}
    original_records = {}
    for name in ('docs/art/production4/recovery/candidates.json', 'docs/art/production4/wood/processed/candidates.json'):
        original_records.update({r['id']:r for r in read(name)['candidates']})
    errors, rows = [], []
    reviews = {r['id']: r for r in read(output / 'animation/final_reviews.json')['verdicts']}
    groups = {tuple(g['group']):g for g in report['groups']}
    for target, identity in report['selected'].items():
        row, installed = records[identity], manifest.get(target)
        reasons = []
        if installed is None or installed.get('candidate_id') != identity:
            reasons.append('installed_identity_changed')
        elif any(pixels(installed[field]) != pixels(row['images'][kind]) for field, kind in (('path','sprite'),('native_path','native'))):
            reasons.append('installed_pixels_differ_from_reviewed')
        if envelope['candidate_fingerprints'].get(row['slot_id']) != candidate_fingerprint(row):
            reasons.append('installer_fingerprint_changed')
        source_id = row.get('original_candidate_id', identity)
        original = original_records.get(source_id)
        binding = envelope['original_bindings'].get(source_id)
        if original is None or binding is None or candidate_fingerprint(original) != binding['fingerprint']:
            reasons.append('original_fingerprint_not_bound')
        elif read(binding['review_index'])['candidate_fingerprints'].get(source_id) != binding['fingerprint']:
            reasons.append('original_index_changed')
        else:
            verdict = next((v for v in read(binding['review_path'])['verdicts'] if v['candidate_id'] == source_id), None)
            if verdict is None or verdict['verdict'] != 'ok':
                reasons.append('original_verdict_not_ok')
        if row['source_kind'] == 'derived':
            parent = records.get(row['source_candidate'])
            if (parent is None or row['derivation_rule'] != 'same_cycle_phase_copy' or
                    (parent.get('source_sheet', parent['sheet']), parent['attempt']) !=
                    (row.get('source_sheet', row['sheet']), row['attempt']) or
                    any(pixels(row['images'][kind]) != pixels(parent['images'][kind]) for kind in ('native','sprite'))):
                reasons.append('copy_not_exact_same_set')
        if row['body']:
            group = groups.get((row['body'], row['view'], row['activity']))
            if group is None or reviews.get(group['id'], {}).get('verdict') != 'ok':
                reasons.append('animation_not_approved')
            elif len({(records[i].get('source_sheet', records[i]['sheet']), records[i]['attempt']) for i in group['candidate_ids']}) != 1:
                reasons.append('mixed_physical_animation')
        rows.append(dict(target=target, candidate_id=identity, inherited_from=source_id,
                         derived=row['source_kind'] == 'derived', errors=reasons))
        errors.extend(f'{target}:{reason}' for reason in reasons)
    changed = []
    for target, row in manifest.items():
        old = baseline.get(target, {})
        if row.get('body') and any(row.get(k) != old.get(k) for k in ('candidate_id','source','qa_status')):
            changed.append(target)
            if target not in report['selected']:
                errors.append(f'{target}:changed_group_without_harvest_review')
    result = dict(check='A20', passed=not errors, errors=errors, selected_frames=len(rows),
                  derived_frames=sum(row['derived'] for row in rows), changed_villager_frames=len(changed),
                  animation_groups=len(groups), borrowed_groups_changed=0, rows=rows)
    atomic_json(output / 'integrity.json', result)
    return result


if __name__ == '__main__':
    result = audit()
    print(json.dumps({key:value for key,value in result.items() if key != 'rows'}))
    raise SystemExit(0 if result['passed'] else 1)
