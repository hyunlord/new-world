# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m unittest discover -s tools/v8 -p test_generic_fixture_run.py
"""Paired ordinary colony component; never grants full-gate publication authority."""
import json
import sys
from collections.abc import Callable
from dataclasses import asdict, dataclass, replace
from pathlib import Path
from typing import Final

from generic_declarations import Declarations
from generic_fixture_registry import Invocation, Registry
from generic_fixtures import ProbeProduct, fixed_observations, run_invocation
from generic_gate import (
    GateContractError,
    document,
    require,
    text,
    typed_json,
    validate_declared_fixture,
)
from generic_observations import ordinary_pair
from generic_process import ProcessResult, ProcessSpec, atomic_json, execute, file_hash
from generic_sources import inventory
from mux2_gate_inputs import Json, Node
from read_summary_normalise import first_difference, normalise_pair

# The committed legacy normaliser has an untyped API; validate its returned JSON.
_normalise: Callable[[Json, Json, str | None], tuple[object, object, list[str]]] = normalise_pair

SUPPORTED: Final = frozenset({
    'colony.u10', 'core.colony_u10', 'read.determinism',
    'replay.generate', 'replay.recorded', 'replay.scripted', 'economy.soak_no_beasts',
    'story.script48', 'neighbours.script48', 'read.story48', 'read.neighbours48',
    'steward.on24_twins', 'read.steward_on24', 'read.m6a_steward_on72',
    'read.soak20', 'core.colony_soak20', 'core.steward_off', 'core.steward_on',
    'm6a.lifecycle_off', 'm6a.lifecycle_on',
})


@dataclass(frozen=True, slots=True)
class Products:
    base: ProbeProduct
    candidate: ProbeProduct

    def arm(self, name: str) -> ProbeProduct:
        require(name in {'base', 'candidate'}, name, 'unknown arm')
        return self.base if name == 'base' else self.candidate


@dataclass(frozen=True, slots=True)
class ComponentResult:
    receipt: Path
    status: str


def prepare(registry: Registry, plans: tuple[Invocation, ...], products: Products) -> None:
    """Reject unsupported or weakened subsets before starting any producer."""
    require(bool(plans), '/plans', 'empty component selection')
    require(len({p.id for p in plans}) == len(plans) and len({p.out for p in plans}) == len(plans),
            '/plans', 'duplicate invocation or output')
    fixtures = {f.id: f for f in registry.fixtures}
    for selected in {p.fixture_id for p in plans}:
        require(selected in fixtures, selected, 'unregistered fixture')
        fixture = fixtures[selected]
        meta = document(fixture.metadata_json)
        require(selected in SUPPORTED and fixture.adapter == 'colony_sidecar' and meta.field('role').value == 'ordinary'
                and not selected.startswith('knowledge.') and meta.field('horizon').field('kind').value == 'requested_days',
                selected, 'unsupported fixture: dedicated baseline/injection/m6a adapter required')
        rows = [p for p in plans if p.fixture_id == selected]
        require({(p.arm, p.repeat) for p in rows} == {(a, r) for a in ('base', 'candidate') for r in fixture.repeats}
                and len(rows) == 2 * len(fixture.repeats), selected, 'incomplete paired repeats')
        for plan in rows:
            product = products.arm(plan.arm)
            require(not plan.blockers and plan.seeds == fixture.seeds and plan.roles == fixture.roles,
                    plan.id, 'blocked or changed occurrence coverage')
            require(plan.id == f'{plan.arm}/{selected}/{plan.repeat}' and plan.cwd == product.input_root,
                    plan.id, 'identity or effective input root mismatch')
            values = {'probe': str(product.executable), 'inputs': str(product.input_root),
                      'harness_root': str(product.input_root), 'out': str(plan.out),
                      'commands_out': str(plan.out / 'commands'), 'chronicle_out': str(plan.out / 'chronicle')}
            if plan.generated_input is not None:
                dependency = plan.generated_input
                require(selected == 'replay.recorded' and dependency.sha256 is None
                        and dependency.producer_invocation == f'{plan.arm}/replay.generate/1', plan.id, 'foreign replay dependency')
                producer = next((p for p in plans if p.id == dependency.producer_invocation), None)
                require(producer is not None, plan.id, 'missing replay producer')
                if producer is None:
                    raise AssertionError('validated producer')
                require(dependency.path == producer.out / 'commands/colony_s5_commands.json', plan.id, 'wrong replay path')
                values['replay_generate_same_arm_commands'] = str(dependency.path.parent)
            require(selected != 'replay.recorded' or plan.generated_input is not None, plan.id, 'missing replay dependency')
            expected = tuple(arg.format_map(values) for arg in fixture.argv) + ('--gate-observations',)
            require(plan.argv == expected and not plan.environment, plan.id, 'changed registered arguments/environment')
            for path_node in meta.field('input_paths').array():
                path = text(path_node)
                require(path in dict(product.input_files), plan.id, 'missing source-bound script input')
    if any(p.fixture_id == 'replay.recorded' for p in plans):
        require(any(p.fixture_id == 'replay.scripted' for p in plans), '/plans', 'missing replay scripted oracle')


def standing(plan: Invocation, product: ProbeProduct, summary: Node, results: dict[str, ProcessResult],
             scorer_outputs: dict[Path, dict[str, str]]) -> list[Json]:
    """Keep the standing assertions, and run existing scorers with byte bindings."""
    evidence: list[Json] = []
    if plan.argv is not None and '--check-determinism' in plan.argv:
        stdout = (plan.out / 'stdout.log').read_text()
        require('DETERMINISM PASS' in stdout and 'DETERMINISM FAIL' not in stdout, plan.id, 'determinism stdout oracle failed')
        evidence.append({'oracle': 'determinism', 'stdout_sha256': file_hash(plan.out / 'stdout.log')})
    if plan.fixture_id in {'economy.soak_no_beasts', 'steward.on24_twins'}:
        for row in summary.field('seeds').array():
            require(row.field('ledger_violation_ticks').value == 0 and row.field('reservation_violation_ticks').value == 0, plan.id, 'ledger/reservation oracle failed')
            if plan.fixture_id == 'economy.soak_no_beasts':
                require(all(type(n) is int and n == 0 for n in row.field('deaths').mapping().values())
                        and row.field('m4').field('lost').value == [] and row.field('workshops_built').number() >= 2
                        and row.field('crafts_done').number() >= 1 and row.field('trees_felled').number() >= 1, plan.id, 'soak oracle failed')
        evidence.append({'oracle': plan.fixture_id, 'status': 'PASS'})
    scorers = {'story.script48': ('docs/v8/m5/m5_score.py', '--story'),
               'read.story48': ('docs/v8/m5/m5_score.py', '--story'),
               'neighbours.script48': ('docs/v8/m5_5/m55_score.py', '--neighbours'),
               'read.neighbours48': ('docs/v8/m5_5/m55_score.py', '--neighbours')}
    if plan.fixture_id in scorers:
        script, mode = scorers[plan.fixture_id]
        require(script in dict(product.input_files), plan.id, 'scorer not in authenticated source inventory')
        python = Path(sys.executable).resolve(strict=True)
        path = plan.out / 'summary.json'
        result = execute(ProcessSpec((str(python), str(product.input_root / script), mode, str(path)),
                         product.input_root, python, file_hash(python), product.input_root,
                         product.input_files, plan.out / 'standing-scorer', external_inputs=((path, file_hash(path)),)))
        scorer_outputs[result.receipt.parent] = inventory(result.receipt.parent, [p.name for p in result.receipt.parent.iterdir()])
        evidence.append({'oracle': script, 'receipt': str(result.receipt), 'status': result.status})
        require(result.status == 'PASS', plan.id, 'standing scorer failed; see standing-scorer/process.json')
    if plan.fixture_id == 'replay.recorded':
        other = f'{plan.arm}/replay.scripted/1'
        require(other in results and results[other].status == 'PASS', plan.id, 'scripted replay did not succeed')
        require((plan.out / 'summary.json').read_bytes() == (results[other].receipt.parent / 'summary.json').read_bytes(),
                plan.id, 'same-arm replay summary differs')
        evidence.append({'oracle': 'byte-identical scripted replay', 'status': 'PASS'})
    return evidence


REPLAY_CHAIN: Final = ("replay.generate", "replay.recorded", "replay.scripted")


def _execution_order(plans: tuple[Invocation, ...]) -> tuple[Invocation, ...]:
    """Order each replay chain while leaving every unrelated plan in its existing slot."""
    replay = iter(sorted((p for p in plans if p.fixture_id in REPLAY_CHAIN),
                         key=lambda p: (p.arm, REPLAY_CHAIN.index(p.fixture_id))))
    return tuple(next(replay) if p.fixture_id in REPLAY_CHAIN else p for p in plans)


def _run_registered(registry: Registry, plans: tuple[Invocation, ...], products: Products,
                    declarations: Declarations, output: Path, stop_after_failure: bool) -> ComponentResult:
    """Run a selected complete paired subset, retaining failed children and all safety rows."""
    prepare(registry, plans, products)
    require(declarations.base == products.base.source_sha and declarations.candidate == products.candidate.source_sha,
            '/declarations', 'source-bound declarations differ from authenticated products')
    require({p.fixture_id for p in plans} <= set(declarations.known_fixtures), '/declarations', 'missing registered fixture')
    require(not output.exists(), str(output), 'component receipt already exists')
    output.parent.mkdir(parents=True, exist_ok=True)
    results: dict[str, ProcessResult] = {}
    errors: list[Json] = []
    executions: list[Json] = []
    artifacts: dict[str, dict[str, str]] = {}
    scorer_outputs: dict[Path, dict[str, str]] = {}
    # Original step19 uses same-arm generate && recorded && scripted.
    completed: set[str] = set()
    ordered = _execution_order(plans)
    not_run: list[Json] = []
    blocked_by: str | None = None
    for plan in ordered:
        if blocked_by is not None:
            not_run.append({'id': plan.id, 'status': 'NOT_RUN', 'blocked_by': blocked_by,
                            'reason': 'original step50 set-e producer chain stopped'})
            continue
        try:
            if plan.fixture_id == "replay.scripted":
                require(f"{plan.arm}/replay.recorded/1" in completed,
                        plan.id, "same-arm recorded replay unsuccessful; scripted child not started")
            if plan.generated_input is not None:
                dependency = plan.generated_input
                require(dependency.producer_invocation in completed,
                        plan.id, 'generated replay producer unsuccessful')
                plan = replace(plan, generated_input=replace(dependency, sha256=file_hash(dependency.path)))
            result = run_invocation(plan, products.arm(plan.arm), ())
            results[plan.id] = result
            executions.append({'id': plan.id, 'receipt': str(result.receipt), 'status': result.status,
                               'generated_input': None if plan.generated_input is None else
                               {'path': str(plan.generated_input.path), 'sha256': plan.generated_input.sha256,
                                'producer': plan.generated_input.producer_invocation}})
            artifacts[plan.id] = inventory(plan.out, [p.name for p in plan.out.iterdir()])
            require(result.status == 'PASS', plan.id, 'producer failed')
            completed.add(plan.id)
        except (OSError, ValueError) as error:
            errors.append({'invocation': plan.id, 'error': str(error)})
            if stop_after_failure:
                blocked_by = plan.id
    pairs: list[Json] = []
    fixtures = {f.id: f for f in registry.fixtures}
    for base in (p for p in plans if p.arm == 'base'):
        candidate = next(p for p in plans if p.arm == 'candidate' and p.fixture_id == base.fixture_id and p.repeat == base.repeat)
        row: dict[str, Json] = {'fixture': base.fixture_id, 'repeat': base.repeat, 'status': 'FAIL', 'worlds': []}
        worlds: list[Json] = []
        row['worlds'] = worlds
        pairs.append(row)
        try:
            require(base.id in results and candidate.id in results, base.fixture_id, 'producer incomplete')
            fixture = fixtures[base.fixture_id]
            left = fixed_observations(fixture, base, results[base.id])
            right = fixed_observations(fixture, candidate, results[candidate.id])
            summaries = (document((base.out / 'summary.json').read_text()), document((candidate.out / 'summary.json').read_text()))
            for plan, summary in zip((base, candidate), summaries):
                require(summary.field('scene').value == 'colony' and tuple(r.field('seed').value for r in summary.field('seeds').array()) == plan.seeds
                        and summary.field('days').value == document(fixture.metadata_json).field('horizon').field('days').value, plan.id, 'summary coverage mismatch')
            raw_a, raw_b, excluded = _normalise(summaries[0].value, summaries[1].value, declarations.base)
            a, b = typed_json(raw_a), typed_json(raw_b)
            difference = first_difference(a, b)
            if base.generated_input is not None and candidate.generated_input is not None:
                left_hash = file_hash(base.generated_input.path)
                right_hash = file_hash(candidate.generated_input.path)
                row['generated_input_hashes'] = {'base': left_hash, 'candidate': right_hash}
                if left_hash != right_hash and difference is None:
                    difference = '/generated_input: recorded command bytes differ between arms'
            row['base_outputs'] = {key: value for key, value in inventory(base.out, ['summary.json', 'gate_observations', 'stdout.log']).items()}
            row['candidate_outputs'] = {key: value for key, value in inventory(candidate.out, ['summary.json', 'gate_observations', 'stdout.log']).items()}
            row.update(base_normalised=a, candidate_normalised=b, excluded_paths=[path for path in excluded],
                       difference=difference, semantic_equal=difference is None,
                       base_summary_sha256=file_hash(base.out / 'summary.json'),
                       candidate_summary_sha256=file_hash(candidate.out / 'summary.json'))
            for key in left:
                world: dict[str, Json] = {'seed': key[0], 'instance': key[1], 'status': 'FAIL',
                                         'base': asdict(left[key]), 'candidate': asdict(right[key])}
                worlds.append(world)
                try:
                    safety = ordinary_pair(left[key], right[key])
                    admission = declarations.validator_fields(fixture.id)
                    admission.update(semantic_equal=difference is None, safety=safety)
                    validate_declared_fixture(json.dumps(admission))
                    world.update(status='PASS', safety=safety)
                except GateContractError as error:
                    world['error'] = str(error)
            if blocked_by is not None:
                row['oracle_status'] = 'NOT_RUN'
                row['oracle_blocked_by'] = blocked_by
                require(False, fixture.id, 'step50 producer chain stopped; standing validation not run')
            oracle_errors: list[str] = []
            for plan, summary in zip((base, candidate), summaries):
                try:
                    row[plan.arm + '_oracles'] = standing(plan, products.arm(plan.arm), summary, results, scorer_outputs)
                except (OSError, ValueError, KeyError, TypeError) as error:
                    oracle_errors.append(str(error))
            row['oracle_errors'] = [error for error in oracle_errors]
            require(not oracle_errors and all(Node(w).field('status').value == 'PASS' for w in worlds), fixture.id, 'world safety/declaration or standing oracle failed')
            row['status'] = 'PASS'
        except (OSError, ValueError, KeyError, TypeError) as error:
            row['error'] = str(error)
            errors.append({'fixture': base.fixture_id, 'repeat': base.repeat, 'error': str(error)})
    for plan in plans:
        if plan.id not in artifacts:
            continue
        try:
            names = [p.name for p in plan.out.iterdir() if p not in scorer_outputs]
            require(inventory(plan.out, names) == artifacts[plan.id], plan.id, 'producer artifacts changed during component')
        except (OSError, ValueError) as error:
            errors.append({'postflight': plan.id, 'error': str(error)})
    for folder, hashes in scorer_outputs.items():
        try:
            require(inventory(folder, [p.name for p in folder.iterdir()]) == hashes, str(folder), 'scorer artifacts changed during component')
        except (OSError, ValueError) as error:
            errors.append({'postflight': str(folder), 'error': str(error)})
    for product in (products.base, products.candidate):
        try:
            require(file_hash(product.executable) == product.executable_sha256 and
                    inventory(product.input_root, [name for name, _ in product.input_files]) == dict(product.input_files),
                    product.source_sha, 'product inputs or executable changed during component')
        except (OSError, ValueError) as error:
            errors.append({'postflight': product.source_sha, 'error': str(error)})
    status = 'FAIL' if errors else 'PASS'
    atomic_json(output, {'schema': 'ordinary_colony_component_v1', 'status': status, 'publication_authority': False,
                        'registry_sha256': registry.sha256, 'declaration_manifest_sha256': declarations.source_manifest_sha256,
                        'base': declarations.base, 'candidate': declarations.candidate,
                        'executions': executions, **({'not_run': not_run} if stop_after_failure else {}), 'pairs': pairs, 'errors': errors,
                        'producer_outputs': {name: dict(hashes) for name, hashes in artifacts.items()},
                        'scorer_outputs': {str(folder): dict(hashes) for folder, hashes in scorer_outputs.items()}})
    return ComponentResult(output, status)


def run_registered(registry: Registry, plans: tuple[Invocation, ...], products: Products,
                   declarations: Declarations, output: Path) -> ComponentResult:
    """Keep the development batch continuation contract unchanged."""
    return _run_registered(registry, plans, products, declarations, output, False)


def run_preservation(registry: Registry, plans: tuple[Invocation, ...], products: Products,
                     declarations: Declarations, output: Path) -> ComponentResult:
    """Run the complete step50 chain; the direct helper supplies its closed mapped order."""
    require(frozenset(p.fixture_id for p in plans) == frozenset(name for name in SUPPORTED if name.startswith('read.')),
            '/preservation', 'complete six-fixture preservation batch required')
    return _run_registered(registry, plans, products, declarations, output, True)
