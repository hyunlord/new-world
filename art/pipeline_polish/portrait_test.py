# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pydantic", "pytest"]
# ///
# How to run: uv run --with pillow --with pydantic --with pytest python -m pytest art/pipeline_polish/portrait_test.py
"""Check identity control selection and the actual blinded image contents."""
from __future__ import annotations

from pathlib import Path

import pytest
from PIL import Image, ImageOps
from pydantic import BaseModel, TypeAdapter

from art.pipeline_polish.portrait_packets import PAPER, distractors, sprite
from art.pipeline_polish.portrait_prepare import DATA, ROOT, TARGETS, digest


class Key(BaseModel):
    id: str
    look: str
    correct: str
    options: dict[str, str]
    image_sha256: str
    portrait: Path


class Reference(BaseModel):
    path: Path
    sha256: str


class Call(BaseModel):
    status: str
    started_at: str
    completed_at: str
    raw: Path
    raw_sha256: str
    source: Path
    tool_sha256: str
    prompt: str
    references: list[Reference]


def test_distractors_keep_cohort_and_include_distinct_hair_and_top() -> None:
    # Given same and different age/sex controls in an intentionally unsorted list.
    population = ("child_F_hide_wrap_bun", "adult_F_fur_cloak_bun",
                  "adult_F_hide_wrap_short", "adult_M_hide_wrap_short",
                  "adult_F_fur_cloak_short", "adult_F_hide_wrap_bun")
    # When selecting the registered three controls.
    result = distractors("adult_F_hide_wrap_bun", population)
    # Then the exact same-top, same-hair, both-different order is deterministic.
    assert result == ("adult_F_hide_wrap_short", "adult_F_fur_cloak_bun", "adult_F_fur_cloak_short")


def test_missing_same_cohort_controls_fail_instead_of_crossing_cohorts() -> None:
    # Given an undersized cohort with tempting opposite-sex controls.
    population = ("adult_F_hide_wrap_bun", "adult_M_hide_wrap_short",
                  "child_F_fur_cloak_short", "adult_F_hide_wrap_short")
    # When three controls cannot be selected, then the packet must not be created.
    with pytest.raises(ValueError, match="Fewer than three"):
        _ = distractors("adult_F_hide_wrap_bun", population)


def test_real_blind_packet_contains_exact_portraits_and_2x_sprite_options() -> None:
    # Given the frozen hidden key; production reviewers never receive this file.
    keys = TypeAdapter(list[Key]).validate_json((DATA / "blind/answer_key.json").read_text())
    assert len(keys) == 34
    assert len({key.look for key in keys}) == 34
    # When decoding each actual question image.
    for key in keys:
        path = DATA / "blind/packet" / f"{key.id}.png"
        assert digest(path) == key.image_sha256
        with Image.open(path) as panel, Image.open(ROOT / key.portrait) as portrait:
            # Then the exact128 portrait and all four2x options match source pixels.
            assert panel.crop((18, 76, 146, 204)).tobytes() == portrait.convert("RGB").tobytes()
            assert key.options[key.correct] == key.look
            assert len(set(key.options.values())) == 4
            for index, option in enumerate("ABCD"):
                person = sprite(key.options[option])
                person = ImageOps.scale(person, 2, Image.Resampling.NEAREST)
                expected = Image.new("RGB", person.size, PAPER)
                expected.paste(person, (0, 0), person)
                x = 180 + index * 176
                assert panel.crop((x, 36, x + person.width, 36 + person.height)).tobytes() == expected.tobytes()


def test_only_nine_authorized_portraits_have_new_files() -> None:
    # Given generated canonical files, then no passing ENV1 look was regenerated.
    actual = {path.stem.removesuffix("_try1") for path in (ROOT / "art/portrait/polish").glob("*.png")}
    assert actual == set(TARGETS)


def test_call_ledger_matches_all_actual_outputs_and_references() -> None:
    # Given the completed ten-call ledger, including the diagnostic scorecard.
    rows = TypeAdapter(list[Call]).validate_json((DATA / "generation_ledger.json").read_text())
    assert len(rows) == 10
    # When rehashing source files, then every recorded generation pair and input agrees.
    for row in rows:
        assert row.status == "COMPLETED"
        assert row.started_at < row.completed_at
        assert digest(row.raw) == row.raw_sha256
        assert digest(row.source) == row.tool_sha256 == row.raw_sha256
        assert row.prompt
        for reference in row.references:
            assert digest(reference.path) == reference.sha256
