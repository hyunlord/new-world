"""Bind UI native observations to completed processes and exact occurrence plans.

This validates evidence, not source/build authenticity or safety acceptance.
Loaded-history continuity remains explicitly unresolved.
"""

import json
from dataclasses import dataclass
from pathlib import Path

from generic_gate import boundary, document, integer, require, text
from generic_observations import Observation, keys
from generic_process import ProcessResult, ProcessSpec, environment_for, file_hash
from generic_ui_observations import (
    UIInstance,
    parse_native,
    require_fresh_history,
    unsigned_string,
    validate_progress,
)
from mux2_gate_inputs import Json, Node


@dataclass(frozen=True)
class InstancePlan:
    reason: str
    seed: int
    initial_tick: int
    terminal_tick: int


@dataclass(frozen=True)
class CheckpointPlan:
    label: str
    node_slot: int
    instance_count: int
    tick: int
    context_json: str = "{}"
    releases_node: bool = False


@dataclass(frozen=True)
class UIPlan:
    producer: str
    invocation_id: str
    process: ProcessSpec
    script: str
    helper: str
    checkpoints: tuple[CheckpointPlan, ...]
    instances: tuple[tuple[InstancePlan, ...], ...]
    classification: str = "native_world_producer"
    completion: str = "producer_finish"
    no_native_reason: str = ""


@dataclass(frozen=True)
class ValidatedUI:
    """Source-bound observations; this type does not mean safety or load admission."""

    instances: tuple[tuple[UIInstance, ...], ...]
    loaded_unqualified: tuple[tuple[int, int], ...]

    def fresh_observations(self) -> tuple[Observation, ...]:
        """Reject all loaded/incomplete histories rather than manufacture zero loss."""
        return tuple(require_fresh_history(instance) for node in self.instances for instance in node)


def _json(node: Node) -> str:
    return json.dumps(node.value, sort_keys=True, separators=(",", ":"), allow_nan=False)


def _bind_process(plan: UIPlan, result: ProcessResult, receipt_sha256: str) -> None:
    spec = plan.process
    require(result.receipt == spec.output_root / "process.json", "/process", "foreign receipt path")
    require(file_hash(result.receipt) == receipt_sha256, "/process", "receipt hash mismatch")
    process = document(result.receipt.read_text())
    require(result.status == "PASS" and result.native_exit == 0 and type(result.pid) is int and result.pid > 0,
            "/process", "native process failed or PID missing")
    expected: dict[str, Json] = {"schema_version": 1, "status": "PASS", "native_exit": 0, "pid": result.pid, "argv": list(spec.argv),
                "cwd": str(spec.cwd), "executable": str(spec.executable),
                "expected_executable_sha256": spec.executable_sha256, "input_root": str(spec.input_root),
                "fresh_directories": list(spec.fresh_directories),
                "output_root": str(spec.output_root), "environment": {k: v for k, v in environment_for(spec).items()},
                "expected_inputs": dict(spec.inputs), "before_inputs": dict(spec.inputs),
                "after_inputs": dict(spec.inputs),
                "expected_external_inputs": {str(path): digest for path, digest in spec.external_inputs},
                "before_external_inputs": {str(path): digest for path, digest in spec.external_inputs},
                "after_external_inputs": {str(path): digest for path, digest in spec.external_inputs},
                "before_executable_sha256": spec.executable_sha256,
                "after_executable_sha256": spec.executable_sha256, "errors": []}
    for key, value in expected.items():
        process.field(key).equals(value)
    process.field("environment_policy").equals("fixed_defaults_plus_explicit_only_no_inheritance")
    if spec.timeout_seconds is not None:
        process.field("timeout_seconds").equals(spec.timeout_seconds)
        process.field("timed_out").equals(False)
    else:
        require("timeout_seconds" not in process.mapping() and "timed_out" not in process.mapping(),
                "/process", "unexpected timeout policy")
    _ = text(process.field("started_at"))
    _ = text(process.field("ended_at"))
    _ = integer(process.field("duration_ns"))
    for name in ("stdout", "stderr"):
        process.field(name + "_sha256").equals(file_hash(spec.output_root / (name + ".log")))
    require(file_hash(spec.executable) == spec.executable_sha256, "/process", "executable changed since run")
    require(len(dict(spec.inputs)) == len(spec.inputs), "/inputs", "duplicate expected input")
    for name, digest in spec.inputs:
        path = Path(name)
        require(not path.is_absolute() and ".." not in path.parts, "/inputs", "unsafe input path")
        require(file_hash(spec.input_root / path) == digest, "/inputs", "input bytes changed since run")

    require(len(dict(spec.external_inputs)) == len(spec.external_inputs), "/inputs", "duplicate external input")
    for path, digest in spec.external_inputs:
        require(file_hash(path) == digest, "/inputs", "external input bytes changed since run")


# Public process authentication shared by non-UI producers; no safety admission.
bind_process = _bind_process


def godot_visible_arguments(argv: tuple[str, ...]) -> tuple[list[Json], list[Json]]:
    """Project only verified Godot 4.6 consumed flags; full argv binds in process.json."""
    visible: list[Json] = []
    index = 1
    while index < len(argv):
        arg = argv[index]
        if arg in {"--", "++"}:
            return visible, list(argv[index + 1:])
        if arg in {"--headless", "--windowed", "--always-on-top"}:
            index += 1
        elif arg in {"--path", "--quit-after", "--rendering-method", "--resolution"}:
            require(index + 1 < len(argv), "/argv", "missing consumed Godot flag value")
            index += 2
        else:
            visible.append(arg)
            index += 1
    return visible, []


def _bind_envelope(node: Node, plan: UIPlan, result: ProcessResult) -> None:
    keys(node, {"schema_version", "producer", "invocation_id", "pid", "argv", "user_args", "project",
                "script", "script_sha256", "helper_sha256", "complete", "completion", "native_exit_code",
                "classification", "no_native_reason", "observed_native_nodes", "checkpoints", "errors"})
    spec = plan.process
    environment = environment_for(spec)
    require(environment.get("WORLDSIM_GATE_OBSERVER") == "1" and
            environment.get("WORLDSIM_GATE_INVOCATION_ID") == plan.invocation_id,
            "/environment", "observer/invocation was not enabled for actual child")
    node.field("schema_version").equals(1)
    node.field("producer").equals(plan.producer)
    node.field("invocation_id").equals(plan.invocation_id)
    node.field("pid").equals(result.pid)
    node.field("complete").equals(True)
    node.field("errors").equals([])
    require(plan.completion in {"producer_finish", "native_tree_exit"}, "/plan", "unknown completion")
    node.field("completion").equals(plan.completion)
    node.field("native_exit_code").equals(0 if plan.completion == "producer_finish" else -1)
    engine_args, user_args = godot_visible_arguments(spec.argv)
    node.field("argv").equals(engine_args)
    node.field("user_args").equals(user_args)
    require(text(node.field("project")).rstrip("/") == str(spec.cwd) and spec.cwd == spec.input_root,
            "/project", "sidecar project differs from actual process cwd/input root")
    node.field("script").equals(plan.script)
    inputs = dict(spec.inputs)
    for script, key in ((plan.script, "script_sha256"), (plan.helper, "helper_sha256")):
        require(script.startswith("res://") and script[6:] in inputs, "/inputs", "script/helper lacks independent input hash")
        node.field(key).equals(inputs[script[6:]])
    node.field("classification").equals(plan.classification)
    node.field("no_native_reason").equals(plan.no_native_reason)
    node.field("observed_native_nodes").equals(len(plan.instances))
    require(plan.classification in {"native_world_producer", "synthetic_renderer_only"}, "/plan", "unknown classification")
    if plan.classification == "synthetic_renderer_only":
        require(not plan.instances and not plan.checkpoints and bool(plan.no_native_reason), "/plan", "synthetic-only cannot contain or replace native worlds")
    else:
        require(bool(plan.instances) and not plan.no_native_reason, "/plan", "native producer missing instance obligations")


@boundary
def validate_ui(path: Path, sidecar_sha256: str, plan: UIPlan, result: ProcessResult,
                process_receipt_sha256: str) -> ValidatedUI:
    """Validate exact receipts, retaining load uncertainty and all actual instances."""
    _bind_process(plan, result, process_receipt_sha256)
    directory = environment_for(plan.process).get("WORLDSIM_GATE_UI_OUT")
    require(directory is not None and path.parent == Path(directory), "/sidecar", "foreign output directory")
    require(path.name == f"{plan.producer}-{result.pid}.json", "/sidecar", "foreign producer/PID filename")
    require(file_hash(path) == sidecar_sha256, "/sidecar", "sidecar hash mismatch")
    node = document(path.read_text())
    _bind_envelope(node, plan, result)
    return validate_lifecycle(node, plan.checkpoints, plan.instances)


@boundary
def validate_lifecycle(node: Node, checkpoint_plan: tuple[CheckpointPlan, ...],
                       instance_plan: tuple[tuple[InstancePlan, ...], ...]) -> ValidatedUI:
    """Validate native lifecycle semantics after an independently authenticated envelope."""
    checkpoints = node.field("checkpoints").array()
    require(len(checkpoints) == len(checkpoint_plan), "/checkpoints", "missing or extra checkpoint")
    states: dict[int, tuple[UIInstance, ...]] = {}
    identities: dict[int, int] = {}
    released: set[int] = set()
    for index, (checkpoint, expected) in enumerate(zip(checkpoints, checkpoint_plan)):
        keys(checkpoint, {"index", "label", "node_instance_id", "observation", "tallies", "commands",
                          "clock", "command_log_capability", "context"})
        checkpoint.field("index").equals(index)
        checkpoint.field("label").equals(expected.label)
        require(type(expected.tick) is int and expected.tick >= 0, "/plan", "unknown or invalid actual horizon")
        require(type(expected.instance_count) is int and expected.instance_count > 0,
                "/plan", "invalid retained-instance obligation")
        require(0 <= expected.node_slot < len(instance_plan) and expected.node_slot not in released,
                "/checkpoint", "unknown or already released native node")
        require(_json(checkpoint.field("context")) == _json(document(expected.context_json)), "/context", "unbound input/intervention/save context")
        identity = unsigned_string(checkpoint.field("node_instance_id"))
        current = parse_native(_json(checkpoint.field("observation")), identity)
        require(len(current) == expected.instance_count, "/instances", "omitted or extra retained instance")
        if expected.node_slot in identities:
            require(identities[expected.node_slot] == identity, "/instances", "node identity changed")
            validate_progress(states[expected.node_slot], current)
        else:
            require(identity not in identities.values(), "/instances", "node reused for independent occurrence")
            require(expected.label == "bootstrap" and len(current) == 1 and current[0].reason == "bootstrap"
                    and current[0].state.initial_tick == 0 and current[0].state.terminal_tick == 0,
                    "/instances", "initial bootstrap was omitted or observed after advancement")
            identities[expected.node_slot] = identity
        require(current[-1].state.terminal_tick == expected.tick, "/horizon", "actual tick differs from exact expected checkpoint")
        require(integer(checkpoint.field("clock").field("tick")) == expected.tick, "/clock", "clock/native observation disagree")
        checkpoint.field("command_log_capability").equals("ticks_kinds_accepted_only")
        _ = checkpoint.field("tallies").mapping()
        _ = checkpoint.field("commands").mapping()
        states[expected.node_slot] = current
        if expected.releases_node:
            released.add(expected.node_slot)
    require(set(states) == set(range(len(instance_plan))), "/instances", "missing native node")
    require(released == set(states), "/instances", "missing explicit terminal/release checkpoint")
    ordered = tuple(states[slot] for slot in range(len(instance_plan)))
    unresolved: list[tuple[int, int]] = []
    for slot, (actual, expected_rows) in enumerate(zip(ordered, instance_plan)):
        require(len(actual) == len(expected_rows), "/instances", "final retained history incomplete")
        for instance, expected in zip(actual, expected_rows):
            require(expected.reason in {"bootstrap", "reset", "stress_reset", "load"}, "/plan", "unknown native lifecycle")
            require(type(expected.terminal_tick) is int and expected.terminal_tick >= expected.initial_tick,
                    "/plan", "unknown actual endpoint; exact-horizon companion required")
            require((instance.reason, instance.seed, instance.state.initial_tick, instance.state.terminal_tick) ==
                    (expected.reason, expected.seed, expected.initial_tick, expected.terminal_tick),
                    "/instances", "actual native identity/horizon differs from required occurrence")
            if instance.reason == "load":
                unresolved.append((slot, instance.ordinal))
            else:
                _ = require_fresh_history(instance)
    return ValidatedUI(ordered, tuple(unresolved))


def require_companion_horizon(actual: UIInstance, companion: UIInstance) -> None:
    """Check exact native horizon/capability; caller separately binds product and inputs."""
    require((actual.seed, actual.state.initial_tick, actual.state.terminal_tick, actual.state.founders,
             actual.state.knowledge_enabled) ==
            (companion.seed, companion.state.initial_tick, companion.state.terminal_tick, companion.state.founders,
             companion.state.knowledge_enabled), "/companion", "companion does not match actual native world horizon")
