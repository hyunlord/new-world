"""Source-qualified library mechanisms under an authenticated whole-stage Bash wait."""
from pathlib import Path
from typing import Final

from gate_test_summary import parse_log, validate_cases
from generic_gate import boundary, integer, require, text
from generic_library_siblings import LibraryStep, recipe
from generic_process import file_hash
from mux2_gate_inputs import Json, Node

# Frozen source classifications, not an engine-constructor census.
SOURCE_FILES: Final = {
    42: ('rust/crates/sim-systems/src/lib.rs', 'tools/v8/core_gate_checks.sh'),
    43: ('rust/crates/sim-systems/src/lib.rs', 'rust/crates/sim-systems/src/runtime/mind/system_tests.rs', 'tools/v8/core_gate_checks.sh'),
    49: ('rust/crates/sim-core/src/lib.rs', 'rust/crates/sim-core/src/colony/diagnosis.rs'),
}
SOURCE_PINS: Final = {'rust/crates/sim-systems/src/lib.rs': '62eabd404f524dd7f4529dcff35874bd0cfa748e34223f92a569584f90abb60b', 'tools/v8/core_gate_checks.sh': '481d2c10e89cee95709e96c70833287ac99bd5a2b29b1fb7b01057cd4fa6e652', 'rust/crates/sim-systems/src/runtime/mind/system_tests.rs': 'ae71c75a9a7c87a50ba3fdc81d6ae5fe8ae7525e5db4433d40370a7bb0d8421f', 'rust/crates/sim-core/src/lib.rs': '248f9587aa8576fda1b769a0609f1b921a8b5f13244292d78a1b23f7e45bfe2c', 'rust/crates/sim-core/src/colony/diagnosis.rs': 'ddbf149bce0140faaa16c531be091496cc9b027e667303d72b823fd3f6a71c7a'}


def selected_output(raw: str, step: int) -> tuple[str, ...]:
    """Retain original mechanism assertions and reject zero-selection or extra cases."""
    chosen = recipe(LibraryStep(step))
    summary = parse_log(raw)
    validate_cases(summary, chosen.expected, ())
    require(summary.failed == summary.ignored == summary.measured == 0 and summary.passed == len(chosen.expected),
            '/native', 'selected mechanism did not pass exactly')
    running = [line.strip() for line in raw.splitlines() if line.strip().startswith('running ')]
    require(running == [f'running {len(chosen.expected)} '+('test' if len(chosen.expected) == 1 else 'tests')],
            '/native', 'missing or extra native running section')
    return chosen.expected


@boundary
def evaluate_library(session: Node, base_stage: Node, candidate_stage: Node) -> dict[str, Json]:
    from generic_whole_gate_aggregate import LIMIT, aggregate_output
    step = integer(base_stage.field('step'))
    require(step in SOURCE_FILES, '/step', 'unsupported source-classified library')
    candidate_stage.field('step').equals(step)
    chosen = recipe(LibraryStep(step))
    evidence: dict[str, Json] = {}
    for arm, stage in (('base', base_stage), ('candidate', candidate_stage)):
        stage.field('arm').equals(arm)
        raw, pid = aggregate_output(session, stage)
        product = session.field('arms').field(arm)
        root = Path(text(product.field('root')))
        for source in SOURCE_FILES[step]:
            product.field('source_inputs').field(source).equals(SOURCE_PINS[source])
            require(file_hash(root/source) == SOURCE_PINS[source], source, 'unregistered source classification')
        cases = selected_output(raw, step)
        if step == 42:
            require(raw.splitlines().count('BASELINE_ISOLATION source PASS') == 1,
                    '/scanner', 'original source scanner did not succeed exactly once')
        evidence[arm] = {'aggregate_pid': pid, 'aggregate_exit': 0, 'selected': list(cases),
                         'native_child_pid': None, 'native_child_wait': 'NOT_INDEPENDENTLY_RECORDED'}
    return {'status': 'PASS', 'safety': [], 'coverage': [{
        'step': step, 'scope': chosen.source_classification, 'selected': list(chosen.expected),
        'source_described_lit_branches': list(chosen.source_described_lit_branches),
        'branch_observation': 'SOURCE_ASSERTIONS_ONLY', 'ordinary_population_floor': 'NOT_APPLICABLE',
        'attribution': evidence}], 'unsupported': [], 'qualification': LIMIT}
