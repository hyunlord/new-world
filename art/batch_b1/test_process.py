# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic", "pillow", "numpy", "scipy", "scikit-image", "pytest", "typer"]
# ///
# How to run: python3 -m pytest art/batch_b1/test_process.py
"""Whole-sheet provenance, scale and adoption boundary regressions."""

import json
from pathlib import Path
from typing import Literal, assert_never

import pytest
from PIL import Image, ImageDraw
from pydantic import ValidationError

from art.batch_b1.adopt import adopt
from art.batch_b1.models import Generation, VisualVerdict, digest, save
from art.batch_b1.process import process


def fixture_record(tmp_path: Path, age: str = "adult") -> Path:
    image = Image.new("RGBA", (1536, 1024))
    draw = ImageDraw.Draw(image)
    for row in range(2):
        for column in range(5):
            left, top = column * 307 + 70, row * 512 + 100
            width, height = (200, 70) if column == 4 else (65, 200)
            draw.rectangle(
                (left, top, left + width - 1, top + height - 1), fill=(100, 70, 40, 255)
            )
    raw = tmp_path / "raw.png"
    image.save(raw)
    record = tmp_path / "record.json"
    _ = record.write_text(
        json.dumps(
            {
                "raw": str(raw),
                "look": f"{age}_F_bronze_linen_bun",
                "age": age,
                "prompt": "fixture",
                "seed": None,
                "attempt": 1,
            }
        )
    )
    return record


def verdict(trial: Path, score: int = 94) -> Path:
    path = trial / "verdict.json"
    save(
        path,
        VisualVerdict(
            score=score,
            verdict="pass",
            category_match=True,
            differences=(),
            suggestions=(),
            reasoning="Fixture review.",
            summary_sha256=digest(trial / "summary.json"),
        ),
    )
    return path


@pytest.mark.parametrize(("age", "target"), [("adult", 56), ("child", 40)])
def test_common_scale_and_contacts(tmp_path: Path, age: str, target: int) -> None:
    # Given one complete ten-figure source.
    record = fixture_record(tmp_path, age)
    raw_before = (tmp_path / "raw.png").read_bytes()
    trial = tmp_path / "trial"
    # When B1 normalization runs through the actual geometry API.
    result = process(record, trial)
    # Then all eight standing frames use S idle's scale and exact contact line.
    assert result.passed
    assert len(result.frames) == 10
    assert {frame.standing_scale for frame in result.frames} == {target / 200}
    for frame in result.frames:
        with Image.open(trial / frame.path) as image:
            box = image.getchannel("A").point([0] * 16 + [255] * 240).getbbox()
            assert box is not None
            bottom = box[3] - 1
        assert bottom == (72 if frame.pose == "sleep" else 104)
        assert frame.normalization.scale == target / 200
    assert (tmp_path / "raw.png").read_bytes() == raw_before


def test_unknown_age_or_unsafe_look_rejected(tmp_path: Path) -> None:
    # Given malformed identity fields at the JSON boundary.
    data = fixture_record(tmp_path).read_text()
    # When parsing the request.
    with pytest.raises(ValidationError):
        _ = Generation.model_validate_json(
            data.replace("adult_F_bronze_linen_bun", "../../outside")
        )
    # Then an unknown age also fails rather than falling back to adult.
    with pytest.raises(ValidationError):
        _ = Generation.model_validate_json(data.replace('"adult"', '"elder"'))


def test_fourth_generation_attempt_rejected(tmp_path: Path) -> None:
    # Given the session's three-attempt limit for each asset.
    data = fixture_record(tmp_path).read_text()
    # When a generation record requests a fourth attempt.
    # Then parsing refuses it before processing or adoption can occur.
    with pytest.raises(ValidationError, match="less than or equal to 3"):
        _ = Generation.model_validate_json(data.replace('"attempt": 1', '"attempt": 4'))


def test_failed_sheet_cannot_adopt(tmp_path: Path) -> None:
    # Given a source whose sleep panels were replaced by tall squares.
    record = fixture_record(tmp_path)
    with Image.open(tmp_path / "raw.png") as image:
        draw = ImageDraw.Draw(image)
        draw.rectangle((1298, 100, 1497, 299), fill=(100, 70, 40, 255))
        image.save(tmp_path / "raw.png")
    trial = tmp_path / "trial"
    result = process(record, trial)
    # When an otherwise positive reviewer tries adopting the failed sheet.
    with pytest.raises(ValueError, match="structural"):
        _ = adopt(trial, verdict(trial), tmp_path / "delivery")
    # Then no partial frames are adopted.
    assert not result.passed
    assert not (tmp_path / "delivery").exists()


@pytest.mark.parametrize("damage", ["missing", "changed", "low_score", "stale_review"])
def test_invalid_evidence_never_adopts(
    tmp_path: Path, damage: Literal["missing", "changed", "low_score", "stale_review"]
) -> None:
    # Given a structurally valid complete trial and recorded review.
    trial = tmp_path / "trial"
    _ = process(fixture_record(tmp_path), trial)
    review = verdict(trial, 89 if damage == "low_score" else 94)
    match damage:
        case "missing":
            (trial / "S/idle.png").unlink()
        case "changed":
            _ = (trial / "S/idle.png").write_bytes(b"changed")
        case "stale_review":
            with (trial / "summary.json").open("a") as stream:
                _ = stream.write(" ")
        case "low_score":
            pass
        case _:
            assert_never(damage)
    # When adopting the current bytes.
    with pytest.raises((ValueError, OSError)):
        _ = adopt(trial, review, tmp_path / "delivery")
    # Then validation occurs before any delivery directory is made.
    assert not (tmp_path / "delivery").exists()


def test_verified_adoption_copies_exact_bytes(tmp_path: Path) -> None:
    # Given a complete reviewed look.
    trial = tmp_path / "trial"
    result = process(fixture_record(tmp_path), trial)
    # When adopting the exact trial.
    destination = adopt(trial, verdict(trial), tmp_path / "delivery")
    # Then all ten frames retain identical file bytes and sidecars.
    assert len(list(destination.glob("*/*.png"))) == 10
    for frame in result.frames:
        assert digest(destination / frame.path) == frame.sha256
        assert (destination / frame.path).with_suffix(".json").read_bytes() == (
            trial / frame.path
        ).with_suffix(".json").read_bytes()


def test_tools_layout_never_normalizes_a_sleep_frame(tmp_path: Path) -> None:
    # Given ten standing figures and four explicit tool/load columns.
    record = fixture_record(tmp_path)
    generation = Generation.model_validate_json(record.read_bytes())
    generation = generation.model_copy(
        update={
            "layout": "tools",
            "frame_ids": ("idle", "sickle", "hoe", "quern", "basket"),
        }
    )
    save(record, generation)
    with Image.open(tmp_path / "raw.png") as image:
        draw = ImageDraw.Draw(image)
        for row in range(2):
            draw.rectangle((1228, row * 512, 1535, row * 512 + 511), fill=(0, 0, 0, 0))
            draw.rectangle(
                (1298, row * 512 + 100, 1362, row * 512 + 299), fill=(100, 70, 40, 255)
            )
        image.save(tmp_path / "raw.png")
    # When processing and adopting the supplementary tool sheet.
    trial = tmp_path / "trial"
    summary = process(record, trial)
    destination = adopt(trial, verdict(trial), tmp_path / "tools")
    # Then all ten canvases and scales remain standing; named outputs survive.
    assert summary.passed
    assert {frame.normalization.scale for frame in summary.frames} == {56 / 200}
    assert {frame.anchor for frame in summary.frames} == {(40, 104)}
    assert (destination / "E/basket.png").exists()


def test_portrait_cells_keep_entire_frame_and_order(tmp_path: Path) -> None:
    # Given six opaque square source cells with distinct full-frame colours.
    from art.batch_b1.portraits import adopt as adopt_portraits
    from art.batch_b1.portraits import process as process_portraits

    record = fixture_record(tmp_path)
    image = Image.new("RGB", (1536, 1024))
    draw = ImageDraw.Draw(image)
    colours = [(30 + index * 20, 60, 100) for index in range(6)]
    for index, colour in enumerate(colours):
        x, y = index % 3 * 512, index // 3 * 512
        draw.rectangle((x, y, x + 511, y + 511), fill=colour)
    image.save(tmp_path / "raw.png")
    # When full-cell slicing and adoption run.
    trial = tmp_path / "portraits"
    summary = process_portraits(record, trial)
    destination = adopt_portraits(trial, verdict(trial), tmp_path / "delivery")
    # Then every edge and centre retains its own cell and ordering at128px.
    assert len(summary.portraits) == 6
    for index, portrait in enumerate(summary.portraits):
        with Image.open(destination / portrait.path) as result:
            assert result.size == (128, 128)
            assert result.getpixel((0, 0)) == (*colours[index], 255)
            assert result.getpixel((127, 127)) == (*colours[index], 255)


def test_failed_portrait_summary_cannot_adopt(tmp_path: Path) -> None:
    from art.batch_b1.portraits import adopt as adopt_portraits
    from art.batch_b1.portraits import process as process_portraits

    # Given a failed portrait summary and a fresh positive review of its bytes.
    record = fixture_record(tmp_path)
    Image.new("RGB", (1536, 1024), (100, 70, 40)).save(tmp_path / "raw.png")
    trial = tmp_path / "portraits"
    summary = process_portraits(record, trial)
    save(trial / "summary.json", summary.model_copy(update={"passed": False}))
    review = verdict(trial)
    # When adoption is attempted, the failed result remains withheld.
    with pytest.raises(ValueError, match="structural"):
        _ = adopt_portraits(trial, review, tmp_path / "delivery")
    # Then no partial destination is created.
    assert not (tmp_path / "delivery").exists()
