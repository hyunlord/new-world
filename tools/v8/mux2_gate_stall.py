#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: imported by mux2_gate_receipts.py.
"""Recompute the unchanged M-UX2 timing bars from every retained frame."""
import math
from mux2_gate_inputs import Json, Node, require
from mux2_gate_telemetry import REQUIRED, Frame, validate_frame, validate_warmup


def validate_stall(raw: Json) -> None:
    report = Node(raw)
    report.field('purpose').equals('acceptance_5_minutes_each_zoom')
    for key in ('complete', 'pass', 'bar_pass', 'profile_complete', 'steward_enabled'):
        report.field(key).equals(True)
    report.field('errors').equals([])
    require(report.field('seed').number() == 0 and report.field('speed').number() == 1, 'wrong world/speed')
    environment = report.field('environment')
    environment.field('debug').equals(True)
    require(bool(environment.field('gpu').text()), 'GPU unavailable')
    require(environment.field('render_driver').text().lower() not in ('', 'dummy', 'headless'),
            'dummy renderer is not acceptance')
    report.field('bridge').field('descriptor_loaded').equals(True)
    arms = report.field('arms').array()
    require(len(arms) == 2 and sorted(a.field('zoom').number() for a in arms) == [0.5, 1.0],
            'require exactly zoom1 and zoom0.5')
    for arm in arms:
        validate_arm(arm)


def validate_arm(arm: Node) -> None:
    for key in ('complete', 'capture_complete'):
        arm.field(key).equals(True)
    instrumentation = arm.field('instrumentation')
    instrumentation.field('complete').equals(True)
    instrumentation.field('invalid').equals({})
    for key in ('native', 'freeze', 'frame_phases'):
        instrumentation.field(key).equals('MEASURED')
    for label in REQUIRED:
        instrumentation.field(label).equals('INSTRUMENTED')
    for label in ('minimap_process', 'sim_reads_raw_getters'):
        instrumentation.field(label).equals('MEASURED')
    require(arm.field('start_tick').number() == 0, 'arm must start at tick zero')
    arm.field('final_steward').field('enabled').equals(True)
    require(arm.field('end_tick').number() > arm.field('start_tick').number(), 'no simulation progress')
    score = arm.field('score')
    for key in ('complete', 'pass'):
        score.field(key).equals(True)
    require(score.field('warmup_seconds').number() == 10
            and score.field('required_measured_seconds').number() == 300, 'changed timing thresholds')
    samples = arm.field('samples').array()
    require(bool(samples), 'missing raw samples')
    previous = 0.0
    counts = [0] * 5
    over50 = 0
    slow: list[Json] = []
    prior_frame: Frame | None = None
    warmup_observed = 0.0
    for row in samples:
        elapsed = row.field('elapsed_seconds').number()
        interval = row.field('interval_ms').number()
        require(elapsed > previous and interval > 0, 'invalid frame ordering or interval')
        require(abs((elapsed - previous) * 1000 - interval) <= 0.02,
                'interval does not match recorded elapsed time')
        previous = elapsed
        prior_frame = validate_frame(row, prior_frame, 0)
        warmup_observed += prior_frame.wall_warmup
        if interval > 33:
            slow.append(row.value)
        if elapsed <= 10:
            continue
        over50 += int(interval > 50)
        if interval > 33:
            counts[min(4, max(0, math.ceil((elapsed - 10) / 60) - 1))] += 1
    require(prior_frame is not None and prior_frame.tick == arm.field('end_tick').number(),
            'end tick differs from final sample')
    validate_warmup(arm, warmup_observed)
    require(previous >= 310 and score.field('actual_play_seconds').number() == previous,
            'incomplete duration or mismatched elapsed time')
    require(score.field('actual_post_warmup_seconds').number() == previous - 10,
            'incorrect measured duration')
    require(over50 == 0 and all(n <= 3 for n in counts), 'stall acceptance bar failed')
    require(score.field('post_warmup_over50').number() == over50
            and score.field('post_warmup_over33').number() == sum(counts), 'incorrect summary counts')
    score.field('over33_per_minute').equals(counts)
    score.field('all_over33_frames').equals(slow)
