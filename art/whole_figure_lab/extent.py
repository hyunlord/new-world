# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: python3 -m art.whole_figure_lab.extent SELECTION_JSON OUTPUT
"""Exploratory enlarged-canvas playback, never a conforming sprite delivery."""
from __future__ import annotations

import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Final

import numpy as np
from PIL import Image, ImageDraw

from art.pipeline3.capture_lab import gif_from_frames

from .build import ROOT, sha, source_directory
from .samples import FRAMES, PEOPLE, STATES

STATUS: Final = 'diagnostic_nonconforming_canvas'


@dataclass(frozen=True, slots=True)
class Extent:
    """Entire Lanczos-scaled owned crop and its structural contact coordinate."""
    image: Image.Image
    contact: int
    source: Path
    scale: float


def native_directory(selection: Path, look: str, facing: str) -> Path:
    """Use explicit native panel directory, never search for alternative identities."""
    raw = json.loads(selection.read_text())
    path = Path(raw['native_sheets'][f'{look}/{facing}'])
    return path if path.is_absolute() else selection.parent / path


def load_extent(selection: Path, look: str, facing: str, state: str) -> Extent:
    """Verify full crop provenance and apply the recorded idle scale once."""
    directory = source_directory(selection, look, facing, True)
    summary = json.loads((directory / 'sheet_checks.json').read_text())
    native = native_directory(selection, look, facing)
    judges = json.loads((native / 'native_panel_judges.json').read_text())
    path = native / f'{state}_native.png'
    record = next(row for row in judges['records'] if row['frame'] == state)
    if (judges['raw_sha256'] != summary['raw_sha256'] or record['verdict']['sha256'] != sha(path)
            or summary['panel_detection_version'] != 3 or not summary['ownership']['reconstructed_exactly']):
        message = f'Native crop provenance mismatch: {path}'
        raise ValueError(message)
    scale = float(summary['scale'])
    target_height = 40 if look.startswith('child_') else 56
    if scale != target_height / summary['idle_source_height']:
        message = f'Unexpected idle-derived scale: {path}'
        raise ValueError(message)
    with Image.open(path) as source:
        if source.mode != 'RGBA':
            message = f'Native crop is not RGBA: {path}'
            raise ValueError(message)
        rejected = directory / f'{state}_rejected_source.png'
        if rejected.exists():
            with Image.open(rejected) as original:
                if original.size != source.size or original.tobytes() != source.tobytes():
                    message = f'Rejected source differs from owned crop: {path}'
                    raise ValueError(message)
        image = source.resize((max(1, round(source.width * scale)), max(1, round(source.height * scale))), Image.Resampling.LANCZOS)
    ys, _ = np.nonzero(np.asarray(image)[:, :, 3] >= 128)
    if not len(ys):
        message = f'No structural alpha128 contact: {path}'
        raise ValueError(message)
    return Extent(image, int(ys.max()), path, scale)


def stage_extent(frame: Extent, name: str, size: tuple[int, int], baseline: int, bob: int = 0) -> Image.Image:
    """Tile unchanged ground; paste complete resized extent with no clipping."""
    with Image.open(ROOT / f'art/style05/ground_v2/{name}.png') as source:
        ground = source.convert('RGBA')
    result = Image.new('RGBA', size)
    for y in range(0, size[1], ground.height):
        for x in range(0, size[0], ground.width):
            result.alpha_composite(ground, (x, y))
    x, y = (size[0] - frame.image.width) // 2, baseline - frame.contact + bob
    if x < 0 or y < 0 or x + frame.image.width > size[0] or y + frame.image.height > size[1]:
        message = f'Extent would clip: {frame.source}'
        raise ValueError(message)
    result.alpha_composite(frame.image, (x, y))
    return result.convert('RGB')


def panel_extent(frames: dict[tuple[str, str, str], Extent], look: str, state: str,
                 size: tuple[int, int], baseline: int, bob: int = 0) -> Image.Image:
    """Keep enlarged cells and warning outside the unedited figure pixels."""
    result = Image.new('RGB', (size[0] * 2, size[1] * 2 + 28), '#e0bd8e')
    for column, facing in enumerate(('S', 'E')):
        for row, ground in enumerate(('meadow', 'forest_floor')):
            tile = stage_extent(frames[(look, facing, state)], ground, size, baseline, bob)
            result.paste(tile, (column * size[0], row * size[1]))
    draw = ImageDraw.Draw(result)
    draw.text((2, size[1] * 2), 'DIAGNOSTIC / NONCONFORMING', fill='#18110a')
    draw.text((2, size[1] * 2 + 13), 'Exploratory only; S1/R1 not certified', fill='#18110a')
    return result


def build(selection: Path, output: Path) -> None:
    """Keep frozen identity coverage; fail if even one native input is missing."""
    frames = {(p.look, facing, state): load_extent(selection, p.look, facing, state)
              for p in PEOPLE for facing in ('S', 'E') for state in STATES}
    baseline = max(frame.contact for frame in frames.values()) + 4
    size = (max(104, max(frame.image.width for frame in frames.values()) + 8),
            max(112, baseline + max(frame.image.height - frame.contact for frame in frames.values()) + 4))
    for name in ('gifs', 'boards', 'stills', 'textures'):
        (output / name).mkdir(parents=True, exist_ok=True)
    records = []
    sheet_records = []
    for p in PEOPLE:
        for facing in ('S', 'E'):
            directory = source_directory(selection, p.look, facing, True)
            summary_path = directory / 'sheet_checks.json'
            native_manifest = native_directory(selection, p.look, facing) / 'native_panel_judges.json'
            sheet_records.append({'summary_path': str(summary_path), 'summary_sha256': sha(summary_path),
                                  'summary': json.loads(summary_path.read_text()),
                                  'native_manifest_sha256': sha(native_manifest)})
            for state in STATES:
                frame = frames[(p.look, facing, state)]
                target = output / 'textures' / f'{p.look}_{facing}_{state}.png'
                frame.image.save(target)
                sidecar = {'dataset_status': STATUS, 'source': str(frame.source), 'source_sha256': sha(frame.source),
                           'scale': frame.scale, 'scaled_size': frame.image.size, 'contact_y': frame.contact,
                           'foot_line': baseline, 'display_cell': size, 'image_sha256': sha(target),
                           'normalization_status': next(row for row in sheet_records[-1]['summary']['frames'] if row['frame'] == state),
                           'S1_R1_eligible': False}
                target.with_suffix('.json').write_text(json.dumps(sidecar, indent=2)+'\n')
                records.append(sidecar)
    answers = []
    for number, person in enumerate(PEOPLE, 1):
        state, bob = FRAMES[person.activity][1]
        if number <= 6:
            frames[(person.look, 'S', state)].image.save(output / 'stills' / f'{number:02d}.png')
        for zoom in (1, 2):
            directory = output / f'frames_{number:02d}_{zoom}x'
            directory.mkdir(exist_ok=True)
            for index, (state, bob) in enumerate(FRAMES[person.activity]):
                tile = panel_extent(frames, person.look, state, size, baseline, bob)
                tile.resize((tile.width*zoom, tile.height*zoom), Image.Resampling.NEAREST).save(directory/f'frame_{index:02d}.png')
            gif = output/'gifs'/f't5_{number:02d}_{zoom}x.gif'
            gif_from_frames(directory, gif)
            answers.append({'number': number, 'zoom': zoom, **asdict(person), 'dataset_status': STATUS,
                            'item': {'work':'stone_axe','carry':'log_bundle'}.get(person.activity,'none'),
                            'S1_R1_eligible': False, 'sha256': sha(gif)})
            tiles = [panel_extent(frames, person.look, state, size, baseline) for state in STATES]
            board = Image.new('RGB',(tiles[0].width*5,tiles[0].height),'#e0bd8e')
            for index,tile in enumerate(tiles):
                board.paste(tile,(index*tile.width,0))
            board.resize((board.width*zoom,board.height*zoom),Image.Resampling.NEAREST).save(output/'boards'/f'lab_t5_{zoom}x_page{number:02d}.png',optimize=True)
    for zoom in (1,2):
        tiles = [panel_extent(frames,p.look,FRAMES[p.activity][1][0],size,baseline,FRAMES[p.activity][1][1]) for p in PEOPLE]
        board = Image.new('RGB',(tiles[0].width*4,tiles[0].height*3),'#e0bd8e')
        for index,tile in enumerate(tiles):
            board.paste(tile,((index%4)*tile.width,(index//4)*tile.height))
        board.resize((board.width*zoom,board.height*zoom),Image.Resampling.NEAREST).save(output/'boards'/f'lab_t5_{zoom}x.png',optimize=True)
    (output/'answer_key.private.json').write_text(json.dumps(answers,indent=2)+'\n')
    (output/'DATASET_STATUS.txt').write_text(STATUS+'\nExploratory; does not satisfy requested canvas or certify S1/R1.\n')
    (output/'provenance.json').write_text(json.dumps({'dataset_status':STATUS,'S1_R1_eligible':False,
        'display_cell':size,'foot_line':baseline,'frames':records,'sheets':sheet_records,'selection_sha256':sha(selection),
        'method':'Full owned native RGBA extent; recorded idle scale; Lanczos once; alpha128 registration; no crop/clipping/repaint; expanded cells.',
        'frame_plan':FRAMES},indent=2)+'\n')
    oversized=[str(p) for p in output.rglob('*') if p.suffix in ('.png','.gif') and p.stat().st_size>1_500_000]
    if oversized:
        message=f'Commit ceiling exceeded; keep raw evidence external: {oversized}'
        raise ValueError(message)


if __name__ == '__main__':
    build(Path(sys.argv[1]),Path(sys.argv[2]))
