# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/mask_review_assignments_p5.py
"""Package blind review cards with original conversion context, never past verdicts."""
from __future__ import annotations

import hashlib
import json
from PIL import Image, ImageDraw
from mask_inputs_p5 import ROOT, OUTPUT, image, load
from natural_boards_p4 import paste


def main() -> None:
    """Freeze three image-only 35-record reviewer assignments."""
    rows = load(OUTPUT / 'candidates.json')['candidates']
    indexed = {row['candidate_id']: row for row in load(OUTPUT / 'review_index.json')['boards']}
    assignments = []
    for row in rows:
        original = image(indexed[row['id']]['path']).convert('RGB')
        board = Image.new('RGB', (original.width + 340, max(original.height, 350)), (161,158,152))
        board.paste(original, (0,0))
        draw = ImageDraw.Draw(board)
        x = original.width+8
        raw = row['images'].get('raw')
        if raw:
            crop = image(raw)
            crop.thumbnail((300,180), Image.Resampling.NEAREST)
            draw.text((x,5),'Original raw cell',fill='black')
            board.paste(crop,(x,22),crop)
        else:
            draw.text((x,5),'Installed frame: raw crop unavailable',fill='black')
        unfitted = row['images'].get('unfitted')
        if unfitted:
            source = image(unfitted)
            draw.text((x,210),'Unfitted native 4x',fill='black')
            paste(board,source,x,225,4)
        path = OUTPUT / 'round1' / 'boards' / (row['id'].replace(':','_').replace('@','_')+'.png')
        path.parent.mkdir(parents=True,exist_ok=True)
        if path.exists():
            raise FileExistsError(path)
        board.save(path)
        assignments.append({'sprite_id': row['slot_id'], 'candidate_id': row['id'],
                            'path': str(path.relative_to(ROOT)), 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()})
    for number,start in enumerate(range(0,len(assignments),35),1):
        path = OUTPUT / 'round1' / f'assignment_{number}.json'
        path.write_text(json.dumps({'images':assignments[start:start+35]},indent=2)+'\n')
    print(f'{len(assignments)} blind cards, three assignments')


if __name__ == '__main__':
    main()
