#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: imported by m6a_r1_stall.py capture/verify.
"""Seal complete R1 GPU captures, including authentic strict timing failures."""
from __future__ import annotations

import json
import math
import os
import re
from pathlib import Path
import subprocess
import time

from m6a_protocol import R1_DEADLINE
from dataclasses import dataclass
from typing import Final
from mux2_gate_stall import ArmTiming, validate_stall_capture
from mux2_gate_inputs import (BRIDGE, EvidenceError, Json, Node, check_hash, clean_sources, digest,
                             external, git, read_json, require, same_inputs, snapshot)
from mux2_gate_lock import FD_ENV, lane
from mux2_gate_receipts import now, validate_p7_admission, write


AUTHORITY: Final = 'M6a owner decision 2026-10-30 11:20 KST, section 5'


@dataclass(frozen=True, slots=True)
class StallVerdict:
    arms: tuple[ArmTiming, ...]

    @property
    def strict_pass(self) -> bool:
        return all(arm.strict_pass for arm in self.arms)

    @property
    def waiver_eligible(self) -> bool:
        return all(len(arm.over50) <= 2
                   and all(frame.interval_ms <= 60 for frame in arm.over50)
                   and all(count <= 3 for count in arm.over33_per_minute)
                   for arm in self.arms)

    def record(self) -> dict[str, Json]:
        """Serialize recomputed facts, never overwrite the original producer verdict."""
        arms: list[Json] = []
        for arm in self.arms:
            frames: list[Json] = [
                {'elapsed_seconds': frame.elapsed_seconds, 'interval_ms': frame.interval_ms,
                 'tick': frame.tick, 'ticks_advanced': frame.ticks_advanced}
                for frame in arm.over50]
            arms.append({'zoom': arm.zoom, 'strict_pass': arm.strict_pass,
                         'over33_per_minute': list(arm.over33_per_minute), 'over50_frames': frames})
        return {'strict_pass': self.strict_pass, 'waiver_eligible': self.waiver_eligible,
                'arms': arms}


def evaluate(raw: Json) -> StallVerdict:
    """Require complete authentic frame telemetry before considering timing eligibility."""
    return StallVerdict(validate_stall_capture(raw))


def producer_sources(root: Path) -> set[str]:
    """Read the producer's literal hash manifest; reject unsupported or empty syntax."""
    source = (root / 'tools/v8/mux2_stall_test.gd').read_text()
    match = re.search(r'const SOURCE_PATHS: Array\[String\] = \[(.*?)\]', source, re.DOTALL)
    if match is None:
        raise EvidenceError('producer source manifest missing')
    body = match.group(1)
    names = re.findall(r'"([^"\n]+)"', body)
    require(bool(names) and len(names) == len(set(names))
            and not re.sub(r'"[^"\n]+"|[\s,]', '', body), 'invalid producer source manifest')
    return set(names)


def required_artifacts(output: Path) -> None:
    """Require both arm settings and raw producer/log bytes before sealing any inventory."""
    for relative in ('command.log', 'run/stall-report.json'):
        path = output / relative
        require(path.is_file() and path.stat().st_size > 0, 'missing required capture artifact: ' + relative)
    settings = sorted((output / 'settings').glob('stall-*.cfg'))
    matches = [re.fullmatch(r'stall-(\d+)-(1\.0|0\.5)\.cfg', path.name) for path in settings]
    require(len(matches) == 2 and all(match is not None for match in matches), 'missing per-arm settings')
    parsed = [match for match in matches if match is not None]
    require({match.group(2) for match in parsed} == {'1.0', '0.5'}
            and len({match.group(1) for match in parsed}) == 1
            and all(path.stat().st_size > 0 for path in settings), 'invalid per-arm settings')


def validate_output(root: Path, output: Path) -> StallVerdict:
    """Authenticate producer output independently of its subprocess exit status."""
    required_artifacts(output)
    require('SCRIPT ERROR' not in (output / 'command.log').read_text(), 'capture script error')
    report = read_json(output / 'run/stall-report.json')
    verdict = evaluate(report.value)
    report.field('environment').field('settings_dir').equals(str(output / 'settings'))
    validate_p7_admission(report)
    require(report.field('p7_admission').field('deadline_epoch').number() == R1_DEADLINE,
            'stall admission uses another protocol deadline')
    report.field('bridge').field('sha256').equals(digest(root / BRIDGE))
    sources = report.field('source_sha256').mapping()
    require(set(sources) == producer_sources(root), 'producer source hash inventory differs')
    for name, value in sources.items():
        relative = Path(name)
        require(not relative.is_absolute() and '..' not in relative.parts, 'invalid producer source path')
        check_hash(root / relative, Node(value).text())
    return verdict


def check_exit(code: float, verdict: StallVerdict) -> None:
    require(code == (0 if verdict.strict_pass else 1), 'capture exit differs from strict verdict')


def command(root: Path, output: Path, godot: Path) -> list[str]:
    """Keep the ordinary windowed producer and bound all load waits by the R1 deadline."""
    remaining = math.floor(R1_DEADLINE - time.time())
    require(remaining >= 650, 'NOT_RUN: R1 deadline cannot fit both stall arms')
    return ['bash', str(root / 'tools/v8/run_bounded.sh'), str(min(960 + 2 * 8 * 900, remaining)),
            str(godot), '--path', str(root), '--resolution', '1920x1080', '--always-on-top',
            '--windowed', '--script', 'res://tools/v8/mux2_stall_test.gd', '--', str(output / 'run')]


def capture(root: Path, output: Path, godot: Path) -> StallVerdict:
    out = external(output, root)
    require(not out.exists(), 'capture output must be new')
    clean_sources(root)
    require(godot.is_file(), 'Godot executable missing')
    argv = command(root, out, godot)
    out.mkdir(parents=True)
    with lane() as descriptor:
        before = snapshot(root)
        receipt: dict[str, Json] = {'schema': 1, 'kind': 'm6a_r1_stall', 'complete': False,
            'authority': AUTHORITY, 'exit_code': -1, 'started_utc': now(),
            'source_head': git(root, 'rev-parse', 'HEAD'), 'root': str(root),
            'inputs': before, 'godot': str(godot), 'godot_sha256': digest(godot), 'artifacts': {}}
        receipt_path = out / 'receipt.json'
        write(receipt_path, receipt)
        env = dict(os.environ, WORLDSIM_TEST_SETTINGS_DIR=str(out / 'settings'),
                   WORLDSIM_MUX2_PROFILE='1', CORE_P7_RUN='1',
                   CORE_P7_DEADLINE_EPOCH=str(R1_DEADLINE), **{FD_ENV: str(descriptor)})
        (out / 'settings').mkdir()
        code = -1
        try:
            with (out / 'command.log').open('w') as log:
                _ = log.write(json.dumps(argv) + '\n')
                log.flush()
                code = subprocess.run(argv, cwd=root, env=env, stdout=log,
                    stderr=subprocess.STDOUT, pass_fds=(descriptor,), check=False).returncode
            same_inputs(before, snapshot(root))
            clean_sources(root)
            require(git(root, 'rev-parse', 'HEAD') == receipt['source_head'], 'commit changed during capture')
            check_hash(godot, Node(receipt['godot_sha256']).text())
            verdict = validate_output(root, out)
            check_exit(code, verdict)
            receipt['verdict'] = verdict.record()
            artifacts = sorted(p for p in out.rglob('*') if p.is_file() and p != receipt_path)
            receipt['artifacts'] = {str(path): digest(path) for path in artifacts}
            receipt['complete'] = True
        finally:
            receipt['exit_code'] = code
            receipt['finished_utc'] = now()
            receipt['inputs_end'] = snapshot(root)
            write(receipt_path, receipt)
    return verdict


def verify_capture(root: Path, receipt_path: Path) -> StallVerdict:
    path = external(receipt_path, root)
    receipt = read_json(path)
    receipt.field('schema').equals(1)
    receipt.field('kind').equals('m6a_r1_stall')
    receipt.field('authority').equals(AUTHORITY)
    receipt.field('complete').equals(True)
    receipt.field('root').equals(str(root))
    clean_sources(root)
    before = receipt.field('inputs').mapping()
    same_inputs(before, receipt.field('inputs_end').mapping())
    same_inputs(before, snapshot(root))
    receipt.field('source_head').equals(git(root, 'rev-parse', 'HEAD'))
    check_hash(Path(receipt.field('godot').text()), receipt.field('godot_sha256').text())
    artifacts = receipt.field('artifacts').mapping()
    require(bool(artifacts), 'missing raw artifact hashes')
    for name, value in artifacts.items():
        artifact = external(Path(name), root)
        require(artifact.is_relative_to(path.parent), 'artifact lies outside this capture')
        check_hash(artifact, Node(value).text())
    actual = sorted(p for p in path.parent.rglob('*') if p.is_file() and p != path)
    same_inputs(artifacts, {str(item): digest(item) for item in actual})
    verdict = validate_output(root, path.parent)
    check_exit(receipt.field('exit_code').number(), verdict)
    receipt.field('verdict').equals(verdict.record())
    return verdict
