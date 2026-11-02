"""Materialize a source-authenticated, observer-only historical launch export.

Never builds, launches, fetches, registers worktrees, or edits historical sources.
The caller supplies an independently authenticated build-receipt hash.
"""

import argparse
import hashlib
import io
import json
import re
import shutil
import subprocess
import tarfile
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Final

from generic_gate import decode_json

SUPPORTED: Final[dict[str, str]] = {
    "892edca1d31b26e535a3e9b62b37d62b99bb1130": "m1_preknowledge",
    "b1ca032f8fa84cee87d68a185e471b8e2c2bddce": "v7_preknowledge",
}
AUTOLOAD: Final[str] = 'GateLegacyObserver="*res://gate_observer/legacy.gd"\n'
BINARY: Final[str] = "rust/target/debug/libsim_bridge.dylib"


class OverlayError(RuntimeError):
    """Admission or source-bound overlay validation failed."""


@dataclass(frozen=True)
class Request:
    """Inputs whose ownership remains with the runner."""

    repository: Path
    commit: str
    destination: Path
    binary: Path
    build_receipt: Path
    expected_receipt_sha256: str
    observer: Path


def sha256(path: Path) -> str:
    """Hash bytes without loading a native binary into memory."""
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def git(repository: Path, *args: str) -> bytes:
    """Read immutable Git objects or local registration metadata."""
    return subprocess.run(["git", "-C", str(repository), *args], check=True, capture_output=True).stdout


def autoload_delta(source: str) -> str:
    """Insert only the gate autoload while preserving every original config byte."""
    if "GateLegacyObserver" in source or "res://gate_observer/" in source:
        raise OverlayError("observer namespace already exists")
    section = "[autoload]\n"
    if section in source:
        return source.replace(section, section + AUTOLOAD, 1)
    return source + ("" if source.endswith("\n") else "\n") + "\n" + section + AUTOLOAD


def _tree_hashes(repository: Path, commit: str) -> dict[str, str]:
    """Hash original Git blobs so archive attributes cannot silently alter exports."""
    records = git(repository, "ls-tree", "-rz", commit).split(b"\0")
    blobs: list[tuple[str, str]] = []
    for record in filter(None, records):
        header, path = record.split(b"\t", 1)
        _mode, kind, oid = header.decode().split()
        if kind != "blob":
            raise OverlayError("submodule or non-blob source entry is unsupported")
        blobs.append((path.decode(), oid))
    result = subprocess.run(["git", "-C", str(repository), "cat-file", "--batch"], input=("\n".join(oid for _, oid in blobs) + "\n").encode(), check=True, capture_output=True).stdout
    hashes: dict[str, str] = {}
    cursor = 0
    for name, expected_oid in blobs:
        end = result.index(b"\n", cursor)
        oid, kind, size = result[cursor:end].decode().split()
        if oid != expected_oid or kind != "blob":
            raise OverlayError("Git blob response mismatch")
        start = end + 1
        cursor = start + int(size)
        hashes[name] = hashlib.sha256(result[start:cursor]).hexdigest()
        if result[cursor:cursor + 1] != b"\n":
            raise OverlayError("truncated Git blob response")
        cursor += 1
    return hashes


def _admit(request: Request) -> tuple[Path, str, bytes]:
    if not re.fullmatch(r"[0-9a-f]{40}", request.commit) or request.commit not in SUPPORTED:
        raise OverlayError("only exact qualified historical preimages are supported; main uses the native observer")
    resolved = git(request.repository, "rev-parse", request.commit + "^{commit}").decode().strip()
    if resolved != request.commit:
        raise OverlayError("commit resolution mismatch")
    target = request.destination.resolve()
    if request.destination.exists() or request.destination.is_symlink():
        raise OverlayError("destination must be new")
    worktrees = git(request.repository, "worktree", "list", "--porcelain").decode().splitlines()
    roots = [Path(line.removeprefix("worktree ")).resolve() for line in worktrees if line.startswith("worktree ")]
    roots.append(request.repository.resolve())
    if any(target == root or target.is_relative_to(root) for root in roots):
        raise OverlayError("destination is inside a source or registered worktree")
    receipt_bytes = request.build_receipt.read_bytes()
    if hashlib.sha256(receipt_bytes).hexdigest() != request.expected_receipt_sha256:
        raise OverlayError("build receipt does not match runner trust anchor")
    receipt = decode_json(receipt_bytes)
    tree = git(request.repository, "rev-parse", request.commit + "^{tree}").decode().strip()
    if not isinstance(receipt, dict) or receipt.get("schema_version") != 1 or receipt.get("build_status") != "PASS":
        raise OverlayError("unsupported or incomplete build receipt")
    if receipt.get("product_commit") != request.commit or receipt.get("product_tree") != tree:
        raise OverlayError("build provenance does not bind this product tree")
    if not request.binary.is_file() or receipt.get("binary_sha256") != sha256(request.binary):
        raise OverlayError("native binary hash mismatch")
    if not request.observer.is_file():
        raise OverlayError("missing observer implementation")
    return target, tree, receipt_bytes


def materialize(request: Request) -> Path:
    """Create one fresh export, preserving failures instead of deleting evidence."""
    target, tree, receipt_bytes = _admit(request)
    source_project = git(request.repository, "show", request.commit + ":project.godot")
    patched_project = autoload_delta(source_project.decode()).encode()
    extension = git(request.repository, "show", request.commit + ":sim_bridge.gdextension").decode()
    if not re.search(r'^macos\.debug\s*=\s*"res://' + re.escape(BINARY) + r'"\s*$', extension, re.MULTILINE):
        raise OverlayError("unqualified native library descriptor")
    archive_bytes = git(request.repository, "archive", "--format=tar", request.commit)
    tree_hashes = _tree_hashes(request.repository, request.commit)
    files: dict[str, str] = {}
    archive_hashes: dict[str, str] = {}
    with tarfile.open(fileobj=io.BytesIO(archive_bytes), mode="r:") as archive:
        members = archive.getmembers()
        for member in members:
            path = PurePosixPath(member.name)
            if path.is_absolute() or ".." in path.parts or ".git" in path.parts or path.parts[0] == "gate_observer":
                raise OverlayError("unsafe or reserved archive path")
            if member.isfile():
                stream = archive.extractfile(member)
                if stream is None:
                    raise OverlayError("unreadable archived file")
                checksum = hashlib.sha256()
                with stream:
                    while chunk := stream.read(262144):
                        checksum.update(chunk)
                files[member.name] = checksum.hexdigest()
                archive_hashes[member.name] = files[member.name]
            elif member.issym():
                link = (target / member.name).parent / member.linkname
                if not link.resolve().is_relative_to(target):
                    raise OverlayError("archive symlink leaves source export")
                archive_hashes[member.name] = hashlib.sha256(member.linkname.encode()).hexdigest()
            elif not member.isdir():
                raise OverlayError("unsupported archive entry")
        if BINARY in files:
            raise OverlayError("source unexpectedly contains the runtime binary")
        if archive_hashes != tree_hashes:
            raise OverlayError("archive attributes omitted or transformed committed source")
        target.mkdir(parents=True, exist_ok=False)
        archive.extractall(target, filter="data")
    for name, expected in files.items():
        if sha256(target / name) != expected:
            raise OverlayError("archive extraction changed source bytes: " + name)
    _ = (target / "project.godot").write_bytes(patched_project)
    observer_dir = target / "gate_observer"
    observer_dir.mkdir()
    _ = shutil.copyfile(request.observer, observer_dir / "legacy.gd")
    _ = (observer_dir / "build-receipt.json").write_bytes(receipt_bytes)
    binary_target = target / BINARY
    binary_target.parent.mkdir(parents=True, exist_ok=True)
    _ = shutil.copyfile(request.binary, binary_target)
    injected_receipt = decode_json(receipt_bytes)
    assert isinstance(injected_receipt, dict)
    if sha256(binary_target) != injected_receipt["binary_sha256"]:
        raise OverlayError("binary injection changed bytes")
    receipt = {
        "schema_version": 1, "product_commit": request.commit, "product_tree": tree,
        "product_family": SUPPORTED[request.commit], "pristine_export": False,
        "runtime_status": "NOT_RUN", "lifecycle_qualification": "PENDING",
        "census_qualification": "PENDING", "knowledge_enabled": False,
        "knowledge_capability_source": "pinned_preknowledge_native_schema",
        "knowledge_lost": None, "source_files_sha256": tree_hashes,
        "archive_sha256": hashlib.sha256(archive_bytes).hexdigest(),
        "project_preimage_sha256": hashlib.sha256(source_project).hexdigest(),
        "project_overlay_sha256": hashlib.sha256(patched_project).hexdigest(),
        "observer_sha256": sha256(observer_dir / "legacy.gd"),
        "exporter_sha256": sha256(Path(__file__)),
        "build_receipt_sha256": request.expected_receipt_sha256,
        "binary_relative_path": BINARY, "binary_sha256": sha256(binary_target),
        "launch_args": ["--headless", "--path", str(target), "--quit-after", "300"],
        "overlay_scope": ["project.godot: add gate autoload", "gate_observer/: observer and provenance", BINARY + ": authenticated native binary"],
    }
    _ = (observer_dir / "provenance.json").write_text(json.dumps(receipt, indent=2) + "\n")
    return target


def main() -> None:
    """CLI only materializes; execution and qualification belong to the runner."""
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("repository", "destination", "binary", "build-receipt", "observer"):
        _ = parser.add_argument("--" + name, required=True, type=Path)
    _ = parser.add_argument("--commit", required=True)
    _ = parser.add_argument("--expected-receipt-sha256", required=True)
    args: dict[str, object] = vars(parser.parse_args())

    def path_arg(name: str) -> Path:
        value = args[name]
        if not isinstance(value, Path):
            raise OverlayError("expected path argument: " + name)
        return value

    def text_arg(name: str) -> str:
        value = args[name]
        if not isinstance(value, str):
            raise OverlayError("expected text argument: " + name)
        return value

    result = materialize(Request(path_arg("repository"), text_arg("commit"), path_arg("destination"),
                                 path_arg("binary"), path_arg("build_receipt"),
                                 text_arg("expected_receipt_sha256"), path_arg("observer")))
    print(result)


if __name__ == "__main__":
    main()
