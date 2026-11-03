"""Explicit original-product plus executable-H projection; no build/runtime grant."""
import hashlib
from dataclasses import dataclass
from pathlib import Path

from generic_build import (
    derived_snapshot,
    recheck_workspace_pair,
    workspace_pair_source,
)
from generic_gate import document, require
from generic_identity_overlay import selected_roles
from generic_overlay_contract import harness_tool
from generic_process import file_hash
from generic_setup_bridge import immutable_inputs
from generic_setup_prepare import PreparedSetup, warm_projection
from generic_sources import freeze_source, git, manifest_hash
from generic_whole_gate_source import check_materialization
from mux2_gate_inputs import Json, Node


def projected_manifest(harness: Node, product: Node) -> dict[str, Json]:
    """Full P census plus every selected executable H postimage; never O or data."""
    original = product.field('tracked_manifest').mapping()
    expected: dict[str, Json] = {}
    for name, raw in original.items():
        row = Node(raw).array()
        require(len(row) == 3, name, 'invalid original tracked row')
        expected[name] = [row[0].text(), row[2].text()]
    roles = selected_roles(Path(harness.field('root').text()), harness)
    for raw in roles:
        row = Node(raw)
        name = row.field('path').text()
        if row.field('role').text() != 'H' or not name.endswith(('.py', '.sh')):
            continue
        require(harness_tool(name), name, 'H executable outside tooling namespace')
        expected[name] = [row.field('mode').text(), row.field('sha256').text()]
    require(bool(expected), '/projection', 'empty original-product census')
    return expected


@dataclass(frozen=True, slots=True)
class OriginalHarnessContext:
    """Explicit source-owned session reference, not a substitute for a live build pair."""
    session_path: Path
    session_sha256: str
    arm: str

    def admitted_harness(self, session: Node, original: Node) -> Node:
        """Recheck context and original P identity before planning or validating bytes."""
        require(self.arm in {'base', 'candidate'}, '/arm', 'unknown original product arm')
        require(file_hash(self.session_path) == self.session_sha256,
                '/session', 'original projection session changed')
        document(self.session_path.read_text()).equals(session.value)
        session.field('schema').equals('whole_gate_session.v1')
        require('materialization' in session.mapping(), '/session', 'real source materialization required')
        check_materialization(session)
        arm = session.field('arms').field(self.arm)
        original.field('source_sha').equals(arm.field('source_sha').value)
        original.field('tracked_manifest').equals(arm.field('source_before').field('tracked_manifest').value)
        original.field('tracked_count').equals(arm.field('source_before').field('tracked_count').value)
        original.field('tracked_manifest_sha256').equals(arm.field('source_before').field('tracked_manifest_sha256').value)
        original.field('source_clean').equals(True)
        root = Path(original.field('root').text())
        protected = [Path(session.field('harness').field('root').text())]
        protected.extend(Path(Node(raw).field('root').text()) for raw in session.field('arms').mapping().values())
        require(root.is_absolute() and root == root.resolve(strict=True)
                and all(root != other and root not in other.parents and other not in root.parents for other in protected),
                '/root', 'independent original-product projection root required')
        receipt = document(Path(session.field('materialization').field('path').text()).read_text())
        harness = receipt.field('harness')
        require(freeze_source(Path(harness.field('root').text()), harness.field('source_sha').text()) == harness.mapping(),
                '/harness', 'admitted harness snapshot changed')
        return harness

    def expected(self, session: Node, original: Node) -> dict[str, Json]:
        harness = self.admitted_harness(session, original)
        arm = session.field('arms').field(self.arm)
        root = Path(original.field('root').text())
        expected = projected_manifest(harness, original)
        index = git(root, 'ls-files', '--stage', '-z')
        require(hashlib.sha256(index).hexdigest() == arm.field('index_sha256').text(),
                '/index', 'original product index changed')
        before = original.field('tracked_manifest').mapping()
        changed: dict[str, tuple[str | None, str]] = {}
        for name, raw in expected.items():
            digest = Node(raw).array()[1].text()
            prior = None if name not in before else Node(before[name]).array()[2].text()
            if prior != digest:
                changed[name] = (prior, digest)
        require(derived_snapshot(root, original.mapping(), changed, index)
                == {'manifest': expected, 'sha256': manifest_hash(expected)},
                '/projection', 'actual source differs from complete P plus executable H')
        return expected

    def validate_build(self, session: Node, build: Node) -> dict[str, Json]:
        """Source-only check; caller must still authenticate the actual Cargo receipt."""
        original = build.field('source_before')
        build.field('root').equals(original.field('root').value)
        build.field('source_sha').equals(original.field('source_sha').value)
        expected = self.expected(session, original)
        effective: dict[str, Json] = {'manifest': expected, 'sha256': manifest_hash(expected)}
        build.field('effective_source_before').equals(effective)
        build.field('effective_source_after').equals(effective)
        build.field('source_kind').equals('DERIVED_OBSERVER')
        return expected

    def validate_prepared(self, session: Node, prepared: PreparedSetup) -> None:
        """Require actual retained debug/release execution before admitting warm setup."""
        pair = prepared.warm_pair
        require(pair is not None, '/pair', 'actual retained workspace build pair required')
        assert pair is not None
        specs = recheck_workspace_pair(pair)
        proof = workspace_pair_source(pair)
        require(prepared.derived_source == proof and prepared.preparation_kind == 'WARM_BUILD_ANTECEDENTS',
                '/projection', 'prepared source is not the original returned workspace pair')
        original = document(proof.baseline_json)
        original.field('root').equals(str(prepared.inputs.root))
        original.field('source_sha').equals(prepared.inputs.source_sha)
        expected = self.expected(session, original)
        document(proof.effective_json).equals({'manifest': expected, 'sha256': manifest_hash(expected)})
        require(proof.source_kind == 'DERIVED_OBSERVER' and all(spec.overlay is not None for spec in specs),
                '/projection', 'explicit executable-H overlay required')
        require(warm_projection(prepared.inputs, pair) == prepared.environment_delta,
                '/projection', 'actual warm build/setup environment differs')
        immutable_inputs(prepared)
