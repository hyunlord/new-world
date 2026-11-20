"""Check authored action boundaries and deterministic CMU take selection."""

import hashlib
import json
from pathlib import Path

import pytest

import mocap_spec as M


def action(tmp_path: Path, fields: dict) -> M.Action:
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
