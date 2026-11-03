"""Renderer build prerequisites only; import/runtime admission remains unavailable."""
import os
from dataclasses import dataclass
from pathlib import Path

from generic_gate import document, require
from generic_process import file_hash
from generic_whole_gate_qualification import authenticated_build
from mux2_gate_inputs import Node


@dataclass(frozen=True, slots=True)
class RendererBuilds:
    root: Path
    probe: Path
    bridge: Path
    godot: Path
    godot_sha256: str


def selected_godot(session: Node) -> tuple[Path, str]:
    selector = Path(session.field('selector_path').text())
    require(file_hash(selector) == session.field('selector_sha256').text(), '/selector', 'selector changed')
    selected = document(selector.read_text())
    path = Path(selected.field('environment').field('GODOT_BIN').text())
    digest = selected.field('external_inputs').field(str(path)).text()
    session.field('environment').field('GODOT_BIN').equals(str(path))
    require(path == path.resolve(strict=True), '/godot', 'noncanonical selected Godot')
    session.field('external_inputs').field(str(path)).equals(digest)
    require(file_hash(path) == digest and os.access(path, os.X_OK), '/godot', 'selected Godot changed or not executable')
    return path, digest


def authenticate_builds(session: Node, arm: Node, mode: str, evidence: Node) -> RendererBuilds:
    """Join two original generic_build receipts without granting runtime authority."""
    require(mode in {'original_off', 'derived_off', 'derived_on'}, '/mode', 'unknown qualification mode')
    probe = authenticated_build(mode, evidence.field('probe'), arm, 'worldsim_probe', session)
    bridge = authenticated_build(mode, evidence.field('bridge'), arm, 'sim_bridge', session)
    for field in ('root', 'source_sha', 'source_before', 'effective_source_before'):
        bridge.field(field).equals(probe.field(field).value)
    godot, digest = selected_godot(session)
    return RendererBuilds(Path(probe.field('root').text()),
                          Path(probe.field('artifacts').field('worldsim_probe').field('path').text()),
                          Path(bridge.field('artifacts').field('sim_bridge').field('path').text()),
                          godot, digest)
