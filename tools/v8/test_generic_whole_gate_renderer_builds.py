"""Pure receipt boundary tests; fixture tools are never executed."""
import json
import tempfile
import unittest
from pathlib import Path

from generic_process import file_hash
from generic_whole_gate_renderer_builds import authenticate_builds, selected_godot
from mux2_gate_inputs import Node


class RendererBuildTests(unittest.TestCase):
    def test_selected_tool_boundary(self) -> None:
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder).resolve()
            tool = root/'tool'
            _ = tool.write_text('test fixture, not a Godot executable')
            tool.chmod(0o755)
            selector = root/'selector.json'
            _ = selector.write_text(json.dumps({'schema': 'whole_gate.v1', 'selectors': {
                'base': {}, 'candidate': {}, 'cargo': {}, 'rustc': {},
                'harness_root': '/fixture', 'harness_sha': 'fixture',
                'published_sha': 'fixture', 'purpose': 'identity'},
                'environment': {'GODOT_BIN': str(tool)},
                'external_inputs': {str(tool): file_hash(tool)}}))
            session = Node({'selector_path': str(selector), 'selector_sha256': file_hash(selector),
                            'external_inputs': {str(tool): file_hash(tool)}, 'environment': {'GODOT_BIN': str(tool)}})
            self.assertEqual(selected_godot(session), (tool, file_hash(tool)))
            environment = session.field('environment').mapping()
            original = environment.pop('GODOT_BIN')
            with self.assertRaises(ValueError):
                _ = selected_godot(session)
            environment['GODOT_BIN'] = '/different-session-godot'
            with self.assertRaises(ValueError):
                _ = selected_godot(session)
            environment['GODOT_BIN'] = original
            external = session.field('external_inputs').mapping()
            digest = external.pop(str(tool))
            with self.assertRaises(ValueError):
                _ = selected_godot(session)
            external[str(tool)] = digest
            tool.chmod(0o644)
            with self.assertRaises(ValueError):
                _ = selected_godot(session)
            tool.chmod(0o755)
            _ = tool.write_text('changed')
            with self.assertRaises(ValueError):
                _ = selected_godot(session)

    def test_missing_selector_fails(self) -> None:
        with self.assertRaises(ValueError):
            _ = selected_godot(Node({}))

    def test_missing_build_chain_fails(self) -> None:
        with self.assertRaises(ValueError):
            _ = authenticate_builds(Node({}), Node({}), 'original_off', Node({}))

    def test_unknown_mode_fails(self) -> None:
        with self.assertRaises(ValueError):
            _ = authenticate_builds(Node({}), Node({}), 'historical_pass', Node({}))


if __name__ == '__main__':
    _ = unittest.main()
