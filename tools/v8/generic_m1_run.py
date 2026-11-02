"""Exact M1 shell chain with retained ownership; qualification is separately pending."""

import fcntl
import hashlib
import os
import select
import shlex
import shutil
import signal
import stat
import sys
import threading
from dataclasses import dataclass, field
from pathlib import Path
from typing import Final, Literal

import generic_cargo_child as capture
from gate_oracle_child import suite_send
from generic_gate import document, require
from generic_library_run import Library49Request
from generic_m1_commands import COMMANDS, authenticate_source
from generic_process import (
    ProcessResult,
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
from mux2_gate_inputs import Json
from mux2_gate_lock import LOCK

CARGO_NAMES: Final = tuple(command.identity for command in COMMANDS)
NATIVE_NAMES: Final = (*COMMANDS[0].targets, "probe", "jobs", "colony")
PRODUCERS: Final = (
    *("cargo-" + name for name in CARGO_NAMES),
    *NATIVE_NAMES,
    "bounded",
    "godot",
)


@dataclass(frozen=True, slots=True)
class M1Request:
    template: Library49Request
    executables: tuple[Executable, ...]
    godot: Path


@dataclass(frozen=True, slots=True)
class M1Result:
    receipt: Path
    status: Literal["PASS", "FAIL"]
    qualification: Literal["PENDING"] = "PENDING"


@dataclass(slots=True)
class Live:
    """Retains the real worker, request, lane, and control endpoints on UNKNOWN."""

    request: M1Request
    spec: ProcessSpec
    descriptors: dict[str, int]
    thread: threading.Thread | None = None
    result: ProcessResult | None = None
    failure: BaseException | None = None
    anchors: dict[Path, tuple[dict[str, str], tuple[str, ...]]] = field(
        default_factory=dict
    )
    completed: list[str] = field(default_factory=list)
    issued: list[ProcessSpec] = field(default_factory=list)
    published: bool = False


class M1OwnershipUnknown(RuntimeError):
    def __init__(self, live: Live, original: BaseException) -> None:
        super().__init__("M1 completion unknown; retain actual caller FD and worker")
        self.live: Live = live
        self.original: BaseException = original


def frozen(path: Path) -> tuple[dict[str, str], tuple[str, ...]]:
    require(
        path.is_dir() and not path.is_symlink(),
        "/outputs",
        "missing or linked output directory",
    )
    members = tuple(sorted(str(p.relative_to(path)) for p in path.rglob("*")))
    return snapshot(path) if members else {}, members


def namespace(live: Live) -> None:
    root = live.request.template.output
    names = {"capture", "control", "logs", "outer", "plan.json", "shim"}
    if live.issued:
        names.add("listings")
    if live.published:
        names.add("result.json")
    require(
        {p.name for p in root.iterdir()} == names,
        "/outputs",
        "unexpected output namespace",
    )
    require(
        {p.name for p in (root / "capture").iterdir()} == set(PRODUCERS),
        "/outputs",
        "capture namespace changed",
    )
    require(
        {p.name for p in (root / "shim").iterdir()}
        == {"cargo", "native", "bash", "godot"},
        "/outputs",
        "shim namespace changed",
    )
    require(
        {p.name for p in (root / "control").iterdir()}
        == {name + ".fifo" for name in live.descriptors},
        "/outputs",
        "control namespace changed",
    )
    for name, descriptor in live.descriptors.items():
        path = (root / "control" / (name + ".fifo")).lstat()
        held = os.fstat(descriptor)
        require(
            stat.S_ISFIFO(path.st_mode)
            and (path.st_dev, path.st_ino) == (held.st_dev, held.st_ino),
            "/control",
            "pipe replaced",
        )
    if live.issued:
        require(
            {p.name for p in (root / "listings").iterdir()}
            == {spec.output_root.parent.name for spec in live.issued},
            "/outputs",
            "listing namespace changed",
        )
        for parent in {spec.output_root.parent for spec in live.issued}:
            require(
                {p.name for p in parent.iterdir()}
                == {
                    spec.output_root.name
                    for spec in live.issued
                    if spec.output_root.parent == parent
                },
                "/outputs",
                "listing child namespace changed",
            )


def recheck(live: Live) -> None:
    namespace(live)
    for path, value in live.anchors.items():
        require(
            frozen(path) == value,
            "/outputs",
            "completed producer bytes or membership changed",
        )


def checked(request: M1Request) -> dict[str, str]:
    r = request.template
    held = os.fstat(r.held_lane_fd)
    canonical = LOCK.stat()
    require(
        stat.S_ISREG(held.st_mode)
        and (held.st_dev, held.st_ino) == (canonical.st_dev, canonical.st_ino),
        "/lane",
        "actual caller lane required",
    )
    fcntl.flock(r.held_lane_fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
    require(
        not r.output.exists()
        and r.output.is_absolute()
        and r.output == r.output.resolve()
        and not r.output.is_relative_to(r.root)
        and not r.root.is_relative_to(r.output),
        "/output",
        "fresh external output required",
    )
    require(
        len(request.executables) == 7
        and len({e.binary for e in request.executables}) == 7,
        "/native",
        "exact seven binaries required",
    )
    for index, e in enumerate(request.executables):
        command = COMMANDS[0 if index < 4 else index - 3]
        name = (
            COMMANDS[0].targets[index]
            if index < 4
            else command.package.replace("-", "_")
        )
        require(
            e.step == 17
            and e.profile == "release"
            and e.target.package == command.package
            and e.target.name == name
            and e.target.kinds == (("test",) if index < 4 else ("lib",))
            and e.filter == command.filter,
            "/native",
            "wrong closed executable",
        )
        require(
            e.target.manifest
            == r.root / "rust/crates" / command.package / "Cargo.toml",
            "/native",
            "wrong package cwd",
        )
    bound = {str(r.root / name): digest for name, digest in r.inputs}
    require(
        len(bound) == len(r.inputs) and len(dict(r.tools)) == len(r.tools),
        "/inputs",
        "duplicate input/tool",
    )
    for path, digest in r.tools:
        require(
            str(path) not in bound and not path.is_relative_to(r.root),
            "/inputs",
            "tools must be external; declare repository paths in inputs",
        )
        bound[str(path)] = digest
    required = {
        r.cargo,
        r.bash,
        r.python,
        r.helper,
        request.godot,
        Path(__file__).resolve(),
        r.root / "tools/v8/gate.sh",
        r.root / "tools/v8/run_bounded.sh",
        r.root / "tools/v8/test_locale_trf.gd",
        *r.helper.parent.glob("*.py"),
        *(e.binary for e in request.executables),
        *(e.target.source for e in request.executables),
        *(e.target.manifest for e in request.executables),
    }
    require(
        all(str(path) in bound for path in required),
        "/inputs",
        "missing declared source/tool closure",
    )
    require(
        r.helper == Path(capture.__file__).resolve(),
        "/helper",
        "loaded capture helper differs",
    )
    for path, digest in bound.items():
        require(file_hash(Path(path)) == digest, path, "input differs")
    _ = authenticate_source(r.root / "tools/v8/gate.sh")
    require(
        file_hash(r.root / "tools/v8/run_bounded.sh")
        == file_hash(r.helper.parent / "run_bounded.sh"),
        "/wrapper",
        "unreviewed bounded source",
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
    for name in ("bash", "python3", "cat", "grep"):
        selected = shutil.which(
            name, path=env.get("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
        )
        require(selected is not None, "/tool", "missing tool")
        assert selected is not None
        path = Path(selected).resolve()
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


def shim(plan_path: Path, digest: str, kind: str, argv: tuple[str, ...]) -> int:
    """Closed source-owned capture entrypoint; ACK is required before returning to Bash/Cargo."""
    require(file_hash(plan_path) == digest, "/plan", "changed M1 plan")
    plan = read(plan_path)
    root = Path(plan.field("root").text())
    require(
        Path.cwd()
        in {
            root,
            root / "rust",
            *(
                root / "rust/crates" / name
                for name in ("sim-test", "sim-systems", "sim-core")
            ),
        },
        "/cwd",
        "foreign shim cwd",
    )
    if kind == "cargo":
        matches = [command.identity for command in COMMANDS if argv == command.argv[1:]]
        require(len(matches) == 1, "/cargo", "not original command")
        ident = "cargo-" + matches[0]
        os.environ["M1_STAGE"] = matches[0]
    elif kind == "native":
        matches = [
            name
            for name in NATIVE_NAMES
            if tuple(
                row.text()
                for row in plan.field("producers").field(name).field("argv").array()
            )
            == argv
        ]
        require(len(matches) == 1, "/native", "unregistered native argv")
        ident = matches[0]
        stage = "integration" if ident in COMMANDS[0].targets else ident
        require(os.environ.get("M1_STAGE") == stage, "/native", "wrong Cargo stage")
    elif kind in {"bounded", "godot"}:
        ident = kind
    else:
        require(False, "/shim", "unknown closed shim")
        raise AssertionError
    row = plan.field("producers").field(ident)
    expected = tuple(value.text() for value in row.field("argv").array())
    require(
        argv == expected if kind == "native" else argv == expected[1:],
        "/shim",
        "wrong original recipe",
    )
    manifest = Path(row.field("manifest").text())
    manifest_hash = row.field("manifest_sha256").text()
    os.environ["CARGO_CAPTURE_MANIFEST"] = str(manifest)
    os.environ["CARGO_CAPTURE_MANIFEST_SHA256"] = manifest_hash
    code = capture.execute(manifest, manifest_hash, expected)
    request_pipe = Path(plan.field("request_pipe").text())
    ack = Path(plan.field("acknowledgments").field(ident).text())
    with (
        request_pipe.open("wb", buffering=0) as outgoing,
        ack.open("rb", buffering=0) as incoming,
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
            os.read(incoming.fileno(), 1) == b"G",
            "/control",
            "completion not acknowledged",
        )
    if code < 0:
        if -code not in (signal.SIGKILL, signal.SIGSTOP):
            _ = signal.signal(-code, signal.SIG_DFL)
        os.kill(os.getpid(), -code)
    return code


def prepare(request: M1Request, bound: dict[str, str]) -> tuple[Live, Path]:
    r = request.template
    r.output.mkdir()
    controls = r.output / "control"
    controls.mkdir()
    shims = r.output / "shim"
    shims.mkdir()
    (r.output / "logs").mkdir()
    plan_path = r.output / "plan.json"
    module = Path(__file__).resolve()
    for kind in ("cargo", "native", "bounded", "godot"):
        name = "bash" if kind == "bounded" else kind
        path = shims / name
        _ = path.write_text(
            "#!"
            + str(r.bash)
            + "\nexec "
            + shlex.join((str(r.python), "-B", str(module), "--shim", str(plan_path)))
            + ' "$M1_PLAN_SHA256" '
            + shlex.quote(kind)
            + ' "$@"\n'
        )
        path.chmod(0o755)
        bound[str(path)] = file_hash(path)
    descriptors: dict[str, int] = {}
    pipes: dict[str, Json] = {}
    for name in ("request", *PRODUCERS):
        path = controls / (name + ".fifo")
        os.mkfifo(path, 0o600)
        if name != "request":
            pipes[name] = str(path)
    producers: dict[str, Json] = {}
    for ident in PRODUCERS:
        folder = r.output / "capture" / ident
        folder.mkdir(parents=True)
        if ident.startswith("cargo-"):
            command = next(
                command for command in COMMANDS if ident == "cargo-" + command.identity
            )
            argv = (str(r.cargo), *command.argv[1:])
            cwd = r.root / "rust"
            observe = False
        elif ident in NATIVE_NAMES:
            index = NATIVE_NAMES.index(ident)
            binary = request.executables[index]
            argv = (
                str(binary.binary),
                *(
                    ("--test-threads=8",)
                    if index < 4
                    else (COMMANDS[index - 3].filter or "",)
                ),
            )
            cwd = binary.cwd
            observe = True
        elif ident == "bounded":
            argv = (
                str(r.bash),
                "tools/v8/run_bounded.sh",
                "120",
                str(shims / "godot"),
                "--headless",
                "--path",
                ".",
                "--script",
                "res://tools/v8/test_locale_trf.gd",
            )
            cwd = r.root
            observe = False
        else:
            argv = (
                str(request.godot),
                "--headless",
                "--path",
                ".",
                "--script",
                "res://tools/v8/test_locale_trf.gd",
            )
            cwd = r.root
            observe = False
        manifest = folder / "manifest.json"
        capture_root = folder / "native"
        capture_root.mkdir()
        atomic_json(
            manifest,
            {
                "schema_version": 1,
                "invocation_id": r.invocation,
                "step": 17,
                "arm": r.arm,
                "profile": "release",
                "output_root": str(capture_root),
                "launcher_sha256": bound[str(r.helper)],
                "python_sha256": bound[str(r.python)],
                "inputs": {
                    path: digest for path, digest in bound.items() if path != argv[0]
                },
                "artifacts": [{"id": ident, "path": argv[0], "sha256": bound[argv[0]]}],
                "observer_variable": "WORLDSIM_GATE_TEST_OUT" if observe else None,
            },
        )
        producers[ident] = {
            "manifest": str(manifest),
            "manifest_sha256": file_hash(manifest),
            "argv": list(argv),
            "cwd": str(cwd),
        }
    atomic_json(
        plan_path,
        {
            "schema": 1,
            "root": str(r.root),
            "producers": producers,
            "request_pipe": str(controls / "request.fifo"),
            "acknowledgments": pipes,
        },
    )
    env = dict(r.environment)
    env.update(
        {
            "PATH": str(shims) + ":" + env.get("PATH", "/usr/bin:/bin:/usr/sbin:/sbin"),
            "PYTHONPATH": str(r.helper.parent),
            "PYTHONDONTWRITEBYTECODE": "1",
            "M1_PLAN_SHA256": file_hash(plan_path),
            "GODOT_BIN": str(shims / "godot"),
            "CARGO_TARGET_" + r.host.upper().replace("-", "_") + "_RUNNER": str(
                shims / "native"
            ),
        }
    )
    external = {
        path: digest
        for path, digest in bound.items()
        if not Path(path).is_relative_to(r.root)
    }
    external[str(plan_path)] = file_hash(plan_path)
    for ident in PRODUCERS:
        manifest = r.output / "capture" / ident / "manifest.json"
        external[str(manifest)] = file_hash(manifest)
    spec = ProcessSpec(
        (
            str(r.bash),
            "-c",
            authenticate_source(r.root / "tools/v8/gate.sh"),
            str(r.output / "logs"),
        ),
        r.root,
        r.bash,
        bound[str(r.bash)],
        r.root,
        r.inputs,
        r.output / "outer",
        tuple(sorted(env.items())),
        tuple((Path(path), digest) for path, digest in sorted(external.items())),
    )
    try:
        for name in ("request", *PRODUCERS):
            descriptors[name] = os.open(
                controls / (name + ".fifo"), os.O_RDWR | os.O_NONBLOCK
            )
    except BaseException:
        for descriptor in descriptors.values():
            os.close(descriptor)
        raise
    return Live(request, spec, descriptors), plan_path


def joined(live: Live, plan_path: Path, ident: str) -> CapturedProcess:
    r = live.request.template
    row = read(plan_path).field("producers").field(ident)
    manifest = Path(row.field("manifest").text())
    root = manifest.parent / "native"
    entry = read(manifest).field("artifacts").array()[0]
    if ident.startswith("cargo-") or ident == "bounded":
        parent = read(live.spec.output_root / "process-spawn.json").field("pid").value
        require(type(parent) is int, "/parent", "missing actual Bash PID")
        assert type(parent) is int
    else:
        ancestor = (
            "bounded"
            if ident == "godot"
            else "cargo-" + ("integration" if ident in COMMANDS[0].targets else ident)
        )
        parent = (
            read(r.output / "capture" / ancestor / "native/000001/started.json")
            .field("child_pid")
            .value
        )
        require(type(parent) is int, "/parent", "missing actual producer PID")
        assert type(parent) is int
    plan = CapturePlan(
        r.invocation,
        17,
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
    argv = tuple(value.text() for value in row.field("argv").array())
    result = captured_process(
        plan,
        parent,
        entry,
        Path(argv[0]),
        Path(row.field("cwd").text()),
        argv,
        root / "000001",
    )
    require(
        result.capture_complete
        and result.native_exit is not None
        and result.native_exit >= 0,
        "/capture",
        "missing/non-normal actual wait",
    )
    claim = "artifact-" + hashlib.sha256(argv[0].encode()).hexdigest() + ".claim"
    require(
        {p.name for p in root.iterdir()} == {"sequence.lock", "000001", claim}
        and (root / "sequence.lock").read_bytes() == b"1",
        "/capture",
        "extra or omitted capture",
    )
    return result


def serve(live: Live, plan_path: Path) -> None:
    buffer: bytes = b""
    assert live.thread is not None
    while live.thread.is_alive():
        ready, _, _ = select.select((live.descriptors["request"],), (), (), 0.05)
        if not ready:
            continue
        chunk: bytes = os.read(live.descriptors["request"], 4096)
        buffer = buffer + chunk
        require(len(buffer) <= 8192, "/barrier", "oversized completion frame")
        while b"\n" in buffer:
            parts = buffer.split(b"\n", 1)
            raw = parts[0]
            buffer = parts[1]
            node = document(raw.decode())
            ident = node.field("producer").text()
            require(
                set(node.mapping())
                == {"producer", "plan_sha256", "launcher_pid", "code"},
                "/barrier",
                "unexpected completion fields",
            )
            require(
                ident in PRODUCERS and ident not in live.completed,
                "/barrier",
                "duplicate/unknown producer",
            )
            node.field("plan_sha256").equals(file_hash(plan_path))
            recheck(live)
            folder = live.request.template.output / "capture" / ident
            live.anchors[folder] = frozen(folder)
            if ident == "bounded":
                logs = live.request.template.output / "logs"
                live.anchors[logs] = frozen(logs)
            observed = joined(live, plan_path, ident)
            recheck(live)
            node.field("launcher_pid").equals(observed.launcher_pid)
            node.field("code").equals(observed.native_exit)
            live.completed.append(ident)
            require(
                joined(live, plan_path, ident) == observed,
                "/barrier",
                "producer changed before acknowledgment",
            )
            recheck(live)
            require(
                os.write(live.descriptors[ident], b"G") == 1,
                "/ack",
                "short acknowledgment",
            )
    live.thread.join()
    require(not buffer, "/barrier", "truncated completion request")
    if live.failure is not None:
        raise M1OwnershipUnknown(live, live.failure)


def run_owned(request: M1Request) -> M1Result:
    bound = checked(request)
    live, plan_path = prepare(request, bound)

    def worker() -> None:
        try:
            live.result = execute(live.spec)
        except BaseException as error:  # noqa: BLE001 - retain cause and live worker for caller-owned UNKNOWN
            live.failure = error

    live.thread = threading.Thread(target=worker, name="m1-original-shell")
    try:
        live.thread.start()
        serve(live, plan_path)
        require(live.result is not None, "/outer", "missing actual result")
        assert live.result is not None
        live.anchors[live.spec.output_root] = frozen(live.spec.output_root)
        logs = request.template.output / "logs"
        if logs not in live.anchors:
            live.anchors[logs] = frozen(logs)
        recheck(live)
        parent = ListingProcess(live.spec, live.result, file_hash(live.result.receipt))
        required = {
            str(request.template.root / name): digest
            for name, digest in live.spec.inputs
        }
        required.update(
            {str(path): digest for path, digest in live.spec.external_inputs}
        )
        raw = process_output(
            parent, live.spec.argv, live.spec.cwd, environment_for(live.spec), required
        )
        recheck(live)
        return finish(live, plan_path, parent, raw, required)
    except BaseException as error:
        raise M1OwnershipUnknown(live, error) from error


def finish(
    live: Live,
    plan_path: Path,
    parent: ListingProcess,
    raw: bytes,
    required: dict[str, str],
) -> M1Result:
    r = live.request.template
    observed = {name: joined(live, plan_path, name) for name in live.completed}
    cargos = [name for name in live.completed if name.startswith("cargo-")]
    require(
        bool(cargos)
        and cargos == ["cargo-" + name for name in CARGO_NAMES[: len(cargos)]],
        "/chain",
        "non-prefix Cargo execution",
    )
    require(
        all(observed[name].native_exit == 0 for name in cargos[:-1]),
        "/chain",
        "continued after failed Cargo",
    )
    cargo_success = len(cargos) == 4 and observed[cargos[-1]].native_exit == 0
    require(
        ("bounded" in observed) == cargo_success
        and ("godot" in observed) == cargo_success,
        "/chain",
        "missing or unexpected locale completion",
    )
    for index, command in enumerate(COMMANDS):
        names = COMMANDS[0].targets if index == 0 else (command.identity,)
        if (
            "cargo-" + command.identity in observed
            and observed["cargo-" + command.identity].native_exit == 0
        ):
            require(
                all(name in observed for name in names),
                "/chain",
                "successful Cargo omitted selected native binary",
            )
        else:
            require(
                not any(
                    name in observed
                    for later in COMMANDS[index + 1 :]
                    for name in (later.identity,)
                ),
                "/chain",
                "native after stopped prefix",
            )
            break
    for name in PRODUCERS:
        folder = r.output / "capture" / name
        if name not in observed:
            require(
                {p.name for p in folder.iterdir()} == {"manifest.json", "native"}
                and not list((folder / "native").iterdir()),
                "/capture",
                "unjoined producer evidence",
            )
    recheck(live)
    live.anchors[r.output / "capture"] = frozen(r.output / "capture")
    locale = r.output / "logs/m1-trf.log"
    expected = b"".join(
        (observed[name].receipt.parent / "stdout.log").read_bytes() for name in cargos
    )
    require(
        {path.name for path in (r.output / "logs").iterdir()}
        == ({"m1-trf.log"} if cargo_success else set()),
        "/logs",
        "unexpected locale log namespace",
    )
    if cargo_success:
        require(locale.is_file(), "/locale", "missing original merged log")
        expected += locale.read_bytes()
    require(raw == expected, "/stdout", "original source-order output differs")
    cases: list[Json] = []
    failed = parent.result.native_exit != 0
    for index, name in enumerate(NATIVE_NAMES):
        if name not in observed:
            continue
        native = observed[name]
        executable = live.request.executables[index]
        listings: list[bytes] = []
        env = read(native.receipt).field("environment")
        environment = tuple(
            (key, env.field(key).text()) for key in sorted(env.mapping())
        )
        for which, args in (
            ("full", ("--list",)),
            ("ignored", ("--list", "--ignored")),
        ):
            recheck(live)
            out = r.output / "listings" / name / which
            out.parent.mkdir(parents=True, exist_ok=True)
            spec = ProcessSpec(
                (str(executable.binary), *args),
                executable.cwd,
                executable.binary,
                required[str(executable.binary)],
                r.root,
                r.inputs,
                out,
                environment,
                live.spec.external_inputs,
            )
            live.issued.append(spec)
            result = execute(spec)
            live.anchors[out] = frozen(out)
            binding = ListingProcess(spec, result, file_hash(result.receipt))
            require(result.native_exit == 0, "/listing", "independent listing failed")
            data = process_output(
                binding, spec.argv, spec.cwd, environment_for(spec), required
            )
            recheck(live)
            require(
                process_output(
                    binding, spec.argv, spec.cwd, environment_for(spec), required
                )
                == data,
                "/listing",
                "listing changed before freeze",
            )
            recheck(live)
            listings.append(data)
        full, ignored = map(parse_listing, listings)
        require(
            not full.benchmarks
            and not ignored.benchmarks
            and set(ignored.names) <= set(full.names),
            "/listing",
            "invalid full/ignored inventory",
        )
        selection = (
            Selection("default")
            if index < 4
            else Selection("substring", COMMANDS[index - 3].filter)
        )
        section = reconcile_selected(
            (native.receipt.parent / "stdout.log").read_bytes(),
            full,
            ignored,
            selection,
        )
        require(
            native.native_exit != 0 or section.failed == 0,
            "/accounting",
            "native success contradicts failure",
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
                "filtered_out": len(full.names) - len(section.outcomes),
                "world_qualification": "PENDING",
            }
        )
    recheck(live)
    _ = process_output(
        parent, live.spec.argv, live.spec.cwd, environment_for(live.spec), required
    )
    recheck(live)
    if cargo_success:
        passed = any(
            line.startswith(b"TRF_TEST_PASS")
            for line in locale.read_bytes().splitlines()
        )
        failed = (
            failed
            or observed["godot"].native_exit != 0
            or observed["bounded"].native_exit != 0
            or not passed
        )
    for name, descriptor in live.descriptors.items():
        os.close(descriptor)
        (r.output / "control" / (name + ".fifo")).unlink()
    live.descriptors.clear()
    recheck(live)
    receipt = r.output / "result.json"
    status: Literal["PASS", "FAIL"] = "FAIL" if failed else "PASS"
    live.published = True
    value: dict[str, Json] = {
        "schema": 1,
        "component": "M1_RUNTIME",
        "status": status,
        "step": 17,
        "invocation": r.invocation,
        "outer_pid": parent.result.pid,
        "outer_exit": parent.result.native_exit,
        "completed_producers": list(live.completed),
        "skipped_producers": [name for name in PRODUCERS if name not in observed],
        "cases": cases,
        "output_inventory": {
            name: digest for name, digest in snapshot(r.output).items()
        },
        "output_members": list(frozen(r.output)[1]),
        "source_build_qualification": "PENDING",
        "observer_qualification": "PENDING",
        "world_qualification": "PENDING",
        "publication_authority": False,
    }
    atomic_json(receipt, value)
    require(
        read(receipt).value == value,
        "/receipt",
        "terminal receipt changed during publication",
    )
    recheck(live)
    for path, digest in required.items():
        require(file_hash(Path(path)) == digest, "/inputs", "changed after publication")
    return M1Result(receipt, status)


if __name__ == "__main__":
    require(
        len(sys.argv) >= 5 and sys.argv[1] == "--shim",
        "/argv",
        "only closed shim entrypoint supported",
    )
    raise SystemExit(
        shim(Path(sys.argv[2]), sys.argv[3], sys.argv[4], tuple(sys.argv[5:]))
    )
