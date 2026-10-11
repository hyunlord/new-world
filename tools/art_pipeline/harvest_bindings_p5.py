# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: imported by merge_p5_selection and harvest_integrity_p5
"""Immutable candidate-order, pixel-provenance, animation, and review bindings."""
from __future__ import annotations

from copy import deepcopy
import hashlib
import json
from pathlib import Path
from tempfile import TemporaryDirectory

from PIL import Image, ImageSequence
from harvest_boards_p5 import boards
from harvest_p5 import Completion
from install_v4 import InstallError, digest, verify_file
from salvage_verdicts import atomic_json, candidate_fingerprint


def read(path: Path) -> dict:
    return json.loads(path.read_text())


def image_pixels(path: str) -> tuple[tuple[int, int], bytes]:
    with Image.open(path) as opened:
        image = opened.convert('RGBA')
    return image.size, image.tobytes()


def freeze(output: Path, proposals: list[dict], records: dict[str, dict], *, recovery_commit: str = '') -> None:
    """Freeze exact normalized records and animation order before new review begins."""
    path = output / 'animation/candidate_bindings.json'
    if path.exists():
        raise InstallError('Candidate binding sidecar already exists')
    groups = {}
    for proposal in proposals:
        groups[proposal['id']] = dict(candidate_ids=proposal['candidate_ids'],
            candidate_fingerprints=[candidate_fingerprint(records[i]) for i in proposal['candidate_ids']],
            board=proposal['board'], board_sha256=digest(Path(proposal['board'])),
            gif=proposal['gif'], gif_sha256=digest(Path(proposal['gif'])))
    atomic_json(path, dict(schema_version=1, recovery_commit=recovery_commit,
        installer_ledger_sha256=digest(output / 'installer_candidates.json'), groups=groups,
        candidate_fingerprints={identity:candidate_fingerprint(row) for identity,row in records.items()}))


def verify_prepared(output: Path, proposals: list[dict], records: dict[str, dict], bindings: dict) -> None:
    """Reject mutable ledgers or animations before minting a new installer envelope."""
    frozen = read(output / 'animation/candidate_bindings.json')
    verify_file(output / 'installer_candidates.json', frozen['installer_ledger_sha256'])
    for identity, fingerprint in frozen['candidate_fingerprints'].items():
        if identity not in records or candidate_fingerprint(records[identity]) != fingerprint:
            raise InstallError(f'Prepared record changed: {identity}')
    for proposal in proposals:
        group = frozen['groups'].get(proposal['id'])
        if group is None or group['candidate_ids'] != proposal['candidate_ids']:
            raise InstallError(f'Animation candidate order changed: {proposal["id"]}')
        if group['candidate_fingerprints'] != [candidate_fingerprint(records[i]) for i in proposal['candidate_ids']]:
            raise InstallError(f'Animation candidate pixels changed: {proposal["id"]}')
        verify_file(Path(group['board']), group['board_sha256'])
        verify_file(Path(group['gif']), group['gif_sha256'])
    original_cache = {}
    for identity, row in records.items():
        source_id = row.get('original_candidate_id')
        if not source_id or source_id not in bindings:
            continue  # Immutable historical lineage, never selected by this pass.
        binding = bindings[source_id]
        ledger = binding.get('ledger')
        if not ledger:
            ledger = ('docs/art/production4/wood/processed/candidates.json' if 'wood/' in binding['review_index']
                      else 'docs/art/production4/recovery/candidates.json')
        if ledger not in original_cache:
            original_cache[ledger] = {r['id']:r for r in read(Path(ledger))['candidates']}
        original = original_cache[ledger][source_id]
        if candidate_fingerprint(original) != binding['fingerprint']:
            raise InstallError(f'Original reviewed pixels changed: {source_id}')
        if row['source_kind'] == 'generated':
            excluded = {'guards','group_guards','cross_guards','guards_p4',
                        'original_candidate_id','original_candidate_fingerprint'}
            if {k:v for k,v in row.items() if k not in excluded} != {k:v for k,v in original.items() if k not in excluded}:
                raise InstallError(f'Normalized record changed original content: {identity}')
        else:
            source = records.get(row.get('source_candidate'))
            if source is None or row.get('derivation_rule') != 'same_cycle_phase_copy':
                raise InstallError(f'Derived source missing: {identity}')
            if (source.get('source_sheet',source['sheet']),source['attempt']) != (row.get('source_sheet',row['sheet']),row['attempt']):
                raise InstallError(f'Derived source set changed: {identity}')
            if source['frame'] % 2 != row['frame'] % 2:
                raise InstallError(f'Derived source phase changed: {identity}')
        source_image = original if row['source_kind']=='generated' else source
        if any(image_pixels(row['images'][kind]) != image_pixels(source_image['images'][kind]) for kind in ('native','sprite')):
            raise InstallError(f'Installed pixels differ from approved source: {identity}')


def bind_review(output: Path, review_path: Path) -> None:
    """Freeze the fresh review bytes once; subsequent calls verify, never replace."""
    destination = output / 'animation/review_bindings.json'
    if destination.exists():
        data = read(destination)
        verify_file(Path(data['review_path']), data['review_sha256'])
        verify_file(output / 'animation/candidate_bindings.json', data['candidate_bindings_sha256'])
        if Path(data['review_path']).resolve() != review_path.resolve():
            raise InstallError('Review path changed')
    else:
        atomic_json(destination, dict(review_path=str(review_path), review_sha256=digest(review_path),
            candidate_bindings_sha256=digest(output / 'animation/candidate_bindings.json')))


def recover(output: Path, commit: str) -> None:
    """Bind earlier exact-reviewed pixels without inventing a pre-review timestamp."""
    import subprocess
    report = read(output / 'adjudication.json')
    ledger = read(output / 'installer_candidates.json')
    records = {r['id']:r for r in ledger['candidates']}
    envelope = read(output / 'envelope.json')
    for relative in ('installer_candidates.json','envelope.json','adjudication.json','animation/final_reviews.json'):
        path = output / relative
        committed = subprocess.run(['git','show',f'{commit}:{path}'], capture_output=True, check=True).stdout
        if committed != path.read_bytes():
            raise InstallError(f'Cannot recover changed committed evidence: {path}')
    # Re-render each selected reviewed board from its exact committed frame records.
    # This is a post-review proof sidecar, explicitly labelled with the checkpoint.
    baseline = read(output / 'baseline_manifest.json')
    for proposal in report['groups']:
        for identity in proposal['candidate_ids']:
            row = records[identity]
            if envelope['candidate_fingerprints'][row['slot_id']] != candidate_fingerprint(row):
                raise InstallError(f'Cannot recover changed candidate: {identity}')
        reference_row = next(r for r in baseline['assets'] if r.get('body')==proposal['group'][0] and
                             r.get('activity')=='idle' and r.get('view')=='S' and r.get('drawable',True) and r.get('sprite_id'))
        with Image.open(reference_row['path']) as opened:
            reference = opened.convert('RGBA')
        with TemporaryDirectory() as temporary:
            completion = Completion(proposal['kind'], tuple(records[i] for i in proposal['candidate_ids']), proposal['copies'])
            rendered = boards(completion, reference, Path(temporary))
            if image_pixels(rendered['board']) != image_pixels(proposal['board']):
                raise InstallError(f'Reviewed board does not represent frozen frames: {proposal["id"]}')
            with Image.open(rendered['gif']) as first, Image.open(proposal['gif']) as second:
                a = [f.convert('RGBA').tobytes() for f in ImageSequence.Iterator(first)]
                b = [f.convert('RGBA').tobytes() for f in ImageSequence.Iterator(second)]
                if a != b:
                    raise InstallError(f'Reviewed animation differs from frozen frames: {proposal["id"]}')
    freeze(output, report['groups'], records, recovery_commit=commit)
    bind_review(output, output / 'animation/final_reviews.json')
