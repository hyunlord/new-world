# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scikit-image"]
# ///
# How to run: uv run --with scikit-image python -m art.pipeline2.accept_v2 --help
"""Join immutable fresh reviews with numeric and semantic QA; dry-run by default."""
from __future__ import annotations

import argparse
import json
import shutil
from pathlib import Path
from typing import Final

from art.pipeline2.accept_types_v2 import Candidate, Review, review_problem, sha
from art.pipeline2.judge import ModelScores, judge, load_calibration

ROOT: Final = Path(__file__).resolve().parents[2]
DATA: Final = Path('/Users/rexxa/work/arttrial2-data')
DOCS: Final = ROOT / 'docs/v8/arttrial2'
FAMILIES: Final = ('ground', 'plants', 'buildings', 'items', 'misc')


def accepted(numeric: bool, model: bool) -> bool:
    """Preserve preregistered C1/C2 acceptance: numeric AND model only."""
    return numeric and model


def promotion_eligible(style_pass: bool, semantic: bool | None) -> bool:
    """Content QA may withhold consumer promotion without changing measured C2."""
    return style_pass and semantic is True


def read_reviews(path: Path) -> list[Review]:
    """Read the image judge's explicit score transcriptions; never infer scores."""
    document = json.loads(path.read_text())
    records = document['records'] if isinstance(document, dict) else document
    reviews = []
    for row in records:
        if row.get('usable_transcription') is not True:
            continue
        scores = row['scores']
        refs = row['refs'][:2]
        hashes = row['input_sha256']
        raw = Path(row['candidate'])
        reviews.append(Review(raw, hashes[str(raw)], tuple((Path(p), hashes[p]) for p in refs),
                              Path(row['scorecard']), row['scorecard_sha256'],
                              ModelScores(scores['medium'], scores['line'], scores['light'], scores['palette'], row['reason']), path))
    return reviews


def read_candidates(retries: Path | None) -> list[Candidate]:
    """Keep the38 first attempts and append only explicitly registered processed retries."""
    rows = json.loads((DOCS / 'ENV_RESCORE_V2.json').read_text())['records']
    if len(rows) != 38 or len({r['name'] for r in rows}) != 38:
        message = 'First-attempt cohort must contain38 distinct assets'
        raise ValueError(message)
    result = [Candidate(r['name'], r['production_family'], r['family'], 1,
                        Path(r['candidate']), r['sha256'], Path(r['working_candidate']), r['working_sha256'])
              for r in rows]
    if retries:
        known = {r.name: r for r in result}
        for row in json.loads(retries.read_text())['records']:
            first = known[row['name']]
            attempt = row['attempt']
            if type(attempt) is not int or not 2 <= attempt <= 3:
                message = 'Retry attempt must retain historical numbering2 or3'
                raise ValueError(message)
            result.append(Candidate(first.name, first.production_family, first.family, attempt,
                                    Path(row['raw']), row['raw_sha256'],
                                    Path(row['working']), row['working_sha256'],
                                    Path(row['provenance']) if row.get('provenance') else None,
                                    row.get('provenance_sha256')))
    if len({(r.name, r.attempt) for r in result}) != len(result):
        message = 'Duplicate asset/attempt mapping'
        raise ValueError(message)
    return result


def cost_counts() -> tuple[int, int, int]:
    """Count actual generation records, including failed and unaccepted attempts."""
    historical = json.loads((DATA / 'accounting-final.json').read_text())['records']
    old = [r for r in historical if r['family'] in FAMILIES and r['category'] == 'production']
    if len(old) != 59:
        message = 'Historical environment cost ledger must retain all59 generations'
        raise ValueError(message)
    retry_records = list((DATA / 'trial2b/env-retries').glob('*_generation.json'))
    # Require every raw output to have a record; silently omitted pending calls undercount C1.
    raw = list((DATA / 'trial2b/env-retries').glob('*_raw.png'))
    recorded = {Path(json.loads(p.read_text())['raw']).resolve() for p in retry_records}
    if {p.resolve() for p in raw} != recorded:
        message = 'New retry raw outputs and generation records are not closed'
        raise ValueError(message)
    calm = list((DATA / 'trial2b/ground').glob('*/try*_raw.png'))
    return len(old), len(retry_records), len(calm)


def promote(candidate: Candidate, sidecar_json: str) -> None:
    """Copy verified working pixels and archive any historical accepted replacement."""
    if sha(candidate.working) != candidate.working_sha256:
        message = f'Working sprite changed: {candidate.working}'
        raise ValueError(message)
    destination = ROOT / 'art/style05/env' / candidate.production_family / (candidate.name + '.png')
    destination.parent.mkdir(parents=True, exist_ok=True)
    sidecar = destination.with_suffix('.json')
    if destination.exists() and sha(destination) != candidate.working_sha256:
        archive = ROOT / 'art/style05/env/history/trial2' / candidate.production_family
        archive.mkdir(parents=True, exist_ok=True)
        stem = candidate.name + '-' + sha(destination)[:12]
        shutil.copy2(destination, archive / (stem + '.png'))
        if sidecar.exists():
            shutil.copy2(sidecar, archive / (stem + '.json'))
    shutil.copy2(candidate.working, destination)
    _ = sidecar.write_text(sidecar_json)


def main() -> int:
    """Emit reviewable accounting; mutation requires the explicit --promote flag."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--reviews', required=True, type=Path)
    parser.add_argument('--semantic-qa', required=True, type=Path, action='append',
                        help='records: raw_sha256, passed boolean, evidence path + evidence_sha256')
    parser.add_argument('--retry-candidates', type=Path)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--promote', action='store_true')
    args = parser.parse_args()
    calibration = load_calibration(DOCS / 'JUDGE_V2_CALIBRATION.json')
    folds_ok = bool(calibration.leave_one_out) and len(calibration.leave_one_out) == len(calibration.anchors) and all(f.passed for f in calibration.leave_one_out)
    controls = json.loads((DOCS / 'ENV_RESCORE_V2.json').read_text())['negatives']
    controls_ok = len(controls) == 3 and all(sha(Path(r['candidate'])) == r['sha256'] and not judge(Path(r['candidate']), calibration, family='person').numeric_pass for r in controls)
    if not folds_ok or not controls_ok:
        message = 'Calibration does not pass actualLOO and negative controls'
        raise ValueError(message)
    reviews = read_reviews(args.reviews)
    semantics = [row for path in args.semantic_qa for row in json.loads(path.read_text())['records']]
    costs = cost_counts()
    rows, chosen = [], {}
    for candidate in read_candidates(args.retry_candidates):
        matches = [r for r in reviews if r.raw_sha256 == candidate.raw_sha256]
        review = matches[0] if len(matches) == 1 else None
        allowed = {a.sha256 for a in calibration.anchors if a.family == candidate.family}
        problem = review_problem(candidate, review, allowed) if review else 'MISSING_OR_DUPLICATE_FRESH_REVIEW'
        numeric = judge(candidate.raw, calibration, family=candidate.family).numeric_pass
        qa = [r for r in semantics if r['raw_sha256'] == candidate.raw_sha256]
        semantic = None
        if len(qa) == 1 and Path(qa[0]['evidence']).is_file() and sha(Path(qa[0]['evidence'])) == qa[0]['evidence_sha256']:
            semantic = qa[0]['passed'] is True
        passes = accepted(numeric, problem is None and review is not None and review.scores.passed)
        eligible = promotion_eligible(passes, semantic)
        rows.append({'name': candidate.name, 'family': candidate.production_family, 'attempt': candidate.attempt,
                     'raw_sha256': candidate.raw_sha256, 'numeric_pass': numeric, 'model_problem': problem,
                     'semantic_pass': semantic, 'accepted': passes, 'promotion_eligible': eligible})
        if eligible and review is not None and (candidate.name not in chosen or candidate.attempt < chosen[candidate.name][0].attempt):
            chosen[candidate.name] = (candidate, review)
    for candidate, _review in chosen.values():
        if sha(candidate.working) != candidate.working_sha256 or candidate.working.stat().st_size > 1_500_000:
            message = f'Working sprite identity or1.5MB cap failed: {candidate.working}'
            raise ValueError(message)
    first_pass = sum(r['accepted'] for r in rows if r['attempt'] == 1)
    accepted_count = len({r['name'] for r in rows if r['accepted']})
    payload = {'cohort': 38, 'first_pass': first_pass, 'accepted_after_retries': accepted_count,
               'promotion_eligible_assets': len(chosen),
               'acceptance_rule': 'numeric AND model only; semantic eligibility affects promotion separately',
               'c2_first_rate': first_pass / 38, 'c2_after_rate': accepted_count / 38,
               'historical_attempts': costs[0], 'new_retry_attempts': costs[1], 'calm_ground_attempts': costs[2],
               'c1_environment_only': (costs[0] + costs[1]) / accepted_count if accepted_count else None,
               'c1_including_calm_ground_replacement': sum(costs) / accepted_count if accepted_count else None,
               'historical_extra_attempts': 59 - 38, 'counterfactual_note': 'No subtraction from actual C1; historical excess21, not a fixed20 saving.',
               'ground_policy': 'Historical accepted ground is this cohort; calm replacement reviewed separately. Include calm cost when reporting final delivered portfolio.',
               'records': rows, 'promoted': bool(args.promote)}
    _ = args.output.write_text(json.dumps(payload, indent=2) + '\n')
    if args.promote:
        from art.pipeline2.accept_provenance_v2 import build_sidecar
        sidecars = [(candidate, build_sidecar(candidate, review, args.output))
                    for candidate, review in chosen.values()]
        for candidate, sidecar_json in sidecars:
            promote(candidate, sidecar_json)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
