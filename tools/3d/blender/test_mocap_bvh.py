"""Check BVH channel order and forward kinematics without loading Blender."""

from pathlib import Path
from tempfile import TemporaryDirectory
import unittest

import numpy as np

from mocap_bvh import read_bvh


class BvhTest(unittest.TestCase):
    def test_root_rotation_moves_the_child_in_declared_axis_order(self) -> None:
        source = """HIERARCHY
ROOT Hips
{
 OFFSET 0 0 0
 CHANNELS 6 Xposition Yposition Zposition Zrotation Yrotation Xrotation
 JOINT Arm
 {
  OFFSET 1 0 0
  CHANNELS 3 Zrotation Yrotation Xrotation
  End Site
  {
   OFFSET 1 0 0
  }
 }
}
MOTION
Frames: 2
Frame Time: 0.01
0 0 0 0 0 0 0 0 0
0 2 3 0 90 0 0 0 0
"""
        with TemporaryDirectory() as directory:
            path = Path(directory) / "sample.bvh"
            path.write_text(source)
            motion = read_bvh(path)
        arm = motion.joints["Arm"]
        np.testing.assert_allclose(motion.positions[:, arm], ((1, 0, 0), (0, -2, 2)), atol=1e-7)
        self.assertEqual(motion.frame_time, .01)


if __name__ == "__main__":
    unittest.main()
