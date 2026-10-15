import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from compare_identity_pass17 import compare


class IdentityComparison(unittest.TestCase):
    def test_equal_nested_arrays_and_empty_containers(self) -> None:
        value = {'worlds': [{'empty': [], 'map': {}, 'values': [1, 1.0, True, None]}]}
        self.assertEqual(compare(value, value), [])

    def test_missing_and_extra_keys_fail_in_both_directions(self) -> None:
        for value in ([], {}, 0, None):
            with self.subTest(value=value):
                self.assertTrue(compare({'value': value}, {}))
                self.assertTrue(compare({}, {'value': value}))

    def test_empty_container_types_and_list_lengths_fail(self) -> None:
        self.assertTrue(compare([], {}))
        self.assertTrue(compare([], [[]]))
        self.assertTrue(compare({'x': []}, {'x': {}}))

    def test_scalar_types_are_not_coerced(self) -> None:
        for left, right in ((True, 1), (1, 1.0), ('1', 1), (None, False)):
            with self.subTest(left=left, right=right):
                self.assertTrue(compare(left, right))

    def test_float_comparison_is_exact_including_signed_zero(self) -> None:
        self.assertTrue(compare(1.0, 1.0000000000000002))
        self.assertTrue(compare(-0.0, 0.0))
        self.assertEqual(compare(0.1, float('0.1')), [])

    def test_array_order_is_preserved(self) -> None:
        self.assertTrue(compare([1, 2], [2, 1]))

    def test_only_named_timing_keys_are_excluded_recursively(self) -> None:
        self.assertEqual(compare({'timing': {}, 'world': {'wall_seconds': 1, 'ms_per_agent_tick': 2, 'alive': 3}}, {'world': {'alive': 3}}), [])
        self.assertTrue(compare({'time': 1}, {'time': 2}))

    def test_object_order_and_delimiters_do_not_change_paths(self) -> None:
        self.assertEqual(compare({'a': 1, 'b': 2}, {'b': 2, 'a': 1}), [])
        self.assertTrue(compare({'a/b': 1}, {'a': {'b': 1}}))

    def test_cli_reports_structural_failure(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            reference, candidate = root / 'ref.json', root / 'cand.json'
            reference.write_text(json.dumps({'empty': []}))
            candidate.write_text('{}')
            result = subprocess.run([sys.executable, str(Path(__file__).with_name('compare_identity_pass17.py')), str(reference), str(candidate)], text=True, capture_output=True, check=False)
            self.assertEqual(result.returncode, 1)
            self.assertIn('FAIL', result.stdout)

    def test_cli_rejects_nonfinite_json(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / 'invalid.json'
            source.write_text('{"value": NaN}')
            result = subprocess.run([sys.executable, str(Path(__file__).with_name('compare_identity_pass17.py')), str(source), str(source)], text=True, capture_output=True, check=False)
            self.assertEqual(result.returncode, 2)


if __name__ == '__main__':
    unittest.main()
