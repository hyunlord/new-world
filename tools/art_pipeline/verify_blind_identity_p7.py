"""Compare final A11 crops against the already frozen key; never regrade answers."""
from __future__ import annotations
import hashlib
import json
from pathlib import Path
import sys
from PIL import Image, ImageChops, ImageFont
from ingame_qa_types import load_capture
from make_blind_grid_p6 import category, compose_grid, cue_box


def verify(raw: Path, frozen: Path, key_path: Path, output: Path) -> dict:
    """Reconstruct only the same numbered crops and require exact frozen-grid pixels."""
    key_bytes = key_path.read_bytes()
    if key_path.with_suffix('.sha256').read_text().split()[0] != hashlib.sha256(key_bytes).hexdigest():
        raise ValueError('Original frozen key SHA changed')
    key = json.loads(key_bytes)
    records = []
    font = ImageFont.truetype('assets/fonts/galmuri/Galmuri11.ttf',18)
    for zoom in (4,2):
        crops = []
        rows = []
        for answer in key['answers']:
            stem = answer['source'] if zoom == 4 else 'a_lineup_z2'
            capture = load_capture(raw/f'{stem}.png')
            entity = next(row for row in capture.entities if row.ident==answer['agent_id'] and row.role=='blind_lineup')
            rectangle = cue_box(entity,capture.scale)
            source = Image.fromarray(capture.image()).crop(rectangle)
            width,height = round(source.width*zoom/capture.zoom),round(source.height*zoom/capture.zoom)
            crop = source.resize((width,height),Image.Resampling.NEAREST)
            crops.append(crop)
            rows.append({'number':answer['number'],'agent_id':entity.ident,'activity_matches':category(entity)==answer['activity'],
                'frame_matches':entity.frame==answer['frame_id'],'crop_rgb_sha256':hashlib.sha256(crop.tobytes()).hexdigest()})
        composed = compose_grid(crops,font)
        previous_path = frozen/f'blind_{zoom}x.png'
        previous = Image.open(previous_path).convert('RGB')
        identical = previous.size==composed.size and ImageChops.difference(previous,composed).getbbox() is None
        records.append({'zoom':zoom,'grid_pixels_identical':identical,'frozen_grid':str(previous_path),
            'frozen_file_sha256':hashlib.sha256(previous_path.read_bytes()).hexdigest(),
            'final_grid_rgb_sha256':hashlib.sha256(composed.tobytes()).hexdigest(),'crops':rows})
    result = {'passed':all(row['grid_pixels_identical'] and all(crop['activity_matches'] and crop['frame_matches'] for crop in row['crops']) for row in records),
        'source_sha256':{name:hashlib.sha256((raw/name).read_bytes()).hexdigest() for name in ('a_lineup_z4.png','a_lineup_z2.png')},
        'final_raw':str(raw),'key':str(key_path),'key_sha256':hashlib.sha256(key_bytes).hexdigest(),
        'scope':'Frozen A11 calibration lineup identity only; no new key, regrading or natural recognition claim.', 'grids':records}
    output.write_text(json.dumps(result,indent=2)+'\n')
    return result


if __name__ == '__main__':
    result = verify(*(Path(argument) for argument in sys.argv[1:]))
    print(json.dumps({'passed':result['passed'],'zooms':[{'zoom':row['zoom'],'identical':row['grid_pixels_identical']} for row in result['grids']]}))
    raise SystemExit(0 if result['passed'] else 1)
