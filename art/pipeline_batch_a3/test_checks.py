# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic", "scikit-image", "pytest"]
# ///
# How to run: PYTHONPATH=. python -m pytest art/pipeline_batch_a3/test_checks.py
"""Regression checks for alpha rejection and immutable item anchor diagnostics."""

from __future__ import annotations

from PIL import Image

from art.pipeline_batch_a3.checks import check_alpha, inspect_tile


def test_empty_generation_is_rejected() -> None:
    # Given an entirely transparent generation.
    image = Image.new("RGBA", (64, 64))
    # When its source alpha is inspected.
    result = check_alpha(image)
    # Then it cannot become an item candidate.
    assert not result.passed
    assert not result.has_subject


def test_opaque_background_is_rejected() -> None:
    # Given an opaque canvas that must never be silently background-removed.
    image = Image.new("RGBA", (64, 64), (200, 190, 170, 255))
    # When source alpha is inspected.
    result = check_alpha(image)
    # Then the transparency gate rejects it.
    assert not result.passed
    assert not result.has_transparency


def test_wrong_anchor_is_reported_without_repositioning() -> None:
    # Given an otherwise isolated item whose base is four pixels too low.
    image = Image.new("RGBA", (64, 64))
    image.paste((160, 140, 110, 255), (20, 32, 44, 48))
    original = image.tobytes()
    # When the immutable exported tile is inspected.
    result = inspect_tile(image)
    # Then its wrong anchor fails and the inspected pixels remain unchanged.
    assert not result.structural_pass
    assert result.projection.base_error == 4
    assert image.tobytes() == original


def test_valid_anchor_does_not_claim_style_approval() -> None:
    # Given a flat, correctly anchored silhouette with no measurable lighting.
    image = Image.new("RGBA", (64, 64))
    image.paste((160, 140, 110, 255), (20, 28, 44, 44))
    # When the immutable tile is inspected.
    result = inspect_tile(image)
    # Then geometry passes independently while lighting remains failed.
    assert result.structural_pass
    assert not result.projection.light_passed


def test_faint_source_fringe_is_reported_without_rejection() -> None:
    # Given an isolated source with one alpha=1 pixel at the canvas boundary.
    image = Image.new("RGBA", (64, 64))
    image.paste((160, 140, 110, 255), (20, 28, 44, 44))
    image.putpixel((0, 0), (160, 140, 110, 1))
    # When inspected under the existing alpha>=128 silhouette convention.
    result = check_alpha(image)
    # Then the faint pixel is visible in diagnostics without becoming clipping.
    assert result.passed
    assert result.border_alpha_max == 1
    assert result.fringe_pixels == 1
