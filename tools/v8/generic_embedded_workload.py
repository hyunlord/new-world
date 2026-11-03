"""Actual candidate step11 build followed by registered paired determinism, no full-gate claim.

Only the registered tail is covered: literal original shell/cwd/echo equivalence
and intervening smoke chronology remain separate. Historical checks use retained
completed authentication; they never replace old hashes with today's target.
"""
import json
from dataclasses import dataclass, replace
from pathlib import Path
from typing import Literal

import generic_baseline_run as baseline
import generic_embedded_build as build
from gate_oracle_child import import_generated
from generic_cli_godot_products import BaseProbeWitness, check_base_probe_witness
from generic_fixture_registry import (
    ExpansionContext,
    Invocation,
    Registry,
    expand,
    load_registry,
)
from generic_fixture_run import ComponentResult, Products
from generic_fixtures import ProbeProduct
from generic_gate import require
from generic_process import (
    ProcessResult,
    ProcessSpec,
    atomic_json,
    environment_for,
    file_hash,
)
from generic_setup_bridge import held_lane, immutable_inputs
from generic_workspace_cases import ListingProcess, process_output
from generic_workspace_endpoints import EvidenceAnchor, anchor, generated
from mux2_gate_inputs import Json


@dataclass(frozen=True, slots=True)
class Request:
    base: BaseProbeWitness
    candidate: build.EmbeddedRequest
    registry: Registry
    registry_path: Path
    output: Path


@dataclass(frozen=True, slots=True)
class Child:
    binding: ListingProcess
    output: EvidenceAnchor


@dataclass(frozen=True, slots=True)
class _Proof:
    request: Request
    builds: tuple[build.EmbeddedEndpoint, ...]
    children: tuple[Child, ...]
    component: ComponentResult
    output: EvidenceAnchor
    external: tuple[tuple[Path, str, int], ...]


@dataclass(frozen=True, slots=True)
class Endpoint:
    request: Request
    builds: tuple[build.EmbeddedEndpoint, ...]
    children: tuple[Child, ...]
    component: ComponentResult
    receipt: Path
    receipt_sha256: str
    status: Literal['PASS_REGISTERED_STEP11_ONLY', 'FAIL']
    proof: _Proof
    original_shell_envelope: Literal['NOT_QUALIFIED'] = 'NOT_QUALIFIED'
    publication_authority: Literal[False] = False


# Gate-process lifetime issuance, not protection against hostile module mutation.
_ISSUED: dict[int, Endpoint] = {}

class WorkloadFailure(RuntimeError):
    def __init__(self, primary: BaseException, secondary: BaseException | None, receipt: Path | None) -> None:
        super().__init__('step11 failed; original exception retained')
        self.primary: BaseException = primary
        self.secondary: BaseException | None = secondary
        self.receipt: Path | None = receipt


class OwnershipUnknown(WorkloadFailure):
    """Keep the actual caller descriptor; no inferred child settlement."""


def _history(proof: _Proof) -> None:
    require(load_registry(proof.request.registry_path) == proof.request.registry, '/registry', 'registry changed')
    check_base_probe_witness(proof.request.base)
    for endpoint in proof.builds:
        build.check_endpoint(endpoint)
        immutable_inputs(endpoint.request.prepared)
    _sealed(proof)


def _sealed(proof: _Proof) -> None:
    require(file_hash(proof.request.registry_path) == proof.request.registry.sha256, '/registry', 'registry bytes changed')
    ready = proof.request.base.ready
    require(generated(ready.setup.base) == ready.after, '/base', 'BASE endpoint changed')
    for item in (*ready.setup.history, *ready.history):
        item.recheck()
    base = proof.request.base.product
    require(file_hash(base.executable) == base.executable_sha256, '/base', 'original BASE probe changed')
    for name, digest in (*base.input_files, *proof.request.base.runtime_inputs):
        require(file_hash(base.input_root/name) == digest, '/base', 'BASE runtime/source input changed')
    for endpoint in proof.builds:
        for item in endpoint.history:
            item.recheck()
    for child in proof.children:
        child.output.recheck()
    proof.output.recheck()
    # Concrete source/tool bytes are checked after the authentication callbacks.
    for endpoint in proof.builds:
        prepared = endpoint.request.prepared
        for name, digest in prepared.materialized:
            require(file_hash(prepared.inputs.root/name) == digest, '/source', 'retained source changed')
        for path, digest in prepared.external:
            require(file_hash(path) == digest, '/source', 'retained external input changed')
    # Preserve the original complete source namespace after all semantic callbacks.
    ready.setup.closure.check(ready.setup.products.builds)
    for path, digest, mode in proof.external:
        require(file_hash(path) == digest and path.lstat().st_mode == mode,
                '/external', 'original external bytes/mode changed')


def check_endpoint(endpoint: Endpoint) -> None:
    """Authenticate actual retained completion after later authorized target evolution."""
    require(_ISSUED.get(id(endpoint)) is endpoint, '/endpoint', 'actual issued completion required')
    proof = endpoint.proof
    require(endpoint.request is proof.request and endpoint.builds is proof.builds
            and endpoint.children is proof.children and endpoint.component is proof.component,
            '/endpoint', 'reconstructed endpoint identities')
    require(endpoint.receipt == proof.request.output/'receipt.json'
            and file_hash(endpoint.receipt) == endpoint.receipt_sha256,
            '/endpoint', 'receipt changed')
    expected = 'PASS_REGISTERED_STEP11_ONLY' if proof.component.status == 'PASS' else 'FAIL'
    require(endpoint.status == expected and not endpoint.publication_authority
            and endpoint.original_shell_envelope == 'NOT_QUALIFIED', '/endpoint', 'authority changed')
    _history(proof)


def _current(endpoints: tuple[build.EmbeddedEndpoint, ...]) -> None:
    for endpoint in endpoints:
        build.check_endpoint(endpoint)
    for endpoint in endpoints:
        require(generated(endpoint.request.prepared) == endpoint.after,
                '/generated', 'unadmitted generated-state change')
    for endpoint in endpoints:
        for item in endpoint.history:
            item.recheck()


def _product(endpoint: build.EmbeddedEndpoint) -> ProbeProduct:
    prepared = endpoint.request.prepared
    path = prepared.inputs.root/'rust/target/release/worldsim_probe'
    require(path.resolve() == path and path.is_file() and not path.is_symlink(), '/probe', 'missing canonical producer output')
    digest = file_hash(path)
    _current((endpoint,))
    return ProbeProduct(prepared.inputs.source_sha, path, digest, prepared.inputs.root, prepared.materialized)


def _plans(request: Request, products: Products) -> tuple[Invocation, ...]:
    plans: list[Invocation] = []
    for arm in ('base', 'candidate'):
        product = products.arm(arm)
        context = ExpansionContext(product.input_root, product.input_root, request.output/'runs',
                                   products.base.executable, products.candidate.executable)
        plans.extend(plan for plan in expand(request.registry, context)
                     if plan.arm == arm and plan.fixture_id == 'determinism.baseline')
    return tuple(plans)


def _external(request: Request) -> tuple[tuple[Path, str, int], ...]:
    expected: dict[Path, str] = {}
    setup = request.base.ready.setup
    for prepared in (setup.base, setup.candidate.prepared):
        for path, digest in (*prepared.external, *((tool.path, tool.sha256) for tool in prepared.inputs.tools)):
            require(path not in expected or expected[path] == digest, '/external', 'conflicting original external input')
            expected[path] = digest
    result: list[tuple[Path, str, int]] = []
    for path, digest in sorted(expected.items()):
        require(file_hash(path) == digest, '/external', 'original external bytes changed')
        result.append((path, digest, path.lstat().st_mode))
    return tuple(result)


def run_owned(request: Request) -> Endpoint:
    """Run real producers and retain actual paired child waits; caller owns the lane."""
    output = request.output
    children: list[Child] = []
    builds: list[build.EmbeddedEndpoint] = []
    in_flight = False
    receipt: Path | None = None
    endpoint: Endpoint | None = None
    try:
        require(request.candidate.step == 11, '/step', 'only step11 supported')
        held_lane(request.candidate.descriptor)
        check_base_probe_witness(request.base)
        external = _external(request)
        require(request.candidate.prepared is request.base.ready.setup.candidate.prepared,
                '/candidate', 'foreign actual prepared candidate')
        require(output.is_absolute() and output == output.resolve() and output.parent.is_dir()
                and not output.exists(), '/output', 'fresh canonical output required')
        roots = (request.base.product.input_root, request.candidate.prepared.inputs.root)
        require(not roots[0].is_relative_to(roots[1]) and not roots[1].is_relative_to(roots[0]), '/roots', 'independent roots required')
        require(request.candidate.output == output/'candidate-build', '/build', 'wrong build output')
        for root in roots:
            require(not output.is_relative_to(root) and not root.is_relative_to(output), '/output', 'product overlap')
        require(request.registry_path.is_absolute() and request.registry_path.resolve() == request.registry_path
                and not request.registry_path.is_symlink(), '/registry', 'canonical registry required')
        require(load_registry(request.registry_path) == request.registry, '/registry', 'changed registry object')
        output.mkdir()
        receipt = output/'receipt.json'
        builds.append(build.run_owned(request.candidate))
        endpoints = tuple(builds)
        _current(endpoints)
        check_base_probe_witness(request.base)
        base_inputs = dict(request.base.product.input_files)
        for name, digest in request.base.runtime_inputs:
            require(name not in base_inputs or base_inputs[name] == digest, '/base', 'conflicting runtime input')
            base_inputs[name] = digest
        products = Products(replace(request.base.product, input_files=tuple(sorted(base_inputs.items()))), _product(endpoints[0]))
        plans = _plans(request, products)
        baseline.prepare(request.registry, plans, products)
        _current(endpoints)
        require(set(import_generated(output)) == {'.',
                *('candidate-build/'+k if k != '.' else 'candidate-build' for k in import_generated(request.candidate.output))},
                '/output', 'unexpected pre-tail namespace')

        def retain(spec: ProcessSpec, result: ProcessResult) -> None:
            nonlocal in_flight
            # First operation freezes returned output before any semantic/auth callback.
            frozen = anchor(spec.output_root)
            expected = plans[len(children)]
            require(spec.argv == expected.argv and spec.output_root == expected.out,
                    '/child', 'unexpected execution order')
            binding = ListingProcess(spec, result, file_hash(result.receipt))
            required = {str(spec.input_root/name): digest for name, digest in spec.inputs}
            required.update({str(path): digest for path, digest in spec.external_inputs})
            required[str(spec.executable)] = spec.executable_sha256
            try:
                _ = process_output(binding, spec.argv, spec.cwd, environment_for(spec), required)
                check_base_probe_witness(request.base)
                _current(endpoints)
                for old in children:
                    old.output.recheck()
                frozen.recheck()
            except BaseException as error:
                raise OwnershipUnknown(error, None, receipt) from error
            children.append(Child(binding, frozen))
            in_flight = len(children) != len(plans)

        in_flight = True
        component = baseline.run_registered(request.registry, plans, products, output/'baseline.json', retain=retain)
        terminal_before = anchor(output)
        allowed = {'.', 'baseline.json'}
        for folder in (request.candidate.output, *(plan.out for plan in plans)):
            relative = folder.relative_to(output)
            allowed.update(parent.as_posix() for parent in relative.parents if parent != Path('.'))
            allowed.update((relative/Path(name)).as_posix() for name in import_generated(folder))
        require(set(import_generated(output)) == allowed, '/output', 'unexpected completed namespace')
        require(len(children) == len(plans), '/children', 'missing actual native returns')
        _current(endpoints)
        for child in children:
            child.output.recheck()
        held_lane(request.candidate.descriptor)
        terminal_before.recheck()
        state: Literal['PASS_REGISTERED_STEP11_ONLY', 'FAIL'] = 'PASS_REGISTERED_STEP11_ONLY' if component.status == 'PASS' else 'FAIL'
        row: dict[str, Json] = {'schema':'embedded_step11.v1','status':state,'original_shell_envelope':'NOT_QUALIFIED',
            'publication_authority':False,'component':str(component.receipt),'component_sha256':file_hash(component.receipt),
            'builds':[str(item.receipt) for item in endpoints],
            'children':[{'receipt':str(child.binding.result.receipt),'sha256':child.binding.receipt_sha256,
                         'pid':child.binding.result.pid,'native_exit':child.binding.result.native_exit} for child in children]}
        before = import_generated(output)
        expected_text = json.dumps(row,sort_keys=True,indent=2)+'\n'
        atomic_json(receipt,row)
        require(receipt.read_text() == expected_text, '/receipt', 'publication content drift')
        require({key:value for key,value in import_generated(output).items() if key != 'receipt.json'} == before,
                '/output', 'publication namespace drift')
        proof = _Proof(request,endpoints,tuple(children),component,anchor(output),external)
        endpoint = Endpoint(request,endpoints,proof.children,component,receipt,file_hash(receipt),state,proof)
        _ISSUED[id(endpoint)] = endpoint
        check_endpoint(endpoint)
        _current(endpoints)
        held_lane(request.candidate.descriptor)
        # Last callbacks cannot replace an accepted output/history anchor.
        _history(proof)
        for item in endpoints:
            require(generated(item.request.prepared) == item.after, '/generated', 'late generated mutation')
        _sealed(proof)
        return endpoint
    except BaseException as error:
        if endpoint is not None:
            _ = _ISSUED.pop(id(endpoint), None)
        unknown = in_flight or isinstance(error, (build.EmbeddedOwnershipUnknown, OwnershipUnknown))
        kind = OwnershipUnknown if unknown else WorkloadFailure
        failure_path = output/'failure.json' if receipt is not None else None
        if failure_path is not None:
            try:
                atomic_json(failure_path, {'schema':'embedded_step11.failure.v1','status':'FAIL',
                    'ownership':'UNKNOWN_KEEP_CALLER_LANE' if unknown else 'NO_INFLIGHT_OWNED_CALL',
                    'builds':[str(item.receipt) for item in builds],
                    'children':[{'receipt':str(child.binding.result.receipt),'native_exit':child.binding.result.native_exit} for child in children]})
            except BaseException as secondary:
                raise kind(error,secondary,failure_path) from secondary
        raise kind(error,None,failure_path) from error
