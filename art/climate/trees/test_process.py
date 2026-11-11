# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic", "scipy", "scikit-image", "pytest"]
# ///
# How to run: python -m pytest art/climate/trees/test_process.py -q
"""Candidate export preserves provenance and shared tree registration."""

from __future__ import annotations

import json
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path

    from numpy.typing import NDArray

import numpy as np
import pytest
from PIL import Image, ImageDraw
from pydantic import JsonValue, TypeAdapter

from art.climate.trees import process


def inputs(tmp_path: Path) -> Path:
    """Make paired opaque tree shapes on matching transparent source canvases."""
    records: list[JsonValue] = []
    for state, top in (("full", 20), ("winter", 65)):
        image = Image.new("RGBA", (200, 220))
        draw = ImageDraw.Draw(image)
        draw.rectangle((60, top, 140, 169), fill=(80, 122, 56, 255))
        draw.rectangle((90, 160, 110, 199), fill=(90, 60, 35, 255))
        source = tmp_path / f"{state}.png"
        image.save(source)
        asset_id = f"oak_{state}_v1"
        generation = tmp_path / f"{state}.json"
        _ = generation.write_text(
            json.dumps(
                {
                    "id": asset_id,
                    "attempt": 1,
                    "source": str(source),
                    "preserved_source": str(source),
                    "prompt": f"One {state} oak",
                    "reference_images": [str(source)],
                }
            )
        )
        records.append(
            {
                "id": asset_id,
                "species": "oak",
                "state": state,
                "variant": "v1",
                "source": str(source),
                "generation_record": str(generation),
                "transform_reference": "oak_full_v1" if state == "winter" else None,
                "season_tints": {"spring": {"delta_lab": [6, 0, 6]}} if state == "full" else {},
            }
        )
    path = tmp_path / "input.json"
    _ = path.write_text(json.dumps(records))
    return path


def test_pair_keeps_shared_transform_and_shorter_bare_height(tmp_path: Path) -> None:
    # Given a winter source with the same base but a smaller crown.
    path = inputs(tmp_path)
    # When both sizes are exported from the real file boundary.
    rows = process.run(path, tmp_path / "out")
    # Then winter retains summer scale rather than inflating its crown.
    full = next(row for row in rows if row.id == "oak_full_v1_small")
    winter = next(row for row in rows if row.id == "oak_winter_v1_small")
    assert full.transform == winter.transform
    assert winter.checks.opaque_height < full.checks.opaque_height
    assert winter.pair is not None
    assert winter.pair.base_delta_px == 0
    assert winter.status == "CANDIDATE_REVIEW_REQUIRED"


def test_season_export_preserves_alpha_and_unmasked_rgb(tmp_path: Path) -> None:
    # Given explicit spring data and a generated base frame.
    path = inputs(tmp_path)
    # When the authorized seasonal renderer exports a derivative.
    rows = process.run(path, tmp_path / "out")
    base = next(row for row in rows if row.id == "oak_full_v1_small")
    spring = next(row for row in rows if row.id == "oak_full_v1_small_spring")
    # Then every alpha byte and nonfoliage RGB byte remains unchanged.
    with (
        Image.open(base.path) as original,
        Image.open(spring.path) as output,
        Image.open(base.mask_path) as mask,
    ):
        a, b = np.asarray(original), np.asarray(output)
        mask_pixels: NDArray[np.uint8] = np.asarray(mask, dtype=np.uint8)
        protected: NDArray[np.bool_] = np.equal(mask_pixels, 0)
        np.testing.assert_array_equal(a[..., 3], b[..., 3])
        np.testing.assert_array_equal(a[protected], b[protected])
    assert spring.derived
    assert spring.tint is not None


def test_changed_preserved_source_is_rejected_before_outputs(tmp_path: Path) -> None:
    # Given a generation record whose source was replaced by unrelated pixels.
    path = inputs(tmp_path)
    generation = tmp_path / "full.json"
    payload = TypeAdapter(dict[str, JsonValue]).validate_json(generation.read_bytes())
    payload["preserved_source"] = str(tmp_path / "winter.png")
    _ = generation.write_text(json.dumps(payload))
    # When the provenance boundary is parsed.
    with pytest.raises(ValueError, match="source hash"):
        _ = process.run(path, tmp_path / "out")
    # Then no candidate was delivered.
    assert not (tmp_path / "out").exists()


def test_empty_input_cannot_pass(tmp_path: Path) -> None:
    # Given a missing coverage set disguised as a valid JSON list.
    path = tmp_path / "input.json"
    _ = path.write_text("[]")
    # When exporting it.
    with pytest.raises(ValueError, match="nonempty"):
        _ = process.run(path, tmp_path / "out")
    # Then no empty success report is emitted.
    assert not (tmp_path / "out").exists()
