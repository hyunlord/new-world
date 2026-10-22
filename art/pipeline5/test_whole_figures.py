# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pytest"]
# ///
# How to run: python -m pytest art/pipeline5/test_whole_figures.py
"""Boundary tests prevent silent repair, count shortcuts and per-pose scaling."""
import json
from pathlib import Path

import numpy as np
import pytest
from PIL import Image, ImageDraw

from art.pipeline5.attempt_integrity import JudgeBinding, digest
from art.pipeline5.sheet_geometry import normalize, split_panels
from art.pipeline5.sheet_slice import SheetRequest, process
from art.pipeline5.whole_checks import check_whole


def fixture_sheet() -> Image.Image:
    image = Image.new('RGBA', (1536, 1024))
    draw = ImageDraw.Draw(image)
    for index in range(4):
        x = 100+index*300
        draw.rectangle((x, 300-index*20, x+119, 799), fill=(100, 70, 40, 255))
    draw.rectangle((1250, 700, 1489, 799), fill=(100, 70, 40, 255))
    return image


def test_five_components_preserve_every_visible_pixel() -> None:
    source = fixture_sheet()
    panels = split_panels(source)
    assert panels.original_components == 5
    assert len(panels.images) == 5
    assert sum(np.asarray(panel)[:, :, 3].sum() for panel in panels.images) == np.asarray(source)[:, :, 3].sum()


def test_touching_panels_are_split_with_logged_cut_and_no_pixel_loss() -> None:
    source = fixture_sheet()
    ImageDraw.Draw(source).line((219, 600, 400, 600), fill=(100, 70, 40, 255))
    panels = split_panels(source)
    assert panels.original_components == 4
    assert len(panels.split_columns) == 1
    assert sum(np.asarray(panel)[:, :, 3].sum() for panel in panels.images) == np.asarray(source)[:, :, 3].sum()


def test_six_substantial_components_rejected() -> None:
    source = fixture_sheet()
    ImageDraw.Draw(source).rectangle((4, 4, 23, 23), fill=(0, 0, 0, 255))
    with pytest.raises(ValueError, match='panel_count'):
        split_panels(source)


def test_uniform_scale_preserves_pose_size_difference_and_contact() -> None:
    panels = split_panels(fixture_sheet())
    images = [normalize(panel, 56/500, i == 4) for i, panel in enumerate(panels.images)]
    heights = []
    for image in images:
        y, _ = np.nonzero(np.asarray(image)[:, :, 3] >= 128)
        heights.append(int(y.max()-y.min()+1))
    assert heights[0] == 56
    assert heights[3] > heights[0]
    assert all(check_whole(image, 'adult', frame).contact_ok for image, frame in zip(images, ('idle','walk_a','work_a','carry','sleep'), strict=True))


def test_hole_is_rejected_but_open_gap_is_not_hole() -> None:
    image = Image.new('RGBA', (64, 96))
    draw = ImageDraw.Draw(image)
    draw.rectangle((20, 33, 43, 88), fill=(100, 70, 40, 255))
    assert check_whole(image, 'adult', 'idle').passed
    draw.rectangle((28, 50, 31, 53), fill=(0, 0, 0, 0))
    assert check_whole(image, 'adult', 'idle').largest_hole == 16
    draw.rectangle((28, 50, 31, 88), fill=(0, 0, 0, 0))
    assert check_whole(image, 'adult', 'idle').largest_hole == 0


def test_translucent_border_and_detached_opaque_island_fail() -> None:
    image = Image.new('RGBA', (64, 96))
    ImageDraw.Draw(image).rectangle((20, 33, 43, 88), fill=(100, 70, 40, 255))
    image.putpixel((0, 40), (0, 0, 0, 1))
    assert any('alpha_edge' in rule for rule in check_whole(image, 'adult', 'idle').findings)
    image.putpixel((4, 40), (0, 0, 0, 128))
    assert check_whole(image, 'adult', 'idle').component_count == 2


def test_oversize_is_rejected_instead_of_rescaled_or_clipped() -> None:
    with pytest.raises(ValueError, match='clip'):
        normalize(Image.new('RGBA', (200, 200), (1, 2, 3, 255)), 1, False)


def test_rejected_sheet_keeps_diagnostics_without_delivery(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr("art.pipeline5.sheet_slice.verify_judge", fixture_binding)
    raw = tmp_path/'raw.png'
    fixture_sheet().save(raw)
    prompt = tmp_path/'prompt.txt'
    prompt.write_text('fixture')
    judge = tmp_path/'judge.json'
    judge.write_text(json.dumps({'numeric_pass': False}))
    request = SheetRequest(raw, 'adult_M_hide_wrap_short', 'adult', 'S', 1, prompt, judge,
                           tmp_path/'out', tmp_path/'diagnostics')
    assert not process(request)
    assert not request.output_root.exists()
    assert (request.diagnostics_root/request.look/'S/try1/sheet_checks.json').is_file()
    with pytest.raises(FileExistsError):
        process(request)


def test_all_pass_sheet_delivers_five_hashed_sidecars(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr("art.pipeline5.sheet_slice.verify_judge", fixture_binding)
    from art.pipeline5.sheet_slice import digest
    raw = tmp_path/'raw.png'
    fixture_sheet().save(raw)
    prompt = tmp_path/'prompt.txt'
    prompt.write_text('fixture')
    judge = tmp_path/'judge.json'
    judge.write_text(json.dumps({'numeric_pass': True}))
    request = SheetRequest(raw, 'adult_M_hide_wrap_short', 'adult', 'S', 1, prompt, judge,
                           tmp_path/'out', tmp_path/'diagnostics')
    assert process(request)
    output = request.output_root/request.look/'S'
    assert len(list(output.glob('*.png'))) == 5
    for sidecar in output.glob('*.json'):
        data = json.loads(sidecar.read_text())
        assert data['checks']['passed']
        assert data['image_sha256'] == digest(sidecar.with_suffix('.png'))
        assert data['scale'] == 56/500


def test_alpha254_tool_format_uses_same_structural_contact_threshold() -> None:
    panel = Image.new('RGBA', (120, 500), (100, 70, 40, 254))
    image = normalize(panel, 56/500, False)
    assert check_whole(image, 'adult', 'idle').passed
    assert np.asarray(image)[:, :, 3].max() == 254


def test_dust_is_assigned_without_losing_rgba_or_changing_idle_height() -> None:
    source = fixture_sheet()
    source.putpixel((110, 280), (7, 8, 9, 254))
    source.putpixel((111, 280), (10, 11, 12, 254))
    panels = split_panels(source)
    assert panels.original_components == 6
    assert panels.original_major_components == 5
    assert panels.idle_height == 500
    assert panels.dust_area_limit == 1536*1024*0.0001
    assert panels.attached_fragments[0].area == 2
    assert panels.attached_fragments[0].panel_indices == (0,)
    rebuilt = Image.new('RGBA', source.size)
    for panel, box in zip(panels.images, panels.boxes, strict=True):
        rebuilt.paste(panel, box[:2])
    assert np.array_equal(np.asarray(source), np.asarray(rebuilt))
    normalized = normalize(panels.images[0], 56/panels.idle_height, False)
    assert check_whole(normalized, 'adult', 'idle').contact_ok


def test_large_detached_prop_inside_panel_is_not_dust() -> None:
    source = fixture_sheet()
    ImageDraw.Draw(source).rectangle((130, 100, 149, 119), fill=(100, 70, 40, 255))
    with pytest.raises(ValueError, match='panel_count'):
        split_panels(source)


def test_dust_threshold_is_strictly_less_than_area_limit() -> None:
    source = fixture_sheet()
    ImageDraw.Draw(source).rectangle((10, 100, 166, 100), fill=(100, 70, 40, 255))
    assert split_panels(source).attached_fragments[0].area == 157
    source.putpixel((167, 100), (100, 70, 40, 255))
    with pytest.raises(ValueError, match='panel_count'):
        split_panels(source)


def test_overlapping_bboxes_preserve_all_visible_pixels_exactly_once() -> None:
    source = fixture_sheet()
    draw = ImageDraw.Draw(source)
    draw.rectangle((100, 150, 119, 300), fill=(90, 50, 20, 254))
    draw.rectangle((100, 150, 430, 169), fill=(90, 50, 20, 254))
    draw.rectangle((280, 200, 300, 210), fill=(10, 20, 30, 40))
    panels = split_panels(source)
    assert panels.boxes[0][2] > panels.boxes[1][0]
    rgba = np.asarray(source)
    rebuilt = np.zeros_like(rgba)
    assigned = np.zeros(rgba.shape[:2], dtype=int)
    for panel, box in zip(panels.images, panels.boxes, strict=True):
        x0, y0, x1, y1 = box
        pixels = np.asarray(panel)
        visible = pixels[:, :, 3] > 0
        rebuilt[y0:y1, x0:x1][visible] = pixels[visible]
        assigned[y0:y1, x0:x1] += visible
    assert np.array_equal(rebuilt, rgba)
    assert np.all(assigned[rgba[:, :, 3] > 0] == 1)
    assert panels.ownership.reconstructed_exactly
    assert panels.ownership.foreign_major_pixels == 0
    assert split_panels(source).ownership.ownership_sha256 == panels.ownership.ownership_sha256


def test_distance_ties_choose_leftmost_and_dust_stays_intact() -> None:
    from art.pipeline5.panel_ownership import extract_owned
    image = Image.new('RGBA', (22, 8))
    groups = [[(x, 2)] for x in (2, 6, 10, 14, 18)]
    for group in groups:
        image.putpixel(group[0], (100, 70, 40, 254))
    image.putpixel((4, 2), (10, 20, 30, 1))
    dust = [[(4, 5), (5, 5)]]
    for point in dust[0]:
        image.putpixel(point, (50, 60, 70, 254))
    owned = extract_owned(image, groups, dust)
    assert owned.evidence.tied_visible_pixels == 2
    assert owned.evidence.fragments[0].panel_indices == (1,)
    x0, y0, _, _ = owned.boxes[0]
    assert owned.images[0].getpixel((4-x0, 2-y0)) == (10, 20, 30, 1)
    assert owned.evidence.reconstructed_exactly


def test_post_resize_zero_alpha_padding_is_trimmed_without_visible_loss() -> None:
    from art.pipeline5.sheet_geometry import normalize_panel
    source = Image.new('RGBA', (200, 1000))
    ImageDraw.Draw(source).rectangle((40, 400, 159, 899), fill=(100, 70, 40, 254))
    source.putpixel((90, 0), (10, 20, 30, 1))
    scale = 56/500
    resized = source.resize((round(200*scale), round(1000*scale)), Image.Resampling.LANCZOS)
    result = normalize_panel(source, scale, False)
    assert result.transform.trim_box[1] > 0
    dx, dy = result.transform.resized_to_canvas_offset
    src = np.asarray(resized)
    output = np.asarray(result.image)
    ys, xs = np.nonzero(src[:, :, 3] > 0)
    assert np.array_equal(src[ys, xs], output[ys+dy, xs+dx])
    assert np.count_nonzero(output[:, :, 3]) == len(xs)
    assert check_whole(result.image, 'adult', 'idle').contact_ok


def test_visible_low_alpha_tail_is_never_trimmed_to_fit() -> None:
    source = Image.new('RGBA', (64, 120))
    ImageDraw.Draw(source).rectangle((20, 30, 43, 85), fill=(100, 70, 40, 254))
    source.putpixel((32, 119), (10, 20, 30, 1))
    with pytest.raises(ValueError, match='visible extent'):
        normalize(source, 1, False)


def fixture_binding(path: Path, raw: Path) -> JudgeBinding:
    """Synthetic geometry tests substitute only the independent numeric boundary."""
    return JudgeBinding(digest(raw), digest(path), 'synthetic-calibration', json.loads(path.read_text())['numeric_pass'])
