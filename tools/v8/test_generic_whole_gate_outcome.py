# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate_outcome
"""Pure policy join fixtures; these are not native capture or publication evidence."""
import tempfile
import unittest
from pathlib import Path

from mux2_gate_inputs import Json, Node

from generic_process import atomic_json, file_hash
from generic_whole_gate_journal import ORDER, REFERENCE
from generic_whole_gate_outcome import arm_clean


def fixture(root: Path, arm: str, postcheck: int = 0) -> tuple[Node, tuple[Node, ...], Node]:
    session = Node({'output': str(root)})
    rows: list[Node] = []
    for step in REFERENCE if arm == 'base' else ORDER:
        code = 1 if step == 41 else 0
        process = root/f'{step}.json'
        atomic_json(process, {'native_exit': code, 'errors': [] if code == 0 else ['native exit 1'],
                              'status': 'PASS' if code == 0 else 'FAIL'})
        rows.append(Node({'step': step, 'state': 'RETURNED', 'native_exit': code,
                          'process_receipt': str(process), 'process_receipt_sha256': file_hash(process)}))
    (root/arm).mkdir()
    atomic_json(root/arm/'shell-summary.json', {'arm': arm, 'postcheck_status': postcheck, 'stage_status': 1})
    return session, tuple(rows), Node({'native_exit': 1, 'errors': ['native exit 1'], 'status': 'FAIL'})


def evaluation(waived: list[Json], reference: list[Json]) -> Node:
    return Node({'status': 'PASS', 'waived_stage_exits': waived, 'reference_stage_exits': reference})


class OutcomeTests(unittest.TestCase):
    def test_raw_candidate_one_requires_exact_qualified_waiver(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            session, rows, shell = fixture(Path(temporary).resolve(), 'candidate')
            self.assertFalse(arm_clean(session, 'candidate', rows, shell, evaluation([], [])))
            self.assertTrue(arm_clean(session, 'candidate', rows, shell,
                evaluation([{'arm': 'candidate', 'step': 41, 'native_exit': 1}], [])))
            self.assertEqual(shell.field('native_exit').value, 1)

    def test_waiver_cannot_hide_postcheck_failure(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            session, rows, shell = fixture(Path(temporary).resolve(), 'candidate', 1)
            self.assertFalse(arm_clean(session, 'candidate', rows, shell,
                evaluation([{'arm': 'candidate', 'step': 41, 'native_exit': 1}], [])))

    def test_waiver_cannot_hide_source_error(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            session, rows, _ = fixture(Path(temporary).resolve(), 'candidate')
            shell = Node({'native_exit': 1, 'errors': ['native exit 1', 'postflight source changed'], 'status': 'FAIL'})
            self.assertFalse(arm_clean(session, 'candidate', rows, shell,
                evaluation([{'arm': 'candidate', 'step': 41, 'native_exit': 1}], [])))

    def test_base_reference_is_distinct_from_candidate_waiver(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            session, rows, shell = fixture(Path(temporary).resolve(), 'base')
            self.assertTrue(arm_clean(session, 'base', rows, shell,
                evaluation([], [{'arm': 'base', 'step': 41, 'native_exit': 1}])))
            with self.assertRaises(ValueError):
                _ = arm_clean(session, 'base', rows, shell,
                    evaluation([{'arm': 'base', 'step': 41, 'native_exit': 1}], []))


if __name__ == '__main__':
    _ = unittest.main()
