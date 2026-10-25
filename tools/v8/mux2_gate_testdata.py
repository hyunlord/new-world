#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: imported by test_mux2_gate_receipts.py; synthetic fixtures only.
"""Faithful synthetic shape of current M-UX2 frame attribution, never GPU evidence."""
import copy

METRICS = {'buildings': ['process', 'draw', 'rebuild', 'huts', 'floor_prepare', 'door_prepare'],
           'ground': ['process', 'chunk_bakes', 'compose', 'upload', 'strips', 'space_read',
                      'apply_space', 'wall_prepare', 'prepare_grounds', 'trees', 'visible', 'doors', 'wall_warmup'],
           'alerts': ['process', 'poll', 'rebuild'],
           'steward_hud': ['process', 'steward', 'facts', 'learning', 'layout']}


def stall_fixture():
    samples = []
    for i in range(1, 12401):
        ticks = int(i % 10 == 0)
        telemetry = {'frame': i, 'minimap_process_usec': 0,
                     'sim_reads_usec': {}, 'sim_reads_calls': {},
                     'performance': {key: 0 for key in ('process_seconds', 'physics_seconds',
                        'memory_static_bytes', 'objects', 'nodes', 'orphans', 'draw_calls')},
                     'frame_phases': {'status': 'MEASURED', 'post_to_process_usec': 3000,
                                     'process_to_pre_draw_usec': 1000, 'pre_to_post_draw_usec': 21000},
                     'freeze': {'status': 'MEASURED', 'frame': i, 'usec': {}},
                     'native': {'status': 'MEASURED', 'frame': i, 'epoch': 1,
                                'frame_ticks': ticks, 'executed_ticks': i // 10,
                                'totals_usec': {'process': 100, 'tick': ticks * 50, 'prune': 0,
                                                'process_residual': 100 - ticks * 50, 'tick_residual': 0},
                                'systems_usec': {'FixtureSystem': ticks * 50}}}
        for label, metrics in METRICS.items():
            telemetry[label] = {'status': 'MEASURED', 'frame': i,
                                'usec': {metric: 0 for metric in metrics}}
        telemetry['buildings'].update(counts={'sprite_creates': 0, 'sprite_frees': 0,
                                              'texture_requests': 0}, texture_ids=[])
        if i == 1:
            telemetry['ground']['usec']['wall_warmup'] = 100
        samples.append({'elapsed_seconds': i / 40, 'interval_ms': 25.0,
                        'post_draw_usec': 1000000 + i * 25000, 'tick': i // 10,
                        'ticks_advanced': ticks, 'sampling_overhead_usec': 5, 'telemetry': telemetry})
    score = {'complete': True, 'pass': True, 'warmup_seconds': 10,
             'required_measured_seconds': 300, 'actual_play_seconds': 310,
             'actual_post_warmup_seconds': 300, 'post_warmup_over50': 0,
             'post_warmup_over33': 0, 'over33_per_minute': [0] * 5, 'all_over33_frames': []}
    instrumentation = {'complete': True, 'invalid': {}, 'wall_warmup_observed_usec': 100,
                       **{key: 'MEASURED' for key in ('native', 'freeze', 'frame_phases',
                                                     'minimap_process', 'sim_reads_raw_getters')},
                       **{key: 'INSTRUMENTED' for key in METRICS}}
    warmup = {'complete': True, 'reason': 'complete', 'candidates': 1, 'unique_appearances': 1,
              'elapsed_usec': 100, 'max_candidates': 4096, 'max_appearances': 128,
              'budget_usec': 5000000, 'current_unique_appearances': 1, 'cache_misses': 1}
    arm = {'zoom': 1.0, 'samples': samples, 'complete': True, 'capture_complete': True,
           'start_tick': 0, 'end_tick': 1240, 'score': score, 'wall_warmup': warmup,
           'instrumentation': instrumentation, 'final_steward': {'enabled': True}}
    other = copy.deepcopy(arm)
    other['zoom'] = 0.5
    return {'purpose': 'acceptance_5_minutes_each_zoom', 'complete': True, 'pass': True,
            'bar_pass': True, 'profile_complete': True, 'errors': [], 'seed': 0,
            'speed': 1, 'steward_enabled': True, 'arms': [arm, other],
            'environment': {'debug': True, 'gpu': 'test GPU', 'render_driver': 'metal'},
            'bridge': {'descriptor_loaded': True}}
