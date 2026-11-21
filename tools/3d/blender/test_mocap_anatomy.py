"""Regression checks for the anatomical bake gate."""

from math import cos, radians, sin

import numpy as np
import pytest

from mocap_anatomy import _capsule_penetration, _face_clearance, _joint_limits, _trunk_lean, forearm_penetration
from mocap_kinematics import Track


def test_trunk_lean_reports_the_worst_frame_outside_work() -> None:
    phases = np.asarray([0., .5, 1.])
    angles = np.deg2rad([5., 40., 22.])
    hips = np.zeros((3, 3))
    neck = np.column_stack((np.sin(angles), np.zeros(3), np.cos(angles)))

    result = _trunk_lean(hips, neck, phases, ((.4, .6),), 15.)

    assert result.frame == 2
    assert result.value == pytest.approx(22.)
    assert not result.passed


@pytest.mark.parametrize(("body", "flexion", "cap"), [("p1", 153, 150), ("p4", 148, 145)])
def test_joints_report_each_elbow_flexion_violation(body: str, flexion: int, cap: int) -> None:
    names = ("hips", "upper_arm_R", "lower_arm_R")
    bind = np.repeat(np.eye(4)[None], len(names), axis=0)
    poses = np.repeat(bind[None], 3, axis=0)
    angle = radians(flexion)
    poses[1:, 2, :3, :3] = ((cos(angle), -sin(angle), 0.),
                           (sin(angle), cos(angle), 0.),
                           (0., 0., 1.))
    track = Track(names, (-1, 0, 1), bind, poses, np.ones(len(names)))

    result = _joint_limits(track, body)

    assert result.frame == 1
    assert result.value == pytest.approx(flexion)
    assert result.limit == cap
    assert "lower_arm_R" in result.detail and "1, 2" in result.detail
    assert not result.passed


def test_capsule_penetration_is_measured_at_segment_interiors() -> None:
    first = np.asarray([[[-1., 0., 0.], [1., 0., 0.]],
                        [[-1., 0., 0.], [1., 0., 0.]]])
    second = np.asarray([[[0., -1., .18], [0., 1., .18]],
                         [[0., -1., .26], [0., 1., .26]]])

    depth = _capsule_penetration(first, second, .1, .1)

    assert depth == pytest.approx([.02, 0.])


def test_capsule_penetration_handles_parallel_and_degenerate_axes() -> None:
    first = np.asarray([[[0., 0., 0.], [1., 0., 0.]],
                        [[0., 0., 0.], [0., 0., 0.]]])
    second = np.asarray([[[.5, .19, 0.], [1.5, .19, 0.]],
                         [[.08, 0., 0.], [.08, 0., 0.]]])

    depth = _capsule_penetration(first, second, .1, .1)

    assert depth == pytest.approx([.01, .12])


def test_same_wrist_position_can_use_a_clearer_elbow_pole() -> None:
    torso = {"chest": (np.asarray([[[0., 0., 0.], [0., 0., .2]]]), .1)}
    elbows = np.asarray([[-.12, 0., .1], [.2, .24, .1]])

    depths = forearm_penetration(elbows, np.asarray([.2, 0., .1]), torso)

    assert depths == pytest.approx([.132, 0.])


def test_face_clearance_exempts_only_declared_frames() -> None:
    rest = np.asarray([[0., -.1, 1.6], [0., .1, 1.6], [-.3, 0., 1.], [.3, 0., 1.]])
    posed = np.repeat(rest[None], 3, axis=0)
    posed[0, 2] = rest[0] + [.06, 0., 0.]
    posed[1, 2] = rest[0] + [.01, 0., 0.]
    posed[2, 2] = rest[0] + [.03, 0., 0.]

    result = _face_clearance(rest, posed, (2, 3), np.asarray([0., .5, 1.]), ((.4, .6),), 1.)

    assert result.frame == 2
    assert result.value == pytest.approx(.03)
    assert not result.passed
