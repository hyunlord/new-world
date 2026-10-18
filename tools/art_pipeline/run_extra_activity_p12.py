#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: P12_EXTRA_BASE_PLAN=.../exclusions-before-crops.json P12_DEADLINE_EPOCH=... python3 run_extra_activity_p12.py CHECKOUT NEWDIR SEED DAY HOUR CENTER_KIND
"""Preregister and run one bounded missing-kind capture, preserving failures."""
from __future__ import annotations

import json
import os
import subprocess
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

import run_activity_captures_p12 as base_runner

from run_activity_captures_p12 import (
    QUIET, SCRIPT, Capture, InvalidInput, Json, bounded_launch, bridge_identity,
    integer, launch_command, record, remaining, sha256, source_digest, utc,
    validate_capture, wait_for_quiet,
)


@dataclass(frozen=True, slots=True)
class ExtraCapture:
    capture: Capture
    center_kind: int


def parse_case(values: list[str], winner: int) -> ExtraCapture:
    """Keep supplemental captures in the same two worlds and allowed hours."""
    if len(values) != 4:
        raise InvalidInput('requires SEED DAY HOUR CENTER_KIND')
    seed, day, hour, kind = (int(value) for value in values)
    if (seed, day) not in {(20261010, 30), (winner, 60)}:
        raise InvalidInput('extra capture must reuse a base seed/day')
    if hour not in {11, 14} or kind not in range(7):
        raise InvalidInput('extra capture requires hour11/14 and kind0..6')
    return ExtraCapture(Capture(seed, day, hour, 4), kind)


def run(arguments: list[str]) -> int:
    checkout, output = Path(arguments[0]).resolve(), Path(arguments[1]).absolute()
    winner = 20261018
    spec = parse_case(arguments[2:], winner)
    base_plan = Path(os.environ['P12_EXTRA_BASE_PLAN']).resolve()
    planned: Json = json.loads(base_plan.read_text())
    missing = record(planned)['missing_kinds']
    if not isinstance(missing, list) or spec.center_kind not in missing:
        raise InvalidInput('center kind must be missing in the recorded pre-crop plan')
    deadline = float(os.environ.get('P12_DEADLINE_EPOCH', '0'))
    remaining(deadline)
    seconds = int(os.environ.get('P12_CAPTURE_LIMIT', '1800'))
    godot = Path(os.environ.get('GODOT_BIN', '/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot'))
    if not 1 <= seconds <= 3600 or not os.access(godot, os.X_OK):
        raise InvalidInput('requires executable Godot and capture limit1..3600')
    identity = bridge_identity(checkout, os.environ.get('P12_ACTIVITY_BUILD_PROFILE', 'release'))
    digest = source_digest(checkout)
    output.mkdir(parents=False, exist_ok=False)
    (output / 'base-plan.json').write_bytes(base_plan.read_bytes())
    (output / 'working-tree.patch').write_bytes(subprocess.check_output(['git', 'diff', 'HEAD', '--'], cwd=checkout))
    (output / 'git-status.txt').write_bytes(subprocess.check_output(['git', 'status', '--short'], cwd=checkout))
    head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=checkout, text=True).strip()
    registration = {'preregistered': utc(), 'checkout': str(checkout), 'head': head,
                    'capture': asdict(spec.capture), 'center_kind': spec.center_kind,
                    'centering_rule': 'lowest AgentId arrived holder of center_kind',
                    'source_digest': digest, 'build_identity': asdict(identity),
                    'script_sha256': sha256(checkout / SCRIPT),
                    'wrapper_sha256': sha256(Path(__file__)), 'runner_path': str(Path(base_runner.__file__).resolve()),
                    'runner_sha256': sha256(Path(base_runner.__file__)),
                    'base_plan_path': str(base_plan), 'base_plan_sha256': sha256(base_plan),
                    'fixed_day60_seed': winner, 'deadline_epoch': deadline,
                    'limit_seconds': seconds, 'quiet_lock': str(QUIET), 'pose_oracle_checked': False}
    (output / 'preregistration.json').write_text(json.dumps(registration, indent=2) + '\n')
    # Preregistration and copied reasons exist before lock waits or process launch.
    errors: list[str] = []
    code = 2
    started = utc()
    try:
        wait_for_quiet(deadline, output / 'quiet-waits.log', spec.capture.name)
        if source_digest(checkout) != digest:
            raise InvalidInput('source changed after preregistration')
        environment = os.environ.copy()
        environment.update(P12_ACTIVITY_OUT=str(output), P12_ACTIVITY_BUILD_PROFILE=identity.build_profile,
                           P12_ACTIVITY_SEED=str(spec.capture.seed), P12_ACTIVITY_DAY=str(spec.capture.day),
                           P12_ACTIVITY_HOUR=str(spec.capture.hour), P12_ACTIVITY_ZOOM='4',
                           P12_ACTIVITY_CENTER_KIND=str(spec.center_kind))
        with (output / 'launch.log').open('w') as log:
            code = bounded_launch(launch_command(checkout, godot, seconds), checkout, environment, log, deadline, seconds)
        payload: Json = json.loads((output / 'capture.json').read_text())
        validate_capture(payload, spec.capture)
        captured = record(payload)
        if captured.get('center_kind') != spec.center_kind or captured.get('draw_order_complete') is not True or captured.get('draw_order_schema') != 3:
            raise InvalidInput('capture center/order contract mismatch')
        if source_digest(checkout) != digest:
            raise InvalidInput('source changed during capture')
    except (OSError, ValueError, TypeError, KeyError, IndexError) as error:
        errors.append(str(error))
    log_text = (output / 'launch.log').read_text() if (output / 'launch.log').is_file() else ''
    passed = code == 0 and not errors and (output / 'capture.png').is_file() and 'SCRIPT ERROR' not in log_text and 'ERROR:' not in log_text
    result = {'started': started, 'finished': utc(), 'exit_code': code, 'validation_errors': errors,
              'capture_contract_pass': passed, 'pose_oracle_checked': False,
              'no_arrived_holder': 'no arrived holder of requested added-capture kind' in log_text,
              'preregistration_sha256': sha256(output / 'preregistration.json'),
              'artifacts': {name: sha256(output / name) for name in ['capture.json', 'capture.png', 'launch.log'] if (output / name).is_file()}}
    (output / 'launch.json').write_text(json.dumps(result, indent=2) + '\n')
    return 0 if passed else 1


def main(arguments: list[str]) -> int:
    if len(arguments) != 6:
        print('Usage: run_extra_activity_p12.py CHECKOUT NEWDIR SEED DAY HOUR CENTER_KIND', file=sys.stderr)
        return 2
    try:
        return run(arguments)
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        print(f'Extra capture refused: {error}', file=sys.stderr)
        return 2


if __name__ == '__main__':
    raise SystemExit(main(sys.argv[1:]))
