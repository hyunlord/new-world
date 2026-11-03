"""Source-bound original renderer generator and ASCII worlds; no child wait fiction."""
import json
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Final

from generic_gate import boundary, document, integer, require
from generic_observations import Observation, ProbeWorld, ordinary_pair, parse_probe
from generic_process import file_hash
from generic_renderer_run import PINS
from generic_whole_gate_aggregate import LIMIT, aggregate_output
from generic_whole_gate_perf import Anchors
from mux2_gate_inputs import Json, Node

ASCII_SHA: Final = '3875c7069915139df4202ed1fcb754c742e33323508c0bb8edd9772433333b4d'


@dataclass(frozen=True, slots=True)
class NativeRenderer:
    worlds: tuple[Observation, ...]
    emitted_pids: tuple[int, ...]
    aggregate_pid: int


def original_anchors(stage: Node) -> Anchors:
    return Anchors(tuple((name, Node(value).text()) for name, value in stage.field('observer_artifacts').mapping().items()),
                   tuple((name, integer(Node(value))) for name, value in stage.field('observer_modes').mapping().items()))


@boundary
def collect(session: Node, stage: Node) -> NativeRenderer:
    stage.field('step').equals(23)
    output, aggregate_pid = aggregate_output(session, stage)
    arm = stage.field('arm').text()
    product = session.field('arms').field(arm)
    root = Path(product.field('root').text())
    log = Path(session.field('output').text()) / arm / 'renderer_agreement'
    environment = stage.field('environment')
    require('RENDERER_AGREEMENT_TICK' not in environment.mapping(), '/horizon', 'nondefault original renderer horizon')
    environment.field('WORLDSIM_GATE_OBSERVER').equals('1')
    pins = {**PINS, 'rust/crates/sim-test/src/probe/ascii.rs': ASCII_SHA}
    for name, digest in pins.items():
        product.field('source_inputs').field(name).equals(digest)
        require(file_hash(root / name) == digest, name, 'unreviewed renderer source')
    anchors = original_anchors(stage)
    anchors.subtree(log)
    directory = Path(environment.field('WORLDSIM_GATE_TEST_OUT').text())
    anchors.subtree(directory)
    mirrors = tuple(directory.iterdir())
    require(len(mirrors) == 2, '/renderer', 'exactly generator and ASCII native mirrors required')
    selected: dict[Path, tuple[Node, int]] = {}
    for path in mirrors:
        match = re.fullmatch(r'probe_pid([1-9][0-9]*)_world0.json', path.name)
        require(match is not None, '/renderer', 'foreign renderer native occurrence')
        assert match is not None
        raw = document(anchors.read(path))
        raw.field('schema').equals('worldsim_gate_test_observation_v1')
        pid = integer(raw.field('process_id'))
        require(pid == int(match.group(1)), '/pid', 'native filename/PID mismatch')
        raw.field('world_sequence').equals(0)
        local = Path(raw.field('local_output').text())
        require(local not in selected, '/renderer', 'duplicate original native producer')
        selected[local] = (raw, pid)
    require(set(selected) == {log / 'probe', log}, '/renderer', 'foreign original output association')
    worlds: list[Observation] = []
    pids: list[int] = []
    for local, ticks in ((log / 'probe', 1440), (log, 900)):
        raw, pid = selected[local]
        sidecar = local / 'gate_observations/colony_s100_primary.json'
        raw.field('local_sidecar').equals(str(sidecar))
        observation = document(anchors.read(sidecar))
        raw.field('observation').equals(observation.value)
        world = parse_probe(json.dumps(observation.value), ProbeWorld('colony', 100, 'primary', ticks))
        require(world.founders == 10, '/founders', 'changed renderer founding census')
        worlds.append(world)
        pids.append(pid)
    require(len(set(pids)) == 2, '/pid', 'renderer processes unexpectedly share identity')
    require(len(re.findall(r'^RENDERER_AGREEMENT PASS n=[1-9][0-9]* tiles=[1-9][0-9]* tick=900 appearance=[1-9][0-9]* agree$', output, re.MULTILINE)) == 1,
            '/oracle', 'original complete renderer domain oracle missing')
    for filename in ('probe.log', 'ascii.log', 'godot.log', 'text_people.json', 'godot_people.json',
                     'commands/colony_s100_commands.json', 'ascii_t900.txt'):
        _ = anchors.read(log / filename)
    anchors.subtree(directory)
    anchors.subtree(log)
    _ = aggregate_output(session, stage)
    return NativeRenderer(tuple(worlds), tuple(pids), aggregate_pid)


@boundary
def evaluate_family(session: Node, base: Node, candidate: Node) -> dict[str, Json]:
    base.field('arm').equals('base')
    candidate.field('arm').equals('candidate')
    left, right = collect(session, base), collect(session, candidate)
    safety: list[Json] = [ordinary_pair(a, b) for a, b in zip(left.worlds, right.worlds, strict=True)]
    return {'status': 'PASS', 'safety': safety, 'qualification': LIMIT,
            'base_emitted_pids': list(left.emitted_pids), 'candidate_emitted_pids': list(right.emitted_pids),
            'scope': 'generator1440_ascii900_only_UI_instances_joined_separately'}
