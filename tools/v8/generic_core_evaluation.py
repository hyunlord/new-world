# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -B -m unittest test_generic_core_evaluation
"""Re-evaluate step44's actual paired matrix; never launch worlds or grant authority."""
import json
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

import generic_baseline_run as baseline
import generic_direct_run as direct
import generic_fixture_run as ordinary
from generic_declarations import Declarations
from generic_fixture_registry import ExpansionContext, Invocation, Registry, expand
from generic_fixture_run import Products
from generic_fixtures import fixed_observations
from generic_gate import (
    document,
    integer,
    require,
    typed_json,
    validate_declared_fixture,
)
from generic_observations import ordinary_pair
from generic_process import (
    ProcessResult,
    ProcessSpec,
    atomic_json,
    environment_for,
    file_hash,
)
from generic_sources import inventory
from generic_workspace_cases import ListingProcess, process_output
from mux2_gate_inputs import Json, Node
from read_summary_normalise import first_difference, normalise_pair

_normalise: Callable[[Json, Json, str | None], tuple[object, object, list[str]]] = normalise_pair


@dataclass(frozen=True, slots=True)
class PriorArtifacts:
    """Digest and entire inventory captured by the owner at actual producer return."""
    receipt: Path
    sha256: str
    files: tuple[tuple[str, str], ...]


@dataclass(frozen=True, slots=True)
class EvaluationResult:
    receipt: Path
    sha256: str
    status: Literal['FAIL', 'PENDING']


def recheck(prior: PriorArtifacts) -> None:
    require(prior.receipt.is_absolute() and prior.receipt == prior.receipt.resolve(strict=True), '/prior', 'canonical receipt required')
    require(prior.receipt.name == 'receipt.json' and file_hash(prior.receipt) == prior.sha256, '/prior', 'original receipt digest differs')
    require(len(dict(prior.files)) == len(prior.files) and dict(prior.files).get('receipt.json') == prior.sha256,
            '/prior', 'missing/duplicate original inventory binding')
    root = prior.receipt.parent
    require(inventory(root, [p.name for p in root.iterdir()]) == dict(prior.files), '/prior', 'original artifact inventory changed')


def plans_for(registry: Registry, products: Products, root: Path) -> tuple[Invocation, ...]:
    selected = direct.step_fixtures(44)
    plans: list[Invocation] = []
    for arm in ('base', 'candidate'):
        product = products.arm(arm)
        context = ExpansionContext(product.input_root, product.input_root, root/'runs', products.base.executable, products.candidate.executable)
        plans.extend(p for p in expand(registry, context) if p.arm == arm and p.fixture_id in selected)
    require({p.fixture_id for p in plans} == set(selected), '/coverage', 'missing registered CORE family')
    return tuple(plans)


def authenticate_prior(registry: Registry, products: Products, declarations: Declarations,
                       prior: PriorArtifacts, plans: tuple[Invocation, ...]) -> None:
    recheck(prior)
    require(declarations.base == products.base.source_sha and declarations.candidate == products.candidate.source_sha,
            '/declarations', 'sources differ from products')
    root = prior.receipt.parent; node = document(prior.receipt.read_text())
    fields: dict[str, Json] = {'schema': 'direct_step_components_v1', 'step': 44,
        'fixtures': list(direct.step_fixtures(44)), 'obligations': ['44:'+f for f in direct.step_fixtures(44)],
        'registry_sha256': registry.sha256, 'base': declarations.base, 'candidate': declarations.candidate,
        'declaration_manifest_sha256': declarations.source_manifest_sha256, 'publication_authority': False,
        'whole_step_complete': False, 'original_order_executed': False, 'full_gate_status': 'NOT_QUALIFIED'}
    for key, value in fields.items():node.field(key).equals(value)
    # The old producer result is an additional constraint, never the semantic proof.
    node.field('status').equals('PASS');node.field('errors').equals([])
    rows = node.field('components').array()
    require(len(rows) == 2 and {r.field('component').text() for r in rows} == {'baseline', 'ordinary'}, '/coverage', 'CORE component coverage differs')
    for row in rows:
        name = row.field('component').text(); path = root/(name+'.json')
        row.field('receipt').equals(str(path));row.field('receipt_sha256').equals(file_hash(path));row.field('status').equals('PASS')
        selected = tuple(p for p in plans if (p.fixture_id in baseline.SUPPORTED) == (name == 'baseline'))
        expected: list[Json] = [{'id': p.id, 'fixture': p.fixture_id, 'arm': p.arm, 'repeat': p.repeat,
            'seeds': list(p.seeds), 'roles': list(p.roles), 'cwd': str(p.cwd), 'out': str(p.out),
            'argv': list(p.argv) if p.argv is not None else None} for p in sorted(selected, key=lambda p: (direct.step_fixtures(44).index(p.fixture_id), p.arm))]
        row.field('invocations').equals(expected)
        raw = document(path.read_text());raw.field('status').equals('PASS');raw.field('errors').equals([])
        raw.field('schema').equals('protected_baseline_component_v1' if name == 'baseline' else 'ordinary_colony_component_v1')
        for key in ('registry_sha256', 'base', 'candidate', 'publication_authority'):raw.field(key).equals(fields[key])
        if name == 'ordinary':raw.field('declaration_manifest_sha256').equals(declarations.source_manifest_sha256)
        manifests = raw.field('producer_outputs').mapping()
        require(set(manifests) == {p.id for p in selected}, '/coverage', 'missing/extra producer manifest')
        for p in selected:
            require(inventory(p.out, [x.name for x in p.out.iterdir()]) == Node(manifests[p.id]).mapping(), p.id, 'producer artifact changed')
        if name == 'baseline':baseline.prepare(registry, selected, products)
        else:ordinary.prepare(registry, selected, products)
    for arm in ('base', 'candidate'):
        p = products.arm(arm); bound = node.field('products').field(arm)
        for key, value in (('root', str(p.input_root)), ('executable', str(p.executable)), ('executable_sha256', p.executable_sha256)):bound.field(key).equals(value)
        require(file_hash(p.executable) == p.executable_sha256 and inventory(p.input_root, [n for n, _ in p.input_files]) == dict(p.input_files), arm, 'product changed')


def process(plan: Invocation, products: Products, prior: PriorArtifacts) -> ProcessResult:
    product = products.arm(plan.arm)
    require(plan.argv is not None and plan.generated_input is None, plan.id, 'unsupported invocation')
    assert plan.argv is not None
    path = plan.out/'process.json'; node = document(path.read_text())
    # Decode actual producer values; no invented caller PASS or synthetic result.
    code = integer(node.field('native_exit')); pid = integer(node.field('pid'))
    node.field('status').equals('PASS');require(code == 0 and pid > 0, plan.id, 'original child failed')
    result = ProcessResult(path, 'PASS', code, pid)
    environment = dict(plan.environment, WORLDSIM_GIT_SHA=product.source_sha, WORLDSIM_GATE_OBSERVER='1')
    spec = ProcessSpec(plan.argv, plan.cwd, product.executable, product.executable_sha256, product.input_root,
                       tuple(sorted(product.input_files)), plan.out, tuple(sorted(environment.items())))
    captured = dict(prior.files).get(str(path.relative_to(prior.receipt.parent)))
    require(captured is not None, plan.id, 'missing captured native receipt digest')
    assert captured is not None
    _ = process_output(ListingProcess(spec, result, captured), spec.argv, spec.cwd, environment_for(spec),
                       {str(product.executable): product.executable_sha256})
    return result


def evaluate_pair(registry: Registry, products: Products, declarations: Declarations,
                  prior: PriorArtifacts, base: Invocation, candidate: Invocation) -> dict[str, Json]:
    fixture = next(f for f in registry.fixtures if f.id == base.fixture_id)
    left = fixed_observations(fixture, base, process(base, products, prior))
    right = fixed_observations(fixture, candidate, process(candidate, products, prior))
    require(left.keys() == right.keys(), base.id, 'world occurrence coverage differs')
    protected = base.fixture_id in baseline.SUPPORTED
    comparison: dict[str, Json]
    if protected:
        comparison = baseline.identity(base, candidate, declarations.base)
        difference = None
    else:
        summaries = [document((p.out/'summary.json').read_text()) for p in (base, candidate)]
        for p, summary in zip((base, candidate), summaries, strict=True):
            summary.field('scene').equals('colony')
            summary.field('days').equals(document(fixture.metadata_json).field('horizon').field('days').value)
            require(tuple(integer(row.field('seed')) for row in summary.field('seeds').array()) == p.seeds, p.id, 'summary coverage differs')
            if p.argv is not None and '--check-determinism' in p.argv:
                stdout = (p.out/'stdout.log').read_text()
                require('DETERMINISM PASS' in stdout and 'DETERMINISM FAIL' not in stdout, p.id, 'determinism failed')
        a, b, excluded = _normalise(summaries[0].value, summaries[1].value, declarations.base)
        difference = first_difference(typed_json(a), typed_json(b))
        comparison = {'semantic_equal': difference is None, 'difference': difference, 'excluded_paths': list(excluded)}
    worlds: list[Json] = []
    for key in left:
        safety = ordinary_pair(left[key], right[key])
        if not protected:
            admission = declarations.validator_fields(fixture.id)
            admission.update(semantic_equal=difference is None, safety=safety)
            validate_declared_fixture(json.dumps(admission))
        worlds.append({'seed': key[0], 'instance': key[1], 'safety': safety})
    return {'fixture': fixture.id, 'repeat': base.repeat, 'comparison': comparison, 'worlds': worlds, 'status': 'PASS'}


def evaluate(registry: Registry, products: Products, declarations: Declarations,
             prior: PriorArtifacts, output: Path) -> EvaluationResult:
    """Read-only aggregate of the exact CORE matrix; only fresh external audit output."""
    require(output.is_absolute() and output == output.resolve() and not output.exists(), '/output', 'fresh canonical output required')
    for root in (prior.receipt.parent, products.base.input_root, products.candidate.input_root):
        require(not output.is_relative_to(root) and not root.is_relative_to(output), '/output', 'overlapping evidence/product roots')
    output.mkdir(parents=True)
    errors: list[Json] = []; pairs: list[Json] = []; original: list[Json] = []
    try:
        recheck(prior)
        for name in ('receipt', 'baseline', 'ordinary'):
            path = prior.receipt.parent/(name+'.json')
            raw = document(path.read_text())
            original.append({'path': str(path), 'sha256': file_hash(path), 'status': raw.field('status').text(), 'errors': raw.field('errors').value})
        plans = plans_for(registry, products, prior.receipt.parent)
        authenticate_prior(registry, products, declarations, prior, plans)
        for base in (p for p in plans if p.arm == 'base'):
            candidate = next(p for p in plans if p.arm == 'candidate' and p.fixture_id == base.fixture_id and p.repeat == base.repeat)
            try:pairs.append(evaluate_pair(registry, products, declarations, prior, base, candidate))
            except (OSError, ValueError) as error:errors.append({'fixture': base.fixture_id, 'repeat': base.repeat, 'error': str(error)})
        authenticate_prior(registry, products, declarations, prior, plans)
    except (OSError, ValueError) as error:errors.append({'original_artifacts': str(error)})
    status: Literal['FAIL', 'PENDING'] = 'FAIL' if errors else 'PENDING'
    path = output/'evaluation.json'
    atomic_json(path, {'schema': 'generic_core_evaluation_v1', 'step': 45, 'status': status,
        'evaluated_scope': 'FAIL' if errors else 'PASS', 'fixtures': list(direct.step_fixtures(44)),
        'base': products.base.source_sha, 'candidate': products.candidate.source_sha, 'registry_sha256': registry.sha256,
        'declaration_manifest_sha256': declarations.source_manifest_sha256, 'prior': {'path': str(prior.receipt), 'sha256': prior.sha256, 'files': dict(prior.files)},
        'pairs': pairs, 'errors': errors, 'original_results': original, 'ordinary_safety': 'PENDING_SOURCE_BUILD_QUALIFICATION',
        'source_build_authority': 'CALLER_OWNED_PENDING', 'global_coverage': 'PENDING', 'final_conjunction': 'PENDING',
        'generic_performance': 'PENDING_SEPARATE_BENCH_AUTHENTICATION',
        'legacy_core': {name: 'HISTORICAL_NOT_FRESH_ACCEPTANCE' for name in ('heldout_outcomes','heldout_why','story','performance')},
        'publication_authority': False, 'full_gate_claim': False})
    return EvaluationResult(path, file_hash(path), status)
