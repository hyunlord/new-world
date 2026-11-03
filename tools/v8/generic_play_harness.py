"""Reject an uninstrumented product launcher before the play owner can start."""
import stat
from pathlib import Path
from typing import Final

from generic_gate import require
from generic_process import file_hash
from generic_sources import safe_path
from mux2_gate_inputs import Node

LAUNCHER: Final = 'tools/play.sh'


def check_launcher(session: Node, arm: str) -> None:
    """Caller has loaded the authenticated session and full materialization closure."""
    harness = session.field('harness')
    product = session.field('arms').field(arm)
    selected = safe_path(Path(harness.field('root').text()), LAUNCHER)
    actual = safe_path(Path(product.field('root').text()), LAUNCHER)
    expected = harness.field('inputs').field(LAUNCHER).text()
    require(product.field('source_inputs').field(LAUNCHER).value == expected,
            '/play/launcher', 'product launcher differs from admitted H launcher')
    for path in (selected, actual):
        mode = path.lstat().st_mode
        require(stat.S_ISREG(mode) and stat.S_IMODE(mode) == 0o755,
                '/play/launcher', 'launcher must retain regular executable H mode')
        require(file_hash(path) == expected, '/play/launcher', 'launcher bytes differ from admitted H')
