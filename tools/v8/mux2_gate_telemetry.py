#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: imported by mux2_gate_stall.py.
"""Independently check every current producer timing field and the 3D rebuild attribution.

M3D-SWITCH E2: the 2D ground and building timings (and the wall warm-up) are replaced by World3D's per-layer process
time (`world3d`: World3D itself and each layer, from process-order probes in mux2_perf_monitor.gd).
"""
from dataclasses import dataclass
from typing import Final
from mux2_gate_inputs import Node, require

REQUIRED: Final = {'world3d': ('world3d', 'terrain', 'flora', 'sky', 'structures', 'props', 'plants', 'people',
                              'animals', 'visitors', 'overlay', 'labels', 'camera_rig'),
    'alerts': ('process', 'poll', 'rebuild'),
    'steward_hud': ('process', 'steward', 'facts', 'learning', 'layout')}


def nonnegative(node: Node, *, integer: bool = False) -> float:
    value = node.number()
    require(value >= 0 and (not integer or type(node.value) is int), f'{node.label}: invalid counter')
    return value


def counters(node: Node, keys: tuple[str, ...] = (), *, integer: bool = False) -> None:
    values = node.mapping()
    require(set(keys) <= values.keys(), f'{node.label}: missing attribution fields')
    for key in values:
        nonnegative(node.field(key), integer=integer)


def measured(node: Node, frame: float) -> None:
    node.field('status').equals('MEASURED')
    require(nonnegative(node.field('frame'), integer=True) == frame, f'{node.label}: stale frame')


@dataclass(frozen=True, slots=True)
class Frame:
    frame: float
    post_draw: float
    tick: float
    epoch: float
    executed_ticks: float
    rebuild: float


def validate_frame(row: Node, previous: Frame | None, start_tick: float) -> Frame:
    telemetry = row.field('telemetry')
    frame = nonnegative(telemetry.field('frame'), integer=True)
    post = nonnegative(row.field('post_draw_usec'), integer=True)
    tick = nonnegative(row.field('tick'), integer=True)
    advanced = nonnegative(row.field('ticks_advanced'), integer=True)
    before_tick = start_tick if previous is None else previous.tick
    require(advanced <= 8 and tick - before_tick == advanced, 'raw tick delta mismatch')
    nonnegative(row.field('sampling_overhead_usec'), integer=True)
    if previous is not None:
        require(frame > previous.frame, 'non-increasing telemetry frame')
        require(abs((post - previous.post_draw) / 1000 - row.field('interval_ms').number()) <= 0.02,
                'post-draw timestamp does not match interval')
    nonnegative(telemetry.field('minimap_process_usec'), integer=True)
    for key in ('sim_reads_usec', 'sim_reads_calls'):
        counters(telemetry.field(key), integer=True)
    counters(telemetry.field('performance'), ('process_seconds', 'physics_seconds',
             'memory_static_bytes', 'objects', 'nodes', 'orphans', 'draw_calls'))
    for label, keys in REQUIRED.items():
        node = telemetry.field(label)
        measured(node, frame)
        counters(node.field('usec'), keys, integer=True)
    freeze = telemetry.field('freeze')
    measured(freeze, frame)
    counters(freeze.field('usec'), integer=True)
    phase = telemetry.field('frame_phases')
    phase.field('status').equals('MEASURED')
    values = [nonnegative(phase.field(key), integer=True) for key in
              ('post_to_process_usec', 'process_to_pre_draw_usec', 'pre_to_post_draw_usec')]
    require(abs(sum(values) / 1000 - row.field('interval_ms').number()) <= 0.02,
            'frame phases do not cover the measured interval')
    native = telemetry.field('native')
    measured(native, frame)
    epoch = nonnegative(native.field('epoch'), integer=True)
    executed = nonnegative(native.field('executed_ticks'), integer=True)
    require(nonnegative(native.field('frame_ticks'), integer=True) == advanced,
            'native and world tick delta disagree')
    require(executed == tick - start_tick, 'native executed ticks disagree with world')
    if previous is not None:
        require(epoch == previous.epoch and executed - previous.executed_ticks == advanced,
                'native epoch/counter changed')
    counters(native.field('totals_usec'), ('process', 'tick', 'prune', 'process_residual', 'tick_residual'))
    require(bool(native.field('systems_usec').mapping()), 'missing native system attribution')
    counters(native.field('systems_usec'))
    layers = telemetry.field('world3d').field('usec')
    rebuild = sum(nonnegative(layers.field(key), integer=True) for key in layers.mapping())
    return Frame(frame, post, tick, epoch, executed, rebuild)


def validate_rebuild(arm: Node, observed: float) -> None:
    rebuild = arm.field('rebuild')
    # The world drawn by the end of the arm: every terrain chunk built (the 3D counterpart of a finished warm-up).
    rebuild.field('terrain_complete').equals(True)
    require(rebuild.field('observed_usec').number() == observed
            and arm.field('instrumentation').field('rebuild_observed_usec').number() == observed,
            '3D layer time attribution summary mismatch')
