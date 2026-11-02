"""Offline setup contract tests; no Cargo or Godot execution."""
import json
import subprocess
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from typing import final

from generic_gate import GateContractError
from generic_process import file_hash
from generic_setup_prepare import (
    COMMANDS,
    IMPORT,
    PHASES,
    TOOLS,
    WORKSPACE_LINES,
    SetupInputs,
    TargetCheckpoint,
    Tool,
    import_exit,
    prepare,
    recipe,
    validate_transition_shape,
)


class SetupTests(unittest.TestCase):
    def test_original_second_exit_does_not_hide_first_failure(self) -> None:
        self.assertEqual(import_exit(1, 0), 0)
        self.assertEqual(import_exit(0, 1), 1)
        self.assertIsNone(import_exit(None, 0))
        self.assertIsNone(import_exit(0, None))


@final
class Fixture:
    def __init__(self, home: Path) -> None:
        self.home = home
        self.root = home/'product'
        self.root.mkdir()
        self.bin = home/'bin'
        self.bin.mkdir()
        self.cargo_home = home/'cargo-home'
        self.cargo_home.mkdir()
        files = {
            '.gitignore': 'rust/Cargo.lock\nrust/target/\n.godot/\n',
            'tools/v8/gate.sh': '\n'.join(f"step {n} {name} bash -c '{cmd}'" for n, name, cmd in COMMANDS[:2])+'\n'+'\n'.join(WORKSPACE_LINES)+'\n'+
                f"step 6 clippy bash -c '{COMMANDS[2][2]}'\n",
            'tools/v8/run_bounded.sh': '# synthetic bound source\n',
            'rust/rust-toolchain.toml': '[toolchain]\nchannel="1.93"\n',
            'rust/crates/sim-bridge/Cargo.toml': '[package]\nname="sim-bridge"\n[lib]\ncrate-type=["cdylib","rlib"]\n',
            'sim_bridge.gdextension': '[configuration]\nentry_symbol = "gdext_rust_init"\ncompatibility_minimum = 4.1\nreloadable = true\n\n[libraries]\nlinux.debug.x86_64 = "res://rust/target/debug/libsim_bridge.so"\nlinux.release.x86_64 = "res://rust/target/release/libsim_bridge.so"\nlinux.debug.arm64 = "res://rust/target/debug/libsim_bridge.so"\nlinux.release.arm64 = "res://rust/target/release/libsim_bridge.so"\nmacos.debug = "res://rust/target/debug/libsim_bridge.dylib"\nmacos.release = "res://rust/target/release/libsim_bridge.dylib"\nwindows.debug.x86_64 = "res://rust/target/debug/sim_bridge.dll"\nwindows.release.x86_64 = "res://rust/target/release/sim_bridge.dll"\n',
            'project.godot': '[application]\n',
            'rust/.cargo/config.toml': '[net]\noffline=true\n',
        }
        for name, value in files.items():
            p = self.root/name
            p.parent.mkdir(parents=True, exist_ok=True)
            _ = p.write_text(value)
        _ = self.git('init', '-q')
        _ = self.git('add', '.')
        _ = self.git('-c', 'user.name=Test', '-c', 'user.email=t@example.invalid', 'commit', '-qm', 'synthetic source')
        self.sha = self.git('rev-parse', 'HEAD').strip()
        _ = (self.root/'rust/Cargo.lock').write_text('# bound ignored synthetic lock\n')
        self.origin = home/'lock-origin.json'
        _ = self.origin.write_text(json.dumps({'path':str(self.root/'rust/Cargo.lock'), 'sha256':file_hash(self.root/'rust/Cargo.lock')}))
        tools: list[Tool] = []
        for name in sorted(TOOLS):
            p = self.bin/name
            _ = p.write_text('#!/bin/sh\nexit 97\n')
            p.chmod(0o755)
            tools.append(Tool(name, p, file_hash(p)))
        env = {'HOME':str(home), 'PATH':str(self.bin), 'LANG':'C', 'LC_ALL':'C',
               'CARGO_HOME':str(self.cargo_home), 'CARGO_BUILD_JOBS':'8', 'GODOT_BIN':str(self.bin/'godot'),
               'WORLDSIM_TEST_SETTINGS_DIR':str(home/'settings'), 'PYTHONNOUSERSITE':'1', 'PYTHONDONTWRITEBYTECODE':'1'}
        effective = tuple(sorted((name, file_hash(self.root/name)) for name in [*files, 'rust/Cargo.lock']))
        self.inputs = SetupInputs(self.root, self.sha, self.cargo_home, tuple(tools), tuple(sorted(env.items())), effective,
                                  ((str(self.root/'rust/.cargo/config.toml'),file_hash(self.root/'rust/.cargo/config.toml')),),
                                  self.origin, file_hash(self.origin), home/'settings', home/'evidence')

    def git(self, *args: str) -> str:
        return subprocess.check_output(('git', '-C', str(self.root), *args), text=True)


class PreparationTests(unittest.TestCase):
    def test_exact_recipes_and_partition_without_producers(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            f = Fixture(Path(temp).resolve())
            prepared = prepare(f.inputs)
            self.assertEqual(prepared.status, 'PREPARED_ONLY')
            self.assertFalse(prepared.execution_authority)
            self.assertIn('rust/.cargo/config.toml', dict(prepared.materialized))
            self.assertNotIn(f.root/'rust/.cargo/config.toml', dict(prepared.external))
            for step in (2, 3, 6):
                result = recipe(prepared, step)
                self.assertEqual(result.argv[:2], (str(f.bin/'bash'), '-c'))
                self.assertEqual(result.cwd, f.root)
            imports = recipe(prepared, 3)
            self.assertEqual(imports.argv[2], IMPORT+'; '+IMPORT)
            self.assertEqual(imports.occurrences, ('import.first', 'import.second'))
            self.assertEqual(imports.timeout_seconds, 600)
            for path in (f.inputs.output, f.inputs.settings, f.root/'rust/target', f.root/'.godot'):
                self.assertFalse(path.exists())

    def test_current_source_recipe_change_rejected_even_after_commit(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            f = Fixture(Path(temp).resolve())
            p = f.root/'tools/v8/gate.sh'
            _ = p.write_text(p.read_text().replace('cargo build -p', 'cargo build --locked -p'))
            _ = f.git('add', '.')
            _ = f.git('-c', 'user.name=Test', '-c', 'user.email=t@example.invalid', 'commit', '-qm', 'changed recipe')
            effective = tuple((name, file_hash(f.root/name)) for name, _ in f.inputs.effective_inputs)
            with self.assertRaisesRegex(GateContractError, 'original command'):
                _ = prepare(replace(f.inputs, source_sha=f.git('rev-parse', 'HEAD').strip(), effective_inputs=effective))

    def test_dirty_hidden_source_and_missing_effective_input(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            f = Fixture(Path(temp).resolve())
            with self.assertRaisesRegex(GateContractError, 'complete unique'):
                _ = prepare(replace(f.inputs, effective_inputs=f.inputs.effective_inputs[1:]))
            _ = f.git('update-index', '--assume-unchanged', 'project.godot')
            _ = (f.root/'project.godot').write_text('changed')
            with self.assertRaisesRegex(GateContractError, 'tracked bytes'):
                _ = prepare(f.inputs)

    def test_generated_state_and_config_injection_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            f = Fixture(Path(temp).resolve())
            (f.root/'rust/target').mkdir()
            with self.assertRaisesRegex(GateContractError, 'expected absent'):
                _ = prepare(f.inputs)
            (f.root/'rust/target').rmdir()
            _ = (f.cargo_home/'config.toml').write_text('[net]\noffline=true\n')
            with self.assertRaisesRegex(GateContractError, 'Cargo config'):
                _ = prepare(f.inputs)

    def test_stale_preparation_and_environment_injection(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            f = Fixture(Path(temp).resolve())
            for pair in (('RUSTFLAGS','-C opt-level=3'), ('RUSTC_WRAPPER','/fake')):
                with self.assertRaises(GateContractError):
                    _ = prepare(replace(f.inputs, environment=(*f.inputs.environment, pair)))
            base = prepare(f.inputs)
            empty = prepare(replace(f.inputs, environment=(*f.inputs.environment, ('RUSTC_WORKSPACE_WRAPPER',''))))
            self.assertNotEqual(base.inputs.environment, empty.inputs.environment)
            _ = (f.root/'rust/Cargo.lock').write_text('late mutation')
            with self.assertRaisesRegex(GateContractError, 'effective input'):
                _ = recipe(base, 2)

    def test_checkpoint_shape_never_supplies_producer_authority(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            before = TargetCheckpoint(Path(temp).resolve(), 'a'*40, 'initial', '0'*64, '1'*64, None)
            for phase in PHASES[1:]:
                after = TargetCheckpoint(before.root, before.source_sha, phase, before.digest(), '2'*64, '3'*64)
                self.assertEqual(validate_transition_shape(before, after), 'PRODUCER_AUTHENTICATION_REQUIRED')
                with self.assertRaises(GateContractError):
                    _ = validate_transition_shape(before, replace(after, predecessor_sha256='9'*64))
                with self.assertRaises(GateContractError):
                    _ = validate_transition_shape(before, replace(after, root=Path('/')))
                before = after
            with self.assertRaises(GateContractError):
                _ = validate_transition_shape(before, replace(before, phase='after2', predecessor_sha256=before.digest()))


    def test_lock_origin_wrong_bridge_and_tool_bytes(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            f = Fixture(Path(temp).resolve())
            with self.assertRaises(GateContractError):
                _ = prepare(replace(f.inputs, lock_origin_sha256='0'*64))
            p = f.root/'sim_bridge.gdextension'
            _ = p.write_text('[libraries]\nmacos.debug = "res://rust/target/release/libsim_bridge.dylib"\n')
            _ = f.git('add', '.')
            _ = f.git('-c', 'user.name=Test', '-c', 'user.email=t@example.invalid', 'commit', '-qm', 'wrong bridge')
            changed = replace(f.inputs, source_sha=f.git('rev-parse', 'HEAD').strip(),
                              effective_inputs=tuple((name,file_hash(f.root/name)) for name,_ in f.inputs.effective_inputs))
            with self.assertRaisesRegex(GateContractError, 'debug bridge'):
                _ = prepare(changed)
        with tempfile.TemporaryDirectory() as temp:
            f = Fixture(Path(temp).resolve())
            _ = (f.bin/'cargo').write_text('#!/bin/sh\nexit 96\n')
            with self.assertRaisesRegex(GateContractError, 'tool changed'):
                _ = prepare(f.inputs)

    def test_path_shadowing_and_duplicate_tools(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            f = Fixture(Path(temp).resolve())
            with self.assertRaisesRegex(GateContractError, 'exact tool set'):
                _ = prepare(replace(f.inputs, tools=(*f.inputs.tools, f.inputs.tools[0])))
            shadow = f.home/'shadow'
            shadow.mkdir()
            p = shadow/'cargo'
            _ = p.write_text('#!/bin/sh\nexit 0\n')
            p.chmod(0o755)
            env = tuple((k, str(shadow)+':'+v if k == 'PATH' else v) for k,v in f.inputs.environment)
            with self.assertRaisesRegex(GateContractError, 'PATH tool mismatch'):
                _ = prepare(replace(f.inputs, environment=env))

    def test_unrelated_committed_source_is_not_pinned_to_old_digest(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            f = Fixture(Path(temp).resolve())
            _ = (f.root/'rust/farming.rs').write_text('// synthetic unrelated source change\n')
            _ = f.git('add', '.')
            _ = f.git('-c', 'user.name=Test', '-c', 'user.email=t@example.invalid', 'commit', '-qm', 'future source')
            changed = replace(f.inputs, source_sha=f.git('rev-parse', 'HEAD').strip(),
                              effective_inputs=(*f.inputs.effective_inputs, ('rust/farming.rs',file_hash(f.root/'rust/farming.rs'))))
            self.assertEqual(prepare(changed).status, 'PREPARED_ONLY')

    def test_clippy_before_workspace_source_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            f = Fixture(Path(temp).resolve())
            p = f.root/'tools/v8/gate.sh'
            lines = p.read_text().splitlines()
            _ = p.write_text('\n'.join([*lines[:2],lines[4],*lines[2:4]])+'\n')
            _ = f.git('add', '.')
            _ = f.git('-c', 'user.name=Test', '-c', 'user.email=t@example.invalid', 'commit', '-qm', 'wrong order')
            changed = replace(f.inputs, source_sha=f.git('rev-parse', 'HEAD').strip(),
                              effective_inputs=tuple((name,file_hash(f.root/name)) for name,_ in f.inputs.effective_inputs))
            with self.assertRaisesRegex(GateContractError, 'source order'):
                _ = prepare(changed)

    def test_duplicate_or_misplaced_extension_mapping_rejected(self) -> None:
        for source in (
            'macos.debug = "res://rust/target/debug/libsim_bridge.dylib"\n',
            '[DEFAULT]\nmacos.debug = "res://rust/target/debug/libsim_bridge.dylib"\n[libraries]\n',
            '[libraries]\nmacos.debug = "res://rust/target/debug/libsim_bridge.dylib"\nmacos.debug = "res://foreign.dylib"\n',
            '[configuration]\nmacos.debug = "res://rust/target/debug/libsim_bridge.dylib"\n[libraries]\nmacos.debug = "res://foreign.dylib"\n',
        ):
            with self.subTest(source=source), tempfile.TemporaryDirectory() as temp:
                f = Fixture(Path(temp).resolve())
                _ = (f.root/'sim_bridge.gdextension').write_text(source)
                _ = f.git('add', '.')
                _ = f.git('-c', 'user.name=Test', '-c', 'user.email=t@example.invalid', 'commit', '-qm', 'ambiguous extension')
                changed = replace(f.inputs, source_sha=f.git('rev-parse', 'HEAD').strip(),
                                  effective_inputs=tuple((name,file_hash(f.root/name)) for name,_ in f.inputs.effective_inputs))
                with self.assertRaises(GateContractError):
                    _ = prepare(changed)

    def test_checkpoint_invalid_identity_and_initial_bridge_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            initial = TargetCheckpoint(Path(temp).resolve(), 'a'*40, 'initial', '0'*64, '1'*64, None)
            for invalid in (replace(initial, root=Path('relative')), replace(initial, source_sha='not-a-sha'),
                            replace(initial, bridge_sha256='2'*64), replace(initial, predecessor_sha256='3'*64)):
                after = replace(invalid, phase='after2', predecessor_sha256=invalid.digest(), bridge_sha256='4'*64)
                with self.subTest(invalid=invalid), self.assertRaises(GateContractError):
                    _ = validate_transition_shape(invalid, after)

    def test_unreviewed_platform_selectors_rejected(self) -> None:
        for selector in ('macos.debug.arm64', 'macos', 'macos.debug.custom', 'linux'):
            with self.subTest(selector=selector), tempfile.TemporaryDirectory() as temp:
                f = Fixture(Path(temp).resolve())
                p = f.root/'sim_bridge.gdextension'
                _ = p.write_text(p.read_text()+f'{selector} = "res://foreign.dylib"\n')
                _ = f.git('add', '.')
                _ = f.git('-c', 'user.name=Test', '-c', 'user.email=t@example.invalid', 'commit', '-qm', 'extra selector')
                changed = replace(f.inputs, source_sha=f.git('rev-parse', 'HEAD').strip(),
                                  effective_inputs=tuple((name,file_hash(f.root/name)) for name,_ in f.inputs.effective_inputs))
                with self.assertRaises(GateContractError):
                    _ = prepare(changed)

    def test_reviewed_library_map_requires_every_exact_path(self) -> None:
        for mutation in ('missing', 'changed'):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as temp:
                f = Fixture(Path(temp).resolve())
                p = f.root/'sim_bridge.gdextension'
                original = p.read_text()
                source = ('\n'.join(line for line in original.splitlines() if not line.startswith('linux.release.arm64'))+'\n'
                          if mutation == 'missing' else original.replace('res://rust/target/release/libsim_bridge.dylib', 'res://foreign.dylib'))
                _ = p.write_text(source)
                _ = f.git('add', '.')
                _ = f.git('-c', 'user.name=Test', '-c', 'user.email=t@example.invalid', 'commit', '-qm', 'changed map')
                changed = replace(f.inputs, source_sha=f.git('rev-parse', 'HEAD').strip(),
                                  effective_inputs=tuple((name,file_hash(f.root/name)) for name,_ in f.inputs.effective_inputs))
                with self.assertRaisesRegex(GateContractError, 'selector map changed'):
                    _ = prepare(changed)
