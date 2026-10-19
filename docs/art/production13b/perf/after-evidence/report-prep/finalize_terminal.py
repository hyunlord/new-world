"""Apply verified terminal disposition to generated external reports only."""
import json
import hashlib
from pathlib import Path
from collections import Counter

BASE = Path('/Users/rexxa/work/art-production13b-evidence')
PREP = BASE / 'after-report-prep'
audit_path = BASE / 'part4-terminal-audit.json'
audit = json.loads(audit_path.read_text())
report = json.loads((PREP / 'comparison.json').read_text())
counts = Counter(row['classification'] for row in audit['rows'])
assert dict(counts) == audit['counts']
assert counts == {'VALID': 4, 'COLLECTED_INVALID': 3, 'PRELAUNCH_REFUSAL': 21, 'POLICY_REFUSAL_WRONG_INHERITED_ITEM': 1}
by_path = {row['path']: row for row in audit['rows']}
probes, policies = [], []
for row in report['measurement_probes']:
    rel = str(Path(row['source']['path']).relative_to(BASE / 'after'))
    classification = by_path[rel]['classification']
    row['classification'] = classification
    if classification == 'POLICY_REFUSAL_WRONG_INHERITED_ITEM':
        assert not row['start'] and not row['end']
        policies.append(row)
    else:
        for end in ('start', 'end'):
            assert row[end]['epoch'] == by_path[rel][end]['epoch']
            assert row[end]['wall_seconds'] == by_path[rel][end]['wall_seconds']
            assert row[end]['load_1m'] == by_path[rel][end]['load_1m']
        probes.append(row)
assert len(probes) == 28 and len(policies) == 1
assert set(report['matrix']) == {'W0', 'W1', 'W200', 'W500'}
assert not report['sections'] and not report['ablation']
report['status'] = 'TERMINAL_NOT_VERIFIED'
report['measurement_probes'] = probes
report['policy_denials_without_probes'] = policies
report['terminal_audit'] = {'path': str(audit_path), 'sha256': hashlib.sha256(audit_path.read_bytes()).hexdigest(), **{key: value for key, value in audit.items() if key != 'rows'}}
report['unverified_matrix'] = report.pop('pending_matrix')
report['unverified_sections'] = report.pop('pending_sections')
report['ablation_status'] = 'NOT_VERIFIED_NO_COLLECTOR_LAUNCHED'
for row in report['H1_H11']:
    if row['after_status'] == 'PENDING': row['after_status'] = 'NOT_VERIFIED_IN_PART4_TABLE'
(PREP / 'comparison.json').write_text(json.dumps(report, indent=2, allow_nan=False)+'\n')
text = (PREP / 'comparison.md').read_text().replace('비교 초안', '최종 부분 비교').replace('전체 관문: 진행 중.', 'Part4 종료: NOT_VERIFIED.').replace('PENDING', 'NOT_VERIFIED')
# The generic draft prints two empty policy rows; these are not probe endpoints.
text = '\n'.join(line for line in text.splitlines() if not (line.startswith('| after-W-multi-attempt4.json.validity.json |'))) + '\n'
text += '\n정책 거부(프로브 없음): after-W-multi-attempt4.json.validity.json — P13_ITEM=after-ablation-W200 상속 오류. 파일5는 환경을 해제한 실제 W-multi 재시도였으며 시작 load 17.5810546875로 거부되었다. 프로브 28회/56 endpoint + 정책 거부 1회 = 29 slots.\n'
(PREP / 'comparison.md').write_text(text)
agg_path = BASE / 'after-final-aggregation/after-detail.json'
agg = json.loads(agg_path.read_text())
agg['terminal_status'] = 'NOT_VERIFIED'
agg['terminal_admission_counts'] = dict(counts)
agg['terminal_missing_acceptance'] = {'matrix': ['W-multi'], 'sections': ['W200', 'W-multi', 'W500'], 'ablation': ['W200']}
agg['uncollected_raw_coverage'] = agg.pop('pending')
agg['raw_inventory_caveat'] = 'Seven raw captures include three collected-invalid attempts. Complete raw shape never establishes admission. Only the four official-summary sources in after-report-prep/comparison.json enter before/after comparisons.'
agg_path.write_text(json.dumps(agg, indent=2, allow_nan=False)+'\n')
md = BASE / 'after-final-aggregation/after-detail.md'
text = md.read_text().replace('Pending:', 'Uncollected raw coverage (terminal NOT_VERIFIED):')
md.write_text('# Terminal NOT_VERIFIED — raw inventory includes invalid captures\n\n'+agg['raw_inventory_caveat']+'\n\n'+text)
verification = {'status': 'TERMINAL_NOT_VERIFIED', 'admitted_matrix': sorted(report['matrix']), 'official_statistic_checks': 192, 'calibration_rows': len(report['calibration_runs']), 'measurement_probe_sidecars': len(probes), 'real_endpoint_rows': 2*len(probes), 'policy_denials_without_probes': len(policies), 'slots': len(probes)+len(policies), 'counts': dict(counts), 'invalid_raw_used_in_comparison': False, 'reusable_helpers_modified': False}
(PREP / 'terminal-verification.json').write_text(json.dumps(verification, indent=2)+'\n')
print(json.dumps(verification))
