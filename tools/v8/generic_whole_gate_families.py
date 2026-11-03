# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -B -m unittest test_generic_whole_gate_observations
"""Join source-selected native/UI families without treating subsets as complete."""
from dataclasses import dataclass
from pathlib import Path

from generic_gate import GateContractError, document, require
from generic_process import file_hash
from mux2_gate_inputs import EvidenceError, Json, Node


@dataclass(frozen=True, slots=True)
class Families:
    safety: tuple[Json, ...]
    coverage: tuple[Json, ...]
    errors: tuple[str, ...]
    incomplete: tuple[str, ...]
    completed: tuple[str, ...]


def evaluate(session: Node, steps: dict[tuple[str, int], Node], *, descriptor: int | None = None) -> Families:
    """Only original source-selected occurrences execute validators, never workloads."""
    safety: list[Json] = []
    coverage: list[Json] = []
    errors: list[str] = []
    incomplete: list[str] = []
    completed: list[str] = []
    auxiliary = Path(session.field('output').text()) / 'auxiliary-index/smoke.json'
    if auxiliary.exists():
        from generic_whole_gate_ui_smoke_join import evaluate_family as smoke
        try:
            digest = file_hash(auxiliary)
            index = document(auxiliary.read_text())
            index.field('root').equals(str(Path(session.field('output').text()) / 'auxiliaries/smoke'))
            index.field('code').equals(0)
            safety.extend(smoke(session, index.field('anchor'), descriptor))
            require(file_hash(auxiliary) == digest, '/auxiliary', 'original smoke index changed during validation')
            coverage.append({'step': 9, 'scope': 'original_and_actual_matched_smoke_companions', 'status': 'PASS'})
            completed.append('ui.smoke')
        except (GateContractError, EvidenceError) as error:
            errors.append(f'UI step9: {error}')
    if ('base', 34) in steps and ('candidate', 34) in steps:
        from generic_whole_gate_ui_fixed import evaluate_family as render
        try:
            rows = render(session, steps['base', 34], steps['candidate', 34])
            safety.extend(rows)
            coverage.append({'step': 34, 'scope': 'three_source_selected_render_producers',
                             'status': 'PASS', 'independent_child_wait': False})
            # Whole render_and_pack completion also requires original step36 below.
        except (GateContractError, EvidenceError) as error:
            errors.append(f'UI step34: {error}')
    if ('base', 46) in steps and ('candidate', 46) in steps:
        from generic_whole_gate_ui_regressions import evaluate_family as regressions
        try:
            rows = regressions(session, steps['base', 46], steps['candidate', 46])
            safety.extend(rows)
            coverage.append({'step': 46, 'scope': 'toast_layout_why_both_locales',
                             'status': 'PASS', 'independent_child_wait': False})
            # Original step40 joins separately below.
        except (GateContractError, EvidenceError) as error:
            errors.append(f'UI step46: {error}')
    if ('base', 40) in steps and ('candidate', 40) in steps:
        from generic_whole_gate_ui_why import evaluate_family as why
        try:
            safety.extend(why(session, steps['base', 40], steps['candidate', 40]))
            coverage.append({'step': 40, 'scope': 'original_why_lifecycle', 'status': 'PASS'})
            if ('base', 46) in steps and ('candidate', 46) in steps and not any('UI step46:' in error for error in errors):
                completed.append('why_toast_layout')
        except (GateContractError, EvidenceError) as error:
            errors.append(f'UI step40: {error}')
    for step in (23, 36):
        if ('base', step) not in steps or ('candidate', step) not in steps:
            continue
        from generic_whole_gate_ui_selected import evaluate_family as selected_ui
        try:
            safety.extend(selected_ui(session, steps['base', step], steps['candidate', step]))
            coverage.append({'step': step, 'scope': 'source_selected_ui_only',
                             'status': 'PASS', 'independent_child_wait': False})
            if step == 23:
                from generic_whole_gate_renderer import (
                    evaluate_family as renderer_native,
                )
                result = renderer_native(session, steps['base', step], steps['candidate', step])
                safety.extend(row.value for row in Node(result).field('safety').array())
                coverage.append({'step': step, 'result': result})
                completed.append('renderer.agreement')
            else:
                from generic_whole_gate_pack import evaluate_family as pack_native
                result = pack_native(session, steps['base', step], steps['candidate', step])
                coverage.append({'step': step, 'result': result})
                if ('base', 34) in steps and ('candidate', 34) in steps and not any('UI step34:' in error for error in errors):
                    completed.append('render_and_pack')
        except (GateContractError, EvidenceError) as error:
            errors.append(f'UI step{step}: {error}')
    if ('base', 39) in steps and ('candidate', 39) in steps:
        from generic_whole_gate_ui_visibility import evaluate_family as visibility
        try:
            rows = visibility(session, steps['base', 39], steps['candidate', 39])
            safety.extend(rows)
            coverage.append({'step': 39, 'scope': 'actual_visibility_save_load_worlds', 'status': 'PASS'})
            completed.append('visibility')
        except (GateContractError, EvidenceError) as error:
            errors.append(f'UI step39: {error}')
    if ('base', 38) in steps and ('candidate', 38) in steps:
        from generic_whole_gate_oracles import evaluate_family as oracles
        try:
            coverage.append({'step': 38, 'result': oracles(session, steps['base', 38], steps['candidate', 38])})
        except (GateContractError, EvidenceError) as error:
            errors.append(f'oracle step38: {error}')
    for step in (4, 5, 17, 24, 37, 42, 43, 49):
        if ('base', step) not in steps or ('candidate', step) not in steps:
            continue
        from generic_whole_gate_native import evaluate_family as native
        try:
            result = native(session, steps['base', step], steps['candidate', step])
            safety.extend(row.value for row in Node(result).field('safety').array())
            coverage.append({'step': step, 'result': result})
            if step == 24 and Node(result).field('status').value == 'PASS':
                completed.append('save_roundtrip')
            if Node(result).field('status').value != 'PASS':
                incomplete.append(f'native step{step}: source-selected subset only; see original coverage result')
        except (GateContractError, EvidenceError) as error:
            errors.append(f'native step{step}: {error}')
    for step, family in ((12, 'perf.natural_quick'), (27, 'nav.soak')):
        if ('base', step) not in steps or ('candidate', step) not in steps:
            continue
        from generic_whole_gate_perf_binding import evaluate_family as performance
        try:
            result = performance(session, steps['base', step], steps['candidate', step])
            validated = Node(result)
            validated.field('status').equals('PASS')
            safety.extend(row.value for row in validated.field('safety').array())
            coverage.append({'step': step, 'result': result})
            completed.append(family)
        except (GateContractError, EvidenceError) as error:
            errors.append(f'performance step{step}: {error}')
    if ('candidate', 25) in steps:
        from generic_whole_gate_bench import evaluate as bench
        try:
            coverage.append({'step': 25, 'result': bench(session, steps['candidate', 25])})
            completed.append('bench.timed_rss')
        except (GateContractError, EvidenceError) as error:
            errors.append(f'bench step25: {error}')
    return Families(tuple(safety), tuple(coverage), tuple(errors), tuple(incomplete), tuple(completed))
