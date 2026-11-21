"""Verify the retarget hand-off reconstructs the same armature pose."""

from pathlib import Path

import numpy as np
import pytest

from mocap_kinematics import Track, check_prop, rigid_rotations, write_track


def test_export_keeps_parent_local_rotation_and_root_translation(tmp_path: Path) -> None:
    root = np.eye(4)
    root[:3, 3] = (1., 2., 3.)
    local = np.array(((0., -1., 0., .3), (1., 0., 0., .2), (0., 0., 1., .1), (0., 0., 0., 1.)))
    rest = np.stack((np.eye(4), np.eye(4)))
    rest[1, 1, 3] = .5
    track = Track(("hips", "spine"), (-1, 0), rest, np.stack((root, root @ local))[None],
                  np.array((.5, .25)))
    output = tmp_path / "walk.npz"
    write_track(output, track, np.array((0.,)), 30, .7)
    with np.load(output, allow_pickle=False) as exported:
        child = np.eye(4)
        child[:3, :3] = exported["local_rotations"][0, 1]
        child[:3, 3] = exported["local_offsets"][0, 1]
        np.testing.assert_allclose(exported["root_transforms"][0] @ child, root @ local)
        np.testing.assert_allclose(exported["rest_offsets"][1], (0., .5, 0.))
        assert tuple(exported["bone_names"]) == ("hips", "spine")
        assert tuple(exported["parent_indices"]) == (-1, 0)
        assert exported["fps"].item() == 30
        assert exported["body_scale"].item() == .7


def test_prop_scale_checks_both_bounds_and_missing_data() -> None:
    assert check_prop(.6, 1.6, (.3, .4)).passed
    assert not check_prop(.8, 1.6, (.3, .4)).passed
    assert not check_prop(.4, 1.6, (.3, .4)).passed
    assert not check_prop(.6, 1.6, None).passed
    assert check_prop(None, 1.6, None).value is None


def test_export_corrects_small_matrix_drift_and_rejects_scale() -> None:
    noisy = np.diag((1.00013, 1.0, 1.0))
    noisy[1, 2] = .0001
    rotation = rigid_rotations(noisy)
    np.testing.assert_allclose(rotation.T @ rotation, np.eye(3), atol=1e-12)
    assert np.linalg.det(rotation) == pytest.approx(1.)
    with pytest.raises(ValueError, match="scaled or mirrored"):
        rigid_rotations(np.diag((1.1, 1.0, 1.0)))
