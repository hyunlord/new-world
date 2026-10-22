# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scikit-image"]
# ///
# How to run: uv run --with pillow --with numpy --with scikit-image python -m art.pipeline2.retry_numeric_v2
"""Inventory existing attempts for failed assets; never generate or reset try counts."""
import hashlib
import json
from dataclasses import asdict
from pathlib import Path
from typing import Final

from art.pipeline2.judge import judge, load_calibration

ROOT: Final = Path(__file__).resolve().parents[2]
DATA: Final = Path('/Users/rexxa/work/arttrial2-data')
DOCS: Final = ROOT / 'docs/v8/arttrial2'


def main() -> None:
    """Verify sidecar attempt identities and score every already-existing raw retry."""
    calibration = load_calibration(DOCS / 'JUDGE_V2_CALIBRATION.json')
    first = json.loads((DOCS / 'ENV_RESCORE_V2.json').read_text())['records']
    records = []
    for candidate in first:
        if candidate['numeric_pass']:
            continue
        family, name = candidate['production_family'], candidate['name']
        directory = DATA / family
        attempts = []
        for sidecar in sorted(directory.glob(name + '_try[123].json')):
            detail = json.loads(sidecar.read_text())
            attempt = detail.get('try_count', detail.get('try', int(sidecar.stem[-1])))
            raw = directory / f'{name}_try{attempt}_raw.png'
            digest = hashlib.sha256(raw.read_bytes()).hexdigest()
            expected = detail.get('raw_sha256', detail.get('source_sha256'))
            if expected is not None and expected != digest:
                message = f'Raw provenance hash mismatch: {sidecar}'
                raise ValueError(message)
            verdict = judge(raw, calibration, family=candidate['family'])
            attempts.append({'try': attempt, 'sidecar': str(sidecar), 'sidecar_sha256':
                             hashlib.sha256(sidecar.read_bytes()).hexdigest(),
                             'expected_raw_sha256': expected,
                             'verdict': asdict(verdict)})
        used = max(a['try'] for a in attempts)
        records.append({'name': name, 'family': family, 'attempts_consumed': used,
                        'generation_ledger': str(directory / 'LEDGER.json'),
                        'existing_numeric_pass_tries': [a['try'] for a in attempts
                                                       if a['verdict']['numeric_pass']],
                        'next_permitted_try': used + 1 if used < 3 else None,
                        'remaining_generation_attempts': max(0, 3 - used),
                        'action': 'Await fresh model scores; do not generate automatically',
                        'attempts': attempts})
    result = {'scope': 'Eight first-attempt numeric failures; existing generations only',
              'rule': 'First attempt counts as1; generation cap3 across TRIAL2 and TRIAL2b',
              'records': records}
    _ = (DOCS / 'ENV_RETRY_NUMERIC_V2.json').write_text(json.dumps(result, indent=2) + '\n')
    for record in records:
        print(record['name'], 'used', record['attempts_consumed'], 'numeric_pass_tries',
              record['existing_numeric_pass_tries'], 'next', record['next_permitted_try'])


if __name__ == '__main__':
    main()
