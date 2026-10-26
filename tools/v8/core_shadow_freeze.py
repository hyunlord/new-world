#!/usr/bin/env python3
"""Bind externally built old-world shadow/control binaries to a source snapshot.

Usage: prepare --repo ROOT --out SNAPSHOT; seal --help; check --help.
No build or executable is launched. Receipts attest builder provenance; hashes
prove byte equality, not reproducible compilation or the builder's honesty.
An independently retained manifest SHA is required at every gate invocation.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import sys
from typing import Final, TypeAlias

JSON: TypeAlias = str | int | bool | None | list['JSON'] | dict[str, 'JSON']
Document: TypeAlias = dict[str, JSON]
CURRENT_DIRS: Final = ('rust/crates/sim-core/src', 'rust/crates/sim-core/data', 'rust/crates/sim-engine/src',
                       'rust/crates/sim-systems/src/runtime')
REQUIRED_FILES: Final = ('rust/crates/sim-core/data/colony/mind.ron', 'rust/Cargo.lock',
    'rust/Cargo.toml', 'rust/rust-toolchain.toml', 'rust/crates/sim-core/Cargo.toml', 'rust/crates/sim-engine/Cargo.toml',
    'rust/crates/sim-systems/Cargo.toml', 'rust/crates/sim-test/Cargo.toml',
    'rust/crates/sim-core/src/colony/mind/selection.rs',
    'rust/crates/sim-systems/src/commands.rs',
    'rust/crates/sim-systems/src/mind_observer.rs',
    'rust/crates/sim-systems/src/subprofile.rs',
    'rust/crates/sim-systems/src/perf_arms.rs',
    'rust/crates/sim-systems/src/runtime/mind/facts.rs',
    'rust/crates/sim-systems/src/runtime/mind/record.rs',
    'rust/crates/sim-systems/src/runtime/mind/behaviours/mod.rs')
PROVENANCE_DIRS: Final = (*CURRENT_DIRS, 'rust/crates/sim-systems/src',
                         'rust/crates/sim-test/src')
EXCLUDED_DIRS: Final = ('rust/crates/sim-systems/src/runtime/decision',
                       'rust/crates/sim-systems/src/runtime/baseline')
EXCLUDED_FILES: Final = ('rust/crates/sim-engine/src/registration.rs',
    'rust/crates/sim-systems/src/runtime/mod.rs',
    'rust/crates/sim-systems/src/runtime/jobs/mod.rs',
    'rust/crates/sim-systems/src/runtime/jobs/seam.rs')


class FreezeError(Exception):
    """The evidence binding is absent, malformed, changed, or incomplete."""


def canonical(path: Path) -> Path:
    absolute = path.absolute()
    if absolute != absolute.resolve() or any(p.is_symlink() for p in (absolute, *absolute.parents)):
        raise FreezeError(f'noncanonical or symlink path: {path}')
    return absolute


def digest(path: Path) -> str:
    canonical(path)
    before = path.stat()
    hasher = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            hasher.update(block)
    after = path.stat()
    # Reading can update atime; only identity/content metadata detects mutation.
    fields = ('st_dev', 'st_ino', 'st_size', 'st_mtime_ns', 'st_ctime_ns')
    if any(getattr(before, field) != getattr(after, field) for field in fields):
        raise FreezeError(f'file changed while hashing: {path}')
    return hasher.hexdigest()


def unique_pairs(pairs: list[tuple[str, JSON]]) -> Document:
    result: Document = {}
    for key, value in pairs:
        if key in result:
            raise FreezeError(f'duplicate JSON key: {key}')
        result[key] = value
    return result


def load(path: Path) -> Document:
    canonical(path)
    value: JSON = json.loads(path.read_text(), object_pairs_hook=unique_pairs)
    if not isinstance(value, dict):
        raise FreezeError(f'expected JSON object: {path}')
    return value


def external(repo: Path, path: Path) -> Path:
    path = canonical(path)
    if path.is_relative_to(repo):
        raise FreezeError(f'evidence must be external to checkout: {path}')
    return path


def inventory(repo: Path, frozen: bool = False) -> Document:
    files: set[Path] = set()
    for name in PROVENANCE_DIRS if frozen else CURRENT_DIRS:
        directory = canonical(repo / name)
        if not directory.is_dir():
            raise FreezeError(f'missing source scope: {name}')
        for path in directory.rglob('*'):
            canonical(path)
            relative = path.relative_to(repo).as_posix()
            excluded = relative in EXCLUDED_FILES or any(
                relative == prefix or relative.startswith(prefix + '/') for prefix in EXCLUDED_DIRS)
            if path.is_file() and (frozen or not excluded):
                files.add(path)
    for name in REQUIRED_FILES:
        path = canonical(repo / name)
        if not path.is_file():
            raise FreezeError(f'missing required file: {name}')
        files.add(path)
    return {path.relative_to(repo).as_posix(): digest(path) for path in sorted(files)}


def write_new(repo: Path, output: Path, data: Document) -> None:
    external(repo, output)
    try:
        with output.open('x') as stream:
            json.dump(data, stream, indent=2, sort_keys=True)
            stream.write('\n')
    except FileExistsError as error:
        raise FreezeError(f'refusing evidence overwrite: {output}') from error


def pin(path: Path, expected: JSON) -> None:
    if not isinstance(expected, str) or len(expected) != 64 or digest(path) != expected:
        raise FreezeError(f'SHA-256 mismatch: {path}')


def version(data: Document) -> None:
    if type(data.get('schema_version')) is not int or data['schema_version'] != 1:
        raise FreezeError('unsupported schema_version')


def reference(repo: Path, path: Path) -> Document:
    return {'path': str(external(repo, path)), 'sha256': digest(path), 'size': path.stat().st_size}


def read_reference(repo: Path, value: JSON) -> Path:
    if not isinstance(value, dict) or set(value) != {'path', 'sha256', 'size'}:
        raise FreezeError('malformed artifact reference')
    name = value['path']
    if not isinstance(name, str) or not Path(name).is_absolute():
        raise FreezeError('artifact path must be absolute')
    path = external(repo, Path(name))
    pin(path, value['sha256'])
    if type(value['size']) is not int or value['size'] != path.stat().st_size:
        raise FreezeError(f'size mismatch: {path}')
    return path


def prepare(repo: Path, output: Path) -> None:
    repo = canonical(repo)
    current, frozen = inventory(repo), inventory(repo, True)
    if current != inventory(repo) or frozen != inventory(repo, True):
        raise FreezeError('source changed during snapshot')
    write_new(repo, output, {'schema_version': 1, 'kind': 'source_snapshot',
        'checker_sha256': digest(Path(__file__).resolve()),
        'current_files': current, 'frozen_files': frozen})


def validate_receipt(receipt: Document, snapshot_sha: str, shadow: Path, control: Path) -> None:
    version(receipt)
    if set(receipt) != {'schema_version', 'snapshot_sha256', 'shadow', 'control'} or receipt['snapshot_sha256'] != snapshot_sha:
        raise FreezeError('receipt does not attest this source snapshot')
    for name, binary, features in [('shadow', shadow, ['mind_shadow']), ('control', control, [])]:
        row = receipt[name]
        if not isinstance(row, dict) or set(row) != {'sha256', 'exit_code', 'features', 'command'}:
            raise FreezeError(f'invalid builder receipt: {name}')
        if type(row['exit_code']) is not int or row['exit_code'] != 0 or row['features'] != features:
            raise FreezeError(f'build unsuccessful or wrong features: {name}')
        if not isinstance(row['command'], str) or not row['command'].strip():
            raise FreezeError('missing builder command attestation')
        pin(binary, row['sha256'])


def seal(repo: Path, snapshot: Path, snapshot_sha: str, receipt: Path,
         shadow: Path, control: Path, output: Path) -> None:
    repo = canonical(repo)
    pin(snapshot, snapshot_sha)
    data = load(snapshot)
    version(data)
    if data.get('kind') != 'source_snapshot' or data.get('checker_sha256') != digest(Path(__file__).resolve()):
        raise FreezeError('snapshot kind/checker mismatch')
    if data.get('current_files') != inventory(repo) or data.get('frozen_files') != inventory(repo, True):
        raise FreezeError('source changed between preparation and builds')
    if canonical(shadow) == canonical(control):
        raise FreezeError('shadow and control must be separate artifacts')
    validate_receipt(load(receipt), snapshot_sha, shadow, control)
    write_new(repo, output, {**data, 'kind': 'frozen_shadow',
        'snapshot': reference(repo, snapshot), 'receipt': reference(repo, receipt),
        'shadow': reference(repo, shadow), 'control': reference(repo, control)})


def check(repo: Path, manifest: Path, manifest_sha: str) -> None:
    repo = canonical(repo)
    external(repo, manifest)
    pin(manifest, manifest_sha)
    data = load(manifest)
    version(data)
    if set(data) != {'schema_version', 'kind', 'checker_sha256', 'current_files', 'frozen_files', 'snapshot', 'receipt', 'shadow', 'control'}:
        raise FreezeError('invalid manifest fields')
    if data['kind'] != 'frozen_shadow' or data['checker_sha256'] != digest(Path(__file__).resolve()):
        raise FreezeError('manifest kind/checker mismatch')
    if data['current_files'] != inventory(repo):
        raise FreezeError('current semantic source inventory differs from frozen shadow')
    snapshot, receipt, shadow, control = (read_reference(repo, data[key]) for key in ('snapshot', 'receipt', 'shadow', 'control'))
    original = load(snapshot)
    if original != {key: data[key] for key in ('schema_version', 'checker_sha256', 'current_files', 'frozen_files')} | {'kind': 'source_snapshot'}:
        raise FreezeError('manifest differs from pinned source snapshot')
    validate_receipt(load(receipt), digest(snapshot), shadow, control)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='action', required=True)
    for action in ('prepare', 'seal', 'check'):
        command = sub.add_parser(action)
        command.add_argument('--repo', type=Path, required=True)
        if action != 'check':
            command.add_argument('--out', type=Path, required=True)
        if action == 'seal':
            for name in ('snapshot', 'receipt', 'shadow', 'control'):
                command.add_argument('--' + name, type=Path, required=True)
            command.add_argument('--snapshot-sha256', required=True)
        if action == 'check':
            command.add_argument('--manifest', type=Path, required=True)
            command.add_argument('--manifest-sha256', required=True)
    args = parser.parse_args()
    try:
        match args.action:
            case 'prepare': prepare(args.repo, args.out)
            case 'seal': seal(args.repo, args.snapshot, args.snapshot_sha256, args.receipt, args.shadow, args.control, args.out)
            case 'check': check(args.repo, args.manifest, args.manifest_sha256)
        print(f'{args.action}: verified')
        return 0
    except (FreezeError, OSError, ValueError) as error:
        print(f'FAIL: {error}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
