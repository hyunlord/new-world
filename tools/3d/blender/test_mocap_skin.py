"""Verify the offline joint correction on a small synthetic mesh."""

import unittest

import numpy as np

from mocap_deform import check_deformation
from mocap_skin import relax_body_skin, relax_joint_skin, stabilize_garment_weights


class MocapSkinTest(unittest.TestCase):
    def setUp(self) -> None:
        triangle = np.array([[0., 0., 1.], [.03, 0., 1.], [0., .04, 1.]])
        self.rest = np.vstack([triangle + [.08 * i, 0., 0.] for i in range(4)]
                              + [triangle + [0., 0., -.95]])
        self.faces = np.arange(15).reshape(-1, 3)
        self.weights = np.repeat([
            [.7, 0., 0., 0.], [.55, .4, 0., 0.],
            [0., 0., .7, 0.], [0., 0., .55, .4], [0., 0., 0., 0.],
        ], 3, axis=0)
        self.grips = (0, 3)

    def test_stretch_and_collapse_repair_preserves_pins_and_loop(self) -> None:
        frames = np.stack([self.rest] * 3)
        frames[:, 12:, 1] -= .005
        frames[1, 1, 0] += .042
        frames[1, 5, 1] = .007
        frames[-1, 0, 0] = -0.
        original = frames.copy()
        before = check_deformation(self.rest, frames, self.faces, self.weights, local_edges=True)
        self.assertGreater(before.max_edge_ratio, 1.35)
        self.assertLess(before.min_area_ratio, .35)

        corrected = relax_joint_skin(self.rest, frames, self.faces, self.weights, self.grips, loop=True)

        self.assertTrue(np.array_equal(frames, original))
        fixed = [0, 3, 12, 13, 14]
        self.assertTrue(np.array_equal(corrected[:, fixed].view(np.uint64), frames[:, fixed].view(np.uint64)))
        self.assertTrue(np.array_equal(corrected[0], corrected[-1]))
        for result in (corrected, corrected.astype("<f2")):
            self.assertTrue(check_deformation(self.rest, result, self.faces, self.weights, local_edges=True).passed)

    def test_single_nonloop_rigid_frame_needs_no_material_change(self) -> None:
        rotated = self.rest[:, [1, 2, 0]] + [.05, -.10, .70]
        corrected = relax_joint_skin(self.rest, rotated[None], self.faces, self.weights, self.grips, loop=False)
        np.testing.assert_allclose(corrected[0], rotated, rtol=0., atol=1.e-11)

    def test_hem_weights_match_across_center_and_preserve_feet(self) -> None:
        points = np.array([[0., 0., 0.], [0., 0., 1.6], [-.006, 0., .50], [.006, 0., .50]])
        rows = [{'foot_L': 1.}, {'head': 1.}, {'upper_leg_R': 1.}, {'upper_leg_L': 1.}]
        fixed = stabilize_garment_weights(points, rows)
        self.assertEqual(fixed[:2], rows[:2])
        self.assertEqual(fixed[2]['hips'], 1.)
        self.assertEqual(fixed[3]['hips'], 1.)
        self.assertTrue(all(abs(sum(row.values()) - 1.) < 1.e-12 for row in fixed))

    def test_body_repair_preserves_palm_joint_pins_and_exact_loop(self) -> None:
        rest = self.rest.copy()
        rest[12:, 2] += .4
        frames = np.stack([rest] * 3)
        frames[:, 13, 0] += .10
        corrected = relax_body_skin(rest, frames, self.faces, (*self.grips, 12),
                                    loop=True, arm_weights=self.weights)
        np.testing.assert_array_equal(corrected[:, :13], frames[:, :13])
        np.testing.assert_array_equal(corrected[0], corrected[-1])
        for result in (corrected, corrected.astype('<f2')):
            self.assertTrue(check_deformation(rest, result, self.faces, self.weights, local_edges=True).passed)

    def test_body_repair_moves_boot_collar_above_planted_sole(self) -> None:
        rest = np.array([[0., 0., .025], [.02, 0., .025], [0., .01, .085]])
        posed = rest[None].copy()
        posed[0, 2, 2] += .25
        corrected = relax_body_skin(rest, posed, np.array([[0, 1, 2]]), [], loop=False)

        np.testing.assert_array_equal(corrected[:, :2], posed[:, :2])
        self.assertTrue(check_deformation(rest, corrected.astype('<f2'), np.array([[0, 1, 2]]),
                                         np.zeros((3, 4)), check_joints=False).passed)


if __name__ == "__main__":
    unittest.main()
