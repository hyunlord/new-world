# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: imported by measure_p8.py and test_measurement_p8.py
"""Evaluate new CPU, destination geometry and paired-world requirements."""
from __future__ import annotations

import json
import math
from statistics import median
from typing import NotRequired, TypedDict

from ingame_qa_pixels import Check, result


class CpuSample(TypedDict):
    process_ms: float
    render_cpu_ms: float
    setup_cpu_ms: float
    gpu_ms: float


class CpuBlock(TypedDict):
    layers_enabled: bool
    samples: list[CpuSample]
    tick: int
    camera: list[float]


class PerformanceEvidence(TypedDict, total=False):
    valid: bool
    errors: list[str]
    normalization_errors: list[str]
    blocks: list[CpuBlock]
    alive_count: int
    first_three_alive_tick: int
    seed: int
    framing: str
    simulation_frozen: bool
    vsync_disabled: bool
    vsync_restored: bool
    run_id: str
    uptime: str
    processes: str
    process_priorities: list[int]
    old_pacing_ms: list[float]


class World(TypedDict):
    pressure: int
    threshold: int
    departure_tick: int | None


class PairedEvidence(TypedDict, total=False):
    P: World
    G: World
    same_seed_and_cast_days: bool
    both_reached_day120: bool


class WayFrame(TypedDict):
    frame: str
    walking: bool
    site_on_screen: bool
    stake_visible: bool
    arrows: list[list[float]]
    screen_center: list[float]
    target_screen: list[float]
    distance_tiles: float
    expected_distance_tiles: float
    metadata_errors: NotRequired[list[str]]


def performance(evidence: PerformanceEvidence) -> Check:
    """Require the exact controlled 12-block experiment before comparing medians."""
    errors: list[str] = []
    if evidence.get('valid') is False:
        errors.append('raw measurement explicitly invalid')
        errors.extend('raw: ' + reason for reason in evidence.get('errors', []))
    errors.extend('normalization: ' + reason for reason in evidence.get('normalization_errors', []))
    blocks = evidence.get('blocks', [])
    if len(blocks) != 12:
        errors.append('requires twelve alternating 50-frame blocks')
    for field in ('simulation_frozen', 'vsync_disabled', 'vsync_restored'):
        if evidence.get(field) is not True:
            errors.append(field + ' absent/false')
    if evidence.get('alive_count', 0) < 3 or evidence.get('framing') != 'FV':
        errors.append('requires first >=3 alive FV frame')
    if not evidence.get('run_id') or not evidence.get('uptime') or 'processes' not in evidence:
        errors.append('same-process identity or host load record absent')
    if evidence.get('process_priorities') != [-1000, 1000]:
        errors.append('process timing brackets absent')
    samples: dict[bool, list[float]] = {True: [], False: []}
    with_setup: dict[bool, list[float]] = {True: [], False: []}
    previous: bool | None = None
    camera = blocks[0]['camera'] if blocks else []
    tick = evidence.get('first_three_alive_tick', -1)
    for index, block in enumerate(blocks):
        enabled = block['layers_enabled']
        if enabled == previous:
            errors.append(f'block {index}: not alternating')
        previous = enabled
        if block['tick'] != tick or block['camera'] != camera:
            errors.append(f'block {index}: tick/camera changed')
        if len(block['samples']) != 50:
            errors.append(f'block {index}: expected 50 counted frames')
        for sample in block['samples']:
            values = [sample[key] for key in ('process_ms', 'render_cpu_ms', 'setup_cpu_ms', 'gpu_ms')]
            if not all(math.isfinite(value) and value >= 0 for value in values):
                errors.append(f'block {index}: invalid CPU/GPU number')
                continue
            cpu = sample['process_ms'] + sample['render_cpu_ms']
            samples[enabled].append(cpu)
            with_setup[enabled].append(cpu + sample['setup_cpu_ms'])
    if any(len(values) != 300 for values in samples.values()):
        errors.append('requires 300 valid frames in each layer state')
    on, off = (median(samples[state]) if samples[state] else 0 for state in (True, False))
    if off <= 0 or not (on <= 1.15 * off or on <= off + 1.0):
        errors.append('CPU median exceeds both thresholds or off baseline is zero')
    details = {'on_median_ms': on, 'off_median_ms': off,
               'ratio': on / off if off > 0 else None, 'delta_ms': on - off,
               'samples': [len(samples[state]) for state in (True, False)],
               'including_setup_diagnostic_ms': [median(with_setup[state]) if with_setup[state] else None for state in (True, False)],
               'formula': 'median(process_ms + render_cpu_ms); setup CPU separate',
               'old_pacing_ms': evidence.get('old_pacing_ms', []),
               'gpu_status': 'measured' if any(sample['gpu_ms'] > 0 for block in blocks for sample in block['samples']) else 'unavailable',
               'errors': errors}
    return result('A30', not errors, json.dumps(details))


def paired_worlds(evidence: PairedEvidence) -> Check:
    """Require higher P/theta plus strictly earlier spoil-world departure."""
    p, g = evidence.get('P'), evidence.get('G')
    if p is None or g is None:
        return Check('A33', 'N/A', 'Missing paired worlds')
    valid = evidence.get('same_seed_and_cast_days') and evidence.get('both_reached_day120')
    ratios = [world['pressure'] / world['threshold'] if world['threshold'] > 0 else None for world in (p, g)]
    pressure_valid = ratios[0] is not None and ratios[1] is not None and ratios[0] > ratios[1]
    early, late = p['departure_tick'], g['departure_tick']
    departure_valid = early is not None and (late is None or early < late)
    return result('A33', bool(valid and pressure_valid and departure_valid), json.dumps({
        'meter_ratios': ratios, 'departure_ticks': [early, late],
        'pressure_valid': pressure_valid, 'departure_valid': departure_valid,
        'same_seed_and_cast_days': evidence.get('same_seed_and_cast_days'),
        'both_reached_day120': evidence.get('both_reached_day120')}))


def way(frames: list[WayFrame]) -> Check:
    """Independently compare centre-target and centre-arrow vectors."""
    errors: list[str] = []
    rows = []
    for frame in frames:
        if not frame['walking']:
            continue
        arrows = frame['arrows']
        angle = None
        valid = frame['stake_visible'] and not arrows
        if not frame['site_on_screen']:
            valid = not frame['stake_visible'] and len(arrows) == 1
            if len(arrows) == 1:
                cx, cy = frame['screen_center']
                ax, ay = arrows[0][0] - cx, arrows[0][1] - cy
                tx, ty = frame['target_screen'][0] - cx, frame['target_screen'][1] - cy
                length = math.hypot(ax, ay) * math.hypot(tx, ty)
                angle = math.degrees(math.acos(max(-1.0, min(1.0, (ax * tx + ay * ty) / length)))) if length else 180.0
                valid = valid and angle <= 10 and abs(frame['distance_tiles'] - frame['expected_distance_tiles']) <= 1
        valid = valid and not frame.get('metadata_errors', [])
        if not valid:
            errors.append(frame['frame'])
        rows.append({'frame': frame['frame'], 'angle_degrees': angle, 'valid': valid, 'metadata_errors': frame.get('metadata_errors', [])})
    return result('A32_GEOMETRY', bool(rows) and not errors, json.dumps({'frames': rows, 'errors': errors,
                  'blind': 'Separate SHA-bound two-observer direction assessment required for A32'}))
