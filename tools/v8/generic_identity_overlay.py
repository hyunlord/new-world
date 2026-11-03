"""Explicit identity census over admitted H/P, without observer qualification."""
import ast
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Final

from generic_gate import require
from generic_overlay_contract import (
    HARNESS_EXACT_PATHS,
    OBSERVER_EXACT_PATHS,
    IdentityInput,
    harness_tool,
)
from generic_process import file_hash
from generic_sources import safe_path
from mux2_gate_inputs import Json, Node

EXPLICIT: Final = ('tools/play.sh', 'tools/v8/gate.sh', 'tools/v8/read_summary_normalise.py',
                  'tools/v8/gate_f10_no_kill.json', 'tools/v8/generic_whole_gate_ui_companions.py',
                  'tools/v8/generic_whole_gate_stall_capture.py', 'tools/v8/generic_whole_gate_acceptance.py')
MATRIX: Final = 'tools/v8/whole_gate_observer_matrix.json'


def compatible(name: str) -> bool:
    return name in HARNESS_EXACT_PATHS or name in OBSERVER_EXACT_PATHS or name.startswith(
        ('rust/crates/sim-test/src/', 'rust/crates/sim-bridge/src/', 'tools/v8/', 'docs/v8/'))


def identity_namespace(harness: Node, product: Node) -> list[Json]:
    """Union of both admitted trees, never the caller's declared overlay subset."""
    h = harness.field('tracked_manifest').mapping()
    p = product.field('tracked_manifest').mapping()
    names = sorted(name for name in h.keys() | p.keys() if compatible(name))
    require(bool(names), '/identity', 'empty compatible namespace')
    rows: list[Json] = []
    for name in names:
        require(name in h and name in p, name, 'compatible path missing from H or P')
        before, after = Node(h[name]).array(), Node(p[name]).array()
        require(len(before) == len(after) == 3, name, 'invalid admitted tracked row')
        mode, digest = before[0].text(), before[2].text()
        require(mode in {'100644', '100755'}, name, 'identity source must be regular')
        require(after[0].text() == mode and after[2].text() == digest,
                name, 'compatible H/P mode or bytes differ')
        rows.append({'path': name, 'mode': mode, 'sha256': digest})
    return rows


def selected_roles(root: Path, harness: Node) -> list[Json]:
    """Derive complete H import closure and O set from the admitted H matrix."""
    from generic_gate import document
    manifest = harness.field('tracked_manifest').mapping()
    require(str(root) == harness.field('root').text(), '/identity', 'wrong admitted H root')

    def read(name: str) -> str:
        require(name in manifest, name, 'missing admitted H dependency')
        fields = Node(manifest[name]).array()
        require(fields[0].text() in {'100644', '100755'}, name, 'nonregular selected source')
        path = safe_path(root, name)
        require(file_hash(path) == fields[2].text(), name, 'admitted H dependency changed')
        return path.read_text()

    matrix = document(read(MATRIX))
    matrix.field('schema').equals('whole_gate_observer_matrix.v1')
    observers = matrix.field('observer_postimages').mapping()
    require(bool(observers), '/identity', 'missing reviewed observer selection')
    for name, raw in observers.items():
        require(compatible(name), name, 'observer outside compatible namespace')
        _ = read(name)
        row = Node(manifest[name]).array()
        Node(raw).equals([row[0].text(), row[2].text()])
    pending: list[str] = list(EXPLICIT)
    selected: set[str] = set()
    while pending:
        name = pending.pop()
        if name in selected:
            continue
        require(harness_tool(name),
                name, 'H role is not named executable tooling')
        raw = read(name)
        selected.add(name)
        dependencies: set[str] = set()
        if name.endswith('.py'):
            for node in ast.walk(ast.parse(raw)):
                modules = [alias.name for alias in node.names] if isinstance(node, ast.Import) else (
                    [node.module] if isinstance(node, ast.ImportFrom) and node.module else [])
                for module in modules:
                    dependency = 'tools/v8/' + module.replace('.', '/') + '.py'
                    if dependency in manifest:
                        dependencies.add(dependency)
        for match in re.finditer(r'tools/v8/[A-Za-z0-9_./-]+\.(?:py|sh)\b', raw):
            if match.group() in manifest:
                dependencies.add(match.group())
        pending.extend(sorted(dependencies - selected))
    require(not selected & observers.keys(), '/identity', 'ambiguous H/O roles')
    result: list[Json] = []
    for name in sorted(selected | observers.keys()):
        row = Node(manifest[name]).array()
        observer = name in observers
        result.append({'path': name, 'role': 'O' if observer else 'H',
                       'sha256': row[2].text(), 'mode': row[0].text(),
                       'qualification': 'NOT_RUN' if observer else 'HARNESS'})
    return result


def validate_identity(root: Path, harness: Node, product: Node, manifest: Node, selected: Node) -> None:
    """Caller must supply admitted snapshots; this pure validator does not admit a build."""
    manifest.field('schema_version').equals(2)
    manifest.field('kind').equals('whole_gate_identity.v1')
    manifest.field('harness_root').equals(str(root))
    manifest.field('harness_sha').equals(harness.field('source_sha').value)
    manifest.field('product_base_sha').equals(product.field('source_sha').value)
    manifest.field('allowed_paths').equals([])
    manifest.field('files').equals([])
    manifest.field('identity_namespace').equals(identity_namespace(harness, product))
    selected.field('harness_sha').equals(harness.field('source_sha').value)
    selected.field('product_sha').equals(product.field('source_sha').value)
    selected.field('qualification').equals('PRODUCT_UNQUALIFIED')
    selected.field('files').equals(selected_roles(root, harness))


@dataclass(frozen=True, slots=True)
class IdentityContext:
    """Whole-gate admitted snapshots and external anchors, revalidated before each use."""
    harness_json: str
    product_json: str
    binding_json: str
    role_path: Path
    role_sha256: str
    role_mode: int
    selected_path: Path
    selected_sha256: str
    selected_mode: int

    def validate(self, spec: IdentityInput) -> None:
        from generic_gate import document
        from generic_sources import freeze_binding, freeze_source
        h, p, binding = document(self.harness_json), document(self.product_json), document(self.binding_json)
        root = Path(h.field('root').text())
        require(str(spec.root) == p.field('root').text() and spec.source_sha == p.field('source_sha').text(),
                '/identity', 'context is for another admitted product')
        require(freeze_source(root, h.field('source_sha').text()) == h.mapping(), '/identity', 'admitted H changed')
        require(freeze_source(spec.root, spec.source_sha) == p.mapping(), '/identity', 'admitted P changed')
        require(freeze_binding(root, h.field('source_sha').text(), binding.field('base_product_sha').text(),
                               binding.field('purpose').text(), candidate_sha=binding.field('candidate_product_sha').text())
                == binding.mapping(), '/identity', 'original H/P binding changed')
        require(spec.source_sha in {binding.field('base_product_sha').text(), binding.field('candidate_product_sha').text()},
                '/identity', 'product outside admitted arms')
        for path, digest, mode in ((self.role_path, self.role_sha256, self.role_mode),
                                   (self.selected_path, self.selected_sha256, self.selected_mode)):
            require(file_hash(path) == digest and path.lstat().st_mode == mode,
                    str(path), 'identity external input changed')
        require(spec.overlay is not None, '/identity', 'explicit identity overlay required')
        assert spec.overlay is not None
        require(file_hash(spec.overlay.manifest) == spec.overlay.manifest_sha256
                and file_hash(spec.overlay.patch) == spec.overlay.patch_sha256
                and spec.overlay.patch.read_bytes() == b'', '/identity', 'identity overlay artifact changed')
        manifest = document(spec.overlay.manifest.read_text())
        manifest.field('selected_source').equals({'path': str(self.selected_path),
                                                'sha256': self.selected_sha256, 'mode': self.selected_mode})
        roles = document(self.role_path.read_text())
        roles.field('schema').equals('whole_gate_layers.v1')
        roles.field('product_sha').equals(spec.source_sha)
        roles.field('harness_sha').equals(h.field('source_sha').value)
        roles.field('files').equals([])
        validate_identity(root, h, p, manifest, document(self.selected_path.read_text()))
