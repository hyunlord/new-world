# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: python3 -m art.pawn_lab.build MANIFEST CHECKS OUTPUT
"""Build hashed pawn review specimens using checker-admitted source layers."""
from __future__ import annotations
import hashlib
import json
import sys
from dataclasses import asdict
from pathlib import Path
from PIL import Image, ImageDraw
from art.pipeline3.layer_schema import load_layer
from art.pipeline3.capture_lab import gif_from_frames
from art.pipeline4.pawn_gen import PawnLook, compose_layers
from .samples import PEOPLE, FRAMES, Person

ROOT = Path(__file__).resolve().parents[2]


def look(person: Person, facing: str, state: str) -> PawnLook:
    values = asdict(person)
    values.pop('activity')
    return PawnLook(**values, facing=facing, state=state)


def ground(name: str) -> Image.Image:
    with Image.open(ROOT / f'art/style05/ground_v2/{name}.png') as image:
        return image.convert('RGBA').crop((0, 0, 104, 112))


def stage(pawn: Image.Image, name: str) -> Image.Image:
    result = ground(name)
    offset = ((104 - pawn.width) // 2, 104 - pawn.height)
    if pawn.size == (64, 96):
        shadow = Image.new('RGBA', result.size)
        ImageDraw.Draw(shadow).ellipse((offset[0] + 20, offset[1] + 84,
                                       offset[0] + 43, offset[1] + 91), fill=(0, 0, 0, 100))
        result.alpha_composite(shadow)
    result.alpha_composite(pawn, offset)
    return result.convert('RGB')


def build(manifest: Path, checks_path: Path, output: Path) -> None:
    raw = json.loads(manifest.read_text())
    checks = json.loads(checks_path.read_text())
    layers = [load_layer(manifest.parent / file) for file in raw['layers']]
    paths = [manifest, *[manifest.parent / file for file in raw['layers']],
             *[layer.image for layer in layers], *[layer.mask for layer in layers]]
    hashes = {str(path.resolve()): hashlib.sha256(path.read_bytes()).hexdigest() for path in paths}
    if checks['route'] != 'P' or any(checks['input_hashes'].get(path) != digest for path, digest in hashes.items()):
        raise ValueError('Checker evidence does not match current pawn bytes')
    status = 'STRUCTURAL_PASS_REVIEW_PENDING' if checks['passed'] else 'DIAGNOSTIC_REJECTED'
    output.mkdir(parents=True, exist_ok=True)
    (output / 'gifs').mkdir(exist_ok=True)
    (output / 'textures').mkdir(exist_ok=True)
    (output / 'stills').mkdir(exist_ok=True)
    answers = []
    sprites = []
    for number, person in enumerate(PEOPLE, 1):
        if number <= 6:
            compose_layers(manifest.parent, look(person, 'S', FRAMES[person.activity][0])).save(output / 'stills' / f'{number:02d}.png')
        for zoom in (1, 2):
            directory = output / f'combo_{number:02d}_{zoom}x'
            directory.mkdir(exist_ok=True)
            for index, state in enumerate(FRAMES[person.activity]):
                panel = Image.new('RGB', (208, 224))
                for column, facing in enumerate(('S', 'E')):
                    pawn = compose_layers(manifest.parent, look(person, facing, state))
                    for row, name in enumerate(('meadow', 'forest_floor')):
                        panel.paste(stage(pawn, name), (column * 104, row * 112))
                panel.resize((208 * zoom, 224 * zoom), Image.Resampling.NEAREST).save(directory / f'frame_{index:02d}.png')
            gif = output / 'gifs' / f'p_{number:02d}_{zoom}x.gif'
            gif_from_frames(directory, gif)
            answers.append({'number': number, 'zoom': zoom, **asdict(person),
                            'facings': ['S', 'E'], 'grounds': ['meadow', 'forest_floor'],
                            'sha256': hashlib.sha256(gif.read_bytes()).hexdigest()})
        for facing in ('S', 'E'):
            files = []
            for index, state in enumerate(('idle', 'walk_a', 'walk_b', 'work_a', 'work_b', 'carry', 'sleep')):
                target = output / 'textures' / f'p_{number:02d}_{facing}_{index}.png'
                compose_layers(manifest.parent, look(person, facing, state)).save(target)
                files.append(str(target.resolve()))
            sprites.append({'number': number, 'facing': facing, 'files': files})
    for name in ('meadow', 'forest_floor'):
        ground(name).save(output / f'{name}.png')
    (output / 'DATASET_STATUS.txt').write_text(status + '\nReview evidence is not acceptance.\n')
    (output / 'answer_key.private.json').write_text(json.dumps(answers, indent=2) + '\n')
    (output / 'godot_manifest.json').write_text(json.dumps({'status': status, 'sprites': sprites,
        'ground': str((output / 'meadow.png').resolve()), 'source_hashes': hashes,
        'textures_sha256': {file: hashlib.sha256(Path(file).read_bytes()).hexdigest()
                            for sprite in sprites for file in sprite['files']}}, indent=2) + '\n')
    (output / 'provenance.json').write_text(json.dumps({'status': status, 'input_hashes': hashes,
        'checks_sha256': hashlib.sha256(checks_path.read_bytes()).hexdigest(),
        'method': 'checked stored layers; shared registration; 1x composition and nearest 2x; GIF-only palette encoding',
        'panel_layout': 'S left/E right, meadow top/forest floor bottom',
        'ground_sources': {str(ROOT / f'art/style05/ground_v2/{name}.png'): hashlib.sha256((ROOT / f'art/style05/ground_v2/{name}.png').read_bytes()).hexdigest() for name in ('meadow', 'forest_floor')},
        'runtime': 'Pillow frames; native browser playback and Godot runtime separate'}, indent=2) + '\n')
    print(status)


if __name__ == '__main__':
    build(*map(Path, sys.argv[1:4]))
