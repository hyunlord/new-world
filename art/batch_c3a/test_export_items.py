# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic", "scikit-image", "pytest"]
# ///
# How to run: PYTHONPATH=. python -m pytest art/batch_c3a/test_export_items.py -q
"""Count exports retain ENV1's complete-item layouts and transparent margins."""

import pytest
from PIL import Image
from pydantic import ValidationError

from art.batch_c3a.export_items import Selection, count_tiles


def test_count_layout_uses_whole_icons_at_fixed_offsets() -> None:
    # Given a single opaque pixel in an otherwise transparent 64px icon.
    sprite = Image.new("RGBA", (64, 64))
    sprite.putpixel((32, 32), (120, 80, 40, 255))
    # When count variants are composed.
    exports = count_tiles(sprite, counts=True)
    # Then the single is unchanged and the exact independent ENV1 layout agrees.
    assert [name for name, _ in exports] == ["", "_count_2_8", "_count_9_plus"]
    assert exports[0][1].tobytes() == sprite.tobytes()
    expected = Image.new("RGBA", (64, 64))
    scaled = sprite.resize((42, 42), Image.Resampling.LANCZOS)
    for offset in ((3, 3), (16, 16), (3, 20)):
        expected.alpha_composite(scaled, offset)
    assert exports[1][1].tobytes() == expected.tobytes()


def test_equipment_has_no_count_variants() -> None:
    # Given a complete equipment icon.
    sprite = Image.new("RGBA", (64, 64), (50, 30, 10, 255))
    # When export is requested without counts.
    exports = count_tiles(sprite, counts=False)
    # Then no count pile is invented.
    assert len(exports) == 1
    assert exports[0][1].tobytes() == sprite.tobytes()


def test_selection_rejects_out_of_budget_attempt() -> None:
    # Given a requested fourth attempt.
    # When it is parsed as an approved selection.
    # Then no out-of-budget attempt can be published.
    with pytest.raises(ValidationError):
        _ = Selection.model_validate({"approved": {"iron_adze": 4}})
