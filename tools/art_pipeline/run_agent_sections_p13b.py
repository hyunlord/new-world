#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: P13_DEADLINE_EPOCH=<epoch> python3 tools/art_pipeline/run_agent_sections_p13.py CHECKOUT NEW_OUTPUT_DIR
"""Run one bounded, quiet-locked diagnostic W500 section session."""
from __future__ import annotations

import hashlib
from session_validity_p13 import require_valid, valid_snapshot

import json
import math
import statistics
import os
import signal
import subprocess
import sys
import time
from pathlib import Path

from generate_agent_sections_p13 import NAMES, generate
from run_alternating_p13 import provenance
from summarize_perf_p13 import EvidenceError, Json

Record = dict[str, Json]


def sha(path: Path) -> str:
    """Hash an actual source/build artifact."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_capture(payload: Record) -> None:
    """Fail closed on incomplete frames, invalid timers, or mismatched state."""
    if payload.get('valid') is not True or payload.get('errors') != []:
        raise EvidenceError('Capture did not report valid=true with no errors')
    world = payload.get('world')
    running = world in ('W200', 'W-multi')
    if world not in ('W500', 'W200', 'W-multi'):
        raise EvidenceError('Unknown section world')
    seed, tick, population = (19603, 90840, 0) if world == 'W-multi' else (15100, 1440, 200 if running else 500)
    expected = {'world': world, 'seed': seed, 'requested_tick': tick,
                'requested_population': population, 'zoom': 4, 'window_size': [1920, 1080],
                'warmup_frames': 30, 'counted_frames': 300, 'diagnostic_only': True,
                'instrumented': True, 'founding_groups_status': 'unavailable_separate_callback'}
    for key, value in expected.items():
        if payload.get(key) != value:
            raise EvidenceError(f'Invalid session field: {key}')
    initial = payload.get('initial')
    if not isinstance(initial, dict) or initial.get('tick') != tick or (not running and initial != payload.get('final')):
        raise EvidenceError('Missing or changed frozen state')
    if (population > 0 and initial.get('population') != population) or initial.get('zoom') != [4, 4] or initial.get('simulation_frozen') is not False:
        raise EvidenceError('Population, zoom or world callback mode differs from matrix')
    tile = payload.get('framing_tile')
    if not isinstance(tile, list) or len(tile) != 2 or any(not isinstance(value, (int, float)) or not math.isfinite(value) for value in tile):
        raise EvidenceError('Missing finite framing tile')
    if initial.get('camera') != [456 + 16 * tile[0], 36 + 16 * tile[1]]:
        raise EvidenceError('Camera differs from settlement framing')
    installed = payload.get('installation')
    if not isinstance(installed, dict) or installed.get('valid') is not True:
        raise EvidenceError('Diagnostic installation was not validated')
    for key in ('source_sha256', 'output_sha256', 'clock_sha256'):
        digest = installed.get(key)
        if not isinstance(digest, str) or len(digest) != 64 or any(char not in '0123456789abcdef' for char in digest):
            raise EvidenceError(f'Missing installation digest: {key}')
    conditions = {'inspector_visible': False, 'influence_channel': -1, 'layers_enabled': True,
                  'renderer_callback_enabled': True, 'world_callback_enabled': True, 'speed_setting': 2.0 if running else 0.0}
    if payload.get('conditions') != conditions or payload.get('speed_setting') != (2.0 if running else 0.0):
        raise EvidenceError('Initial UI/layer/speed conditions differ from matrix')
    if running and not any(row.get('ticks_run', 0) > 0 for row in payload.get('samples', [])):
        raise EvidenceError('Running sections have no ticks')
    prior = -1
    for group, count in (('discarded', 30), ('samples', 300)):
        rows = payload.get(group)
        if not isinstance(rows, list) or len(rows) != count:
            raise EvidenceError(f'Expected {count} {group} rows')
        for row in rows:
            if not isinstance(row, dict) or row.get('paired') is not True or (not running and row.get('state') != initial):
                raise EvidenceError('Invalid process row')
            if row.get('conditions') != conditions:
                raise EvidenceError('UI/layer/callback conditions changed')
            for metric in ('process_ms', 'render_cpu_ms', 'setup_cpu_ms', 'primary_cpu_ms'):
                value = row.get(metric)
                if not isinstance(value, (int, float)) or isinstance(value, bool) or not math.isfinite(value) or value < 0:
                    raise EvidenceError(f'Invalid frame metric: {metric}')
            frame = row.get('process_frame')
            if not isinstance(frame, int) or frame <= prior:
                raise EvidenceError('Nonmonotonic process frames')
            prior = frame
            sections = row.get('agent_sections')
            if not isinstance(sections, dict) or sections.get('valid') is not True or sections.get('complete') is not True or sections.get('process_frame') != frame:
                raise EvidenceError('Invalid, incomplete or stale section callback')
            values = sections.get('exclusive_usec')
            if not isinstance(values, dict) or set(values) != {name + '_usec' for name in NAMES}:
                raise EvidenceError('Missing or extra section fields')
            if any(not isinstance(value, int) or isinstance(value, bool) or value < 0 for value in values.values()):
                raise EvidenceError('Negative or noninteger timer')
            if sum(values.values()) != sections.get('total_usec') or sections.get('remainder_usec') != 0:
                raise EvidenceError('Section total does not match callback')
            if sections.get('founding_groups_usec') is not None or sections.get('founding_groups_status') != 'unavailable_separate_callback':
                raise EvidenceError('Child callback incorrectly attributed to parent')
    restored = payload.get('restoration')
    if not isinstance(restored, dict) or any(restored.get(key) is not True for key in ('vsync_restored', 'layers_restored', 'camera_process_restored', 'probes_removed')) or payload.get('max_fps_restored') is not True:
        raise EvidenceError('Missing restoration evidence')


def main() -> int:
    """Preserve one attempt and its terminal exit; never retry implicitly."""
    if len(sys.argv) != 3:
        raise EvidenceError('Usage: run_agent_sections_p13.py CHECKOUT NEW_OUTPUT_DIR')
    checkout, output = (Path(value).resolve() for value in sys.argv[1:])
    deadline = int(os.environ.get('P13_DEADLINE_EPOCH', '0'))
    if deadline <= time.time():
        raise EvidenceError('Set a future P13_DEADLINE_EPOCH from the original 12-hour task deadline')
    root = Path(__file__).resolve().parents[2]
    source = checkout / 'scripts/ui/agent_renderer.gd'
    helper = checkout / 'scripts/test/art/agent_section_clock_p13.gd'
    harness = checkout / 'scripts/test/art/profile_agent_sections_p13.gd'
    loader = checkout / 'scripts/test/art/section_profile_loader_p13.gd'
    for required in (source, helper, harness, loader):
        if not required.is_file():
            raise EvidenceError(f'Missing required diagnostic file: {required}')
    generated = generate(source.read_text())
    before = provenance(checkout)
    output.mkdir(parents=True, exist_ok=False)
    script = output / 'agent_renderer_profile_p13.gd'
    script.write_text(generated)
    manifest: Record = {'schema_version': 1, 'source': str(source), 'output': str(script),
                        'source_sha256': sha(source), 'output_sha256': sha(script),
                        'clock_sha256': sha(helper), 'generator_sha256': sha(root / 'tools/art_pipeline/generate_agent_sections_p13.py'),
                        'ordinary_statements_preserved': True,
                        'exclusive_sections': [name + '_usec' for name in NAMES],
                        'founding_groups_status': 'unavailable_separate_callback'}
    manifest_path = output / 'agent_renderer_profile_p13.manifest.json'
    manifest_path.write_text(json.dumps(manifest, indent=2) + '\n')
    environment = dict(os.environ, P13_VALIDITY_OUT=str(output / "frames.json.validity.json"), P13_SECTIONS_SCRIPT=str(script), P13_SECTIONS_MANIFEST=str(manifest_path),
                       P13_SECTIONS_OUT=str(output / 'frames.json'), P13_SECTIONS_HEAD=str(before['head']),
                       P13_SECTIONS_BUILD_SHA256=str(before['build_sha256']))
    command = ['bash', str(root / 'tools/art_pipeline/with_quiet_lock_p13b.sh'), 'agent-sections-' + environment.get('P13_SECTIONS_WORLD', 'W500'), '3600',
               environment.get('GODOT_BIN', '/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot'),
               '--path', str(checkout), '--resolution', '1920x1080', '--script', 'res://scripts/test/art/profile_agent_sections_p13.gd']
    launch: Record = {'before': before, 'deadline_epoch': deadline, 'command': command,
                      'manifest': manifest, 'started_epoch': time.time()}
    (output / 'launch.json').write_text(json.dumps(launch, indent=2) + '\n')
    remaining = deadline - time.time()
    if remaining <= 0:
        raise EvidenceError('Deadline elapsed during preparation; no session launched')
    with (output / 'session.log').open('x') as stream:
        process = subprocess.Popen(command, env=environment, stdout=stream, stderr=subprocess.STDOUT, start_new_session=True)
        try:
            # Admission closes at the task deadline; finish one bounded hold.
            code = process.wait(timeout=remaining + 3600 + 10)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGTERM)
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait()
            code = 124
    (output / 'session.exit').write_text(str(code) + '\n')
    after = provenance(checkout)
    launch.update({'after': after, 'finished_epoch': time.time(), 'exit_code': code})
    (output / 'launch.json').write_text(json.dumps(launch, indent=2) + '\n')
    errors: list[str] = []
    section_statistics = {}
    if code != 0:
        errors.append(f'Session exited {code}')
    if before != after or manifest['source_sha256'] != sha(source) or manifest['output_sha256'] != sha(script) or manifest['clock_sha256'] != sha(helper):
        errors.append('Sources/build/generated artifacts changed during session')
    if 'ERROR' in (output / 'session.log').read_text():
        errors.append('Session log contains ERROR')
    contended = False
    try:
        validity = json.loads((output / 'frames.json.validity.json').read_text())
        if not isinstance(validity, dict):
            raise EvidenceError('Session validity evidence must be an object')
        contended = not all(isinstance(validity.get(key), dict) and valid_snapshot(validity[key], validity.get("reference_seconds")) for key in ('start', 'end'))
        require_valid(output / 'frames.json.validity.json')
    except ValueError as exc:
        errors.append(str(exc))
    except OSError as exc:
        errors.append('Missing session validity evidence: ' + str(exc))
    try:
        payload = json.loads((output / 'frames.json').read_text())
        if not isinstance(payload, dict):
            raise EvidenceError('Capture JSON must be an object')
        validate_capture(payload)
        for key in (*[name + '_usec' for name in NAMES], 'total_usec'):
            values = sorted(row['agent_sections']['total_usec'] if key == 'total_usec' else row['agent_sections']['exclusive_usec'][key] for row in payload['samples'])
            section_statistics[key] = {'median': statistics.median(values), 'p90': values[math.ceil(len(values) * 0.90) - 1], 'p99': values[math.ceil(len(values) * 0.99) - 1], 'max': values[-1]}
        if payload.get('build_commit') != before['head'] or payload.get('build_sha256') != before['build_sha256']:
            raise EvidenceError('Capture build provenance differs from launch')
        installed = payload.get('installation', {})
        if not isinstance(installed, dict):
            raise EvidenceError('Capture installation metadata must be an object')
        for key in ('source_sha256', 'output_sha256', 'clock_sha256'):
            if installed.get(key) != manifest[key]:
                raise EvidenceError(f'Capture installation differs: {key}')
    except (OSError, json.JSONDecodeError, EvidenceError) as exc:
        errors.append(str(exc))
    result: Record = {'status': 'NOT_VERIFIED (contended)' if contended else ('VALID' if not errors else 'NOT_VERIFIED'), 'valid': not errors, 'errors': errors, 'exit_code': code,
                      'diagnostic_only': True, 'section_statistics_usec': section_statistics,
                      'percentile_method': 'nearest-rank', 'founding_groups_status': 'unavailable_separate_callback'}
    (output / 'summary.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))
    return 0 if not errors else 1


if __name__ == '__main__':
    raise SystemExit(main())
