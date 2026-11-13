# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "pytest"]
# ///
# How to run: python3 -m pytest art/batch_r1/test_process.py
"""Synthetic geometry verifies registration, rejection and evidence identity."""

from pathlib import Path

import pytest
from PIL import Image, ImageDraw

from art.batch_r1.process import FACINGS, Request, process
from art.pipeline5.attempt_integrity import digest
from art.pipeline5.whole_checks import FRAMES


def fixture(root: Path, age: str = "adult") -> Request:
    raw = root / "raw.png"
    sheet = Image.new("RGBA", (1536, 1024))
    draw = ImageDraw.Draw(sheet)
    for row in range(2):
        for column in range(5):
            x, y = column * 307 + 110, row * 512 + 100
            height = 120 if row == 1 and column == 0 else 100
            draw.rectangle((x, y, x + 29, y + height - 1), fill=(95, 60, 32, 255))
    sheet.save(raw)
    reference = root / "reference"
    for facing in FACINGS:
        (reference / facing).mkdir(parents=True)
        for frame in FRAMES:
            size = (112, 80) if frame == "sleep" else (80, 112)
            Image.new("RGBA", size, (100, 70, 30, 255)).save(
                reference / facing / (frame + ".png")
            )
    return Request(raw, age + "_M_plant_fibre_short", root / "output", reference)


@pytest.mark.parametrize(("age", "target"), [("adult", 56), ("child", 40)])
def test_shared_s_idle_scale_and_separate_sleep(
    tmp_path: Path, age: str, target: int
) -> None:
    request = fixture(tmp_path, age)
    result = process(request)
    assert len(result.frames) == 10
    assert result.frames[0].standing_scale == target / 100
    assert result.frames[5].standing_scale == target / 100
    assert (
        result.frames[5].checks.visible_height > result.frames[0].checks.visible_height
    )
    assert (
        not result.structural_pass
    )  # E idle intentionally taller than the fixed S-idle target.
    assert result.frames[4].checks.passed
    assert result.frames[9].checks.passed
    assert result.frames[0].reference_sha256 == digest(request.reference / "S/idle.png")
    assert result.raw_sha256 == digest(request.raw)
    assert (request.output / "comparison_1x.png").exists()
    assert (request.output / "comparison_2x.png").exists()
    with Image.open(request.output / "S/idle.png") as image:
        assert image.size == (80, 112)
        bounds = image.getbbox()
        assert bounds is not None
        assert bounds[3] == 105
    with pytest.raises(FileExistsError):
        _ = process(request)


def test_equal_idles_pass(tmp_path: Path) -> None:
    request = fixture(tmp_path)
    with Image.open(request.raw) as sheet:
        draw = ImageDraw.Draw(sheet)
        draw.rectangle((110, 612, 139, 731), fill=(0, 0, 0, 0))
        draw.rectangle((110, 612, 139, 711), fill=(95, 60, 32, 255))
        sheet.save(request.raw)
    assert process(request).structural_pass


@pytest.mark.parametrize("malformation", ["mode", "size", "count"])
def test_bad_sheet_records_rejection(tmp_path: Path, malformation: str) -> None:
    request = fixture(tmp_path)
    with Image.open(request.raw) as image:
        if malformation == "mode":
            image.convert("RGB").save(request.raw)
        elif malformation == "size":
            image.resize((768, 512)).save(request.raw)
        else:
            ImageDraw.Draw(image).rectangle((110, 100, 139, 199), fill=(0, 0, 0, 0))
            image.save(request.raw)
    result = process(request)
    assert not result.structural_pass
    assert result.error
    assert not result.frames
    assert (request.output / "summary.json").exists()
