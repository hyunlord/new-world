# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic", "pytest"]
# ///
# How to run: python -m pytest art/climate_colour/test_models.py -q
"""Missing historical olive metadata must never route a tree to a shrub."""

from art.climate_colour.models import Row


def test_legacy_olive_uses_tree_canvas_instead_of_shrub_default() -> None:
    row = Row.model_validate(
        {
            "id": "wild_olive_full_v2_large",
            "source": "art/flora/climate/trees/wild_olive_full_v2_large.png",
            "sha256": "a" * 64,
            "canvas": [192, 192],
            "anchor": [96, 180],
            "footprint": [1, 1],
            "climate": None,
            "season": None,
        }
    )
    assert row.lane == "trees"
    assert row.size == "large"
    assert row.path.endswith("wild_olive_full_v2_large.png")
    assert row.climate is None
