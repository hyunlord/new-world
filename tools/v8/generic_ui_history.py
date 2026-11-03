"""Authenticate visibility save continuity without rewriting native load receipts."""
import hashlib
import json
from dataclasses import dataclass
from pathlib import Path

from generic_gate import boundary, document, integer, require, text
from generic_observations import Observation
from generic_process import ProcessResult, file_hash
from generic_ui_observations import (
    UIInstance,
    parse_native,
    require_fresh_history,
    unsigned_string,
)
from mux2_gate_inputs import Node

from generic_ui_receipts import UIPlan, ValidatedUI, validate_ui


@dataclass(frozen=True)
class UIEvidence:
    sidecar: Path
    sidecar_sha256: str
    plan: UIPlan
    process: ProcessResult
    process_sha256: str


@dataclass(frozen=True)
class SaveLink:
    save: Path
    save_sha256: str
    manifest: Path
    manifest_sha256: str
    scene_id: str
    prefix_checkpoint: int
    link_checkpoint: int
    loaded_checkpoint: int
    bridge_input: str


@dataclass(frozen=True)
class JoinedHistory:
    observation: Observation
    evidence_json: str


def _read(evidence: UIEvidence) -> Node:
    _ = validate_ui(evidence.sidecar, evidence.sidecar_sha256, evidence.plan,
                evidence.process, evidence.process_sha256)
    raw = evidence.sidecar.read_bytes()
    require(hashlib.sha256(raw).hexdigest() == evidence.sidecar_sha256, "/history", "sidecar changed after validation")
    return document(raw.decode("utf-8"))


def _checkpoint(node: Node, index: int, label: str) -> tuple[Node, UIInstance]:
    rows = node.field("checkpoints").array()
    require(type(index) is int and 0 <= index < len(rows), "/history", "checkpoint index out of range")
    row = rows[index]
    row.field("label").equals(label)
    identity = unsigned_string(row.field("node_instance_id"))
    instances = parse_native(json.dumps(row.field("observation").value), identity)
    return row.field("context"), instances[-1]


def _input(evidence: UIEvidence, path: Path, digest: str) -> None:
    require(dict(evidence.plan.process.external_inputs).get(path) == digest,
            "/history", "loader lacks authenticated external save/manifest input")
    require(file_hash(path) == digest, "/history", "save/manifest bytes changed")


def _argument(evidence: UIEvidence, flag: str, expected: str) -> None:
    args = evidence.plan.process.argv
    positions = [index for index, value in enumerate(args) if value == flag]
    require(len(positions) == 1 and positions[0] + 1 < len(args) and args[positions[0] + 1] == expected,
            "/history", "missing or ambiguous producer process argument " + flag)


@boundary
def join_visibility_history(generator: UIEvidence, loader: UIEvidence, link: SaveLink) -> JoinedHistory:
    """Join one complete generated prefix to its exact loaded native occurrence."""
    require(generator.plan.producer == "visibility.generator" and loader.plan.producer == "visibility.loader",
            "/history", "unsupported save producer/loader")
    require(generator.plan.script == "res://tools/v8/mux2_visibility_saves.gd" and
            loader.plan.script == "res://tools/v8/mux2_visibility_test.gd", "/history", "foreign producer script")
    require(generator.plan.invocation_id != loader.plan.invocation_id, "/history", "reused process invocation")
    _argument(generator, "--script", generator.plan.script)
    _argument(loader, "--script", loader.plan.script)
    args = generator.plan.process.argv
    require(args.count("--") == 1 and args[args.index("--") + 1:] == (str(link.save.parent),),
            "/history", "generator requires exact positional output process argument")
    _argument(loader, "--manifest", str(link.manifest))
    require(link.manifest.parent == link.save.parent, "/history", "manifest outside generator output")
    source = _read(generator)
    destination = _read(loader)
    bridge_hash = dict(generator.plan.process.inputs).get(link.bridge_input)
    require(bridge_hash is not None and dict(loader.plan.process.inputs).get(link.bridge_input) == bridge_hash,
            "/history", "native products differ or are unbound")
    assert bridge_hash is not None
    _input(loader, link.save, link.save_sha256)
    _input(loader, link.manifest, link.manifest_sha256)
    validated=validate_ui(loader.sidecar,loader.sidecar_sha256,loader.plan,loader.process,loader.process_sha256)
    return join_validated_history(source,destination,validated,loader.plan.checkpoints[link.loaded_checkpoint].node_slot,
                                  link,bridge_hash,generator.sidecar_sha256,generator.process_sha256,
                                  loader.sidecar_sha256,loader.process_sha256)


def join_validated_history(source: Node, destination: Node, validated: ValidatedUI, loader_slot: int,
                           link: SaveLink, bridge_hash: str, generator_sidecar_sha256: str,
                           generator_process_sha256: str, loader_sidecar_sha256: str,
                           loader_process_sha256: str) -> JoinedHistory:
    """Join native prefix semantics after the caller authenticates both producers and save inputs."""
    require(link.link_checkpoint == link.prefix_checkpoint + 1, "/history", "save link is not adjacent to prefix")
    prefix_context, prefix = _checkpoint(source, link.prefix_checkpoint, "save_prefix")
    saved_context, saved = _checkpoint(source, link.link_checkpoint, "save_link")
    loaded_context, initial = _checkpoint(destination, link.loaded_checkpoint, "loaded_initial")
    prefix_raw = source.field("checkpoints").array()[link.prefix_checkpoint].field("observation").field("instances").array()[-1]
    loaded_raw = destination.field("checkpoints").array()[link.loaded_checkpoint].field("observation").field("instances").array()[-1]
    loaded_raw.field("knowledge_lost").equals(prefix_raw.field("knowledge_lost").value)
    require(prefix == saved, "/history", "native state changed across save")
    _ = require_fresh_history(prefix)
    prefix_context.field("save_path").equals(str(link.save))
    prefix_context.field("scene_id").equals(link.scene_id)
    saved_context.field("prefix_checkpoint").equals(link.prefix_checkpoint)
    saved_context.field("save_path").equals(str(link.save))
    saved_context.field("save_sha256").equals(link.save_sha256)
    saved_context.field("scene_id").equals(link.scene_id)
    loaded_context.field("save_path").equals(str(link.save))
    loaded_context.field("save_sha256").equals(link.save_sha256)
    loaded_context.field("manifest_path").equals(str(link.manifest))
    loaded_context.field("manifest_sha256").equals(link.manifest_sha256)
    manifest_raw = link.manifest.read_bytes()
    require(hashlib.sha256(manifest_raw).hexdigest() == link.manifest_sha256, "/history", "manifest changed during read")
    rows = document(manifest_raw.decode("utf-8")).field("scenes").array()
    matching = [row for row in rows if text(row.field("id")) == link.scene_id]
    require(len(matching) == 1, "/history", "missing or duplicate manifest scene")
    row = matching[0]
    loaded_context.field("manifest_row").equals(row.value)
    row.field("save").equals(str(link.save))
    row.field("sha256").equals(link.save_sha256)
    require(integer(row.field("tick")) == prefix.state.terminal_tick, "/history", "manifest tick differs")
    require(initial.reason == "load" and not initial.history_complete and
            initial.history_gap == "initial_history_unauthenticated", "/history", "native load has corruption or foreign lifecycle")
    require((initial.seed, initial.state.initial_tick, initial.state.terminal_tick, initial.initial_agent_ids,
             initial.terminal_agent_ids, initial.state.knowledge_enabled, initial.state.losses, initial.chronicle_cursor) ==
            (prefix.seed, prefix.state.terminal_tick, prefix.state.terminal_tick, prefix.terminal_agent_ids,
             prefix.terminal_agent_ids, prefix.state.knowledge_enabled, prefix.state.losses, prefix.chronicle_cursor),
            "/history", "loaded prefix census, capability, tick or cumulative history differs")
    slot = loader_slot
    terminal = validated.instances[slot][initial.ordinal]
    require(terminal.history_gap == "initial_history_unauthenticated" and not terminal.history_complete,
            "/history", "loaded continuation has native corruption")
    require(terminal.state.losses[:len(prefix.state.losses)] == prefix.state.losses,
            "/history", "loaded continuation lost authenticated prefix")
    evidence = {"schema_version": 1, "kind": "authenticated_visibility_save_prefix",
                "generator_sidecar_sha256": generator_sidecar_sha256, "generator_process_sha256": generator_process_sha256,
                "loader_sidecar_sha256": loader_sidecar_sha256, "loader_process_sha256": loader_process_sha256,
                "save_sha256": link.save_sha256, "manifest_sha256": link.manifest_sha256,
                "save_path": str(link.save), "manifest_path": str(link.manifest), "scene_id": link.scene_id,
                "prefix_checkpoint": link.prefix_checkpoint, "link_checkpoint": link.link_checkpoint,
                "loaded_checkpoint": link.loaded_checkpoint, "generator_node": str(prefix.node_id),
                "generator_ordinal": prefix.ordinal, "loader_node": str(initial.node_id),
                "loader_ordinal": initial.ordinal, "bridge_sha256": bridge_hash,
                "raw_loaded_history_complete": False}
    chain = json.dumps(evidence, sort_keys=True, separators=(",", ":"))
    state = Observation(prefix.state.initial_tick, terminal.state.terminal_tick, prefix.state.founders,
                        terminal.state.alive, prefix.state.knowledge_enabled, terminal.state.losses, chain)
    return JoinedHistory(state, chain)
