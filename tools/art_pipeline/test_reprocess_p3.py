# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 -m unittest discover -s tools/art_pipeline -p test_reprocess_p3.py
"""Repository evidence coverage tests prevent silently omitting rejected attempts."""
import json
import unittest

from reprocess_p3 import ROOT, inventory


class ReprocessCoverageTest(unittest.TestCase):
    def test_every_unaccepted_physical_attempt_has_exactly_one_coverage_route(self) -> None:
        # Given the immutable Art Pass 2 final slot decisions and raw ledgers.
        old = ROOT / 'docs/art/production2'
        final = json.loads((old / 'final_selection.json').read_text())
        ledger = json.loads((old / 'candidates_round2.json').read_text())
        expected = {r['id'] for r in ledger['candidates'] if r['source_kind'] == 'generated'
                    and final['slots'][r['slot_id']]['disposition'] == 'unfilled'}
        # When the complete reprocessing inventory is built.
        result = inventory(ROOT)
        originals = [row['record']['id'] for row in result['sources'] if row['ledger'].endswith('/candidates_round2.json')]
        # Then all physical attempts are covered once, with N3 explicitly linked to P1.
        self.assertEqual(set(originals) | set(result['p1_n3_candidates']), expected)
        self.assertFalse(set(originals) & set(result['p1_n3_candidates']))
        self.assertEqual(len(originals), len(set(originals)))
        self.assertEqual(result['original_candidate_count'], 298)
        self.assertEqual(len(result['categories']['fill_rejected']), 49)
        self.assertEqual(len(result['categories']['pending_cycle_completion']), 14)
        self.assertEqual(len(result['categories']['no coherent guard-passing selected candidate']), 54)
        self.assertEqual(len(result['sources']), 366)

    def test_no_installed_fill_is_silently_reprocessed(self) -> None:
        # Given the frozen installation selection.
        selected = json.loads((ROOT / 'docs/art/production2/installation_selection.json').read_text())
        accepted = {row.get('candidate_id') for row in selected['slots'].values()}
        # When all reprocessing sources are selected.
        sources = inventory(ROOT)['sources']
        fills = [row['record']['id'] for row in sources if '/fills/' in row['ledger']]
        # Then every uninstalled physical fill remains visible and accepted installed cells are preserved.
        self.assertEqual(len(fills), 80)
        self.assertFalse(set(fills) & accepted)


if __name__ == '__main__':
    unittest.main()
