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
from harvest_baseline_p5 import rgba_hash
from harvest_bindings_p5 import verify_prepared, bind_review
from install_v4 import InstallError, verify_file
from mask_installation_p5 import physical_set
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




def current_content(root: Path, manifest: dict) -> dict:
    """Hash native sprites and actual rendered mask texels, irrespective of labels."""
    atlas = json.loads((root / 'assets/art/atlas/region_atlas.json').read_text())
    pages, result = {}, {}
    for target, row in manifest.items():
        if not row.get('body'):
            continue
        values = {}
        for field in ('path', 'native_path'):
            with Image.open(root / row[field]) as image:
                values[field] = dict(size=list(image.size), rgba_sha256=rgba_hash(image))
        region = atlas['sprites'].get(target)
        if region:
            page = region['page'].removeprefix('res://')
            if page not in pages:
                with Image.open(root / page) as image:
                    pages[page] = image.convert('RGBA')
            x,y,w,h = region['rect']
            values['mask'] = dict(size=[w,h], rgba_sha256=rgba_hash(pages[page].crop((x,y,x+w,y+h))))
        result[target] = values
    return result


def content_changed(target: str, row: dict, old: dict, content: dict, baseline: dict) -> bool:
    return (any(row.get(k) != old.get(k) for k in ('candidate_id','source','qa_status'))
            or content.get(target) != baseline.get(target))


def audit_all(root: Path = Path.cwd()) -> dict:
    """Audit the final pass-five manifest across immutable per-installation reports."""
    read = lambda path: json.loads(Path(path).read_text())
    base = root / 'docs/art/production5'
    manifest = {r['sprite_id']:r for r in read(root / 'assets/art/manifest.json')['assets']
                if r.get('sprite_id') and r.get('drawable', True)}
    baseline = {r['sprite_id']:r for r in read(base / 'harvest/baseline_manifest.json')['assets']
                if r.get('sprite_id') and r.get('drawable', True)}
    installs = []
    for path in sorted(base.glob('**/adjudication.json')):
        if not (path.parent / 'installer_candidates.json').exists():
            continue
        report = read(path)
        if report.get('applied'):
            installs.append((path.parent, report))
    rows, errors, covered, groups = [], [], set(), {}
    content = current_content(root, manifest)
    mask_report_path = base / 'masks/mask_installation.json'
    mask_overrides = {}
    if mask_report_path.exists():
        mask_payload = read(mask_report_path)
        if mask_payload.get('applied'):
            mask_overrides = {entry['target']:entry for entry in mask_payload['frames']
                              if entry.get('installed') and pixels(manifest[entry['target']]['path']) == pixels(entry['sprite_path'])}
    baseline_content = read(base / 'harvest/baseline_content.json')['frames']
    originals = {}
    for name in ('docs/art/production4/recovery/candidates.json', 'docs/art/production4/wood/processed/candidates.json'):
        originals.update({r['id']:r for r in read(root / name)['candidates']})
    for directory, report in installs:
        records = {r['id']:r for r in read(directory / 'installer_candidates.json')['candidates']}
        envelope = read(directory / 'envelope.json')
        try:
            verify_prepared(directory, report['groups'], records, envelope['original_bindings'])
            if not (directory / 'animation/review_bindings.json').exists():
                raise InstallError('Missing frozen review binding')
            bind_review(directory, directory / 'animation/final_reviews.json')
        except (InstallError, KeyError, OSError) as error:
            errors.append(f'{directory}:binding_integrity:{error}')
        reviews = {r['id']:r for r in read(directory / 'animation/final_reviews.json')['verdicts']}
        for group in report['groups']:
            installed = [manifest.get(records[i].get('target_slot', records[i]['slot_id']), {}) for i in group['candidate_ids']]
            if all(row.get('candidate_id') == identity for row, identity in zip(installed, group['candidate_ids'])):
                key = tuple(group['group'])
                groups[key] = dict(group=group['group'], source=group['source'],
                    copied_frames=[records[i]['frame'] for i in group['candidate_ids'] if records[i]['source_kind']=='derived'],
                    animation_review=group['id'], candidate_ids=group['candidate_ids'],
                    installation=str(directory.relative_to(root)), board=group['board'], gif=group['gif'])
                if reviews.get(group['id'], {}).get('verdict') != 'ok':
                    errors.append(f'{key}:animation_not_ok')
                if len({(records[i].get('source_sheet', records[i]['sheet']), records[i]['attempt']) for i in group['candidate_ids']}) != 1:
                    errors.append(f'{key}:mixed_physical_sets')
        for target, identity in report['selected'].items():
            installed = manifest.get(target, {})
            if installed.get('candidate_id') != identity:
                continue  # Superseded groups are verified by their later installation report.
            record = records[identity]
            reasons = []
            if any(pixels(str(root / installed[field])) != pixels(record['images'][kind])
                   for field,kind in (('path','sprite'),('native_path','native'))):
                reasons.append('installed_pixels_changed')
            if record['body'] in ('adult_a','adult_b'):
                mask_path = mask_overrides.get(target, {}).get('mask_path', record['images'].get('sprite_hair_mask'))
                if not mask_path:
                    reasons.append('approved_hair_mask_missing')
                else:
                    with Image.open(mask_path) as mask:
                        expected = dict(size=list(mask.size), rgba_sha256=rgba_hash(mask))
                    if content[target].get('mask') != expected:
                        reasons.append('installed_mask_changed')
            if candidate_fingerprint(record) != envelope['candidate_fingerprints'].get(record['slot_id']):
                reasons.append('installer_fingerprint_changed')
            source_id = record.get('original_candidate_id', identity)
            binding = envelope['original_bindings'].get(source_id, {})
            original = originals.get(source_id)
            if binding.get('ledger'):
                original = next((r for r in read(root / binding['ledger'])['candidates'] if r['id']==source_id), None)
            if original is None or candidate_fingerprint(original) != binding.get('fingerprint'):
                reasons.append('original_fingerprint_changed')
            elif read(root / binding['review_index'])['candidate_fingerprints'].get(source_id) != binding['fingerprint']:
                reasons.append('review_index_changed')
            else:
                verdict = next((r for r in read(root / binding['review_path'])['verdicts'] if r['candidate_id']==source_id), {})
                if verdict.get('verdict') != 'ok':
                    reasons.append('frame_not_approved')
            if record['source_kind'] == 'derived':
                source = records[record['source_candidate']]
                if record['derivation_rule'] != 'same_cycle_phase_copy' or any(
                        pixels(record['images'][kind]) != pixels(source['images'][kind]) for kind in ('native','sprite')):
                    reasons.append('not_exact_phase_copy')
                if (source.get('source_sheet',source['sheet']),source['attempt']) != (record.get('source_sheet',record['sheet']),record['attempt']):
                    reasons.append('copy_mixed_source')
            if record['body'] and (record['body'],record['view'],record['activity']) not in groups:
                reasons.append('animation_group_not_bound')
            covered.add(target)
            errors.extend(f'{target}:{reason}' for reason in reasons)
            rows.append(dict(target=target, candidate_id=identity, installation=str(directory.relative_to(root)), errors=reasons))
    mask_report = base / 'masks/mask_installation.json'
    if mask_report.exists() and read(mask_report).get('applied'):
        payload = read(mask_report)
        for path, expected in read(base / 'masks/installation_bindings.json')['files'].items():
            try:
                verify_file(root / path, expected)
            except InstallError as error:
                errors.append(f'mask_installation_binding:{error}')
        for entry in payload['frames']:
            target = entry['target']
            installed = manifest[target]
            if pixels(str(root / installed['path'])) != pixels(entry['sprite_path']):
                continue  # A later generated replacement owns this slot now.
            index = read(root / entry['review_index'])
            candidates = {r['id']:r for r in read(root / entry['ledger'])['candidates']}
            record = candidates[entry['review_candidate_id']]
            verdicts = read(root / entry['review_path'])['verdicts']
            if candidate_fingerprint(record) != index['candidate_fingerprints'].get(record['id']):
                errors.append(f'{target}:mask_fingerprint_changed')
            if not any(r['candidate_id']==record['id'] and r['verdict']=='ok' for r in verdicts):
                errors.append(f'{target}:mask_not_approved')
            with Image.open(root / entry['mask_path']) as mask:
                expected_mask = dict(size=list(mask.size), rgba_sha256=rgba_hash(mask))
            if content[target].get('mask') != expected_mask:
                errors.append(f'{target}:installed_mask_changed')
            covered.add(target)
        for group in payload['groups']:
            current = [manifest[target] for target in group['frames']]
            if any(content[row['sprite_id']]['path']['rgba_sha256'] != group['frame_rgba_sha256'][row['sprite_id']] for row in current):
                continue
            reviews = read(root / group['review_path'])['verdicts']
            if not any(v['id']==group['id'] and v['verdict']=='ok' for v in reviews):
                errors.append(f'{group["group"]}:mask_animation_not_approved')
            try:
                verify_file(Path(group['board']), group['board_sha256'])
                verify_file(Path(group['gif']), group['gif_sha256'])
            except InstallError as error:
                errors.append(f'{group["group"]}:mask_animation_changed:{error}')
            sources = sorted({physical_set(row,manifest) for row in current})
            if sources != group['physical_sets'] or len(sources)!=1 or sources[0].startswith('unknown:'):
                errors.append(f'{group["group"]}:mask_physical_set_changed')
            key = tuple(group['group'])
            sheet, attempt = sources[0].rsplit('@',1)
            groups[key] = dict(group=group['group'],source=[sheet,int(attempt)],
                copied_frames=[r['frame'] for r in current if r['source'].startswith('derived:')],
                candidate_ids=[r.get('candidate_id') for r in current],animation_review=group['id'],
                installation='docs/art/production5/masks',board=group['board'],gif=group['gif'])
    changed = []
    for target, row in manifest.items():
        old = baseline.get(target, {})
        if row.get('body') and content_changed(target, row, old, content, baseline_content):
            changed.append(target)
            if target not in covered:
                errors.append(f'{target}:changed_without_bound_review')
    result = dict(check='A20', passed=not errors, errors=errors, installations=len(installs),
        final_bound_frames=len(covered), changed_villager_frames=len(changed), animation_groups=len(groups), rows=rows)
    atomic_json(base / 'harvest/integrity_final.json', result)
    atomic_json(base / 'harvest/installed_groups_final.json', {'groups':list(groups.values())})
    return result

if __name__ == '__main__':
    result = audit_all()
    print(json.dumps({key:value for key,value in result.items() if key != 'rows'}))
    raise SystemExit(0 if result['passed'] else 1)
