# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Recount raw attempts and audit immutable provenance without modifying assets.

Run from repository root: python3 -m art.pipeline3.cost_report
"""
from __future__ import annotations

import hashlib
import json
import sys
from collections import Counter
from datetime import UTC, datetime
from pathlib import Path
from typing import TypedDict, cast


class Reference(TypedDict):
    path: str
    sha256: str


class Metadata(TypedDict, total=False):
    schema_version: int
    kind: str
    image: str
    image_sha256: str
    mask: str
    mask_sha256: str
    operations: list[str]
    generation_calls: int
    generation_record: str
    source_sidecar: str
    source_sidecar_sha256: str
    source_image_sha256: str
    source_mask_sha256: str
    raw: str
    raw_sha256: str
    prompt: str
    prompt_file: str
    references: list[str | Reference]
    reference_hashes: dict[str, str]
    reference_sha256: dict[str, str]
    generation_seconds: float
    elapsed_seconds: float
    elapsed_minutes: float


Call = TypedDict('Call', {
    'layer': str, 'try': str, 'raw': str, 'raw_sha256': str,
    'seconds': float | None, 'timing_semantics': str,
})

ROOT = Path(__file__).resolve().parents[1]
RAW = Path('/Users/rexxa/work/arttrial3-data/route_a')
OUT = ROOT.parent / 'docs/v8/arttrial3'


def read(path: Path) -> Metadata:
    value = cast(Metadata | list[Metadata], json.loads(path.read_text()))
    return value if isinstance(value, dict) else {}


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def resolve(value: str, base: Path) -> Path:
    path = Path(value)
    if path.is_absolute():
        return path
    local = base / path
    return local if local.exists() else ROOT.parent / path


def generation_metadata(path: Path) -> Metadata:
    metadata = read(path)
    record = metadata.get('generation_record')
    if record and Path(record).exists():
        combined = read(Path(record))
        combined.update(metadata)
        metadata = combined
    return metadata


def audit(path: Path) -> list[str]:
    metadata = generation_metadata(path)
    issues: list[str] = []
    for label, value, expected in [
        ('image', metadata.get('image', path.with_suffix('.png').name), metadata.get('image_sha256')),
        ('mask', metadata.get('mask', ''), metadata.get('mask_sha256')),
    ]:
        asset = resolve(value, path.parent)
        if not asset.is_file():
            issues.append(f'{label}: missing file {asset}')
        elif not expected:
            issues.append(f'{label}: recorded SHA256 missing; current={digest(asset)}')
        elif digest(asset) != expected:
            issues.append(f'{label}: SHA256 MISMATCH')
    if not metadata.get('operations'):
        issues.append('operations: missing')
    if metadata.get('generation_calls') == 0:
        source = Path(metadata.get('source_sidecar', ''))
        if not source.exists():
            issues.append('derived source: missing sidecar')
        else:
            src = read(source)
            for key, asset in [('source_sidecar_sha256', source),
                               ('source_image_sha256', source.with_suffix('.png')),
                               ('source_mask_sha256', source.parent / src.get('mask', ''))]:
                if metadata.get(key) != digest(asset):
                    issues.append(f'{key}: MISMATCH')
        return issues
    raw = resolve(metadata.get('raw', ''), path.parent)
    if not raw.is_file():
        issues.append('raw: missing')
    elif metadata.get('raw_sha256') != digest(raw):
        issues.append('raw: SHA256 missing or MISMATCH')
    if not metadata.get('prompt'):
        prompt = resolve(metadata.get('prompt_file', ''), path.parent)
        if not prompt.is_file() or not prompt.read_text().strip():
            issues.append('prompt: missing or unresolved prompt_file')
    hashes = metadata.get('reference_hashes', metadata.get('reference_sha256', {}))
    refs = metadata.get('references', [])
    if not refs:
        issues.append('references: missing')
    for ref in refs:
        value = ref['path'] if isinstance(ref, dict) else ref
        expected = ref.get('sha256') if isinstance(ref, dict) else hashes.get(value)
        asset = resolve(value, path.parent)
        if not asset.is_file():
            issues.append(f'reference: missing {value}')
        elif not expected:
            issues.append(f'reference: recorded SHA256 missing {value}')
        elif digest(asset) != expected:
            issues.append(f'reference: SHA256 MISMATCH {value}')
    return issues


def main() -> None:
    accepted = [p for p in sorted((ROOT / 'character/route_a').glob('*.json'))
                if read(p).get('schema_version') == 1 and 'kind' in read(p)]
    adults = [p for p in accepted if read(p).get('generation_calls') != 0]
    calls: list[Call] = []
    for raw in sorted(RAW.glob('*/try*/raw.png')):
        sidecars = [p for p in raw.parent.glob('*.json') if read(p).get('schema_version') == 1]
        record = raw.parent / 'generation.json'
        metadata: Metadata = read(record) if record.exists() else (read(sidecars[0]) if sidecars else {})
        seconds = metadata.get('generation_seconds', metadata.get('elapsed_seconds'))
        timing = 'tool_call_wall_seconds'
        if seconds is None and 'elapsed_minutes' in metadata:
            seconds = metadata['elapsed_minutes'] * 60
            timing = 'includes_orchestration_upper_bound'
        calls.append({'layer': raw.parent.parent.name, 'try': raw.parent.name,
                      'raw': str(raw), 'raw_sha256': digest(raw), 'seconds': seconds,
                      'timing_semantics': timing if seconds is not None else 'NOT_RECORDED'})
    counts = Counter(c['layer'] for c in calls)
    durations = [c['seconds'] for c in calls if c['seconds'] is not None]
    direct = sum(c['seconds'] for c in calls if c['seconds'] is not None and c['timing_semantics'] == 'tool_call_wall_seconds')
    total = sum(durations)
    n = len(adults)
    unique_min, unique_max = 136, 196
    audits = {p.stem: audit(p) for p in accepted}
    report = {'snapshot_utc': datetime.now(UTC).isoformat(),
              'inventory_frozen': '--frozen' in sys.argv, 'generation_calls': len(calls),
              'accepted_generated_layers': n, 'accepted_derived_layers': len(accepted)-n,
              'attempts_per_accepted_generated_layer': len(calls)/n if n else None,
              'recorded_tool_wall_seconds': direct, 'all_recorded_elapsed_seconds': total,
              'calls_missing_time': sum(c['seconds'] is None for c in calls),
              'recorded_elapsed_minutes_per_accepted_layer_bounds': [direct/60/n,total/60/n] if n else None,
              'credits': 'UNKNOWN_NOT_EXPOSED', 'labor_minutes': 'NOT_MEASURED',
              'route_b_submitted_jobs': 0, 'route_b_credits': 0,
              'projection': {'literal_compositions': 10368, 'unique_layers_range': [unique_min,unique_max],
                             'recorded_elapsed_minutes_range': [direct/60/n*unique_min,total/60/n*unique_max] if n else None,
                             'not_a_delivery_forecast': True},
              'attempts_by_layer': dict(sorted(counts.items())), 'calls': calls,
              'accepted_provenance_issues': audits}
    OUT.mkdir(parents=True, exist_ok=True)
    _ = (OUT/'COST.json').write_text(json.dumps(report,indent=2)+'\n')
    lines = ['# Cost and projection — Route A', '', f"Snapshot: {report['snapshot_utc']}. Inventory frozen: {report['inventory_frozen']}.", '',
             f'- Actual generated calls: {len(calls)} (every raw try directory counted once; failures retained).',
             f'- Accepted generated layers: {n}; separately derived layers: {len(accepted)-n} (zero new calls).',
             f'- All-route attempts / accepted generated layer: {len(calls)/n:.3f}.',
             f'- Recorded tool-call seconds: {direct:.3f}; including orchestration elapsed records: {total:.3f}.',
             f'- Recorded elapsed minutes / accepted layer bounds: {direct/60/n:.3f}–{total/60/n:.3f}.',
             '- These bounds describe recorded generation intervals only, not labor, sequential route duration or uniform API latency.',
             '- Imagegen credits/currency: UNKNOWN_NOT_EXPOSED, never zero. Human/agent labor NOT_MEASURED.',
             '- B: zero submitted generation jobs and zero generation credits; preflight labor remains unmeasured.', '',
             '## Projection assumptions', '',
             'Literal product space: 4 bodies ×3 tops ×4 hair ×6 items ×2 headgear ×3 directions ×6 frames =10,368 compositions.',
             'Layer reuse avoids one generation per composition. Conservative full body-specific wardrobe: 4×[16 body frames +(3 tops +4 hair +2 headgear)×3 directions +6 items] =196 unique layers.',
             'Shared hair/headgear/items lower estimate: 4×16 bodies +4×3×3 tops +(4 hair +2 headgear)×3 directions +6 items×3 direction variants =136 layers.',
             f'Applying measured all-route cost per accepted layer yields {direct/60/n*unique_min:.1f}–{total/60/n*unique_max:.1f} recorded generation minutes.',
             f'At the observed attempts ratio this implies {len(calls)/n*unique_min:.1f}–{len(calls)/n*unique_max:.1f} calls, not guaranteed successes.',
             'Sharing compatibility is hypothetical; authored sleep variants, more item poses, revisions, derivative repair, testing, composition, review and integration can increase cost. Failed route completeness invalidates any delivery promise.',
             'No credit range can be inferred without billing records. Raw calls and their timing semantics are enumerated in COST.json.', '']
    _ = (OUT/'COST.md').write_text('\n'.join(lines))
    audit_lines = ['# Accepted provenance audit', '', f"Snapshot: {report['snapshot_utc']}; read-only checks against current bytes.", '',
                   'Checks: generated raw/image/mask hashes, prompt resolution, references and hashes, operation log; derived source sidecar/image/mask hashes.',
                   'This does not prove semantic accuracy of masks or listed operations. Failure attempts are counted in COST.json; accepted assets alone are audited here.', '']
    for name, issues in audits.items():
        audit_lines.append(f"- {name}: " + ('PASS' if not issues else '; '.join(issues)))
    _ = (OUT/'PROVENANCE_AUDIT.md').write_text('\n'.join(audit_lines)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k not in ('calls','accepted_provenance_issues')},indent=2))


if __name__ == '__main__':
    main()
