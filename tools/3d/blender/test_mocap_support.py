"""Check ground support and the per-foot plant/step windows without Blender."""

import numpy as np
import pytest

from mocap_balance import COM_RESPONSE, INBOARD_RESERVE_M, stationary_offset
from mocap_spec import Plausibility
from mocap_support import evaluate_support


BONES = ("hips", "spine", "chest", "neck", "head") + tuple(
    f"{part}_{side}" for side in ("L", "R")
    for part in ("upper_arm", "lower_arm", "hand", "upper_leg", "lower_leg", "foot")
)
PHASES = np.array((0., .25, .5, .75, 1.))


def feet():
    rectangle = np.array(((-.02, -.05, 0.), (.02, -.05, 0.), (.02, .05, 0.), (-.02, .05, 0.)))
    return {side: np.repeat((rectangle + (x, 0., 0.))[None], len(PHASES), axis=0)
            for side, x in (("R", -.1), ("L", .1))}


def tracks():
    heads = np.zeros((len(PHASES), len(BONES), 3))
    heads[:, :, 2] = .7
    return heads, heads + (0., 0., .1)


def evaluate(soles, settings=Plausibility(), *, foot_lock=False, loop=False, pose=None, stride=None):
    return evaluate_support(
        PHASES, BONES, *(tracks() if pose is None else pose), soles, settings,
        foot_lock=foot_lock, loop=loop, stride=stride,
    )


def test_static_and_dynamic_frames_use_their_own_balance_margins():
    pose = tracks()
    for joints in pose:
        joints[1, :, 0] = .145
        joints[2, :, 0] = .121
    result = evaluate(feet(), Plausibility(dynamic=((.25, .25),)), pose=pose)["balance"]
    assert not result.passed
    assert result.frame == 2
    assert result.value == pytest.approx(.001)
    assert result.limit == 0.

    for joints in pose:
        joints[2, :, 0] = .12
    result = evaluate(feet(), Plausibility(dynamic=((.25, .25),)), pose=pose)["balance"]
    assert result.passed
    assert result.frame == 1
    assert result.value == pytest.approx(.025)
    assert result.limit == .03

    for joints in pose:
        joints[1, :, 0] = .151
    result = evaluate(feet(), Plausibility(dynamic=((.25, .25),)), pose=pose)["balance"]
    assert not result.passed
    assert result.value == pytest.approx(.031)


def test_balance_disregards_hovering_foot_and_requires_support_or_airborne():
    soles = feet()
    soles["R"][1, :, 2] = .03
    soles["R"][2, :, 2] = soles["L"][2, :, 2] = .09
    unsupported = evaluate(soles, Plausibility(dynamic=((0., 1.),)))["balance"]
    assert not unsupported.passed
    assert unsupported.frame == 2
    assert unsupported.value is None
    assert "support" in unsupported.detail

    one_foot = evaluate(soles, Plausibility(airborne=((.5, .5),)))["balance"]
    assert not one_foot.passed
    assert one_foot.frame == 1
    assert one_foot.value == pytest.approx(.08)


def test_balance_projects_weighted_segment_centres():
    pose = tracks()
    hip = BONES.index("hips")
    pose[0][1, hip, 0] = 1.2
    pose[1][1, hip, 0] = 1.4
    result = evaluate(feet(), pose=pose)["balance"]
    # Only hips shift, at 11.17% mass and 38.85% of the bone's caudal-to-cranial length.
    assert result.value == pytest.approx(.1117 * (1.2 + .2 * .3885) - .12)


def test_balance_checks_diagonal_hull_boundary_and_degenerate_contacts():
    soles = {
        "R": np.repeat(np.array([[[-.1, -.1, 0.], [.1, .1, 0.]]]), len(PHASES), axis=0),
        "L": np.repeat(np.array([[[-.1, .1, 0.]]]), len(PHASES), axis=0),
    }
    pose = tracks()
    for joints in pose:
        joints[1:, :, :2] = (.02, 0.)
    result = evaluate(soles, pose=pose)["balance"]
    assert not result.passed
    assert result.value == pytest.approx(.02 / np.sqrt(2.))

    soles["L"][:, :, 2] = .02
    line = evaluate(soles, pose=pose)["balance"]
    assert line.value == pytest.approx(result.value)
    soles["R"][:, 1, 2] = .02
    point = evaluate(soles, pose=pose)["balance"]
    assert point.value == pytest.approx(np.hypot(.12, .1))


@pytest.mark.parametrize(("support", "bone"), (("hand_R", "hand_R"), ("knee_R", "lower_leg_R")))
def test_declared_support_must_physically_touch_ground(support, bone):
    soles = feet()
    soles["R"][1, :, 2] = .03
    pose = tracks()
    index = BONES.index(bone)
    pose[0][1, index] = (-.10, 0., .01)
    pose[1][1, index] = (-.10, 0., .15)
    assert not evaluate(soles, pose=pose)["balance"].passed
    settings = Plausibility(support={support: ((.25, .25),)})
    assert evaluate(soles, settings, pose=pose)["balance"].passed
    for joints in pose:
        joints[1, index, 2] += .2
    assert not evaluate(soles, settings, pose=pose)["balance"].passed


def test_declared_steps_apply_only_to_the_specified_foot_in_the_window():
    soles = feet()
    soles["R"][2, :, 2] = .08
    soles["L"][3, :, 2] = .07
    settings = Plausibility(steps={"R": ((.5, .5),)})
    check = evaluate(soles, settings)["foot_lift"]
    assert not check.passed
    assert check.frame == 3
    assert check.value == pytest.approx(.07)
    assert check.limit == .05
    assert "L" in check.detail
    soles["L"][3, :, 2] = .05
    assert evaluate(soles, settings)["foot_lift"].passed


def test_plant_slide_checks_displacement_between_any_two_frames():
    soles = feet()
    soles["R"][1, 0, 0] += .006
    soles["R"][2, 0, 0] -= .006
    assert evaluate(soles)["foot_slide"].passed
    result = evaluate(soles, foot_lock=True)["foot_slide"]
    assert not result.passed
    assert result.value == pytest.approx(.012)
    assert result.limit == .01
    assert result.frame == 2


def test_plant_slide_ends_at_a_release_but_follows_the_loop_seam():
    soles = feet()
    soles["L"][3:, :, 0] += .02
    settings = Plausibility(plants={"L": ((0., .25), (.75, 1.))})
    assert evaluate(soles, settings)["foot_slide"].passed
    result = evaluate(soles, settings, loop=True)["foot_slide"]
    assert not result.passed
    assert result.value == pytest.approx(.02)
    assert result.frame == 0


def test_in_place_slide_restores_stride_across_output_phases_and_the_loop():
    soles = feet()
    stride = np.array((.3, -.4, 0.))
    soles["L"][:2] -= PHASES[:2, None, None] * stride
    soles["L"][3:] -= (PHASES[3:, None, None] - 1.) * stride
    settings = Plausibility(plants={"L": ((0., .25), (.75, 1.))})
    in_place = evaluate(soles, settings, loop=True)
    assert not in_place["foot_slide"].passed
    world = evaluate(soles, settings, loop=True, stride=stride)
    assert world["foot_slide"].passed
    assert world["foot_slide"].value == pytest.approx(0.)
    assert world["balance"] == in_place["balance"]
    assert world["foot_lift"] == in_place["foot_lift"]


@pytest.mark.parametrize("scale", (1., .7))
def test_stationary_correction_points_back_into_real_contacts(scale):
    soles = {side: frames[0] for side, frames in feet().items()}
    heads, tails = (samples[0] for samples in tracks())
    heads[:, 0] = tails[:, 0] = .18
    shift = stationary_offset(BONES, heads, tails, soles, Plausibility(), phase=.3, scale=scale)
    inset = INBOARD_RESERVE_M * .12 / .13
    assert shift[0] == pytest.approx(-(.06 + inset / scale) / COM_RESPONSE)
    assert shift[1] == pytest.approx(0.)
    heads[:, 0] = tails[:, 0] = .119
    shift = stationary_offset(BONES, heads, tails, soles, Plausibility(), phase=.3, scale=scale)
    assert shift[0] == pytest.approx(-(-.001 + inset / scale) / COM_RESPONSE)
    heads[:, 0] = tails[:, 0] = .0
    assert stationary_offset(BONES, heads, tails, soles, Plausibility(), phase=.3, scale=scale).tolist() == [0., 0.]
