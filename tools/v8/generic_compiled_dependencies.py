"""Schema3 membership bindings; compiler/build provenance remains caller authority."""
from dataclasses import dataclass
from pathlib import Path

from generic_gate import boundary, integer, require, text
from generic_observations import keys
from generic_process import check_digest, file_hash, real_path
from mux2_gate_inputs import Node


@dataclass(frozen=True, slots=True)
class CompiledDependency:
    id: str
    path: Path
    sha256: str


@boundary
def read_compiled_dependencies(plan: Node, policy: Node) -> tuple[CompiledDependency, ...]:
    """Check a distinct input-backed category, never authorize it as a launch target."""
    require(integer(policy.field('schema_version')) in (3, 4), '/compiled', 'schema3/4 required')
    inputs = plan.field('inputs').mapping()
    module = Path(__file__).resolve()
    require(inputs.get(str(module)) == file_hash(module), '/compiled', 'reader source is not bound')
    artifacts = plan.field('artifacts').array()
    artifact_ids = {text(row.field('id')) for row in artifacts}
    artifact_paths = {text(row.field('path')) for row in artifacts}
    provenance = policy.field('compiled_dependency_provenance')
    keys(provenance, {'path', 'sha256'})
    evidence = Path(text(provenance.field('path')))
    real_path(evidence, directory=False)
    evidence_hash = text(provenance.field('sha256')); check_digest(evidence_hash, '/compiled/provenance')
    require(str(evidence) not in artifact_paths and inputs.get(str(evidence)) == evidence_hash
            and file_hash(evidence) == evidence_hash, '/compiled', 'provenance is not an immutable input')
    rows: list[CompiledDependency] = []
    paths: set[Path] = set()
    for row in policy.field('compiled_dependencies').array():
        keys(row, {'id', 'path', 'sha256'})
        ident, path, checksum = text(row.field('id')), Path(text(row.field('path'))), text(row.field('sha256'))
        real_path(path, directory=False); check_digest(checksum, '/compiled/sha256')
        require(bool(ident) and ident not in artifact_ids and str(path) not in artifact_paths
                and path not in paths and path != evidence, '/compiled', 'duplicate or overlapping compiled dependency')
        require(inputs.get(str(path)) == checksum and file_hash(path) == checksum, '/compiled', 'compiled member is not an unchanged input')
        rows.append(CompiledDependency(ident, path, checksum)); paths.add(path)
    identifiers = [row.id for row in rows]
    require(identifiers == sorted(set(identifiers)), '/compiled', 'compiled IDs must be sorted unique')
    return tuple(rows)
