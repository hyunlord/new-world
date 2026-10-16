"""Recovery accounting distinguishes physical slots, attempts, and logical targets."""
import unittest
import recovery_candidates_p4 as recovery


class RecoveryTests(unittest.TestCase):
    def test_physical_approval_excludes_all_attempts_not_other_physical_sources(self):
        rows = [
            {'sprite_id': 'target', 'physical_slot_id': 'cell1', 'candidate_id': 'a', 'verdict': 'ok', 'reason': ''},
            {'sprite_id': 'target', 'physical_slot_id': 'cell1', 'candidate_id': 'b', 'verdict': 'reject', 'reason': 'region_map_wrong'},
            {'sprite_id': 'target', 'physical_slot_id': 'cell2', 'candidate_id': 'c', 'verdict': 'reject', 'reason': 'region_map_wrong'},
            {'sprite_id': 'target', 'physical_slot_id': 'cell2', 'candidate_id': 'd', 'verdict': 'reject', 'reason': 'region_map_wrong'},
        ]
        self.assertEqual([r['candidate_id'] for r in recovery.eligible_rows(rows)], ['c', 'd'])

    def test_unrelated_rejection_is_not_recovered(self):
        rows = [{'sprite_id': 'a', 'candidate_id': 'a', 'verdict': 'reject', 'reason': 'wrong_pose'}]
        self.assertEqual(recovery.eligible_rows(rows), [])

    def test_duplicate_attempt_is_rejected(self):
        row = {'sprite_id': 'a', 'candidate_id': 'a', 'verdict': 'reject', 'reason': 'region_map_wrong'}
        with self.assertRaises(recovery.RecoveryError):
            recovery.eligible_rows([row, row])


if __name__ == '__main__':
    unittest.main()
