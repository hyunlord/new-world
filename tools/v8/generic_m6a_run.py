"""Execute the standing M6a food components; no full-gate publication authority.

Build/source authentication and exclusive runtime ownership belong to the caller.
The original fixed food scorer is retained for both actual native M6a arms.
"""
import json
import sys
from collections.abc import Callable
from dataclasses import asdict, replace
from pathlib import Path
from typing import Final

from generic_declarations import Declarations
from generic_fixture_registry import Invocation, Registry, enforce_required_workload
from generic_fixture_run import ComponentResult, Products
from generic_fixtures import ProbeProduct, run_invocation
from generic_gate import document, require, typed_json, validate_declared_fixture
from generic_m6a_observations import (
    M6aObservation,
    paired_safety,
    paired_selected_safety,
    parse_fixed,
    parse_matched_companion,
    parse_selected_pack,
)
from generic_observations import ProbeWorld
from generic_process import ProcessResult, ProcessSpec, atomic_json, execute, file_hash
from generic_sources import inventory
from mux2_gate_inputs import Json
from read_summary_normalise import first_difference, normalise_pair

FIXED: Final = 'm6a.food72'
SELECTED: Final = 'm6a.food_data_only'
DURATION: Final = 72 * 1440
MAXIMUM: Final = 6 * 1440
PACK: Final = 'rust/crates/sim-test/data/m6a_apple/'
SCORERS: Final = ('tools/v8/m6a_score.py', 'tools/v8/m6a_protocol.py',
                 'tools/v8/m6a_evidence.py', 'tools/v8/m6a_causal.py')
_normalise: Callable[[Json, Json, str | None], tuple[object, object, list[str]]] = normalise_pair


def prepare(registry: Registry, plans: tuple[Invocation, ...], products: Products) -> None:
    """Require the exact two standing fixtures, all seeds and both full primaries."""
    fixtures = {fixture.id: fixture for fixture in registry.fixtures}
    expected = {(arm, FIXED, seed) for arm in ('base', 'candidate') for seed in (0, 1)}
    expected |= {(arm, SELECTED, 0) for arm in ('base', 'candidate')}
    require(len(plans) == 6 and {(p.arm, p.fixture_id, p.seeds) for p in plans} ==
            {(arm, fixture, (seed,)) for arm, fixture, seed in expected}, '/plans', 'exact M6a seed/arm coverage required')
    require(len({p.id for p in plans}) == len({p.out for p in plans}) == 6, '/plans', 'duplicate M6a output/identity')
    for name in (FIXED, SELECTED):
        require(name in fixtures, name, 'missing registered fixture')
        enforce_required_workload(document(fixtures[name].metadata_json).mapping(), {})
        require(fixtures[name].repeats == ('1',), name, 'changed repeat coverage')
    for product in (products.base, products.candidate):
        require(bool(product.input_files), '/inputs', 'missing authenticated content/source inventory')
        require(set(SCORERS) | {PACK + name for name in ('pack.ron', 'items.ron', 'plants.ron')} <=
                dict(product.input_files).keys(), '/inputs', 'missing scorer or selected content binding')
    for name in (*SCORERS, *(PACK + name for name in ('pack.ron', 'items.ron', 'plants.ron'))):
        require(dict(products.base.input_files)[name] == dict(products.candidate.input_files)[name],
                name, 'same original scorer and selected pack required across arms')
    for plan in plans:
        product = products.arm(plan.arm)
        fixed = plan.fixture_id == FIXED
        suffix = f'1/seed-{plan.seeds[0]}' if fixed else '1'
        require(plan.id == f'{plan.arm}/{plan.fixture_id}/{suffix}' and plan.repeat == '1'
                and plan.roles == ('primary',) and plan.generated_input is None, plan.id, 'changed occurrence identity')
        require(plan.cwd == product.input_root and plan.blockers in ((), ('adapter_not_implemented',)),
                plan.id, 'foreign inputs or unresolved obligations')
        expected_argv = ((str(product.executable), 'm6a', '--seed', str(plan.seeds[0]), '--days', '72', '--out', str(plan.out))
                         if fixed else (str(product.executable), 'm6a-data-only', '--out', str(plan.out)))
        require(plan.argv in (expected_argv, expected_argv + ('--gate-observations',)), plan.id, 'changed primary workload')
        environment = () if fixed else (('WORLDSIM_CONTENT_PACK', str(product.input_root / PACK / 'pack.ron')),)
        require(plan.environment == environment, plan.id, 'changed selected content/environment')
        require(all(not plan.out.is_relative_to(p.input_root) for p in (products.base, products.candidate)),
                plan.id, 'output overlaps source inputs')
        require(all(plan.out == other.out or not plan.out.is_relative_to(other.out) for other in plans),
                plan.id, 'overlapping producer outputs')


def reference(path: Path) -> dict[str, Json]:
    return {'path': str(path), 'sha256': file_hash(path), 'size': path.stat().st_size}


def observe(plan: Invocation, result: ProcessResult, *, companion_ticks: int | None = None) -> M6aObservation:
    require(result.status == 'PASS' and result.native_exit == 0 and result.receipt == plan.out / 'process.json',
            plan.id, 'native producer unsuccessful or foreign receipt')
    scene = 'm6a' if plan.fixture_id == FIXED else ('m6a-data-only' if companion_ticks is None else 'm6a-data-only-companion')
    folder = plan.out / 'observed/gate_observations'
    name = f'{scene}_s{plan.seeds[0]}_primary.json'
    require(folder.is_dir() and {p.name for p in folder.iterdir()} == {name}, plan.id, 'missing or extra native observations')
    sidecar = folder / name
    report = plan.out / 'observed' / ('summary.json' if scene == 'm6a' else ('data-only.json' if companion_ticks is None else 'data-only-companion.json'))
    _ = file_hash(sidecar)
    _ = file_hash(report)
    if scene == 'm6a':
        return parse_fixed(sidecar.read_text(), report.read_text(), ProbeWorld(scene, plan.seeds[0], 'primary', DURATION))
    if companion_ticks is not None:
        return parse_matched_companion(sidecar.read_text(), report.read_text(), companion_ticks, MAXIMUM)
    return parse_selected_pack(sidecar.read_text(), report.read_text(), MAXIMUM)


def score_fixed(arm: str, plans: tuple[Invocation, ...], product: ProbeProduct, output: Path) -> ProcessResult:
    """Adapt actual native M6a outputs to the original fixture scorer protocol.

    The scorer's `candidate` denotes its native food-report schema, not product
    identity. Preserve actual product_arm/source_sha separately for BASE as well.
    """
    output.mkdir(exist_ok=False)
    worlds: list[Json] = []
    external: dict[Path, str] = {}
    for plan in sorted(plans, key=lambda p: p.seeds):
        paths = [plan.out / 'observed' / name for name in ('summary.json', 'events.jsonl', 'chronicle.json')]
        paths.append(plan.out / 'stdout.log')
        external.update((path, file_hash(path)) for path in paths)
        worlds.append({'seed': plan.seeds[0], 'exit_code': 0,
                       'files': [reference(path) for path in paths[:-1]], 'log': reference(paths[-1])})
    cohort = output / 'scorer-cohort.json'
    atomic_json(cohort, {'schema_version': 1, 'kind': 'm6a_cohort_v1', 'arm': 'candidate',
                        'product_arm': arm, 'source_sha': product.source_sha,
                        'projection': 'native_m6a_schema_for_original_fixture_scorer_not_legacy_base_protocol',
                        'purpose': 'fixture', 'evaluation_protocol': 'legacy', 'days': 72, 'seeds': [0, 1],
                        'exit_code': 0, 'source_unchanged': True, 'worlds': worlds,
                        'binary': reference(product.executable), 'observer': reference(Path(__file__).resolve()),
                        'scorer': reference(product.input_root / SCORERS[0])})
    external[cohort] = file_hash(cohort)
    external[Path(__file__).resolve()] = file_hash(Path(__file__).resolve())
    # Input inventory already covers a harness-local component source.
    external = {path: digest for path, digest in external.items() if not path.is_relative_to(product.input_root)}
    python = Path(sys.executable).resolve(strict=True)
    result = execute(ProcessSpec((str(python), str(product.input_root / SCORERS[0]), 'fixture', '--receipt', str(cohort),
                                  '--out', str(output / 'score.json')), product.input_root, python, file_hash(python),
                                 product.input_root, product.input_files, output / 'process',
                                 environment=(('PYTHONDONTWRITEBYTECODE', '1'),), external_inputs=tuple(external.items())))
    require(result.status == 'PASS', arm, 'original food fixture scorer failed')
    document((output / 'score.json').read_text()).field('status').equals('PASS')
    return result


def run_registered(registry: Registry, plans: tuple[Invocation, ...], products: Products,
                   declarations: Declarations, output: Path) -> ComponentResult:
    """Run full primaries first; then add BASE at the candidate's actual endpoint."""
    prepare(registry, plans, products)
    require(declarations.base == products.base.source_sha and declarations.candidate == products.candidate.source_sha
            and {FIXED, SELECTED} <= set(declarations.known_fixtures), '/declarations', 'unbound declarations')
    require(not output.exists(), str(output), 'component receipt already exists')
    require(output.is_absolute() and all(not output.is_relative_to(p.input_root)
            for p in (products.base, products.candidate)), str(output), 'receipt overlaps source inputs')
    derived = [next(p for p in plans if p.arm == arm and p.fixture_id == FIXED).out.parent / 'standing-scorer'
               for arm in ('base', 'candidate')]
    derived.append(next(p for p in plans if p.arm == 'base' and p.fixture_id == SELECTED).out.parent / 'matched-companion')
    destinations = [p.out for p in plans] + derived
    require(all(not path.exists() and not path.is_symlink() for path in destinations), '/outputs', 'fresh producer outputs required')
    require(all(not output.is_relative_to(path) and not path.is_relative_to(output) for path in destinations)
            and all(not a.is_relative_to(b) for i, a in enumerate(destinations) for j, b in enumerate(destinations) if i != j),
            '/outputs', 'overlapping component outputs')
    output.parent.mkdir(parents=True, exist_ok=True)
    executions: list[Json] = []
    errors: list[Json] = []
    observations: dict[str, M6aObservation] = {}
    artifacts: dict[str, Json] = {}
    for plan in plans:
        try:
            assert plan.argv is not None
            argv = plan.argv if '--gate-observations' in plan.argv else plan.argv + ('--gate-observations',)
            argv = tuple(str(plan.out / 'observed') if index == argv.index('--out') + 1 else value
                         for index, value in enumerate(argv))
            actual = replace(plan, argv=argv)
            result = run_invocation(actual, products.arm(plan.arm), (), resolved_blockers=plan.blockers)
            executions.append({'id': plan.id, 'receipt': reference(result.receipt), 'status': result.status})
            artifacts[plan.id] = {key: value for key, value in inventory(plan.out, [p.name for p in plan.out.iterdir()]).items()}
            observations[plan.id] = observe(actual, result)
        except (OSError, ValueError, KeyError) as error:
            errors.append({'id': plan.id, 'error': str(error)})
    scorers: list[Json] = []
    for arm in ('base', 'candidate'):
        selected = tuple(p for p in plans if p.arm == arm and p.fixture_id == FIXED)
        target = selected[0].out.parent / 'standing-scorer'
        try:
            require(all(p.id in observations for p in selected), arm, 'food worlds incomplete')
            result = score_fixed(arm, selected, products.arm(arm), target)
            scorers.append({'arm': arm, 'receipt': reference(result.receipt), 'score': reference(target / 'score.json')})
        except (OSError, ValueError, KeyError) as error:
            errors.append({'arm': arm, 'scorer': str(target), 'error': str(error)})
    pairs: list[Json] = []
    for base in (p for p in plans if p.arm == 'base'):
        candidate = next(p for p in plans if p.arm == 'candidate' and p.fixture_id == base.fixture_id and p.seeds == base.seeds)
        row: dict[str, Json] = {'fixture': base.fixture_id, 'seed': base.seeds[0], 'status': 'FAIL'}
        pairs.append(row)
        try:
            require(base.id in observations and candidate.id in observations, base.fixture_id, 'incomplete primary evidence')
            left, right = observations[base.id], observations[candidate.id]
            row.update(base=asdict(left), candidate=asdict(right))
            if base.fixture_id == FIXED:
                safety = paired_safety(left, right)
                a, b, exclusions = _normalise(document((base.out / 'observed/summary.json').read_text()).value,
                                               document((candidate.out / 'observed/summary.json').read_text()).value, declarations.base)
                difference = first_difference(typed_json(a), typed_json(b))
                row.update(difference=difference, excluded_paths=[p for p in exclusions])
                admission = declarations.validator_fields(FIXED)
                admission.update(semantic_equal=difference is None, safety=safety)
                validate_declared_fixture(json.dumps(admission))
            else:
                require(left.native_passed and right.native_passed, SELECTED, 'both primary chain oracles must pass')
                ticks = right.world.terminal_tick
                require(0 < ticks <= MAXIMUM, SELECTED, 'invalid candidate actual horizon')
                companion_out = base.out.parent / 'matched-companion'
                companion = replace(base, id=base.id + '/matched-companion', out=companion_out,
                                    argv=(str(products.base.executable), 'm6a-data-only', '--out', str(companion_out / 'observed'),
                                          '--matched-horizon-ticks', str(ticks), '--gate-observations'))
                result = run_invocation(companion, products.base, (), resolved_blockers=base.blockers)
                executions.append({'id': companion.id, 'receipt': reference(result.receipt), 'status': result.status})
                artifacts[companion.id] = {key: value for key, value in inventory(companion.out, [p.name for p in companion.out.iterdir()]).items()}
                matched = observe(companion, result, companion_ticks=ticks)
                row['base_companion'] = asdict(matched)
                safety = paired_selected_safety(left, right, matched)
            row.update(status='PASS', safety=safety)
        except (OSError, ValueError, KeyError) as error:
            row['error'] = str(error)
            errors.append({'fixture': base.fixture_id, 'seed': base.seeds[0], 'error': str(error)})
    try:
        for product in (products.base, products.candidate):
            require(file_hash(product.executable) == product.executable_sha256 and
                    inventory(product.input_root, [name for name, _ in product.input_files]) == dict(product.input_files),
                    product.source_sha, 'product inputs or executable changed during component')
        for identity, frozen in artifacts.items():
            plan = next((p for p in plans if p.id == identity), None)
            path = plan.out if plan is not None else derived[-1]
            require(inventory(path, [p.name for p in path.iterdir()]) == frozen, identity, 'native artifacts changed during component')
    except (OSError, ValueError) as error:
        errors.append({'postflight': str(error)})
    status = 'FAIL' if errors else 'PASS'
    atomic_json(output, {'schema': 'm6a_component_v1', 'status': status, 'publication_authority': False,
                        'registry_sha256': registry.sha256, 'base': declarations.base, 'candidate': declarations.candidate,
                        'declaration_manifest_sha256': declarations.source_manifest_sha256,
                        'executions': executions, 'scorers': scorers, 'pairs': pairs, 'artifacts': artifacts, 'errors': errors,
                        'limit': 'Component only; caller authenticates builds/source and exclusive runtime lane.'})
    return ComponentResult(output, status)
