"""Synthetic sidecars bind to independent process artifacts; never run Godot."""

import copy
import json
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from typing import override

from generic_gate import GateContractError
from generic_process import ProcessResult, ProcessSpec, environment_for, file_hash
from generic_ui_receipts import (
    CheckpointPlan,
    InstancePlan,
    UIPlan,
    ValidatedUI,
    godot_visible_arguments,
    require_companion_horizon,
    validate_ui,
)
from mux2_gate_inputs import Json, Node


def obj(value: Json, *path: str | int) -> dict[str, Json]:
    node = Node(value)
    for key in path:
        node = node.field(key) if isinstance(key, str) else node.array()[key]
    return node.mapping()


def arr(value: Json, *path: str | int) -> list[Json]:
    node = Node(value)
    for key in path:
        node = node.field(key) if isinstance(key, str) else node.array()[key]
    if not isinstance(node.value, list):
        raise TypeError("expected fixture list")
    return node.value


def checkpoints(value: Json) -> list[dict[str, Json]]:
    return [row.mapping() for row in Node(value).field("checkpoints").array()]


def native(tick: int) -> dict[str, Json]:
    return {"schema_version": 1, "status": "OBSERVED", "node_instance_id": "42", "instances": [{
        "instance_ordinal": 0, "reason": "bootstrap", "seed": "7", "initial_tick": "0",
        "initial_population": 1, "initial_agent_ids": [str(2**64 - 1)], "terminal_tick": str(tick),
        "terminal_alive": 1, "terminal_agent_ids": [str(2**64-1)], "closed": False, "history_complete": True, "history_gap": None,
        "chronicle_cursor": 0, "knowledge_enabled": True,
        "knowledge_capability_source": "resources.rules.colony.knowledge", "knowledge_lost": []}]}


class UIReceiptTests(unittest.TestCase):
    root: Path
    cwd: Path
    output: Path
    sidecars: Path
    spec: ProcessSpec
    plan: UIPlan
    process: dict[str, Json]
    result: ProcessResult
    path: Path
    data: dict[str, Json]

    @override
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name).resolve()
        self.cwd = self.root / "project"
        self.cwd.mkdir()
        for name in ("producer.gd", "helper.gd"):
            _ = (self.cwd / name).write_text("# synthetic source\n")
        self.output = self.root / "process"
        self.output.mkdir()
        self.sidecars = self.root / "sidecars"
        self.sidecars.mkdir()
        executable = self.root / "godot"
        _ = executable.write_bytes(b"not executable; synthetic evidence only")
        argv = (str(executable), "--headless", "--path", str(self.cwd), "--script", "res://producer.gd", "--", "--fixture")
        self.spec = ProcessSpec(argv, self.cwd, executable, file_hash(executable), self.cwd,
                                tuple((name, file_hash(self.cwd / name)) for name in ("producer.gd", "helper.gd")), self.output,
                                (("WORLDSIM_GATE_OBSERVER", "1"), ("WORLDSIM_GATE_INVOCATION_ID", "run1"), ("WORLDSIM_GATE_UI_OUT", str(self.sidecars))))
        self.plan = UIPlan("fixture", "run1", self.spec, "res://producer.gd", "res://helper.gd",
                           (CheckpointPlan("bootstrap", 0, 1, 0), CheckpointPlan("terminal", 0, 1, 30, releases_node=True)),
                           ((InstancePlan("bootstrap", 7, 0, 30),),))
        for name in ("stdout", "stderr"):
            _ = (self.output / (name + ".log")).write_text("")
        self.process = {"schema_version": 1, "status": "PASS", "native_exit": 0, "pid": 123, "argv": list(argv),
                        "cwd": str(self.cwd), "executable": str(executable), "expected_executable_sha256": self.spec.executable_sha256,
                        "input_root": str(self.cwd), "fresh_directories": [],
                        "output_root": str(self.output), "environment": {k:v for k,v in environment_for(self.spec).items()},
                        "expected_inputs": dict(self.spec.inputs), "before_inputs": dict(self.spec.inputs), "after_inputs": dict(self.spec.inputs),
                        "expected_external_inputs": {}, "before_external_inputs": {}, "after_external_inputs": {},
                        "before_executable_sha256": self.spec.executable_sha256, "after_executable_sha256": self.spec.executable_sha256,
                        "environment_policy": "fixed_defaults_plus_explicit_only_no_inheritance", "started_at": "2026-11-02T00:00:00Z",
                        "ended_at": "2026-11-02T00:00:01Z", "duration_ns": 1_000_000_000, "errors": [],
                        "stdout_sha256": file_hash(self.output / "stdout.log"), "stderr_sha256": file_hash(self.output / "stderr.log")}
        self.result = ProcessResult(self.output / "process.json", "PASS", 0, 123)
        self.path = self.sidecars / "fixture-123.json"
        self.data = {"schema_version": 1, "producer": "fixture", "invocation_id": "run1", "pid": 123,
                     "argv": list(argv[4:-2]), "user_args": ["--fixture"], "project": str(self.cwd) + "/",
                     "script": "res://producer.gd", "script_sha256": dict(self.spec.inputs)["producer.gd"],
                     "helper_sha256": dict(self.spec.inputs)["helper.gd"], "complete": True, "completion": "producer_finish",
                     "native_exit_code": 0, "classification": "native_world_producer", "no_native_reason": "",
                     "observed_native_nodes": 1, "errors": [], "checkpoints": [self.checkpoint(0, "bootstrap", 0), self.checkpoint(1, "terminal", 30)]}

    def checkpoint(self, index: int, label: str, tick: int) -> dict[str, Json]:
        return {"index": index, "label": label, "node_instance_id": "42", "observation": native(tick),
                "tallies": {}, "commands": {}, "clock": {"tick": tick}, "context": {},
                "command_log_capability": "ticks_kinds_accepted_only"}

    def validate(self) -> ValidatedUI:
        _ = self.result.receipt.write_text(json.dumps(self.process))
        _ = self.path.write_text(json.dumps(self.data))
        return validate_ui(self.path, file_hash(self.path), self.plan, self.result, file_hash(self.result.receipt))

    def test_actual_godot_consumed_arguments_projection(self) -> None:
        self.assertEqual(godot_visible_arguments(("godot", "--headless", "--path", "/project", "--quit-after", "900")), ([], []))
        self.assertEqual(godot_visible_arguments(("godot", "--headless", "--path", "/project", "--script", "res://smoke.gd", "--", "--frames", "900")),
                         (["--script", "res://smoke.gd"], ["--frames", "900"]))
        self.assertEqual(godot_visible_arguments(("godot", "--unknown")), (["--unknown"], []))

    def test_verified_render_flags_are_consumed(self) -> None:
        self.assertEqual(godot_visible_arguments(("godot", "--headless", "--rendering-method",
                         "gl_compatibility", "--resolution", "1920x1080", "--windowed",
                         "--always-on-top", "--script", "res://probe.gd", "--", "--cache")),
                         (["--script", "res://probe.gd"], ["--cache"]))

    def test_timeout_receipt_matches_original_bound(self) -> None:
        self.spec = replace(self.spec, timeout_seconds=180)
        self.plan = replace(self.plan, process=self.spec)
        with self.assertRaises(GateContractError):
            _ = self.validate()

        self.process.update(timeout_seconds=180, timed_out=False)
        _ = self.validate()
        for timeout, timed_out in ((300, False), (180, True)):
            self.process.update(timeout_seconds=timeout, timed_out=timed_out)
            with self.assertRaises(GateContractError):
                _ = self.validate()

    def test_exact_bound_receipt_keeps_unsigned_identity(self) -> None:
        value = self.validate()
        self.assertEqual(value.instances[0][0].initial_agent_ids, (2**64 - 1,))
        self.assertEqual(value.fresh_observations()[0].terminal_tick, 30)

    def test_outer_identity_hash_and_exit_tampering_rejected(self) -> None:
        original = copy.deepcopy(self.data)
        mutations: tuple[tuple[str, Json], ...] = (("pid", 124), ("invocation_id", "other"), ("script_sha256", "0" * 64),
                           ("helper_sha256", "0" * 64), ("native_exit_code", 1), ("project", "/elsewhere/"),
                           ("argv", []), ("complete", False))
        for key, value in mutations:
            self.data = copy.deepcopy(original)
            self.data[key] = value
            with self.subTest(key=key), self.assertRaises(GateContractError):
                _ = self.validate()

    def test_independent_process_failure_or_mutation_rejected(self) -> None:
        mutations: tuple[tuple[str, Json], ...] = (("pid", 9), ("native_exit", 2), ("status", "FAIL"), ("after_inputs", {}))
        for key, value in mutations:
            original = copy.deepcopy(self.process)
            self.process[key] = value
            with self.subTest(key=key), self.assertRaises(GateContractError):
                _ = self.validate()
            self.process = original
        _ = (self.cwd / "producer.gd").write_text("changed after child")
        with self.assertRaises(GateContractError):
            _ = self.validate()

    def test_missing_bootstrap_extra_checkpoint_and_unknown_horizon_rejected(self) -> None:
        original = copy.deepcopy(self.data)
        _ = arr(self.data, "checkpoints").pop(0)
        with self.assertRaises(GateContractError):
            _ = self.validate()
        self.data = original
        self.plan = replace(self.plan, checkpoints=(self.plan.checkpoints[0], replace(self.plan.checkpoints[1], tick=None)))
        with self.assertRaisesRegex(GateContractError, "unknown"):
            _ = self.validate()

    def test_terminal_must_be_explicit_and_tick_exact(self) -> None:
        original = self.plan
        for checkpoint in (replace(original.checkpoints[1], releases_node=False), replace(original.checkpoints[1], tick=29)):
            self.plan = replace(original, checkpoints=(original.checkpoints[0], checkpoint))
            with self.assertRaises(GateContractError):
                _ = self.validate()

    def test_synthetic_zero_cannot_replace_required_native_node(self) -> None:
        self.data.update(classification="synthetic_renderer_only", no_native_reason="geometry", checkpoints=[], observed_native_nodes=0)
        with self.assertRaises(GateContractError):
            _ = self.validate()
        self.plan = replace(self.plan, classification="synthetic_renderer_only", no_native_reason="geometry", checkpoints=(), instances=())
        value = self.validate()
        self.assertEqual(value.instances, ())

    def test_loaded_instance_is_retained_but_never_fresh(self) -> None:
        end = arr(self.data, "checkpoints", 1, "observation", "instances")
        obj(end[0])["closed"] = True
        loaded = copy.deepcopy(obj(end[0]))
        loaded.update(instance_ordinal=1, reason="load", initial_tick="30", terminal_tick="30", closed=False,
                      history_complete=False, history_gap="initial_history_unauthenticated")
        end.append(loaded)
        self.plan = replace(self.plan, checkpoints=(self.plan.checkpoints[0], replace(self.plan.checkpoints[1], instance_count=2)),
                            instances=((InstancePlan("bootstrap", 7, 0, 30), InstancePlan("load", 7, 30, 30)),))
        value = self.validate()
        self.assertEqual(value.loaded_unqualified, ((0, 1),))
        with self.assertRaisesRegex(GateContractError, "save-prefix"):
            _ = value.fresh_observations()

    def test_context_and_log_bytes_are_independently_bound(self) -> None:
        obj(self.data, "checkpoints", 1)["context"] = {"save_sha256": "0" * 64}
        with self.assertRaisesRegex(GateContractError, "unbound"):
            _ = self.validate()
        obj(self.data, "checkpoints", 1)["context"] = {}
        _ = (self.output / "stderr.log").write_text("changed after process receipt")
        with self.assertRaises(GateContractError):
            _ = self.validate()

    def test_extra_native_reset_cannot_hide_inside_a_known_node(self) -> None:
        rows = arr(self.data, "checkpoints", 1, "observation", "instances")
        added = copy.deepcopy(obj(rows[0]))
        obj(rows[0])["closed"] = True
        added.update(instance_ordinal=1, reason="reset")
        rows.append(added)
        with self.assertRaisesRegex(GateContractError, "retained instance"):
            _ = self.validate()

    def test_actual_companion_horizon_must_match_exactly(self) -> None:
        instance = self.validate().instances[0][0]
        require_companion_horizon(instance, instance)
        different = replace(instance, state=replace(instance.state, terminal_tick=31))
        with self.assertRaisesRegex(GateContractError, "horizon"):
            require_companion_horizon(instance, different)

    def test_stale_sidecar_or_process_digest_rejected(self) -> None:
        _ = self.validate()
        for sidecar_hash, process_hash in (("0" * 64, file_hash(self.result.receipt)), (file_hash(self.path), "0" * 64)):
            with self.assertRaisesRegex(GateContractError, "hash mismatch"):
                _ = validate_ui(self.path, sidecar_hash, self.plan, self.result, process_hash)


if __name__ == "__main__":
    _ = unittest.main()
