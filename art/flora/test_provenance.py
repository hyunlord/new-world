"""Rejected attempts remain charged to the real image-generation budget."""
from pathlib import Path

import pytest

from art.flora.provenance import AttemptInput, validate_budget


def test_duplicate_calls_cannot_hide_in_ledger() -> None:
    # Given a duplicated generation slot.
    row = AttemptInput(id='sample', attempt=1, source=Path('raw.png'), prompt='tree')
    # When its budget is checked, then duplicate accounting fails.
    with pytest.raises(ValueError, match='unique-call'):
        validate_budget((row, row))


def test_missing_attempt_cannot_reset_retry_count() -> None:
    # Given only attempt3, without preserved earlier calls.
    row = AttemptInput(id='sample', attempt=3, source=Path('raw.png'), prompt='tree')
    # When its budget is checked, then the incomplete ledger fails.
    with pytest.raises(ValueError, match='gaps'):
        validate_budget((row,))


def test_three_recorded_attempts_are_all_charged() -> None:
    # Given all three calls, including two rejected candidates.
    rows = tuple(AttemptInput(id='sample', attempt=n, source=Path(f'raw{n}.png'), prompt='tree') for n in (1, 2, 3))
    # When budget is checked.
    count = validate_budget(rows)
    # Then all calls remain accounted for.
    assert count['sample'] == 3
