"""Summary equality is required in addition to safe world endpoints."""
import json
import tempfile
import unittest
from pathlib import Path

import read_summary_normalise as normalizer
from generic_declarations import Declarations
from generic_gate import GateContractError, document
from generic_process import file_hash
from mux2_gate_inputs import Node

import generic_whole_gate_comparison as subject
import test_generic_whole_gate_observations as fixtures
from generic_whole_gate_probe import probe_safety


class ComparisonTests(unittest.TestCase):
    def test_safe_but_changed_ordinary_summary_fails_zero_declaration_proof(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp).resolve()
            fixture=fixtures.ProbeRoutingTests()
            registry,plans=fixture.fixture(root)
            left,right=(fixture.collect(registry,p) for p in plans)
            for plan in plans:
                _ = (plan.out/'summary.json').write_text(json.dumps({'scene':'colony','days':10,'seeds':[{'seed':seed,'days':10} for seed in plan.seeds],'test_value':1}))
            module=Path(normalizer.__file__).resolve()
            session=document(json.dumps({'acceptance':{'normalizer':{'path':str(module),'sha256':file_hash(module)}}}))
            declarations=Declarations('b'*40,'a'*40,'self_proof',('colony.u10',),(),(),(),(),'c'*64)
            def stage(path: Path) -> Node:
                return document(json.dumps({'observer_artifacts':{str(p):file_hash(p) for p in path.rglob('*') if p.is_file()}}))
            safety=probe_safety(left,right)
            result=subject.compare(session,declarations,stage(plans[0].out),stage(plans[1].out),left,right,safety)
            self.assertTrue(result['semantic_equal'])
            _ = (plans[1].out/'summary.json').write_text(json.dumps({'scene':'colony','days':10,'seeds':[{'seed':seed,'days':10} for seed in plans[1].seeds],'test_value':2}))
            with self.assertRaisesRegex(GateContractError,'undeclared fixture difference'):
                _ = subject.compare(session,declarations,stage(plans[0].out),stage(plans[1].out),left,right,safety)
