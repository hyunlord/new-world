# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -B -m unittest test_generic_whole_gate_comparison
"""BASE-relative comparisons for original unpaired shell probe producers."""
import json
from collections.abc import Callable
from pathlib import Path

import read_summary_normalise as normalizer
from compare_chronicles import compare as compare_chronicles
from generic_baseline_run import SUPPORTED as BASELINES
from generic_baseline_run import raw_digests
from generic_declarations import Declarations
from generic_gate import (
    boundary,
    document,
    require,
    validate_declared_fixture,
)
from generic_process import file_hash
from mux2_gate_inputs import Json, Node
from read_summary_normalise import first_difference

from generic_whole_gate_f10 import bound_output
from generic_whole_gate_probe import ProbeEvidence

_normalise: Callable[[Json, Json, str | None], tuple[Json, Json, list[str]]] = normalizer.normalise_pair


def _anchors(stage: Node, root: Path) -> None:
    """Check every retained output used by the comparator, including namespace."""
    files = {Path(name): value for name, value in stage.field('observer_artifacts').mapping().items()
             if Path(name).is_relative_to(root)}
    require(bool(files), str(root), 'missing original comparison outputs')
    require({p for p in root.rglob('*') if p.is_file()} == files.keys(), str(root), 'comparison output namespace changed')
    for path, digest in files.items():
        require(not path.is_symlink() and file_hash(path) == digest, str(path), 'original comparison bytes changed')


@boundary
def compare(session: Node, declarations: Declarations, base_stage: Node, candidate_stage: Node,
            base: ProbeEvidence, candidate: ProbeEvidence, safety: tuple[Json, ...]) -> dict[str, Json]:
    """One committed normalizer; intentional changes never waive ordinary safety."""
    fixture = base.plan.fixture_id
    require(candidate.plan.fixture_id == fixture, '/comparison', 'wrong pair')
    configured = session.field('acceptance').field('normalizer')
    module = Path(normalizer.__file__).resolve(strict=True)
    configured.field('sha256').equals(file_hash(module))
    require(file_hash(Path(configured.field('path').text())) == file_hash(module),
            '/normalizer', 'selected and loaded harness normalizer bytes differ')
    sources = (base_stage, candidate_stage)
    evidence = (base, candidate)
    for stage, item in zip(sources, evidence, strict=True):
        _anchors(stage, item.plan.out)
    a = document(bound_output(base_stage, base.plan.out / 'summary.json')).value
    b = document(bound_output(candidate_stage, candidate.plan.out / 'summary.json')).value
    try:
        left, right, excluded = _normalise(a, b, declarations.base)
    except ValueError as error:
        require(False, '/normalizer', str(error))
        raise AssertionError('unreachable rejected normalization')
    for value, item in ((a, base), (b, candidate)):
        summary = Node(value)
        if fixture in BASELINES:
            summary.field('schema_version').equals(13)
            require(item.plan.argv is not None, fixture, 'missing planned scene')
            if item.plan.argv is None:
                raise AssertionError('validated command')
            summary.field('scene').equals(item.plan.argv[item.plan.argv.index('--scene') + 1])
            require(tuple(row.field('seed').value for row in summary.field('runs').array()) == item.plan.seeds,
                    fixture, 'baseline summary seed coverage differs')
        else:
            summary.field('scene').equals('colony')
            require(tuple(row.field('seed').value for row in summary.field('seeds').array()) == item.plan.seeds,
                    fixture, 'colony summary seed coverage differs')
            summary.field('days').equals(item.worlds[0].observation.terminal_tick // 1440)
    difference = first_difference(left, right)
    raw: dict[str, Json] = {}
    additional: list[tuple[Node, Path]] = []
    if fixture in BASELINES:
        require(difference is None, fixture, f'protected baseline summary differs: {difference}')
        before, after = raw_digests(base.plan), raw_digests(candidate.plan)
        require(before == after, fixture, 'protected baseline raw sample/death bytes differ')
        raw['raw_sample_digests'] = dict(before)
        if fixture == 'identity.baseline_samples':
            folders = tuple(item.plan.out.parent / 'chronicle' for item in evidence)
            for stage, folder in zip(sources, folders, strict=True):
                _anchors(stage, folder)
                additional.append((stage, folder))
            try:
                raw['chronicle'] = compare_chronicles(*folders)
            except ValueError as error:
                require(False, fixture, f'protected baseline chronicle differs: {error}')
    elif fixture == 'knowledge.f10_injected':
        require(difference is None, fixture, f'fault control summary differs: {difference}')
    else:
        if fixture in {'replay.generate', 'replay.recorded'}:
            seeds = base.plan.seeds if fixture == 'replay.generate' else (5,)
            for seed in seeds:
                paths = tuple(item.plan.out.parent / f'log/colony_s{seed}_commands.json' for item in evidence)
                additional.extend((stage, path.parent) for stage, path in zip(sources, paths, strict=True))
                command_bytes = tuple(bound_output(stage, path) for stage, path in zip(sources, paths, strict=True))
                if command_bytes[0] != command_bytes[1]:
                    difference = difference or f'/generated_input/colony_s{seed}_commands.json: recorded command bytes differ'
        for row in safety:
            admission = declarations.validator_fields(fixture)
            admission.update(semantic_equal=difference is None, safety=row)
            validate_declared_fixture(json.dumps(admission))
        require(bool(safety), fixture, 'no ordinary safety rows')
    for stage, item in zip(sources, evidence, strict=True):
        _anchors(stage, item.plan.out)
    for stage, folder in additional:
        _anchors(stage, folder)
    return {'fixture': fixture, 'semantic_equal': difference is None, 'difference': difference,
            'excluded_paths': list(excluded), **raw}
