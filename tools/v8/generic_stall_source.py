# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Read-only consistency with pinned observer bytes, not patch/build provenance."""
import hashlib
import os
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

from generic_build import OBSERVER_EXACT_PATHS, Overlay, derived_snapshot
from generic_gate import GateContractError, boundary, document, require
from generic_process import file_hash, real_path
from generic_sources import git, manifest_hash, safe_path
from mux2_gate_inputs import Json, Node


@dataclass(frozen=True, slots=True)
class DerivedSourceSpec:
    root: Path
    origin_sha: str
    overlay: Overlay


@dataclass(frozen=True, slots=True)
class DerivedSourceBinding:
    spec: DerivedSourceSpec
    origin_manifest_sha256: str
    index_sha256: str
    changed: tuple[tuple[str, str | None, str], ...]
    effective_source_sha256: str
    evidence: tuple[tuple[str, str], ...]
    source_kind: Literal['DERIVED_OBSERVER'] = 'DERIVED_OBSERVER'
    patch_application_provenance: Literal['NOT_VERIFIED'] = 'NOT_VERIFIED'
    build_admission: Literal['PENDING'] = 'PENDING'
    runtime_admission: Literal['PENDING'] = 'PENDING'
    observer_semantics: Literal['NOT_RUN'] = 'NOT_RUN'
    publication_authority: Literal[False] = False


@dataclass(frozen=True, slots=True)
class Origin:
    manifest: dict[str, Json]
    index: bytes
    blobs: dict[str, tuple[str, str, str]]


def _origin(spec: DerivedSourceSpec) -> Origin:
    require(git(spec.root, 'rev-parse', '--show-object-format').strip() == b'sha1', '/origin', 'unsupported object format')
    require(git(spec.root, 'rev-parse', 'HEAD').decode().strip() == spec.origin_sha, '/origin', 'HEAD differs')
    blobs: dict[str, tuple[str, str, str]] = {}
    manifest: dict[str, Json] = {}
    for row in git(spec.root, 'ls-tree', '-rz', spec.origin_sha).split(b'\0'):
        if not row:
            continue
        details, raw_name = row.split(b'\t', 1)
        mode, kind, oid = details.decode().split()
        name = os.fsdecode(raw_name)
        require(kind == 'blob' and mode in {'100644', '100755', '120000'} and name not in blobs,
                name, 'unsupported or duplicate origin entry')
        _ = safe_path(spec.root, name)
        digest = hashlib.sha256(git(spec.root, 'cat-file', 'blob', oid)).hexdigest()
        blobs[name] = (mode, oid, digest)
        manifest[name] = [mode, oid, digest]
    index_bytes = git(spec.root, 'ls-files', '--stage', '-z')
    index: dict[str, tuple[str, str]] = {}
    for row in index_bytes.split(b'\0'):
        if not row:
            continue
        details, raw_name = row.split(b'\t', 1)
        mode, oid, stage = details.decode().split()
        name = os.fsdecode(raw_name)
        require(stage == '0' and name not in index, name, 'conflicted or duplicate index')
        index[name] = (mode, oid)
    require(bool(blobs) and index == {name: (row[0], row[1]) for name, row in blobs.items()},
            '/index', 'index differs from origin tree')
    return Origin(manifest, index_bytes, blobs)


def _changes(spec: DerivedSourceSpec, origin: Origin) -> dict[str, tuple[str | None, str]]:
    overlay = spec.overlay
    node = document(overlay.manifest.read_text())
    node.field('schema_version').equals(1)
    node.field('product_base_sha').equals(spec.origin_sha)
    node.field('observer_patch_sha256').equals(overlay.patch_sha256)
    allowed = tuple(row.text() for row in node.field('allowed_paths').array())
    require(bool(allowed) and len(set(allowed)) == len(allowed), '/overlay', 'empty or duplicate paths')
    changed: dict[str, tuple[str | None, str]] = {}
    for row in node.field('files').array():
        name = row.field('path').text()
        _ = safe_path(spec.root, name)
        require(name not in changed and name in allowed, name, 'duplicate or unlisted path')
        require(name in OBSERVER_EXACT_PATHS or name.startswith(
            ('rust/crates/sim-test/src/', 'rust/crates/sim-bridge/src/', 'tools/v8/', 'docs/v8/')),
            name, 'outside observer structural scope')
        before_value = row.field('base0_sha256').value
        before: str | None
        if before_value is None:
            before = None
        else:
            before = row.field('base0_sha256').text()
            require(re.fullmatch('[0-9a-f]{64}', before) is not None, name, 'invalid preimage')
        after = row.field('observer_sha256').text()
        require(re.fullmatch('[0-9a-f]{64}', after) is not None and before != after, name, 'invalid postimage')
        row.field('operation').equals('add' if before is None else 'modify')
        original = origin.blobs.get(name)
        require((original is None) == (before is None), name, 'add/modify differs from origin')
        if original is not None:
            require(original[0] in {'100644', '100755'} and original[2] == before,
                    name, 'preimage or regular-file mode differs')
        changed[name] = (before, after)
    require(set(changed) == set(allowed), '/overlay', 'manifest path coverage differs')
    # Admit only simple textual add/modify patches. No application is performed.
    patch = overlay.patch.read_text()
    headers = tuple((match[1], match[2]) for match in re.finditer(
        r'^diff --git a/([^\n]+) b/([^\n]+)$', patch, re.MULTILINE))
    require(bool(headers) and all(left == right and left in changed for left, right in headers)
            and len(headers) == len(changed) and {left for left, _ in headers} == set(changed),
            '/patch', 'unsupported or mismatched patch headers')
    require(not re.search(r'^(?:GIT binary patch|Binary files |(?:old|new) mode |deleted file mode |'
                          + r'(?:rename|copy) (?:from|to) |similarity index |dissimilarity index )', patch, re.MULTILINE),
            '/patch', 'unsupported patch form')
    paths: set[str] = set()
    for raw in git(spec.root, 'apply', '--numstat', '-z', str(overlay.patch)).split(b'\0'):
        if not raw:
            continue
        fields = raw.split(b'\t', 2)
        require(len(fields) == 3 and fields[0].isdigit() and fields[1].isdigit(), '/patch', 'unsupported numstat')
        name = os.fsdecode(fields[2])
        require(name in changed and name not in paths, name, 'duplicate or foreign patch path')
        paths.add(name)
    require(paths == set(changed), '/patch', 'patch path coverage differs')
    return changed


def _evidence(spec: DerivedSourceSpec) -> tuple[tuple[str, str], ...]:
    rows = ((spec.overlay.patch, spec.overlay.patch_sha256), (spec.overlay.manifest, spec.overlay.manifest_sha256))
    require(rows[0][0] != rows[1][0], '/overlay', 'patch and manifest overlap')
    for path, expected in rows:
        real_path(path, directory=False)
        require(not path.is_relative_to(spec.root), str(path), 'overlay evidence must be external')
        require(re.fullmatch('[0-9a-f]{64}', expected) is not None and file_hash(path) == expected,
                str(path), 'overlay evidence changed')
    return tuple((str(path), digest) for path, digest in rows)


@boundary
def authenticate_derived_source(spec: DerivedSourceSpec) -> DerivedSourceBinding:
    """Verify source consistency twice, without applying a patch or granting build authority."""
    try:
        real_path(spec.root, directory=True)
        require(re.fullmatch('[0-9a-f]{40}', spec.origin_sha) is not None, '/origin', 'full SHA required')
        evidence = _evidence(spec)
        origin = _origin(spec)
        changed = _changes(spec, origin)
        source: dict[str, Json] = {'source_sha': spec.origin_sha, 'tracked_manifest': origin.manifest}
        effective = derived_snapshot(spec.root, source, changed, origin.index)
        require(_origin(spec) == origin and _evidence(spec) == evidence, '/source', 'source anchors changed')
        require(derived_snapshot(spec.root, source, changed, origin.index) == effective,
                '/source', 'effective source changed during authentication')
        require(_evidence(spec) == evidence, '/overlay', 'evidence changed during final read')
        digest = Node(effective).field('sha256').text()
        return DerivedSourceBinding(spec, manifest_hash(origin.manifest), hashlib.sha256(origin.index).hexdigest(),
                                    tuple((name, before, after) for name, (before, after) in sorted(changed.items())),
                                    digest, evidence)
    except OSError as error:
        raise GateContractError('SOURCE_UNREADABLE', str(spec.root), str(error)) from error


def recheck_derived_source(binding: DerivedSourceBinding) -> DerivedSourceBinding:
    """Compare recomputed facts to original anchors; constructed result fields confer no authority."""
    current = authenticate_derived_source(binding.spec)
    require(current == binding, '/binding', 'source binding differs from initial anchors')
    return current
