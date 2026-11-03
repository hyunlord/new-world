# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_static_run.StaticTests
"""Pinned historical shell data for tests of retired gate modes and adapters."""
import hashlib
from pathlib import Path
from typing import Final

from generic_gate import require

LEGACY_GATE_SHA256: Final = 'ba949251efa089536c44b9883515d017c7254122b4167756833dfd7070d8c523'
LEGACY_GATE_COMMIT: Final = 'c9c5055423e0502938fa999f2e83ea48e271dc78'


def legacy_gate_bytes(path: Path | None = None) -> bytes:
    """Read exact historical tools/v8/gate.sh bytes, independent of adapter constants."""
    selected = Path(__file__).with_name('fixtures')/'gate_legacy_p107.sh' if path is None else path
    raw = selected.read_bytes()
    require(hashlib.sha256(raw).hexdigest() == LEGACY_GATE_SHA256,
            str(selected), 'historical gate fixture changed')
    return raw
