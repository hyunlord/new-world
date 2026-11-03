"""Historical build identity joined to actual candidate setup; no runtime admission."""
import json
import stat
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

from generic_build import recheck_workspace_source
from generic_cli_prepare import (
    CliSetupSourceClosure,
    PairedSetupSourceClosure,
    setup_source_closure,
)
from generic_cli_products import CliProducts, recheck_products
from generic_gate import document, require
from generic_process import file_hash, real_path
from generic_setup_bridge import immutable_inputs
from generic_setup_clippy import ClippyEndpoint, check_clippy_endpoint
from generic_setup_prepare import BRIDGE, PreparedSetup, recipe
from generic_sources import freeze_binding, freeze_source
from generic_workspace_endpoints import (
    EvidenceAnchor,
    GeneratedState,
    ImportPredecessor,
    WorkspaceEndpoint,
    anchor,
    generated,
)

InputFiles = tuple[tuple[str, str], ...]


@dataclass(frozen=True, slots=True)
class _HandoffAnchor:
    products: CliProducts
    prepared: PreparedSetup
    before: GeneratedState
    evidence: tuple[EvidenceAnchor, ...]
    selected: tuple[tuple[Path, str, int], ...]
    source_modes: tuple[tuple[Path, int], ...]
    immutable: CliSetupSourceClosure | PairedSetupSourceClosure


@dataclass(frozen=True, slots=True)
class SetupProductHandoff:
    products: CliProducts
    prepared: PreparedSetup
    retained_anchor: _HandoffAnchor | None = None


@dataclass(frozen=True, slots=True)
class _RuntimeAnchor:
    handoff: SetupProductHandoff
    endpoint: ClippyEndpoint
    runtime_inputs: tuple[InputFiles, InputFiles]


@dataclass(frozen=True, slots=True)
class RuntimeProducts:
    handoff: SetupProductHandoff
    endpoint: ClippyEndpoint
    runtime_inputs: tuple[InputFiles, InputFiles]
    retained_anchor: _RuntimeAnchor | None = None
    runtime_qualification: Literal['NOT_RUN'] = 'NOT_RUN'
    publication_authority: Literal[False] = False


def _history(handoff: SetupProductHandoff) -> _HandoffAnchor:
    bound = handoff.retained_anchor
    require(bound is not None, '/handoff', 'actual begin handoff required')
    assert bound is not None
    require(handoff.products is bound.products and handoff.prepared is bound.prepared,
            '/handoff', 'retained object identity differs')
    products = bound.products
    require(bound.prepared.warm_pair is products.builds.candidate
            and _target_files(bound.before.target) == products.runtime_inputs[1]
            and document(bound.before.cache).mapping() == document(bound.before.settings).mapping() == {},
            '/handoff', 'original admitted setup boundary differs')
    for source in products.sources:
        recheck_workspace_source(source)
    selected = products.builds.prepared.selectors
    require(json.dumps(freeze_source(selected.harness_root, selected.harness_sha), sort_keys=True)
            == products.builds.prepared.harness_json, '/harness', 'harness source changed')
    require(json.dumps(freeze_binding(selected.harness_root, selected.harness_sha, selected.published_sha,
            selected.purpose, candidate_sha=selected.candidate.source_sha), sort_keys=True)
            == products.builds.prepared.binding_json, '/harness', 'source relation changed')
    immutable_inputs(bound.prepared)
    for path, mode in bound.source_modes:
        require(path.lstat().st_mode == mode, '/source', 'source kind/mode changed')
    for evidence in bound.evidence:
        evidence.recheck()
    bound.immutable.check(bound.products.builds)
    return bound


def _selected(bound: _HandoffAnchor) -> None:
    for path, digest, mode in bound.selected:
        real_path(path, directory=False)
        require(path.lstat().st_mode == mode and file_hash(path) == digest,
                '/normal', 'selected original binary changed')


def begin_setup_handoff(products: CliProducts, prepared: PreparedSetup) -> SetupProductHandoff:
    return _begin_setup_handoff(products, prepared, setup_source_closure(products.builds))


def begin_paired_setup_handoff(products: CliProducts, prepared: PreparedSetup,
                               immutable: PairedSetupSourceClosure) -> SetupProductHandoff:
    immutable.check(products.builds)
    return _begin_setup_handoff(products, prepared, immutable)


def _begin_setup_handoff(products: CliProducts, prepared: PreparedSetup,
                         immutable: CliSetupSourceClosure | PairedSetupSourceClosure) -> SetupProductHandoff:
    recheck_products(products)
    require(prepared.warm_pair is products.builds.candidate and prepared.warm_pair is not None
            and prepared.preparation_kind == 'WARM_BUILD_ANTECEDENTS', '/handoff', 'exact warm candidate pair required')
    _ = recipe(prepared, 2)
    require(prepared.inputs.root == products.probes.candidate.input_root
            and prepared.inputs.source_sha == products.probes.candidate.source_sha,
            '/handoff', 'candidate source differs')
    before = generated(prepared)
    require(document(before.cache).mapping() == {} and document(before.settings).mapping() == {},
            '/handoff', 'pre-setup cache/settings must be absent')
    selected = tuple((item.binary, item.sha256, item.binary.lstat().st_mode)
                     for arm in products.normal for item in arm.products)
    modes = tuple((product.input_root/name, (product.input_root/name).lstat().st_mode)
                  for product in (products.probes.base, products.probes.candidate) for name, _ in product.input_files)
    bound = _HandoffAnchor(products, prepared, before,
                           (anchor(products.output), anchor(products.builds.prepared.selectors.output)), selected, modes, immutable)
    result = SetupProductHandoff(products, prepared, bound)
    recheck_products(products)
    _ = _history(result)
    _selected(bound)
    require(generated(prepared) == before, '/handoff', 'generated state changed while admitting handoff')
    bound.immutable.check(bound.products.builds)
    return result


def _target_files(state: str) -> InputFiles:
    node = document(state)
    files: list[tuple[str, str]] = []
    for name in node.mapping():
        member = node.field(name)
        mode = member.field('mode').value
        require(type(mode) is int and (stat.S_ISREG(mode) or stat.S_ISDIR(mode)), '/target', 'unsupported generated kind')
        assert isinstance(mode, int)
        require(name == '.' or (not Path(name).is_absolute() and '..' not in Path(name).parts), '/target', 'invalid relative member')
        if stat.S_ISREG(mode):
            require(name != '.', '/target', 'target root is not a directory')
            files.append(('rust/target/'+name, member.field('sha256').text()))
        else:
            member.field('sha256').equals(None)
    return tuple(sorted(files))


def _chain(bound: _HandoffAnchor, endpoint: ClippyEndpoint) -> None:
    prepared = bound.prepared
    require(endpoint.request.prepared is prepared, '/chain', 'foreign setup endpoint')
    check_clippy_endpoint(endpoint)
    release = endpoint.predecessor
    chain = release.request.chain
    require(chain is not None and isinstance(chain.predecessor, WorkspaceEndpoint), '/chain', 'missing debug predecessor')
    assert chain is not None and isinstance(chain.predecessor, WorkspaceEndpoint)
    debug = chain.predecessor
    first = debug.request.chain
    require(first is not None and isinstance(first.predecessor, ImportPredecessor), '/chain', 'missing actual import owner')
    assert first is not None and isinstance(first.predecessor, ImportPredecessor)
    imported = first.predecessor
    owner = imported.owner
    require(release.step == 5 and debug.step == 4 and release.arm == debug.arm == 'candidate'
            and release.prepared is prepared and debug.prepared is prepared
            and chain.prepared is prepared and first.prepared is prepared and owner.prepared is prepared,
            '/chain', 'wrong ordered setup identity')
    require(release.before == debug.after and endpoint.before == release.after, '/chain', 'disconnected generated endpoints')
    control = owner.controller
    # snapshot() revalidates live target inputs and can cancel an already closed
    # owner. Historical joins use its retained actual wait and sealed terminal.
    require(control.started and control.closed and control.active is None and not control.buffer
            and not control.error and tuple(control.completed) == ('import.first', 'import.second')
            and control.shell.returncode == imported.result.shell_exit, '/chain', 'import owner not settled')
    terminal = document((control.plan.evidence/'suite-terminal.json').read_text())
    terminal.field('settlement').equals('SETTLED')
    terminal.field('plan_sha256').equals(control.plan.digest)
    terminal.field('shell_pid').equals(control.shell.pid)
    terminal.field('shell_exit').equals(control.shell.returncode)
    terminal.field('completed').equals(list(control.completed))
    require(file_hash(imported.result.receipt) == imported.receipt_sha256, '/chain', 'import receipt changed')
    row = document(imported.result.receipt.read_text())
    row.field('prior_bridge_sha256').equals(owner.prior.receipt_sha256)
    row.field('plan_sha256').equals(control.plan.digest)
    row.field('shell_pid').equals(control.shell.pid)
    row.field('first_native_exit').equals(imported.result.first_exit)
    row.field('second_native_exit').equals(imported.result.second_exit)
    require(imported.result.shell_exit == 0 and imported.result.status == 'PASS_STEP3_ONLY', '/chain', 'failed import chain')
    checkpoint = document((control.plan.rows[-1].evidence/'import-checkpoint.json').read_text())
    require(control.import_checkpoint == checkpoint.mapping(), '/chain', 'retained import checkpoint differs')
    checkpoint.field('target').equals(dict(_target_files(debug.before.target)))
    checkpoint.field('cache').equals(document(debug.before.cache).mapping())
    checkpoint.field('settings').equals(document(debug.before.settings).mapping())
    bridge = owner.prior.result
    require(bridge.receipt == prepared.inputs.output/'receipt.json'
            and file_hash(bridge.receipt) == owner.prior.receipt_sha256 and bridge.native_exit == 0,
            '/chain', 'missing actual bridge endpoint')
    bridge_row = document(bridge.receipt.read_text())
    bridge_row.field('target_inventory').equals(checkpoint.field('target').value)
    bridge_row.field('bridge_sha256').equals(bridge.bridge_sha256)
    bridge_row.field('source_sha').equals(prepared.inputs.source_sha)
    require(dict(_target_files(endpoint.after.target)).get(BRIDGE) == bridge.bridge_sha256,
            '/bridge', 'successor changed original bridge binary')
    # Actual warm step2 reruns recipe and full pair immediately before execute;
    # only the same PreparedSetup identity is accepted through this retained chain.
    for history in endpoint.history:
        history.recheck()


def finish_setup_handoff(handoff: SetupProductHandoff, endpoint: ClippyEndpoint) -> RuntimeProducts:
    bound = _history(handoff)
    _chain(bound, endpoint)
    _selected(bound)
    require(generated(bound.prepared) == endpoint.after, '/runtime', 'current state differs from actual endpoint')
    runtime = (bound.products.runtime_inputs[0], _target_files(endpoint.after.target))
    result = RuntimeProducts(handoff, endpoint, runtime, _RuntimeAnchor(handoff, endpoint, runtime))
    check_runtime_current(result)
    return result


def check_runtime_history(result: RuntimeProducts) -> None:
    retained = result.retained_anchor
    require(retained is not None and result.runtime_qualification == 'NOT_RUN' and result.publication_authority is False,
            '/runtime', 'retained setup handoff required')
    assert retained is not None
    require(result.handoff is retained.handoff and result.endpoint is retained.endpoint
            and result.runtime_inputs is retained.runtime_inputs, '/runtime', 'retained endpoint identity differs')
    bound = _history(result.handoff)
    _chain(bound, result.endpoint)
    require(result.runtime_inputs == (bound.products.runtime_inputs[0], _target_files(result.endpoint.after.target)),
            '/runtime', 'runtime map differs from original/producer endpoints')
    bound.immutable.check(bound.products.builds)


def check_runtime_current(result: RuntimeProducts) -> None:
    check_runtime_history(result)
    bound = _history(result.handoff)
    # Base has no setup producer; its original complete current pair remains required.
    from generic_build import recheck_workspace_pair
    _ = recheck_workspace_pair(bound.products.builds.base)
    _ = _history(result.handoff)
    _selected(bound)
    require(generated(bound.prepared) == result.endpoint.after, '/runtime', 'unowned generated-state change')
    for evidence in bound.evidence:
        evidence.recheck()
    bound.immutable.check(bound.products.builds)


def check_candidate_setup(handoff: SetupProductHandoff, endpoint: ClippyEndpoint) -> None:
    """Authenticate actual candidate chain; paired caller owns separate BASE endpoint."""
    bound = _history(handoff)
    _chain(bound, endpoint)
    _selected(bound)
    require(generated(bound.prepared) == endpoint.after, '/runtime', 'candidate endpoint changed')
    for evidence in bound.evidence:
        evidence.recheck()
    bound.immutable.check(bound.products.builds)
