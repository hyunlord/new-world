#!/usr/bin/env python3
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import sys

BASE = Path('/Users/rexxa/work/p17-data/b-evaluation')
REPO = Path('/Users/rexxa/work/new-world-pass4')
DOC = REPO / 'docs/revitalize/probe/p17-b_eval'
PROOF = REPO / 'docs/revitalize/verification/p17-b_eval'
PRODUCER = '47b7244052fcf616bf2efbe7e3f3f9e11aa6fc21'
PREREG = '43f22e81f120a755de33712b02a62469c67ab840'


def sha(path: Path) -> str:
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def main() -> None:
    receipt_path = Path(sys.argv[1])
    assert not receipt_path.exists(), 'refuse receipt overwrite'
    log = (BASE / 'RUNLOG.txt').read_text().splitlines()
    exit_code = int((BASE / 'evaluate.exit').read_text())
    assert exit_code in (0, 1) and log[-1].endswith(f'workloads end exit={exit_code}')
    checked = []
    for line in (BASE / 'raw_sha256.txt').read_text().splitlines():
        expected, name = line.split(None, 1)
        path = (BASE / name).resolve()
        assert path.is_relative_to(BASE) and path.is_file(), name
        assert sha(path) == expected, name
        checked.append({'path': str(path), 'sha256': expected})
    assert len({row['path'] for row in checked}) == len(checked)
    cohorts = []
    for scene in ('game_v22', 'game_v21'):
        assert (BASE / f'{scene}.exit').read_text().strip() == '0'
        source = BASE / scene / 'summary.json'
        summary = json.loads(source.read_text())
        assert source.read_bytes() == (DOC / scene / 'summary.json').read_bytes()
        assert summary['scene'] == scene and summary['determinism_checked'] is True
        assert summary['seeds'] == {'start': 18000, 'end_exclusive': 18032}
        assert set(map(int, summary['days'])) == set(range(1, 121))
        assert sorted(row['seed'] for row in summary['runs']) == list(range(18000, 18032))
        assert all(len(row['alive_daily']) == 120 for row in summary['runs'])
        for seed in range(18000, 18032):
            meta = json.loads((BASE / scene / f'{scene}_s{seed}_trace_meta.json').read_text())
            assert (meta['scene'], meta['seed'], meta['end_tick'], meta['git_sha']) == (scene, seed, 172800, PRODUCER)
            assert meta['born_event_misses'] == meta['component_mismatches'] == meta['pass17']['observation_misses'] == 0
            if scene == 'game_v22':
                family = json.loads((BASE / scene / f'{scene}_s{seed}_family_meta.json').read_text())
                assert (family['scene'], family['seed'], family['end_tick'], family['git_sha']) == (scene, seed, 172800, PRODUCER)
                assert family['schema_version'] == 17 and family['observation_misses'] == 0
                assert family['births_total'] == family['source_births_total']
                assert family['departures_total'] == family['source_departures_total']
        cohorts.append({'scene': scene, 'worlds': 32, 'days': 120, 'deterministic_twins': 32, 'summary_sha256': sha(source)})
    report = json.loads((DOC / 'eval.json').read_text())
    assert (report['schema_version'], report['stage'], report['mode']) == (17, 'b', 'evaluation')
    assert set(report['rules']) == {f'F{n}' for n in range(1, 9)}
    assert all(value in ('PASS', 'FAIL') for value in report['rules'].values())
    verdict = 'FAIL' if 'FAIL' in report['rules'].values() else 'PASS'
    assert report['verdict'] == verdict and exit_code == int(verdict == 'FAIL')
    prereg = json.loads((BASE / 'preregistration.json').read_text())
    assert prereg['preregistration_commit'] == PREREG and prereg['producer_commit'] == PRODUCER
    start = next(line.split()[0] for line in log if line.endswith('game_v22 start'))
    launch_epoch = datetime.fromisoformat(start.replace('Z', '+00:00')).timestamp()
    margin = launch_epoch - prereg['preregistration_commit_epoch']
    assert margin > 0
    for name in ('raw_sha256.txt', 'RUNLOG.txt'):
        assert (BASE / name).read_bytes() == (PROOF / name).read_bytes()
    receipt = {'status': 'PASS', 'verified_at': datetime.now(timezone.utc).isoformat(),
               'scope': 'Terminal original registered analysis, raw hashes, cohort and trace provenance; not a metric acceptance pass.',
               'producer': PRODUCER, 'preregistration_commit': PREREG, 'preregistration_to_first_launch_seconds': margin,
               'raw_files_verified': len(checked), 'hash_mismatches': 0, 'cohorts': cohorts,
               'trace_misses': 0, 'family_misses': 0, 'rules': report['rules'], 'verdict': verdict,
               'eval_json_sha256': sha(DOC / 'eval.json'), 'eval_md_sha256': sha(DOC / 'eval.md'),
               'raw_manifest_sha256': sha(BASE / 'raw_sha256.txt'), 'raw_inputs': checked}
    with receipt_path.open('x') as stream:
        json.dump(receipt, stream, indent=2)
        stream.write('\n')
    print(json.dumps({key: value for key, value in receipt.items() if key != 'raw_inputs'}, indent=2))


if __name__ == '__main__':
    main()
