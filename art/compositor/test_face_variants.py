"""Selector contracts, age boundaries, and explicit source aliases."""
from __future__ import annotations

import pytest
from pydantic import ValidationError

from art.compositor.face_variants import (
    ROOT,
    FacePlan,
    LookVariants,
    baked_frame,
    choose_variant,
)


def test_inventory_and_aliases() -> None:
    plan = FacePlan.read()
    assert len(plan.looks) == 34
    assert sum(look.look.startswith("adult_") for look in plan.looks) == 16
    assert len(plan.fallbacks) == 2
    assert len(plan.owner_sample) == 8
    assert len(set(plan.identity_sample)) == 36
    for alias, source in plan.fallbacks.items():
        assert plan.for_look(alias) == plan.for_look(source)


def test_hash_selection_is_stable_and_reaches_every_variant() -> None:
    look = FacePlan.read().looks[0]
    assert [choose_variant(value, 30, look).id for value in (0, 1, 2, 3, 2**32 + 1)] == ["v1", "v2", "v5", "v6", "v2"]
    assert {choose_variant(value, 30, look).id for value in range(100)} == {"v1", "v2", "v3", "v4", "v5", "v6"}


def test_age_fifty_selects_only_older_adults() -> None:
    plan = FacePlan.read()
    adult = plan.for_look("adult_M_fur_cloak_bun")
    assert choose_variant(0, 49, adult).id == "v1"
    assert {choose_variant(value, 50, adult).id for value in range(100)} == {"v4", "v5"}
    child = plan.for_look("child_M_fur_cloak_bun")
    assert choose_variant(0, 50, child).age_band == "child"


def test_invalid_input_is_explicit() -> None:
    plan = FacePlan.read()
    with pytest.raises(ValueError, match="Unknown character"):
        _ = plan.for_look("missing")
    with pytest.raises(ValueError, match="nonnegative"):
        _ = choose_variant(-1, 30, plan.looks[0])
    with pytest.raises(ValueError, match="nonnegative"):
        _ = choose_variant(1, -1, plan.looks[0])
    with pytest.raises(ValidationError, match="six ordered"):
        _ = LookVariants(look=plan.looks[0].look, variants=plan.looks[0].variants[:5])


def test_baked_reference_resolves_real_source_variant() -> None:
    plan = FacePlan.read()
    look = plan.for_look("adult_M_hide_wrap_long_tied")
    frame = baked_frame(ROOT, look, look.variants[3])
    assert frame.is_file()
    assert frame.parts[-4:] == ("adult_M_hide_wrap_bun", "v4", "S", "idle.png")
