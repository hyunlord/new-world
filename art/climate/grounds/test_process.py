# /// script
# requires-python = ">=3.11"
# dependencies = ["pytest", "pillow", "numpy", "pydantic", "scipy", "scikit-image"]
# ///
# How to run: python -m pytest art/climate/grounds/test_process.py -q
"""Ground boundary tests prevent empty/capped evidence from being accepted."""

from pathlib import Path

import pytest
from pydantic import ValidationError

from art.climate.grounds.process import Generation


def test_attempt_cap_rejects_fourth_generation() -> None:
    """Given a fourth attempt, parsing must reject it before export."""
    with pytest.raises(ValidationError):
        Generation(
            id="tundra_summer",
            attempt=4,
            prompt="one tile",
            refs=(Path("ref"),),
            source=Path("source"),
            raw=Path("raw"),
            seed=None,
        )


def test_missing_source_is_not_a_complete_record() -> None:
    """Given absent source identity, parsing cannot produce evidence."""
    with pytest.raises(ValidationError):
        Generation.model_validate({"id": "tundra_summer", "attempt": 1})


def test_empty_selection_blocks_export(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """No accepted inputs must fail before creating any derivative exports."""
    from art.climate.grounds import export

    (tmp_path / "selection.json").write_text("{}")
    monkeypatch.setattr(export, "DATA", tmp_path)
    with pytest.raises(AssertionError, match="No passing ground master"):
        export.main()
