# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pytest", "pydantic", "scikit-image"]
# ///
# How to run: uv run --with pytest --with pillow --with numpy --with pydantic --with scikit-image python -m pytest art/pipeline_struct/test_normalize.py
"""Normalization preserves image content while declaring scale and placement."""
from __future__ import annotations

import importlib
import importlib.util

import numpy as np
import pytest
from PIL import Image

from art.compositor.models import AssetEntry
from art.pipeline_struct.normalize import AttemptCheck


def candidate(attempt: int, numeric: bool) -> AttemptCheck:
    return AttemptCheck("bedroll", attempt, "sprite", "raw.png", "rawhash", "export.png",
                        "exporthash", (), None, None, None, None, numeric, None,
                        AssetEntry(id="bedroll", path="bedroll.png", width=20, height=14))


def test_normalizer_is_available() -> None:
    # Given the requested standalone export pipeline.
    # When resolving the module.
    found = importlib.util.find_spec("art.pipeline_struct.normalize")
    # Then the exporter exists.
    assert found is not None


def test_uniform_scale_sets_height_and_declares_base() -> None:
    # Given a transparent image with a 40x80 body and unchanged constant RGB.
    module = importlib.import_module("art.pipeline_struct.normalize")
    source = Image.new("RGBA", (100, 120))
    source.paste((100, 150, 200, 255), (30, 20, 70, 100))
    before = source.tobytes()
    # When exported to 40px body height.
    result = module.normalize_sprite(source, 40)
    # Then size and base agree, aspect is preserved and input is untouched.
    rgba = np.asarray(result.image)
    ys, xs = np.nonzero(rgba[:, :, 3] >= 128)
    assert int(ys.max() - ys.min() + 1) == 40
    assert int(xs.max() - xs.min() + 1) == 20
    assert result.anchor[1] == int(ys.max())
    assert result.scale == 0.5
    assert source.tobytes() == before
    assert result.image.getpixel((result.anchor[0], result.anchor[1] - 15)) == (100, 150, 200, 255)


@pytest.mark.parametrize("mode, colour", [("RGB", (100, 150, 200)), ("RGBA", (100, 150, 200, 255)), ("RGBA", (0, 0, 0, 0))])
def test_background_and_empty_images_are_rejected(mode: str, colour: tuple[int, ...]) -> None:
    # Given absent transparency or no opaque subject.
    module = importlib.import_module("art.pipeline_struct.normalize")
    source = Image.new(mode, (64, 64), colour)
    # When normalized.
    # Then no cutout/background replacement is synthesized.
    with pytest.raises(ValueError):
        module.normalize_sprite(source, 30)


def test_component_resamples_exact_size_and_preserves_transparent_hole() -> None:
    # Given a generated component with an enclosed transparent opening.
    module = importlib.import_module("art.pipeline_struct.normalize")
    source = Image.new("RGBA", (80, 160))
    source.paste((80, 60, 40, 255), (10, 20, 70, 140))
    source.paste((0, 0, 0, 0), (25, 50, 55, 110))
    before = source.tobytes()
    # When globally resampled to the declared structural dimensions.
    result = module.normalize_component(source, (24, 60))
    # Then dimensions match and no filling or local pixel surgery occurred.
    assert result.image.size == (24, 60)
    assert result.image.getpixel((12, 30))[3] == 0
    assert result.crop == (10, 20, 70, 140)
    assert source.tobytes() == before


def test_visually_failed_numeric_pass_cannot_displace_readable_candidate() -> None:
    # Given a tiny numeric-pass first try and a readable third try with metric failure.
    module = importlib.import_module("art.pipeline_struct.normalize")
    attempts = (candidate(1, True), candidate(3, False))
    readings = (module.AssetReading(asset="bedroll", attempt=1, readable=False, reading="Tiny"),
                module.AssetReading(asset="bedroll", attempt=3, readable=True, reading="98px narrower than target"))
    # When choosing a display candidate.
    chosen = module.select_attempt(attempts, readings)
    # Then visual failure never wins and metric failure is not erased.
    assert chosen.attempt == 3
    assert not chosen.numeric_ready
    assert "NUMERIC_FAIL" in chosen.entry.status
    assert "98px narrower" in chosen.entry.status


def test_readable_numeric_pass_precedes_later_readable_metric_failure() -> None:
    # Given two readable attempts with different numeric outcomes.
    module = importlib.import_module("art.pipeline_struct.normalize")
    readings = tuple(module.AssetReading(asset="bedroll", attempt=n, readable=True, reading="Readable") for n in (1, 2))
    # When selecting.
    chosen = module.select_attempt((candidate(1, True), candidate(2, False)), readings)
    # Then the verified numeric candidate remains preferred.
    assert chosen.attempt == 1
    assert "OWNER_PENDING" in chosen.entry.status


def test_all_visual_failures_remain_visually_failed() -> None:
    # Given no visually usable candidate, even though numerics pass.
    module = importlib.import_module("art.pipeline_struct.normalize")
    readings = tuple(module.AssetReading(asset="bedroll", attempt=n, readable=False, reading="Tiny") for n in (1, 2))
    # When retaining a diagnostic candidate.
    chosen = module.select_attempt((candidate(1, True), candidate(2, True)), readings)
    # Then latest diagnostic is clearly failed, not accepted.
    assert chosen.attempt == 2
    assert chosen.entry.status.startswith("VISUAL_FAIL")
    assert chosen.numeric_ready  # Literal metric evidence remains unchanged.


def test_unreviewed_candidate_is_preferred_over_explicit_visual_failure() -> None:
    # Given an unreviewed older attempt and an explicitly failed newer attempt.
    module = importlib.import_module("art.pipeline_struct.normalize")
    readings = (module.AssetReading(asset="bedroll", attempt=2, readable=False, reading="Tiny"),)
    # When selecting with no readable entries.
    chosen = module.select_attempt((candidate(1, False), candidate(2, True)), readings)
    # Then the excluded visual failure cannot silently win.
    assert chosen.attempt == 1
    assert chosen.entry.status == "UNREVIEWED_NUMERIC_FAIL"
