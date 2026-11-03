"""Pure collector boundaries with synthetic observations; no native qualification."""
import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from generic_gate import GateContractError
from generic_process import file_hash
from mux2_gate_inputs import Json, Node
from test_generic_observations import receipt

import generic_whole_gate_pack as pack
import generic_whole_gate_renderer as renderer


class SiblingTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name).resolve()
        self.product = self.root / 'product'
        self.product.mkdir()
        self.output = self.root / 'output'
        self.output.mkdir()
        self.source = Path(os.environ.get('WORLDSIM_TEST_SOURCE_ROOT', str(Path(__file__).resolve().parents[2])))

    def source_files(self, pins: dict[str, str]) -> Node:
        for name, sha in pins.items():
            path = self.product / name
            path.parent.mkdir(parents=True, exist_ok=True)
            _ = path.write_bytes((self.source / name).read_bytes())
            self.assertEqual(file_hash(path), sha)
        return Node({'output': str(self.output), 'arms': {'base': {
            'root': str(self.product), 'source_inputs': dict(pins)}}})

    def stage(self, step: int, environment: dict[str, Json]) -> Node:
        files = {str(p): file_hash(p) for p in self.output.rglob('*') if p.is_file()}
        modes = {str(p): p.stat().st_mode for p in (self.output, *self.output.rglob('*'))}
        return Node({'step': step, 'arm': 'base', 'environment': environment,
                     'observer_artifacts': files, 'observer_modes': modes})

    def test_renderer_actual_paths_horizons_and_mirror_identity(self) -> None:
        session = self.source_files({**renderer.PINS, 'rust/crates/sim-test/src/probe/ascii.rs': renderer.ASCII_SHA})
        log = self.output / 'base/renderer_agreement'
        mirrors = self.output / 'native-worlds'
        mirrors.mkdir()
        for index, (local, ticks) in enumerate(((log / 'probe', 1440), (log, 900))):
            folder = local / 'gate_observations'
            folder.mkdir(parents=True)
            sidecar = folder / 'colony_s100_primary.json'
            observed = receipt()
            observed.update(seed=100, occurrence='primary', requested_ticks=ticks,
                            requested_terminal_tick=ticks, terminal_tick=ticks)
            _ = sidecar.write_text(json.dumps(observed))
            _ = (mirrors / f'probe_pid{100 + index}_world0.json').write_text(json.dumps({
                'schema': 'worldsim_gate_test_observation_v1', 'process_id': 100 + index,
                'world_sequence': 0, 'local_output': str(local), 'local_sidecar': str(sidecar),
                'observation': observed}))
        for name in ('probe.log', 'ascii.log', 'godot.log', 'text_people.json', 'godot_people.json',
                     'commands/colony_s100_commands.json', 'ascii_t900.txt'):
            path = log / name
            path.parent.mkdir(parents=True, exist_ok=True)
            _ = path.write_text('{}')
        stage = self.stage(23, {'WORLDSIM_GATE_OBSERVER': '1', 'WORLDSIM_GATE_TEST_OUT': str(mirrors)})
        oracle = 'RENDERER_AGREEMENT PASS n=10 tiles=10 tick=900 appearance=10 agree\n'
        with patch.object(renderer, 'aggregate_output', return_value=(oracle, 90)):
            result = renderer.collect(session, stage)
            self.assertEqual(tuple(w.terminal_tick for w in result.worlds), (1440, 900))
            self.assertEqual(result.emitted_pids, (100, 101))
            _ = (mirrors / 'probe_pid101_world0.json').write_text('{}')
            with self.assertRaises(GateContractError):
                _ = renderer.collect(session, stage)

    def test_pack_requires_complete_nine_child_envelope(self) -> None:
        session = self.source_files({pack.SOURCE: pack.SOURCE_SHA,
            'tools/v8/data_only_ext.sh': pack.SCRIPT36_SHA,
            'tools/v8/godot_bin.sh': pack.GODOT_ENV_SHA})
        log = self.output / 'base/data_only_ext'
        log.mkdir(parents=True)
        child = ('\nrunning 1 test\ntest selected_pack_child ... ok\n\n'
                 'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.01s\n\n')
        header = '     Running tests/data_only_ext.rs (synthetic)\n'
        outer = ('\nrunning 2 tests\ntest selected_pack_child ... ok\ntest ' + pack.PARENT + ' ... ' + child * 9 +
                 'ok\n\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s\n\n')
        path = log / 'simulation.log'
        _ = path.write_text(header + outer)
        oracle = 'DATA_ONLY_EXT PASS: seven-family simulation and selected-scene UI; rendered capture is separate'
        with patch.object(pack, 'aggregate_output', return_value=(oracle, 90)):
            result = pack.collect(session, self.stage(36, {}))
            self.assertEqual(len(Node(result).field('nested_ranges').array()), 9)
            self.assertFalse(result['independent_listing'])
            _ = path.write_text(header + outer.replace(child, '', 1))
            with self.assertRaises(GateContractError):
                _ = pack.collect(session, self.stage(36, {}))

    def test_why_requires_both_original_steps_and_smoke_failure_stays_failure(self) -> None:
        from generic_whole_gate_families import evaluate
        session = Node({'output': str(self.output)})
        stages = {('base', 46): Node({}), ('candidate', 46): Node({})}
        with patch('generic_whole_gate_ui_regressions.evaluate_family', return_value=()):
            result = evaluate(session, stages)
            self.assertNotIn('why_toast_layout', result.completed)
            stages['base', 40] = Node({})
            stages['candidate', 40] = Node({})
            with patch('generic_whole_gate_ui_why.evaluate_family', return_value=()):
                result = evaluate(session, stages)
                self.assertIn('why_toast_layout', result.completed)
        index = self.output / 'auxiliary-index/smoke.json'
        index.parent.mkdir()
        _ = index.write_text(json.dumps({'root': str(self.output / 'auxiliaries/smoke'), 'code': 1}))
        result = evaluate(session, {})
        self.assertTrue(result.errors)
        self.assertNotIn('ui.smoke', result.completed)

    def test_oracle_suite_exact_logs_and_no_world_classification(self) -> None:
        from mux2_gate_oracle_log import EXPECTED_COUNTS, PREFIXES

        import generic_whole_gate_oracles as oracles
        pins = {**oracles.NO_WORLD_PINS,
                **{'tools/v8/' + name: sha for name, sha in oracles.PROTOCOL_PINS.items()}}
        session = self.source_files(pins)
        log = self.output / 'base/mux2-oracles'
        log.mkdir(parents=True)
        stdout: list[str] = []
        for name in oracles.SUITE_NAMES:
            filename = ('stall-self-test' if name == 'mux2_stall_test' else name) + '.log'
            if name == 'mux2_stall_test':
                raw = 'MUX2_STALL_SELF_TEST failures=[]\n'
            elif name == 'mux2_wall_cache_test':
                labels = ('base', 'cache_hit', 'material_alias', 'changed_tint', 'changed_top', 'reloaded_images', 'after_invalidation')
                raw = ''.join(f'MUX2_WALL_CACHE_HASHES {label} synthetic\n' for label in labels) + 'MUX2_WALL_CACHE_DONE failures=[]\n'
                stdout.append(f'MUX2_ORACLE_PASS {name} checks=7')
            else:
                count, prefix = EXPECTED_COUNTS[name], PREFIXES[name]
                suffix = 'RESULT' if name == 'mux2_badge_extent_test' else 'DONE'
                raw = (prefix + '_PASS synthetic\n') * count + f'{prefix}_{suffix} checks={count} failures=0\n'
                stdout.append(f'MUX2_ORACLE_PASS {name} checks={count}')
            _ = (log / filename).write_text(raw)
        native, ui = self.output / 'native', self.output / 'ui'
        native.mkdir()
        ui.mkdir()
        environment: dict[str, Json] = {'WORLDSIM_GATE_TEST_OUT': str(native), 'WORLDSIM_GATE_UI_OUT': str(ui)}
        with patch.object(oracles, 'aggregate_output', return_value=('\n'.join(stdout), 90)):
            result = oracles.collect(session, self.stage(38, environment))
            self.assertEqual(len(Node(result).field('coverage').array()), 16)
            _ = (native / 'unexpected-world.json').write_text('{}')
            with self.assertRaises(GateContractError):
                _ = oracles.collect(session, self.stage(38, environment))
