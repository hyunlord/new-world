"""Receipt parser controls; synthetic logs are parser fixtures, never Godot evidence."""
import unittest

from mux2_tree_conversion_probe_check import validate_log


def valid_log(unique_mode: bool = False) -> str:
    lines = ["Godot Engine startup", "MUX2_TREE_BRIDGE_PASS probe_explicitly_enabled",
             "MUX2_TREE_BRIDGE_PASS actual_content_bootstrap", "MUX2_TREE_BRIDGE_PASS actual_bridge_factory"]
    shapes = [(0, 0), (1, 1), (4096, 1), (4096, 3), (128, 128), (3, 2), (5, 4)]
    for call in range(4):
        if call == 2:
            lines.append("MUX2_TREE_BRIDGE_PASS world_reset")
        lines.append(f"MUX2_TREE_CONVERSION actual version={call // 2} rows=10 unique=2 conversions={2 if unique_mode else 10} exact=true")
        for fixture, (rows, unique) in enumerate(shapes):
            lines.append(f"MUX2_TREE_CONVERSION fixture={fixture} rows={rows} unique={unique} conversions={unique if unique_mode else rows} exact=true")
        lines.append("MUX2_TREE_CONVERSION negative_order_detected=true")
        for label in ("exact_schema", "aligned_columns", "original_tile_order", "actual_version"):
            lines.append(f"MUX2_TREE_BRIDGE_PASS call{call}_{label}")
        if call % 2:
            lines.append(f"MUX2_TREE_BRIDGE_PASS call{call}_repeated_exact_snapshot")
    lines.append("MUX2_TREE_BRIDGE_DONE checks=22 failures=0")
    return "\n".join(lines)


class ReceiptTests(unittest.TestCase):
    def test_baseline_and_candidate_accept_their_real_count_contract(self) -> None:
        self.assertEqual(validate_log(valid_log()), [])
        self.assertEqual(validate_log(valid_log(True), True), [])

    def test_optimization_expectation_is_red_on_baseline(self) -> None:
        self.assertTrue(validate_log(valid_log(), True))

    def test_occurrence_expectation_is_red_on_candidate(self) -> None:
        self.assertTrue(validate_log(valid_log(True)))

    def test_duplicate_fixture_cannot_replace_missing_fixture_at_same_total(self) -> None:
        self.assertTrue(validate_log(valid_log().replace("fixture=6 rows=5 unique=4 conversions=5", "fixture=5 rows=3 unique=2 conversions=3", 1)))

    def test_fixture_shape_is_independent_of_reported_counts(self) -> None:
        self.assertTrue(validate_log(valid_log().replace("fixture=2 rows=4096 unique=1 conversions=4096", "fixture=2 rows=1 unique=1 conversions=1", 1)))

    def test_fixture_id_out_of_range_rejected(self) -> None:
        self.assertTrue(validate_log(valid_log().replace("fixture=6", "fixture=7", 1)))

    def test_missing_and_extra_rows_rejected(self) -> None:
        lines = valid_log().splitlines()
        self.assertTrue(validate_log("\n".join(lines[:-2] + lines[-1:])))
        self.assertTrue(validate_log(valid_log() + "\n" + lines[5]))

    def test_per_call_boundary_cannot_be_reordered(self) -> None:
        lines = valid_log().splitlines()
        a = next(i for i, line in enumerate(lines) if "negative_order" in line)
        lines[a], lines[a + 1] = lines[a + 1], lines[a]
        self.assertTrue(validate_log("\n".join(lines)))

    def test_extra_false_negative_control_rejected(self) -> None:
        self.assertTrue(validate_log(valid_log() + "\nMUX2_TREE_CONVERSION negative_order_detected=false"))

    def test_false_or_duplicate_done_rejected(self) -> None:
        self.assertTrue(validate_log(valid_log().replace("failures=0", "failures=1")))
        self.assertTrue(validate_log(valid_log() + "\nMUX2_TREE_BRIDGE_DONE checks=22 failures=0"))

    def test_unknown_duplicate_missing_and_invalid_fields_rejected(self) -> None:
        for replacement in ("rows=-1", "rows=abc", "rows=10 rows=10", "", "rows=10 extra=1"):
            with self.subTest(replacement=replacement):
                self.assertTrue(validate_log(valid_log().replace("rows=10", replacement, 1)))

    def test_false_exact_and_invalid_cardinality_rejected(self) -> None:
        self.assertTrue(validate_log(valid_log().replace("exact=true", "exact=false", 1)))
        self.assertTrue(validate_log(valid_log().replace("rows=10 unique=2", "rows=10 unique=11", 1)))

    def test_failed_check_cannot_hide_behind_successful_done(self) -> None:
        self.assertTrue(validate_log(valid_log().replace("PASS actual_content_bootstrap", "FAIL actual_content_bootstrap")))

    def test_prefixed_or_unknown_protocol_lines_rejected(self) -> None:
        self.assertTrue(validate_log(valid_log().replace("MUX2_TREE_CONVERSION actual", "junk MUX2_TREE_CONVERSION actual", 1)))
        self.assertTrue(validate_log(valid_log() + "\nMUX2_TREE_CONVERSION unknown=1"))

    def test_runtime_errors_rejected(self) -> None:
        self.assertTrue(validate_log(valid_log() + "\nSCRIPT ERROR: fixture failed"))


if __name__ == "__main__":
    unittest.main()
