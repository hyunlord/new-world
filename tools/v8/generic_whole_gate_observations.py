"""Closed whole-shell observation routes; parsing never authenticates a producer.

The launcher binds registry/plans and native outputs to its actual journal. These
collectors consume those original outputs without executing a second workload or
manufacturing ProcessResults. Unsupported routes are finite admission blockers.
"""
from dataclasses import dataclass, replace
from pathlib import Path
from typing import Final

from generic_fixture_registry import BACKLOG_IDS, DIRECT_IDS, Registry
from generic_gate import boundary, document, integer, require, text
from generic_process import file_hash, real_path
from generic_whole_gate_paired import PAIRED, Paired, authenticate, recheck
from generic_whole_gate_probe import (
    ProbeEvidence,
    collect_probe,
    probe_safety,
    recheck_probe,
)
from mux2_gate_inputs import Json, Node

__all__ = [
    'ProbeEvidence',
    'authenticate_stage',
    'collect_probe',
    'evaluate',
    'probe_safety',
    'recheck_probe',
    'route',
    'routes',
]

PROBE_IDS: Final = DIRECT_IDS - {
    'm6a.food72', 'm6a.food_data_only',
}
MISSING: Final = {
    'm6a.food72': 'actual early-stop and matched-horizon companion join',
    'm6a.food_data_only': 'selected pack and actual food horizon join',
    'perf.natural_quick': 'actual perf run-index/child source join',
    'bench.timed_rss': 'actual bench child journal and acceptance result join',
    'nav.soak': 'actual nav epochs and child source join',
    'ui.smoke': 'whole-shell native UI lifecycle/process binding',
    'ui.play': 'original owner join callable; native observer qualification remains pending',
    'renderer.agreement': 'original three producers and four native instances join',
    'render_and_pack': 'original render/pack native journal lifecycle join',
    'visibility': 'actual generator/loader state and native lifecycle join',
    'why_toast_layout': 'locale/toast/native versus synthetic layout classification',
    'stall': 'authenticated capture/child-chain and full safety conjunction',
    'save_roundtrip': 'actual native PID plus original save/load history joins',
    'named_integration_boundaries': 'native test case/world classification and child joins',
}


IMPLEMENTED: Final = frozenset(MISSING) - {'named_integration_boundaries'}


@dataclass(frozen=True, slots=True)
class Route:
    fixture: str
    supported: bool  # Callable collection readiness; never returned qualification or safety PASS.
    missing: str | None


def routes(registry: Registry) -> tuple[Route, ...]:
    """Reject silently omitted/added families; registry completeness is not assumed."""
    actual = {fixture.id for fixture in registry.fixtures}
    backlog = {text(document(raw).field('id')) for raw in registry.backlog}
    require(actual == set(DIRECT_IDS) and backlog == set(BACKLOG_IDS), '/routes', 'registry family set changed')
    require(PROBE_IDS | MISSING.keys() == DIRECT_IDS | BACKLOG_IDS, '/routes', 'route table incomplete')
    return tuple(Route(name, name in PROBE_IDS or name in IMPLEMENTED, MISSING.get(name)) for name in sorted(actual | backlog))


STEP_ORDER: Final = (*range(1, 37), 42, 43, *range(37, 42), *range(44, 52))
WORLD_STEPS: Final = frozenset({4, 5, 9, 10, 11, 12, 17, 18, 19, 21, 23, 24, 25,
                              26, 27, 28, 29, 30, 34, 35, 36, 37, 38, 39, 40, 41,
                              42, 43, 44, 46, 47, 48, 49, 50})


def route(session: Node, step: int, output: Path) -> dict[str, str]:
    """Environment-only routing; actual owner creates fresh external directories.

    One shared step ID is intentionally not claimed to meet dedicated per-native
    UI validators. PID-qualified emitted files remain available for their joins.
    """
    require(step in STEP_ORDER, '/step', 'foreign original step')
    root = Path(text(session.field('output')))
    require(output.is_absolute() and output.is_relative_to(root) and output != root,
            '/output', 'observer output outside session')
    require('..' not in output.parts, '/output', 'noncanonical routing path')
    if step not in WORLD_STEPS:
        return {}
    environment = {
        'WORLDSIM_GATE_OBSERVER': '1',
        'WORLDSIM_GATE_TEST_OUT': str(output / 'native-worlds'),
        'WORLDSIM_GATE_UI_OUT': str(output / 'ui-worlds'),
        'WORLDSIM_GATE_INVOCATION_ID': f'whole-gate:step{step}:{output}',
    }
    if step in {9, 21}:
        environment['WORLDSIM_GATE_UI_BOOT'] = '1'
    for key in environment:
        require(key not in session.field('environment').mapping(), key, 'reserved observer override')
    return environment


@boundary
def authenticate_stage(node: Node, session: Node) -> int:
    """Authenticate actual aggregate process only, never label it a native child."""
    step = integer(node.field('step'))
    require(step in STEP_ORDER, '/step', 'foreign stage')
    node.field('state').equals('RETURNED')
    receipt = Path(text(node.field('process_receipt')))
    real_path(receipt, directory=False)
    require(receipt.is_relative_to(Path(text(session.field('output')))), '/receipt', 'foreign output')
    digest = text(node.field('process_receipt_sha256'))
    require(file_hash(receipt) == digest, '/receipt', 'stage process receipt changed')
    raw = document(receipt.read_text())
    for field in ('native_exit', 'pid', 'cwd', 'environment'):
        raw.field(field).equals(node.field(field).value)
    # This authenticates settlement only. Exit 1 remains fatal unless the final
    # native capture and every other route independently admit its narrow waiver.
    provisional_stall = (step == 41 and node.field('arm').value in {'base', 'candidate'}
                         and raw.field('native_exit').value == 1)
    raw.field('status').equals('FAIL' if provisional_stall else 'PASS')
    raw.field('native_exit').equals(1 if provisional_stall else 0)
    raw.field('errors').equals(['native exit 1'] if provisional_stall else [])
    require(integer(raw.field('pid')) > 0, '/pid', 'missing aggregate process identity')
    for channel in ('stdout', 'stderr'):
        require(file_hash(receipt.parent / (channel + '.log')) ==
                text(raw.field(channel + '_sha256')), '/process', 'raw process log changed')
    return step


def evaluate(session: Node, journal: tuple[Node, ...], output: Path, *, descriptor: int | None = None) -> dict[str, Json]:
    """Evaluate source-loaded A session against actual stage and probe artifacts.

    This returns explicit finite missing routes until dedicated original native
    joins are implemented. No caller status row, declaration, or partial journal
    can produce full-gate PASS. A additionally authenticates its whole-shell exit.
    """
    from generic_fixture_registry import ExpansionContext, expand, load_registry
    from generic_gate import GateContractError

    require(output.is_relative_to(Path(text(session.field('output')))), '/output', 'foreign evaluation')
    registry_path = Path(text(session.field('registry').field('path')))
    registry_hash = text(session.field('registry').field('sha256'))
    require(file_hash(registry_path) == registry_hash, '/registry', 'registry changed')
    registry = load_registry(registry_path)
    table = routes(registry)
    errors: list[Json] = []
    incomplete: list[Json] = [f'{r.fixture}: {r.missing}' for r in table if r.missing is not None]
    steps: dict[tuple[str, int], Node] = {}
    for node in journal:
        try:
            number = integer(node.field('step'))
            arm_name = text(node.field('arm'))
            require(arm_name in {'base', 'candidate'}, '/journal', 'unknown arm')
            require((arm_name, number) not in steps, '/journal', 'duplicate step')
            steps[arm_name, number] = node
            _ = authenticate_stage(node, session)
        except GateContractError as error:
            errors.append(str(error))
    if tuple(number for arm_name, number in steps if arm_name == 'candidate') != STEP_ORDER:
        incomplete.append('missing or out-of-order original stage sequence')
    config = session.field('observations')
    require(set(config.mapping()) == {'probes'}, '/observations', 'unknown observation configuration')
    planned: dict[tuple[str, str, str], ProbeEvidence] = {}
    probe_steps: dict[tuple[str, str, str], int] = {}
    comparisons: list[Json] = []
    paired: list[Paired] = []
    paired_safety_rows: list[Json] = []
    completed_dedicated: set[str] = set()
    for step_number in sorted(PAIRED):
        if ('candidate', step_number) not in steps:
            incomplete.append(f'missing paired stage {step_number}')
            continue
        try:
            evidence = authenticate(session, steps['candidate', step_number], registry)
            paired.append(evidence)
            if step_number in {47, 48}:
                from generic_whole_gate_m6a import evaluate as evaluate_m6a
                paired_safety_rows.extend(evaluate_m6a(evidence))
                completed_dedicated.update(p.fixture_id for p in evidence.plans)
            for plan in evidence.plans:
                if plan.fixture_id not in PROBE_IDS:
                    continue
                directory = plan.out / 'gate_observations'
                files = tuple((Path(name).name, digest) for name, digest in evidence.closure.files
                              if Path(name).parent == directory)
                mode = dict(evidence.closure.modes)[str(directory)]
                key = (plan.fixture_id, plan.arm, plan.repeat)
                require(key not in planned, '/paired', 'duplicate paired probe')
                planned[key] = collect_probe(registry, plan, files, mode)
        except (GateContractError, KeyError, StopIteration) as error:
            errors.append(str(error))
    expected = {(f.id, arm, repeat) for f in registry.fixtures if f.id in PROBE_IDS
                for arm in ('base', 'candidate') for repeat in f.repeats}
    for row in config.field('probes').array():
        try:
            require(set(row.mapping()) == {'fixture_id', 'arm', 'repeat', 'output', 'step'},
                    row.label, 'unknown probe routing fields')
            identity = (text(row.field('fixture_id')), text(row.field('arm')), text(row.field('repeat')))
            require(identity in expected and identity not in planned, row.label, 'foreign or duplicate invocation')
            step_number = integer(row.field('step'))
            fixture = next(f for f in registry.fixtures if f.id == identity[0])
            consumers = document(fixture.metadata_json).field('consumers').array()
            mapped = tuple(v.value for v in consumers)
            require(step_number in mapped or (step_number == 44 and identity[0] in
                    {'core.steward_off', 'core.steward_on'}) or
                    (step_number == 28 and identity[0] == 'knowledge.f10_no_kill'),
                    row.label, 'fixture is not registered at this original step')
            stage = steps[identity[1], step_number]
            _ = authenticate_stage(stage, session)
            arm = session.field('arms').field(identity[1])
            root = Path(text(arm.field('root')))
            directory = Path(text(row.field('output')))
            require(directory.is_relative_to(Path(text(session.field('output')))), row.label, 'foreign producer output')
            candidates = expand(registry, ExpansionContext(root, root, output / 'plans',
                                root / 'rust/target/release/worldsim_probe', root / 'rust/target/release/worldsim_probe'))
            plan = next(p for p in candidates if (p.fixture_id, p.arm, p.repeat) == identity)
            plan = replace(plan, out=directory)
            native_dir = directory / 'gate_observations'
            artifacts = stage.field('observer_artifacts').mapping()
            original = tuple((Path(name).name, text(stage.field('observer_artifacts').field(name))) for name in artifacts
                             if Path(name).parent == native_dir)
            mode = integer(stage.field('observer_directories').field(str(native_dir)))
            planned[identity] = collect_probe(registry, plan, original, mode)
            probe_steps[identity] = step_number
        except (GateContractError, KeyError, StopIteration) as error:
            errors.append(str(error))
    missing = expected - planned.keys()
    incomplete.extend('missing probe ' + '/'.join(identity) for identity in sorted(missing))
    incomplete = [row for row in incomplete if not any(str(row).startswith(name + ":") for name in completed_dedicated)]
    safety: list[Json] = list(paired_safety_rows)
    for fixture, arm, repeat in sorted(planned):
        if arm != 'base' or (fixture, 'candidate', repeat) not in planned:
            continue
        try:
            if fixture == 'knowledge.f10_injected':
                from generic_whole_gate_f10 import evaluate as evaluate_f10
                rows = evaluate_f10(session, steps['base', 28], steps['candidate', 28],
                    planned[fixture, 'base', repeat], planned[fixture, 'candidate', repeat],
                    planned['knowledge.f10_no_kill', 'base', repeat], planned['knowledge.f10_no_kill', 'candidate', repeat])
            else:
                rows = probe_safety(planned[fixture, arm, repeat], planned[fixture, 'candidate', repeat])
            if (fixture, 'base', repeat) in probe_steps:
                from generic_whole_gate_acceptance import parse_declarations
                from generic_whole_gate_comparison import compare
                comparison = compare(session, parse_declarations(session.field('declarations')),
                    steps['base', probe_steps[fixture, 'base', repeat]],
                    steps['candidate', probe_steps[fixture, 'candidate', repeat]],
                    planned[fixture, 'base', repeat], planned[fixture, 'candidate', repeat], rows)
                comparisons.append(comparison)
            safety.append({'fixture': fixture, 'repeat': repeat, 'worlds': list(rows)})
        except (GateContractError, KeyError) as error:
            errors.append(str(error))
    from generic_whole_gate_families import evaluate as evaluate_families
    families = evaluate_families(session, steps, descriptor=descriptor)
    safety.extend(families.safety)
    errors.extend(families.errors)
    incomplete.extend(families.incomplete)
    incomplete = [row for row in incomplete if not any(str(row).startswith(name + ':') for name in families.completed)]
    # Last checks retain original bytes. No terminal rebaselining or caller PASS.
    for evidence in planned.values():
        try:
            recheck_probe(evidence)
        except GateContractError as error:
            errors.append(str(error))
    for evidence in paired:
        try:
            recheck(evidence.closure)
        except GateContractError as error:
            errors.append(str(error))
    for node in steps.values():
        try:
            _ = authenticate_stage(node, session)
        except GateContractError as error:
            errors.append(str(error))
    require(file_hash(registry_path) == registry_hash, '/registry', 'registry changed during evaluation')
    stall: dict[str, Json] = {'stall_status': 'NOT_ADMITTED_MISSING_NATIVE_CAPTURE_JOIN',
                             'waived_stage_exits': []}
    others_missing = [row for row in incomplete if not str(row).startswith('stall:')]
    stall_results: list[Json] = []
    waivers: list[Json] = []
    references: list[Json] = []
    for arm_name in ('base', 'candidate'):
        if (arm_name, 41) not in steps:
            incomplete.append(f'stall: missing {arm_name} original capture')
            continue
        stage = steps[arm_name, 41]
        admitted = False
        if not errors and not others_missing:
            from generic_whole_gate_stall import OtherStep
            from generic_whole_gate_stall_binding import evaluate_stage
            try:
                # Both arms' returned stages and every world route were checked
                # above. Candidate has the entire original 51-step sequence.
                others = tuple(OtherStep(n, 'PASS' if steps['candidate', n].field('native_exit').value == 0
                                         else 'FAIL') for n in range(1, 52) if n != 41)
                result = evaluate_stage(session, stage, others, bool(safety))
                Node(result).field('status').equals('PASS')
                stall_results.append(result)
                if arm_name == 'base':
                    Node(result).field('waived_stage_exits').equals([])
                    references.extend(row.value for row in Node(result).field('reference_stage_exits').array())
                else:
                    waivers.extend(row.value for row in Node(result).field('waived_stage_exits').array())
                admitted = True
            except GateContractError as error:
                errors.append(f'stall {arm_name} step41: {error}')
        if stage.field('native_exit').value == 1 and not admitted:
            errors.append(f'stall {arm_name} step41: original native exit 1 has no authenticated waiver')
    if len(stall_results) == 2:
        incomplete = others_missing
        stall = {'stall_status': 'WAIVED' if waivers else 'STRICT_PASS',
                 'waived_stage_exits': waivers}
    status = 'FAIL' if errors else ('INCOMPLETE' if incomplete else 'PASS')
    return {'status': status, 'errors': errors,
            'missing_routes': incomplete, 'safety': safety, 'family_coverage': list(families.coverage), 'comparisons': comparisons,
            'publication_authority': False, 'stall_waiver': stall['stall_status'],
            'waived_stage_exits': stall['waived_stage_exits'] if status == 'PASS' else [],
            'reference_stage_exits': references if status == 'PASS' else []}
