# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/mask_raw_context_p5.py
"""Recover original raw context from ancestry without rewriting blind review cards."""
from pathlib import Path
from PIL import Image, ImageDraw
from mask_inputs_p5 import ROOT, OUTPUT, load, image
from remask_candidates_p5 import write
from natural_boards_p4 import paste


def main() -> None:
    """Expose true raw crops omitted by the recovery ledger's top-level images."""
    records = []
    for row in load(OUTPUT/'candidates.json')['candidates']:
        if row['mask_scope']=='c':
            continue
        ancestor = row.get('ancestor_record',{})
        raw_path = row.get('raw_path')
        raw_box = row.get('raw_box')
        if not raw_path or not Path(raw_path).is_file() or not raw_box:
            continue
        raw = image(raw_path).crop(tuple(raw_box))
        raw.thumbnail((310,310),Image.Resampling.NEAREST)
        board=Image.new('RGB',(600,350),(161,158,152))
        draw=ImageDraw.Draw(board)
        draw.text((8,4),row['id']+' | original raw | unfitted 4x',fill='black')
        board.paste(raw,(8,28),raw)
        unfitted = row['images'].get('unfitted') or ancestor.get('images',{}).get('unfitted')
        if unfitted and Path(unfitted).is_file():
            paste(board,image(unfitted),335,35,4)
        path=OUTPUT/'raw_context'/(row['id'].replace(':','_').replace('@','_')+'.png')
        path.parent.mkdir(parents=True,exist_ok=True)
        board.save(path)
        records.append({'candidate_id':row['id'],'path':str(path.relative_to(ROOT)),
                        'raw_source':raw_path,'raw_box':raw_box,'unfitted_source':unfitted})
    write(OUTPUT/'raw_context/index.json',{'images':records,
        'note':'Supplement only. Original round1 cards and verdicts retained unchanged; recovery ancestry supplied these raw crops.'})
    print('raw supplemental cards',len(records))


if __name__=='__main__':
    main()
