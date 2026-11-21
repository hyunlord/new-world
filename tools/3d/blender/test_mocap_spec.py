"""Check authored action boundaries and deterministic CMU take selection."""

import hashlib
import json
from pathlib import Path

import pytest

import mocap_spec as M


def action(tmp_path: Path, fields: dict[str, M.Value]) -> M.Action:
    spec = tmp_path / "clip.ron"
    spec.write_text(json.dumps({"pick_up": {
        "tags": ["pick up", "bend"],
        "trim": [0.1, 1.1],
        "loop": False,
        "foot_lock": True,
        "contacts": [0.5],
        "target": {"kind": "reach", "anchor": [0, -0.4, 0.15]},
    } | fields}))
    return M.load(spec)[0]


def test_preferred_take_preserves_the_official_capture_clock(tmp_path: Path) -> None:
    spec = action(tmp_path, {"takes": ["79_13"]})
    index = tmp_path / "index.json"
    index.write_text(json.dumps({
        "79_13": {"description": "mixing batter", "tags": ["mix"], "source_fps": 60},
    }))
    raw = tmp_path / "raw"
    raw.mkdir()
    (raw / "79_13.bvh").touch()

    take = M.select(spec, index)

    assert (take.id, take.fps, take.path) == ("79_13", 60.0, raw / "79_13.bvh")


def test_tag_search_ranks_downloaded_files_and_breaks_ties_by_id(tmp_path: Path) -> None:
    spec = action(tmp_path, {})
    index = tmp_path / "index.json"
    index.write_text(json.dumps({
        "115_03": {"description": "pick up box, bend from waist", "tags": []},
        "115_02": {"description": "Bend and Pick-Up", "tags": []},
        "115_01": {"description": "pick up box, bend from waist", "tags": []},
        "01_01": {"description": "pick up box", "tags": []},
    }))
    raw = tmp_path / "raw"
    raw.mkdir()
    for take in ("115_03", "115_02", "01_01"):
        (raw / f"{take}.bvh").touch()

    assert M.select(spec, index).id == "115_02"


def test_missing_pinned_take_cannot_silently_reuse_another_takes_trim(tmp_path: Path) -> None:
    spec = action(tmp_path, {"takes": ["115_01"]})
    index = tmp_path / "index.json"
    index.write_text(json.dumps({
        take: {"description": "pick up box, bend from waist", "tags": []}
        for take in ("115_01", "115_02")
    }))
    raw = tmp_path / "raw"
    raw.mkdir()
    (raw / "115_02.bvh").touch()

    with pytest.raises(M.SpecError, match="115_01"):
        M.select(spec, index)


def test_rejects_a_changed_indexed_download(tmp_path: Path) -> None:
    spec = action(tmp_path, {"takes": ["115_01"]})
    index = tmp_path / "index.json"
    index.write_text(json.dumps({
        "115_01": {"description": "pick up", "sha256": hashlib.sha256(b"original").hexdigest()},
    }))
    raw = tmp_path / "raw"
    raw.mkdir()
    (raw / "115_01.bvh").write_bytes(b"changed")

    with pytest.raises(M.SpecError, match="SHA-256"):
        M.select(spec, index)


def test_loads_and_selects_a_static_base_layer(tmp_path: Path) -> None:
    spec = action(tmp_path, {"base_layer": {
        "takes": ["23_03"], "trim": [2.7, 2.7], "heading": -15, "weight": 0.8,
        "mask": {"root": 1, "spine": 0.4, "upper_leg_R": 1, "foot_L": 1},
    }})
    index = tmp_path / "index.json"
    index.write_text(json.dumps({"23_03": {"description": "kneeling participant"}}))
    raw = tmp_path / "raw"
    raw.mkdir()
    (raw / "23_03.bvh").touch()

    layer = spec.base_layer
    assert layer is not None
    take = M.select(layer, index)

    assert (take.id, layer.trim, layer.heading, layer.weight) == ("23_03", (2.7, 2.7), -15, 0.8)
    assert layer.mask == {"root": 1, "spine": 0.4, "upper_leg_R": 1, "foot_L": 1}


def test_loads_per_body_arm_limits(tmp_path: Path) -> None:
    spec = action(tmp_path, {"arm_limits": {
        "default": {"upper_roll": 20, "lower_roll": 70, "elevation": 80},
        "p4": {"upper_roll": 15, "lower_roll": 60, "elevation": 65, "clavicle_lift": 0.025},
    }})

    assert spec.arm_limits == {"default": M.ArmLimits(20, 70, 80, 0),
                               "p4": M.ArmLimits(15, 60, 65, 0.025)}


def test_existing_specs_need_no_layer_or_arm_limits(tmp_path: Path) -> None:
    spec = action(tmp_path, {})

    assert spec.base_layer is None
    assert spec.arm_limits == {}


def test_loads_output_timing_independently_of_source_trim(tmp_path: Path) -> None:
    spec = action(tmp_path, {
        "duration": 1.35,
        "retime": [[0, 0], [0.68, 0.84], [1, 1]],
        "impact_window": [0.13, 0.07],
    })

    assert spec.trim == (0.1, 1.1)
    assert spec.duration == 1.35
    assert spec.retime == ((0, 0), (0.68, 0.84), (1, 1))
    assert spec.impact_window == (0.13, 0.07)


def test_omitted_output_timing_preserves_source_clock(tmp_path: Path) -> None:
    spec = action(tmp_path, {})

    assert (spec.duration, spec.retime, spec.impact_window) == (None, (), (0.14, 0.14))


def test_placement_can_follow_the_captured_direction(tmp_path: Path) -> None:
    spec = action(tmp_path, {"prop": "basket", "hands": {"R": {"target": "prop"}},
                            "target": {"kind": "reach", "anchor": [0, -0.4, 0.15], "release": True}})

    assert spec.target.release and not spec.reverse


def test_load_carries_with_two_grips_and_in_place_gait(tmp_path: Path) -> None:
    spec = action(tmp_path, {"prop": "logs", "in_place": True, "foot_lock": False,
                            "hands": {"R": {"target": "prop"}, "L": {"target": "prop", "offset": [.48, 0, 0]}},
                            "target": {"kind": "carry"}})

    assert spec.in_place and spec.target.kind == "carry"


def test_windup_uses_a_spec_peak_and_world_offset(tmp_path: Path) -> None:
    spec = action(tmp_path, {"prop": "axe", "hands": {"R": {"target": "prop"}},
                            "target": {"kind": "swing", "anchor": [0, -0.4, 0.55],
                                       "windup": [.35, [-.125, -.1, .015]]}})

    assert spec.target.windup == (.35, (-.125, -.1, .015))


def test_combining_specs_rejects_duplicate_visual_keys(tmp_path: Path) -> None:
    action(tmp_path, {})
    path = tmp_path / "clip.ron"

    with pytest.raises(M.SpecError, match="visual key must be unique"):
        M.load_many((path, path))


def test_set_down_reverses_the_pick_up_source_with_planted_feet() -> None:
    spec = Path(__file__).parent / "mocap"
    pick, = M.load(spec / "pick_up.ron")
    place, = M.load(spec / "set_down.ron")
    assert place.takes == ("115_01",)
    assert place.reverse and not pick.reverse
    assert place.foot_lock and place.target.release
    assert (place.tags, place.trim, place.heading, place.posture, place.hands) == (
        pick.tags, pick.trim, pick.heading, pick.posture, pick.hands)
    assert place.contacts[0] == pytest.approx(1. - pick.contacts[0])


def test_loads_phase_limits_for_torso_lean(tmp_path: Path) -> None:
    curve: M.Value = [[0, 9], [0.5, 23], [1, 9]]
    assert action(tmp_path, {"torso_lean": curve}).torso_lean == ((0., 9.), (.5, 23.), (1., 9.))


@pytest.mark.parametrize(("field", "value"), [
    ("duration", 0),
    ("duration", float("inf")),
    ("retime", []),
    ("retime", [[0, 0], [0.9, 1]]),
    ("retime", [[0, 0], [0.6, 0.7], [0.5, 0.8], [1, 1]]),
    ("retime", [[0, 0], [0.6, 0.7], [0.7, 0.7], [1, 1]]),
    ("impact_window", [0.2, 0]),
    ("impact_window", [0.55, 0.2]),
    ("torso_lean", [[0, 9], [0.6, 20], [0.5, 10], [1, 9]]),
    ("torso_lean", [[0, 9], [1, 65]]),
    ("torso_lean", [[0, 9], [0.9, 9]]),
])
def test_rejects_invalid_output_timing(tmp_path: Path, field: str, value: M.Value) -> None:
    with pytest.raises(M.SpecError, match=field):
        action(tmp_path, {field: value})


@pytest.mark.parametrize(("overrides", "message"), [
    ({"trim": [2.8, 2.7]}, "base layer trim"),
    ({"weight": 1.1}, "outside"),
    ({"mask": {"RightUpLeg": 1}}, "RightUpLeg"),
    ({"mask": {"hips": -0.5}}, "outside"),
    ({"mask": {}}, "base layer mask"),
    ({"takes": []}, "pin or search tags"),
])
def test_rejects_invalid_base_layers(tmp_path: Path, overrides: dict[str, M.Value], message: str) -> None:
    layer: dict[str, M.Value] = {"takes": ["23_03"], "trim": [2.7, 3.9], "mask": {"hips": 1}}

    with pytest.raises(M.SpecError, match=message):
        action(tmp_path, {"base_layer": layer | overrides})


@pytest.mark.parametrize(("body", "limits", "message"), [
    ("p5", {"upper_roll": 20, "lower_roll": 55, "elevation": 90}, "p5"),
    ("p4", {"upper_roll": 20, "elevation": 90}, "lower_roll"),
    ("p3", {"upper_roll": 20, "lower_roll": -1, "elevation": 90}, "outside"),
    ("p2", {"upper_roll": 20, "lower_roll": 60, "elevation": 91}, "outside"),
    ("p1", {"upper_roll": 20, "lower_roll": 60, "elevation": 90, "clavicle_lift": -0.1}, "outside"),
])
def test_rejects_invalid_arm_limits(tmp_path: Path, body: str, limits: dict[str, M.Value], message: str) -> None:
    with pytest.raises(M.SpecError, match=message):
        action(tmp_path, {"arm_limits": {body: limits}})


@pytest.mark.parametrize("fields", [
    {"trim": [1.1, 0.1]},
    {"trim": [True, 1.1]},
    {"takes": ["../unsafe"]},
    {"speed": {"p5": 1}},
    {"hands": {"R": {"target": "anchor", "slide": [[0.5, 0.1], [0.2, 0.2]]}}},
    {"contacts": [1.2]},
])
def test_rejects_invalid_authoring_data(tmp_path: Path, fields: dict) -> None:
    with pytest.raises(M.SpecError):
        action(tmp_path, fields)
