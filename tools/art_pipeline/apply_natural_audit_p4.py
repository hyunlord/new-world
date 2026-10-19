# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
"""Dry-run a complete installed audit; apply only an explicit final envelope digest."""
from __future__ import annotations

import argparse
from collections import Counter
from dataclasses import replace
import json
from pathlib import Path

from install_v4 import (InstallError, InstallPlan, InstallRequest, PlannedSprite, Target,
                       apply_plan, digest, fallback_for, validate_assembled, verify_file)
from hair_inventory_p4 import load_image, digest as pixel_digest
from hair_registry_p4 import HairFrame, HairRegistry, ResolvedHair
from v4_contracts import FrameCandidate, FrameSpec, WorldScale, target_size

ROOT = Path(__file__).resolve().parents[2]
REPORT = Path('docs/art/production4/natural_audit/apply_report.json')


def safe_sources(sprites: tuple[PlannedSprite, ...], verdicts: dict[str, str]) -> tuple[PlannedSprite, ...]:
    """Only fresh-ok, already accepted pixels can supply the existing fallback chain."""
    return tuple(sprite for sprite in sprites if sprite.qa_status == 'accepted' and
                 verdicts.get(sprite.slot_id) == 'ok')


def replacement(old: PlannedSprite, sources: tuple[PlannedSprite, ...], icons: frozenset[str]) -> PlannedSprite | None:
    """Keep explicit idle-icon placeholders honest; regenerate all other rejected frames."""
    if old.qa_status == 'fallback' and old.fallback_kind == 'idle_icon':
        return old
    sheet, cell = old.slot_id.rsplit('_', 1)
    limit = target_size(FrameSpec(sheet, int(cell), 1, old.body, old.view, old.activity))
    return fallback_for(Target(old.slot_id, old.body, old.view, old.activity, old.frame), sources, icons, limit)


def replacement_mask(sprite: PlannedSprite, sources: tuple[PlannedSprite, ...], hair: HairRegistry) -> ResolvedHair:
    """Copy source masks only for identical RGBA and compatible anatomical exclusions."""
    bbox = sprite.sprite.getchannel('A').getbbox()
    excluded = sprite.body == 'child' or sprite.activity in ('asleep', 'lying') or not bbox or bbox[2]-bbox[0] > bbox[3]-bbox[1]
    source = next((source for source in sources if source.slot_id == sprite.source_sprite), None)
    if not excluded and source is not None and source.sprite.size == sprite.sprite.size and source.sprite.tobytes() == sprite.sprite.tobytes():
        return hair.resolve(HairFrame(source.body, source.activity, source.sprite))
    return hair.resolve(HairFrame(sprite.body, sprite.activity, sprite.sprite))


def plan_audit(root: Path, review_paths: tuple[Path, ...]) -> tuple[InstallPlan, dict]:
    """Bind source files, resolve rejections, and report blockers without asset writes."""
    manifest_path = root / 'assets/art/manifest.json'
    manifest = json.loads(manifest_path.read_text())
    evidence = {manifest_path: digest(manifest_path)}
    actors = []
    others = []
    for row in manifest['assets']:
        if not row.get('sprite_id') or not row.get('drawable', True) or not row['path'].startswith('assets/art/production/'):
            continue
        if not row.get('native_path'):
            continue
        for field in ('path', 'native_path'):
            path = root / row[field]
            evidence[path] = digest(path)
        item = PlannedSprite(row['sprite_id'], row.get('body', ''), row.get('view', ''), row.get('activity', ''),
            row.get('frame', 0), row['qa_status'], row['source'], row.get('source_sprite', ''),
            row.get('contains_basket', False), row.get('fallback_kind', ''), row.get('activity_icon', ''),
            load_image(root / row['native_path']), load_image(root / row['path']), row.get('candidate_id', ''))
        (actors if item.body else others).append(item)
    reviews = {}
    for supplied in review_paths:
        path = supplied if supplied.is_absolute() else root / supplied
        envelope = json.loads(path.read_text())
        evidence[path] = digest(path)
        for source, expected in envelope.get('review_sources_sha256', {}).items():
            source_path = root / source
            verify_file(source_path, expected)
            evidence[source_path] = expected
        if envelope.get('board_index'):
            index = root / envelope['board_index']
            verify_file(index, envelope['board_index_sha256'])
            evidence[index] = envelope['board_index_sha256']
            for pages in json.loads(index.read_text()).values():
                for page in pages:
                    board = root / page['path']
                    verify_file(board, page['sha256'])
                    evidence[board] = page['sha256']
        for row in envelope['verdicts']:
            name = row['sprite_id']
            if name in reviews or row['verdict'] not in ('ok', 'reject'):
                raise InstallError(f'Duplicate or invalid final verdict: {name}')
            reviews[name] = row['verdict']
    if set(reviews) != {sprite.slot_id for sprite in actors}:
        raise InstallError('Natural audit must cover each installed villager exactly once')
    sources = safe_sources(tuple(actors), reviews)
    icons = frozenset(sprite.slot_id for sprite in others if sprite.slot_id.startswith('U1_'))
    chosen = []
    changed = []
    blocked = []
    retained_blocked = []
    for sprite in actors:
        result = sprite if reviews[sprite.slot_id] == 'ok' else replacement(sprite, sources, icons)
        if result is None:
            blocked.append(sprite.slot_id)
            retained = sprite if sprite.qa_status == 'fallback' and sprite.fallback_kind else replace(
                sprite, qa_status='fallback', fallback_kind='retained_installed')
            chosen.append(retained)
            retained_blocked.append(sprite.slot_id)
            changed.append({'frame_id': sprite.slot_id, 'from': sprite.qa_status, 'to': retained.qa_status,
                'fallback_kind': retained.fallback_kind, 'source_sprite': retained.source_sprite,
                'blocked_reason': 'no_fresh_ok_fallback_source', 'original_source': sprite.source})
            continue
        chosen.append(result)
        if reviews[sprite.slot_id] == 'reject':
            changed.append({'frame_id': sprite.slot_id, 'from': sprite.qa_status, 'to': result.qa_status,
                            'fallback_kind': result.fallback_kind, 'source_sprite': result.source_sprite,
                            'kept_existing_idle_icon': result is sprite})
    errors = []
    source_candidates = []
    selections = []
    for relative in ('docs/art/production3/reprocess/selection.json', 'docs/art/production3/installation_selection.json'):
        selection_path = root / relative
        selections.append(json.loads(selection_path.read_text()))
        evidence[selection_path] = digest(selection_path)
    ledgers = {}
    for sprite in chosen:
        if sprite.qa_status not in ('accepted', 'derived'):
            continue
        decision = next((selection['slots'][sprite.slot_id] for selection in selections
            if selection['slots'].get(sprite.slot_id, {}).get('candidate_id') == sprite.candidate_id), None)
        if not decision:
            errors.append(f'Missing original accepted candidate: {sprite.slot_id}')
            continue
        path = root / decision['ledger']
        verify_file(path, decision['ledger_sha256'])
        evidence[path] = digest(path)
        if path not in ledgers:
            ledgers[path] = {row['id']: row for row in json.loads(path.read_text())['candidates']}
        record = ledgers[path][sprite.candidate_id]
        review_path = root / decision['verdict']
        verify_file(review_path, decision['verdict_sha256'])
        evidence[review_path] = digest(review_path)
        original_review = json.loads(review_path.read_text())
        if not any(row['candidate_id'] == sprite.candidate_id and row['verdict'] == 'ok' for row in original_review['verdicts']):
            errors.append(f'Original candidate lacks approval: {sprite.slot_id}')
        for kind, image in (('native', sprite.native), ('sprite', sprite.sprite)):
            image_path = root / record['images'][kind]
            evidence[image_path] = digest(image_path)
            original = load_image(image_path)
            if image.size != original.size or image.tobytes() != original.tobytes():
                errors.append(f'Installed {kind} changed from approved candidate: {sprite.slot_id}')
        if not record['guards']['passed'] or record['guards']['errors'] or any(record.get(key, {}).get('errors') for key in ('group_guards', 'cross_guards')):
            errors.append(f'Original technical guards failed: {sprite.slot_id}')
        metrics = record['metrics']
        sheet, cell = sprite.slot_id.rsplit('_', 1)
        candidate = FrameCandidate(FrameSpec(sheet, int(cell), 1, sprite.body, sprite.view, sprite.activity, sprite.native.size),
            sprite.native, metrics['mean_delta_e'], metrics['palette_match_percent'], metrics['binary_alpha'],
            metrics['berry_texels'], metrics['fire_texels'])
        source_candidates.append((Target(sprite.slot_id, sprite.body, sprite.view, sprite.activity, sprite.frame), candidate))
    diagnostics = ()
    try:
        diagnostics = validate_assembled(tuple(chosen + others), tuple(source_candidates), WorldScale())
    except InstallError as error:
        errors.append(str(error))
    hair = HairRegistry.load(root)
    mask_proposals = []
    for sprite in chosen:
        mask = replacement_mask(sprite, sources, hair)
        mask_proposals.append({'frame_id': sprite.slot_id, 'source': mask.source,
                               'source_mask_path': mask.path, 'verdict': mask.verdict})
    for path in (root / 'tools/art_pipeline/reviewed_hair_maps_p4.json', root / 'tools/art_pipeline/hair_palette_p4.json'):
        evidence[path] = digest(path)
    for mask in hair.maps.values():
        if mask.path:
            evidence[root / mask.path] = mask.sha256
    selection = review_paths[0] if review_paths[0].is_absolute() else root / review_paths[0]
    plan = InstallPlan(InstallRequest(root, selection), digest(selection), tuple(chosen), manifest.get('suppressed', {}),
                       tuple(evidence.items()), tuple(blocked), tuple(diagnostics))
    report = {'applied': False, 'review_paths': [str(path) for path in review_paths],
        'safe_fallback_sources': [source.slot_id for source in sources], 'blocked_targets': blocked,
        'retained_blocked_explicit_fallbacks': retained_blocked, 'guard_errors': errors, 'fallback_guard_diagnostics': list(diagnostics), 'changes': changed,
        'before': dict(Counter(sprite.qa_status for sprite in actors)),
        'after': dict(Counter(sprite.qa_status for sprite in chosen)),
        'fallbacks_after': dict(Counter(sprite.fallback_kind for sprite in chosen if sprite.fallback_kind)),
        'mask_proposals': mask_proposals, 'evidence': {str(path): sha for path, sha in evidence.items()},
        'selection_sha256': plan.selection_sha256}
    return plan, report


def persist_changed_masks(plan: InstallPlan, report: dict) -> InstallPlan:
    """After explicit apply authorization, freeze replacement masks before installer preflight."""
    root = plan.request.root
    for path, expected in plan.evidence:
        verify_file(path, expected)
    registry_path = root / 'tools/art_pipeline/reviewed_hair_maps_p4.json'
    registry = json.loads(registry_path.read_text())
    hair = HairRegistry.load(root)
    sources = tuple(sprite for sprite in plan.sprites if sprite.slot_id in report['safe_fallback_sources'])
    changes = []
    for sprite in plan.sprites:
        row = registry['frames'][sprite.slot_id]
        rgba = pixel_digest(sprite.sprite.tobytes())
        if row['rgba_sha256'] == rgba:
            continue
        mask = replacement_mask(sprite, sources, hair)
        path = root / row['mask_path']
        updated = dict(row, rgba_sha256=rgba, size=list(sprite.sprite.size), source=mask.source,
                       verdict=mask.verdict, inherited_mask_path=mask.path)
        changes.append((path, mask.image, updated, sprite.slot_id))
    for path, mask, row, name in changes:
        mask.save(path, optimize=True)
        row['mask_sha256'] = digest(path)
        row['hair_texels'] = mask.getchannel('R').histogram()[3]
        registry['frames'][name] = row
    registry_path.write_text(json.dumps(registry, indent=2, sort_keys=True) + '\n')
    report['replacement_masks_written'] = [name for _, _, _, name in changes]
    updated_paths = {registry_path, *(path for path, _, _, _ in changes)}
    return replace(plan, evidence=tuple((path, digest(path) if path in updated_paths else expected)
                                       for path, expected in plan.evidence))


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=ROOT)
    parser.add_argument('--reviews', type=Path, nargs='+', required=True)
    parser.add_argument('--apply-sha256')
    parser.add_argument('--allow-retained-blocked', action='store_true')
    args = parser.parse_args()
    plan, report = plan_audit(args.root, tuple(args.reviews))
    if args.apply_sha256:
        if len(args.reviews) != 1 or (report['blocked_targets'] and not args.allow_retained_blocked) or report['guard_errors']:
            raise InstallError('Application requires one final envelope and zero blockers/guard errors')
        if args.apply_sha256 != plan.selection_sha256:
            raise InstallError('Final envelope digest mismatch')
        plan = persist_changed_masks(plan, report)
        apply_plan(plan, args.apply_sha256)
        report['applied'] = True
    path = args.root / REPORT
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({key: report[key] for key in ('applied', 'before', 'after', 'blocked_targets', 'guard_errors')}))


if __name__ == '__main__':
    main()
