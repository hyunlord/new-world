# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Provisional stall evidence only; final all-step and world-safety admission is separate."""
import math
import re
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Literal

from generic_gate import decode_json
from m6a_r1_stall_capture import producer_sources, required_artifacts
from mux2_gate_inputs import (
    BRIDGE,
    EvidenceError,
    Json,
    Node,
    check_hash,
    clean_sources,
    digest,
    git,
    require,
    same_inputs,
    snapshot,
)
from mux2_gate_stall import ArmTiming, validate_stall_capture


@dataclass(frozen=True, slots=True)
class StallVerdict:
    strict_pass: bool
    status: Literal['STRICT_PASS', 'WAIVER_ELIGIBLE']
    arms: tuple[ArmTiming, ...]


@dataclass(frozen=True, slots=True)
class CaptureBinding:
    """Facts from the trusted child runner, never from a user-authored capture report."""
    source_sha: str
    argv: tuple[str, ...]
    process_exit: int
    receipt_sha256: str


@dataclass(frozen=True, slots=True)
class StallEvidence:
    source_sha: str
    verdict: StallVerdict
    original_exit: int
    original_complete: bool
    receipt_sha256: str
    artifacts: tuple[tuple[str, str], ...]
    publication_authority: Literal[False] = False


def parse(raw: str) -> Node:
    """Reject ambiguous JSON before using existing typed telemetry validators."""
    def pairs(rows: list[tuple[str, Json]]) -> dict[str, Json]:
        result: dict[str, Json] = {}
        for key, value in rows:
            require(key not in result, f'duplicate JSON key: {key}')
            result[key] = value
        return result

    def constant(value: str) -> Json:
        raise EvidenceError(f'nonfinite JSON: {value}')

    def finite(value: Json) -> None:
        match value:
            case float():
                require(math.isfinite(value), 'nonfinite JSON number')
            case list():
                for child in value:
                    finite(child)
            case dict():
                for child in value.values():
                    finite(child)
            case None | bool() | int() | str():
                return
    try:
        value = decode_json(raw, pairs_hook=pairs, constant_hook=constant)
    except (ValueError, TypeError) as error:
        raise EvidenceError(f'invalid stall JSON: {error}') from error
    finite(value)
    node = Node(value)
    _ = node.mapping()
    return node


def assess_timing(raw: str) -> StallVerdict:
    """Retain strict33 diagnostics; a waiver requires an actual over50 frame."""
    arms = validate_stall_capture(parse(raw).value, strict=False)
    strict = all(arm.strict_pass for arm in arms)
    require(all(len(arm.over50) <= 2 and all(frame.interval_ms <= 60 for frame in arm.over50)
                for arm in arms), 'stall exceeds two over50 frames per zoom or 60ms maximum')
    require(strict or any(arm.over50 for arm in arms), 'strict33-only failure has no over50 waiver')
    return StallVerdict(strict, 'STRICT_PASS' if strict else 'WAIVER_ELIGIBLE', arms)


def real_path(path: Path) -> None:
    require(path.is_absolute() and path.resolve(strict=True) == path, f'noncanonical or symlink path: {path}')


def artifact_inventory(capture: Path) -> dict[str, Json]:
    result: dict[str, Json] = {}
    for path in sorted(capture.rglob('*')):
        require(not path.is_symlink(), f'symlink capture artifact: {path}')
        require(path.is_dir() or path.is_file(), f'special capture artifact: {path}')
        if path.is_file() and path != capture / 'receipt.json':
            result[str(path)] = digest(path)
    return result


def authenticate_capture(root: Path, capture: Path, binding: CaptureBinding) -> StallEvidence:
    """Authenticate a completed child capture without upgrading its raw strict failure.

    Caller authenticates the runner itself and initial/end source freeze, then joins
    this provisional result with every required step and ordinary-world safety.
    """
    try:
        return _authenticate(root, capture, binding)
    except (OSError, ValueError) as error:
        raise EvidenceError(f'unreadable or invalid stall evidence: {error}') from error


def _authenticate(root: Path, capture: Path, binding: CaptureBinding) -> StallEvidence:
    real_path(root)
    real_path(capture)
    require(not capture.is_relative_to(root), 'capture evidence must be external')
    require(re.fullmatch('[0-9a-f]{40}', binding.source_sha) is not None, 'full source SHA required')
    require(type(binding.process_exit) is int and binding.process_exit in (0, 1), 'invalid native capture exit')
    require(re.fullmatch('[0-9a-f]{64}', binding.receipt_sha256) is not None, 'runner receipt hash required')
    clean_sources(root)
    require(git(root, 'rev-parse', 'HEAD') == binding.source_sha, 'source SHA changed')
    receipt_path = capture / 'receipt.json'
    real_path(receipt_path)
    check_hash(receipt_path, binding.receipt_sha256)
    receipt = parse(receipt_path.read_text())
    receipt.field('schema').equals(1)
    receipt.field('kind').equals('stall')
    receipt.field('source_head').equals(binding.source_sha)
    receipt.field('root').equals(str(root))
    before = receipt.field('inputs').mapping()
    same_inputs(before, receipt.field('inputs_end').mapping())
    current = snapshot(root)
    same_inputs(before, current)
    for name in current:
        real_path(root / name)
    godot = Path(receipt.field('godot').text())
    real_path(godot)
    check_hash(godot, receipt.field('godot_sha256').text())
    started = datetime.fromisoformat(receipt.field('started_utc').text())
    finished = datetime.fromisoformat(receipt.field('finished_utc').text())
    require(started.tzinfo is not None and finished.tzinfo is not None and finished > started,
            'invalid capture timestamps')
    required_artifacts(capture)
    artifacts = artifact_inventory(capture)
    report_path = capture / 'run/stall-report.json'
    report_text = report_path.read_text()
    verdict = assess_timing(report_text)
    report = parse(report_text)
    report.field('environment').field('settings_dir').equals(str(capture / 'settings'))
    report.field('bridge').field('sha256').equals(digest(root / BRIDGE))
    sources = report.field('source_sha256').mapping()
    require(set(sources) == producer_sources(root), 'producer source inventory mismatch')
    for name, value in sources.items():
        path = root / name
        real_path(path)
        require(path.is_relative_to(root), 'producer source outside root')
        check_hash(path, Node(value).text())
    expected_exit = 0 if verdict.strict_pass else 1
    require(binding.process_exit == expected_exit, 'actual child exit differs from strict capture verdict')
    receipt.field('exit_code').equals(expected_exit)
    receipt.field('complete').equals(verdict.strict_pass)
    command = (capture / 'command.log').read_text()
    lines = command.splitlines()
    require(bool(binding.argv) and bool(lines), 'missing actual capture argv')
    parse('{"argv":' + lines[0] + '}').field('argv').equals(list(binding.argv))
    require('--headless' not in binding.argv and '--windowed' in binding.argv
            and 'res://tools/v8/mux2_stall_test.gd' in binding.argv
            and str(root) in binding.argv and str(capture / 'run') in binding.argv
            and str(godot) in binding.argv, 'wrong windowed capture command')
    terminals = re.findall(r'^MUX2_STALL_DONE complete=true pass=(true|false) errors=\[\] evidence=(.+)$', command, re.MULTILINE)
    require(terminals == [('true' if verdict.strict_pass else 'false', str(report_path))], 'invalid stall terminal')
    require('SCRIPT ERROR' not in command and not re.search(r'^ERROR:', command, re.MULTILINE), 'capture engine/script error')
    sealed = receipt.field('artifacts').mapping()
    if verdict.strict_pass:
        same_inputs(sealed, artifacts)
    else:
        require(sealed == {}, 'unexpected failed-capture artifact sealing')
    same_inputs(current, snapshot(root))
    same_inputs(artifacts, artifact_inventory(capture))
    clean_sources(root)
    require(git(root, 'rev-parse', 'HEAD') == binding.source_sha, 'source changed during stall audit')
    check_hash(godot, receipt.field('godot_sha256').text())
    check_hash(receipt_path, binding.receipt_sha256)
    return StallEvidence(binding.source_sha, verdict, expected_exit, verdict.strict_pass,
                         binding.receipt_sha256,
                         tuple((name, Node(value).text()) for name, value in sorted(artifacts.items())))
