"""Keep the second-pass retry allowance scoped to the anvil."""

from pathlib import Path
from typing import Literal

import pytest
from pydantic import ValidationError

from art.batch_c3a.workshops.stations import Raw


def test_anvil_accepts_six_total_attempts() -> None:
    """Given three additional owner-approved tries, the sixth can be read."""
    row = Raw(id="anvil", raw=Path("/external/anvil6.png"), attempt=6, prompt="test")
    assert row.attempt == 6


@pytest.mark.parametrize("asset,attempt", [("anvil", 7), ("mason_bench", 4)])
def test_other_attempts_remain_blocked(
    asset: Literal["anvil", "mason_bench"], attempt: int
) -> None:
    """The anvil exception must not expand the bench budget or allow try7."""
    with pytest.raises(ValidationError):
        _ = Raw(id=asset, raw=Path("/external/raw.png"), attempt=attempt, prompt="test")
