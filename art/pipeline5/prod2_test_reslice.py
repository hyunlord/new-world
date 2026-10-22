# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "scikit-image", "pytest"]
# ///
# How to run: python -m pytest art/pipeline5/prod2_test_reslice.py
"""Selection and comparison evidence must not hide failures or altered pixels."""
from dataclasses import replace
from pathlib import Path

from art.pipeline5.prod1_records import SheetRecord
from art.pipeline5.prod2_reslice import best, output_hashes


def test_first_pass_wins_and_reject_ties_keep_earliest() -> None:
    base = SheetRecord('adult_M_hide_wrap_short', 'adult', 1, '', '', '', '', False, 8, ())
    records = (base, replace(base, attempt=2), replace(base, attempt=3, passing_panels=7))
    assert best(records).attempt == 1
    assert best((base, replace(base, attempt=2, passed=True, passing_panels=10),
                 replace(base, attempt=3, passed=True, passing_panels=10))).attempt == 2
    assert best((base, replace(base, attempt=2, passing_panels=9))).attempt == 2


def test_comparison_only_normalizes_root_paths(tmp_path: Path) -> None:
    roots = (tmp_path / 'run1', tmp_path / 'run2')
    for root in roots:
        root.mkdir()
        _ = (root / 'frame.json').write_text('{"raw":"' + str(root) + '/raw.png"}')
        _ = (root / 'frame.png').write_bytes(b'exact PNG content')
    assert output_hashes(roots[0]) == output_hashes(roots[1])
    _ = (roots[1] / 'frame.png').write_bytes(b'changed PNG content')
    assert output_hashes(roots[0]) != output_hashes(roots[1])
