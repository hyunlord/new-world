"""Actual tiny Python/Bash M1-chain fixtures; no Cargo/Godot/WorldSim execution."""

import fcntl
import os
import shutil
import sys
import unittest
from dataclasses import replace
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import override
from unittest.mock import patch

import generic_cargo_child
import generic_m1_run as m1
from generic_gate import GateContractError
from generic_library_run import Library49Request
from generic_m1_commands import COMMANDS
from generic_process import ProcessResult, ProcessSpec, atomic_json, execute, file_hash
from generic_workspace_capture import CapturedProcess, read
from generic_workspace_cases import ListingProcess, process_output
from generic_workspace_tests import Executable, Target
from mux2_gate_inputs import Json


class M1RuntimeTests(unittest.TestCase):
    def fixture(
        self, base: Path, fail: int | None = None, *, locale_pass: bool = True
    ) -> m1.M1Request:
        root = base / "source"
        root.mkdir()
        out = base / "out"
        helper = Path(generic_cargo_child.__file__).resolve()
        python = Path(sys.executable).resolve()
        bash = Path("/bin/bash").resolve()
        files: list[Path] = []
        for name in ("gate.sh", "run_bounded.sh", "test_locale_trf.gd"):
            path = root / "tools/v8" / name
            path.parent.mkdir(parents=True, exist_ok=True)
            _ = path.write_bytes((helper.parent / name).read_bytes())
            files.append(path)
        binaries: list[Executable] = []
        for index, name in enumerate(m1.NATIVE_NAMES):
            command = COMMANDS[0 if index < 4 else index - 3]
            package = root / "rust/crates" / command.package
            source = package / ("tests/" + name + ".rs" if index < 4 else "src/lib.rs")
            source.parent.mkdir(parents=True, exist_ok=True)
            _ = source.write_text("// synthetic")
            manifest = package / "Cargo.toml"
            _ = manifest.write_text("# synthetic")
            files.extend((source, manifest))
            binary = base / ("native-" + name)
            case = (command.filter or "") + "synthetic"
            argv = ["--test-threads=8"] if index < 4 else [command.filter]
            body = f"""import sys
case={case!r}
if '--list' in sys.argv:
 print('0 tests, 0 benchmarks' if '--ignored' in sys.argv else case+': test\\n'+{'"other: test\\n"' if index >= 4 else '""'}+'\\n{2 if index >= 4 else 1} {"tests" if index >= 4 else "test"}, 0 benchmarks')
else:
 assert sys.argv[1:]=={argv!r},sys.argv
 print('\\nrunning 1 test\\ntest '+case+' ... {"FAILED" if fail == index else "ok"}\\n\\ntest result: {"FAILED" if fail == index else "ok"}. {0 if fail == index else 1} passed; {1 if fail == index else 0} failed; 0 ignored; 0 measured; {1 if index >= 4 else 0} filtered out; finished in 0.00s\\n')
 raise SystemExit({101 if fail == index else 0})
"""
            _ = binary.write_text("#!" + str(python) + "\n" + body)
            binary.chmod(0o755)
            binaries.append(
                Executable(
                    Target(
                        command.package,
                        command.package,
                        name if index < 4 else command.package.replace("-", "_"),
                        ("test",) if index < 4 else ("lib",),
                        source,
                        manifest,
                        False,
                        True,
                    ),
                    binary,
                    17,
                    "release",
                    command.filter,
                )
            )
        cargo = base / "cargo"
        commands = [list(command.argv[1:]) for command in COMMANDS]
        invocations = [
            [
                (
                    str(e.binary),
                    list(("--test-threads=8",) if i < 4 else (COMMANDS[i - 3].filter,)),
                    str(e.cwd),
                )
                for i, e in enumerate(binaries)
                if (stage == 0 and i < 4) or (stage > 0 and i == stage + 3)
            ]
            for stage in range(4)
        ]
        _ = cargo.write_text(
            "#!"
            + str(python)
            + """\nimport os,shlex,subprocess,sys
commands="""
            + repr(commands)
            + """\ninvocations="""
            + repr(invocations)
            + """
stage=commands.index(sys.argv[1:]);runner=shlex.split(os.environ['CARGO_TARGET_TEST_HOST_RUNNER'])
for binary,args,cwd in invocations[stage]:
 code=subprocess.call([*runner,binary,*args],cwd=cwd)
 if code:raise SystemExit(code)
"""
        )
        cargo.chmod(0o755)
        godot = base / "godot"
        _ = godot.write_text(
            "#!"
            + str(python)
            + "\nprint("
            + repr("TRF_TEST_PASS synthetic" if locale_pass else "NO_MARKER")
            + ")\n"
        )
        godot.chmod(0o755)
        paths = {
            *helper.parent.glob("*.py"),
            Path(m1.__file__).resolve(),
            cargo,
            godot,
            python,
            bash,
            *(e.binary for e in binaries),
        }
        envpath = "/opt/homebrew/bin:/usr/bin:/bin"
        for name in ("cat", "grep", "python3", "bash"):
            found = shutil.which(name, path=envpath)
            assert found is not None
            paths.add(Path(found).resolve())
        tools = tuple((path, file_hash(path)) for path in sorted(paths))
        inputs = tuple(
            (str(path.relative_to(root)), file_hash(path))
            for path in sorted(set(files))
        )
        lane = base / "lane"
        descriptor = os.open(lane, os.O_CREAT | os.O_RDWR, 0o600)
        fcntl.flock(descriptor, fcntl.LOCK_EX)
        self.addCleanup(os.close, descriptor)
        template = Library49Request(
            root,
            out,
            "synthetic-m1",
            "candidate",
            binaries[0],
            cargo,
            bash,
            python,
            helper,
            "test-host",
            inputs,
            tools,
            (("PATH", envpath),),
            descriptor,
        )
        return m1.M1Request(template, tuple(binaries), godot)

    def test_full_original_chain_observes_all_producers(self) -> None:
        with TemporaryDirectory() as temporary:
            base = Path(temporary).resolve()
            request = self.fixture(base)
            with patch.object(m1, "LOCK", base / "lane"):
                result = m1.run_owned(request)
            self.assertEqual(result.status, "PASS")
            self.assertEqual(result.qualification, "PENDING")
            row = read(result.receipt)
            self.assertEqual(
                [value.text() for value in row.field("completed_producers").array()],
                [
                    *COMMANDS[0].targets,
                    "cargo-integration",
                    "probe",
                    "cargo-probe",
                    "jobs",
                    "cargo-jobs",
                    "colony",
                    "cargo-colony",
                    "godot",
                    "bounded",
                ],
            )
            self.assertEqual(len(row.field("cases").array()), 7)
            self.assertEqual(row.field("skipped_producers").value, [])

    def test_native_failure_preserves_original_and_chain_suffix(self) -> None:
        with TemporaryDirectory() as temporary:
            base = Path(temporary).resolve()
            request = self.fixture(base, fail=4)
            with patch.object(m1, "LOCK", base / "lane"):
                result = m1.run_owned(request)
            self.assertEqual(result.status, "FAIL")
            row = read(result.receipt)
            self.assertEqual(row.field("outer_exit").value, 101)
            self.assertIn(
                "cargo-jobs",
                tuple(value.text() for value in row.field("skipped_producers").array()),
            )
            self.assertIn(
                "godot",
                tuple(value.text() for value in row.field("skipped_producers").array()),
            )

    def cleanup_tiny_unknown(self, live: m1.Live) -> None:
        # Only these synthetic fixtures: unblock each fixed ACK and join the actual worker.
        assert live.thread is not None
        if live.thread.is_alive():
            for name, descriptor in live.descriptors.items():
                if name != "request":
                    _ = os.write(descriptor, b"G")
        live.thread.join(30)
        self.assertFalse(live.thread.is_alive(), "test-owned worker did not settle")
        for descriptor in live.descriptors.values():
            os.close(descriptor)
        live.descriptors.clear()

    def test_late_listing_cannot_add_an_untracked_empty_directory(self) -> None:
        actual = execute
        with TemporaryDirectory() as temporary:
            base = Path(temporary).resolve()
            request = self.fixture(base)
            injected: list[Path] = []

            def changed(spec: ProcessSpec) -> ProcessResult:
                result = actual(spec)
                if spec.output_root.name == "ignored":
                    (request.template.output / "untracked").mkdir(exist_ok=True)
                    injected.append(spec.output_root)
                return result

            with (
                patch.object(m1, "LOCK", base / "lane"),
                patch.object(m1, "execute", changed),
                self.assertRaises(m1.M1OwnershipUnknown) as caught,
            ):
                _ = m1.run_owned(request)
            try:
                self.assertEqual(
                    injected,
                    [request.template.output / "listings/colony_commands/ignored"],
                )
                cause = caught.exception.original
                self.assertIsInstance(cause, GateContractError)
                self.assertEqual(
                    cause.args,
                    ("CONTRACT_REJECTED /outputs: unexpected output namespace",),
                )
                self.assertIs(caught.exception.live.request, request)
            finally:
                self.cleanup_tiny_unknown(caught.exception.live)

    def test_unbound_or_injected_request_rejects_before_output(self) -> None:
        with TemporaryDirectory() as temporary:
            base = Path(temporary).resolve()
            request = self.fixture(base)
            variants = (
                replace(
                    request,
                    template=replace(
                        request.template,
                        environment=(
                            (*request.template.environment, ("BASH_ENV", "/injected"))
                        ),
                    ),
                ),
                replace(
                    request,
                    template=replace(
                        request.template, tools=request.template.tools[1:]
                    ),
                ),
                replace(
                    request,
                    executables=(
                        replace(request.executables[0], filter="foreign"),
                        *request.executables[1:],
                    ),
                ),
            )
            for invalid in variants:
                with (
                    self.subTest(request=invalid.template.environment),
                    patch.object(m1, "LOCK", base / "lane"),
                    self.assertRaises(ValueError),
                ):
                    _ = m1.run_owned(invalid)
                self.assertFalse(request.template.output.exists())

    def test_unknown_retains_live_worker_and_actual_lane(self) -> None:
        class Unprintable(RuntimeError):
            @override
            def __str__(self) -> str:
                raise RuntimeError("diagnostic must not run")

        with TemporaryDirectory() as temporary:
            base = Path(temporary).resolve()
            request = self.fixture(base)

            def broken(_live: m1.Live, _plan: Path, _ident: str) -> CapturedProcess:
                raise Unprintable()

            with (
                patch.object(m1, "LOCK", base / "lane"),
                patch.object(m1, "joined", broken),
                self.assertRaises(m1.M1OwnershipUnknown) as caught,
            ):
                _ = m1.run_owned(request)
            live = caught.exception.live
            try:
                self.assertIs(live.request, request)
                self.assertIsInstance(caught.exception.original, Unprintable)
                self.assertIsNotNone(live.thread)
                assert live.thread is not None
                self.assertTrue(live.thread.is_alive())
                self.assertFalse((request.template.output / "result.json").exists())
                competing = os.open(base / "lane", os.O_RDWR)
                try:
                    with self.assertRaises(BlockingIOError):
                        fcntl.flock(competing, fcntl.LOCK_EX | fcntl.LOCK_NB)
                finally:
                    os.close(competing)
            finally:
                # Test-owned tiny chain only: release each exact ACK once, then wait its retained worker.
                for name, descriptor in live.descriptors.items():
                    if name != "request":
                        _ = os.write(descriptor, b"G")
                assert live.thread is not None
                live.thread.join(30)
                self.assertFalse(live.thread.is_alive())
                for descriptor in live.descriptors.values():
                    os.close(descriptor)
                live.descriptors.clear()
            self.assertIsNotNone(live.result)

    def test_final_publication_cannot_replace_frozen_empty_directory(self) -> None:
        with TemporaryDirectory() as temporary:
            base = Path(temporary).resolve()
            request = self.fixture(base)
            injected: list[Path] = []

            def changed(path: Path, value: dict[str, Json]) -> None:
                atomic_json(path, value)
                if path.name == "result.json":
                    (request.template.output / "capture/integration-extra").mkdir()
                    injected.append(path)

            with (
                patch.object(m1, "LOCK", base / "lane"),
                patch.object(m1, "atomic_json", changed),
                self.assertRaises(m1.M1OwnershipUnknown) as caught,
            ):
                _ = m1.run_owned(request)
            try:
                self.assertEqual(injected, [request.template.output / "result.json"])
                cause = caught.exception.original
                self.assertIsInstance(cause, GateContractError)
                self.assertEqual(
                    cause.args,
                    ("CONTRACT_REJECTED /outputs: capture namespace changed",),
                )
                self.assertIs(caught.exception.live.request, request)
            finally:
                self.cleanup_tiny_unknown(caught.exception.live)

    def test_original_locale_oracle_failure_is_not_native_failure(self) -> None:
        with TemporaryDirectory() as temporary:
            base = Path(temporary).resolve()
            request = self.fixture(base, locale_pass=False)
            with patch.object(m1, "LOCK", base / "lane"):
                result = m1.run_owned(request)
            self.assertEqual(result.status, "FAIL")
            self.assertEqual(read(result.receipt).field("outer_exit").value, 1)
            for name in ("godot", "bounded"):
                terminal = read(
                    request.template.output
                    / "capture"
                    / name
                    / "native/000001/receipt.json"
                )
                self.assertEqual(terminal.field("native_exit").value, 0)

    def test_first_authentication_cannot_adopt_new_completed_output(self) -> None:
        for producer in ("outer", "full"):
            with self.subTest(producer=producer), TemporaryDirectory() as temporary:
                base = Path(temporary).resolve()
                request = self.fixture(base)

                def changed(
                    binding: ListingProcess,
                    argv: tuple[str, ...],
                    cwd: Path,
                    environment: dict[str, str],
                    required_inputs: dict[str, str],
                    selected: str = producer,
                ) -> bytes:
                    data = process_output(
                        binding, argv, cwd, environment, required_inputs
                    )
                    if binding.spec.output_root.name == selected:
                        (binding.spec.output_root / "injected-empty").mkdir(
                            exist_ok=True
                        )
                    return data

                with (
                    patch.object(m1, "LOCK", base / "lane"),
                    patch.object(m1, "process_output", changed),
                    self.assertRaises(m1.M1OwnershipUnknown) as caught,
                ):
                    _ = m1.run_owned(request)
                live = caught.exception.live
                assert live.thread is not None
                self.assertFalse(live.thread.is_alive())
                for descriptor in live.descriptors.values():
                    os.close(descriptor)
                live.descriptors.clear()

    def test_first_capture_authentication_cannot_adopt_new_output(self) -> None:
        original = m1.joined
        with TemporaryDirectory() as temporary:
            base = Path(temporary).resolve()
            request = self.fixture(base)

            def changed(live: m1.Live, plan: Path, ident: str) -> CapturedProcess:
                result = original(live, plan, ident)
                (request.template.output / "capture" / ident / "injected-empty").mkdir(
                    exist_ok=True
                )
                return result

            with (
                patch.object(m1, "LOCK", base / "lane"),
                patch.object(m1, "joined", changed),
                self.assertRaises(m1.M1OwnershipUnknown) as caught,
            ):
                _ = m1.run_owned(request)
            live = caught.exception.live
            assert live.thread is not None
            try:
                self.assertTrue(live.thread.is_alive())
                self.assertFalse((request.template.output / "result.json").exists())
            finally:
                for name, descriptor in live.descriptors.items():
                    if name != "request":
                        _ = os.write(descriptor, b"G")
                live.thread.join(30)
                self.assertFalse(live.thread.is_alive())
                for descriptor in live.descriptors.values():
                    os.close(descriptor)
                live.descriptors.clear()

    def test_earlier_unknown_fails_mutation_assertion_and_cleans_worker(self) -> None:
        nested = M1RuntimeTests(
            "test_late_listing_cannot_add_an_untracked_empty_directory"
        )
        retained: list[m1.Live] = []

        def earlier(live: m1.Live, _plan: Path, _ident: str) -> CapturedProcess:
            retained.append(live)
            raise RuntimeError("earlier unrelated authentication failure")

        try:
            with patch.object(m1, "joined", earlier), self.assertRaises(AssertionError):
                nested.test_late_listing_cannot_add_an_untracked_empty_directory()
        finally:
            _ = nested.doCleanups()
        self.assertEqual(len(retained), 1)
        self.assertEqual(retained[0].descriptors, {})
        assert retained[0].thread is not None
        self.assertFalse(retained[0].thread.is_alive())
