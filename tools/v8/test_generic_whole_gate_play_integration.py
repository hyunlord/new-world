"""Collection readiness is independent of actual play native qualification."""
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

from generic_whole_gate_play import QUALIFICATION_INPUTS
from mux2_gate_inputs import Json, Node

from generic_whole_gate_families import evaluate


def pending_rows() -> tuple[Json,...]:
    return tuple({'arm':arm,'qualification':'INCOMPLETE',
        'pending_native_qualification':list(QUALIFICATION_INPUTS),
        'actual_native_children':16,'original_case_exits':[3,2,4,0,0,0]}
        for arm in ('base','candidate'))


class PlayIntegrationTests(unittest.TestCase):
    def test_callable_join_runs_but_pending_is_not_safety_or_completed(self) -> None:
        with TemporaryDirectory() as folder:
            session=Node({'output':str(Path(folder).resolve())})
            steps={('base',21):Node({'arm':'base'}),('candidate',21):Node({'arm':'candidate'})}
            with patch('generic_whole_gate_play_join.evaluate_family',return_value=pending_rows()) as call:
                result=evaluate(session,steps)
            call.assert_called_once_with(session,steps['base',21],steps['candidate',21])
            self.assertEqual(result.errors,());self.assertEqual(result.safety,());self.assertEqual(result.completed,())
            self.assertEqual(len(result.incomplete),8)
            self.assertEqual(Node(result.coverage[0]).field('status').text(),'INCOMPLETE')

    def test_missing_original_arm_does_not_invoke_or_complete(self) -> None:
        with TemporaryDirectory() as folder,patch('generic_whole_gate_play_join.evaluate_family') as call:
            result=evaluate(Node({'output':str(Path(folder).resolve())}),{('candidate',21):Node({})})
        call.assert_not_called()
        self.assertEqual(result.completed,());self.assertEqual(result.safety,())
        self.assertIn('ui.play: missing paired original stage21 owner evidence',result.incomplete)

    def test_forged_qualified_row_is_rejected_without_safety(self) -> None:
        rows=pending_rows()
        first=dict(Node(rows[0]).mapping());first['qualification']='PRODUCT_QUALIFIED'
        with TemporaryDirectory() as folder,patch('generic_whole_gate_play_join.evaluate_family',return_value=(first,rows[1])):
            result=evaluate(Node({'output':str(Path(folder).resolve())}),
                {('base',21):Node({}),('candidate',21):Node({})})
        self.assertEqual(len(result.errors),1);self.assertEqual(result.safety,());self.assertEqual(result.completed,())


if __name__=='__main__':_=unittest.main()
