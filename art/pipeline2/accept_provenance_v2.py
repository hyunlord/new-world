# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scikit-image"]
# ///
# How to run: imported by art.pipeline2.accept_v2 during promotion preflight
"""Self-contained production sidecars retaining original records and current evidence."""
from __future__ import annotations

import json
from dataclasses import asdict
from pathlib import Path
from typing import Final

from PIL import Image

from art.pipeline2.accept_types_v2 import Candidate, Review, review_problem, sha
from art.pipeline2.judge import FEATURES, judge, load_calibration

DOCS: Final = Path(__file__).resolve().parents[2] / "docs/v8/arttrial2"


def build_sidecar(candidate: Candidate, review: Review, evidence: Path) -> str:
    """Serialize actual generation, processing, numeric and model provenance without guesses."""
    processing_path = candidate.provenance or candidate.working.with_suffix('.json')
    if not processing_path.is_file():
        message = f'Missing required processing sidecar: {processing_path}'
        raise ValueError(message)
    if candidate.provenance_sha256 is not None and sha(processing_path) != candidate.provenance_sha256:
        message = f'Processing provenance changed: {processing_path}'
        raise ValueError(message)
    if sha(candidate.working) != candidate.working_sha256:
        message = f'Working sprite changed: {candidate.working}'
        raise ValueError(message)
    processing = json.loads(processing_path.read_text())
    raw_sidecar = candidate.raw.with_name(candidate.raw.stem.removesuffix('_raw') + '.json')
    retry_generation = candidate.raw.with_name(candidate.raw.stem.removesuffix('_raw') + '_generation.json')
    generation_path = raw_sidecar if raw_sidecar.is_file() else retry_generation
    generation = json.loads(generation_path.read_text()) if generation_path.is_file() else {}
    combined = generation | processing
    prompt = combined.get('prompt')
    operations = processing.get('operations')
    if not isinstance(prompt, str) or not prompt.strip() or not operations:
        message = f'Full prompt and ordered processing operations are required: {processing_path}'
        raise ValueError(message)
    refs = combined.get('reference_images', combined.get('anchors', []))
    generation_refs = []
    for ref in refs:
        path = Path(ref['path'] if isinstance(ref, dict) else ref)
        actual_hash = sha(path) if path.is_file() else None
        recorded = ref.get('sha256') if isinstance(ref, dict) else combined.get('anchor_sha256', {}).get(str(path))
        if recorded is not None and actual_hash is not None and recorded != actual_hash:
            message = f'Generation reference changed: {path}'
            raise ValueError(message)
        generation_refs.append({'path': str(path), 'sha256': recorded or actual_hash,
                                'hash_basis': 'recorded generation hash' if recorded else 'recomputed preserved source bytes',
                                'unavailable_reason': None if recorded or actual_hash else 'Reference bytes and historical hash unavailable'})
    if len(generation_refs) < 2 or any(ref['sha256'] is None for ref in generation_refs):
        message = f'Two generation reference identities required: {processing_path}'
        raise ValueError(message)
    with Image.open(candidate.working) as image:
        dimensions = list(image.size)
    calibration_path = DOCS / 'JUDGE_V2_CALIBRATION.json'
    calibration = load_calibration(calibration_path)
    allowed = {a.sha256 for a in calibration.anchors if a.family == candidate.family}
    problem = review_problem(candidate, review, allowed)
    if problem is not None:
        message = f'Cannot write accepted sidecar: {problem}'
        raise ValueError(message)
    numeric = judge(candidate.raw, calibration, review.scores, family=candidate.family,
                    model_family_verified=True)
    if not numeric.passed:
        message = f'Cannot write accepted sidecar for failing numeric/model evidence: {candidate.name}'
        raise ValueError(message)
    thresholds = next(r for r in calibration.ranges if r.family == candidate.family)
    model_record = None
    if review.manifest and review.manifest.is_file():
        model_data = json.loads(review.manifest.read_text())
        records = model_data['records'] if isinstance(model_data, dict) else model_data
        matches = [r for r in records if r['candidate'] == str(review.raw)
                   and r['scorecard_sha256'] == review.scorecard_sha256]
        if len(matches) == 1:
            model_record = matches[0]
    if model_record is None:
        message = 'Full fresh-model context/provenance record is required'
        raise ValueError(message)
    start = combined.get('started_at', combined.get('start_utc', combined.get('start', combined.get('started_ms'))))
    end = combined.get('completed_at', combined.get('ended_at', combined.get('end_utc', combined.get('end', combined.get('ended_ms')))))
    wall = combined.get('wall_seconds', combined.get('duration_seconds', combined.get('generation_seconds')))
    generation_id = combined.get('generation_id')
    payload = {
        'status': 'accepted under judge v2, TRIAL2b', 'asset': candidate.name,
        'family': candidate.production_family, 'try_count': candidate.attempt,
        'prompt': prompt, 'anchors': generation_refs,
        'generation_id': generation_id,
        'generation_id_note': combined.get('generation_id_note') or (None if generation_id else 'No explicit generation ID in retained metadata; source path preserved without inventing an ID'),
        'original': str(candidate.raw), 'original_sha256': candidate.raw_sha256,
        'working_source': str(candidate.working), 'working_sha256': candidate.working_sha256,
        'working_size': dimensions, 'operations': operations,
        'anchor_point': processing.get('anchor_point'),
        'anchor_point_note': processing.get('anchor_point_note', 'origin top-left; x right, y down' if processing.get('anchor_point') is not None else 'Not recorded or not applicable; no placement coordinate invented'),
        'generation_started_at': start, 'generation_completed_at': end,
        'generation_timestamp_note': combined.get('timestamp_note') or ('Retained source values; started_ms/ended_ms are epoch milliseconds' if 'started_ms' in combined else 'Retained recorded values; missing values were not captured'),
        'wall_seconds': wall,
        'wall_seconds_note': None if wall is not None else 'Generation wall time unavailable in retained metadata; not estimated from review timing',
        'numeric': asdict(numeric),
        'numeric_thresholds': {'features': list(FEATURES), 'family': thresholds.family,
                               'lower': thresholds.lower, 'upper': thresholds.upper,
                               'comparison_size': calibration.comparison_size, 'stroke_comparison_size': 1024},
        'calibration_path': str(calibration_path), 'calibration_sha256': sha(calibration_path),
        'model': {'scores': asdict(review.scores), 'references': [{'path': str(p), 'sha256': h} for p, h in review.references],
                  'scorecard': str(review.scorecard), 'scorecard_sha256': review.scorecard_sha256,
                  'review_manifest': str(review.manifest) if review.manifest else None,
                  'review_manifest_sha256': sha(review.manifest) if review.manifest else None,
                  'full_record': model_record,
                  'identity_note': 'Actual image-generation model scorecard; underlying model version not exposed; call context is in full_record'},
        'acceptance_evidence': str(evidence), 'acceptance_evidence_sha256': sha(evidence),
        'generation_record': {'path': str(generation_path) if generation_path.is_file() else None,
                              'sha256': sha(generation_path) if generation_path.is_file() else None,
                              'full_record': generation or None},
        'processing_record': {'path': str(processing_path), 'sha256': sha(processing_path),
                              'full_record': processing},
        'historical_record_note': 'Nested historical numeric/status fields are preserved as history; top-level status and numeric/model are TRIAL2b decisions.',
    }
    return json.dumps(payload, indent=2) + '\n'
