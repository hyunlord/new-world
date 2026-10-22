# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: uv run art/pipeline3/b_render_finish.py RENDER_OUTPUT
"""Lanczos artwork resize with native-index-derived masks and provenance sidecars."""
import hashlib
import json
import sys
from pathlib import Path
from typing import Final

import numpy as np
from numpy.typing import NDArray
from PIL import Image

LabelImage = NDArray[np.uint8]
KINDS: Final = {'BODY': 'body', 'TOP': 'top', 'HAIR': 'hair', 'ITEM': 'item', 'CARRY': 'carry'}


def support_labels(labels: LabelImage, support: NDArray[np.bool_], radius: int = 3) -> LabelImage:
    """Extend native labels only into rendered ink/resampling support, never RGBA."""
    result = labels.copy()
    result[~support] = 0
    if not np.any(result):
        return result
    for _ in range(radius):
        missing = support & (result == 0)
        if not np.any(missing):
            break
        previous = result.copy()
        padded = np.pad(previous, 1)
        for dy, dx in ((-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (-1, 1), (1, -1), (1, 1)):
            shifted = padded[1 + dy:1 + dy + result.shape[0], 1 + dx:1 + dx + result.shape[1]]
            fill = support & (result == 0) & (shifted != 0)
            result[fill] = shifted[fill]
    result[~support] = 0
    return result


def finish_layer(source: Path, destination: Path) -> None:
    """Write 1x data without painting, alpha repair, palette snapping or copyback."""
    name = source.stem
    prefix, direction, frame = name.split('_adult_')[0], *name.split('_adult_')[1].split('_', 1)
    kind = KINDS[prefix.split('_')[0]]
    size = (96, 64) if frame == 'sleep' else (64, 96)
    image = Image.open(source).convert('RGBA').resize(size, Image.Resampling.LANCZOS)
    rgba = np.array(image)
    native_material = source.with_name(name + '_material.png')
    labels = np.array(Image.open(native_material).convert('L').resize(size, Image.Resampling.NEAREST))
    mask = support_labels(labels, rgba[:, :, 3] > 0)
    target = destination / (name + '.png')
    image.save(target)
    mask_path = destination / (name + '_mask.png')
    Image.fromarray(mask).save(mask_path)
    meta: dict[str, str | int | list[str] | list[int]] = {'schema_version': 1, 'id': name, 'image': target.name, 'mask': mask_path.name,
            'kind': kind, 'variant': prefix.partition('_')[2], 'body_type': 'adult',
            'direction': direction, 'frame': frame, 'scale': 1, 'route': 'B',
            'operations': ['native Blender Cycles+Freestyle render', 'Lanczos4x_to1x',
                           'native material index nearest resize; supported ink fringe extends <=3px'],
            'source_4x': str(source), 'source_sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
            'material_pass_sha256': hashlib.sha256(native_material.read_bytes()).hexdigest(),
            'image_sha256': hashlib.sha256(target.read_bytes()).hexdigest(),
            'mask_sha256': hashlib.sha256(mask_path.read_bytes()).hexdigest()}
    if kind == 'body':
        anatomy_path = source.with_name(name + '_anatomy.png')
        anatomical = np.array(Image.open(anatomy_path).convert('L').resize(size, Image.Resampling.NEAREST))
        anatomical = support_labels(anatomical, rgba[:, :, 3] >= 128, radius=1)
        anatomical_target = destination / (name + '_anatomy.png')
        Image.fromarray(anatomical).save(anatomical_target)
        meta['anatomical_mask'] = anatomical_target.name
        meta['anatomical_source_sha256'] = hashlib.sha256(anatomy_path.read_bytes()).hexdigest()
        meta['anatomical_provenance'] = 'native IndexOB from source anatomical_id; support-confined extension <=1px'
        if frame == 'sleep':
            ys, xs = np.nonzero(anatomical == 1)
            if len(xs):
                meta['head_box'] = [int(xs.min()), int(ys.min()), int(xs.max()) + 1, int(ys.max()) + 1]
    if kind == 'top':
        meta['extras'] = ['collar', 'cloak_hem'] if prefix.endswith('fur_cloak') else ['collar']
    if kind in ('item', 'carry'):
        native_metadata = json.loads(source.with_suffix('.render.json').read_text())
        meta['shoulder' if kind == 'carry' else 'grip'] = native_metadata['anchor_game']
        meta['anchor_source'] = native_metadata['anchor_source']
        meta['anchor_provenance'] = 'world_to_camera_view projection; no nearest-support replacement'
    (destination / (name + '.json')).write_text(json.dumps(meta, indent=2) + '\n')


def main() -> None:
    """Keep raw4x intact and write sidecars only for artwork, not ID passes."""
    output = Path(sys.argv[1])
    destination = output / 'layers'
    destination.mkdir(exist_ok=True)
    for source in sorted((output / 'raw4x').glob('*.png')):
        if source.stem.endswith(('_material', '_anatomy')):
            continue
        finish_layer(source, destination)
    for source in sorted(output.glob('look*_idle_S_4x.png')):
        with Image.open(source) as image:
            image.resize((64, 96), Image.Resampling.LANCZOS).save(source.with_name(source.name.replace('_4x', '_1x')))


if __name__ == '__main__':
    main()
