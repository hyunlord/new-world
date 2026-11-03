"""Pure source-local grouping of eight native twin mirrors; no execution authority."""
import json
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Final

from generic_gate import boundary, integer, require, text
from generic_observations import Observation, ProbeWorld, ordinary_pair, parse_probe
from generic_test_endpoint_types import fields
from mux2_gate_inputs import Json, Node

CASES: Final = {
    'baseline': 'probe::gate_observation::tests::baseline_twins_emit_native_capabilities_without_changing_raw_samples',
    'colony': 'probe::gate_observation::tests::native_colony_twins_emit_separate_receipts_without_summary_changes',
}
DIRECT: Final = frozenset('probe::gate_observation::tests::' + name for name in (
    'pending_and_short_run_cannot_claim_terminal_completion',
    'baseline_capability_is_explicit_and_observation_is_read_only',
    'loaded_tick_without_prior_history_proof_remains_incomplete',
    'failed_loop_and_nonmonotonic_history_cannot_be_complete_evidence',
))


@dataclass(frozen=True, slots=True)
class Twin:
    scene: str
    run: str
    occurrence: str
    pid: int
    sequence: int
    counter: int
    output: str
    path: str
    observation: Observation


@boundary
def select_twins(rows: tuple[tuple[Path, Node], ...]) -> tuple[Twin, ...]:
    groups: dict[str, list[Twin]] = {}
    controlled: dict[str, set[str]] = {}
    identities: set[tuple[int, int]] = set()
    for path, node in rows:
        output = Path(text(node.field('local_output')))
        if not output.name.startswith('gate-observer-'):
            continue
        fields(node, 'schema process_id world_sequence thread_name local_output local_sidecar observation')
        node.field('schema').equals('worldsim_gate_test_observation_v1')
        pid, sequence = integer(node.field('process_id')), integer(node.field('world_sequence'))
        require(pid > 0 and path.name == f'probe_pid{pid}_world{sequence}.json', '/identity', 'mirror identity mismatch')
        require((pid, sequence) not in identities, '/identity', 'duplicate native sequence')
        identities.add((pid, sequence))
        match = re.fullmatch(r'gate-observer-' + str(pid) + r'-(0|[1-9][0-9]*)', output.name)
        require(match is not None and output.is_absolute() and '..' not in output.parts
                and str(output) == text(node.field('local_output')), '/output', 'noncanonical source-local output')
        assert match is not None
        counter = int(match.group(1))
        thread = node.field('thread_name').value
        if isinstance(thread, str):
            require(thread in DIRECT, '/thread', 'unclassified named direct observer case')
            continue
        node.field('thread_name').equals(None)
        observation_node = node.field('observation')
        scene = text(observation_node.field('scene'))
        occurrence = text(observation_node.field('occurrence'))
        observation_node.field('seed').equals(0)
        node.field('local_sidecar').equals(str(output / 'gate_observations' / f'{scene}_s0_{occurrence}.json'))
        # This intentional-loss fixture has its own unresolved safety obligation.
        if observation_node.field('requested_ticks').value == 2880:
            require(scene == 'colony' and occurrence in {'primary', 'determinism_twin'}, '/debug-kill', 'unknown controlled family')
            _ = parse_probe(json.dumps(observation_node.value), ProbeWorld(scene, 0, occurrence, 2880))
            checkpoints = observation_node.field('checkpoints').array()
            require(len(checkpoints) == 2, '/debug-kill', 'missing intervention evidence')
            for checkpoint, phase in zip(checkpoints, ('before_tick_commands', 'before_debug_kill'), strict=True):
                checkpoint.field('phase').equals(phase)
                checkpoint.field('tick').equals(1800)
                checkpoint.field('script_index').equals(0)
                checkpoint.field('script_spec').field('cmd').equals('debug_kill')
            occurrences = controlled.setdefault(str(output), set())
            require(occurrence not in occurrences, '/debug-kill', 'duplicate controlled occurrence')
            occurrences.add(occurrence)
            continue
        require(scene in CASES and occurrence in {'primary', 'determinism_twin'}, '/world', 'unclassified twin world')
        observation = parse_probe(json.dumps(observation_node.value), ProbeWorld(scene, 0, occurrence, 1440))
        require(observation.knowledge_enabled == (scene == 'colony'), '/knowledge', 'source scene capability differs')
        require(observation.checkpoints_json == '[]', '/checkpoints', 'twins have no intervention')
        groups.setdefault(str(output), []).append(Twin(scene, '', occurrence, pid, sequence, counter,
                                                      str(output), str(path), observation))
    require(len(controlled) <= 1 and all(value == {'primary', 'determinism_twin'} for value in controlled.values()),
            '/debug-kill', 'ambiguous controlled output retained outside twins')
    require(len({pid for pid, _ in identities}) == 1, '/pid', 'source-prefix mirrors span library emitters')
    require(len(groups) == 4, '/twins', 'missing or extra source output groups')
    by_scene: dict[str, list[list[Twin]]] = {scene: [] for scene in CASES}
    for members in groups.values():
        require(len(members) == 2 and {row.occurrence for row in members} == {'primary', 'determinism_twin'},
                '/group', 'missing or duplicate occurrence')
        require(len({(row.scene, row.pid, row.counter) for row in members}) == 1,
                '/group', 'mixed source output group')
        ordered = sorted(members, key=lambda row: row.occurrence == 'determinism_twin')
        require(ordered[0].sequence < ordered[1].sequence, '/order', 'twin precedes primary')
        by_scene[ordered[0].scene].append(ordered)
    require(len({row.pid for members in groups.values() for row in members}) == 1,
            '/pid', 'twins require one library emitter')
    require(len({members[0].counter for members in groups.values()}) == 4,
            '/counter', 'source Temp counter reused across outputs')
    result: list[Twin] = []
    from dataclasses import replace
    for scene in sorted(by_scene):
        runs = sorted(by_scene[scene], key=lambda members: members[0].counter)
        require(len(runs) == 2, '/scene', 'missing or extra scene-specific group')
        require(max(row.sequence for row in runs[0]) < min(row.sequence for row in runs[1]),
                '/order', 'observed run began before ordinary returned')
        for label, members in zip(('ordinary', 'observed'), runs, strict=True):
            result.extend(replace(row, run=label) for row in members)
    return tuple(result)


@boundary
def pair_twins(left: tuple[Twin, ...], right: tuple[Twin, ...]) -> dict[str, Json]:
    require(len(left) == len(right) == 8, '/pair', 'eight actual worlds required per arm')
    safety: list[Json] = []
    coverage: list[Json] = []
    for base, candidate in zip(left, right, strict=True):
        require((base.scene, base.run, base.occurrence) == (candidate.scene, candidate.run, candidate.occurrence),
                '/pair', 'source occurrence mismatch')
        safety.append(ordinary_pair(base.observation, candidate.observation))
        coverage.append({'target': 'sim_test', 'test': CASES[base.scene], 'run': base.run,
                         'occurrence': base.occurrence, 'scene': base.scene, 'seed': 0, 'ticks': 1440,
                         'base_emitted_pid': base.pid, 'candidate_emitted_pid': candidate.pid,
                         'base_sequence': base.sequence, 'candidate_sequence': candidate.sequence,
                         'base_temp_counter': base.counter, 'candidate_temp_counter': candidate.counter,
                         'base_output': base.output, 'candidate_output': candidate.output,
                         'base_raw_path': base.path, 'candidate_raw_path': candidate.path})
    return {'status': 'PASS', 'safety': safety, 'coverage': coverage, 'library_complete': False,
            'observer_neutrality': 'NOT_ESTABLISHED_BOTH_RUNS_ENVIRONMENT_ENABLED',
            'unresolved': ['named direct mechanisms', 'debug-kill fixture and preliminary selection world',
                           'baseline seed7 scene fixture and other sim_test worlds']}
