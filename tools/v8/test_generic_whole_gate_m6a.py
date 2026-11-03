"""Synthetic schema joins over original closure; not native qualification."""
import json
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path

from generic_fixture_registry import ExpansionContext, expand, load_registry
from generic_gate import GateContractError
from generic_m6a_run import DURATION
from generic_process import file_hash
from test_generic_m6a_observations import fixed_report, sidecar

import generic_whole_gate_m6a as subject
from generic_whole_gate_paired import Closure


class M6aJoins(unittest.TestCase):
    def test_actual_report_and_original_sidecar_must_share_fixed_horizon(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp).resolve()
            registry = load_registry(Path(__file__).parent / 'gate_fixtures.json')
            plan = next(p for p in expand(registry,ExpansionContext(root,root,root/'runs',root/'probe',root/'probe'))
                        if p.fixture_id=='m6a.food72' and p.arm=='base')
            plan = replace(plan,seeds=(0,))
            directory = plan.out/'observed/gate_observations'
            directory.mkdir(parents=True)
            raw = sidecar(end=DURATION, maximum=DURATION)
            report = fixed_report()
            report.update(days=72,end_tick=DURATION,observed_ticks=DURATION,invariant_checks=2*DURATION)
            path = directory/'m6a_s0_primary.json'
            _ = path.write_text(json.dumps(raw))
            _ = (directory.parent/'summary.json').write_text(json.dumps(report))
            original = Closure(root,root/'acceptance.json',tuple((str(p),file_hash(p)) for p in root.rglob('*') if p.is_file()),
                               tuple((str(p),p.lstat().st_mode) for p in (root,*root.rglob('*'))))
            self.assertEqual(subject.observe(original,plan).world.terminal_tick,DURATION)
            _ = path.write_text('{}')
            with self.assertRaisesRegex(GateContractError,'bytes changed'):
                _ = subject.observe(original,plan)
