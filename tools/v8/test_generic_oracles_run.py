"""Offline original-oracle and actual tiny-process ownership tests."""
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

import generic_oracles_run as run


class AuditTests(unittest.TestCase):
    def test_final_original_grep_accepts_repeated_matching_line(self) -> None:
        with TemporaryDirectory() as directory:
            log = Path(directory)/'log'
            _ = log.write_text('MUX2_STALL_SELF_TEST failures=[]\n'*2)
            self.assertEqual(run.audit_log('mux2_stall_test', log), '')

    def test_final_original_grep_rejects_script_error(self) -> None:
        with TemporaryDirectory() as directory:
            log = Path(directory)/'log'
            _ = log.write_text('MUX2_STALL_SELF_TEST failures=[]\nSCRIPT ERROR\n')
            with self.assertRaises(ValueError):
                _ = run.audit_log('mux2_stall_test', log)


# Each synthetic process exercises the original Bash body and original checker.
# Source admission is tested separately; this helper does not certify Godot or builds.
import fcntl
import os
import subprocess
import sys
from collections.abc import Generator
from contextlib import contextmanager
from dataclasses import dataclass
from typing import override
from unittest.mock import patch

import gate_oracle_child as child
import mux2_gate_oracle_log as checker
from generic_fixtures import ProbeProduct
from generic_gate import document
from generic_m6a_structural import Tooling, import_closure
from generic_process import atomic_json, file_hash

ROOT = Path(child.__file__).resolve().parent


@dataclass(frozen=True)
class Synthetic:
    base: Path
    output: Path
    plan: child.SuitePlan
    lane_fd: int
    inputs: run.OracleInputs


@contextmanager
def synthetic(*, fail_native: int = 0, fail_oracle: int = 0, stall_bad: bool = False) -> Generator[Synthetic]:
    with TemporaryDirectory() as folder:
        base = Path(folder).resolve()
        product = base/'product'
        tools = product/'tools/v8'
        tools.mkdir(parents=True)
        output = base/'output'
        output.mkdir()
        evidence, logs = output/'evidence', output/'logs'
        evidence.mkdir()
        logs.mkdir()
        lock = base/'lock'
        lock.touch()
        filenames = ('gate_oracle_child.py', 'gate_stall_child.py', 'generic_gate.py', 'generic_process.py',
            'generic_sources.py', 'mux2_gate_inputs.py', 'mux2_gate_lock.py', 'run_bounded.sh',
            'mux2_gate_checks.sh', 'godot_bin.sh', 'mux2_gate_oracle_log.py')
        for name in filenames:
            value = (ROOT/name).read_text()
            if name == 'mux2_gate_lock.py':
                value = value.replace("Path('/Users/rexxa/work/mux2-data/.mux2-gpu.lock')", 'Path('+repr(str(lock))+')')
            _ = (tools/name).write_text(value)
        for name in child.SUITE_NAMES:
            _ = (tools/(name+'.gd')).write_text('synthetic placeholder: no Godot execution')
        tiny = base/'tiny'
        _ = tiny.write_text('#!'+sys.executable+'\n'
            + 'import os,sys\nfrom pathlib import Path\n'
            + 'assert os.read(0,1)==b""\n'
            + 'assert not any(k.startswith("WORLDSIM_ORACLE_") for k in os.environ)\n'
            + 'assert "WORLDSIM_MUX2_LANE_FD" not in os.environ\n'
            + 'name=Path(sys.argv[sys.argv.index("--script")+1]).stem\n'
            + 'names='+repr(child.SUITE_NAMES)+'\nprefixes='+repr(checker.PREFIXES)+'\ncounts='+repr(checker.EXPECTED_COUNTS)+'\n'
            + 'ordinal=names.index(name)+1\n'
            + 'if ordinal=='+str(fail_native)+':raise SystemExit(7)\n'
            + 'if ordinal=='+str(fail_oracle)+':print("invalid oracle");raise SystemExit(0)\n'
            + 'if ordinal==16:print('+repr('invalid final' if stall_bad else 'MUX2_STALL_SELF_TEST failures=[]')+');raise SystemExit(0)\n'
            + 'prefix=prefixes[name]\n'
            + 'if name=="mux2_wall_cache_test":\n'
            + ' for label in ("base","cache_hit","material_alias","changed_tint","changed_top","reloaded_images","after_invalidation"):print(prefix+"_HASHES "+label+" synthetic")\n'
            + ' print(prefix+"_DONE failures=[]")\n'
            + 'else:\n'
            + ' for index in range(counts[name]):print(prefix+"_PASS synthetic")\n'
            + ' print(prefix+("_RESULT" if name=="mux2_badge_extent_test" else "_DONE")+" checks="+str(counts[name])+" failures=0")\n')
        tiny.chmod(0o755)
        environment = {'PATH': str(Path(sys.executable).parent)+':/usr/bin:/bin', 'LANG': 'C', 'LC_ALL': 'C',
            'PYTHONNOUSERSITE': '1', 'PYTHONDONTWRITEBYTECODE': '1', 'PYTHONCOERCECLOCALE': '0',
            '__CF_USER_TEXT_ENCODING': f'0x{os.getuid():X}:0x3:0x33', 'GODOT_BIN': str(tiny)}
        sources = {str(path): file_hash(path) for path in (*tools.iterdir(), tiny, Path(sys.executable).resolve(), Path('/bin/bash').resolve())}
        planfile = output/'plan.json'
        atomic_json(planfile, {'schema': child.SUITE_SCHEMA, 'root': str(product), 'evidence': str(evidence), 'logs': str(logs),
            'godot': str(tiny), 'inputs': {k: v for k, v in sources.items()}, 'environment': {k: v for k, v in environment.items()}})
        plan = child.load_suite(planfile, file_hash(planfile))
        inputs = run.OracleInputs(ProbeProduct('a'*40, tiny, file_hash(tiny), product, ()),
            Tooling(Path(sys.executable).resolve(), file_hash(Path(sys.executable).resolve()), Path('/bin/bash').resolve(),
                file_hash(Path('/bin/bash').resolve()), ROOT, (), environment['PATH']))
        with lock.open('a') as lane, patch.object(child, 'LOCK', lock):
            fcntl.flock(lane.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
            yield Synthetic(base, output, plan, lane.fileno(), inputs)


def clean_unknown(live: run.LiveInvocation) -> None:
    """Only test-owned handles may be settled/closed; production has no recovery bypass."""
    if live.shell is not None:
        _ = live.shell.wait(timeout=5)
    for descriptor, closed in ((live.channels.request_write, live.request_write_closed),
                               (live.channels.control_read, live.control_read_closed)):
        if not closed:
            os.close(descriptor)
    if live.controller is None:
        if not live.control_write_closed:
            os.close(live.channels.control_write)
        os.close(live.channels.request_read)
    elif not live.controller.closed:
        if live.controller.control_write >= 0:
            os.close(live.controller.control_write)
        os.close(live.controller.request_read)
    live.stdout.close()
    live.stderr.close()


class SuiteTests(unittest.TestCase):
    def test_original_all16_and_checkers_complete_with_retained_lane(self) -> None:
        with synthetic() as fixture:
            live, result = run.launch(fixture.plan, fixture.lane_fd, fixture.output)
            self.assertEqual(result.completed, child.SUITE_NAMES)
            self.assertEqual(result.shell_exit, 0)
            with (fixture.base/'lock').open('a') as other, self.assertRaises(BlockingIOError):
                fcntl.flock(other.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
            with patch.object(run, 'inputs_current', return_value=()):
                final = run.finish(fixture.inputs, live, result, fixture.output)
            self.assertEqual(final.status, 'PASS')
            self.assertEqual(len(document(final.receipt.read_text()).field('oracles').array()), 16)
            self.assertIsNotNone(live.shell)

    def test_native_failure_stops_exact_original_prefix(self) -> None:
        for index in (1, 8, 16):
            with self.subTest(index=index), synthetic(fail_native=index) as fixture:
                live, result = run.launch(fixture.plan, fixture.lane_fd, fixture.output)
                self.assertEqual(result.completed, child.SUITE_NAMES[:index])
                self.assertEqual(result.shell_exit, 7)
                with patch.object(run, 'inputs_current', return_value=()):
                    final = run.finish(fixture.inputs, live, result, fixture.output)
                self.assertEqual(final.status, 'FAIL')
                self.assertEqual(len(document(final.receipt.read_text()).field('oracles').array()), 16)

    def test_original_checker_failure_stops_exact_prefix(self) -> None:
        for index in (1, 8, 15):
            with self.subTest(index=index), synthetic(fail_oracle=index) as fixture:
                live, result = run.launch(fixture.plan, fixture.lane_fd, fixture.output)
                self.assertEqual(result.completed, child.SUITE_NAMES[:index])
                self.assertEqual(result.shell_exit, 1)
                with patch.object(run, 'inputs_current', return_value=()):
                    self.assertEqual(run.finish(fixture.inputs, live, result, fixture.output).status, 'FAIL')

    def test_original_final_grep_failure_preserved(self) -> None:
        with synthetic(stall_bad=True) as fixture:
            live, result = run.launch(fixture.plan, fixture.lane_fd, fixture.output)
            self.assertEqual(result.completed, child.SUITE_NAMES)
            with patch.object(run, 'inputs_current', return_value=()):
                self.assertEqual(run.finish(fixture.inputs, live, result, fixture.output).status, 'FAIL')

    def test_exception_crossing_popen_intent_is_unknown_even_without_handle(self) -> None:
        with synthetic() as fixture, patch.object(subprocess, 'Popen', side_effect=OSError('ambiguous launch')):
            with self.assertRaises(run.OracleSuiteOwnershipUnknown) as caught:
                _ = run.launch(fixture.plan, fixture.lane_fd, fixture.output)
            live = caught.exception.live
            try:
                self.assertTrue(live.launch_intent)
                self.assertIsNone(live.shell)
                self.assertFalse((fixture.output/'result.json').exists())
                with (fixture.base/'lock').open('a') as other, self.assertRaises(BlockingIOError):
                    fcntl.flock(other.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
            finally:
                clean_unknown(live)

    def test_unprintable_start_failure_retains_actual_shell_and_channels(self) -> None:
        class Unprintable(RuntimeError):
            @override
            def __str__(self) -> str:
                raise TypeError('cannot render')
        with synthetic() as fixture, patch.object(child.SuiteController, 'start', side_effect=Unprintable()):
            with self.assertRaises(run.OracleSuiteOwnershipUnknown) as caught:
                _ = run.launch(fixture.plan, fixture.lane_fd, fixture.output)
            try:
                self.assertIsNotNone(caught.exception.live.shell)
                self.assertFalse((fixture.output/'result.json').exists())
                self.assertIsInstance(caught.exception.failures[0], Unprintable)
            finally:
                clean_unknown(caught.exception.live)

    def test_checker_mutation_after_freeze_rejects(self) -> None:
        with synthetic() as fixture:
            live, result = run.launch(fixture.plan, fixture.lane_fd, fixture.output)
            original = checker.validate_log
            def mutate(name: str, path: Path) -> None:
                original(name, path)
                with path.open('a') as stream:
                    _ = stream.write('\n')
            with patch.object(checker, 'validate_log', side_effect=mutate), patch.object(run, 'inputs_current', return_value=()), \
                    self.assertRaisesRegex(ValueError, 'changed during audit'):
                _ = run.finish(fixture.inputs, live, result, fixture.output)
            self.assertFalse((fixture.output/'result.json').exists())


@contextmanager
def admission() -> Generator[run.OracleInputs]:
    """Real source bytes with synthetic cache/library stubs; no launch qualification."""
    with TemporaryDirectory() as directory:
        base = Path(directory).resolve()
        product = base/'product'
        tools = base/'tools'
        tools.mkdir()
        for name in import_closure(Path(run.__file__).parent, 'generic_oracles_run.py'):
            # The adapter's static closure is closed below across the root tool directory.
            source = Path(run.__file__) if name == 'generic_oracles_run.py' else ROOT/name
            _ = (tools/name).write_bytes(source.read_bytes())
        pending = list(tools.iterdir())
        names: set[str] = set()
        for source in pending:
            names.update(import_closure(ROOT, source.name) if source.name != 'generic_oracles_run.py' else ())
        # Start with the structural and fixture closures; they are the adapter's repository imports.
        for entry in ('generic_m6a_structural.py', 'generic_fixtures.py', 'gate_oracle_child.py', 'mux2_gate_oracle_log.py'):
            names.update(import_closure(ROOT, entry))
        for name in names | set(run.PROTOCOL_PINS):
            _ = (tools/name).write_bytes((ROOT/name).read_bytes())
        for name in run.NO_WORLD_PINS:
            target = product/name
            target.parent.mkdir(parents=True, exist_ok=True)
            _ = target.write_bytes((ROOT.parent.parent/name).read_bytes())
        for source in tools.iterdir():
            target = product/'tools/v8'/source.name
            target.parent.mkdir(parents=True, exist_ok=True)
            _ = target.write_bytes(source.read_bytes())
        for name in ('scripts', 'assets', 'localization', 'shaders', '.godot/imported'):
            (product/name).mkdir(parents=True, exist_ok=True)
        for name in ('sim_bridge.gdextension', '.godot/extension_list.cfg', '.godot/global_script_class_cache.cfg',
                     '.godot/uid_cache.bin', 'rust/target/debug/libsim_bridge.dylib', 'rust/target/release/libsim_bridge.dylib', 'tools/v8/gate.sh'):
            path = product/name
            path.parent.mkdir(parents=True, exist_ok=True)
            _ = path.write_text('synthetic unused resource')
        python = Path(sys.executable).resolve()
        bash = Path('/bin/bash').resolve()
        yield run.OracleInputs(
            ProbeProduct('a'*40, python, file_hash(python), product,
                         tuple(sorted((path.relative_to(product).as_posix(), file_hash(path)) for path in product.rglob('*') if path.is_file()))),
            Tooling(python, file_hash(python), bash, file_hash(bash), tools,
                    tuple(sorted((path.name, file_hash(path)) for path in tools.iterdir())),
                    str(Path(sys.executable).parent)+':/usr/bin:/bin'))


class AdmissionTests(unittest.TestCase):
    def test_exact_loaded_source_and_current_resources_bind(self) -> None:
        with admission() as inputs, patch.object(run, 'resolution_paths', return_value=set()):
            sources = dict(run.inputs_current(inputs))
            self.assertEqual(sources[inputs.product.executable], inputs.product.executable_sha256)
            self.assertIn(inputs.product.input_root/'tools/v8/gate.sh', sources)

    def test_resealed_changed_no_world_source_rejected(self) -> None:
        from dataclasses import replace
        with admission() as inputs, patch.object(run, 'resolution_paths', return_value=set()):
            name = 'tools/v8/mux2_stall_test.gd'
            _ = (inputs.product.input_root/name).write_text('changed self-test could instantiate a world')
            updated = dict(inputs.product.input_files)
            updated[name] = file_hash(inputs.product.input_root/name)
            changed = replace(inputs, product=replace(inputs.product, input_files=tuple(updated.items())))
            with self.assertRaisesRegex(ValueError, 'unreviewed no-world'):
                _ = run.inputs_current(changed)

    def test_imported_oracle_cannot_be_substituted_by_sealed_shadow_file(self) -> None:
        with admission() as inputs, patch.object(run, 'resolution_paths', return_value=set()), \
                patch.object(checker, '__file__', str(inputs.tools.root/'mux2_gate_oracle_log.py')), \
                self.assertRaisesRegex(ValueError, 'loaded coordinator differs'):
            _ = run.inputs_current(inputs)

    def test_resource_omission_and_materialized_tool_mutation_reject(self) -> None:
        from dataclasses import replace
        with admission() as inputs, patch.object(run, 'resolution_paths', return_value=set()):
            changed = replace(inputs, product=replace(inputs.product, input_files=tuple(
                (name, digest) for name, digest in inputs.product.input_files if name != 'rust/target/release/libsim_bridge.dylib')))
            with self.assertRaisesRegex(ValueError, 'incomplete resources'):
                _ = run.inputs_current(changed)
            _ = (inputs.product.input_root/'tools/v8/mux2_gate_oracle_log.py').write_text('print("false PASS")')
            with self.assertRaisesRegex(ValueError, 'product input changed'):
                _ = run.inputs_current(inputs)

    def test_prelaunch_admission_failure_never_crosses_popen(self) -> None:
        with admission() as inputs, patch.object(run, 'resolution_paths', return_value=set()), \
                patch.object(subprocess, 'Popen') as popen:
            _ = (inputs.product.input_root/'project.godot').write_text('unreviewed autoload')
            with self.assertRaises(ValueError):
                _ = run.run_component(inputs, inputs.product.input_root.parent/'new-output', -1)
            popen.assert_not_called()


class OwnershipBoundaryTests(unittest.TestCase):
    def test_complete_public_component_ignores_inherited_shell_injection(self) -> None:
        with synthetic() as fixture:
            injection = fixture.base/'injection.sh'
            _ = injection.write_text('exit 0\n')
            with patch.object(run, 'inputs_current', return_value=fixture.plan.inputs), \
                    patch.dict(os.environ, {'BASH_ENV': str(injection), 'PYTHONOPTIMIZE': '2'}):
                result = run.run_component(fixture.inputs, fixture.base/'component', fixture.lane_fd)
            self.assertEqual(result.status, 'PASS')
            receipt = document(result.receipt.read_text())
            self.assertEqual(len(receipt.field('completed').array()), 16)
            environment = document((result.receipt.parent/'caller.json').read_text()).field('environment').mapping()
            self.assertNotIn('BASH_ENV', environment)
            self.assertNotIn('PYTHONOPTIMIZE', environment)
            self.assertIs(receipt.field('publication_eligible').value, False)

    def test_fault_after_actual_grant_retains_handles_without_exception_rendering(self) -> None:
        class Unprintable(RuntimeError):
            @override
            def __str__(self) -> str:
                raise TypeError('cannot render')
        original = child.SuiteController.service_ready
        def fault(owner: child.SuiteController) -> None:
            original(owner)
            if owner.active is not None:
                raise Unprintable()
        with synthetic() as fixture, patch.object(child.SuiteController, 'service_ready', fault):
            with self.assertRaises(run.OracleSuiteOwnershipUnknown) as caught:
                _ = run.launch(fixture.plan, fixture.lane_fd, fixture.output)
            live = caught.exception.live
            try:
                self.assertIsNotNone(live.shell)
                self.assertIsNotNone(live.controller)
                self.assertFalse((fixture.output/'result.json').exists())
                with (fixture.base/'lock').open('a') as other, self.assertRaises(BlockingIOError):
                    fcntl.flock(other.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
            finally:
                clean_unknown(live)
            self.assertFalse(fixture.plan.rows[1].evidence.exists())

    def test_popen_spawns_then_raises_retains_unknown_with_no_returned_handle(self) -> None:
        from typing import BinaryIO
        actual = subprocess.Popen[bytes]
        shells: list[subprocess.Popen[bytes]] = []
        def ambiguous(argv: tuple[str, ...], *, cwd: Path, env: dict[str, str], stdin: int,
                      stdout: BinaryIO, stderr: BinaryIO, pass_fds: tuple[int, ...]) -> subprocess.Popen[bytes]:
            shells.append(actual(argv, cwd=cwd, env=env, stdin=stdin, stdout=stdout, stderr=stderr, pass_fds=pass_fds))
            raise RuntimeError('test discarded return handle after actual spawn')
        with synthetic() as fixture, patch.object(subprocess, 'Popen', side_effect=ambiguous):
            with self.assertRaises(run.OracleSuiteOwnershipUnknown) as caught:
                _ = run.launch(fixture.plan, fixture.lane_fd, fixture.output)
            try:
                self.assertIsNone(caught.exception.live.shell)
                self.assertTrue(caught.exception.live.launch_intent)
                self.assertEqual(len(shells), 1)
                _ = shells[0].wait(timeout=5)
                self.assertFalse(tuple(fixture.plan.evidence.glob('*/native-start.json')))
            finally:
                for process in shells:
                    if process.poll() is None:
                        _ = process.wait(timeout=5)
                clean_unknown(caught.exception.live)

    def test_omitted_completed_children_cannot_be_resealed_as_empty_prefix(self) -> None:
        from dataclasses import replace
        with synthetic(fail_native=1) as fixture:
            live, result = run.launch(fixture.plan, fixture.lane_fd, fixture.output)
            with self.assertRaisesRegex(ValueError, 'foreign suite completion'):
                _ = run.finish(fixture.inputs, live, replace(result, completed=()), fixture.output)
            self.assertFalse((fixture.output/'result.json').exists())

    def test_terminal_file_without_returned_close_does_not_settle_caller(self) -> None:
        original = child.SuiteController.close_settled
        def failure(owner: child.SuiteController) -> child.SuiteResult:
            _ = original(owner)
            raise RuntimeError('failure after helper terminal publication')
        with synthetic(fail_native=1) as fixture, patch.object(child.SuiteController, 'close_settled', failure):
            with self.assertRaises(run.OracleSuiteOwnershipUnknown) as caught:
                _ = run.launch(fixture.plan, fixture.lane_fd, fixture.output)
            try:
                self.assertTrue((fixture.plan.evidence/'suite-terminal.json').is_file())
                self.assertFalse((fixture.output/'caller.json').exists())
                self.assertFalse((fixture.output/'result.json').exists())
            finally:
                clean_unknown(caught.exception.live)


class EagerProvenanceTests(unittest.TestCase):
    def test_late_unused_import_cannot_change_source_admission(self) -> None:
        import importlib
        with admission() as inputs, patch.object(run, 'resolution_paths', return_value=set()):
            before = run.inputs_current(inputs)
            self.assertNotIn('core_ui_successor_perf_validate', run.EAGER_SOURCES)
            _ = importlib.import_module('core_ui_successor_perf_validate')
            self.assertEqual(run.inputs_current(inputs), before)
            self.assertIn('core_ui_successor_perf_validate.py', dict(inputs.tools.files))

    def test_used_module_replacement_with_same_path_still_rejects(self) -> None:
        from types import ModuleType
        replacement = ModuleType('mux2_gate_oracle_log')
        replacement.__file__ = checker.__file__
        with admission() as inputs, patch.object(run, 'resolution_paths', return_value=set()), \
                patch.dict(sys.modules, {'mux2_gate_oracle_log': replacement}), \
                self.assertRaisesRegex(ValueError, 'loaded coordinator differs'):
            _ = run.inputs_current(inputs)

    def test_used_helper_missing_from_module_table_rejects(self) -> None:
        with admission() as inputs, patch.object(run, 'resolution_paths', return_value=set()), patch.dict(sys.modules):
            del sys.modules['mux2_gate_oracle_log']
            with self.assertRaisesRegex(ValueError, 'loaded coordinator differs'):
                _ = run.inputs_current(inputs)

    def test_lazy_unused_source_remains_sealed_after_import(self) -> None:
        import importlib
        _ = importlib.import_module('core_ui_successor_perf_validate')
        with admission() as inputs, patch.object(run, 'resolution_paths', return_value=set()):
            path = inputs.tools.root/'core_ui_successor_perf_validate.py'
            _ = path.write_text(path.read_text()+'\n# changed unused but sealed source\n')
            with self.assertRaisesRegex(ValueError, 'tool source changed'):
                _ = run.inputs_current(inputs)
