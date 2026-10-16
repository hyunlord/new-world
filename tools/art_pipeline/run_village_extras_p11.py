#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: P11_ACTIVITY_BUILD_PROFILE=debug P11_DEADLINE_EPOCH=... python3 run_village_extras_p11.py CHECKOUT NEWDIR FROZEN_C_ANSWERS
"""Capture every preregistered supplemental Z1 instant after c answers freeze."""
from __future__ import annotations

import json
import os
import subprocess
import sys
from dataclasses import asdict
from pathlib import Path

import run_activity_captures_p11 as base


def fixed_cases(winner: int) -> list[base.Capture]:
    return [base.Capture(seed, day, hour, 1) for hour in (11, 14)
            for seed, day in ((20261010, 30), (winner, 60))]


def capture_environment(checkout_output: Path, case: base.Capture, profile: str) -> dict[str, str]:
    env = os.environ.copy()
    env.pop('P11_ACTIVITY_CENTER_KIND', None)
    env.update(P11_ACTIVITY_OUT=str(checkout_output), P11_ACTIVITY_BUILD_PROFILE=profile,
               P11_ACTIVITY_SEED=str(case.seed), P11_ACTIVITY_DAY=str(case.day),
               P11_ACTIVITY_HOUR=str(case.hour), P11_ACTIVITY_ZOOM='1')
    return env


def run(arguments: list[str]) -> int:
    checkout, output, answers = (Path(value).resolve() for value in arguments)
    if not answers.is_file() or not answers.read_bytes().strip():
        raise base.InvalidInput('requires nonempty frozen c answers before preregistration')
    answer_sha = base.sha256(answers)  # Do not interpret answers or access a key.
    selection = checkout / 'docs/art/production11/activity/selection.json'
    winner = base.integer(base.record(base.record(json.loads(selection.read_text()))['winner'])['seed'])
    cases = fixed_cases(winner)
    deadline = float(os.environ.get('P11_DEADLINE_EPOCH', '0'))
    base.remaining(deadline)
    seconds = int(os.environ.get('P11_CAPTURE_LIMIT', '1800'))
    godot = Path(os.environ.get('GODOT_BIN', '/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot'))
    if not 1 <= seconds <= 3600 or not os.access(godot, os.X_OK):
        raise base.InvalidInput('requires executable Godot and capture limit1..3600')
    identity = base.bridge_identity(checkout, os.environ.get('P11_ACTIVITY_BUILD_PROFILE', 'release'))
    digest = base.source_digest(checkout)
    output.mkdir(parents=False, exist_ok=False)
    (output / 'selection.json').write_bytes(selection.read_bytes())
    for filename, command in [('working-tree.patch', ['git', 'diff', 'HEAD', '--']),
                              ('git-status.txt', ['git', 'status', '--short'])]:
        (output / filename).write_bytes(subprocess.check_output(command, cwd=checkout))
    registration = {'preregistered': base.utc(), 'checkout': str(checkout),
        'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=checkout, text=True).strip(),
        'cases': [asdict(case) for case in cases], 'order': 'hour11 then14; day30 then60 within hour',
        'camera': 'standard Z1 overview; center_kind absent/-1',
        'selection_rule': 'capture all four cases regardless of summary counts; no crops in runner',
        'source_digest': digest, 'build_identity': asdict(identity),
        'script_sha256': base.sha256(checkout / base.SCRIPT), 'wrapper_sha256': base.sha256(Path(__file__)),
        'runner_path': str(Path(base.__file__).resolve()), 'runner_sha256': base.sha256(Path(base.__file__)),
        'selection_sha256': base.sha256(selection), 'c_answers_path': str(answers),
        'c_answers_sha256': answer_sha, 'deadline_epoch': deadline, 'limit_seconds': seconds,
        'quiet_lock': str(base.QUIET), 'evidence_extension': True, 'acceptance_not_assessed': True}
    prereg = output / 'preregistration.json'
    prereg.write_text(json.dumps(registration, indent=2) + '\n')
    results = []
    for case in cases:
        directory = output / case.name
        directory.mkdir()
        errors: list[str] = []
        code = 2
        started = base.utc()
        try:
            base.wait_for_quiet(deadline, directory / 'quiet-waits.log', case.name)
            if base.source_digest(checkout) != digest or base.sha256(answers) != answer_sha:
                raise base.InvalidInput('source or frozen c answers changed since preregistration')
            with (directory / 'launch.log').open('w') as log:
                code = base.bounded_launch(base.launch_command(checkout, godot, seconds), checkout,
                    capture_environment(directory, case, identity.build_profile), log, deadline, seconds)
            payload = json.loads((directory / 'capture.json').read_text())
            base.validate_capture(payload, case)
            if payload.get('center_kind') != -1 or payload.get('draw_order_complete') is not True or payload.get('draw_order_schema') != 2:
                raise base.InvalidInput('standard overview/order contract mismatch')
            if base.source_digest(checkout) != digest or base.sha256(answers) != answer_sha:
                raise base.InvalidInput('source or frozen c answers changed during capture')
        except (OSError, ValueError, TypeError, KeyError, IndexError) as error:
            errors.append(str(error))
        log_text = (directory / 'launch.log').read_text() if (directory / 'launch.log').is_file() else ''
        passed = code == 0 and not errors and (directory / 'capture.png').is_file() and 'SCRIPT ERROR' not in log_text and 'ERROR:' not in log_text
        result = {'case': asdict(case), 'started': started, 'finished': base.utc(), 'exit_code': code,
            'capture_contract_pass': passed, 'validation_errors': errors,
            'preregistration_sha256': base.sha256(prereg), 'pose_oracle_checked': False,
            'artifacts': {name: base.sha256(directory / name) for name in ('capture.json', 'capture.png', 'launch.log') if (directory / name).is_file()}}
        (directory / 'launch.json').write_text(json.dumps(result, indent=2) + '\n')
        results.append(result)
    (output / 'results.json').write_text(json.dumps({'cases': results, 'acceptance_not_assessed': True}, indent=2) + '\n')
    return 0 if all(row['capture_contract_pass'] for row in results) else 1


def main(arguments: list[str]) -> int:
    if len(arguments) != 3:
        print('Usage: run_village_extras_p11.py CHECKOUT NEWDIR FROZEN_C_ANSWERS', file=sys.stderr)
        return 2
    try:
        return run(arguments)
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        print(f'Village extension refused: {error}', file=sys.stderr)
        return 2


if __name__ == '__main__':
    raise SystemExit(main(sys.argv[1:]))
