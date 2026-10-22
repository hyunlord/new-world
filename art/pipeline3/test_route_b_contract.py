# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pytest"]
# ///
# How to run: python -m pytest art/pipeline3/test_route_b_contract.py
"""Explicit route B scope never weakens historical route A acceptance."""
from dataclasses import replace
from pathlib import Path
from art.pipeline3.check_set import inventory
from art.pipeline3.set_checks import fixed_overlays, fixed_upper, combinations
from art.pipeline3.test_structure import body, overlay, mutate
from art.pipeline3.structure import FRAMES
from art.pipeline3.layer_schema import Layer


def adult_inventory(tmp_path: Path) -> list[Layer]:
    base = body(tmp_path)
    layers = [replace(base,id=f'{direction}_{frame}',direction=direction,frame=frame)
              for direction in ('S','E') for frame in FRAMES]
    layers.append(replace(base,id='sleep',frame='sleep'))
    for direction in ('S','E'):
        for kind,variants in (('top',('hide_wrap','fur_cloak')),('hair',('short','long_tied'))):
            for variant in variants:
                layers.append(replace(base,id=f'{direction}_{variant}',kind=kind,direction=direction,variant=variant))
    layers += [replace(base,id='axe',kind='item',variant='stone_axe'),replace(base,id='logs',kind='carry',variant='log_bundle')]
    return layers


def test_b_inventory_accepts_adult_but_a_requires_derivatives(tmp_path: Path) -> None:
    layers = adult_inventory(tmp_path)
    result = inventory(layers,['S','E'],route='B')
    assert all(r.passed for r in result)
    assert not all(r.passed for r in inventory(layers,['S','E']))


def test_unrecognized_route_cannot_change_inventory(tmp_path: Path) -> None:
    result = inventory(adult_inventory(tmp_path),['S','E'],route='skip')
    assert not all(r.passed for r in result)


def test_b_overlays_must_be_idle(tmp_path: Path) -> None:
    top = replace(overlay(tmp_path,'top','top'),frame='walk_a')
    result = fixed_overlays([top])
    assert not all(r.passed for r in result)


def test_b_duplicate_overlay_variant_rejected(tmp_path: Path) -> None:
    top = overlay(tmp_path,'top','top')
    result = fixed_overlays([top,replace(top,id='duplicate')])
    assert not all(r.passed for r in result)


def test_body_animation_keeps_every_frame_composition(tmp_path: Path) -> None:
    idle = body(tmp_path)
    walk = replace(body(tmp_path,'walk'),frame='walk_a')
    mutate(walk,(22,35,24,37),(180,180,180,255))
    layers = [idle,walk,overlay(tmp_path,'top','top'),overlay(tmp_path,'hair','hair')]
    result = combinations(layers)
    assert {r.ids[0] for r in result} == {idle.id,walk.id}
    assert all(r.passed for r in fixed_overlays(layers))
    assert not all(r.passed for r in fixed_upper(layers))


def test_manifest_routes_body_lock_without_generic_bypass(tmp_path: Path) -> None:
    import json
    from art.pipeline3.check_set import run
    idle = body(tmp_path)
    walk = replace(body(tmp_path,'walk'),frame='walk_a')
    mutate(walk,(22,35,24,37),(180,180,180,255))
    layers = [idle,walk,overlay(tmp_path,'top','top'),overlay(tmp_path,'hair','hair')]
    sidecars = []
    for layer in layers:
        path = tmp_path/f'{layer.id}.json'
        path.write_text(json.dumps({'schema_version':1,'id':layer.id,'kind':layer.kind,
            'body_type':'adult','direction':'S','frame':layer.frame,'scale':1,
            'mask':layer.mask.name,'operations':['synthetic only']}))
        sidecars.append(path.name)
    observed = {}
    for route in ('A','B','skip'):
        manifest = tmp_path/f'{route}.json'
        manifest.write_text(json.dumps({'route':route,'directions':['S','E'],
                                       'layers':sidecars,'require_minimum':False}))
        output = tmp_path/route
        run(manifest,output)
        observed[route] = json.loads((output/'CHECKS.json').read_text())
    assert observed['B']['subset_passed'] is True
    assert observed['B']['combination_count'] == 2
    assert observed['B']['passed'] is False
    assert observed['A']['subset_passed'] is False
    assert observed['skip']['subset_passed'] is False


def test_b_props_match_direction_while_a_keeps_historical_scope(tmp_path: Path) -> None:
    layers = [body(tmp_path),overlay(tmp_path,'top','top'),overlay(tmp_path,'hair','hair'),
              overlay(tmp_path,'axe_s','item'),replace(overlay(tmp_path,'axe_e','item'),direction='E')]
    result = combinations(layers,route='B')
    assert any('axe_s' in row.ids for row in result)
    assert not any('axe_e' in row.ids for row in result)
    assert any('axe_e' in row.ids for row in combinations(layers))


def test_report_hash_tracks_anatomical_input_bytes(tmp_path: Path) -> None:
    import json
    import hashlib
    from PIL import Image
    from art.pipeline3.check_set import run
    layer = body(tmp_path)
    anatomy = tmp_path/'anatomy.png'
    Image.new('L',(64,96),0).save(anatomy)
    (tmp_path/'body.json').write_text(json.dumps({'schema_version':1,'id':'body','kind':'body',
        'body_type':'adult','direction':'S','frame':'idle','scale':1,'mask':layer.mask.name,
        'operations':['synthetic only'],'anatomical_mask':anatomy.name}))
    manifest = tmp_path/'manifest.json'
    manifest.write_text(json.dumps({'route':'B','directions':['S','E'],'layers':['body.json']}))
    run(manifest,tmp_path/'before')
    before = json.loads((tmp_path/'before/CHECKS.json').read_text())
    Image.new('L',(64,96),1).save(anatomy)
    run(manifest,tmp_path/'after')
    after = json.loads((tmp_path/'after/CHECKS.json').read_text())
    key = str(anatomy.resolve())
    assert before['input_hashes'][key] != after['input_hashes'][key]
    assert after['input_hashes'][key] == hashlib.sha256(anatomy.read_bytes()).hexdigest()
