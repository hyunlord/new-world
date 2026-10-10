# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/audit_converter_literal.py
"""Literal BRIEF6.1 acceptance audit; exit1 means specification remains unmet."""
from __future__ import annotations
import io
import json
import numpy as np
from PIL import Image
from test_normalize_v3 import enlarged
from pixel_normalize_v3 import recover_cell_v3, quantize_native
from palette_tools import rgb_to_lab


def run() -> dict:
    palette=np.array([[72,64,44],[95,84,60],[131,110,80],[174,150,111],[212,182,142],
                      [210,25,30],[30,130,70],[40,100,200]],dtype=np.uint8)
    source=palette[np.random.default_rng(91).integers(0,len(palette),(28,20))]
    cases=[]
    for scale in (5.3,5.9,6.4):
        for phase in (0.,1.7,3.1):
            image=enlarged(source,scale,phase)
            png,_=recover_cell_v3(image)
            buffer=io.BytesIO();image.save(buffer,format='JPEG',quality=88)
            with Image.open(io.BytesIO(buffer.getvalue())) as opened:jpeg,_=recover_cell_v3(opened.convert('RGB'))
            matched=quantize_native(jpeg,[tuple(map(int,color)) for color in palette])
            ratios=[]
            for out in (png,jpeg,matched):
                delta=np.linalg.norm(rgb_to_lab(np.asarray(out)[...,:3])-rgb_to_lab(source),axis=2)
                ratios.append(float((delta<3).mean()))
            cases.append({'scale':scale,'phase':phase,'png_size':list(png.size),'jpeg_size':list(jpeg.size),
                          'png_fraction':ratios[0],'raw_jpeg_fraction':ratios[1],'known_palette_jpeg_fraction':ratios[2],
                          'passed':png.size==(20,28) and jpeg.size==(20,28) and ratios[0]>=.98 and ratios[1]>=.90})
    trap=np.tile(np.array([40,45,200,205],dtype=np.uint8),5)[None,:,None]
    trap=np.broadcast_to(trap,(28,20,3)).copy();trap[1::2]+=np.uint8(7)
    result,meta=recover_cell_v3(enlarged(trap,4.,1.7))
    harmonic={'expected_period':4.,'actual_period':meta['period_xy'],'expected_size':[20,28],
              'actual_size':list(result.size),'passed':result.size==(20,28) and abs(meta['period_xy'][0]-4)<.1}
    return {'literal_section_6_1_passed':all(case['passed'] for case in cases) and harmonic['passed'],
            'cases':cases,'harmonic':harmonic}

if __name__=='__main__':
    report=run()
    print(json.dumps(report,indent=2))
    raise SystemExit(0 if report['literal_section_6_1_passed'] else 1)
