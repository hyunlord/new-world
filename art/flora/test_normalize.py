# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pydantic", "pytest"]
# ///
# How to run: python3 -m pytest art/flora/test_normalize.py -q
"""Whole-object transforms preserve source bytes and paired registration."""
import json
from pathlib import Path

import pytest
from PIL import Image, ImageDraw

from art.flora.normalize import (
    apply_transform,
    digest,
    normalize_all,
    opaque_bbox,
    plan_transform,
    production_spec,
)


def source_image() -> Image.Image:
    image = Image.new("RGBA", (512, 512))
    draw = ImageDraw.Draw(image)
    draw.rectangle((228, 260, 280, 459), fill=(90, 65, 40, 255))
    draw.ellipse((100, 60, 410, 340), fill=(70, 120, 45, 255))
    return image


@pytest.mark.parametrize("asset", ["sample_large_v1", "sample_small_v1", "sample_stump"])
def test_registered_geometry_is_applied_once(asset: str) -> None:
    # Given a transparent whole object and a D2 geometry record.
    source = source_image()
    spec = production_spec(asset)
    # When the registered whole-object transformation is applied.
    result = apply_transform(source, plan_transform(source, spec))
    # Then the base is exact and opaque height is within raster tolerance.
    bbox = opaque_bbox(result)
    assert bbox is not None
    assert result.size == spec.canvas
    assert bbox[3] - 1 == spec.anchor[1]
    assert abs(bbox[3] - bbox[1] - spec.height) <= 2


def test_bare_uses_same_transform_without_refitting() -> None:
    # Given a bare counterpart with a deliberately shorter upper branch extent.
    source = source_image()
    bare = Image.new("RGBA", source.size)
    ImageDraw.Draw(bare).rectangle((228, 260, 280, 459), fill=(90, 65, 40, 255))
    transform = plan_transform(source, production_spec("sample_large_v1"))
    # When both sources receive precisely the same transform.
    summer_output = apply_transform(source, transform)
    bare_output = apply_transform(bare, transform)
    # Then the exposed lower trunk is byte identical, and winter is not height-fitted.
    assert summer_output.crop((0, 145, 192, 192)).tobytes() == bare_output.crop((0, 145, 192, 192)).tobytes()
    bbox = opaque_bbox(bare_output)
    assert bbox is not None
    assert bbox[3] - bbox[1] < 160


def test_canvas_mismatch_rejects_unproven_bare_registration() -> None:
    # Given a transform registered on a different source canvas.
    transform = plan_transform(source_image(), production_spec("sample_large_v1"))
    # When an incompatible bare source arrives; then normalization rejects it.
    with pytest.raises(ValueError, match="canvases differ"):
        apply_transform(Image.new("RGBA", (400, 512)), transform)


def test_pipeline_preserves_original_and_records_pair(tmp_path: Path) -> None:
    # Given external completed records for a summer and bare source.
    external = tmp_path / "external"
    external.mkdir()
    source = external / "summer.png"
    bare = external / "bare.png"
    source_image().save(source)
    source_image().save(bare)
    original_hash = digest(source)
    completed = external / "completed.json"
    completed.write_text(json.dumps([
        {"id": "sample_large_v1", "source": str(source), "prompt": "reference"},
        {"id": "sample_large_bare", "source": str(bare)},
    ]))
    # When the public pipeline runs against a temporary repository.
    records = normalize_all(completed, tmp_path / "repo")
    # Then immutable originals and paired operations are hash-bound in evidence.
    assert digest(source) == original_hash
    assert digest(Path(records[0].preserved_original)) == original_hash
    assert records[0].transform == records[1].transform
    assert records[1].transform_reference == "sample_large_v1"
    assert records[0].provenance["prompt"] == "reference"
    assert (tmp_path / "repo/docs/v8/arttrees/GENERATION.json").exists()
