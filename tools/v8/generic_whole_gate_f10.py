# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -B -m unittest test_generic_f10
"""Original F10 intervention boundaries plus independently executed no-kill control."""
import json
from pathlib import Path

from generic_f10 import companion_delta, injection, paired_precondition
from generic_f10_run import NO_KILL, SCRIPT
from generic_gate import document, integer, require, text
from generic_process import file_hash, real_path
from mux2_gate_inputs import Json, Node

from generic_whole_gate_probe import ProbeEvidence, probe_safety, recheck_probe


def bound_output(stage: Node, path: Path) -> str:
    """Read a producer-completion anchor, never adopt the current file as authority."""
    real_path(path, directory=False)
    digest = text(stage.field('observer_artifacts').field(str(path)))
    require(file_hash(path) == digest, str(path), 'original F10 output changed')
    raw = path.read_text()
    require(file_hash(path) == digest, str(path), 'F10 output changed during read')
    return raw


def evaluate(session: Node, base_stage: Node, candidate_stage: Node,
             base: ProbeEvidence, candidate: ProbeEvidence,
             base_control: ProbeEvidence, candidate_control: ProbeEvidence) -> tuple[Json, ...]:
    """F10 terminal losses remain fault evidence; ordinary controls still require zero."""
    require(base.plan.fixture_id == candidate.plan.fixture_id == 'knowledge.f10_injected', '/f10', 'wrong injected fixture')
    require(base_control.plan.fixture_id == candidate_control.plan.fixture_id == 'knowledge.f10_no_kill', '/f10', 'missing no-kill control')
    require(base.plan.arm == base_control.plan.arm == 'base' and
            candidate.plan.arm == candidate_control.plan.arm == 'candidate', '/f10', 'wrong arms')
    scripts: list[str] = []
    summaries: list[Node] = []
    for arm, stage, observed in (('base', base_stage, base), ('candidate', candidate_stage, candidate)):
        root_node = session.field('arms').field(arm)
        root = Path(text(root_node.field('root')))
        values: list[str] = []
        for name in (SCRIPT, NO_KILL):
            path = root / name
            digest = text(root_node.field('source_inputs').field(name))
            require(file_hash(path) == digest, name, 'F10 script differs from bound source')
            values.append(path.read_text())
        companion_delta(values[0], values[1])
        scripts.append(values[0])
        summary = document(bound_output(stage, observed.plan.out / 'summary.json'))
        summary.field('scene').equals('colony')
        summary.field('days').equals(4)
        summary_rows = summary.field('seeds').array()
        require(tuple(integer(row.field('seed')) for row in summary_rows) == observed.plan.seeds, '/f10', 'summary seed coverage changed')
        for row in summary_rows:
            row.field('days').equals(4)
        summaries.append(summary)
        directory = observed.plan.out / 'commands'
        names = {str(directory / f'colony_s{seed}_commands.json') for seed in observed.plan.seeds}
        require({name for name in stage.field('observer_artifacts').mapping() if Path(name).parent == directory} == names,
                '/f10', 'missing or extra command logs')
        require({str(p) for p in directory.iterdir()} == names, '/f10', 'command namespace changed')
        recheck_probe(observed)
    require(scripts[0] == scripts[1], '/f10', 'cross-arm fault script changed')
    rows: list[Json] = []
    for left, right in zip(base.worlds, candidate.worlds, strict=True):
        require((left.seed, left.role) == (right.seed, right.role), '/f10', 'mismatched fault occurrences')
        preconditions = tuple(injection(world.observation, scripts[index],
            bound_output(stage, evidence.plan.out / f'commands/colony_s{world.seed}_commands.json'),
            json.dumps(summaries[index].field('seeds').array()[world.seed].value))
            for index, (stage, evidence, world) in enumerate(((base_stage, base, left), (candidate_stage, candidate, right))))
        rows.append({'seed': left.seed, 'fault_precondition': paired_precondition(*preconditions)})
    rows.extend(probe_safety(base_control, candidate_control))
    for arm, stage, evidence in (('base', base_stage, base), ('candidate', candidate_stage, candidate)):
        for name in stage.field('observer_artifacts').mapping():
            if Path(name).is_relative_to(evidence.plan.out):
                _ = bound_output(stage, Path(name))
        root_node = session.field('arms').field(arm)
        for name in (SCRIPT, NO_KILL):
            require(file_hash(Path(text(root_node.field('root'))) / name) ==
                    text(root_node.field('source_inputs').field(name)), name, 'F10 source changed during evaluation')
    for evidence in (base, candidate, base_control, candidate_control):
        recheck_probe(evidence)
    return tuple(rows)
