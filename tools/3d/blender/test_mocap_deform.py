"""Check the mocap deformation gate without loading Blender."""

import unittest

import numpy as np

import mocap_deform


class MocapDeformTest(unittest.TestCase):
    def setUp(self) -> None:
        triangle = np.array([[0., 0., 0.], [.03, 0., 0.], [0., .04, 0.]])
        self.rest = np.concatenate([triangle + [.08 * i, 0., 0.] for i in range(5)])
        self.triangles = np.arange(15).reshape(-1, 3)
        self.weights = np.repeat([
            [.7, 0., 0., 0.], [.55, .4, 0., 0.],
            [0., 0., .7, 0.], [0., 0., .55, .4], [0., 0., 0., 0.],
        ], 3, axis=0)

    def test_rigid_pose_preserves_lengths_and_areas(self) -> None:
        posed = self.rest[:, [1, 2, 0]] + [1., 2., 3.]
        report = mocap_deform.check_deformation(self.rest, posed[None], self.triangles, self.weights)
        self.assertTrue(report.passed)
        self.assertAlmostEqual(report.max_edge_ratio, 1.)
        self.assertAlmostEqual(report.min_area_ratio, 1.)
        self.assertEqual(report.edge_sample_count, 15)
        self.assertEqual(report.triangle_sample_count, 4)

    def test_stretch_on_trunk_fails_at_the_identified_frame(self) -> None:
        frames = np.stack([self.rest] * 3)
        frames[1, 13, 0] += .021
        report = mocap_deform.check_deformation(self.rest, frames, self.triangles, self.weights)
        self.assertFalse(report.passed)
        self.assertAlmostEqual(report.max_edge_ratio, 1.7)
        self.assertEqual(report.max_edge_frame, 1)
        self.assertEqual(report.frames[1].max_edge_vertices, (12, 13))
        self.assertGreater(report.min_area_ratio, report.min_area)

    def test_joint_edge_scope_reports_trunk_stretch_without_failing(self) -> None:
        frames = np.stack([self.rest] * 3)
        frames[1, 13, 0] += .021
        report = mocap_deform.check_deformation(
            self.rest, frames, self.triangles, self.weights, local_edges=True,
        )
        self.assertTrue(report.passed)
        self.assertEqual(report.edge_scope, "shoulder_elbow")
        self.assertEqual(report.edge_sample_count, 12)
        self.assertAlmostEqual(report.max_edge_ratio, 1.)
        self.assertAlmostEqual(report.max_body_edge_ratio, 1.7)
        self.assertEqual(report.max_body_edge_frame, 1)
        self.assertEqual(report.frames[1].max_body_edge_vertices, (12, 13))

    def test_joint_stretch_fails_with_good_area(self) -> None:
        frames = np.stack([self.rest] * 3)
        frames[1, 1, 0] += .021
        report = mocap_deform.check_deformation(
            self.rest, frames, self.triangles, self.weights, local_edges=True,
        )
        self.assertFalse(report.passed)
        self.assertAlmostEqual(report.max_edge_ratio, 1.7)
        self.assertEqual(report.max_edge_frame, 1)
        self.assertEqual(report.frames[1].max_edge_vertices, (0, 1))
        self.assertGreater(report.min_area_ratio, report.min_area)

    def test_elbow_collapse_fails_without_an_edge_stretch(self) -> None:
        frames = np.stack([self.rest] * 3)
        frames[2, 5, 1] *= .2
        report = mocap_deform.check_deformation(self.rest, frames, self.triangles, self.weights)
        self.assertFalse(report.passed)
        self.assertAlmostEqual(report.min_area_ratio, .2)
        self.assertEqual(report.min_area_frame, 2)
        self.assertEqual(report.frames[2].min_area_triangle, 1)
        self.assertLess(report.max_edge_ratio, report.max_stretch)

    def test_tiny_rest_edge_and_sliver_do_not_dominate(self) -> None:
        detail = np.array([[0., 0., 0.], [.0001, 0., 0.], [.01, .00001, 0.]])
        rest = np.concatenate([self.rest, detail])
        poses = rest.copy()
        poses[16] = [.0003, 0., 0.]
        poses[17, 1] = 0.
        triangles = np.vstack([self.triangles, [15, 16, 17]])
        weights = np.vstack([self.weights, np.tile(self.weights[0], (3, 1))])
        report = mocap_deform.check_deformation(rest, poses[None], triangles, weights)
        self.assertTrue(report.passed)
        self.assertEqual(report.edge_sample_count, 17)
        self.assertEqual(report.triangle_sample_count, 4)

    def test_missing_shoulder_coverage_fails(self) -> None:
        self.weights[:3] = 0.
        with self.assertRaisesRegex(ValueError, "shoulder_R"):
            mocap_deform.check_deformation(self.rest, self.rest[None], self.triangles, self.weights)


if __name__ == "__main__":
    unittest.main()
