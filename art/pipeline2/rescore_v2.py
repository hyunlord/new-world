# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scikit-image"]
# ///
# How to run: uv run --with scikit-image python -m art.pipeline2.rescore_v2
"""Re-score the frozen 38 first attempts without inventing new model evidence."""
import hashlib
import json
from collections import Counter
from dataclasses import asdict
from pathlib import Path
from typing import Final

from art.pipeline2.judge import ModelScores, calibrate, judge

ROOT: Final = Path(__file__).resolve().parents[2]
DATA: Final = Path('/Users/rexxa/work/arttrial2-data')
DOCS: Final = ROOT / 'docs/v8/arttrial2'


def main() -> None:
    """Measure original try1 generation bytes; retain working-image and legacy review identities."""
    calibration = calibrate(ROOT / 'art/style05/anchors')
    _ = (DOCS / 'JUDGE_V2_CALIBRATION.json').write_text(json.dumps(asdict(calibration), indent=2) + '\n')
    source = DOCS / 'ENV_MODEL_REVIEW.json'
    records = json.loads(source.read_text())['first_wave']['records']
    if len(records) != 38:
        message = 'Frozen first-attempt cohort must contain exactly 38 assets'
        raise ValueError(message)
    results = []
    counts: Counter[str] = Counter()
    for record in records:
        path = Path(record['candidate'])
        if hashlib.sha256(path.read_bytes()).hexdigest() != record['candidate_sha256']:
            message = f'First-attempt candidate changed: {path}'
            raise ValueError(message)
        working_path = path
        path = (DATA / 'ground' / (record['name'] + '_try1.png') if record['family'] == 'ground'
                else path.with_stem(path.stem + '_raw'))
        family = 'items' if record['family'] == 'misc' else record['family']
        scores = ModelScores(record['medium'], record['line'], record['light'],
                             record['palette'], record['reason'])
        verdict = judge(path, calibration, scores, family=family)
        row = asdict(verdict)
        row.update(name=record['name'], production_family=record['family'],
                   working_candidate=str(working_path), working_sha256=record['candidate_sha256'],
                   legacy_model_source=str(source.relative_to(ROOT)),
                   legacy_scorecard=record['scorecard'],
                   legacy_scorecard_sha256=record['scorecard_sha256'],
                   legacy_reference_sha256=record['reference_sha256'],
                   numeric_and_legacy_model=verdict.numeric_pass and scores.passed,
                   fresh_same_family_review_required=True)
        results.append(row)
        key = record['family']
        counts[key + ':total'] += 1
        counts[key + ':numeric_pass'] += int(verdict.numeric_pass)
        counts[key + ':legacy_combined_pass'] += int(verdict.numeric_pass and scores.passed)
    negatives = []
    for name in ('negative_painterly', 'negative_woodcut', 'negative_photo_try1'):
        for family in ('person',):
            negatives.append(asdict(judge(DATA / (name + '.png'), calibration, family=family)))
    report = {'version': 2, 'cohort': '38 first-attempt raw generations; working bytes separately retained',
                  'calibration_status': 'FAIL' if any(v['numeric_pass'] for v in negatives) or not all(f.passed for f in calibration.leave_one_out) else 'PASS',
                  'strict_acceptance': 'NOT_VERIFIED: fresh same-family model review missing',
                  'numeric_pass': sum(r['numeric_pass'] for r in results),
                  'numeric_and_legacy_model_pass': sum(r['numeric_and_legacy_model'] for r in results),
                  'accepted': 0, 'counts': dict(sorted(counts.items())),
                  'limitations': ['Misc mapped to props/items; smoke has no genuinely matched anchor.',
                               'Two plants are planner-selected first-attempt references: their self-scores are not independent validation.',
                               'Legacy model references include whole scenes; reused scores are diagnostic only.',
                               'Numeric comparison uses native raw crops and raw candidates,256px local measures and1024px stroke normalization; delivered-size QA separate.'],
                  'negatives': negatives, 'records': results}
    _ = (DOCS / 'ENV_RESCORE_V2.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({k: v for k, v in report.items() if k not in ('records', 'negatives')}, indent=2))


if __name__ == '__main__':
    main()
