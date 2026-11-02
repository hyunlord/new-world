# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m unittest discover -s tools/v8 -p test_generic_process.py
"""Real small children exercise retained outcomes and input binding failures."""
import json
import os
import signal
import sys
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

from generic_gate import GateContractError, document
from generic_process import ProcessSpec, atomic_json, execute, file_hash


class ProcessTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(prefix="gate-process-tests-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.executable = Path(sys.executable).resolve()

    def spec(self, source: str, *arguments: str) -> ProcessSpec:
        script = self.root / "child.py"
        script.write_text(source, encoding="utf-8")
        return ProcessSpec(argv=(str(self.executable), "-I", "-S", str(script), *arguments),
                           cwd=self.root, executable=self.executable,
                           executable_sha256=file_hash(self.executable),
                           input_root=self.root, inputs=(("child.py", file_hash(script)),),
                           output_root=self.root / "output")

    def test_fresh_settings_created_before_child_and_bound(self) -> None:
        spec = replace(self.spec("from pathlib import Path\nassert Path('output/settings/nested').is_dir()\nassert not list(Path('output/settings/nested').iterdir())\n"),
                       fresh_directories=("settings/nested",))
        result = execute(spec)
        self.assertEqual(result.status, "PASS")
        self.assertEqual(json.loads(result.receipt.read_text())["fresh_directories"], ["settings/nested"])

    def test_unsafe_fresh_directories_never_launch(self) -> None:
        spec = self.spec("raise RuntimeError('must not execute')\n")
        for names in (("../escape",), ("/absolute",), (".",), ("",), ("settings/../escape",),
                      ("settings", "settings"), ("process.json",), ("stdout.log/nested",),
                      ("process-start.json.pending",), ("settings//nested",), ("STDOUT.LOG",), ("settings", "SETTINGS"), ("bad\0name",)):
            with self.subTest(names=names), patch("generic_process.subprocess.Popen") as launch:
                with self.assertRaises(GateContractError):
                    execute(replace(spec, fresh_directories=names))
                launch.assert_not_called()
        self.assertFalse(spec.output_root.exists())

    def test_symlink_in_fresh_subtree_cannot_escape(self) -> None:
        spec = replace(self.spec("raise RuntimeError('must not execute')\n"), fresh_directories=("settings/nested",))
        outside = self.root / "outside"
        outside.mkdir()
        original = Path.mkdir
        def inject(path: Path, mode: int = 0o777, parents: bool = False, exist_ok: bool = False) -> None:
            original(path, mode=mode, parents=parents, exist_ok=exist_ok)
            if path == spec.output_root:
                (path / "settings").symlink_to(outside, target_is_directory=True)
        with patch.object(Path, "mkdir", inject), patch("generic_process.subprocess.Popen") as launch:
            result = execute(spec)
            launch.assert_not_called()
        self.assertEqual((result.status, result.pid), ("FAIL", None))
        self.assertFalse((outside / "nested").exists())

    def test_success_records_actual_process_and_output_bytes(self) -> None:
        # Given a real child; when executed; then its native outcome and logs bind.
        result = execute(self.spec("import sys\nprint('hello')\nprint('error text', file=sys.stderr)\n"))
        receipt = json.loads(result.receipt.read_text())
        self.assertEqual(result.status, "PASS")
        self.assertEqual(result.native_exit, 0)
        self.assertGreater(result.pid or 0, 0)
        self.assertEqual((self.root / "output/stdout.log").read_bytes(), b"hello\n")
        self.assertEqual(receipt["stderr_sha256"], file_hash(self.root / "output/stderr.log"))
        self.assertFalse(receipt["build_provenance_verified"])
        self.assertNotIn("timeout_seconds", document(result.receipt.read_text()).mapping())
        self.assertNotIn("timed_out", document(result.receipt.read_text()).mapping())

    def test_timeout_terminates_and_reaps_owned_child(self) -> None:
        spec = replace(self.spec("import time\nprint('started', flush=True)\ntime.sleep(60)\n"), timeout_seconds=0.5)
        result = execute(spec)
        receipt = document(result.receipt.read_text()).mapping()
        self.assertEqual(result.status, "FAIL")
        self.assertEqual(result.native_exit, -signal.SIGTERM)
        self.assertEqual(receipt["timeout_seconds"], 0.5)
        self.assertIs(receipt["timed_out"], True)
        self.assertIn("started", (spec.output_root / "stdout.log").read_text())
        self.assertIsNotNone(result.pid)
        if result.pid is not None:
            with self.assertRaises(ChildProcessError):
                _ = os.waitpid(result.pid, os.WNOHANG)

    def test_timeout_remains_failure_when_child_handles_termination_with_zero_exit(self) -> None:
        spec = replace(self.spec("import signal, sys, time\nsignal.signal(signal.SIGTERM, lambda *_: sys.exit(0))\nprint('ready', flush=True)\ntime.sleep(60)\n"), timeout_seconds=0.5)
        result = execute(spec)
        self.assertEqual((result.status, result.native_exit), ("FAIL", 0))
        self.assertIs(document(result.receipt.read_text()).mapping()["timed_out"], True)

    def test_timeout_kills_and_reaps_child_ignoring_termination(self) -> None:
        spec = replace(self.spec("import signal, time\nsignal.signal(signal.SIGTERM, signal.SIG_IGN)\nprint('ready', flush=True)\ntime.sleep(60)\n"), timeout_seconds=0.5)
        result = execute(spec)
        self.assertEqual((result.status, result.native_exit), ("FAIL", -signal.SIGKILL))
        self.assertIs(document(result.receipt.read_text()).mapping()["timed_out"], True)
        if result.pid is not None:
            with self.assertRaises(ChildProcessError):
                _ = os.waitpid(result.pid, os.WNOHANG)

    def test_configured_timeout_success_is_bound(self) -> None:
        result = execute(replace(self.spec("print('done')\n"), timeout_seconds=5))
        receipt = document(result.receipt.read_text()).mapping()
        self.assertEqual(result.status, "PASS")
        self.assertEqual(receipt["timeout_seconds"], 5)
        self.assertIs(receipt["timed_out"], False)

    def test_invalid_timeout_never_creates_output_or_launches(self) -> None:
        spec = self.spec("raise RuntimeError('must not execute')\n")
        for timeout in (True, False, 0, -1, float('inf'), float('nan')):
            with self.subTest(timeout=timeout), patch("generic_process.subprocess.Popen") as launch:
                with self.assertRaises(GateContractError):
                    _ = execute(replace(spec, timeout_seconds=timeout))
                launch.assert_not_called()
                self.assertFalse(spec.output_root.exists())

    def test_nonzero_exit_retains_logs_and_failure(self) -> None:
        # Given a child failing after output; when executed; then evidence survives.
        result = execute(self.spec("import sys\nprint('before failure')\nsys.exit(7)\n"))
        self.assertEqual((result.status, result.native_exit), ("FAIL", 7))
        self.assertIn("before failure", (self.root / "output/stdout.log").read_text())
        self.assertIn("native exit 7", json.loads(result.receipt.read_text())["errors"])

    def test_input_mutation_fails_even_with_zero_native_exit(self) -> None:
        # Given a declared script that edits itself; when executed; then endpoint binding fails.
        result = execute(self.spec("from pathlib import Path\nPath(__file__).write_text('changed')\n"))
        receipt = json.loads(result.receipt.read_text())
        self.assertEqual((result.status, result.native_exit), ("FAIL", 0))
        self.assertNotEqual(receipt["before_inputs"], receipt["after_inputs"])

    def test_deleted_input_is_retained_failure(self) -> None:
        # Given an input deletion; when executed; then missing endpoint is failed evidence.
        result = execute(self.spec("from pathlib import Path\nPath(__file__).unlink()\n"))
        self.assertEqual((result.status, result.native_exit), ("FAIL", 0))
        self.assertTrue(any("postflight" in error for error in json.loads(result.receipt.read_text())["errors"]))

    def test_wrong_executable_digest_never_launches(self) -> None:
        # Given a wrong pin; when run is admitted; then no child is launched.
        spec = replace(self.spec("raise RuntimeError('must not execute')\n"), executable_sha256="0" * 64)
        result = execute(spec)
        self.assertEqual((result.status, result.pid, result.native_exit), ("FAIL", None, None))
        self.assertEqual((self.root / "output/stdout.log").read_bytes(), b"")

    def test_stale_output_is_never_replaced(self) -> None:
        # Given previous evidence; when the same root is requested; then it is retained.
        spec = self.spec("print('first')\n")
        first = execute(spec).receipt.read_bytes()
        with self.assertRaises(GateContractError):
            execute(spec)
        self.assertEqual((spec.output_root / "process.json").read_bytes(), first)

    def test_inherited_overrides_are_absent_and_explicit_values_are_bound(self) -> None:
        # Given ambient gameplay overrides; when an explicit policy runs; then none leak in.
        spec = replace(self.spec("import os\nprint(os.environ.get('WORLDSIM_SECRET', 'absent'))\nprint(os.environ['WORLDSIM_GATE_OBSERVER'])\n"),
                       environment=(("WORLDSIM_GATE_OBSERVER", "1"),))
        with patch.dict(os.environ, {"WORLDSIM_SECRET": "inherited", "PYTHONPATH": "/bad"}):
            result = execute(spec)
        self.assertEqual(result.status, "PASS")
        self.assertEqual((spec.output_root / "stdout.log").read_text(), "absent\n1\n")
        self.assertNotIn("PYTHONPATH", json.loads(result.receipt.read_text())["environment"])

    def test_symlink_input_fails_without_launch(self) -> None:
        # Given a symlink input; when preflight runs; then the child is not started.
        spec = self.spec("print('must not run')\n")
        (self.root / "link").symlink_to("child.py")
        spec = replace(spec, inputs=(("link", file_hash(self.root / "child.py")),))
        result = execute(spec)
        self.assertEqual((result.status, result.pid), ("FAIL", None))

    def test_input_escape_fails_without_launch(self) -> None:
        # Given a relative escape; when preflight runs; then it is rejected.
        spec = replace(self.spec("print('must not run')\n"), inputs=(("../outside", "0" * 64),))
        result = execute(spec)
        self.assertEqual((result.status, result.pid), ("FAIL", None))

    def test_atomic_receipt_refuses_overwrite(self) -> None:
        # Given a completed receipt; when republished; then the original bytes survive.
        path = self.root / "receipt.json"
        atomic_json(path, {"status": "first"})
        first = path.read_bytes()
        with self.assertRaises(FileExistsError):
            atomic_json(path, {"status": "second"})
        self.assertEqual(path.read_bytes(), first)

    def test_executable_path_must_match_argv(self) -> None:
        # Given an unbound argv executable; when validated; then no execution is admitted.
        spec = replace(self.spec("print('must not run')\n"), argv=("python3", "-c", "pass"))
        with self.assertRaises(GateContractError):
            execute(spec)

    def test_executable_mutation_fails_with_retained_native_success(self) -> None:
        # Given an executable script replacing its own bytes; when it exits zero; then binding fails.
        executable = self.root / "mutating-executable"
        executable.write_text("#!/bin/sh\nexec " + str(self.executable) +
                              " -I -S -c 'from pathlib import Path; import sys; Path(sys.argv[1]).write_text(\"changed\")' \"$0\"\n")
        executable.chmod(0o700)
        spec = ProcessSpec(argv=(str(executable),), cwd=self.root, executable=executable,
                           executable_sha256=file_hash(executable), input_root=self.root,
                           inputs=(), output_root=self.root / "output")
        result = execute(spec)
        self.assertEqual((result.status, result.native_exit), ("FAIL", 0))
        receipt = json.loads(result.receipt.read_text())
        self.assertNotEqual(receipt["before_executable_sha256"], receipt["after_executable_sha256"])

    def test_spawn_error_has_terminal_receipt_and_empty_logs(self) -> None:
        # Given executable bytes with no valid format; when spawn fails; then failure is retained.
        executable = self.root / "invalid-executable"
        executable.write_bytes(b"not an executable format")
        executable.chmod(0o700)
        spec = ProcessSpec(argv=(str(executable),), cwd=self.root, executable=executable,
                           executable_sha256=file_hash(executable), input_root=self.root,
                           inputs=(), output_root=self.root / "output")
        result = execute(spec)
        self.assertEqual((result.status, result.native_exit, result.pid), ("FAIL", None, None))
        self.assertTrue(any("spawn:" in error for error in json.loads(result.receipt.read_text())["errors"]))

    def test_directory_is_not_an_exact_input_file(self) -> None:
        # Given a directory in the file list; when preflight checks it; then no child runs.
        spec = self.spec("print('must not run')\n")
        (self.root / "directory").mkdir()
        result = execute(replace(spec, inputs=(("directory", "0" * 64),)))
        self.assertEqual((result.status, result.pid), ("FAIL", None))

    def test_no_shell_interpretation_of_arguments(self) -> None:
        # Given shell metacharacters as argv; when executed; then they remain literal.
        argument = "$(touch surprise); echo x"
        result = execute(self.spec("import sys\nprint(sys.argv[1])\n", argument))
        self.assertEqual(result.status, "PASS")
        self.assertEqual((self.root / "output/stdout.log").read_text(), argument + "\n")
        self.assertFalse((self.root / "surprise").exists())


class ExternalInputTests(unittest.TestCase):
    """Mock the child only; exercise real file/hash/receipt paths without native execution."""
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="gate-external-input-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.inputs = self.root / "inputs"
        self.inputs.mkdir()
        self.executable = self.root / "bound-executable"
        self.executable.write_bytes(b"never executed by these synthetic tests")
        self.executable.chmod(0o700)
        self.input = self.inputs / "static.json"
        self.input.write_text("{}")
        self.generated = self.root / "generated_commands.json"
        self.generated.write_text('[{"tick":1}]')
        self.specification = ProcessSpec((str(self.executable),"--commands",str(self.generated)),
            self.inputs,self.executable,file_hash(self.executable),self.inputs,
            (("static.json",file_hash(self.input)),),self.root / "out",
            external_inputs=((self.generated,file_hash(self.generated)),))

    def run_mocked(self, spec, action=None):
        with patch("generic_process.subprocess.Popen") as launch:
            child = launch.return_value.__enter__.return_value
            child.pid = 4242
            child.wait.side_effect = lambda: (action() if action else None) or 0
            result = execute(spec)
            called = launch.called
        return result,json.loads(result.receipt.read_text()),called

    def test_external_original_path_bound_before_and_after(self):
        result,receipt,called = self.run_mocked(self.specification)
        self.assertTrue(called)
        self.assertEqual(result.status,"PASS")
        expected = {str(self.generated):file_hash(self.generated)}
        self.assertEqual(receipt["expected_external_inputs"],expected)
        self.assertEqual(receipt["before_external_inputs"],expected)
        self.assertEqual(receipt["after_external_inputs"],expected)
        self.assertEqual(receipt["argv"],list(self.specification.argv))
        self.assertEqual(receipt["pid"],4242)
        self.assertEqual(receipt["native_exit"],0)
        self.assertFalse(receipt["build_provenance_verified"])

    def test_external_mutation_and_deletion_retain_failures(self):
        for index,delete in enumerate((False,True)):
            self.generated.write_text("original")
            spec = replace(self.specification,output_root=self.root/f"out{index}",external_inputs=((self.generated,file_hash(self.generated)),))
            def mutate(delete: bool = delete) -> None:
                if delete: self.generated.unlink()
                else: self.generated.write_text("changed")
            result,receipt,called = self.run_mocked(spec,mutate)
            self.assertTrue(called)
            self.assertEqual((result.status,result.native_exit),("FAIL",0))
            self.assertTrue(any("external input" in error for error in receipt["errors"]))
            self.assertNotEqual(receipt["before_external_inputs"],receipt["after_external_inputs"])

    def test_invalid_external_hash_symlink_noncanonical_directory_and_overlap_never_launch(self):
        link = self.root / "link";link.symlink_to(self.generated)
        cases = [(self.generated,"0"*64),(link,file_hash(self.generated)),
                 (Path("relative.json"),"0"*64),(self.root,"0"*64),
                 (self.inputs/"static.json",file_hash(self.input)),
                 (self.root/"out5"/"future.json","0"*64)]
        for index,entry in enumerate(cases):
            spec=replace(self.specification,output_root=self.root/f"out{index}",external_inputs=(entry,))
            result,receipt,called=self.run_mocked(spec)
            with self.subTest(entry=entry):
                self.assertFalse(called)
                self.assertEqual((result.status,result.pid),("FAIL",None))
                self.assertTrue(receipt["errors"])

    def test_duplicate_external_inputs_rejected_without_child(self):
        spec=replace(self.specification,external_inputs=self.specification.external_inputs*2)
        with patch("generic_process.subprocess.Popen") as launch,self.assertRaises(GateContractError):
            execute(spec)
        launch.assert_not_called()

    def test_default_empty_external_inputs_preserves_original_bindings(self):
        result,receipt,called=self.run_mocked(replace(self.specification,external_inputs=()))
        self.assertTrue(called)
        self.assertEqual(result.status,"PASS")
        for name in ("expected_external_inputs","before_external_inputs","after_external_inputs"):
            self.assertEqual(receipt[name],{})
        self.assertEqual(receipt["before_inputs"],{"static.json":file_hash(self.input)})


if __name__ == "__main__":
    unittest.main()
