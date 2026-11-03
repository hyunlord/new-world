# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Adapt authenticated whole-stage endpoints without inventing legacy build receipts."""
from pathlib import Path
from typing import TYPE_CHECKING

from generic_gate import require
from mux2_gate_inputs import Node

if TYPE_CHECKING:
    from generic_bench import Product

def from_view(session: Node, arm: str, selected: Node) -> 'Product':
    """A authenticates the view; this conversion retains its actual stage receipt."""
    from generic_bench import Product
    source = session.field('arms').field(arm)
    root = Path(source.field('root').text())
    runtime: list[tuple[str, str]] = []
    for name, raw in selected.field('runtime_inputs').mapping().items():
        path = Path(name)
        require(path.is_absolute() and path.is_relative_to(root) and path == path.resolve(strict=True),
                name, 'runtime outside admitted arm')
        runtime.append((path.relative_to(root).as_posix(), Node(raw).text()))
    return Product(root, Path(selected.field('executable').text()), source.field('source_sha').text(),
        selected.field('executable_sha256').text(), Path(selected.field('build_receipt').text()),
        selected.field('build_receipt_sha256').text(),
        tuple(sorted((name, Node(raw).text()) for name, raw in selected.field('source_inputs').mapping().items())),
        tuple(sorted(runtime)))
