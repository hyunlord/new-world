"""Recompute the pinned historical raw receipt with an explicit repository mapping.

Only the old config's checkout-location metadata is mapped to the current owned
checkout during validation. Original config/raw bytes and result remain intact.
No process executes in the historical checkout; frozen verifier source is unchanged.
"""
from __future__ import annotations

from pathlib import Path
import sys

import core_ui_successor as ui
import core_perf_check as perf


def mapped_config(config: perf.Document, expected: perf.Document, root: Path) -> perf.Document:
    """Accept only the exact inherited config, then map its repo metadata."""
    ui.need(config.values == expected.values, 'unbound historical config mapping')
    value = dict(config.values)
    value['repo'] = str(root)
    return perf.Document.parse(value)


def recompute(session: Path, output: Path) -> int:
    """Run unchanged raw verifier under an exact, reviewed location adapter."""
    root = Path(__file__).resolve().parents[2]
    data = ui.load_config(root)
    ui.source(root, data)
    evaluation, _ = ui.history(root, data)
    original_path = ui.read_ref(root, evaluation['perf_config'])
    ui.need(original_path.parent == session, 'wrong historical session')
    original = perf.Document.read(original_path)
    expected_checker = ui.inventory.committed(root, ui.BASE, 'tools/v8')['tools/v8/core_perf_check.py']
    ui.need(ui.freeze.digest(Path(perf.__file__).resolve()) == expected_checker, 'historical raw verifier changed')
    original_verify = perf.verify_config

    def verify_mapped(config: perf.Document) -> None:
        original_verify(mapped_config(config, original, root))

    perf.verify_config = verify_mapped
    try:
        result = perf.check(session, output)
    finally:
        perf.verify_config = original_verify
    ui.source(root, data)
    ui.need(ui.freeze.digest(output) == ui.freeze.digest(ui.read_ref(root, evaluation['perf_receipt'])),
            'historical raw recomputation differs')
    return result


if __name__ == '__main__':
    if len(sys.argv) != 3:
        raise SystemExit('usage: core_ui_successor_history.py HISTORICAL_SESSION NEW_EXTERNAL_RECEIPT')
    raise SystemExit(recompute(Path(sys.argv[1]), Path(sys.argv[2])))
