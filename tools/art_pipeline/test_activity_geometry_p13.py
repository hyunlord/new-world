"""Offline geometry tests; synthetic rectangles are never capture evidence."""
import unittest
import prepare_blind_crops_p13 as crops


class GeometryTests(unittest.TestCase):
    def geometry(self, body, icon, blockers=()):
        self.assertTrue(hasattr(crops, 'holder_geometry'), 'P12 body/icon geometry is not implemented')
        return crops.holder_geometry(body, icon, list(blockers))

    def test_union_grow_square_native(self):
        result = self.geometry([100, 100, 20, 40], [104, 80, 12, 12])
        self.assertEqual(result['box'], [72, 72, 148, 148])
        self.assertEqual(result['upscale'], 1)

    def test_icon_alone_majority_excludes(self):
        result = self.geometry([100, 100, 20, 40], [100, 80, 10, 10], [[100, 80, 6, 10]])
        self.assertEqual(result['body_covered_fraction'], 0)
        self.assertEqual(result['icon_covered_fraction'], .6)
        self.assertTrue(result['excluded'])

    def test_exact_half_allowed_and_union_not_double_counted(self):
        result = self.geometry([0, 0, 10, 10], [20, 0, 10, 10], [[0, 0, 5, 10]] * 2)
        self.assertEqual(result['body_covered_fraction'], .5)
        self.assertFalse(result['excluded'])

    def test_body_alone_majority_excludes(self):
        self.assertTrue(self.geometry([0, 0, 10, 10], [20, 0, 10, 10], [[0, 0, 6, 10]])['excluded'])

    def test_fractional_bounds_include_grown_union(self):
        result = self.geometry([100.25, 100.25, 20, 40], [104.25, 80.25, 12, 12])
        left, top, right, bottom = result['box']
        self.assertEqual(right-left, bottom-top)
        self.assertLessEqual(top, 72.25)
        self.assertGreaterEqual(bottom, 148.25)

    def test_independent_draw_ranks_and_foreign_only(self):
        row = {'node_path': '/holder', 'draw_rank': 1, 'icon_draw_rank': 4,
               'viewport_rect': [0, 0, 10, 10], 'icon_viewport_rect': [20, 0, 10, 10]}
        draw = [{'node_path': '/holder/load', 'draw_rank': 5, 'rect': [0, 0, 10, 10]},
                {'node_path': '/other', 'draw_rank': 3, 'rect': [20, 0, 10, 10]}]
        result = crops.row_geometry(row, draw)
        self.assertEqual(result['body_covered_fraction'], 0)
        self.assertEqual(result['icon_covered_fraction'], 0)
        row['icon_draw_rank'] = 2
        self.assertEqual(crops.row_geometry(row, draw)['icon_covered_fraction'], 1)

    def test_crowd_requires_25_and_zero_both(self):
        row = {'node_path': '/holder', 'draw_rank': 1, 'icon_draw_rank': 2,
               'viewport_rect': [0, 0, 10, 10], 'icon_viewport_rect': [20, 0, 10, 10],
               'entity_bits': 0, 'visible_in_tree': True, 'fully_inside_viewport': True}
        data = {'crowd_centre_tile': [0, 0], 'raw_agent_snapshot': {'ids': list(range(25)), 'xs': [0]*25, 'ys': [0]*25},
                'selected_entity_bits': 0, 'activity_holders': [row], 'draw_rects': [],
                'complete': True, 'errors': [], 'draw_order_schema': 3, 'draw_order_complete': True, 'zoom': 4}
        self.assertTrue(crops.crowd_proof(data)['zero_coverage_crowd_proven'])
        data['raw_agent_snapshot']['xs'][-1] = 10.001
        self.assertFalse(crops.crowd_proof(data)['zero_coverage_crowd_proven'])
        data['raw_agent_snapshot']['xs'][-1] = 10
        data['draw_rects'] = [{'node_path': '/other', 'draw_rank': 3, 'rect': [20, 0, .1, 10]}]
        self.assertFalse(crops.crowd_proof(data)['zero_coverage_crowd_proven'])

    def test_fixed_capture_worlds(self):
        from run_activity_captures_p13 import cases, InvalidInput
        self.assertEqual(len(cases(20261018)), 6)
        with self.assertRaises(InvalidInput):
            cases(20261019)


if __name__ == '__main__':
    unittest.main()
