"""Pure source classification and retained aggregate-output regression checks."""
import re
import shutil
import tempfile
import unittest
from pathlib import Path
from typing import final
from unittest.mock import patch

import generic_process
from gate_test_summary import parse_log
from generic_whole_gate_doctests import EXPECTED, classify_doctests, source_closure
from mux2_gate_inputs import Json

ROOT = Path(generic_process.__file__).resolve().parents[2]


def actual_sections() -> list[Json]:
    raw = (Path(__file__).parent / 'fixtures/original_doctests.txt').read_text()
    parts = re.split(r'^\s*Doc-tests (\w+)\s*$', raw, flags=re.MULTILINE)
    rows: list[Json] = []
    for index in range(1, len(parts), 2):
        summary = parse_log(parts[index + 1])
        assert summary.passed == summary.failed == summary.ignored == summary.filtered_out == 0
        assert summary.summaries == 1 and not summary.outcomes
        rows.append({'package': parts[index].replace('_', '-'), 'kind': 'doctest', 'cases': []})
    return rows


@final
class DoctestTests(unittest.TestCase):
    def __init__(self, methodName: str = 'runTest') -> None:
        super().__init__(methodName)
        self.inputs = {name: digest for package in EXPECTED
                       for name, digest in source_closure(ROOT, package).items()}
        self.rows = actual_sections()

    def test_retained_actual_groups_and_source_review(self) -> None:
        result = classify_doctests(ROOT, self.inputs, self.rows)
        self.assertEqual(len(result), 5)
        self.assertTrue(all(isinstance(row, dict) and row['status'] == 'PASS' for row in result))

    def test_nonempty_actual_snippet_is_incomplete(self) -> None:
        self.rows[0] = {'package': 'sim-bridge', 'kind': 'doctest',
                        'cases': [{'name': 'src/lib.rs - (line 1)', 'status': 'ok'}]}
        result = classify_doctests(ROOT, self.inputs, self.rows)
        first = result[0]
        assert isinstance(first, dict)
        self.assertEqual(first['status'], 'INCOMPLETE')

    def test_duplicate_missing_unknown_groups_reject(self) -> None:
        for rows in (self.rows[:-1], self.rows + self.rows[:1],
                     self.rows[:-1] + [{'kind': 'doctest', 'package': 'foreign', 'cases': []}]):
            with self.subTest(rows=rows), self.assertRaises(ValueError):
                _ = classify_doctests(ROOT, self.inputs, rows)

    def test_unbound_source_rejects(self) -> None:
        _ = self.inputs.pop('rust/crates/sim-engine/src/lib.rs')
        with self.assertRaises(ValueError):
            _ = classify_doctests(ROOT, self.inputs, self.rows)

    def test_changed_admitted_source_is_incomplete(self) -> None:
        original = source_closure
        name = 'rust/crates/sim-engine/src/lib.rs'
        self.inputs[name] = '0' * 64
        def changed(root: Path, package: str) -> dict[str, str]:
            result = original(root, package)
            if name in result:
                result[name] = '0' * 64
            return result
        with patch('generic_whole_gate_doctests.source_closure', side_effect=changed):
            result = classify_doctests(ROOT, self.inputs, self.rows)
        selected = next(row for row in result if isinstance(row, dict) and row['package'] == 'sim-engine')
        assert isinstance(selected, dict)
        self.assertEqual(selected['status'], 'INCOMPLETE')

    def test_new_source_is_in_inventory(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            for name in source_closure(ROOT, 'sim-engine'):
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                _ = shutil.copyfile(ROOT / name, path)
            added = 'rust/crates/sim-engine/src/new_doc.rs'
            _ = (root / added).write_text('/// ```\n/// let world = ();\n/// ```\npub struct New;\n')
            self.assertIn(added, source_closure(root, 'sim-engine'))


if __name__ == '__main__':
    _ = unittest.main()
