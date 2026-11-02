"""Closed step37 original helper execution; product and world qualification remain pending."""

import fcntl
import hashlib
import json
import os
import select
import shlex
import shutil
import signal
import stat
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import BinaryIO, Final, Literal

import generic_cargo_child as capture
from gate_oracle_child import suite_send
from gate_test_summary import SummaryError
from generic_gate import GateContractError, document, require
from generic_library_run import Library49Request
from generic_m1_run import frozen
from generic_process import (
    ProcessSpec,
    atomic_json,
    environment_for,
    execute,
    file_hash,
)
from generic_selected_cases import Selection, reconcile_selected
from generic_workspace_capture import (
    CapturedProcess,
    CapturePlan,
    captured_process,
    read,
    snapshot,
)
from generic_workspace_cases import ListingProcess, parse_listing, process_output
from generic_workspace_tests import Executable, Inventory
from mux2_gate_inputs import Json, Node
from mux2_gate_lock import FD_ENV, LOCK

TARGETS: Final = (
    "steward_defence",
    "steward_defence_fingerprint",
    "colony_flee_mux2",
    "colony_wildlife",
    "mux2_firewood",
    "mux2_fuel_sleep",
    "colony_steward",
    "steward_defence_migration",
    "legacy_migration_current",
    "b5_flora",
)
B5_CASE: Final = "actual_h_format9_fixture_loads_as_oak_then_resaves_as_current_format"
CARGO_PHASES: Final = ("defence", "migration10", "legacy", "flora")
PHASES: Final = (
    "cargo-defence",
    "validator",
    "cargo-migration10",
    "cargo-legacy",
    "cargo-flora",
)
PRODUCERS: Final = (*PHASES, *TARGETS)
HELPER_SHA: Final = "527313c36d44e0f462a1986d181718c370ca819e282fbbf7e6f029982222471c"


def cargo_arguments() -> tuple[tuple[str, ...], ...]:
    base = ("test", "--release", "-p", "sim-test")
    return (
        base
        + tuple(part for target in TARGETS[:7] for part in ("--test", target))
        + ("--", "--test-threads=8"),
        base + ("--test", TARGETS[7], "--", "--ignored"),
        base + ("--test", TARGETS[8], "--", "--ignored"),
        base + ("--test", TARGETS[9], B5_CASE, "--", "--ignored", "--exact"),
    )


def phases(v10: Path, manifest: Path) -> tuple[tuple[str, ...], ...]:
    commands = cargo_arguments()
    return (
        commands[0],
        ("tools/v8/mux2_gate_fixtures.py", str(v10), str(manifest)),
        *commands[1:],
    )


def selection(index: int) -> Selection:
    require(0 <= index < 10, "/selection", "unknown target")
    if index < 7:
        return Selection("default")
    if index < 9:
        return Selection("ignored-only")
    return Selection("exact-ignored", B5_CASE)


def native_args(index: int) -> tuple[str, ...]:
    if index < 7:
        return ("--test-threads=8",)
    if index < 9:
        return ("--ignored",)
    return (B5_CASE, "--ignored", "--exact")


@dataclass(frozen=True, slots=True)
class DefenceRequest:
    template: Library49Request
    executables: tuple[Executable, ...]
    legacy: Path
    h9: Path
    v10: Path
    manifest: Path


@dataclass(frozen=True, slots=True)
class DefenceResult:
    receipt: Path
    status: Literal["PASS", "FAIL"]
    qualification: Literal["PENDING"] = "PENDING"


@dataclass(slots=True)
class Live:
    """Actual parent and live descriptors survive every post-intent ambiguous failure."""

    request: DefenceRequest
    spec: ProcessSpec
    bound: dict[str, str]
    plan: Path
    controls: dict[str, int]
    stdout: BinaryIO
    stderr: BinaryIO
    shell: subprocess.Popen[bytes] | None = None
    launch_intent: bool = False
    spawn_expected: bytes = b""
    completed: list[str] = field(default_factory=list)
    observed: dict[str, CapturedProcess] = field(default_factory=dict)
    anchors: dict[Path, tuple[dict[str, str], tuple[str, ...]]] = field(
        default_factory=dict
    )
    fixture_anchors: dict[Path, tuple[dict[str, str], tuple[str, ...]]] = field(
        default_factory=dict
    )
    listings: list[ProcessSpec] = field(default_factory=list)
    published: bool = False


class DefenceOwnershipUnknown(RuntimeError):
    def __init__(self, live: Live, original: BaseException) -> None:
        super().__init__(
            "step37 ownership unknown; retain actual parent, streams and caller FD"
        )
        self.live: Live = live
        self.original: BaseException = original


def checked(request: DefenceRequest) -> dict[str, str]:
    r = request.template
    held = os.fstat(r.held_lane_fd)
    canonical = LOCK.stat()
    require(
        stat.S_ISREG(held.st_mode)
        and (held.st_dev, held.st_ino) == (canonical.st_dev, canonical.st_ino),
        "/lane",
        "actual canonical lane required",
    )
    fcntl.flock(r.held_lane_fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
    require(
        r.root == r.root.resolve()
        and r.output.is_absolute()
        and r.output == r.output.resolve()
        and not r.output.exists()
        and not r.output.is_relative_to(r.root)
        and not r.root.is_relative_to(r.output),
        "/output",
        "fresh external root required",
    )
    require(
        len(request.executables) == 10
        and len({e.binary for e in request.executables}) == 10,
        "/native",
        "ten exact binaries required",
    )
    for index, e in enumerate(request.executables):
        require(
            e.step == 37
            and e.profile == "release"
            and e.target.package == "sim-test"
            and e.target.name == TARGETS[index]
            and e.target.kinds == ("test",)
            and e.target.manifest == r.root / "rust/crates/sim-test/Cargo.toml"
            and e.filter == (B5_CASE if index == 9 else None),
            "/native",
            "wrong original selected artifact",
        )
    bound = {str(r.root / name): digest for name, digest in r.inputs}
    require(
        len(bound) == len(r.inputs) and len(dict(r.tools)) == len(r.tools),
        "/inputs",
        "duplicate input",
    )
    for path, digest in r.tools:
        require(
            str(path) not in bound and not path.is_relative_to(r.root),
            "/inputs",
            "external tools must not overlap root inputs",
        )
        bound[str(path)] = digest
    required = {
        r.cargo,
        r.bash,
        r.python,
        r.helper,
        Path(__file__).resolve(),
        *r.helper.parent.glob("*.py"),
        *(e.binary for e in request.executables),
        *(e.target.source for e in request.executables),
        r.root / "rust/crates/sim-test/Cargo.toml",
        *(
            r.root / "tools/v8" / name
            for name in (
                "gate.sh",
                "mux2_gate_checks.sh",
                "mux2_gate_lock.py",
                "mux2_gate_inputs.py",
                "godot_bin.sh",
                "mux2_gate_fixtures.py",
            )
        ),
    }
    require(
        all(str(path) in bound for path in required),
        "/inputs",
        "missing source/tool closure",
    )
    require(
        r.helper == Path(capture.__file__).resolve(),
        "/helper",
        "capture helper differs",
    )
    for path, digest in bound.items():
        require(file_hash(Path(path)) == digest, path, "declared bytes differ")
    require(
        file_hash(r.root / "tools/v8/mux2_gate_checks.sh") == HELPER_SHA,
        "/source",
        "changed original helper",
    )
    require(
        [
            line
            for line in (r.root / "tools/v8/gate.sh").read_text().splitlines()
            if line.startswith("step 37 ")
        ]
        == [
            'step 37 mux2_defence bash tools/v8/mux2_gate_checks.sh defence "$LOG/mux2-defence"'
        ],
        "/source",
        "changed original gate step",
    )
    for name in (
        "mux2_gate_lock.py",
        "mux2_gate_inputs.py",
        "godot_bin.sh",
        "mux2_gate_fixtures.py",
    ):
        require(
            file_hash(r.root / "tools/v8" / name) == file_hash(r.helper.parent / name),
            "/source",
            "original helper dependency differs",
        )
    for folder in (request.legacy, request.v10):
        require(
            folder.is_dir()
            and folder == folder.resolve()
            and not folder.is_relative_to(r.root),
            "/fixtures",
            "external canonical fixture directory required",
        )
        for path in folder.rglob("*"):
            require(not path.is_symlink(), "/fixtures", "linked fixture member")
            if path.is_file():
                require(
                    bound.get(str(path)) == file_hash(path),
                    "/fixtures",
                    "fixture file unbound",
                )
    for path in (request.h9, request.manifest):
        require(
            not path.is_relative_to(r.root) and bound.get(str(path)) == file_hash(path),
            "/fixtures",
            "unbound external fixture/manifest",
        )
    inventory_h = request.legacy / "b4/h-format9-fixture/tick4000.save"
    require(
        file_hash(inventory_h) == file_hash(request.h9),
        "/fixtures",
        "H fixture differs from legacy inventory",
    )
    allowed = {
        "PATH",
        "HOME",
        "TMPDIR",
        "LANG",
        "LC_ALL",
        "CARGO_HOME",
        "RUSTUP_HOME",
        "RUSTC",
        "RUSTDOC",
        "CARGO_TARGET_DIR",
        "CARGO_BUILD_JOBS",
        "RUSTC_WRAPPER",
        "RUSTC_WORKSPACE_WRAPPER",
        "DYLD_LIBRARY_PATH",
        "LD_LIBRARY_PATH",
        "__CF_USER_TEXT_ENCODING",
    }
    env = dict(r.environment)
    require(
        len(env) == len(r.environment) and set(env) <= allowed,
        "/environment",
        "unapproved override",
    )
    require(
        all(
            env.get(key, "") == ""
            for key in ("RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER")
        ),
        "/environment",
        "nonempty wrapper",
    )
    require(
        bool(env.get("CARGO_BUILD_JOBS")),
        "/environment",
        "bind original gate build jobs explicitly",
    )
    for name in ("bash", "python3", "dirname", "mkdir", "shasum", "ps"):
        located = shutil.which(name, path=env.get("PATH", "/usr/bin:/bin"))
        require(located is not None, "/tool", "missing original tool")
        assert located is not None
        path = Path(located).resolve()
        require(
            bound.get(str(path)) == file_hash(path), "/tool", "unbound resolved tool"
        )
        if name in {"bash", "python3"}:
            require(
                path == (r.bash if name == "bash" else r.python),
                "/tool",
                "different interpreter",
            )
    return bound


def shim(path: Path, digest: str, kind: str, argv: tuple[str, ...]) -> int:
    require(file_hash(path) == digest, "/plan", "plan changed")
    plan = read(path)
    root = Path(plan.field("root").text())
    python = plan.field("python").text()
    if kind == "python3" and argv == ("tools/v8/mux2_gate_lock.py", "--check"):
        require(Path.cwd() == root, "/lock", "wrong checker cwd")
        with Path(plan.field("start_pipe").text()).open("rb", buffering=0) as ready:
            require(
                os.read(ready.fileno(), 1) == b"G", "/start", "outer observer not ready"
            )
        os.execve(python, (python, *argv), dict(os.environ))
    if kind == "cargo":
        matches = [
            name
            for name, args in zip(CARGO_PHASES, cargo_arguments(), strict=True)
            if argv == args
        ]
        require(len(matches) == 1, "/cargo", "unknown original command")
        ident = "cargo-" + matches[0]
        os.environ["M37_STAGE"] = matches[0]
    elif kind == "native":
        matches = [
            name
            for name in TARGETS
            if argv
            == tuple(
                n.text()
                for n in plan.field("producers").field(name).field("argv").array()
            )
        ]
        require(len(matches) == 1, "/native", "unknown selected argv")
        ident = matches[0]
        index = TARGETS.index(ident)
        require(
            os.environ.get("M37_STAGE") == CARGO_PHASES[0 if index < 7 else index - 6],
            "/native",
            "wrong Cargo owner",
        )
    else:
        require(kind == "python3", "/shim", "unknown shim")
        ident = "validator"
    row = plan.field("producers").field(ident)
    expected = tuple(n.text() for n in row.field("argv").array())
    require(
        argv == (expected if kind == "native" else expected[1:])
        and Path.cwd() == Path(row.field("cwd").text()),
        "/shim",
        "wrong source-owned argv/cwd",
    )
    manifest = Path(row.field("manifest").text())
    manifest_hash = row.field("manifest_sha256").text()
    os.environ["CARGO_CAPTURE_MANIFEST"] = str(manifest)
    os.environ["CARGO_CAPTURE_MANIFEST_SHA256"] = manifest_hash
    if ident in PHASES:
        record_ancestry(plan, ident, manifest.parent)
    code = capture.execute(manifest, manifest_hash, expected)
    with (
        Path(plan.field("request_pipe").text()).open("wb", buffering=0) as outgoing,
        Path(plan.field("acks").field(ident).text()).open(
            "rb", buffering=0
        ) as incoming,
    ):
        suite_send(
            outgoing.fileno(),
            {
                "producer": ident,
                "plan_sha256": digest,
                "launcher_pid": os.getpid(),
                "code": code,
            },
        )
        require(
            os.read(incoming.fileno(), 1) == b"G", "/ack", "completion not acknowledged"
        )
    if code < 0:
        if -code not in (signal.SIGKILL, signal.SIGSTOP):
            _ = signal.signal(-code, signal.SIG_DFL)
        os.kill(os.getpid(), -code)
    return code


def record_ancestry(plan: Node, ident: str, output: Path) -> None:
    """Record observed ancestry, not executable identity inferred from process comm."""
    spawn = Path(plan.field("spawn").text())
    outer = read(spawn).field("pid").value
    require(type(outer) is int, "/ancestry", "missing actual outer PID")
    parent = os.getppid()
    ps = Path(plan.field("ps").text())
    digest = plan.field("ps_sha256").text()
    require(file_hash(ps) == digest, "/ancestry", "ps executable changed")
    argv = (str(ps), "-p", str(parent), "-o", "pid=,ppid=,comm=")
    observed = subprocess.run(
        argv,
        stdin=subprocess.DEVNULL,
        capture_output=True,
        check=False,
    )
    require(file_hash(ps) == digest, "/ancestry", "ps changed during observation")
    atomic_json(
        output / "ancestry.json",
        {
            "schema": 1,
            "producer": ident,
            "launcher_pid": os.getpid(),
            "parent_pid": parent,
            "outer_pid": outer,
            "spawn_sha256": file_hash(spawn),
            "ps": str(ps),
            "ps_sha256": digest,
            "ps_argv": list(argv),
            "ps_exit": observed.returncode,
            "ps_stdout": observed.stdout.decode(),
            "ps_stderr": observed.stderr.decode(),
            "executable_identity_from_comm": "NOT_VERIFIED",
        },
    )


def authenticate_ancestry(live: Live, name: str) -> int:
    assert live.shell is not None
    path = live.request.template.output / "capture" / name / "ancestry.json"
    row = read(path)
    require(
        set(row.mapping())
        == {
            "schema",
            "producer",
            "launcher_pid",
            "parent_pid",
            "outer_pid",
            "spawn_sha256",
            "ps",
            "ps_sha256",
            "ps_argv",
            "ps_exit",
            "ps_stdout",
            "ps_stderr",
            "executable_identity_from_comm",
        },
        "/ancestry",
        "unexpected witness fields",
    )
    row.field("schema").equals(1)
    row.field("producer").equals(name)
    row.field("outer_pid").equals(live.shell.pid)
    row.field("spawn_sha256").equals(hashlib.sha256(live.spawn_expected).hexdigest())
    parent = row.field("parent_pid").value
    launcher = row.field("launcher_pid").value
    require(
        type(parent) is int and type(launcher) is int, "/ancestry", "invalid actual PID"
    )
    assert type(parent) is int
    plan = read(live.plan)
    ps = Path(plan.field("ps").text())
    row.field("ps").equals(str(ps))
    row.field("ps_sha256").equals(plan.field("ps_sha256").value)
    require(file_hash(ps) == row.field("ps_sha256").text(), "/ancestry", "changed ps")
    row.field("ps_argv").equals([str(ps), "-p", str(parent), "-o", "pid=,ppid=,comm="])
    row.field("ps_exit").equals(0)
    row.field("ps_stderr").equals("")
    values = row.field("ps_stdout").text().strip().split(maxsplit=2)
    require(
        len(values) == 3 and values[0] == str(parent) and values[1].isdigit(),
        "/ancestry",
        "unresolved ps observation",
    )
    if parent != live.shell.pid:
        require(
            name.startswith("cargo-") and int(values[1]) == live.shell.pid,
            "/ancestry",
            "unjoined intermediate process",
        )
    row.field("executable_identity_from_comm").equals("NOT_VERIFIED")
    pending = read(path.parent / "native/000001/pending.json")
    pending.field("launcher_pid").equals(launcher)
    pending.field("parent_pid").equals(parent)
    return parent


def prepare(request: DefenceRequest, bound: dict[str, str]) -> Live:
    r = request.template
    r.output.mkdir()
    for name in ("control", "shim", "logs", "outer"):
        (r.output / name).mkdir()
    plan = r.output / "plan.json"
    module = Path(__file__).resolve()
    controls: dict[str, int] = {}
    acks: dict[str, Json] = {}
    for kind in ("cargo", "native", "python3"):
        path = r.output / "shim" / kind
        _ = path.write_text(
            "#!"
            + str(r.bash)
            + "\nexec "
            + shlex.join((str(r.python), "-B", str(module), "--shim", str(plan)))
            + ' "$M37_PLAN_SHA256" '
            + shlex.quote(kind)
            + ' "$@"\n'
        )
        path.chmod(0o755)
        bound[str(path)] = file_hash(path)
    for name in ("request", "start", *PRODUCERS):
        path = r.output / "control" / (name + ".fifo")
        os.mkfifo(path, 0o600)
        if name not in {"request", "start"}:
            acks[name] = str(path)
    producers: dict[str, Json] = {}
    for name in PRODUCERS:
        folder = r.output / "capture" / name
        folder.mkdir(parents=True)
        native = folder / "native"
        native.mkdir()
        if name.startswith("cargo-"):
            argv = (
                str(r.cargo),
                *cargo_arguments()[CARGO_PHASES.index(name.removeprefix("cargo-"))],
            )
            cwd = r.root / "rust"
            observer = False
        elif name == "validator":
            argv = (
                str(r.python),
                "tools/v8/mux2_gate_fixtures.py",
                str(request.v10),
                str(request.manifest),
            )
            cwd = r.root
            observer = False
        else:
            index = TARGETS.index(name)
            binary = request.executables[index]
            argv = (str(binary.binary), *native_args(index))
            cwd = binary.cwd
            observer = True
        manifest = folder / "manifest.json"
        atomic_json(
            manifest,
            {
                "schema_version": 1,
                "invocation_id": r.invocation,
                "step": 37,
                "arm": r.arm,
                "profile": "release",
                "output_root": str(native),
                "launcher_sha256": bound[str(r.helper)],
                "python_sha256": bound[str(r.python)],
                "inputs": {
                    path: digest for path, digest in bound.items() if path != argv[0]
                },
                "artifacts": [{"id": name, "path": argv[0], "sha256": bound[argv[0]]}],
                "observer_variable": "WORLDSIM_GATE_TEST_OUT" if observer else None,
            },
        )
        producers[name] = {
            "argv": list(argv),
            "cwd": str(cwd),
            "manifest": str(manifest),
            "manifest_sha256": file_hash(manifest),
        }
    atomic_json(
        plan,
        {
            "schema": 1,
            "root": str(r.root),
            "python": str(r.python),
            "producers": producers,
            "request_pipe": str(r.output / "control/request.fifo"),
            "start_pipe": str(r.output / "control/start.fifo"),
            "spawn": str(r.output / "outer/spawn.json"),
            "ps": str(
                Path(
                    shutil.which(
                        "ps", path=dict(r.environment).get("PATH", "/usr/bin:/bin")
                    )
                    or ""
                ).resolve()
            ),
            "ps_sha256": bound[
                str(
                    Path(
                        shutil.which(
                            "ps", path=dict(r.environment).get("PATH", "/usr/bin:/bin")
                        )
                        or ""
                    ).resolve()
                )
            ],
            "acks": acks,
        },
    )
    env = dict(r.environment)
    env.update(
        {
            "PATH": str(r.output / "shim") + ":" + env.get("PATH", "/usr/bin:/bin"),
            "PYTHONPATH": str(r.helper.parent),
            "PYTHONDONTWRITEBYTECODE": "1",
            "M37_PLAN_SHA256": file_hash(plan),
            FD_ENV: str(r.held_lane_fd),
            "CARGO_TARGET_" + r.host.upper().replace("-", "_") + "_RUNNER": str(
                r.output / "shim/native"
            ),
            "WORLDSIM_MUX_FIXTURES": str(request.legacy),
            "WORLDSIM_B5_LEGACY9": str(request.h9),
            "MUX2_V10_FIXTURES": str(request.v10),
            "MUX2_V10_MANIFEST": str(request.manifest),
        }
    )
    external = {
        path: digest
        for path, digest in bound.items()
        if not Path(path).is_relative_to(r.root)
    }
    external[str(plan)] = file_hash(plan)
    for name in PRODUCERS:
        p = r.output / "capture" / name / "manifest.json"
        external[str(p)] = file_hash(p)
    spec = ProcessSpec(
        (
            str(r.bash),
            "tools/v8/mux2_gate_checks.sh",
            "defence",
            str(r.output / "logs"),
        ),
        r.root,
        r.bash,
        bound[str(r.bash)],
        r.root,
        r.inputs,
        r.output / "outer",
        tuple(sorted(env.items())),
        tuple((Path(p), h) for p, h in sorted(external.items())),
    )
    stdout = (spec.output_root / "stdout.log").open("xb")
    try:
        stderr = (spec.output_root / "stderr.log").open("xb")
        try:
            for name in ("request", "start", *PRODUCERS):
                controls[name] = os.open(
                    r.output / "control" / (name + ".fifo"), os.O_RDWR | os.O_NONBLOCK
                )
        except BaseException:
            stderr.close()
            raise
    except BaseException:
        stdout.close()
        for descriptor in controls.values():
            os.close(descriptor)
        raise
    live = Live(request, spec, bound, plan, controls, stdout, stderr)
    live.fixture_anchors = {
        request.legacy: frozen(request.legacy),
        request.v10: frozen(request.v10),
    }
    return live


def recheck(live: Live) -> None:
    r = live.request.template
    if live.shell is not None:
        require(
            bool(live.spawn_expected)
            and (live.spec.output_root / "spawn.json").read_bytes() == live.spawn_expected,
            "/spawn",
            "spawn bytes changed from prepublication expectation",
        )
    names = {"control", "shim", "logs", "outer", "capture", "plan.json"}
    if live.listings:
        names.add("listings")
    if live.published:
        names.add("result.json")
    require(
        {p.name for p in r.output.iterdir()} == names,
        "/outputs",
        "changed root namespace",
    )
    require(
        {p.name for p in (r.output / "outer").iterdir()}
        == (
            {"stdout.log", "stderr.log", "spawn.json"}
            if live.shell is not None
            else {"stdout.log", "stderr.log"}
        ),
        "/outputs",
        "changed outer namespace",
    )
    require(
        {p.name for p in (r.output / "capture").iterdir()} == set(PRODUCERS),
        "/outputs",
        "changed capture namespace",
    )
    require(
        {p.name for p in (r.output / "shim").iterdir()}
        == {"cargo", "native", "python3"},
        "/outputs",
        "changed shim namespace",
    )
    require(
        not list((r.output / "logs").iterdir()),
        "/outputs",
        "unexpected original defence log output",
    )
    require(
        {p.name for p in (r.output / "control").iterdir()}
        == {name + ".fifo" for name in live.controls},
        "/control",
        "changed pipe namespace",
    )
    for name, descriptor in live.controls.items():
        path = (r.output / "control" / (name + ".fifo")).lstat()
        actual = os.fstat(descriptor)
        require(
            stat.S_ISFIFO(path.st_mode)
            and (path.st_dev, path.st_ino) == (actual.st_dev, actual.st_ino),
            "/control",
            "replaced pipe",
        )
    if live.listings:
        require(
            {p.name for p in (r.output / "listings").iterdir()}
            == {s.output_root.parent.name for s in live.listings},
            "/outputs",
            "changed listing namespace",
        )
        for parent in {s.output_root.parent for s in live.listings}:
            require(
                {p.name for p in parent.iterdir()}
                == {
                    s.output_root.name
                    for s in live.listings
                    if s.output_root.parent == parent
                },
                "/outputs",
                "changed listing members",
            )
    for path, anchor in (*live.anchors.items(), *live.fixture_anchors.items()):
        require(
            frozen(path) == anchor, "/outputs", "frozen bytes or membership changed"
        )


def joined(live: Live, name: str) -> CapturedProcess:
    r = live.request.template
    row = read(live.plan).field("producers").field(name)
    manifest = Path(row.field("manifest").text())
    root = manifest.parent / "native"
    argv = tuple(n.text() for n in row.field("argv").array())
    entry = read(manifest).field("artifacts").array()[0]
    assert live.shell is not None
    parent = live.shell.pid
    if name in PHASES:
        parent = authenticate_ancestry(live, name)
    if name in TARGETS:
        index = TARGETS.index(name)
        ancestor = "cargo-" + CARGO_PHASES[0 if index < 7 else index - 6]
        value = (
            read(r.output / "capture" / ancestor / "native/000001/started.json")
            .field("child_pid")
            .value
        )
        require(type(value) is int, "/parent", "missing actual Cargo PID")
        assert type(value) is int
        parent = value
    plan = CapturePlan(
        r.invocation,
        37,
        r.arm,
        "release",
        live.spec,
        "",
        manifest,
        row.field("manifest_sha256").text(),
        Inventory((), ()),
        root,
        (),
    )
    child = captured_process(
        plan,
        parent,
        entry,
        Path(argv[0]),
        Path(row.field("cwd").text()),
        argv,
        root / "000001",
    )
    require(
        child.capture_complete
        and child.native_exit is not None
        and (child.native_exit >= 0 or name in TARGETS),
        "/capture",
        "missing normal actual wait",
    )
    claim = "artifact-" + hashlib.sha256(argv[0].encode()).hexdigest() + ".claim"
    require(
        {p.name for p in root.iterdir()} == {"sequence.lock", "000001", claim}
        and (root / "sequence.lock").read_bytes() == b"1",
        "/capture",
        "extra occurrence or claim",
    )
    return child


def service(live: Live) -> None:
    assert live.shell is not None
    buffer: bytes = b""
    while True:
        ready, _, _ = select.select((live.controls["request"],), (), (), 0.05)
        if ready:
            chunk: bytes = os.read(live.controls["request"], 4096)
            buffer = buffer + chunk
            require(len(buffer) <= 8192, "/frame", "oversized frame")
        while b"\n" in buffer:
            parts = buffer.split(b"\n", 1)
            frame = document(parts[0].decode())
            buffer = parts[1]
            require(
                set(frame.mapping())
                == {"producer", "plan_sha256", "launcher_pid", "code"},
                "/frame",
                "unexpected fields",
            )
            name = frame.field("producer").text()
            require(
                name in PRODUCERS and name not in live.completed,
                "/frame",
                "unexpected or repeated producer",
            )
            path = live.request.template.output / "capture" / name
            live.anchors[path] = frozen(path)
            frame.field("plan_sha256").equals(file_hash(live.plan))
            recheck(live)
            child = joined(live, name)
            frame.field("launcher_pid").equals(child.launcher_pid)
            frame.field("code").equals(child.native_exit)
            recheck(live)
            live.observed[name] = child
            live.completed.append(name)
            require(os.write(live.controls[name], b"G") == 1, "/ack", "short ACK")
        if live.shell.poll() is not None:
            require(not buffer, "/frame", "truncated frame")
            _ = live.shell.wait()
            return


def run_defence_owned(request: DefenceRequest) -> DefenceResult:
    live = prepare(request, checked(request))
    try:
        recheck(live)
        for path, digest in live.bound.items():
            require(
                file_hash(Path(path)) == digest,
                "/inputs",
                "input changed before launch",
            )
        live.launch_intent = True
        live.shell = subprocess.Popen(
            live.spec.argv,
            cwd=live.spec.cwd,
            env=environment_for(live.spec),
            stdin=subprocess.DEVNULL,
            stdout=live.stdout,
            stderr=live.stderr,
            pass_fds=(request.template.held_lane_fd,),
        )
        spawn_value: dict[str, Json] = {
            "pid": live.shell.pid,
            "argv": list(live.spec.argv),
            "lane_fd": request.template.held_lane_fd,
        }
        live.spawn_expected = (
            json.dumps(spawn_value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode()
        atomic_json(live.spec.output_root / "spawn.json", spawn_value)
        require(
            read(live.spec.output_root / "spawn.json").value == spawn_value,
            "/spawn",
            "spawn record changed during publication",
        )
        recheck(live)
        require(
            os.write(live.controls["start"], b"G") == 1, "/start", "short startup ACK"
        )
        service(live)
        require(
            live.shell.returncode is not None and live.shell.returncode >= 0,
            "/shell",
            "signal or missing actual wait",
        )
        live.stdout.close()
        live.stderr.close()
        live.anchors[live.spec.output_root] = frozen(live.spec.output_root)
        return finish(live)
    except BaseException as failure:
        raise DefenceOwnershipUnknown(live, failure) from failure


def finish(live: Live) -> DefenceResult:
    r = live.request.template
    assert live.shell is not None
    require(live.shell.returncode is not None, "/shell", "no actual parent wait")
    observed = {name: joined(live, name) for name in live.completed}
    recheck(live)
    actual_phases = [name for name in live.completed if name in PHASES]
    require(
        actual_phases == list(PHASES[: len(actual_phases)]),
        "/chain",
        "non-prefix phase order",
    )
    require(
        all(observed[name].native_exit == 0 for name in actual_phases[:-1]),
        "/chain",
        "continued after failed phase",
    )
    for ordinal, phase in enumerate(PHASES):
        names = (
            TARGETS[:7]
            if ordinal == 0
            else ((TARGETS[ordinal + 5],) if ordinal >= 2 else ())
        )
        if phase in observed and observed[phase].native_exit == 0:
            require(
                all(name in observed for name in names),
                "/coverage",
                "successful Cargo omitted target",
            )
        if phase not in observed:
            require(
                not any(name in observed for name in names),
                "/coverage",
                "native without completed Cargo owner",
            )
    if live.shell.returncode == 0:
        require(
            actual_phases == list(PHASES)
            and all(row.native_exit == 0 for row in observed.values()),
            "/chain",
            "shell success omitted or failed child",
        )
    for name in PRODUCERS:
        path = r.output / "capture" / name
        if name not in observed:
            require(
                {p.name for p in path.iterdir()} == {"manifest.json", "native"}
                and not list((path / "native").iterdir()),
                "/capture",
                "unjoined evidence",
            )
    if actual_phases:
        expected_stdout = b"MUX2_GATE_LOCK_HELD\n" + b"".join(
            (observed[name].receipt.parent / "stdout.log").read_bytes()
            for name in actual_phases
        )
        expected_stderr = b"".join(
            (observed[name].receipt.parent / "stderr.log").read_bytes()
            for name in actual_phases
        )
        require(
            (live.spec.output_root / "stdout.log").read_bytes() == expected_stdout
            and (live.spec.output_root / "stderr.log").read_bytes() == expected_stderr,
            "/outer",
            "original source-order streams differ",
        )
    live.anchors[r.output / "capture"] = frozen(r.output / "capture")
    cases: list[Json] = []
    failed = live.shell.returncode != 0
    required = {str(r.root / n): h for n, h in live.spec.inputs}
    required.update({str(p): h for p, h in live.spec.external_inputs})
    for name in live.completed:
        if name not in TARGETS:
            continue
        index = TARGETS.index(name)
        binary = live.request.executables[index]
        native = observed[name]
        env = read(native.receipt).field("environment")
        environment = tuple((k, env.field(k).text()) for k in sorted(env.mapping()))
        raw_listings: list[bytes] = []
        for label, args in (
            ("full", ("--list",)),
            ("ignored", ("--list", "--ignored")),
        ):
            recheck(live)
            out = r.output / "listings" / name / label
            out.parent.mkdir(parents=True, exist_ok=True)
            spec = ProcessSpec(
                (str(binary.binary), *args),
                binary.cwd,
                binary.binary,
                required[str(binary.binary)],
                r.root,
                r.inputs,
                out,
                environment,
                live.spec.external_inputs,
            )
            live.listings.append(spec)
            result = execute(spec)
            live.anchors[out] = frozen(out)
            binding = ListingProcess(spec, result, file_hash(result.receipt))
            require(result.native_exit == 0, "/listing", "listing failed")
            raw_listings.append(
                process_output(
                    binding, spec.argv, spec.cwd, environment_for(spec), required
                )
            )
            recheck(live)
        full, ignored = map(parse_listing, raw_listings)
        try:
            section = reconcile_selected(
                (native.receipt.parent / "stdout.log").read_bytes(),
                full,
                ignored,
                selection(index),
            )
        except (GateContractError, SummaryError) as error:
            if native.native_exit == 0:
                raise
            failed = True
            match error:
                case GateContractError():
                    code, location = error.code, error.path
                case SummaryError():
                    code, location = "LIBTEST_SUMMARY", "/native/stdout"
            cases.append(
                {
                    "producer": name,
                    "pid": native.child_pid,
                    "native_exit": native.native_exit,
                    "case_accounting": "INCOMPLETE_FAILED_PROCESS",
                    "error_code": code,
                    "error_path": location,
                    "raw_stdout_sha256": file_hash(
                        native.receipt.parent / "stdout.log"
                    ),
                    "world_safety": "PENDING",
                }
            )
            continue
        require(
            native.native_exit != 0 or section.failed == 0,
            "/native",
            "native zero contradicts failed cases",
        )
        failed = failed or native.native_exit != 0 or section.failed != 0
        cases.append(
            {
                "producer": name,
                "pid": native.child_pid,
                "native_exit": native.native_exit,
                "selected": [row.name for row in section.outcomes],
                "ignored": [
                    row.name for row in section.outcomes if row.status == "ignored"
                ],
                "filtered": len(full.names) - len(section.outcomes),
                "world_safety": "PENDING",
            }
        )
    recheck(live)
    for path, digest in required.items():
        require(
            file_hash(Path(path)) == digest,
            "/inputs",
            "input changed before publication",
        )
    for name, descriptor in live.controls.items():
        os.close(descriptor)
        (r.output / "control" / (name + ".fifo")).unlink()
    live.controls.clear()
    recheck(live)
    status: Literal["PASS", "FAIL"] = "FAIL" if failed else "PASS"
    receipt = r.output / "result.json"
    live.published = True
    value: dict[str, Json] = {
        "schema": 1,
        "component": "MUX2_DEFENCE_RUNTIME",
        "step": 37,
        "invocation": r.invocation,
        "status": status,
        "caller": {
            "argv": list(live.spec.argv),
            "cwd": str(live.spec.cwd),
            "environment": {k: v for k, v in environment_for(live.spec).items()},
            "pass_fds": [r.held_lane_fd],
            "pid": live.shell.pid,
            "native_exit": live.shell.returncode,
            "wait_observed": True,
        },
        "completed_producers": list(live.completed),
        "skipped_producers": [name for name in PRODUCERS if name not in observed],
        "cases": cases,
        "output_inventory": {k: v for k, v in snapshot(r.output).items()},
        "output_members": list(frozen(r.output)[1]),
        "source_build_qualification": "PENDING",
        "observer_qualification": "PENDING",
        "fixture_provenance": "PENDING",
        "world_safety": "PENDING",
        "publication_authority": False,
    }
    atomic_json(receipt, value)
    require(
        read(receipt).value == value, "/receipt", "terminal changed during publication"
    )
    recheck(live)
    for path, digest in required.items():
        require(
            file_hash(Path(path)) == digest,
            "/inputs",
            "input changed after publication",
        )
    return DefenceResult(receipt, status)


if __name__ == "__main__":
    require(
        len(sys.argv) >= 5 and sys.argv[1] == "--shim",
        "/argv",
        "only closed shim entrypoint",
    )
    raise SystemExit(
        shim(Path(sys.argv[2]), sys.argv[3], sys.argv[4], tuple(sys.argv[5:]))
    )
