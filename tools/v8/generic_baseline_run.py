"""Protected baseline component: strict identity and native safety, no declarations.

Caller owns source/build qualification and the global lane. Synthetic executables
can test this orchestration; they never establish native simulation evidence.
"""
import hashlib
import json
from collections.abc import Callable
from pathlib import Path
from typing import Final

from compare_chronicles import compare as compare_chronicles
from generic_fixture_registry import Invocation, Registry, parse_fixture
from generic_fixture_run import ComponentResult, Products
from generic_fixtures import fixed_observations, run_invocation
from generic_gate import GateContractError, document, integer, require, typed_json
from generic_observations import ordinary_pair
from generic_process import ProcessResult, ProcessSpec, atomic_json, file_hash
from generic_sources import inventory
from mux2_gate_inputs import Json
from read_summary_normalise import first_difference, normalise_pair

SUPPORTED: Final = frozenset({'identity.baseline_samples', 'identity.baseline_twins', 'determinism.baseline', 'core.baseline10'})
RETIRED: Final = frozenset({'interventions_applied_total', 'fire_blocked_settlements', 'cold_snap_nights_left', 'wet_burn_wood_total', 'wet_burn_wood'})
_normalise: Callable[[Json, Json, str | None], tuple[object, object, list[str]]] = normalise_pair


def prepare(registry: Registry, plans: tuple[Invocation, ...], products: Products) -> None:
    """Require the complete paired workload for each selected protected fixture."""
    require(bool(plans), '/plans', 'empty baseline selection')
    require(len({p.id for p in plans}) == len(plans) and len({p.out for p in plans}) == len(plans), '/plans', 'duplicate identity/output')
    selected = {p.fixture_id for p in plans}
    require(selected <= SUPPORTED, '/plans', 'nonbaseline or unknown fixture')
    require(not selected.intersection({'identity.baseline_samples', 'identity.baseline_twins'}) or
            {'identity.baseline_samples', 'identity.baseline_twins'} <= selected, '/plans', 'identity requires samples and twins')
    fixtures = {f.id: f for f in registry.fixtures}
    for name in selected:
        require(name in fixtures, name, 'unregistered fixture')
        fixture = fixtures[name]
        require(parse_fixture(document(fixture.metadata_json).value) == fixture and fixture.adapter == 'baseline_sidecar', name, 'registry metadata differs from compiled workload')
        rows = [p for p in plans if p.fixture_id == name]
        require(len(rows) == 2 * len(fixture.repeats) and {(p.arm, p.repeat) for p in rows} ==
                {(arm, repeat) for arm in ('base', 'candidate') for repeat in fixture.repeats}, name, 'incomplete paired repeats')
        for plan in rows:
            product = products.arm(plan.arm)
            values = {'probe': str(product.executable), 'inputs': str(product.input_root), 'harness_root': str(product.input_root),
                      'out': str(plan.out), 'chronicle_out': str(plan.out / 'chronicle'), 'commands_out': str(plan.out / 'commands')}
            argv = tuple(arg.format_map(values) for arg in fixture.argv) + ('--gate-observations',)
            require(plan.id == f'{plan.arm}/{name}/{plan.repeat}' and plan.argv == argv and plan.cwd == product.input_root
                    and plan.seeds == fixture.seeds and plan.roles == fixture.roles and not plan.blockers
                    and not plan.environment and plan.generated_input is None, plan.id, 'changed registered baseline invocation')
            require(not plan.out.is_relative_to(product.input_root), plan.id, 'baseline evidence must be outside product input tree')
            for other in plans:
                require(other is plan or not plan.out.is_relative_to(other.out), plan.id, 'overlapping output roots')
            removed = 'tools/v8/identity_removed_keys.txt'
            require(removed in dict(product.input_files) and file_hash(product.input_root / removed) == dict(product.input_files)[removed], plan.id, 'unbound retired-key policy')
            keys = {line.split('#', 1)[0].strip() for line in (product.input_root / removed).read_text().splitlines()} - {''}
            require(frozenset(keys) == RETIRED, plan.id, 'protected raw identity exclusions changed')


def raw_digests(plan: Invocation) -> dict[str, str]:
    """Match standing line-wise raw identity, preserving order and zero-only retirements."""
    assert plan.argv is not None, 'prepared argv'
    scene = plan.argv[plan.argv.index('--scene') + 1]
    expected = {f'{scene}_s{seed}{suffix}.jsonl' for seed in plan.seeds for suffix in ('', '_deaths')}
    require({p.name for p in plan.out.glob('*.jsonl')} == expected, plan.id, 'missing or extra raw sample/death files')
    result: dict[str, str] = {}
    for name in sorted(expected):
        path = plan.out / name
        require(path.is_file() and not path.is_symlink(), name, 'unsafe raw file')
        lines = path.read_text().splitlines()
        require(bool(lines) or name.endswith('_deaths.jsonl'), name, 'missing raw samples')
        digest = hashlib.sha256()
        for line in lines:
            row = document(line).mapping()
            _ = row.pop('git_sha', None)
            for key in RETIRED.intersection(row):
                require(row[key] in (0, 0.0, False, None, '', [], {}), name, 'retired raw key has nonzero value')
                del row[key]
            digest.update(json.dumps(row, sort_keys=True).encode() + b'\n')
        result[name] = digest.hexdigest()
    return result


def summary(plan: Invocation) -> Json:
    """Validate baseline seed/day coverage before applying the committed normalizer."""
    assert plan.argv is not None, 'prepared argv'
    row = document((plan.out / 'summary.json').read_text())
    row.field('schema_version').equals(13)
    row.field('scene').equals(plan.argv[plan.argv.index('--scene') + 1])
    row.field('seeds').equals({'start': plan.seeds[0], 'end_exclusive': plan.seeds[-1] + 1})
    require(tuple(integer(seed.field('seed')) for seed in row.field('runs').array()) == plan.seeds, plan.id, 'summary run coverage differs')
    days = int(plan.argv[plan.argv.index('--days') + 1])
    require(set(row.field('days').mapping()) == {str(day) for day in range(1, days + 1)}, plan.id, 'summary day coverage differs')
    twin = '--check-determinism' in plan.argv
    row.field('determinism_checked').equals(twin)
    if twin:
        stdout = (plan.out / 'stdout.log').read_text()
        require('DETERMINISM PASS' in stdout and 'DETERMINISM FAIL' not in stdout, plan.id, 'native determinism oracle failed')
    return row.value


def identity(base: Invocation, candidate: Invocation, base_sha: str) -> dict[str, Json]:
    """Protected comparison; no semantic differences can be declared away."""
    a, b, excluded = _normalise(summary(base), summary(candidate), base_sha)
    difference = first_difference(typed_json(a), typed_json(b))
    require(difference is None, base.fixture_id, f'protected baseline summary differs: {difference}')
    left, right = raw_digests(base), raw_digests(candidate)
    require(left == right, base.fixture_id, 'protected baseline raw samples/deaths differ')
    result: dict[str, Json] = {'excluded_summary_paths': [path for path in excluded],
                              'raw_sample_digests': {key: value for key, value in left.items()}}
    if base.fixture_id == 'identity.baseline_samples':
        expected = {f'{kind}_{seed}.json' for seed in base.seeds for kind in ('chronicle', 'relations')}
        for plan in (base, candidate):
            folder = plan.out / 'chronicle'
            require(folder.is_dir() and {p.name for p in folder.iterdir()} == expected, plan.id, 'chronicle/relations coverage differs')
            for name in expected:
                path = folder / name
                require(path.is_file() and not path.is_symlink(), name, 'unsafe chronicle')
                parsed = document(path.read_text())
                if 'entries' in parsed.mapping():
                    _ = parsed.field('entries').array()
        result['chronicle'] = compare_chronicles(base.out / 'chronicle', candidate.out / 'chronicle')
    return result


def run_registered(registry: Registry, plans: tuple[Invocation, ...], products: Products, output: Path, *,
                   retain: Callable[[ProcessSpec, ProcessResult], None] | None = None) -> ComponentResult:
    """Execute selected paired baseline workloads and retain all failures and safety rows."""
    prepare(registry, plans, products)
    require(not output.exists(), str(output), 'component receipt exists')
    require(all(not output.is_relative_to(p.out) for p in plans), str(output), 'component receipt overlaps producer output')
    output.parent.mkdir(parents=True, exist_ok=True)
    results: dict[str, ProcessResult] = {}
    executions: list[Json] = []
    errors: list[Json] = []
    artifacts: dict[str, dict[str, str]] = {}
    for plan in plans:
        try:
            result = (run_invocation(plan, products.arm(plan.arm), ()) if retain is None else
                      run_invocation(plan, products.arm(plan.arm), (), retain=retain))
            results[plan.id] = result
            executions.append({'id': plan.id, 'receipt': str(result.receipt), 'receipt_sha256': file_hash(result.receipt), 'status': result.status})
            artifacts[plan.id] = inventory(plan.out, [p.name for p in plan.out.iterdir()])
            require(result.status == 'PASS', plan.id, 'baseline producer failed')
        except (OSError, ValueError) as error:
            if retain is not None and plan.id not in results:
                raise
            errors.append({'invocation': plan.id, 'error': str(error)})
    pairs: list[Json] = []
    fixtures = {f.id: f for f in registry.fixtures}
    for base in (p for p in plans if p.arm == 'base'):
        candidate = next(p for p in plans if p.arm == 'candidate' and p.fixture_id == base.fixture_id and p.repeat == base.repeat)
        worlds: list[Json] = []
        row: dict[str, Json] = {'fixture': base.fixture_id, 'repeat': base.repeat, 'status': 'FAIL', 'worlds': worlds}
        pairs.append(row)
        try:
            require(base.id in results and candidate.id in results, base.id, 'producer incomplete')
            before = [artifacts[p.id] for p in (base, candidate)]
            left = fixed_observations(fixtures[base.fixture_id], base, results[base.id])
            right = fixed_observations(fixtures[base.fixture_id], candidate, results[candidate.id])
            require(left.keys() == right.keys(), base.id, 'native world coverage differs')
            failed = False
            for key in left:
                world: dict[str, Json] = {'seed': key[0], 'instance': key[1], 'status': 'FAIL',
                                         'knowledge_enabled': left[key].knowledge_enabled}
                worlds.append(world)
                try:
                    world['safety'] = ordinary_pair(left[key], right[key])
                    world['status'] = 'PASS'
                except GateContractError as error:
                    world['error'] = str(error)
                    failed = True
            try:
                row['identity'] = identity(base, candidate, products.base.source_sha)
            except (OSError, ValueError) as error:
                row['identity_error'] = str(error)
                failed = True
            for plan, hashes in zip((base, candidate), before, strict=True):
                require(inventory(plan.out, [child.name for child in plan.out.iterdir()]) == hashes, plan.id, 'outputs changed during comparison')
                row[plan.arm + '_outputs'] = {key: value for key, value in hashes.items()}
            require(not failed, base.fixture_id, 'protected baseline identity or world safety failed')
            row['status'] = 'PASS'
        except (OSError, ValueError, KeyError) as error:
            row['error'] = str(error)
            errors.append({'fixture': base.fixture_id, 'error': str(error)})
    for plan in plans:
        if plan.id not in artifacts:
            continue
        try:
            require(inventory(plan.out, [p.name for p in plan.out.iterdir()]) == artifacts[plan.id], plan.id, 'producer artifacts changed during component')
        except (OSError, ValueError) as error:
            errors.append({'postflight': plan.id, 'error': str(error)})
    for product in (products.base, products.candidate):
        try:
            require(file_hash(product.executable) == product.executable_sha256 and
                    inventory(product.input_root, [name for name, _ in product.input_files]) == dict(product.input_files),
                    product.source_sha, 'product inputs or executable changed during component')
        except (OSError, ValueError) as error:
            errors.append({'postflight': product.source_sha, 'error': str(error)})
    status = 'FAIL' if errors else 'PASS'
    atomic_json(output, {'schema': 'protected_baseline_component_v1', 'status': status, 'publication_authority': False,
                         'registry_sha256': registry.sha256, 'base': products.base.source_sha, 'candidate': products.candidate.source_sha,
                         'executions': executions, 'pairs': pairs, 'errors': errors,
                         'producer_outputs': {name: dict(hashes) for name, hashes in artifacts.items()}})
    return ComponentResult(output, status)
