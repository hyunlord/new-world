# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "scikit-image"]
# ///
# How to run: PYTHONPATH=. uv run art/pipeline5/prod3_reslice.py DATA_ROOT SUMMARY_JSON
"""Re-slice ten frozen missing looks and audit, but never replace, 26 accepted looks."""
from __future__ import annotations

import json
import sys
from collections.abc import Callable
from concurrent.futures import ProcessPoolExecutor
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Final, TypedDict

from art.pipeline5.attempt_integrity import digest
from art.pipeline5.prod1_process import evaluate
from art.pipeline5.prod1_records import JsonValue, ProductionRequest, SheetRecord
from art.pipeline5.prod1_storage import publish
from art.pipeline5.whole_checks import check_whole_path
from art.whole_figure_lab.prod1_boards import boards

REPO: Final = Path(__file__).resolve().parents[2]
ROUTE: Final = REPO / 'art/character/route_i'
MISSING: Final = (
    'adult_F_hide_wrap_bun', 'adult_M_fur_cloak_long_tied',
    'adult_M_fur_cloak_short', 'adult_M_hide_wrap_long_tied', 'adult_M_loincloth_bun',
    'child_F_fur_cloak_bun', 'child_F_fur_cloak_short', 'child_F_loincloth_bun',
    'child_F_loincloth_long_tied', 'child_F_loincloth_short',
)


class Identity(TypedDict):
    raw: str
    raw_sha256: str
    prompt_sha256: str
    attempt: int


@dataclass(frozen=True, slots=True)
class Source:
    look: str
    selection: str
    raw: Path
    prompt: Path
    attempt: int
    raw_sha256: str
    prompt_sha256: str


@dataclass(frozen=True, slots=True)
class Task:
    source: Source
    evidence: Path


def tree_hashes(root: Path) -> dict[str, str]:  # path-to-digest index
    """Hash every existing file, including sidecars and manifest."""
    return {str(p.relative_to(root)): digest(p) for p in sorted(root.rglob('*')) if p.is_file()}


def read_identity(path: Path, *, decode: Callable[[str], JsonValue] = json.loads) -> Identity:
    """Parse the persisted source identity fields at the JSON boundary."""
    match decode(path.read_text()):
        case {'raw': str(raw), 'raw_sha256': str(raw_sha256),
              'prompt_sha256': str(prompt_sha256), 'attempt': int(attempt)} if type(attempt) is int:
            return Identity(raw=raw, raw_sha256=raw_sha256,
                            prompt_sha256=prompt_sha256, attempt=attempt)
        case _:
            message = f'invalid persisted source identity: {path}'
            raise ValueError(message)


def frozen_source(look: str, raw: Path, record_path: Path, selection: str) -> Source:
    """Require preserved raw and prompt identities before evaluation."""
    record = read_identity(record_path)
    source = Source(look, selection, raw, raw.with_name('prompt.txt'), record['attempt'],
                    digest(raw), digest(raw.with_name('prompt.txt')))
    if (source.raw_sha256, source.prompt_sha256) != (record['raw_sha256'], record['prompt_sha256']):
        message = f'frozen source identity mismatch: {raw}'
        raise ValueError(message)
    return source


def request_for(task: Task) -> ProductionRequest:
    """Use the existing evaluator without reserving or generating a new attempt."""
    s = task.source
    return ProductionRequest(s.raw, s.look, s.look.split('_')[0], s.attempt,
                             s.prompt, ROUTE, task.evidence, 'ART-PROD3 frozen-source alpha>=16 evaluation')


def run_one(task: Task) -> SheetRecord:
    """Record a complete evaluation in an immutable external directory."""
    request = request_for(task)
    request.diagnostic.mkdir(parents=True, exist_ok=False)
    record = evaluate(request)
    _ = (request.diagnostic / 'sheet_checks.json').write_text(json.dumps(asdict(record), indent=2) + '\n')
    print(f'{task.evidence.name}: {record.look} {record.passing_panels}/10', flush=True)
    return record


def main() -> int:
    """Preserve old acceptance, publish new passes, and retain every comparison."""
    if len(sys.argv) != 3:
        print('DATA_ROOT SUMMARY_JSON')
        return 2
    data, summary = map(Path, sys.argv[1:])
    run = data / 'reslice'
    run.mkdir(parents=True, exist_ok=False)
    accepted = tuple(sorted(p.parent.name for p in ROUTE.glob('*/MANIFEST.json')))
    if len(accepted) != 26 or set(accepted) & set(MISSING):
        message = 'expected exactly 26 prior accepted looks and ten disjoint missing looks'
        raise ValueError(message)
    baseline = {look: tree_hashes(ROUTE / look) for look in accepted}
    _ = (run / 'ACCEPTED_BEFORE.json').write_text(json.dumps(baseline, indent=2) + '\n')
    sources: list[Source] = []
    for look in MISSING:
        is_new = look == 'child_F_fur_cloak_bun'
        root = Path('/Users/rexxa/work/artprod2-data/new-production' if is_new
                    else '/Users/rexxa/work/artprod1-data/production') / look / 'try1'
        sources.append(frozen_source(look, root / 'raw.png', root / 'sheet_checks.json', 'new1' if is_new else 'old1'))
    for look in accepted:
        manifest = read_identity(ROUTE / look / 'MANIFEST.json')
        sources.append(frozen_source(look, Path(manifest['raw']), ROUTE / look / 'MANIFEST.json', 'accepted_manifest'))
    frozen = {str(path): digest(path) for s in sources for path in (s.raw, s.prompt)}
    _ = (run / 'SOURCES_BEFORE.json').write_text(json.dumps(frozen, indent=2) + '\n')
    tasks = tuple(Task(s, run / ('missing10' if s.look in MISSING else 'accepted26')) for s in sources)
    with ProcessPoolExecutor(max_workers=4) as executor:
        missing_results = tuple(executor.map(run_one, tasks[:10]))
        print(f'MISSING10 COMPLETE: {sum(r.passed for r in missing_results)}/10 passes; ' +
              f'adults {sum(r.passed and r.age == "adult" for r in missing_results)}/5', flush=True)
        results = missing_results + tuple(executor.map(run_one, tasks[10:]))
    details: list[dict[str, JsonValue]] = []
    diffs: list[dict[str, JsonValue]] = []
    for task, result in zip(tasks, results, strict=True):
        frames: list[JsonValue] = []
        for frame in result.frames:
            key = f'{frame.facing}/{frame.frame}'
            output = Path(result.diagnostics) / (key + '.png')
            sidecar = output.with_suffix('.json')
            normalization = json.loads(sidecar.read_text())['normalization'] if sidecar.exists() else None
            original = ROUTE / result.look / (key + '.png')
            same = digest(original) == digest(output) if original.exists() and output.exists() else None
            if result.look in accepted and same is not True:
                diffs.append({'look': result.look, 'frame': key,
                              'before_sha256': digest(original),
                              'after_sha256': digest(output) if output.exists() else None})
            frames.append({'frame': key, 'structural_pass': frame.checks.passed,
                           'panel_pass': frame.judge.numeric_pass if frame.judge else False,
                           'findings': list(frame.checks.findings), 'error': frame.error,
                           'normalization': normalization, 'accepted_png_byte_identical': same,
                           'preserved_asset_fresh_checks': asdict(check_whole_path(original, result.age, frame.frame))
                           if original.exists() else None})
        details.append({'look': result.look, 'selection': task.source.selection,
                        'raw': str(task.source.raw), 'raw_sha256': result.raw_sha256,
                        'prompt_sha256': result.prompt_sha256, 'passed': result.passed,
                        'passing_panels': result.passing_panels,
                        'structural_passes': sum(f.checks.passed for f in result.frames),
                        'panel_judge_passes': sum(f.judge is not None and f.judge.numeric_pass for f in result.frames),
                        'diagnostics': result.diagnostics, 'frames': frames})
    originals_unchanged = all(digest(Path(path)) == sha for path, sha in frozen.items())
    accepted_unchanged = baseline == {look: tree_hashes(ROUTE / look) for look in accepted}
    if not originals_unchanged or not accepted_unchanged:
        message = 'preservation invariant failed before publication'
        raise ValueError(message)
    for task, result in zip(tasks, results, strict=True):
        if result.look in MISSING and result.passed:
            if (ROUTE / result.look).exists():
                message = f'refuse to replace accepted look: {result.look}'
                raise ValueError(message)
            publish(request_for(task), result)
    manifests = sorted(ROUTE.glob('*/MANIFEST.json'))
    adult_count = sum(p.parent.name.startswith('adult_') for p in manifests)
    missing = sorted(set(MISSING) - {p.parent.name for p in manifests})
    adopted = len(manifests) >= 32 and adult_count >= 17
    adult_only = not adopted and adult_count >= 17
    payload = {'new_generations': 0, 'missing_evaluations': 10, 'accepted_evaluations': 26,
               'newly_accepted': len(manifests) - 26, 'accepted_count': len(manifests),
               'adult_count': adult_count, 'child_count': len(manifests) - adult_count,
               'missing': missing, 'adopted': adopted, 'adult_only': adult_only,
               'accepted_rerun_passes': sum(r.passed for r in results if r.look in accepted),
               'accepted_png_byte_identical': not diffs, 'accepted_png_diffs': diffs,
               'accepted_original_tree_unchanged': baseline == {look: tree_hashes(ROUTE / look) for look in accepted},
               'source_originals_unchanged': originals_unchanged, 'looks': details,
               'baseline_hashes': str(run / 'ACCEPTED_BEFORE.json'),
               'source_hashes': str(run / 'SOURCES_BEFORE.json')}
    summary.parent.mkdir(parents=True, exist_ok=True)
    _ = summary.write_text(json.dumps(payload, indent=2) + '\n')
    _ = (run / 'RESULTS.json').write_text(json.dumps(payload, indent=2) + '\n')
    root_manifest = {'schema': 1, 'status': 'ADOPTED' if adopted else 'ADULT_SET_ADOPTED' if adult_only else 'CANDIDATE_NOT_ADOPTED',
                'pass': 'ART-PROD3', 'template': '../TEMPLATE_V3.md',
                'accepted_looks': [{'look': p.parent.name, 'manifest': str(p.relative_to(ROUTE)), 'sha256': digest(p)} for p in manifests],
                'accepted_count': len(manifests), 'adult_count': adult_count,
                'child_count': len(manifests) - adult_count, 'missing_looks': missing,
                'readability': 'PROD2 R1 PASS; source and reviewed accepted assets preserved',
                'generation_count': 0, 'prior_prod2_generation_count': 27,
                'visibility_threshold': 16, 'verification': '../../../docs/v8/artprod3/RESLICE_RESULTS.json'}
    _ = (ROUTE / 'MANIFEST.json').write_text(json.dumps(root_manifest, indent=2) + '\n')
    boards(ROUTE, REPO / 'art/character/boards')
    print(json.dumps({key: value for key, value in payload.items() if key not in ('looks', 'accepted_png_diffs')}))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
