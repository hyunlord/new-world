"""Behavioral checks for strict per-reviewer POLISH acceptance bars."""
from __future__ import annotations

import pytest

from art.pipeline_polish.blind_score import (
    AnimalTrial,
    IdentityTrial,
    ItemTrial,
    score_animals,
    score_identity,
    score_items,
)


def test_animal_score_does_not_average_reviewers_or_scales() -> None:
    # Given one reviewer just below the 2x pose bar, despite strong 1x answers.
    rows = tuple(AnimalTrial(id=f"{scale}-{i}", scale=scale, species_correct=True,
                            pose_correct=i < (20 if scale == 1 else 15))
                 for scale in (1, 2) for i in range(20))
    # When scoring the complete set.
    result = score_animals(rows)
    # Then the failed group prevents a pass.
    assert result.passed is False
    assert result.groups[1].pose_correct == 15


def test_animals_exact_thresholds_pass() -> None:
    # Given exactly the registered thresholds.
    rows = tuple(AnimalTrial(id=f"{scale}-{i}", scale=scale, species_correct=i < 19,
                            pose_correct=i < (14 if scale == 1 else 16))
                 for scale in (1, 2) for i in range(20))
    # When scoring both scales.
    result = score_animals(rows)
    # Then equality passes.
    assert result.passed is True


def test_missing_animal_trial_fails_instead_of_shrinking_denominator() -> None:
    # Given an incomplete set.
    rows = tuple(AnimalTrial(id=str(i), scale=1, species_correct=True, pose_correct=True)
                 for i in range(19))
    # When scoring, then reject the incomplete instrument.
    with pytest.raises(ValueError, match="20"):
        _ = score_animals(rows)


def test_identity_requires_thirty_of_exactly_thirtyfour() -> None:
    # Given 29 correct from a complete packet.
    rows = tuple(IdentityTrial(id=str(i), correct=i < 29) for i in range(34))
    # When scoring.
    result = score_identity(rows)
    # Then the original bar still fails.
    assert result.passed is False
    assert result.correct == 29


def test_duplicate_identity_question_is_rejected() -> None:
    # Given 34 rows with a repeated question.
    rows = tuple(IdentityTrial(id="same", correct=True) for _ in range(34))
    # When scoring, then duplicates cannot inflate the total.
    with pytest.raises(ValueError, match="duplicate"):
        _ = score_identity(rows)


ITEMS = ("firewood", "ground_axe", "spear", "cured_hide", "hide_wrap", "fur_cloak",
         "wolf_hide", "clay", "raw_pot", "clay_pot_empty", "clay_pot_filled", "meat",
         "flint_nodule", "log_pile_small", "log_pile_medium", "log_pile_large",
         "stone_pile", "flint_pile", "flint_blade", "stone_axe", "hide", "berry_basket")


def test_item_pair_failure_cannot_be_hidden_by_twentyone_correct() -> None:
    # Given 21/22 exact names but firewood confused with log at 32px.
    rows = tuple(ItemTrial(id=f"{size}-{name}", asset=name, size=size,
                          exact=name != "firewood", garment=name in {"hide_wrap", "fur_cloak"})
                 for size in (32, 64) for name in ITEMS)
    # When scoring.
    result = score_items(rows)
    # Then the aggregate bar passes while the pair gate fails.
    assert result.groups[0].exact_correct == 21
    assert result.groups[0].count_pass is True
    assert result.pairs_pass is False
    assert result.passed is False


def test_garment_recognition_does_not_rescue_exact_kind() -> None:
    # Given all garments named only generically at 64px and four other misses.
    misses = {"hide_wrap", "fur_cloak", "flint_pile", "flint_blade", "wolf_hide", "clay"}
    rows = tuple(ItemTrial(id=f"{size}-{name}", asset=name, size=size,
                          exact=name not in misses, garment=name in {"hide_wrap", "fur_cloak"})
                 for size in (32, 64) for name in ITEMS)
    # When scoring.
    result = score_items(rows)
    # Then garment class cannot improve exact totals.
    assert result.groups[1].exact_correct == 16
    assert result.passed is False
