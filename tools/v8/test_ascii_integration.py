"""Routing-only mocks; world source and history semantics have separate tests."""
import unittest
from unittest.mock import patch

import generic_whole_gate_native as native
from mux2_gate_inputs import Node
from test_generic_whole_gate_workspace_integration import accounting


class AsciiIntegrationTests(unittest.TestCase):
    def test_original_workspace_routes_keep_all_broader_and_event_gaps(self):
        session=Node({'arms':{'base':{'root':'/synthetic','source_inputs':{}},
                              'candidate':{'root':'/synthetic','source_inputs':{}}}})
        for step in (4,5):
            result_scope={'status':'INCOMPLETE','safety':[{'synthetic':True} for _ in range(7)],
                          'coverage':[{'target':'engine_ascii'} for _ in range(8)],
                          'unsupported':['carrier actual first-success horizons differ']}
            base=Node({'step':step,'arm':'base'});candidate=Node({'step':step,'arm':'candidate'})
            with patch('generic_whole_gate_workspace.evaluate_family', return_value=accounting()), \
                 patch('generic_whole_gate_knowledge_defaults.evaluate_knowledge_defaults', return_value={'safety': [], 'coverage': []}), \
                 patch('generic_whole_gate_colony_ffi.evaluate_family', return_value={'status': 'PASS', 'safety': [], 'coverage': []}), \
                 patch('generic_whole_gate_baseline_observer.evaluate_family', return_value={'safety': [], 'coverage': []}), \
                 patch('generic_whole_gate_twins.evaluate_family', return_value={'safety': [], 'coverage': []}), \
                 patch.object(native,'STEWARD_PINS',{}), patch.dict(native.TARGETS,{step:()}), \
                 patch.object(native,'aggregate_output',return_value=('',1)), \
                 patch('generic_whole_gate_native_probes.evaluate_replays',return_value={'safety':[],'coverage':[]}), \
                 patch('generic_whole_gate_appearance.evaluate_appearance',return_value={'safety':[],'coverage':[]}), \
                 patch('generic_whole_gate_ascii.evaluate_ascii',return_value=result_scope) as selected:
                result=native.evaluate_family(session,base,candidate)
                selected.assert_called_once_with(session,base,candidate)
                self.assertEqual(result['status'],'INCOMPLETE')
                self.assertEqual(len(result['unsupported']),5)
                self.assertIn('carrier actual first-success horizons differ', result['unsupported'])
                self.assertEqual(len(result['safety']),7)
                self.assertEqual(len(result['coverage']),8)
                self.assertEqual(result['ascii'],result_scope)

if __name__=='__main__':unittest.main()
