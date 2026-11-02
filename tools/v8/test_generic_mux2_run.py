"""Closed step37 recipe checks; native qualification is not represented by these tests."""

import unittest
from pathlib import Path

import generic_mux2_run as mux
from generic_gate import GateContractError


class RecipeTests(unittest.TestCase):
    def test_original_phases_and_selection(self) -> None:
        self.assertEqual(len(mux.TARGETS), 10)
        self.assertEqual(len(mux.phases(Path("/v10"), Path("/manifest"))), 5)
        rows = mux.cargo_arguments()
        self.assertEqual(rows[0][-2:], ("--", "--test-threads=8"))
        self.assertEqual(rows[1][-2:], ("--", "--ignored"))
        self.assertEqual(rows[2][-2:], ("--", "--ignored"))
        self.assertEqual(rows[3][-3:], ("--", "--ignored", "--exact"))
        self.assertEqual(mux.selection(7).mode, "ignored-only")
        self.assertEqual(mux.selection(9).filter, mux.B5_CASE)


# Full original-check fixtures intentionally require a root-scheduled canonical lane.
# They contain only toy Python executables and header-only validator inputs, not legacy saves.
import fcntl
import json
import os
import shutil
import subprocess
import sys
from collections.abc import Generator
from contextlib import contextmanager
from dataclasses import replace
from tempfile import TemporaryDirectory
from typing import ClassVar, Final, override
from unittest.mock import patch

import generic_cargo_child
from generic_library_run import Library49Request
from generic_process import atomic_json, file_hash
from generic_workspace_capture import CapturedProcess, read
from generic_workspace_cases import ListingProcess, process_output
from generic_workspace_tests import Executable, Target
from mux2_gate_inputs import Json
from mux2_gate_lock import LOCK


def fixture(
    base: Path,
    lane_fd: int,
    *,
    fail: int | None = None,
    bad_validator: bool = False,
    signal_fail: int | None = None,
) -> mux.DefenceRequest:
    root = base / "source"
    root.mkdir()
    helper = Path(generic_cargo_child.__file__).resolve()
    python = Path(sys.executable).resolve()
    bash = Path("/bin/bash").resolve()
    inputs: set[Path] = set()
    for name in (
        "gate.sh",
        "mux2_gate_checks.sh",
        "mux2_gate_lock.py",
        "mux2_gate_inputs.py",
        "godot_bin.sh",
        "mux2_gate_fixtures.py",
    ):
        p = root / "tools/v8" / name
        p.parent.mkdir(parents=True, exist_ok=True)
        _ = p.write_bytes((helper.parent / name).read_bytes())
        inputs.add(p)
    package = root / "rust/crates/sim-test"
    package.mkdir(parents=True)
    manifest = package / "Cargo.toml"
    _ = manifest.write_text("# synthetic")
    inputs.add(manifest)
    executables: list[Executable] = []
    for index, name in enumerate(mux.TARGETS):
        source = package / "tests" / (name + ".rs")
        source.parent.mkdir(exist_ok=True)
        _ = source.write_text("// tiny fixture only")
        inputs.add(source)
        names = (
            ["ordinary", "ignored_case"]
            if index < 7
            else (
                ["ordinary", "ignored_case"]
                if index < 9
                else ["ordinary", mux.B5_CASE, "other_ignored"]
            )
        )
        ignored = ["ignored_case"] if index < 9 else [mux.B5_CASE, "other_ignored"]
        selected = (
            names if index < 7 else ([ignored[0]] if index < 9 else [mux.B5_CASE])
        )
        binary = base / ("native-" + name)
        body = f"""import sys
names={names!r};ignored={ignored!r};selected={selected!r}
if '--list' in sys.argv:
 listed=ignored if '--ignored' in sys.argv else names
 print('\\n'.join(name+': test' for name in listed)+'\\n\\n'+str(len(listed))+(' test' if len(listed)==1 else ' tests')+', 0 benchmarks')
else:
 assert sys.argv[1:]=={list(mux.native_args(index))!r},sys.argv
 if {signal_fail == index!r}:
  import os,signal
  os.kill(os.getpid(),signal.SIGTERM)
 print('\\nrunning '+str(len(selected))+' tests')
 for name in selected:
  status='ignored' if {index < 7!r} and name in ignored else ('FAILED' if {fail == index!r} else 'ok')
  print('test '+name+' ... '+status)
 print('\\ntest result: {"FAILED" if fail == index else "ok"}. {0 if fail == index else 1} passed; {1 if fail == index else 0} failed; {1 if index < 7 else 0} ignored; 0 measured; {0 if index < 7 else len(names) - 1} filtered out; finished in 0.00s\\n')
 raise SystemExit({101 if fail == index else 0})
"""
        _ = binary.write_text("#!" + str(python) + "\n" + body)
        binary.chmod(0o755)
        executables.append(
            Executable(
                Target(
                    "sim-test",
                    "sim-test",
                    name,
                    ("test",),
                    source,
                    manifest,
                    False,
                    True,
                ),
                binary,
                37,
                "release",
                mux.B5_CASE if index == 9 else None,
            )
        )
    cargo = base / "cargo"
    invocations: list[list[tuple[str, list[str]]]] = []
    for group in (tuple(reversed(range(7))), (7,), (8,), (9,)):
        invocations.append(
            [(str(executables[i].binary), list(mux.native_args(i))) for i in group]
        )
    _ = cargo.write_text(
        "#!"
        + str(python)
        + """\nimport os,shlex,subprocess,sys
commands="""
        + repr([list(a) for a in mux.cargo_arguments()])
        + """\ninvocations="""
        + repr(invocations)
        + """
stage=commands.index(sys.argv[1:]);runner=shlex.split(os.environ['CARGO_TARGET_TEST_HOST_RUNNER'])
for binary,args in invocations[stage]:
 code=subprocess.call([*runner,binary,*args],cwd="""
        + repr(str(package))
        + """)
 if code:raise SystemExit(code)
"""
    )
    cargo.chmod(0o755)
    legacy = base / "legacy"
    legacy.mkdir()
    h9 = legacy / "b4/h-format9-fixture/tick4000.save"
    h9.parent.mkdir(parents=True)
    _ = h9.write_bytes(b"TOY NOT A GENUINE SAVE")
    v10 = base / "v10"
    v10.mkdir()
    hashes: dict[str, str] = {}
    for i in range(4):
        p = v10 / f"toy-{i}.save"
        _ = p.write_bytes(b"WORLDSIM" + (10).to_bytes(4, "little") + b"TOY HEADER ONLY")
        hashes[p.name] = file_hash(p)
    if bad_validator:
        hashes["toy-0.save"] = "0" * 64
    original_manifest = base / "fixture-manifest.json"
    _ = original_manifest.write_text(json.dumps(hashes))
    paths = {
        *helper.parent.glob("*.py"),
        Path(mux.__file__).resolve(),
        cargo,
        python,
        bash,
        h9,
        original_manifest,
        *v10.iterdir(),
        *(e.binary for e in executables),
    }
    envpath = "/opt/homebrew/bin:/usr/bin:/bin"
    for name in ("bash", "python3", "dirname", "mkdir", "shasum", "ps"):
        located = shutil.which(name, path=envpath)
        assert located is not None
        paths.add(Path(located).resolve())
    template = Library49Request(
        root,
        base / "out",
        "tiny37",
        "candidate",
        executables[0],
        cargo,
        bash,
        python,
        helper,
        "test-host",
        tuple((str(p.relative_to(root)), file_hash(p)) for p in sorted(inputs)),
        tuple((p, file_hash(p)) for p in sorted(paths)),
        (("PATH", envpath), ("CARGO_BUILD_JOBS", "8")),
        lane_fd,
    )
    return mux.DefenceRequest(
        template, tuple(executables), legacy, h9, v10, original_manifest
    )


class NegativeTests(unittest.TestCase):
    def test_wrong_lane_rejected_before_launch(self) -> None:
        with TemporaryDirectory() as temporary:
            base = Path(temporary).resolve()
            with (base / "wrong-lane").open("w") as lane:
                request = fixture(base, lane.fileno())
                with self.assertRaises(ValueError):
                    _ = mux.run_defence_owned(request)
                self.assertFalse(request.template.output.exists())


@unittest.skipUnless(
    os.environ.get("M37_RUN_ORIGINAL_LOCK_TESTS") == "1",
    "canonical lane positive fixtures explicitly not scheduled",
)
class OwnedRuntimeTests(unittest.TestCase):
    @contextmanager
    def lane(self) -> Generator[int]:
        fd = os.open(LOCK, os.O_RDWR)
        try:
            fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BaseException:
            os.close(fd)
            raise
        try:
            yield fd
        finally:
            os.close(fd)

    def cleanup(self, live: mux.Live) -> None:
        if live.shell is not None:
            if live.shell.poll() is None:
                for name, fd in live.controls.items():
                    if name != "request":
                        _ = os.write(fd, b"G")
            _ = live.shell.wait(timeout=30)
        live.stdout.close()
        live.stderr.close()
        for fd in live.controls.values():
            os.close(fd)
        live.controls.clear()

    def test_full_original_helper_observes_actual_order(self) -> None:
        with self.lane() as fd, TemporaryDirectory() as temporary:
            request = fixture(Path(temporary).resolve(), fd)
            try:
                result = mux.run_defence_owned(request)
            except mux.DefenceOwnershipUnknown as error:
                self.cleanup(error.live)
                raise
            self.assertEqual(result.status, "PASS")
            self.assertEqual(result.qualification, "PENDING")
            row = read(result.receipt)
            completed = [v.text() for v in row.field("completed_producers").array()]
            self.assertEqual(completed[:7], list(reversed(mux.TARGETS[:7])))
            self.assertEqual(len(completed), 15)
            self.assertEqual(len(row.field("cases").array()), 10)
            self.assertEqual(
                len(
                    list(
                        (request.template.output / "listings").glob("*/*/process.json")
                    )
                ),
                20,
            )
            self.assertIn(
                b"MUX2_GATE_LOCK_HELD",
                (request.template.output / "outer/stdout.log").read_bytes(),
            )
            self.assertIn(
                b"MUX2_V10_FIXTURES_PASS files=4",
                (
                    request.template.output
                    / "capture/validator/native/000001/stdout.log"
                ).read_bytes(),
            )

    def test_each_original_phase_failure_stops_suffix(self) -> None:
        for failed_index in (6, None, 7, 8, 9):
            with (
                self.subTest(failed_index=failed_index),
                self.lane() as fd,
                TemporaryDirectory() as temporary,
            ):
                request = fixture(
                    Path(temporary).resolve(),
                    fd,
                    fail=failed_index,
                    bad_validator=failed_index is None,
                )
                try:
                    result = mux.run_defence_owned(request)
                except mux.DefenceOwnershipUnknown as error:
                    self.cleanup(error.live)
                    raise
                self.assertEqual(result.status, "FAIL")
                row = read(result.receipt)
                completed = [v.text() for v in row.field("completed_producers").array()]
                phases = [v for v in completed if v in mux.PHASES]
                count = {6: 1, None: 2, 7: 3, 8: 4, 9: 5}[failed_index]
                self.assertEqual(phases, list(mux.PHASES[:count]))
                self.assertNotEqual(row.field("caller").field("native_exit").value, 0)

    def test_reaped_native_signal_is_failed_not_unsettled_parent(self) -> None:
        with self.lane() as fd, TemporaryDirectory() as temporary:
            request = fixture(Path(temporary).resolve(), fd, signal_fail=6)
            try:
                result = mux.run_defence_owned(request)
            except mux.DefenceOwnershipUnknown as error:
                self.cleanup(error.live)
                raise
            self.assertEqual(result.status, "FAIL")
            row = read(result.receipt)
            cases = row.field("cases").array()
            self.assertEqual(len(cases), 1)
            self.assertEqual(cases[0].field("native_exit").value, -15)
            self.assertEqual(
                cases[0].field("case_accounting").value, "INCOMPLETE_FAILED_PROCESS"
            )
            self.assertEqual(row.field("caller").field("native_exit").value, 241)

    def test_unknown_retains_actual_parent_and_lane(self) -> None:
        with self.lane() as fd, TemporaryDirectory() as temporary:
            request = fixture(Path(temporary).resolve(), fd)
            seen: list[mux.Live] = []

            def broken(live: mux.Live, _name: str) -> CapturedProcess:
                seen.append(live)
                raise RuntimeError("owned fault")

            with (
                patch.object(mux, "joined", broken),
                self.assertRaises(mux.DefenceOwnershipUnknown) as caught,
            ):
                _ = mux.run_defence_owned(request)
            try:
                self.assertEqual(seen, [caught.exception.live])
                self.assertEqual(caught.exception.original.args, ("owned fault",))
                self.assertIsNotNone(caught.exception.live.shell)
                assert caught.exception.live.shell is not None
                self.assertIsNone(caught.exception.live.shell.poll())
                self.assertTrue(caught.exception.live.launch_intent)
                competing = os.open(LOCK, os.O_RDWR)
                try:
                    with self.assertRaises(BlockingIOError):
                        fcntl.flock(competing, fcntl.LOCK_EX | fcntl.LOCK_NB)
                finally:
                    os.close(competing)
            finally:
                self.cleanup(caught.exception.live)

    def test_first_listing_auth_and_publication_membership(self) -> None:
        for stage in ("listing", "publication"):
            with (
                self.subTest(stage=stage),
                self.lane() as fd,
                TemporaryDirectory() as temporary,
            ):
                request = fixture(Path(temporary).resolve(), fd)
                changed: list[str] = []

                def auth(
                    binding: ListingProcess,
                    argv: tuple[str, ...],
                    cwd: Path,
                    environment: dict[str, str],
                    required: dict[str, str],
                    selected: str = stage,
                    observations: list[str] = changed,
                ) -> bytes:
                    raw = process_output(binding, argv, cwd, environment, required)
                    if selected == "listing":
                        (binding.spec.output_root / "extra-empty").mkdir(exist_ok=True)
                        observations.append("listing")
                    return raw

                def publish(
                    path: Path,
                    value: dict[str, Json],
                    selected: str = stage,
                    output: Path = request.template.output,
                    observations: list[str] = changed,
                ) -> None:
                    atomic_json(path, value)
                    if selected == "publication" and path.name == "result.json":
                        (output / "capture/extra-empty").mkdir()
                        observations.append("publication")

                with (
                    patch.object(mux, "process_output", auth),
                    patch.object(mux, "atomic_json", publish),
                    self.assertRaises(mux.DefenceOwnershipUnknown) as caught,
                ):
                    _ = mux.run_defence_owned(request)
                try:
                    self.assertTrue(changed)
                    self.assertEqual(changed[0], stage)
                finally:
                    self.cleanup(caught.exception.live)


class AncestryTests(unittest.TestCase):
    def test_real_subshell_observation_is_distinct_from_executable_identity(
        self,
    ) -> None:
        with TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            spawn = root / "spawn.json"
            plan = root / "plan.json"
            ps = Path("/bin/ps").resolve()
            atomic_json(
                plan, {"spawn": str(spawn), "ps": str(ps), "ps_sha256": file_hash(ps)}
            )
            reader, writer = os.pipe()
            code = "import os,sys; from pathlib import Path; from generic_workspace_capture import read; from generic_mux2_run import record_ancestry; assert os.read(int(sys.argv[1]),1)==b'G'; record_ancestry(read(Path(sys.argv[2])),'cargo-defence',Path(sys.argv[3]))"
            environment = {
                "PATH": "/usr/bin:/bin",
                "PYTHONPATH": str(Path(mux.__file__).parent)
                + ":"
                + str(Path(generic_cargo_child.__file__).parent),
                "PYTHONDONTWRITEBYTECODE": "1",
            }
            child = subprocess.Popen(
                (
                    "/bin/bash",
                    "-c",
                    '(cd "$1" && "$2" -c "$3" "$4" "$5" "$1")',
                    "test",
                    str(root),
                    sys.executable,
                    code,
                    str(reader),
                    str(plan),
                ),
                env=environment,
                pass_fds=(reader,),
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )
            os.close(reader)
            try:
                atomic_json(spawn, {"pid": child.pid})
                self.assertEqual(os.write(writer, b"G"), 1)
                stdout, stderr = child.communicate(timeout=10)
            finally:
                os.close(writer)
                if child.poll() is None:
                    child.terminate()
                    _ = child.communicate(timeout=10)
            self.assertEqual(child.returncode, 0, (stdout, stderr))
            row = read(root / "ancestry.json")
            values = row.field("ps_stdout").text().strip().split(maxsplit=2)
            self.assertEqual(values[0], str(row.field("parent_pid").value))
            self.assertEqual(row.field("outer_pid").value, child.pid)
            if row.field("parent_pid").value != child.pid:
                self.assertEqual(values[1], str(child.pid))
            self.assertEqual(
                row.field("executable_identity_from_comm").value, "NOT_VERIFIED"
            )


ORIGINAL_CHECKED: Final = mux.checked
ORIGINAL_FIXTURE: Final = fixture


class IsolatedMachineryTests(OwnedRuntimeTests):
    """Modified-lock fixture only: explicitly bypass product source admission, never fake hashes."""

    __unittest_skip__: ClassVar[bool] = False

    @override
    @contextmanager
    def lane(self) -> Generator[int]:
        with TemporaryDirectory() as temporary:
            lock = Path(temporary).resolve() / "isolated-lane"
            with lock.open("w") as stream:
                fcntl.flock(stream.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)

                def transformed(
                    base: Path,
                    lane_fd: int,
                    *,
                    fail: int | None = None,
                    bad_validator: bool = False,
                    signal_fail: int | None = None,
                ) -> mux.DefenceRequest:
                    request = ORIGINAL_FIXTURE(
                        base,
                        lane_fd,
                        fail=fail,
                        bad_validator=bad_validator,
                        signal_fail=signal_fail,
                    )
                    path = request.template.root / "tools/v8/mux2_gate_lock.py"
                    source = path.read_text()
                    old = "LOCK: Final = Path('/Users/rexxa/work/mux2-data/.mux2-gpu.lock')"
                    self.assertEqual(source.count(old), 1)
                    _ = path.write_text(
                        source.replace(
                            old, "LOCK: Final = Path(" + repr(str(lock)) + ")"
                        )
                    )
                    inputs = tuple(
                        (name, file_hash(request.template.root / name))
                        for name, _digest in request.template.inputs
                    )
                    return replace(
                        request, template=replace(request.template, inputs=inputs)
                    )

                def machinery_inputs(request: mux.DefenceRequest) -> dict[str, str]:
                    r = request.template
                    self.assertEqual(
                        os.fstat(r.held_lane_fd).st_ino, lock.stat().st_ino
                    )
                    bound = {str(r.root / name): digest for name, digest in r.inputs}
                    bound.update({str(path): digest for path, digest in r.tools})
                    for path, digest in bound.items():
                        self.assertEqual(file_hash(Path(path)), digest, path)
                    return bound

                with (
                    patch.object(mux, "LOCK", lock),
                    patch.object(mux, "checked", machinery_inputs),
                    patch(__name__ + ".fixture", transformed),
                    patch(__name__ + ".LOCK", lock),
                ):
                    yield stream.fileno()

    def test_spawn_publication_cannot_add_outer_member(self) -> None:
        with self.lane() as fd, TemporaryDirectory() as temporary:
            request = fixture(Path(temporary).resolve(), fd)
            injected: list[Path] = []

            def publish(path: Path, value: dict[str, Json]) -> None:
                atomic_json(path, value)
                if path.name == "spawn.json":
                    (path.parent / "extra-empty").mkdir()
                    injected.append(path)

            with (
                patch.object(mux, "atomic_json", publish),
                self.assertRaises(mux.DefenceOwnershipUnknown) as caught,
            ):
                _ = mux.run_defence_owned(request)
            try:
                self.assertEqual(
                    injected, [request.template.output / "outer/spawn.json"]
                )
                self.assertEqual(
                    caught.exception.original.args,
                    ("CONTRACT_REJECTED /outputs: changed outer namespace",),
                )
                assert caught.exception.live.shell is not None
                self.assertIsNone(caught.exception.live.shell.poll())
            finally:
                self.cleanup(caught.exception.live)

    def test_transformed_checker_is_not_admitted_as_original_source(self) -> None:
        with self.lane() as fd, TemporaryDirectory() as temporary:
            request = fixture(Path(temporary).resolve(), fd)
            with self.assertRaisesRegex(
                ValueError, "original helper dependency differs"
            ):
                _ = ORIGINAL_CHECKED(request)
            self.assertFalse(request.template.output.exists())


class InitialAnchorRegressionTests(unittest.TestCase):
    def test_spawn_and_first_capture_mutations_cannot_be_adopted(self) -> None:
        for vector in ("spawn", "capture"):
            with self.subTest(vector=vector):
                self.reject_mutation(vector)

    def reject_mutation(self, vector: str) -> None:
        case=IsolatedMachineryTests()
        with case.lane() as fd, TemporaryDirectory() as temporary:
            request=fixture(Path(temporary).resolve(),fd)
            original=mux.recheck
            changed: list[Path]=[]
            def callback(live: mux.Live) -> None:
                if not changed and vector=="spawn" and live.shell is not None:
                    path=live.spec.output_root/"spawn.json"
                    _=path.write_bytes(b'{"unbound_extra":"late",'+path.read_bytes()[1:])
                    changed.append(path)
                elif not changed and vector=="capture":
                    for path in (request.template.output/"capture").iterdir():
                        if (path/"native/000001/receipt.json").exists() and path.name not in live.completed:
                            (path/"late-empty").mkdir();changed.append(path);break
                original(live)
            failure: mux.DefenceOwnershipUnknown | None=None
            try:
                with patch.object(mux,"recheck",callback):
                    with self.assertRaises(mux.DefenceOwnershipUnknown) as caught:
                        _=mux.run_defence_owned(request)
                    failure=caught.exception
                self.assertTrue(changed)
                self.assertIsInstance(failure.original,GateContractError)
                assert isinstance(failure.original,GateContractError)
                self.assertEqual(failure.original.path,"/spawn" if vector=="spawn" else "/outputs")
                self.assertFalse((request.template.output/"result.json").exists())
            finally:
                if failure is not None:case.cleanup(failure.live)
