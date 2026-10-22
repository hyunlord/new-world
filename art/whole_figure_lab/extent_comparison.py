# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: python3 -m art.whole_figure_lab.extent_comparison SELECTION OUTPUT
"""Historical six-row comparison with explicitly nonconforming whole extents."""
import json
import sys
from pathlib import Path

from PIL import Image, ImageDraw

from .build import ROOT, sha
from .extent import STATUS, load_extent, stage_extent


def build(selection: Path, output: Path) -> None:
    """Reuse exact historical pixels without silently matching absent facings."""
    historical=ROOT/'art/character/boards/compare_ABP_2x.png'
    records_path=Path('/Users/rexxa/work/arttrial4-data/round0/lab/boards/compare_provenance.json')
    rows=json.loads(records_path.read_text());items=[]
    for row in rows:
        attrs=row['pawn_attributes'];facing=attrs['facing'];note='expanded canvas; exploratory'
        if facing=='N':
            facing='S';note='N absent; S alternative only'
        top=attrs['top'] if attrs['top']!='none' else 'loincloth'
        hair=attrs['hair'] if attrs['hair']!='none' else 'short'
        state='carry' if attrs['item']=='log_bundle' else attrs['state']
        if state=='sleep': note='sleep: loincloth/short differs'
        look=f'adult_M_{top}_{hair}'
        frame=load_extent(selection,look,facing,state)
        items.append((frame,note,attrs))
    baseline=max(f.contact for f,_,_ in items)+4
    size=(max(104,max(f.image.width for f,_,_ in items)+8),max(112,baseline+max(f.image.height-f.contact for f,_,_ in items)+4))
    row_height=max(244,size[1]*2+32)
    board=Image.new('RGB',(624+size[0]*2,32+row_height*6),'#e0bd8e')
    with Image.open(historical) as image:
        board.paste(image.crop((0,0,624,32)),(0,0))
        for index in range(6):
            board.paste(image.crop((0,32+index*244,624,32+(index+1)*244)),(0,32+index*row_height))
    draw=ImageDraw.Draw(board);draw.text((628,4),'DIAGNOSTIC NONCONFORMING',fill='#18110a')
    records=[]
    for index,(frame,note,attrs) in enumerate(items):
        y=32+index*row_height
        tile=stage_extent(frame,'meadow',size,baseline).resize((size[0]*2,size[1]*2),Image.Resampling.NEAREST)
        board.paste(tile,(624,y));draw.text((628,y+size[1]*2),note,fill='#18110a')
        records.append({'source':str(frame.source),'sha256':sha(frame.source),'scale':frame.scale,'historical_attributes':attrs,'note':note})
    output.mkdir(parents=True,exist_ok=True);target=output/'compare_ABPT_2x.png';board.save(target,optimize=True)
    (output/'compare_provenance.json').write_text(json.dumps({'dataset_status':STATUS,'S1_R1_eligible':False,
        'historical_sha256':sha(historical),'historical_row_pixels':'unchanged, repositioned only',
        'display_cell':size,'foot_line':baseline,'rows':records,'board_sha256':sha(target)},indent=2)+'\n')


if __name__=='__main__':
    build(Path(sys.argv[1]),Path(sys.argv[2]))
