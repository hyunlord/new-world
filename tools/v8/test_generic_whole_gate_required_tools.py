"""Pure filesystem and preflight tests; fake executables are never executed."""
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from generic_gate import GateContractError, document
from generic_process import file_hash
from generic_whole_gate_contract import load_session
from generic_whole_gate_required_tools import (
    REQUIRED,
    check_required_path_tools,
    required_path_tools,
)
from generic_whole_gate_source import prepare_session
from mux2_gate_inputs import Json, Node


def executable(root: Path, name: str) -> Path:
    path = root / name
    _ = path.write_text('#!/bin/sh\nexit 0\n')
    path.chmod(0o700)
    return path


def selector(root: Path) -> Path:
    tools = {name: executable(root, name) for name in ('python3', 'bash', 'cargo', 'rustc')}
    arm: dict[str, Json] = {'root': str(root), 'source_sha': '0' * 40, 'cargo_home': str(root),
                            'configs': [], 'environment': [], 'overlay': None}
    config: dict[str, Json] = {'schema': 'whole_gate.v1', 'selectors': {
        'harness_root': str(root), 'harness_sha': '0' * 40, 'published_sha': '0' * 40,
        'purpose': 'self_proof', 'base': arm, 'candidate': dict(arm),
        'cargo': {'name': 'cargo', 'path': str(tools['cargo']), 'sha256': file_hash(tools['cargo'])},
        'rustc': {'name': 'rustc', 'path': str(tools['rustc']), 'sha256': file_hash(tools['rustc'])}},
        'output': str(root / 'output'), 'layers': [],
        'python': {'path': str(tools['python3']), 'sha256': file_hash(tools['python3'])},
        'bash': {'path': str(tools['bash']), 'sha256': file_hash(tools['bash'])},
        'environment': {'PATH': str(root), 'GODOT_BIN': str(tools['cargo'])},
        'external_inputs': {str(tools['cargo']): file_hash(tools['cargo'])},
        'acceptance': {}, 'observations': {}}
    path = root / 'selector.json'
    _ = path.write_text(json.dumps(config))
    return path


class RequiredToolsPreflightTests(unittest.TestCase):
    def test_missing_rustdoc_rejected_before_source_preparation(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            config = selector(root)
            with patch('generic_cli_prepare.prepare_cli', side_effect=RuntimeError('preparation reached')) as prepare:
                with self.assertRaisesRegex(GateContractError, 'rustdoc.*missing required PATH tool'):
                    _ = prepare_session(config)
                prepare.assert_not_called()
            self.assertFalse((root / 'output').exists())


    def test_relative_or_empty_path_rejected_before_any_resolution(self) -> None:
        for suffix in ('relative', '', '.'):
            with tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary).resolve()
                config = selector(root)
                node = document(config.read_text()).mapping()
                environment = dict(Node(node['environment']).mapping())
                environment['PATH'] = str(root) + ':' + suffix
                node['environment'] = environment
                _ = config.write_text(json.dumps(node))
                with patch('shutil.which', side_effect=RuntimeError('resolution reached')) as resolve:
                    with self.subTest(suffix=suffix), self.assertRaisesRegex(GateContractError, 'PATH components'):
                        _ = prepare_session(config)
                    resolve.assert_not_called()
                self.assertFalse((root / 'output').exists())


class RequiredToolBindingTests(unittest.TestCase):
    def test_valid_bound_executables_and_aliases(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            binary = executable(root, 'canonical')
            for name in REQUIRED:
                (root / name).symlink_to(binary)
            result = required_path_tools(str(root), {str(binary): file_hash(binary)})
            self.assertEqual(result, {name: str(binary) for name in REQUIRED})

    def test_each_tool_missing_unbound_or_nonexecutable_rejected(self) -> None:
        for name in REQUIRED:
            for change in ('missing', 'unbound', 'nonexecutable', 'changed'):
                with tempfile.TemporaryDirectory() as temporary:
                    root = Path(temporary).resolve()
                    paths = {tool: executable(root, tool) for tool in REQUIRED}
                    external = {str(path): file_hash(path) for path in paths.values()}
                    path = paths[name]
                    if change == 'missing':
                        path.unlink()
                    elif change == 'unbound':
                        del external[str(path)]
                    elif change == 'nonexecutable':
                        path.chmod(0o600)
                    else:
                        _ = path.write_text('#!/bin/sh\nexit 1\n')
                    with self.subTest(name=name, change=change), self.assertRaises(GateContractError):
                        _ = required_path_tools(str(root), external)

    def test_retarget_to_even_another_bound_executable_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            first = executable(root, 'first')
            second = executable(root, 'second')
            for name in REQUIRED:
                (root / name).symlink_to(first)
            external = {str(first): file_hash(first), str(second): file_hash(second)}
            selected = required_path_tools(str(root), external)
            value: Json = {'schema': 'whole_gate_session.v1', 'materialization': {},
                           'environment': {'PATH': str(root)},
                           'external_inputs': {name: digest for name, digest in external.items()},
                           'required_path_tools': selected}
            session = Node(value)
            check_required_path_tools(session)
            (root / 'rustdoc').unlink()
            (root / 'rustdoc').symlink_to(second)
            with self.assertRaises(ValueError):
                check_required_path_tools(session)
            stored = root / 'session.json'
            _ = stored.write_text(json.dumps(value))
            with self.assertRaises(ValueError):
                _ = load_session(stored, file_hash(stored))

    def test_alias_hash_alone_does_not_bind_canonical_target(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            binary = executable(root, 'canonical')
            for name in REQUIRED:
                (root / name).symlink_to(binary)
            external = {str(root / name): file_hash(binary) for name in REQUIRED}
            with self.assertRaisesRegex(GateContractError, 'canonical external input'):
                _ = required_path_tools(str(root), external)


if __name__ == '__main__':
    _ = unittest.main()
