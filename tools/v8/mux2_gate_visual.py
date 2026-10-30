#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: imported by mux2_gate_receipts.py.
"""Validate generated scene provenance and complete visual artifact coverage."""
from pathlib import Path
import re
from mux2_gate_inputs import BRIDGE, Node, check_hash, digest, read_json, require


def validate_visibility(root: Path, out: Path) -> None:
    manifest_path = out / 'saves/manifest.json'
    manifest = read_json(manifest_path)
    manifest.field('schema').equals(1)
    require(manifest.field('seed').number() == 0, 'wrong visibility seed')
    manifest.field('steward').equals('on')
    manifest.field('bridge_sha256').equals(digest(root / BRIDGE))
    manifest.field('script_sha256').equals(digest(root / 'tools/v8/mux2_visibility_saves.gd'))
    expected = {f'day{day}_{phase}' for day in (1, 20) for phase in ('noon', 'night')}
    saves = manifest.field('scenes').array()
    require(len(saves) == 4 and {r.field('id').text() for r in saves} == expected,
            'four unique current-policy saves required')
    for row in saves:
        day = row.field('day').number()
        hour = row.field('hour').number()
        require(day in (1, 20) and hour in (12, 22), 'incorrect saved clock')
        require(row.field('id').text() == f'day{int(day)}_{"noon" if hour == 12 else "night"}',
                'saved id and clock disagree')
        check_hash(Path(row.field('save').text()), row.field('sha256').text())
    report = read_json(out / 'run/visibility.json')
    report.field('schema').equals(2)
    report.field('pass').equals(True)
    report.field('failures').equals([])
    report.field('inputs_unchanged').equals(True)
    report.field('changed_inputs').equals([])
    report.field('manifest_sha256').equals(digest(manifest_path))
    report.field('bridge_sha256').equals(digest(root / BRIDGE))
    scenes = report.field('scenes').array()
    expected_scenes = {f'{name}_{scale}' for name in expected for scale in ('half', 'quarter')}
    require(len(scenes) == 8 and {r.field('id').text() for r in scenes} == expected_scenes,
            'eight unique GPU scenes required')
    for scene in scenes:
        validate_scene(scene, out / 'run')


def validate_scene(scene: Node, output: Path) -> None:
    name = scene.field('id').text()
    half = name.endswith('_half')
    require(scene.field('zoom').number() == (0.5 if half else 0.25), 'wrong scene zoom')
    scene.field('lod').equals(not half)
    scene.field('viewport').equals([1920, 1080])
    scene.field('pass').equals(True)
    on = [n.number() for n in scene.field('on_screen_ids').array()]
    off = [n.number() for n in scene.field('off_screen_ids').array()]
    all_ids = [n.number() for n in scene.field('all_ids').array()]
    # R2: partly clipped people are recorded, not judged (scenes before R2 carry no list).
    partial = ([int(n.field('entity_bits').number()) for n in scene.field('partial').array()]
               if 'partial' in scene.mapping() else [])
    require(bool(on) and len(set(on + off + partial)) == len(on + off + partial)
            and sorted(on + off + partial) == sorted(all_ids), 'missing/duplicate person coverage')
    rows = scene.field('rows').array()
    require(len(rows) == len(on) and sorted(r.field('entity_bits').number() for r in rows) == sorted(on),
            'missing/duplicate attributed person')
    directory = output / name
    for key, filename in [('full_sha256', 'full.png'), ('ground_sha256', 'ground.png')]:
        check_hash(directory / filename, scene.field(key).text())
    for row in rows:
        row.field('pass').equals(True)
        row.field('valid').equals(True)
        minimum = 60 if half else 140
        require(row.field('required_pixels').number() == minimum
                and row.field('qualifying_pixels').number() >= minimum, 'visibility bar failed')
        bits = int(row.field('entity_bits').number())
        for key, prefix in [('full_sha256', 'person'), ('removed_sha256', 'without'),
                            ('restored_sha256', 'restored')]:
            check_hash(directory / f'{prefix}-{bits}.png', row.field(key).text())


def validate_why(log: Path) -> list[Path]:
    text = log.read_text()
    require('SCRIPT ERROR' not in text and 'MUX2_WHY_FAIL' not in text, 'why runtime error')
    require(re.findall(r'^MUX2_WHY_DONE\b[^\n]*$', text, re.M)
            == ['MUX2_WHY_DONE failures=0 checks=42'],
            'why test requires exactly 42 completed checks')
    require(len(re.findall(r'^MUX2_WHY_PASS ', text, re.M)) == 42, 'why PASS coverage mismatch')
    outputs = set(re.findall(r'^MUX2_WHY_OUTPUT (.+)$', text, re.M))
    require(len(outputs) == 1, 'missing GPU screenshot output')
    directory = Path(next(iter(outputs)))
    images = [directory / f'why-{scale}.png' for scale in ('1.0', '1.25')]
    require(all(p.is_absolute() and p.is_file() for p in images), 'missing why scale images')
    return images
