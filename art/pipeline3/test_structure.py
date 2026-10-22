# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pytest"]
# ///
# How to run: python -m pytest art/pipeline3/test_structure.py
"""Synthetic geometry only. These fixtures are never production character art."""
from dataclasses import replace
from pathlib import Path
import numpy as np
from PIL import Image
from art.pipeline3.check_layer import check_layer
from art.pipeline3.layer_schema import Layer
from art.pipeline3.set_checks import combinations, fixed_upper
from art.pipeline3.check_set import inventory


def body(tmp_path: Path, name: str = 'body', scale: int = 1) -> Layer:
    pixels = np.zeros((96,64,4),dtype=np.uint8)
    pixels[32:50,20:44] = (154,154,154,255)
    pixels[50:70,18:46] = (154,154,154,255)
    pixels[52:67,14:20] = (154,154,154,255)
    pixels[52:67,44:51] = (154,154,154,255)
    pixels[70:88,20:29] = (154,154,154,255)
    pixels[70:88,35:44] = (154,154,154,255)
    pixels = np.repeat(np.repeat(pixels,scale,axis=0),scale,axis=1)
    image,mask = tmp_path/f'{name}.png',tmp_path/f'{name}_mask.png'
    Image.fromarray(pixels).save(image)
    Image.fromarray((pixels[:,:,3]>0).astype(np.uint8)).save(mask)
    return Layer(name,image,mask,'body','adult','S','idle',scale,('synthetic test only',))


def mutate(layer: Layer, box: tuple[int,int,int,int], color: tuple[int,int,int,int]) -> None:
    with Image.open(layer.image) as source:
        pixels = np.array(source)
    x0,y0,x1,y1 = box
    pixels[y0:y1,x0:x1] = color
    Image.fromarray(pixels).save(layer.image)
    Image.fromarray((pixels[:,:,3]>0).astype(np.uint8)).save(layer.mask)


def rules(layer: Layer) -> set[str]:
    return {f.rule for f in check_layer(layer).findings}


def test_good_layer_and_open_leg_gap(tmp_path: Path) -> None:
    assert check_layer(body(tmp_path)).passed


def test_hollow_torso(tmp_path: Path) -> None:
    layer = body(tmp_path)
    mutate(layer,(25,55,35,60),(0,0,0,0))
    assert 'holes' in rules(layer)


def test_floating_arm(tmp_path: Path) -> None:
    layer = body(tmp_path)
    mutate(layer,(53,53,57,66),(154,154,154,255))
    assert 'connectivity' in rules(layer)


def test_shifted_foot_line(tmp_path: Path) -> None:
    layer = body(tmp_path)
    mutate(layer,(20,88,29,92),(154,154,154,255))
    assert 'feet' in rules(layer)


def test_mask_gap(tmp_path: Path) -> None:
    layer = body(tmp_path)
    with Image.open(layer.mask) as source:
        mask = np.array(source)
    mask[60,30] = 0
    Image.fromarray(mask).save(layer.mask)
    assert 'mask_completeness' in rules(layer)


def test_hole_area_scales_in_game_pixels(tmp_path: Path) -> None:
    layer = body(tmp_path,scale=4)
    assert check_layer(layer).passed
    mutate(layer,(100,220,112,236),(0,0,0,0)) #12gamepx2 is allowed
    assert 'holes' not in rules(layer)
    mutate(layer,(100,220,116,236),(0,0,0,0)) #16gamepx2 fails
    assert 'holes' in rules(layer)


def test_fixed_upper_color_and_alpha(tmp_path: Path) -> None:
    idle = body(tmp_path,'idle')
    frame = replace(body(tmp_path,'walk'),frame='walk_a')
    assert all(r.passed for r in fixed_upper([idle,frame]))
    mutate(frame,(22,35,24,37),(180,180,180,255))
    assert not fixed_upper([idle,frame])[-1].passed
    mutate(frame,(22,35,24,37),(154,154,154,254))
    assert not fixed_upper([idle,frame])[-1].passed


def overlay(tmp_path: Path, name: str, kind: str) -> Layer:
    pixels = np.zeros((96,64,4),dtype=np.uint8)
    region = 2
    if kind == 'top':
        pixels[50:70,18:46] = (192,192,192,255)
        region = 3
    elif kind == 'hair':
        pixels[32:36,20:44] = (60,60,60,255)
    else:
        pixels[63:67,48:51] = (100,100,100,255)
        region = 5
    image,mask = tmp_path/f'{name}.png',tmp_path/f'{name}_mask.png'
    Image.fromarray(pixels).save(image)
    Image.fromarray((pixels[:,:,3]>0).astype(np.uint8)*region).save(mask)
    return Layer(name,image,mask,kind,'adult','S','idle',1,('synthetic test only',),grip=(50,66),shoulder=(50,66))


def test_top_coverage_and_extent(tmp_path: Path) -> None:
    top = overlay(tmp_path,'top','top')
    assert check_layer(top).passed
    with Image.open(top.image) as source:
        pixels = np.array(source)
    pixels[50:60,18:30] = 0
    Image.fromarray(pixels).save(top.image)
    assert 'coverage' in rules(top)
    pixels[5,5] = (192,192,192,255)
    Image.fromarray(pixels).save(top.image)
    assert 'overlay_extent' in rules(top)


def test_combinations_exhaustive_including_simultaneous_props(tmp_path: Path) -> None:
    layers = [body(tmp_path),replace(body(tmp_path,'walk'),frame='walk_a')]
    layers += [overlay(tmp_path,'top1','top'),overlay(tmp_path,'top2','top')]
    layers += [overlay(tmp_path,'hair1','hair'),overlay(tmp_path,'hair2','hair')]
    layers += [overlay(tmp_path,'axe','item'),overlay(tmp_path,'logs','carry')]
    results = combinations(layers)
    assert len(results) == 2*2*2*2*2
    assert all(result.passed for result in results)
    assert sum('axe' in r.ids and 'logs' in r.ids for r in results) == 8


def test_body_cannot_show_through_translucent_top(tmp_path: Path) -> None:
    b = body(tmp_path)
    top = overlay(tmp_path,'top','top')
    hair = overlay(tmp_path,'hair','hair')
    with Image.open(top.image) as source:
        pixels = np.array(source)
    pixels[60,30,3] = 254
    Image.fromarray(pixels).save(top.image)
    assert any(f.rule == 'body_through_top' for f in combinations([b,top,hair])[0].findings)


def test_invalid_grip_and_shift_clipping(tmp_path: Path) -> None:
    item = overlay(tmp_path,'axe','item')
    assert 'anchor' in rules(replace(item,grip=(0,0)))
    b = body(tmp_path)
    top,hair = overlay(tmp_path,'top','top'),overlay(tmp_path,'hair','hair')
    bad = replace(item,grip=(-20,-20))
    assert any(f.rule == 'clipping' for r in combinations([b,top,hair,bad]) for f in r.findings)


def test_empty_inventory_rejected() -> None:
    assert not inventory([],['S','E','N'])[0].passed


def test_hands_require_actual_skin_at_declared_anchor(tmp_path: Path) -> None:
    b = body(tmp_path)
    mutate(b,(14,62,18,68),(0,0,0,0))
    assert 'hand_alignment' in rules(replace(b,hands=((14,66),(50,66))))


def test_union_hole_created_by_prop_is_rejected(tmp_path: Path) -> None:
    b,top,hair = body(tmp_path),overlay(tmp_path,'top','top'),overlay(tmp_path,'hair','hair')
    item = overlay(tmp_path,'bridge','item')
    with Image.open(item.image) as source:
        pixels = np.zeros_like(np.array(source))
    pixels[85:88,20:44] = (100,100,100,255)
    Image.fromarray(pixels).save(item.image)
    item = replace(item,grip=(50,66)) #zero placement translation, deliberately diagnostic
    assert any(f.rule == 'union_holes' for r in combinations([b,top,hair,item]) for f in r.findings)


def test_report_and_failure_board_preserve_rejection(tmp_path: Path) -> None:
    import json
    from art.pipeline3.check_set import run
    layer = body(tmp_path)
    mutate(layer,(25,55,35,60),(0,0,0,0))
    sidecar = tmp_path/'body.json'
    sidecar.write_text(json.dumps({'schema_version':1,'id':'body','kind':'body','body_type':'adult',
                                  'direction':'S','frame':'idle','scale':1,'mask':'body_mask.png',
                                  'operations':['synthetic test only']}))
    manifest = tmp_path/'manifest.json'
    manifest.write_text(json.dumps({'route':'SYNTHETIC','directions':['S','E'],
                                   'layers':['body.json'],'require_minimum':False}))
    output = tmp_path/'report'
    assert run(manifest,output) is False
    report = json.loads((output/'CHECKS.json').read_text())
    assert report['layer_count'] == 1 and report['layer_pass_count'] == 0
    assert report['passed'] is False
    assert str(layer.image.resolve()) in report['hashes']
    with Image.open(output/'failures.png') as board:
        assert board.size == (1024,460)


def test_wrong_canvas_and_empty_body_rejected(tmp_path: Path) -> None:
    b = body(tmp_path)
    Image.new('RGBA',(32,48)).save(b.image)
    assert 'canvas' in rules(b)
    Image.new('RGBA',(64,96)).save(b.image)
    Image.new('L',(64,96)).save(b.mask)
    assert 'empty' in rules(b)


def test_shifted_head_and_wrong_hair_region_rejected(tmp_path: Path) -> None:
    b = body(tmp_path)
    mutate(b,(20,32,44,38),(0,0,0,0))
    assert 'head_alignment' in rules(b)
    hair = overlay(tmp_path,'hair','hair')
    with Image.open(hair.mask) as source:
        mask = np.array(source)
    mask[mask == 2] = 5
    Image.fromarray(mask).save(hair.mask)
    assert 'mask_semantics' in rules(hair)
