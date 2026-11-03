"""Synthetic original-output F10 join; no runtime qualification."""
import json
import tempfile
import unittest
from pathlib import Path

import test_generic_f10 as f10_fixture
from generic_f10 import KILL
from generic_f10_run import NO_KILL, SCRIPT
from generic_fixture_registry import ExpansionContext, expand, load_registry
from generic_gate import GateContractError, document
from generic_process import file_hash
from mux2_gate_inputs import Json, Node
from test_generic_observations import receipt

import generic_whole_gate_f10 as subject
from generic_whole_gate_probe import ProbeEvidence, collect_probe


class F10JoinTests(unittest.TestCase):
    def test_fault_boundaries_and_real_control_occurrences_are_both_required(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp).resolve()
            registry = load_registry(Path(__file__).parent / 'gate_fixtures.json')
            observed, commands, summary = f10_fixture.F10Tests().inputs()
            arms: dict[str, Json] = {}
            stages: dict[str, Node] = {}
            evidence: dict[tuple[str,str], ProbeEvidence] = {}
            for arm in ('base','candidate'):
                source = root/arm
                (source/'tools/v8').mkdir(parents=True)
                _ = (source/SCRIPT).write_text(json.dumps([KILL]))
                _ = (source/NO_KILL).write_text('[]')
                arms[arm] = {'root':str(source),'source_inputs':{name:file_hash(source/name) for name in (SCRIPT,NO_KILL)}}
                stagefiles: dict[str,str] = {}
                for plan in expand(registry,ExpansionContext(source,source,root/'runs',source/'probe',source/'probe',source/NO_KILL)):
                    if plan.arm!=arm or not plan.fixture_id.startswith('knowledge.f10'):
                        continue
                    folder = plan.out/'gate_observations'
                    folder.mkdir(parents=True)
                    injected = plan.fixture_id=='knowledge.f10_injected'
                    seed_rows: list[Json] = []
                    for seed in plan.seeds:
                        raw = receipt()
                        raw.update(seed=seed,requested_ticks=5760,requested_terminal_tick=5760,terminal_tick=5760)
                        if injected:
                            raw.update(terminal_alive=9,knowledge_lost=[{'tick':1800,'node':'doorway','holder':9}],checkpoints=document('{"rows":'+observed.checkpoints_json+'}').field('rows').value)
                        path=folder/f'colony_s{seed}_primary.json'
                        _ = path.write_text(json.dumps(raw))
                        if injected:
                            directory=plan.out/'commands'
                            directory.mkdir(exist_ok=True)
                            _ = (directory/f'colony_s{seed}_commands.json').write_text(json.dumps(commands))
                            seed_rows.append(dict(summary,seed=seed,days=4))
                    if injected:
                        _ = (plan.out/'summary.json').write_text(json.dumps({'scene':'colony','days':4,'seeds':seed_rows}))
                    files=tuple((p.name,file_hash(p)) for p in folder.iterdir())
                    evidence[arm,plan.fixture_id]=collect_probe(registry,plan,files,folder.stat().st_mode)
                    stagefiles.update({str(p):file_hash(p) for p in plan.out.rglob('*') if p.is_file()})
                stages[arm]=document(json.dumps({'observer_artifacts':stagefiles}))
            session=document(json.dumps({'arms':arms}))
            args=(session,stages['base'],stages['candidate'],evidence['base','knowledge.f10_injected'],
                  evidence['candidate','knowledge.f10_injected'],evidence['base','knowledge.f10_no_kill'],evidence['candidate','knowledge.f10_no_kill'])
            self.assertEqual(len(subject.evaluate(*args)),16)
            command_path=evidence['candidate','knowledge.f10_injected'].plan.out/'commands/colony_s0_commands.json'
            _ = command_path.write_text('[]')
            with self.assertRaisesRegex(GateContractError,'changed'):
                _ = subject.evaluate(*args)
