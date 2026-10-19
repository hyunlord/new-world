"""Standalone ablation must retain source identity and literal control coverage."""
from copy import deepcopy
import unittest
import json
import tempfile
from pathlib import Path

from cpu_probe_p13 import KERNEL_ID

from summarize_ablation_p13 import validate
from summarize_perf_p13 import EvidenceError, METRICS, validate_provenance, load_session


def fixture():
    nodes = ['WorldSim', 'SimReads', 'Camera2D', 'UI']
    roles = {'WorldSim': 'simulation_disabled_control', 'SimReads': 'synchronous_reader_process_control', 'Camera2D': 'already_frozen_camera_control'}
    rows = []
    labels = ['ablation_all_on_0']
    previous = labels[0]
    for node in nodes:
        off, after = 'off_' + node, 'on_after_' + node
        rows.append({'node': node, 'role': roles.get(node, 'presentation'),
                     'removable_cost': node not in roles, 'layer_only': False,
                     'descendants_disabled_as_group': [], 'off_block': off,
                     'neighbour_blocks': [previous, after]})
        labels.extend([off, after])
        previous = after
    labels.extend(['ablation_floor', 'ablation_restored'])
    frame = 0
    blocks = []
    for label in labels:
        disabled = label in {'off_WorldSim', 'ablation_floor'}
        samples = []
        tick = 1440
        for _ in range(180):
            frame += 1
            tick += int(not disabled)
            sample = {metric: 0.0 for metric in METRICS}
            sample.update(primary_cpu_ms=2.0, process_ms=1.0, render_cpu_ms=1.0,
                          ticks_run=int(not disabled), paired=True, process_frame=frame,
                          state={'tick': tick, 'camera': [0, 0], 'zoom': [4, 4],
                                 'simulation_frozen': disabled})
            samples.append(sample)
        blocks.append({'label': label, 'condition': 'ablation_off' if label.startswith('off_') else 'ablation_floor' if label == 'ablation_floor' else 'ablation_reference',
                       'zoom': 4, 'speed': 2, 'max_fps': 60, 'vsync_disabled': True,
                       'camera_process_frozen': True, 'window_size': [1920, 1080],
                       'fresh_setup': True, 'start_state': {'tick': 1440},
                       'simulation_disabled': disabled,
                       'diagnostic_control': 'simulation_disabled' if disabled else 'running',
                       'discarded': samples[:30], 'samples': samples[30:]})
    data = {'world': 'W200', 'phase': 'baseline', 'ablation': rows,
            'ablation_top_level_inventory': nodes,
            'ablation_floor_role': 'simulation_disabled_all_targets_off',
            'initial': {'tick': 1440, 'camera': [0, 0]},
            'restoration': {key: True for key in ('vsync_restored', 'layers_restored', 'camera_process_restored', 'probes_removed')},
            'ablation_visibility_restored': True}
    return data, blocks


class AblationIntegrity(unittest.TestCase):
    def test_controls_and_running_restoration(self):
        data, blocks = fixture()
        validate(data, blocks)

    def test_missing_top_level_node_rejected(self):
        data, blocks = fixture()
        data['ablation_top_level_inventory'].append('MissingRenderer')
        with self.assertRaisesRegex(EvidenceError, 'coverage'):
            validate(data, blocks)

    def test_simulator_off_cannot_claim_ticks(self):
        data, blocks = fixture()
        block = next(row for row in blocks if row['label'] == 'off_WorldSim')
        block['samples'][0]['ticks_run'] = 1
        with self.assertRaises(EvidenceError):
            validate(data, blocks)

    def test_restored_control_cannot_stay_paused(self):
        data, blocks = fixture()
        for row in blocks[-1]['samples']:
            row['ticks_run'] = 0
            row['state']['tick'] = 1470
        with self.assertRaises(EvidenceError):
            validate(data, blocks)

    def test_simreads_not_removable_cache_cost(self):
        data, blocks = fixture()
        data['ablation'][1]['removable_cost'] = True
        with self.assertRaisesRegex(EvidenceError, 'role'):
            validate(data, blocks)

    def test_ablation_only_cannot_enter_full_matrix(self):
        # Given valid admission and an ablation-only payload; matrix rejects its shape.
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'session.json'
            path.write_text(json.dumps({'world': 'W200', 'phase': 'baseline', 'ablation_only': True}))
            sample = {'wall_seconds': 2.0, 'load_1m': 1.0}
            Path(str(path) + '.validity.json').write_text(json.dumps({
                'kernel_id': KERNEL_ID, 'reference_seconds': 2.0,
                'start': sample, 'end': sample, 'exit_code': 0}))
            with self.assertRaisesRegex(EvidenceError, 'ablation-only'):
                load_session(path)

    def test_provenance_rejects_missing_and_mismatched_fields(self):
        baseline = {'phase': 'baseline', 'build_commit': 'a' * 40,
                    'baseline_commit': 'a' * 40, 'build_sha256': 'b' * 64,
                    'collector_sha256': 'c' * 64, 'checkout': '/tmp/measure',
                    'build_profile': 'release', 'release_sha256': 'b' * 64}
        validate_provenance(baseline)
        for key in baseline:
            data = deepcopy(baseline)
            del data[key]
            with self.subTest(key=key), self.assertRaises(EvidenceError):
                validate_provenance(data)
        data = dict(baseline, release_sha256='d' * 64)
        with self.assertRaises(EvidenceError):
            validate_provenance(data)


if __name__ == '__main__':
    unittest.main()
