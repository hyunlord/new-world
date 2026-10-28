"""UI successor rejects changed runtime, incomplete history and hidden deltas."""
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import core_ui_successor as ui
import core_ui_successor_history as history
import core_perf_check as perf


class UISuccessorTests(unittest.TestCase):
    def test_git_tree_binds_symlink_blob_and_mode_without_following(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            ui.inventory.git(root, 'init', '-q')
            (root / 'scripts').mkdir()
            script = root / 'scripts/tool.gd'
            script.write_text('same content')
            (root / '.agents').mkdir()
            link = root / '.agents/skills'
            link.symlink_to('/missing-target-never-followed')
            ui.inventory.git(root, 'add', 'scripts/tool.gd', '.agents/skills')
            tree = ui.inventory.git(root, 'write-tree').decode().strip()
            before = ui.committed_tree(root, tree)
            self.assertTrue(before['.agents/skills'].startswith('120000:'))
            self.assertEqual(ui.exact_delta(before, ui.committed_tree(root, tree)), {})
            ui.inventory.git(root, 'update-index', '--chmod=+x', 'scripts/tool.gd')
            mode_tree = ui.inventory.git(root, 'write-tree').decode().strip()
            after = ui.committed_tree(root, mode_tree)
            self.assertEqual(before['scripts/tool.gd'].split(':')[1],
                             after['scripts/tool.gd'].split(':')[1])
            self.assertNotEqual(before['scripts/tool.gd'], after['scripts/tool.gd'])
            self.assertEqual(set(ui.exact_delta(before, after)), {'scripts/tool.gd'})
            link.unlink()
            link.symlink_to('/different-missing-target')
            ui.inventory.git(root, 'add', '.agents/skills')
            changed_tree = ui.inventory.git(root, 'write-tree').decode().strip()
            with self.assertRaises(ui.freeze.FreezeError):
                ui.exact_delta(after, ui.committed_tree(root, changed_tree))

    def test_only_exact_historical_config_can_map_repository(self) -> None:
        original = perf.Document.parse({'repo': '/historical', 'source': 'fixed'})
        mapped = history.mapped_config(original, original, Path('/owned'))
        self.assertEqual(mapped.values, {'repo': '/owned', 'source': 'fixed'})
        self.assertEqual(original.values['repo'], '/historical')
        with self.assertRaises(ui.freeze.FreezeError):
            history.mapped_config(perf.Document.parse({'repo': '/historical', 'source': 'changed'}),
                                  original, Path('/owned'))

    def test_committed_addition_and_deletion_are_both_declared(self) -> None:
        self.assertEqual(ui.exact_delta({'scripts/removed.gd': 'old'}, {'assets/ui/new.json': 'new'}),
                         {'scripts/removed.gd': {'before_entry': 'old', 'after_entry': None},
                          'assets/ui/new.json': {'before_entry': None, 'after_entry': 'new'}})

    def test_historical_validation_dependencies_are_frozen(self) -> None:
        for path in ('tools/v8/core_perf_check.py', 'tools/v8/core_eval_check.py',
                     'tools/v8/core_shadow_freeze.py', 'tools/v8/core_story_check.py',
                     'tools/v8/bench_ledger.py'):
            for after in ({path: 'changed'}, {}):
                with self.assertRaises(ui.freeze.FreezeError):
                    ui.exact_delta({path: 'original'}, after)
        self.assertEqual(len(ui.exact_delta({'tools/v8/gate.sh': 'old'},
                                           {'tools/v8/gate.sh': 'new'})), 1)

    def test_root_runtime_inputs_are_in_live_proof(self) -> None:
        self.assertIn('project.godot', ui.LIVE_SCOPES)
        self.assertIn('sim_bridge.gdextension', ui.LIVE_SCOPES)
        for path in ('project.godot', 'sim_bridge.gdextension'):
            with self.assertRaises(ui.freeze.FreezeError):
                ui.exact_delta({path: 'original'}, {path: 'changed'})

    def test_dirty_root_runtime_input_fails_live_validation(self) -> None:
        for filename in ('project.godot', 'sim_bridge.gdextension'):
            with tempfile.TemporaryDirectory() as directory:
                root = Path(directory).resolve()
                path = root / filename
                path.write_text('committed')
                expected = {filename: ui.freeze.digest(path)}
                def names(_root: Path, *args: str) -> bytes:
                    return filename.encode() + b'\0' if args[-1] == filename else b''
                def committed(_root: Path, _head: str, scope: str) -> dict[str, str]:
                    return expected if scope == filename else {}
                with patch.object(ui.inventory, 'git', side_effect=names), \
                     patch.object(ui.inventory, 'committed', side_effect=committed):
                    ui.verify_live(root, 'head')
                    path.write_text('dirty runtime configuration')
                    with self.assertRaises(ui.freeze.FreezeError):
                        ui.verify_live(root, 'head')

    def test_exact_delta_and_no_rust_mutation(self) -> None:
        before = {'scripts/a.gd': 'a', 'rust/a.rs': 'r'}
        after = {'scripts/a.gd': 'b', 'rust/a.rs': 'r'}
        expected = {'scripts/a.gd': {'before_entry': 'a', 'after_entry': 'b'}}
        self.assertEqual(ui.exact_delta(before, after), expected)
        with self.assertRaises(ui.freeze.FreezeError):
            ui.exact_delta(before, {**after, 'rust/a.rs': 'changed'})
        with self.assertRaises(ui.freeze.FreezeError):
            ui.exact_delta(before, {**after, '.cargo/config': 'changed'})

    def test_gate_requires_every_unique_step_and_terminal_success(self) -> None:
        gate = 'head ' + ui.ACCEPTED_HEAD + '\n' + ''.join(
            f'{i:2} test exit 0 1s started 00:00:00\n' for i in range(1, 46))
        gate += 'finished today status 0\n'
        terminal = {'status': 'EXITED', 'actual_exit_code': 0, 'wrapper_exit_code': 0}
        ui.accepted_gate(gate, terminal)
        for broken in (gate.replace('45 test', '44 test'), gate.replace('45 test exit 0', '45 test exit 1'), gate.replace('status 0', 'status 1')):
            with self.assertRaises(ui.freeze.FreezeError):
                ui.accepted_gate(broken, terminal)
        with self.assertRaises(ui.freeze.FreezeError):
            ui.accepted_gate(gate, {**terminal, 'actual_exit_code': 143})

    def test_historical_failures_are_not_fresh_passes(self) -> None:
        original = {'status': 'FAIL', 'failures': ['n200_side64/tick_mean_us>1.1']}
        ui.same_historical_result(original, original, 1)
        with self.assertRaises(ui.freeze.FreezeError):
            ui.same_historical_result(original, {'status': 'PASS', 'failures': []}, 0)
        with self.assertRaises(ui.freeze.FreezeError):
            ui.same_historical_result(original, original, 0)

    def test_missing_or_changed_reference_fails(self) -> None:
        with tempfile.TemporaryDirectory() as name:
            path = Path(name).resolve() / 'evidence'
            path.write_text('original')
            ref = ui.reference(path)
            self.assertEqual(ui.read_ref(Path('/not-this-root'), ref), path)
            path.write_text('changed')
            with self.assertRaises(ui.freeze.FreezeError):
                ui.read_ref(Path('/not-this-root'), ref)
