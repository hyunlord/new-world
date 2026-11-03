"""Live two-arm setup admission; auxiliary BASE jobs are not candidate gate steps."""
import stat
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path
from typing import Literal, final

from gate_oracle_child import import_generated
from generic_build import recheck_workspace_source
from generic_cli_prepare import PairedSetupSourceClosure, paired_setup_source_closure
from generic_cli_products import CliProducts, recheck_products
from generic_cli_setup_handoff import (
    SetupProductHandoff,
    begin_paired_setup_handoff,
    check_candidate_setup,
)
from generic_fixture_run import Products
from generic_fixtures import ProbeProduct
from generic_gate import document, require
from generic_process import file_hash
from generic_setup_bridge import (
    BridgeResult,
    CargoWaitEvidence,
    immutable_inputs,
    run_bridge_owned,
)
from generic_setup_clippy import ClippyEndpoint
from generic_setup_import import (
    ImportFailure,
    ImportOwner,
    PriorBridge,
    start_import_pair,
)
from generic_setup_prepare import BRIDGE, PreparedSetup, recipe
from generic_ui_smoke_run import check_product
from generic_workspace_endpoints import (
    EvidenceAnchor,
    GeneratedState,
    ImportPredecessor,
    admit_import,
    anchor,
    encoded,
    generated,
)


@dataclass(frozen=True, slots=True)
class GodotSetup:
    products: CliProducts
    base: PreparedSetup
    candidate: SetupProductHandoff
    closure: PairedSetupSourceClosure
    before: tuple[GeneratedState, GeneratedState]
    history: tuple[EvidenceAnchor, ...]
    selected: tuple[tuple[Path, str, int], ...]


@dataclass(frozen=True, slots=True)
class BaseGodotReady:
    setup: GodotSetup
    predecessor: ImportPredecessor
    after: GeneratedState
    history: tuple[EvidenceAnchor, ...]
    role: Literal['AUXILIARY_BASE_BEFORE_CANDIDATE_STEP1'] = 'AUXILIARY_BASE_BEFORE_CANDIDATE_STEP1'


@dataclass(frozen=True, slots=True)
class GodotProducts:
    base: BaseGodotReady
    candidate: ClippyEndpoint
    products: Products
    runtime_qualification: Literal['NOT_RUN'] = 'NOT_RUN'
    full_gate_claim: Literal[False] = False


# Bounded process-lifetime identities prevent copied descriptors selecting new history.
_SETUPS: dict[int, GodotSetup] = {}
_BASES: dict[int, BaseGodotReady] = {}
_PRODUCTS: dict[int, GodotProducts] = {}
_RAW: dict[int, tuple[Callable[[], None], tuple[tuple[Path, str, int], ...]]] = {}


def _raw_terminal(setup: GodotSetup, states: tuple[tuple[PreparedSetup, GeneratedState], ...],
                  history: tuple[EvidenceAnchor, ...], *, candidate_selected: bool = True) -> None:
    """Original raw closure after semantic callbacks; no endpoint recapture."""
    require(_SETUPS.get(id(setup)) is setup and id(setup) in _RAW, '/raw', 'unissued transition')
    original_source_check, external = _RAW[id(setup)]
    original_source_check()
    for path, digest, mode in external:
        require(file_hash(path) == digest and path.lstat().st_mode == mode, '/raw', 'original external bytes/mode changed')
    for path, digest, mode in setup.selected:
        if not candidate_selected and not path.is_relative_to(setup.base.inputs.root):
            continue
        require(file_hash(path) == digest and path.lstat().st_mode == mode, '/raw', 'original input bytes/mode changed')
    for evidence in (*setup.history, *history):
        require(encoded(import_generated(evidence.root)) == evidence.state, '/raw', 'original history changed')
    for prepared, state in states:
        require(encoded(import_generated(prepared.inputs.root/'rust/target')) == state.target
                and encoded(import_generated(prepared.inputs.root/'.godot')) == state.cache
                and encoded(import_generated(prepared.inputs.settings)) == state.settings,
                '/raw', 'original generated endpoint changed')



def _source(setup: GodotSetup, *, candidate_selected: bool = True) -> None:
    require(_SETUPS.get(id(setup)) is setup, '/setup', 'actual issued two-arm transition required')
    setup.closure.check(setup.products.builds)
    for proof in setup.products.sources:
        recheck_workspace_source(proof)
    for prepared in (setup.base, setup.candidate.prepared):
        immutable_inputs(prepared)
    for evidence in setup.history:
        evidence.recheck()
    for path, digest, mode in setup.selected:
        if not candidate_selected and not path.is_relative_to(setup.base.inputs.root):
            continue
        require(file_hash(path) == digest and path.lstat().st_mode == mode,
                '/normal', 'original selected binary changed')
    setup.closure.check(setup.products.builds)


def begin_godot_setup(products: CliProducts, base: PreparedSetup, candidate: PreparedSetup) -> GodotSetup:
    """Call while both original build products and caches are still untouched."""
    recheck_products(products)
    require(base.warm_pair is products.builds.base and candidate.warm_pair is products.builds.candidate,
            '/setup', 'wrong actual warm arm pairs')
    require(base.inputs.root == products.probes.base.input_root
            and candidate.inputs.root == products.probes.candidate.input_root
            and base.inputs.source_sha == products.probes.base.source_sha
            and candidate.inputs.source_sha == products.probes.candidate.source_sha,
            '/setup', 'source arm differs')
    require(base.inputs.settings != candidate.inputs.settings and base.inputs.output != candidate.inputs.output,
            '/setup', 'independent auxiliary settings/evidence required')
    for prepared in (base, candidate):
        _ = recipe(prepared, 2)
    godots = [next(tool for tool in p.inputs.tools if tool.name == 'godot') for p in (base, candidate)]
    require(godots[0] == godots[1], '/godot', 'same actual Godot tool required')
    closure = paired_setup_source_closure(products.builds)
    handoff = begin_paired_setup_handoff(products, candidate, closure)
    before = (generated(base), generated(candidate))
    require(all(document(state.cache).mapping() == document(state.settings).mapping() == {} for state in before),
            '/setup', 'both original cache/settings must be absent')
    history = (anchor(products.output), anchor(products.builds.prepared.selectors.output))
    selected = tuple((item.binary, item.sha256, item.binary.lstat().st_mode)
                     for arm in products.normal for item in arm.products)
    result = GodotSetup(products, base, handoff, closure, before, history, selected)
    external: dict[Path, str] = {}
    for prepared in (base, candidate):
        for path, digest in (*prepared.external, *((tool.path, tool.sha256) for tool in prepared.inputs.tools)):
            require(path not in external or external[path] == digest, '/raw', 'conflicting external input')
            external[path] = digest
    _SETUPS[id(result)] = result
    _RAW[id(result)] = (closure.check, tuple((path, digest, path.lstat().st_mode) for path, digest in sorted(external.items())))
    recheck_products(products)
    _source(result)
    require((generated(base), generated(candidate)) == before, '/setup', 'initial state changed during admission')
    _raw_terminal(result, ((base, before[0]), (candidate, before[1])), ())
    return result


def before_base_setup(setup: GodotSetup) -> None:
    """Check immediately before invoking existing bridge/import owners with caller FD."""
    _source(setup)
    require((generated(setup.base), generated(setup.candidate.prepared)) == setup.before,
            '/setup', 'unowned change before auxiliary producers')
    _ = recipe(setup.base, 2)
    _source(setup)
    require((generated(setup.base), generated(setup.candidate.prepared)) == setup.before,
            '/setup', 'initial state changed during prelaunch check')
    _raw_terminal(setup, ((setup.base, setup.before[0]), (setup.candidate.prepared, setup.before[1])), ())


@final
class BaseSetupFailure(RuntimeError):
    def __init__(self, primary: BaseException, bridge: BridgeResult | None, owner: ImportOwner | None, descriptor: int) -> None:
        super().__init__('auxiliary BASE preparation failed; original error and ownership retained')
        self.primary = primary
        self.bridge = bridge
        self.owner = owner
        self.held_lane_fd = descriptor


def run_base_setup_owned(setup: GodotSetup, wait: CargoWaitEvidence,
                         output: Path, descriptor: int) -> BaseGodotReady:
    """Run auxiliary original producers; no exception certifies settlement or releases FD."""
    bridge: BridgeResult | None = None
    owner: ImportOwner | None = None
    try:
        before_base_setup(setup)
        bridge = run_bridge_owned(setup.base, wait, descriptor)
        owner = start_import_pair(setup.base, PriorBridge(bridge, file_hash(bridge.receipt), wait), output, descriptor)
        result = owner.finish()
        return finish_base_setup(setup, ImportPredecessor(owner, result, file_hash(result.receipt)))
    except BaseException as primary:
        if type(primary) is ImportFailure and primary.owner is not None:
            owner = primary.owner
        raise BaseSetupFailure(primary, bridge, owner, descriptor) from primary


def finish_base_setup(setup: GodotSetup, predecessor: ImportPredecessor) -> BaseGodotReady:
    """Admit actual settled BASE bridge/import; failure/UNKNOWN objects never mint readiness."""
    _source(setup)
    require(predecessor.owner.prepared is setup.base and predecessor.result.shell_exit == 0
            and predecessor.result.status == 'PASS_STEP3_ONLY', '/base', 'wrong or failed auxiliary owner')
    history = admit_import(setup.base, predecessor)
    after = generated(setup.base)
    require(generated(setup.candidate.prepared) == setup.before[1], '/order', 'candidate setup already changed')
    result = BaseGodotReady(setup, predecessor, after, history)
    _BASES[id(result)] = result
    check_base_current(result)
    require(generated(setup.candidate.prepared) == setup.before[1], '/order', 'candidate changed during base admission')
    _raw_terminal(setup, ((setup.base, after), (setup.candidate.prepared, setup.before[1])), history)
    return result


def check_base_current(base: BaseGodotReady) -> None:
    _check_base_current(base, candidate_selected=True)


def _check_base_current(base: BaseGodotReady, *, candidate_selected: bool) -> None:
    require(_BASES.get(id(base)) is base, '/base', 'actual issued auxiliary endpoint required')
    _source(base.setup, candidate_selected=candidate_selected)
    # BASE has no later producer here; actual import current validation remains valid.
    _ = admit_import(base.setup.base, base.predecessor)
    for evidence in base.history:
        evidence.recheck()
    _source(base.setup, candidate_selected=candidate_selected)
    require(generated(base.setup.base) == base.after, '/base', 'auxiliary endpoint changed')
    _raw_terminal(base.setup, ((base.setup.base, base.after),), base.history, candidate_selected=candidate_selected)


def _files(state: str, prefix: str) -> tuple[tuple[str, str], ...]:
    result: list[tuple[str, str]] = []
    node = document(state)
    for name in node.mapping():
        row = node.field(name)
        mode = row.field('mode').value
        require(type(mode) is int and (stat.S_ISREG(mode) or stat.S_ISDIR(mode)), '/generated', 'unsupported kind')
        require(name == '.' or (not Path(name).is_absolute() and '..' not in Path(name).parts), '/generated', 'unsafe path')
        assert isinstance(mode, int)
        if stat.S_ISREG(mode):
            require(name != '.', '/generated', 'root must be directory')
            result.append((prefix+'/'+name, row.field('sha256').text()))
        else:
            row.field('sha256').equals(None)
    return tuple(sorted(result))


def _product(prepared: PreparedSetup, source: ProbeProduct, state: GeneratedState) -> ProbeProduct:
    godot = next(tool for tool in prepared.inputs.tools if tool.name == 'godot')
    inputs = dict(source.input_files)
    for name, digest in (*_files(state.target, 'rust/target'), *_files(state.cache, '.godot')):
        require(name not in inputs or inputs[name] == digest, '/inputs', 'conflicting source/runtime input')
        inputs[name] = digest
    require(BRIDGE in inputs, '/bridge', 'missing actual bridge endpoint')
    return ProbeProduct(source.source_sha, godot.path, godot.sha256, source.input_root, tuple(sorted(inputs.items())))


def finish_godot_products(base: BaseGodotReady, candidate: ClippyEndpoint) -> GodotProducts:
    check_base_current(base)
    setup = base.setup
    check_candidate_setup(setup.candidate, candidate)
    products = Products(_product(setup.base, setup.products.probes.base, base.after),
                        _product(setup.candidate.prepared, setup.products.probes.candidate, candidate.after))
    result = GodotProducts(base, candidate, products)
    _PRODUCTS[id(result)] = result
    check_godot_products(result)
    return result


def check_godot_products(result: GodotProducts) -> None:
    require(_PRODUCTS.get(id(result)) is result, '/products', 'actual issued paired Godot products required')
    base = result.base
    setup = base.setup
    check_base_current(base)
    check_candidate_setup(setup.candidate, result.candidate)
    expected = Products(_product(setup.base, setup.products.probes.base, base.after),
                        _product(setup.candidate.prepared, setup.products.probes.candidate, result.candidate.after))
    require(result.products == expected and result.runtime_qualification == 'NOT_RUN' and not result.full_gate_claim,
            '/products', 'derived descriptor or authority changed')
    for product in (result.products.base, result.products.candidate):
        check_product(product)
    # Last comparisons use original producer anchors, never adopt a new endpoint.
    _source(setup)
    for evidence in (*base.history, *result.candidate.history):
        evidence.recheck()
    require(generated(setup.base) == base.after and generated(setup.candidate.prepared) == result.candidate.after,
            '/products', 'terminal generated state changed during authentication')
    _raw_terminal(setup, ((setup.base, base.after), (setup.candidate.prepared, result.candidate.after)),
                  (*base.history, *result.candidate.history))


@dataclass(frozen=True, slots=True)
class BaseProbeWitness:
    """Original normal BASE probe plus its actual auxiliary endpoint; no candidate target check."""
    ready: BaseGodotReady
    product: ProbeProduct
    runtime_inputs: tuple[tuple[str, str], ...]


_WITNESSES: dict[int, BaseProbeWitness] = {}


def base_probe_witness(ready: BaseGodotReady) -> BaseProbeWitness:
    _check_base_current(ready, candidate_selected=False)
    result = BaseProbeWitness(ready, ready.setup.products.probes.base, _files(ready.after.target, 'rust/target'))
    _WITNESSES[id(result)] = result
    check_base_probe_witness(result)
    return result


def check_base_probe_witness(witness: BaseProbeWitness) -> None:
    require(_WITNESSES.get(id(witness)) is witness, '/base/probe', 'actual retained BASE witness required')
    ready = witness.ready
    _check_base_current(ready, candidate_selected=False)
    require(witness.product is ready.setup.products.probes.base
            and witness.runtime_inputs == _files(ready.after.target, 'rust/target'),
            '/base/probe', 'original descriptor or actual runtime endpoint replaced')
    require(file_hash(witness.product.executable) == witness.product.executable_sha256,
            '/base/probe', 'original normal executable changed')
    # Preserve both immutable sources/tool/history; only candidate generated binaries may evolve.
    _source(ready.setup, candidate_selected=False)
    for evidence in ready.history:
        evidence.recheck()
    require(generated(ready.setup.base) == ready.after, '/base/probe', 'BASE generated endpoint changed')
    _raw_terminal(ready.setup, ((ready.setup.base, ready.after),), ready.history, candidate_selected=False)
