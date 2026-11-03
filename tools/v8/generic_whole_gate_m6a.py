# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -B -m unittest test_generic_whole_gate_m6a
"""M6a sidecar/report joins over original B paired-producer evidence."""
from pathlib import Path

from generic_fixture_registry import Invocation
from generic_gate import document, integer, require, text
from generic_m6a_observations import (
    M6aObservation,
    paired_safety,
    paired_selected_safety,
    parse_fixed,
    parse_matched_companion,
    parse_selected_pack,
)
from generic_m6a_run import DURATION, FIXED, MAXIMUM
from generic_observations import ProbeWorld
from mux2_gate_inputs import Json

from generic_whole_gate_paired import Closure, Paired, recheck


def _raw(original: Closure, path: Path) -> str:
    require(str(path) in dict(original.files), str(path), 'unbound M6a output')
    return path.read_text()


def observe(original: Closure, plan: Invocation) -> M6aObservation:
    """Reuse actual schema/horizon parsers; no fake native result or max-time rounding."""
    recheck(original)
    folder = plan.out / 'observed/gate_observations'
    if plan.fixture_id == FIXED:
        scene, report = 'm6a', 'summary.json'
    else:
        scene, report = 'm6a-data-only', 'data-only.json'
    sidecar = folder / f'{scene}_s{plan.seeds[0]}_primary.json'
    require({name for name, _ in original.files if Path(name).parent == folder} == {str(sidecar)},
            str(folder), 'missing or extra M6a world')
    raw = _raw(original, sidecar)
    report_raw = _raw(original, plan.out / 'observed' / report)
    if plan.fixture_id == FIXED:
        return parse_fixed(raw, report_raw, ProbeWorld(scene, plan.seeds[0], 'primary', DURATION))
    return parse_selected_pack(raw, report_raw, MAXIMUM)


def evaluate(evidence: Paired) -> tuple[Json, ...]:
    """Keep actual primary oracles and the additional BASE matched-horizon child."""
    require(evidence.step in {47, 48}, '/m6a', 'wrong paired component')
    original = evidence.closure
    recheck(original)
    rows: list[Json] = []
    for base in (p for p in evidence.plans if p.arm == 'base'):
        candidate = next(p for p in evidence.plans if p.arm == 'candidate' and
                         (p.fixture_id, p.seeds, p.repeat) == (base.fixture_id, base.seeds, base.repeat))
        left, right = observe(original, base), observe(original, candidate)
        if base.fixture_id == FIXED:
            safety = paired_safety(left, right)
        else:
            folder = base.out.parent / 'matched-companion'
            native = document(_raw(original, folder / 'process.json'))
            native.field('status').equals('PASS')
            native.field('native_exit').equals(0)
            native.field('errors').equals([])
            require(integer(native.field('pid')) > 0, '/m6a', 'missing companion native PID')
            native.field('cwd').equals(str(base.cwd))
            native.field('output_root').equals(str(folder))
            require(base.argv is not None, '/m6a', 'missing registered command')
            if base.argv is None:
                raise AssertionError('validated command')
            native.field('argv').equals([base.argv[0], 'm6a-data-only', '--out', str(folder / 'observed'),
                '--matched-horizon-ticks', str(right.world.terminal_tick), '--gate-observations'])
            product = evidence.receipt.field('products').field('base')
            native.field('executable').equals(product.field('executable').value)
            for key in ('expected_executable_sha256', 'before_executable_sha256', 'after_executable_sha256'):
                native.field(key).equals(product.field('executable_sha256').value)
            native.field('environment').field('WORLDSIM_GATE_OBSERVER').equals('1')
            native.field('environment').field('WORLDSIM_GIT_SHA').equals(evidence.receipt.field('base').value)
            for stream in ('stdout', 'stderr'):
                native.field(stream + '_sha256').equals(dict(original.files)[str(folder / (stream + '.log'))])
            sidecar = folder / 'observed/gate_observations/m6a-data-only-companion_s0_primary.json'
            require({name for name, _ in original.files if Path(name).parent == sidecar.parent} == {str(sidecar)},
                    str(sidecar.parent), 'missing or extra companion world')
            companion = parse_matched_companion(_raw(original, sidecar),
                _raw(original, folder / 'observed/data-only-companion.json'), right.world.terminal_tick, MAXIMUM)
            component = document(_raw(original, original.root / 'component/m6a.json'))
            executions = [row for row in component.field('executions').array()
                          if text(row.field('id')) == base.id + '/matched-companion']
            require(len(executions) == 1, '/m6a', 'missing or duplicate companion execution')
            executions[0].field('status').equals('PASS')
            reference = executions[0].field('receipt')
            reference.field('path').equals(str(folder / 'process.json'))
            reference.field('sha256').equals(dict(original.files)[str(folder / 'process.json')])
            safety = paired_selected_safety(left, right, companion)
        rows.append({'fixture': base.fixture_id, 'seeds': list(base.seeds), 'repeat': base.repeat, 'safety': safety})
    recheck(original)
    return tuple(rows)
