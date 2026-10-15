from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import subprocess
import sys

REPO = Path('/Users/rexxa/work/new-world-pass4')
DATA = Path('/Users/rexxa/work/p17-data')
PROOF = REPO / 'docs/revitalize/verification/p17-f3_correction'


def digest(path: Path) -> str:
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def now() -> str:
    return datetime.now(timezone.utc).isoformat()


def main() -> int:
    head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=REPO, text=True).strip()
    assert head.startswith('51e1629a')
    assert not subprocess.check_output(['git', 'diff', '--name-only', '--', 'rust', 'tools'], cwd=REPO)
    assert not subprocess.check_output(['git', 'diff', '--name-only', '47b72440', '--', 'rust'], cwd=REPO)
    old_manifest = json.loads((REPO / 'docs/revitalize/verification/p17-b_prereg/evaluator_sources.json').read_text())
    source_hashes = {name: digest(REPO / name) for name in old_manifest['files_sha256']}
    sources = {'evaluator_commit': head, 'simulation_producer_commit': '47b7244052fcf616bf2efbe7e3f3f9e11aa6fc21',
               'created_at': now(), 'files_sha256': source_hashes,
               'original_evaluator_manifest_sha256': digest(REPO / 'docs/revitalize/verification/p17-b_prereg/evaluator_sources.json')}
    with (PROOF / 'evaluator_sources.json').open('x') as stream:
        json.dump(sources, stream, indent=2)
        stream.write('\n')
    for mode, raw, output in (
        ('tuning', DATA / 'b-tuning/r1', REPO / 'docs/revitalize/probe/p17-b_tuning/r1-f3-corrected'),
        ('evaluation', DATA / 'b-evaluation', REPO / 'docs/revitalize/probe/p17-b_eval'),
    ):
        receipt_path = PROOF / (mode + '_replay.json')
        assert not receipt_path.exists()
        if mode == 'tuning':
            output.mkdir(exist_ok=False)
        else:
            for name in ('eval.json', 'eval.md'):
                assert (output / name).read_bytes() == (output / 'original_registered' / name).read_bytes()
        input_hashes = {}
        for line in (raw / 'raw_sha256.txt').read_text().splitlines():
            expected, relative = line.split(None, 1)
            path = (raw / relative).resolve()
            assert path.is_relative_to(raw) and digest(path) == expected
            input_hashes[str(path)] = expected
        command = [sys.executable, 'tools/revitalize/evaluate_pass17.py', '--stage', 'b',
                   '--candidate', str(raw / 'game_v22'), '--baseline', str(raw / 'game_v21'),
                   '--out-json', str(output / 'eval.json'), '--out-md', str(output / 'eval.md')]
        if mode == 'tuning':
            command.append('--tuning')
        else:
            command.extend(['--identity-json', str(REPO / 'docs/revitalize/verification/p17-5_b_identity/identity.json'),
                            '--perf-json', str(REPO / 'docs/revitalize/perf/p17-b/perf.json')])
        started = now()
        with (PROOF / (mode + '_replay.log')).open('x') as stream:
            result = subprocess.run(command, cwd=REPO, stdout=stream, stderr=subprocess.STDOUT, check=False)
        finished = now()
        report = json.loads((output / 'eval.json').read_text())
        assert result.returncode in (0, 1) and report['verdict'] in ('PASS', 'FAIL', 'PENDING')
        assert result.returncode == int(report['verdict'] == 'FAIL')
        assert all(digest(REPO / name) == expected for name, expected in source_hashes.items())
        assert all(digest(Path(name)) == expected for name, expected in input_hashes.items())
        receipt = {'status': 'VERIFIED', 'mode': mode, 'evaluator_commit': head,
                   'simulation_producer_commit': sources['simulation_producer_commit'],
                   'methodology': 'Post-launch correction of both F3 populations; retained data only; original analyses preserved.',
                   'command': command, 'started_at': started, 'finished_at': finished,
                   'exit_code': result.returncode, 'verdict': report['verdict'], 'rules': report['rules'],
                   'input_hashes_verified_before_and_after': input_hashes,
                   'source_hashes_verified_before_and_after': source_hashes,
                   'output_sha256': {str(output / name): digest(output / name) for name in ('eval.json', 'eval.md')}}
        with receipt_path.open('x') as stream:
            json.dump(receipt, stream, indent=2)
            stream.write('\n')
        print(json.dumps({key: value for key, value in receipt.items() if not key.endswith('_and_after')}), flush=True)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
