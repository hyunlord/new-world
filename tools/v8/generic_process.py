# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m unittest discover -s tools/v8 -p test_generic_process.py
"""Execute one trusted gate plan with retained native outcomes and byte bindings.

This is an evidence producer, not a sandbox or build attestation. Endpoint hashes
cannot prove a child never changed and restored inputs between observations.
"""
import hashlib
import json
import math
import os
import re
import stat
import subprocess
import time
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path, PurePosixPath
from typing import Literal

from generic_gate import GateContractError, require
from generic_sources import inventory, safe_path
from mux2_gate_inputs import Json


@dataclass(frozen=True, slots=True)
class ProcessSpec:
    argv: tuple[str, ...]
    cwd: Path
    executable: Path
    executable_sha256: str
    input_root: Path
    inputs: tuple[tuple[str, str], ...]
    output_root: Path
    environment: tuple[tuple[str, str], ...] = ()
    external_inputs: tuple[tuple[Path, str], ...] = ()
    fresh_directories: tuple[str, ...] = ()
    timeout_seconds: float | None = None


@dataclass(frozen=True, slots=True)
class ProcessResult:
    receipt: Path
    status: Literal["PASS", "FAIL"]
    native_exit: int | None
    pid: int | None


def timestamp() -> str:
    return datetime.now(UTC).isoformat()


def real_path(path: Path, *, directory: bool) -> None:
    require(path.is_absolute() and path == path.resolve(strict=True), str(path), "noncanonical or symlink path")
    mode = path.lstat().st_mode
    require(stat.S_ISDIR(mode) if directory else stat.S_ISREG(mode), str(path), "wrong path type")


def file_hash(path: Path) -> str:
    real_path(path, directory=False)
    before = path.stat()
    with path.open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    after = path.stat()
    require((before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns, before.st_ctime_ns) ==
            (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns, after.st_ctime_ns),
            str(path), "file changed during hash")
    return digest


def check_digest(value: str, label: str) -> None:
    require(re.fullmatch(r"[0-9a-f]{64}", value) is not None, label, "invalid SHA256")


def environment_for(spec: ProcessSpec) -> dict[str, str]:
    """No inherited process environment, including inherited simulation overrides."""
    environment = {"PATH": "/usr/bin:/bin:/usr/sbin:/sbin", "LANG": "C", "LC_ALL": "C"}
    seen: set[str] = set()
    for key, value in spec.environment:
        require(re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", key) is not None,
                "/environment", "invalid environment name")
        require(key not in seen and "\0" not in value, "/environment", "duplicate key or NUL value")
        seen.add(key)
        environment[key] = value
    return environment


def atomic_json(path: Path, value: dict[str, Json]) -> None:
    """Publish complete evidence without replacing any existing terminal receipt."""
    pending = path.with_suffix(path.suffix + ".pending")
    with pending.open("x", encoding="utf-8") as stream:
        json.dump(value, stream, indent=2, sort_keys=True, allow_nan=False)
        _ = stream.write("\n")
        stream.flush()
        os.fsync(stream.fileno())
    os.link(pending, path)
    pending.unlink()


def execute(spec: ProcessSpec) -> ProcessResult:
    """Run a shell-free child; native failure is a retained receipt, never discarded."""
    require(bool(spec.argv) and all("\0" not in value for value in spec.argv), "/argv", "empty argv or NUL")
    require(spec.argv[0] == str(spec.executable), "/argv/0", "argv executable differs from bound executable")
    if spec.timeout_seconds is not None:
        require(type(spec.timeout_seconds) in (int, float) and 0 < spec.timeout_seconds < math.inf,
                "/timeout_seconds", "timeout must be finite and positive, excluding bool")
    check_digest(spec.executable_sha256, "/executable_sha256")
    names = [name for name, _ in spec.inputs]
    require(len(names) == len(set(names)), "/inputs", "duplicate declared input")
    for name, digest in spec.inputs:
        check_digest(digest, name)
    external_names = [str(path) for path, _ in spec.external_inputs]
    require(len(external_names) == len(set(external_names)), "/external_inputs", "duplicate external input")
    for path, digest in spec.external_inputs:
        check_digest(digest, str(path))
    require(len({name.casefold() for name in spec.fresh_directories}) == len(spec.fresh_directories), "/fresh_directories", "duplicate directory")
    reserved = {"stdout.log", "stderr.log", "process-start.json", "process-spawn.json", "process.json"}
    reserved |= {name + ".pending" for name in reserved}
    for name in spec.fresh_directories:
        relative = PurePosixPath(name)
        require(bool(name) and "\0" not in name and not relative.is_absolute() and ".." not in relative.parts and
                relative.as_posix() == name and name != "." and relative.parts[0].casefold() not in reserved,
                "/fresh_directories", "unsafe or reserved directory")
    environment = environment_for(spec)
    real_path(spec.cwd, directory=True)
    real_path(spec.input_root, directory=True)
    output = spec.output_root
    require(output.is_absolute() and output.name not in {"", ".", ".."}, str(output), "invalid output root")
    real_path(output.parent, directory=True)
    require(output == output.parent / output.name and not output.exists() and not output.is_symlink(),
            str(output), "output root already exists or is unsafe")
    output.mkdir(mode=0o700)
    receipt: dict[str, Json] = {
        "schema_version": 1, "argv": list(spec.argv), "cwd": str(spec.cwd),
        "executable": str(spec.executable), "expected_executable_sha256": spec.executable_sha256,
        "input_root": str(spec.input_root), "expected_inputs": dict(spec.inputs),
        "output_root": str(output), "environment": {key: value for key, value in environment.items()},
        "expected_external_inputs": {str(path): digest for path, digest in spec.external_inputs},
        "fresh_directories": list(spec.fresh_directories),
        "environment_policy": "fixed_defaults_plus_explicit_only_no_inheritance",
        "started_at": timestamp(), "pid": None, "native_exit": None,
        "status": "FAIL", "errors": [], "build_provenance_verified": False,
    }
    if spec.timeout_seconds is not None:
        receipt["timeout_seconds"] = spec.timeout_seconds
        receipt["timed_out"] = False
    errors: list[str] = []
    before_inputs: dict[str, str] = {}
    before_external_inputs: dict[str, str] = {}
    before_binary: str | None = None
    native_exit: int | None = None
    pid: int | None = None
    started = time.monotonic_ns()
    try:
        for name in spec.fresh_directories:
            directory = output
            for component in PurePosixPath(name).parts:
                directory /= component
                directory.mkdir(mode=0o700, exist_ok=True)
                real_path(directory, directory=True)
        before_binary = file_hash(spec.executable)
        require(os.access(spec.executable, os.X_OK), str(spec.executable), "not executable")
        require(before_binary == spec.executable_sha256, str(spec.executable), "executable hash mismatch")
        for name in names:
            real_path(safe_path(spec.input_root, name), directory=False)
        before_inputs = inventory(spec.input_root, names) if names else {}
        require(before_inputs == dict(spec.inputs), "/inputs", "input hashes mismatch")
        for path, digest in spec.external_inputs:
            real_path(path, directory=False)
            require(not path.is_relative_to(output) and not output.is_relative_to(path),
                    str(path), "external input overlaps output subtree")
            require(not path.is_relative_to(spec.input_root), str(path), "external input belongs in materialized input inventory")
            before_external_inputs[str(path)] = file_hash(path)
            require(before_external_inputs[str(path)] == digest, str(path), "external input hash mismatch")
    except (OSError, GateContractError) as error:
        errors.append(f"preflight: {error}")
    receipt["before_executable_sha256"] = before_binary
    receipt["before_inputs"] = {key: value for key, value in before_inputs.items()}
    receipt["before_external_inputs"] = {key: value for key, value in before_external_inputs.items()}
    receipt["after_external_inputs"] = {}
    atomic_json(output / "process-start.json", receipt)
    with (output / "stdout.log").open("xb") as stdout, (output / "stderr.log").open("xb") as stderr:
        if not errors:
            try:
                with subprocess.Popen(spec.argv, cwd=spec.cwd, env=environment, stdin=subprocess.DEVNULL,
                                      stdout=stdout, stderr=stderr, shell=False, close_fds=True) as child:
                    pid = child.pid
                    try:
                        atomic_json(output / "process-spawn.json", {"pid": pid, "spawned_at": timestamp()})
                    except OSError as error:
                        errors.append(f"spawn receipt: {error}")
                    if spec.timeout_seconds is None:
                        native_exit = child.wait()
                    else:
                        try:
                            native_exit = child.wait(timeout=spec.timeout_seconds)
                        except subprocess.TimeoutExpired:
                            receipt["timed_out"] = True
                            errors.append(f"timeout after {spec.timeout_seconds} seconds")
                            child.terminate()
                            try:
                                native_exit = child.wait(timeout=5)
                            except subprocess.TimeoutExpired:
                                child.kill()
                                native_exit = child.wait()
                    if native_exit != 0:
                        errors.append(f"native exit {native_exit}")
            except OSError as error:
                errors.append(f"spawn: {error}")
    receipt["pid"] = pid
    receipt["native_exit"] = native_exit
    receipt["ended_at"] = timestamp()
    receipt["duration_ns"] = time.monotonic_ns() - started
    try:
        after_binary = file_hash(spec.executable)
        require(os.access(spec.executable, os.X_OK), str(spec.executable), "executable mode changed")
        after_inputs = inventory(spec.input_root, names) if names else {}
        receipt["after_executable_sha256"] = after_binary
        receipt["after_inputs"] = {key: value for key, value in after_inputs.items()}
        require(after_binary == spec.executable_sha256 == before_binary,
                str(spec.executable), "executable changed")
        require(after_inputs == before_inputs == dict(spec.inputs), "/inputs", "inputs changed")
    except (OSError, GateContractError) as error:
        errors.append(f"postflight: {error}")
    # Independently recheck external files even if another postflight binding failed.
    after_external_inputs: dict[str, str] = {}
    for path, digest in spec.external_inputs:
        try:
            after_external_inputs[str(path)] = file_hash(path)
            require(after_external_inputs[str(path)] == before_external_inputs.get(str(path)) == digest,
                    str(path), "external input changed")
        except (OSError, GateContractError) as error:
            errors.append(f"postflight external input: {error}")
    receipt["after_external_inputs"] = {key: value for key, value in after_external_inputs.items()}
    receipt["stdout_sha256"] = file_hash(output / "stdout.log")
    receipt["stderr_sha256"] = file_hash(output / "stderr.log")
    receipt["errors"] = [error for error in errors]
    status: Literal["PASS", "FAIL"] = "PASS" if not errors and native_exit == 0 else "FAIL"
    receipt["status"] = status
    receipt["limit"] = "Endpoint executable/input binding only; no build, descendants, output schema, or whole-gate authority."
    terminal = output / "process.json"
    atomic_json(terminal, receipt)
    return ProcessResult(terminal, status, native_exit, pid)
