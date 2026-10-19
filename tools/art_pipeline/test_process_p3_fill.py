# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 -m unittest discover -s tools/art_pipeline -p test_process_p3_fill.py
"""Physical-cell coverage must not be inflated by derived or reference records."""
from copy import deepcopy
import json
import unittest

from process_p3_fill import ROOT, FillProcessingError, physical_sources


class PhysicalCoverageTest(unittest.TestCase):
    def setUp(self) -> None:
        self.records = json.loads((ROOT / 'docs/art/production2/fills/P2_fill_01_attempt_1.json').read_text())['candidates']

    def test_context_and_derivations_do_not_count_as_physical_attempts(self) -> None:
        # Given an existing ledger with original cells, derived cells and reference context.
        # When physical-cell coverage is selected.
        rows = physical_sources(self.records, 'P2_fill_01')
        # Then only the sixteen original row-major cells count.
        self.assertEqual([row['cell'] for row in rows], list(range(1,17)))
        self.assertTrue(all(row['source_kind'] == 'generated' for row in rows))

    def test_duplicate_physical_cell_cannot_replace_a_missing_cell(self) -> None:
        # Given a corrupt ledger repeating cell 1 instead of cell 16.
        rows = deepcopy(physical_sources(self.records, 'P2_fill_01'))
        rows[-1]['cell'] = 1
        # When coverage is selected, then the source is rejected before review.
        with self.assertRaises(FillProcessingError):
            physical_sources(rows, 'P2_fill_01')


if __name__ == '__main__':
    unittest.main()
