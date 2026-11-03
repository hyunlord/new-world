"""Routing-only mocks; actual mirror shape validation is a separate retained audit."""
import unittest
from unittest.mock import patch

import generic_whole_gate_native as native
from generic_observations import Observation
from generic_whole_gate_appearance import evaluate_appearance
from mux2_gate_inputs import Node
from test_generic_whole_gate_workspace_integration import accounting


class AppearanceIntegrationTests(unittest.TestCase):
    def test_stage4_and5_include_seven_rows_without_clearing_gaps(self):
        session=Node({'arms':{'base':{'root':'/synthetic','source_inputs':{}},
                              'candidate':{'root':'/synthetic','source_inputs':{}}}})
        appearance={'status':'PASS','safety':[{'synthetic':True} for _ in range(7)],
                    'coverage':[{'target':'engine_appearance_ffi'} for _ in range(7)]}
        for step in (4,5):
            base=Node({'step':step,'arm':'base'});candidate=Node({'step':step,'arm':'candidate'})
            with patch('generic_whole_gate_workspace.evaluate_family', return_value=accounting()), \
                 patch('generic_whole_gate_knowledge_defaults.evaluate_knowledge_defaults', return_value={'safety': [], 'coverage': []}), \
                 patch('generic_whole_gate_colony_ffi.evaluate_family', return_value={'status': 'PASS', 'safety': [], 'coverage': []}), \
                 patch('generic_whole_gate_baseline_observer.evaluate_family', return_value={'safety': [], 'coverage': []}), \
                 patch('generic_whole_gate_twins.evaluate_family', return_value={'safety': [], 'coverage': []}), \
                 patch.object(native,'STEWARD_PINS',{}), patch.dict(native.TARGETS,{step:()}), \
                 patch.object(native,'aggregate_output',return_value=('',1)), \
                 patch('generic_whole_gate_native_probes.evaluate_replays',return_value={'safety':[],'coverage':[]}), \
                 patch('generic_whole_gate_ascii.evaluate_ascii',return_value={'safety':[],'coverage':[],'unsupported':[]}), \
                 patch('generic_whole_gate_appearance.evaluate_appearance',return_value=appearance) as selected:
                result=native.evaluate_family(session,base,candidate)
                selected.assert_called_once_with(session,base,candidate)
                self.assertEqual(result['status'],'INCOMPLETE')
                self.assertTrue(result['unsupported'])
                self.assertEqual(len(result['safety']),7)
                self.assertEqual(len(result['coverage']),7)
                self.assertEqual(result['appearance'],appearance)

    def test_appearance_reports_exact_seven_source_occurrences(self):
        observations=tuple(Observation(0,end,10,10,index not in (3,4),(),'[]')
                           for index,end in enumerate((3000,2000,3000,100,4320,900,900)))
        with patch('generic_whole_gate_appearance.collect_appearance',return_value=observations):
            result=evaluate_appearance(Node({}),Node({'step':4,'arm':'base'}),Node({'step':4,'arm':'candidate'}))
        self.assertEqual(len(result['safety']),7)
        self.assertEqual(len(result['coverage']),7)
        self.assertEqual([row['branch'] for row in result['coverage']],
                         ['primary','twin','loaded','primary','primary','render','bridge-replay'])
        self.assertEqual(result['coverage'][4]['candidate_end_tick'],4320)

if __name__=='__main__':unittest.main()
