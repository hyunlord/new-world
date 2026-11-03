"""Normal executable projections from retained actual CLI builds, before setup.

These descriptors never qualify runtime or authorize publication. Generated
setup changes invalidate this handoff; a later endpoint join is still required.
"""
import hashlib
import json
import stat
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

from generic_bench import Product, validate_products
from generic_build import (
    NormalProducts,
    WorkspaceSourceProof,
    normal_workspace_products,
    recheck_normal_products,
    recheck_workspace_source,
    workspace_pair_source,
)
from generic_cli_prepare import CliBuilds, recheck_cli_builds
from generic_fixture_run import Products
from generic_fixtures import ProbeProduct
from generic_gate import document, integer, require
from generic_process import atomic_json, file_hash, real_path
from generic_sources import inventory
from mux2_gate_inputs import Json

InputFiles = tuple[tuple[str, str], ...]

SELECTORS = (('sim-test', 'worldsim_probe'), ('sim-test', 'worldsim_perf'))


@dataclass(frozen=True, slots=True)
class _Anchor:
    builds: CliBuilds
    normal: tuple[NormalProducts, NormalProducts]
    sources: tuple[WorkspaceSourceProof, WorkspaceSourceProof]
    probes: Products
    perf: tuple[Product, Product]
    output: Path
    runtime_inputs: tuple[InputFiles, InputFiles]
    intended: tuple[tuple[str, bytes], ...]
    check_build_closure: Callable[[], None]


@dataclass(frozen=True, slots=True)
class CliProducts:
    builds: CliBuilds
    normal: tuple[NormalProducts, NormalProducts]
    sources: tuple[WorkspaceSourceProof, WorkspaceSourceProof]
    probes: Products
    perf: tuple[Product, Product]
    output: Path
    runtime_inputs: tuple[InputFiles, InputFiles]
    retained_anchor: _Anchor | None = None
    status: Literal['PRODUCT_PROJECTIONS_ONLY'] = 'PRODUCT_PROJECTIONS_ONLY'
    runtime_qualification: Literal['NOT_RUN'] = 'NOT_RUN'
    publication_authority: Literal[False] = False


def _published(anchor: _Anchor) -> None:
    real_path(anchor.output, directory=True)
    require(anchor.output.lstat().st_mode == stat.S_IFDIR | 0o700,
            '/products/output', 'projection directory differs from intended mode')
    require({path.name for path in anchor.output.iterdir()} == {name for name, _ in anchor.intended},
            '/products/output', 'projection output membership changed')
    for name, raw in anchor.intended:
        path = anchor.output/name
        real_path(path, directory=False)
        require(path.lstat().st_mode == stat.S_IFREG | 0o644,
                '/products/output', 'projection file differs from intended mode')
        require(path.read_bytes() == raw, '/products/output', 'projection differs from intended bytes')


def recheck_products(result: CliProducts) -> None:
    """Recheck the retained build boundary; never refresh after setup producers."""
    anchor = result.retained_anchor
    require(anchor is not None, '/products', 'retained actual handoff required')
    assert anchor is not None
    require(result.builds is anchor.builds and result.normal is anchor.normal
            and result.sources is anchor.sources and result.probes is anchor.probes
            and result.perf is anchor.perf and result.output == anchor.output
            and result.runtime_inputs is anchor.runtime_inputs
            and result.status == 'PRODUCT_PROJECTIONS_ONLY'
            and result.runtime_qualification == 'NOT_RUN' and result.publication_authority is False,
            '/products', 'retained projection identity changed')
    recheck_cli_builds(result.builds)
    for pair, normal, source in zip((result.builds.base, result.builds.candidate), result.normal, result.sources, strict=True):
        require(normal.pair is pair and normal.profile == 'release' and normal.names == SELECTORS,
                '/products/arm', 'wrong actual arm/profile/normal selection')
        recheck_normal_products(normal)
        recheck_workspace_source(source)
    validate_products(*result.perf, result.builds.prepared.selectors.purpose)
    # Close mutations from the public validation callbacks against upstream live
    # producer anchors, then check our independently intended publication bytes.
    recheck_cli_builds(result.builds)
    anchor.check_build_closure()
    _published(anchor)


def construct_products(builds: CliBuilds, output: Path) -> CliProducts:
    """Construct both release probe/perf views from actual normal compiler records.

    Perf's existing verifier accepts the sidecar projection fields. Its original
    producer remains the referenced release BuildResult, never a new cargo build.
    Exceptions preserve the partial output; they are not a settled native result.
    """
    recheck_cli_builds(builds)
    original = builds.retained_anchor
    require(original is not None, '/products/builds', 'original CLI closure unavailable')
    assert original is not None
    check_build_closure = original.check_closure
    require(builds.status == 'BUILD_ANTECEDENTS_ONLY' and builds.candidate is not None,
            '/products/builds', 'both actual successful arm pairs required')
    assert builds.candidate is not None
    selected = builds.prepared.selectors
    roots = (selected.base.root, selected.candidate.root, selected.harness_root,
             selected.base.cargo_home, selected.candidate.cargo_home, selected.output)
    real_path(output.parent, directory=True)
    require(output.is_absolute() and output == output.parent.resolve()/output.name
            and not output.exists() and not output.is_symlink()
            and all(not output.is_relative_to(root) and not root.is_relative_to(output) for root in roots),
            '/products/output', 'fresh independent handoff output required')
    pairs = (builds.base, builds.candidate)
    normal = (normal_workspace_products(pairs[0], 'release', SELECTORS),
              normal_workspace_products(pairs[1], 'release', SELECTORS))
    sources = (workspace_pair_source(pairs[0]), workspace_pair_source(pairs[1]))
    probes: list[ProbeProduct] = []
    perf: list[Product] = []
    documents: list[tuple[str, dict[str, Json]]] = []
    runtime_inputs: list[InputFiles] = []
    for arm, pair, selection, source, plan in zip(('base', 'candidate'), pairs, normal, sources, builds.prepared.arms, strict=True):
        probe, executable = selection.products
        require(selection.pair is pair and probe.source_sha == executable.source_sha == plan.release.source_sha
                and probe.binary.parent == executable.binary.parent == plan.release.root/'rust/target/release',
                '/products/arm', 'normal selection differs from actual arm')
        require(pair.release is not None, '/products/release', 'release result unavailable')
        assert pair.release is not None
        row = document(pair.release.receipt.read_bytes().decode())
        native_exit = integer(row.field('exit_code'))
        require(native_exit == 0, '/products/release', 'actual producer failed')
        inputs = dict(source.effective_inputs)
        target = inventory(plan.release.root, ['rust/target'])
        require(not inputs.keys() & target.keys(), '/products/inputs', 'overlapping source/target scopes')
        runtime_inputs.append(tuple(sorted(target.items())))
        files = tuple(sorted(inputs.items()))
        probes.append(ProbeProduct(probe.source_sha, probe.binary, probe.sha256, plan.release.root, files))
        payload: dict[str, Json] = {
            'schema_version': 1, 'kind': 'normal_product_adapter_projection',
            'arm': arm, 'source_sha': executable.source_sha, 'binary_sha256': executable.sha256,
            'profile': 'release', 'exit_code': native_exit,
            'actual_build_receipt': str(pair.release.receipt), 'actual_build_receipt_sha256': file_hash(pair.release.receipt),
            'actual_pair_receipt': str(pair.receipt), 'actual_pair_receipt_sha256': file_hash(pair.receipt),
            'actual_process_receipt': row.field('process_receipt').text(),
            'actual_process_receipt_sha256': file_hash(Path(row.field('process_receipt').text())),
            'normal_selection': {'package': executable.package, 'package_id': executable.package_id,
                'name': executable.name, 'source': str(executable.source), 'manifest': str(executable.manifest),
                'binary': str(executable.binary), 'metadata_sha256': executable.metadata_sha256,
                'compiler_stream_sha256': executable.compiler_stream_sha256,
                'compiler_line': executable.compiler_line, 'compiler_line_sha256': executable.compiler_line_sha256},
            'input_files': dict(files), 'runtime_target_files': {name: digest for name, digest in target.items()}, 'runtime_qualification': 'NOT_RUN', 'publication_authority': False,
        }
        name = f'{arm}-perf.json'
        documents.append((name, payload))
        # The digest is computed from intended bytes, not from a potentially
        # changed publication readback. No receipt is accepted on hash alone.
        raw = (json.dumps(payload, indent=2, sort_keys=True, allow_nan=False)+'\n').encode()
        perf.append(Product(plan.release.root, executable.binary, executable.source_sha, executable.sha256,
                            output/name, hashlib.sha256(raw).hexdigest(), files))
    intended = tuple((name, (json.dumps(payload, indent=2, sort_keys=True, allow_nan=False)+'\n').encode())
                     for name, payload in documents)
    view = Products(probes[0], probes[1])
    performance = (perf[0], perf[1])
    runtime = (runtime_inputs[0], runtime_inputs[1])
    anchor = _Anchor(builds, normal, sources, view, performance, output, runtime, intended, check_build_closure)
    recheck_cli_builds(builds)
    output.mkdir(mode=0o700)
    for name, payload in documents:
        atomic_json(output/name, payload)
    result = CliProducts(builds, normal, sources, view, performance, output, runtime, anchor)
    recheck_products(result)
    return result
