# /// script
# requires-python = ">=3.12"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: uv run --with pillow python -m art.pipeline2.people_v2
"""Extract the TRIAL2b generated sheets with auditable crop-only provenance."""
import hashlib
import json
from pathlib import Path

from PIL import Image

from art.pipeline2.people import cutout

ROOT = Path(__file__).resolve().parents[1]
RAW = Path('/Users/rexxa/work/arttrial2-data/trial2b/people')
DEST = ROOT / 'style05/people'
NAMES = ('head', 'torso', 'upper_arm_l', 'upper_arm_r', 'forearm_l',
         'forearm_r', 'upper_leg_l', 'upper_leg_r', 'lower_leg_l',
         'lower_leg_r', 'foot_l', 'foot_r')


def save_part(source: Path, box: tuple[int, int, int, int], target: Path,
              call_id: str, prompt: Path, tries: int = 1) -> None:
    """Key magenta and resize without drawing, recoloring or outline forcing."""
    target.parent.mkdir(parents=True, exist_ok=True)
    with Image.open(source) as image:
        part = cutout(image.crop(box))
        part.save(target)
    record = {'source': str(source), 'source_sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
              'crop_box': box, 'output_sha256': hashlib.sha256(target.read_bytes()).hexdigest(),
              'tool': 'builtin image_gen', 'tool_call_id': call_id, 'tries': tries,
              'prompt': prompt.read_text(), 'processing': 'magenta key; alpha crop; Lanczos maximum 256; no quantization or outline forcing',
              'status': 'generated candidate; model and readability review separate'}
    target.with_suffix('.json').write_text(json.dumps(record, indent=2) + '\n')


def main() -> None:
    """Export front/east parts, overlays, and the selected person anchor."""
    for facing, call_id in [('front', 'exec-97d02b8b-3d65-48a3-ad43-b9c5dc7bac36'),
                            ('east', 'exec-0b51710b-ce21-4217-be57-81ab90cac394')]:
        source = RAW / f'parts_{facing}_try1_raw.png'
        xs, ys = (0, 350, 680, 970, 1254), (0, 450, 815, 1254)
        for i, name in enumerate(NAMES):
            box = (xs[i % 4], ys[i // 4], xs[i % 4 + 1], ys[i // 4 + 1])
            # Front lower legs include boots in the generated sheet: crop at ankle.
            if facing == 'front' and name.startswith('lower_leg'):
                box = (box[0], 815, box[2], 960)
            folder = 'parts' if facing == 'front' else 'east'
            save_part(source, box, DEST / folder / f'{name}.png', call_id,
                      RAW / f'parts_{facing}_try1_prompt.txt')
    names = ('hair_short', 'hair_long_tied', 'hair_braided', 'top_hide_wrap', 'top_fur_cloak',
             'top_woven_tunic', 'held_stone_axe', 'held_log_bundle', 'held_berry_basket', 'hat')
    top_x, bottom_x = (0, 390, 790, 1220, 1570, 1983), (0, 390, 750, 1220, 1570, 1983)
    for i, name in enumerate(names):
        xs = top_x if i < 5 else bottom_x
        box = (xs[i % 5], 0 if i < 5 else 408, xs[i % 5 + 1], 408 if i < 5 else 793)
        save_part(RAW / 'overlays_try1_raw.png', box, DEST / 'overlays' / f'{name}.png',
                  'exec-c4c6484b-dcdf-4377-b150-13e4a4f67cb5', RAW / 'overlays_try1_prompt.txt')
    for name, box in [('beard', (0, 0, 887, 887)), ('stick', (887, 0, 1774, 887))]:
        save_part(RAW / 'identity_try1_raw.png', box, DEST / 'overlays' / f'{name}.png',
                  'exec-2eb3a441-acf8-4c55-a0aa-a96e8dc3416b', RAW / 'identity_try1_prompt.txt')
    for i, name in enumerate(('head_female_front', 'head_female_east', 'head_male_front', 'head_male_east')):
        x, y = i % 2 * 634, i // 2 * 580
        save_part(RAW / 'heads_revision_try2_raw.png', (x, y, x + 634, 580 if i < 2 else 1241),
                  DEST / 'overlays' / f'{name}.png', 'exec-5b5a1bb0-2ddd-484b-a8b9-ca96f83c3c32',
                  RAW / 'heads_revision_try2_prompt.txt', 2)
    for name, box in [('hammerstone', (0, 0, 887, 887)), ('flint_core', (887, 0, 1774, 887))]:
        save_part(RAW / 'knapping_tools_try1_raw.png', box, DEST / 'overlays' / f'{name}.png',
                  'exec-c4730203-0b44-4b5b-94f9-bb94d789cb66', RAW / 'knapping_tools_try1_prompt.txt')
    save_part(RAW / 'person_anchor_try2_raw.png', (0, 0, 1254, 1254),
              ROOT / 'style05/anchors/person_v2.png', 'exec-b127eb7f-4fb3-41d6-8826-aea1d1b4a01c',
              RAW / 'person_anchor_try2_prompt.txt', 2)


if __name__ == '__main__':
    main()
