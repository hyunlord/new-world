# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pytest"]
# ///
# How to run: python -m pytest art/pipeline3/test_anatomy.py
"""Pixel evidence tests for the owner-approved contact/proportion amendment."""
from dataclasses import replace
from pathlib import Path
import numpy as np
from PIL import Image
from art.pipeline3.anatomy import anatomy_findings
from art.pipeline3.layer_schema import Layer
from art.pipeline3.structure import Pixels


def fixture(tmp_path: Path) -> tuple[Layer, Pixels]:
    """Given: visible, separated extremities connected through a torso."""
    labels = np.zeros((96, 64), dtype=np.uint8)
    labels[32:50, 20:44] = 1
    labels[50:70, 18:46] = 10
    labels[51:62, 12:18] = 6
    labels[51:62, 46:52] = 7
    labels[62:68, 11:18] = 2
    labels[62:68, 46:53] = 3
    labels[70:84, 20:27] = 8
    labels[70:84, 37:44] = 9
    labels[84:88, 18:28] = 4
    labels[84:88, 36:46] = 5
    anatomy = tmp_path / 'anatomy.png'
    Image.fromarray(labels).save(anatomy)
    alpha = np.where(labels > 0, 255, 0).astype(np.uint8)
    layer = Layer('test', tmp_path/'image.png', tmp_path/'mask.png', 'body', 'adult', 'S', 'idle', 1, (), anatomical_mask=anatomy)
    return layer, alpha


def test_good_body(tmp_path: Path) -> None:
    layer, alpha = fixture(tmp_path)
    result = anatomy_findings(layer, alpha)
    assert result == ()


def test_four_pixel_hover(tmp_path: Path) -> None:
    layer, alpha = fixture(tmp_path)
    result = anatomy_findings(layer, np.roll(alpha, -4, axis=0))
    assert any(rule == 'contact_ok' for rule, _ in result)


def test_missing_measurements_fail_closed(tmp_path: Path) -> None:
    layer, alpha = fixture(tmp_path)
    alpha[62:68, 9:20] = 0
    result = anatomy_findings(replace(layer, anatomical_mask=None), alpha)
    assert any(rule == 'limb_ok' for rule, _ in result)


def test_narrow_hand_and_foot(tmp_path: Path) -> None:
    layer, alpha = fixture(tmp_path)
    alpha[62:68, 11:15] = 0
    alpha[84:88, 18:22] = 0
    result = anatomy_findings(layer, alpha)
    assert any(rule == 'limb_ok' and 'hand' in detail for rule, detail in result)
    assert any(rule == 'limb_ok' and 'foot' in detail for rule, detail in result)


def test_thin_limb(tmp_path: Path) -> None:
    layer, alpha = fixture(tmp_path)
    alpha[54:59, 12:16] = 0
    result = anatomy_findings(layer, alpha)
    assert any(rule == 'limb_ok' and 'limb' in detail for rule, detail in result)


def test_small_head(tmp_path: Path) -> None:
    layer, alpha = fixture(tmp_path)
    alpha[32:50, 20:33] = 0
    result = anatomy_findings(layer, alpha)
    assert any(rule == 'limb_ok' and 'head' in detail for rule, detail in result)


def test_contact_cannot_bridge_gap_between_single_pixels(tmp_path: Path) -> None:
    layer, alpha = fixture(tmp_path)
    alpha[82:] = 0
    alpha[87, 22] = 255
    alpha[87, 42] = 255
    result = anatomy_findings(layer, alpha)
    assert any(rule == 'contact_ok' and 'overlap' in detail for rule, detail in result)


def test_sleep_missing_anatomical_measurements_fails(tmp_path: Path) -> None:
    layer, alpha = fixture(tmp_path)
    result = anatomy_findings(replace(layer, frame='sleep', anatomical_mask=None, head_box=(20,32,44,50)), alpha)
    assert any(rule == 'limb_ok' and 'NOT_VERIFIED' in detail for rule, detail in result)


def test_disconnected_anatomical_head_fails(tmp_path: Path) -> None:
    layer, alpha = fixture(tmp_path)
    assert layer.anatomical_mask is not None
    with Image.open(layer.anatomical_mask) as source:
        labels = np.array(source)
    labels[32:50, 30:34] = 10
    Image.fromarray(labels).save(layer.anatomical_mask)
    result = anatomy_findings(layer, alpha)
    assert any(rule == 'limb_ok' and 'head' in detail for rule, detail in result)
