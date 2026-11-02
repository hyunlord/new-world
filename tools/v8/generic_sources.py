# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m unittest discover -s tools/v8 -p test_generic_sources.py
"""Authenticate committed source, local ancestry and separately frozen runtime inputs.

Local remote-tracking refs are not a network publication receipt. The runner must
obtain and recheck the published SHA independently before creating this binding.
"""
import hashlib
import json
import os
import re
import stat
import subprocess
from datetime import UTC, datetime
from pathlib import Path, PurePosixPath

from generic_gate import GateContractError, require
from mux2_gate_inputs import Json


def git(root: Path, *args: str) -> bytes:
    environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
    environment.update(GIT_OPTIONAL_LOCKS="0", GIT_NO_REPLACE_OBJECTS="1", LC_ALL="C")
    result = subprocess.run(["git", "-C", str(root), *args], env=environment,
                            capture_output=True, check=False)
    require(result.returncode == 0, str(root), f"git {args!r}: {result.stderr.decode(errors='replace')}")
    return result.stdout


def safe_path(root: Path, name: str) -> Path:
    relative = PurePosixPath(name)
    require(bool(name) and not relative.is_absolute() and ".." not in relative.parts
            and relative.as_posix() == name and name != ".", name, "unsafe relative input path")
    path = root
    for part in relative.parts[:-1]:
        path /= part
        require(stat.S_ISDIR(path.lstat().st_mode), name, "non-directory input ancestor")
    return root / name


def manifest_hash(value: dict[str, Json]) -> str:
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def freeze_source(root: Path, expected_sha: str) -> dict[str, Json]:
    """Reject dirty/index/hidden-byte drift, without ignoring generated build provenance."""
    require(root.is_absolute() and not root.is_symlink(), str(root), "expected real absolute source root")
    root = root.resolve(strict=True)
    require(re.fullmatch("[0-9a-f]{40}", expected_sha) is not None, str(root), "invalid source SHA")
    require(git(root, "rev-parse", "--show-object-format").strip() == b"sha1", str(root), "unsupported object format")
    require(git(root, "rev-parse", "HEAD").decode().strip() == expected_sha, str(root), "unexpected HEAD")
    status = git(root, "status", "--porcelain=v1", "--untracked-files=all")
    require(not status, str(root), "dirty source checkout")
    tree: dict[str, tuple[str, str]] = {}
    for row in git(root, "ls-tree", "-rz", "HEAD").split(b"\0"):
        if row:
            details, name = row.split(b"\t", 1)
            mode, kind, oid = details.decode().split()
            require(kind == "blob" and mode in {"100644", "100755", "120000"}, str(root), "unsupported tree entry")
            tree[os.fsdecode(name)] = (mode, oid)
    index: dict[str, tuple[str, str]] = {}
    index_bytes = git(root, "ls-files", "--stage", "-z")
    for row in index_bytes.split(b"\0"):
        if row:
            details, name = row.split(b"\t", 1)
            mode, oid, stage = details.decode().split()
            key = os.fsdecode(name)
            require(stage == "0" and key not in index, key, "conflicted or duplicate index entry")
            index[key] = (mode, oid)
    require(bool(tree) and tree == index, str(root), "index differs from HEAD")
    manifest: dict[str, Json] = {}
    try:
        for name, (mode, oid) in sorted(tree.items()):
            path = safe_path(root, name)
            before = path.lstat()
            if mode == "120000":
                require(stat.S_ISLNK(before.st_mode), name, "expected tracked symlink")
                data = os.fsencode(os.readlink(path))
            else:
                require(stat.S_ISREG(before.st_mode), name, "expected regular tracked file")
                require(bool(before.st_mode & 0o111) == (mode == "100755"), name, "executable mode changed")
                data = path.read_bytes()
            after = path.lstat()
            require((before.st_ino, before.st_size, before.st_mtime_ns, before.st_ctime_ns) ==
                    (after.st_ino, after.st_size, after.st_mtime_ns, after.st_ctime_ns), name, "source changed during read")
            blob = hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest()
            require(blob == oid, name, "tracked bytes differ from committed blob")
            manifest[name] = [mode, oid, hashlib.sha256(data).hexdigest()]
    except OSError as error:
        raise GateContractError("SOURCE_UNREADABLE", str(root), str(error)) from error
    require(git(root, "rev-parse", "HEAD").decode().strip() == expected_sha
            and not git(root, "status", "--porcelain=v1", "--untracked-files=all"), str(root), "source changed during inspection")
    require(git(root, "ls-files", "--stage", "-z") == index_bytes, str(root), "index changed during inspection")
    # Git status deliberately trusts assume-unchanged/skip-worktree flags. Re-read
    # bytes after the final Git queries so those flags cannot hide an end mutation.
    try:
        for name, (mode, _) in sorted(tree.items()):
            path = safe_path(root, name)
            before = path.lstat()
            if mode == "120000":
                require(stat.S_ISLNK(before.st_mode), name, "tracked symlink changed during inspection")
                data = os.fsencode(os.readlink(path))
            else:
                require(stat.S_ISREG(before.st_mode) and bool(before.st_mode & 0o111) == (mode == "100755"),
                        name, "tracked type or mode changed during inspection")
                data = path.read_bytes()
            after = path.lstat()
            require((before.st_ino, before.st_size, before.st_mtime_ns, before.st_ctime_ns) ==
                    (after.st_ino, after.st_size, after.st_mtime_ns, after.st_ctime_ns),
                    name, "source changed during final read")
            require(manifest[name] == [mode, tree[name][1], hashlib.sha256(data).hexdigest()],
                    name, "tracked bytes changed during inspection")
    except OSError as error:
        raise GateContractError("SOURCE_UNREADABLE", str(root), str(error)) from error
    return {"source_sha": expected_sha, "root": str(root), "tracked_count": len(tree),
            "tracked_manifest_sha256": manifest_hash(manifest), "tracked_manifest": manifest,
            "source_clean": True, "runtime_inputs_verified": False,
            "limit": "Point-in-time tracked-source proof; generated inputs and binary builds require separate receipts."}


def inventory(root: Path, names: list[str]) -> dict[str, str]:
    """Hash every regular input, including ignored files; symlinks/special files fail closed."""
    require(root.is_absolute() and not root.is_symlink(), str(root), "expected real absolute input root")
    root = root.resolve(strict=True)
    require(bool(names) and len(set(names)) == len(names), str(root), "empty or duplicate input scope")
    result: dict[str, str] = {}

    def visit(path: Path) -> None:
        before = path.lstat()
        if stat.S_ISDIR(before.st_mode):
            children = sorted(path.iterdir())
            for child in children:
                visit(child)
            require(children == sorted(path.iterdir()), str(path), "input directory changed during scan")
        else:
            require(stat.S_ISREG(before.st_mode), str(path), "input is a symlink or special file")
            key = path.relative_to(root).as_posix()
            require(key not in result, key, "overlapping input scopes")
            with path.open("rb") as stream:
                result[key] = hashlib.file_digest(stream, "sha256").hexdigest()
            after = path.lstat()
            require((before.st_ino, before.st_size, before.st_mtime_ns, before.st_ctime_ns) ==
                    (after.st_ino, after.st_size, after.st_mtime_ns, after.st_ctime_ns), key, "input changed during read")
    try:
        for name in names:
            visit(safe_path(root, name))
    except OSError as error:
        raise GateContractError("INPUT_UNREADABLE", str(root), str(error)) from error
    require(bool(result), str(root), "empty input inventory")
    return result


def freeze_binding(root: Path, harness_sha: str, published_sha: str, purpose: str,
                   *, candidate_sha: str | None = None) -> dict[str, Json]:
    """Bind a runner-observed published SHA to a unique local ancestor, never a stale bench pin."""
    for value in (harness_sha, published_sha):
        require(re.fullmatch("[0-9a-f]{40}", value) is not None, str(root), "full commit SHA required")
    require(purpose in {"publication", "self_proof", "defect_proof"}, str(root), "unknown proof purpose")
    if purpose == "self_proof":
        require(candidate_sha is None or candidate_sha == published_sha, str(root), "self proof requires BASE candidate")
        candidate_sha = published_sha
    elif purpose == "publication":
        require(candidate_sha is None or candidate_sha == harness_sha, str(root), "publication requires candidate harness")
        candidate_sha = harness_sha
    else:
        require(candidate_sha is not None and candidate_sha != published_sha, str(root), "defect product must be explicit")
    if candidate_sha is None:
        raise GateContractError("MISSING_PRODUCT", str(root), "candidate product unavailable")
    require(re.fullmatch("[0-9a-f]{40}", candidate_sha) is not None, str(root), "full candidate SHA required")
    local_main = git(root, "rev-parse", "--verify", "refs/remotes/origin/v8/main^{commit}").decode().strip()
    require(local_main == published_sha, str(root), "published main differs from frozen local ref")
    require(git(root, "rev-parse", "--verify", f"{harness_sha}^{{commit}}").decode().strip() == harness_sha,
            str(root), "harness commit unavailable")
    bases = git(root, "merge-base", "--all", harness_sha, published_sha).decode().splitlines()
    require(bases == [published_sha], str(root), "candidate not based on unique published main")
    product_bases = git(root, "merge-base", "--all", candidate_sha, published_sha).decode().splitlines()
    require(product_bases == [published_sha], str(root), "product not based on unique published main")
    if purpose == "defect_proof":
        _ = git(root, "merge-base", "--is-ancestor", harness_sha, candidate_sha)
    return {"schema": 1, "purpose": purpose, "harness_sha": harness_sha,
            "base_product_sha": published_sha,
            "candidate_product_sha": candidate_sha,
            "merge_base": published_sha, "frozen_main": published_sha,
            "publication_authority": False,
            "limit": "Ancestry binding only. Fresh remote readback, independent builds and complete gate evidence remain mandatory."}


def published_binding(root: Path, harness_sha: str, purpose: str,
                      *, candidate_sha: str | None = None) -> dict[str, Json]:
    """Read origin/main around an explicit fetch, then bind the clean harness.

    This updates only the local remote-tracking ref. A publication runner must
    repeat the remote read before its final fast-forward; this is a start receipt.
    """
    before = freeze_source(root, harness_sha)
    reference = "refs/heads/v8/main"

    def remote_head() -> str:
        rows = git(root, "ls-remote", "--exit-code", "origin", reference).decode().splitlines()
        require(len(rows) == 1, "/origin", "published main is missing or ambiguous")
        fields = rows[0].split()
        require(len(fields) == 2 and fields[1] == reference
                and re.fullmatch("[0-9a-f]{40}", fields[0]) is not None,
                "/origin", "invalid published main response")
        return fields[0]

    started = datetime.now(UTC).isoformat()
    published = remote_head()
    _ = git(root, "fetch", "--no-tags", "origin",
        reference + ":refs/remotes/origin/v8/main")
    require(remote_head() == published, "/origin", "published main moved during fetch")
    binding = freeze_binding(root, harness_sha, published, purpose, candidate_sha=candidate_sha)
    require(freeze_source(root, harness_sha) == before, "/source", "harness changed during remote binding")
    binding.update(remote_reference=reference, remote="origin", remote_started_at=started,
                   remote_finished_at=datetime.now(UTC).isoformat(),
                   remote_before_sha=published, remote_after_sha=published,
                   harness_source=before,
                   limit="Start-of-run source/remote binding only; final remote readback and full runtime evidence remain mandatory.")
    return binding
