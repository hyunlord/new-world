"""Wood-sheet guards retain native budgets, cycle checks and literal cell order."""
import unittest
from PIL import Image
from production_sheets import SHEETS
from v4_contracts import FrameCandidate, FrameSpec, guard_frame, guard_group, target_size
from wood_candidates_p4 import BODIES, cells


class WoodCandidatesTests(unittest.TestCase):
    def test_cell_order_and_runtime_definitions(self) -> None:
        for sheet in BODIES:
            rows = cells(sheet)
            self.assertEqual([row['cell'] for row in rows], list(range(1,17)))
            self.assertEqual(len(SHEETS[sheet]), 16)
            for row, spec in zip(rows, SHEETS[sheet]):
                self.assertEqual((row['activity'], row['view'], row['frame']), (spec.activity, spec.view, spec.frame))
        self.assertEqual([row['view'] for row in cells('V_a4')[12:]], ['S','S','E','E'])
        self.assertEqual([cells('W1')[i]['name'] for i in (0,2,4,6)],
                         [f'deadwood_stage_{i}_variant_0' for i in (3,2,1,0)])

    def test_object_caps_and_exact_icon_canvas(self) -> None:
        for index in range(1,17):
            frame = FrameSpec('W1', index, 1)
            expected = (24,16) if index <= 8 else ((16,16) if index in (12,16) else (24,20))
            self.assertEqual(target_size(frame), expected)
            candidate = FrameCandidate(frame, Image.new('RGBA', expected, (120,80,40,255)), 0,100,True,0,0)
            self.assertTrue(guard_frame(candidate).passed)
            oversized = FrameCandidate(frame, Image.new('RGBA',(expected[0]+1,expected[1]),(120,80,40,255)),0,100,True,0,0)
            self.assertIn('geometry', guard_frame(oversized).errors)
        icon = FrameCandidate(FrameSpec('W1',12,1), Image.new('RGBA',(15,16),(120,80,40,255)),0,100,True,0,0)
        self.assertIn('icon_canvas', guard_frame(icon).errors)

    def test_wood_cycles_require_real_leg_changes(self) -> None:
        candidates = [FrameCandidate(FrameSpec('V_a4', i, 1, 'adult_a','S','carry_wood'),
                      Image.new('RGBA',(20,28),(120,80,40,255)),0,100,True,0,0) for i in range(1,5)]
        self.assertIn('leg_difference', guard_group(candidates).errors)
        self.assertIn('pending_cycle_completion', guard_group(candidates[:3]).warnings)
        short = FrameCandidate(candidates[0].frame, Image.new('RGBA',(20,24),(120,80,40,255)),0,100,True,0,0)
        self.assertIn('standing_height', guard_frame(short).errors)


if __name__ == '__main__':
    unittest.main()
