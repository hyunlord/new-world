# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pytest"]
# ///
# How to run: python -m pytest art/pipeline3/test_pawn_contract.py
"""Pawn checks use actual alpha, not anatomy declarations."""
from dataclasses import replace
from pathlib import Path
import numpy as np
from PIL import Image
from art.pipeline3.check_layer import check_layer
from art.pipeline3.layer_schema import Layer


def pawn(tmp_path: Path, bottom: int = 88) -> Layer:
    pixels = np.zeros((96, 64, 4), dtype=np.uint8)
    pixels[32:52, 21:43] = (140, 100, 70, 255)
    pixels[48:58, 28:36] = (140, 100, 70, 255)
    pixels[54:bottom, 19:45] = (140, 100, 70, 255)
    image, mask = tmp_path/'pawn.png', tmp_path/'mask.png'
    Image.fromarray(pixels).save(image)
    Image.fromarray((pixels[:, :, 3] > 0).astype(np.uint8)).save(mask)
    return Layer('pawn', image, mask, 'body', 'adult', 'S', 'idle', 1, (), template_version=2)


def test_pawn_has_no_hand_or_limb_requirement(tmp_path: Path) -> None:
    assert check_layer(pawn(tmp_path)).passed


def test_pawn_contact_rejects_lowest_opaque_pixel_outside_band(tmp_path: Path) -> None:
    assert 'contact_ok' in {f.rule for f in check_layer(pawn(tmp_path, 90)).findings}


def test_pawn_connectivity_rejects_detached_head(tmp_path: Path) -> None:
    layer = pawn(tmp_path)
    with Image.open(layer.image) as source:
        pixels = np.array(source)
    pixels[52:54] = 0
    Image.fromarray(pixels).save(layer.image)
    Image.fromarray((pixels[:, :, 3] > 0).astype(np.uint8)).save(layer.mask)
    assert 'connectivity' in {f.rule for f in check_layer(layer).findings}


def test_pawn_overlay_fit_rejects_detached_cloth(tmp_path: Path) -> None:
    from art.pipeline3.pawn_sets import fixed_overlays
    body = pawn(tmp_path)
    pixels = np.zeros((96, 64, 4), dtype=np.uint8)
    pixels[60:70, 50:55] = (100, 100, 100, 255)
    image, mask = tmp_path/'top.png', tmp_path/'top_mask.png'
    Image.fromarray(pixels).save(image)
    Image.fromarray((pixels[:, :, 3] > 0).astype(np.uint8)*3).save(mask)
    top = replace(body, id='top', kind='top', image=image, mask=mask)
    assert not next(result for result in fixed_overlays([body, top]) if result.id == 'top').passed


def test_pawn_item_contact_requires_two_overlap_pixels(tmp_path: Path) -> None:
    from art.pipeline3.pawn_sets import combinations
    body = pawn(tmp_path)
    empty = np.zeros((96, 64, 4), dtype=np.uint8)
    image, mask = tmp_path/'empty.png', tmp_path/'empty_mask.png'
    Image.fromarray(empty).save(image)
    Image.fromarray(empty[:, :, 3]).save(mask)
    top = replace(body, id='top', kind='top', image=image, mask=mask, variant='none')
    hair = replace(top, id='hair', kind='hair')
    pixels = empty.copy()
    pixels[60, 44:48] = (100, 100, 100, 255)
    item_image, item_mask = tmp_path/'item.png', tmp_path/'item_mask.png'
    Image.fromarray(pixels).save(item_image)
    Image.fromarray((pixels[:, :, 3] > 0).astype(np.uint8)*5).save(item_mask)
    item = replace(body, id='item', kind='item', variant='stone_axe', image=item_image, mask=item_mask, grip=(46, 60))
    assert all('item_contact' in {f.rule for f in result.findings} for result in combinations([body, top, hair, item]))


def test_pawn_inventory_cannot_pass_partial_set(tmp_path: Path) -> None:
    from art.pipeline3.check_set import inventory
    assert not inventory([pawn(tmp_path)], ['S', 'E'], 'P')[0].passed


def test_registration_never_wraps_clipped_pixels(tmp_path: Path) -> None:
    from art.pipeline3.pawn_sets import registered
    pixels = np.zeros((96, 64, 4), dtype=np.uint8)
    pixels[40, 63] = 255
    moved = registered(pixels, replace(pawn(tmp_path), frame='work_a'))
    assert not np.any(moved[:, :, 3])


def test_body_pose_cannot_change_after_registration(tmp_path: Path) -> None:
    from art.pipeline3.pawn_sets import fixed_overlays
    body = pawn(tmp_path)
    frame = replace(body, id='walk', frame='walk_a')
    assert any(f.rule == 'fixed_upper' for result in fixed_overlays([body, frame]) for f in result.findings)


def test_one_pixel_top_cannot_cover_upper_body(tmp_path: Path) -> None:
    body = pawn(tmp_path)
    pixels = np.zeros((96, 64, 4), dtype=np.uint8)
    pixels[60, 30] = 255
    image, mask = tmp_path/'tiny.png', tmp_path/'tiny_mask.png'
    Image.fromarray(pixels).save(image)
    Image.fromarray((pixels[:, :, 3] > 0).astype(np.uint8)*3).save(mask)
    top = replace(body, id='top', kind='top', variant='hide_wrap', image=image, mask=mask)
    assert 'coverage' in {finding.rule for finding in check_layer(top).findings}


def test_schema_rejects_out_of_range_pawn_look(tmp_path: Path) -> None:
    import json
    import pytest
    from art.pipeline3.layer_schema import load_layer
    path = tmp_path/'invalid.json'
    raw = dict(schema_version=1, template_version=2, id='bad', kind='body', body_type='adult', direction='S', frame='idle', scale=1, mask='mask.png', operations=[], skin_tone=4)
    path.write_text(json.dumps(raw))
    with pytest.raises(ValueError, match='skin_tone'):
        load_layer(path)


def test_schema_rejects_unknown_pawn_variant(tmp_path: Path) -> None:
    import json
    import pytest
    from art.pipeline3.layer_schema import load_layer
    path = tmp_path/'invalid.json'
    raw = dict(schema_version=1, template_version=2, id='bad', kind='top', variant='spacesuit', body_type='adult', direction='S', frame='idle', scale=1, mask='mask.png', operations=[])
    path.write_text(json.dumps(raw))
    with pytest.raises(ValueError, match='variant'):
        load_layer(path)


def test_legacy_schema_does_not_gain_carry_frame(tmp_path: Path) -> None:
    import json
    import pytest
    from art.pipeline3.layer_schema import load_layer
    path = tmp_path/'invalid.json'
    raw = dict(schema_version=1, id='bad', kind='body', body_type='adult', direction='S', frame='carry', scale=1, mask='mask.png', operations=[])
    path.write_text(json.dumps(raw))
    with pytest.raises(ValueError, match='frame'):
        load_layer(path)


def test_sampler_does_not_count_duplicate_combinations(tmp_path: Path) -> None:
    from art.pipeline3.pawn_sets import combinations
    body = pawn(tmp_path)
    overlays = [replace(body, id=kind, kind=kind, variant='none') for kind in ('top', 'hair', 'item')]
    samples = combinations([body, *overlays])
    assert len(samples) == 1


def test_pawn_body_requires_head_geometry(tmp_path: Path) -> None:
    layer = pawn(tmp_path)
    with Image.open(layer.image) as source:
        pixels = np.array(source)
    pixels[:54] = 0
    Image.fromarray(pixels).save(layer.image)
    Image.fromarray((pixels[:, :, 3] > 0).astype(np.uint8)).save(layer.mask)
    assert 'head_alignment' in {finding.rule for finding in check_layer(layer).findings}


def test_none_variant_cannot_hide_visible_pixels(tmp_path: Path) -> None:
    layer = replace(pawn(tmp_path), kind='top', variant='none')
    assert 'empty_variant' in {finding.rule for finding in check_layer(layer).findings}


def test_sample_covers_full_adult_attributes_and_states(tmp_path: Path) -> None:
    from itertools import product
    from art.pipeline3.pawn_sets import FRAMES, combinations
    base = pawn(tmp_path)
    bodies = [replace(base, id=f'{sex}_{skin}_{direction}_{state}', sex=sex, skin_tone=skin, direction=direction, frame=state)
              for sex, skin, direction, state in product(('M', 'F'), range(4), ('S', 'E', 'N'), FRAMES)]
    overlays = [replace(base, id=f'{kind}_{variant}_{color}_{direction}', kind=kind, variant=variant, direction=direction, hair_color=color if kind == 'hair' else 0, top_palette=color if kind == 'top' else 0)
                for kind, variants, colors in [('top', ('none', 'hide_wrap', 'fur_cloak', 'woven_tunic'), range(2)),
                                               ('hair', ('none', 'short', 'long_tied', 'bun', 'braid', 'grey_thin'), range(4))]
                for variant, color, direction in product(variants, colors, ('S', 'E', 'N'))]
    items = [replace(base, id=f'item_{variant}_{direction}_{state}', kind='item', variant=variant, direction=direction, frame=state)
             for variant, direction, state in product(('none', 'stone_axe', 'log_bundle', 'basket'), ('S', 'E', 'N'), ('idle', 'work_a', 'work_b'))]
    layers = {layer.id: layer for layer in [*bodies, *overlays, *items]}
    samples = combinations(list(layers.values()))
    assert len(samples) == len({sample.ids for sample in samples}) == 48
    chosen = [layers[sample.ids[0]] for sample in samples]
    assert {layer.frame for layer in chosen} == set(FRAMES)
    assert {layer.sex for layer in chosen} == {'M', 'F'}
    assert {layer.skin_tone for layer in chosen} == set(range(4))
    assert {layer.direction for layer in chosen} == {'S', 'E', 'N'}
    assert {layers[sample.ids[1]].variant for sample in samples} == {'none', 'hide_wrap', 'fur_cloak', 'woven_tunic'}
    assert {layers[sample.ids[2]].variant for sample in samples} == {'none', 'short', 'long_tied', 'bun', 'braid', 'grey_thin'}
    assert {layers[sample.ids[1]].top_palette for sample in samples} == {0, 1}
    assert {layers[sample.ids[2]].hair_color for sample in samples} == {0, 1, 2, 3}
    assert {layers[sample.ids[3]].variant for sample in samples} == {'none', 'stone_axe', 'log_bundle', 'basket'}


def test_top_occlusion_rejects_translucency_over_opaque_body(tmp_path: Path) -> None:
    from art.pipeline3.check_layer import read_pixels
    from art.pipeline3.pawn_sets import top_occlusion
    body = pawn(tmp_path)
    pixels, _ = read_pixels(body)
    top = pixels.copy()
    top[60, 32, 3] = 254
    assert top_occlusion(pixels, top, body)


def test_top_occlusion_accepts_matching_fractional_edge(tmp_path: Path) -> None:
    from art.pipeline3.check_layer import read_pixels
    from art.pipeline3.pawn_sets import top_occlusion
    body = pawn(tmp_path)
    pixels, _ = read_pixels(body)
    pixels[60, 19, 3] = 128
    assert not top_occlusion(pixels, pixels.copy(), body)


def test_sleep_head_can_cross_midline_when_center_is_left(tmp_path: Path) -> None:
    body = pawn(tmp_path)
    with Image.open(body.image) as source:
        pixels = np.rot90(np.array(source))
    image, mask = tmp_path/'sleep.png', tmp_path/'sleep_mask.png'
    Image.fromarray(pixels).save(image)
    Image.fromarray((pixels[:, :, 3] > 0).astype(np.uint8)).save(mask)
    sleep = replace(body, frame='sleep', image=image, mask=mask, head_box=(32, 21, 52, 43))
    assert check_layer(sleep).passed


def test_item_registration_reports_clipping(tmp_path: Path) -> None:
    from art.pipeline3.pawn_sets import place_prop
    pixels = np.zeros((96, 64, 4), dtype=np.uint8)
    pixels[60, 63] = 255
    body = replace(pawn(tmp_path), frame='work_a')
    item = replace(body, kind='item', variant='stone_axe')
    _, intact = place_prop(pixels, body, item)
    assert not intact


def test_carry_item_uses_shoulder_placement(tmp_path: Path) -> None:
    from art.pipeline3.pawn_sets import place_prop
    from art.pipeline4.pawn_spec import GRIPS, CARRY_GRIPS
    pixels = np.zeros((96, 64, 4), dtype=np.uint8)
    gx, gy = GRIPS['S']
    pixels[gy, gx] = 255
    body = replace(pawn(tmp_path), frame='carry')
    item = replace(body, kind='item', variant='log_bundle')
    placed, intact = place_prop(pixels, body, item)
    cx, cy = CARRY_GRIPS['S']
    assert intact and placed[cy, cx, 3] == 255
    assert placed[gy, gx, 3] == 0


def test_pawn_checker_rejects_authoring_scale_outputs(tmp_path: Path) -> None:
    result = check_layer(replace(pawn(tmp_path), scale=4))
    assert not result.passed and result.findings[0].rule == 'scale'
