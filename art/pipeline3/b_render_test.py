# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pytest"]
# ///
# How to run: uv run --with pytest python -m pytest art/pipeline3/b_render_test.py
"""Boundary tests for native ID pass support, never artwork repair."""
import numpy as np

from art.pipeline3.b_render_finish import support_labels


def test_native_ids_extend_into_ink_without_escaping_alpha() -> None:
    # Given an original index in geometry and a larger rendered-ink silhouette.
    native = np.zeros((5, 5), dtype=np.uint8)
    native[2, 2] = 5
    support = np.zeros((5, 5), dtype=bool)
    support[1:4, 1:4] = True
    original = native.copy()
    # When deriving the mask's Freestyle support.
    result = support_labels(native, support)
    # Then IDs cover ink only and the original index evidence is untouched.
    assert np.array_equal(result != 0, support)
    assert np.all(result[support] == 5)
    assert np.array_equal(native, original)


def test_missing_native_ids_cannot_manufacture_anatomy() -> None:
    # Given an absent native anatomical partition.
    native = np.zeros((4, 4), dtype=np.uint8)
    support = np.ones((4, 4), dtype=bool)
    # When deriving an anatomical map.
    result = support_labels(native, support)
    # Then missing evidence stays missing for the structural checker.
    assert not np.any(result)


def test_native_partition_labels_survive_inside_support() -> None:
    # Given two independently labeled native mesh partitions.
    native = np.array([[2, 2, 0, 3, 3]], dtype=np.uint8)
    support = np.ones((1, 5), dtype=bool)
    # When connecting only their rendered ink fringe.
    result = support_labels(native, support)
    # Then no existing native part label can be reassigned.
    assert np.array_equal(result[native != 0], native[native != 0])


def test_unsupported_island_cannot_borrow_distant_anatomy() -> None:
    # Given disjoint silhouettes with native evidence only in the first.
    native = np.array([[1, 0, 0, 0, 0]], dtype=np.uint8)
    support = np.array([[True, True, False, True, True]])
    # When extending support through at most three fringe pixels.
    result = support_labels(native, support, radius=3)
    # Then the separate island retains missing evidence.
    assert result.tolist() == [[1, 1, 0, 0, 0]]


def test_fringe_distance_is_bounded_even_on_connected_support() -> None:
    # Given a long silhouette with only one native labeled endpoint.
    native = np.array([[2, 0, 0, 0, 0, 0]], dtype=np.uint8)
    support = np.ones((1, 6), dtype=bool)
    # When using the anatomical one-pixel fringe allowance.
    result = support_labels(native, support, radius=1)
    # Then only the directly neighboring pixel receives that native label.
    assert result.tolist() == [[2, 2, 0, 0, 0, 0]]
