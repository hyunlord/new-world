"""Normalize the 16 D3 originals and install audited provisional production assets."""
from __future__ import annotations
import hashlib
import json
from pathlib import Path
import numpy as np
from PIL import Image, ImageDraw
from palette_tools import rgb_to_lab
from pixel_normalize import outline
from pixel_normalize_v3 import recover_cell_v3, quantize_native, evaluate_sprite, evaluate_stages
from run_v2 import limits

ROOT = Path(__file__).resolve().parents[2]


def main() -> None:
    source = Path('/Users/rexxa/work/art-production-raw/D3_direction/source.png')
    sheet = Image.open(source).convert('RGBA')
    palette_spec = json.loads((ROOT/'tools/art_pipeline/production_palette.json').read_text())
    palette = [tuple(color['rgb']) for color in palette_spec['colors']]
    darkest = palette[int(np.argmin(rgb_to_lab(np.array(palette))[:,0]))]
    cells = json.loads((ROOT/'tools/art_pipeline/sheet_v1.json').read_text())['cells']
    output = ROOT/'assets/art'
    (output/'direction').mkdir(parents=True, exist_ok=True)
    (output/'atlas').mkdir(exist_ok=True)
    atlas = Image.new('RGBA',(2048,2048))
    entries = {}
    manifest = []
    reports = []
    normalized = []
    for index,cell in enumerate(cells):
        x,y = index%4,index//4
        raw = sheet.crop((x*sheet.width//4,y*sheet.height//4,(x+1)*sheet.width//4,(y+1)*sheet.height//4))
        native,meta = recover_cell_v3(raw)
        sample = Image.fromarray(np.asarray(meta.pop('sample_rgba'),dtype=np.uint8))
        quantized = quantize_native(native,palette)
        normalized.append(quantized)
        name = f"{index+1:02d}_{cell['name']}"
        category,maximum = limits(name)
        metrics = evaluate_sprite(sample,quantized,palette,maximum)
        final = outline(quantized,darkest)
        path = output/'direction'/f'{name}.png'
        final.save(path)
        sprite_id = 'direction_'+name
        ax,ay = 1+(index%8)*64,1+(index//8)*64
        atlas.paste(final,(ax,ay))
        anchor = [final.width//2,final.height-1]
        entries[sprite_id] = {'page':'res://assets/art/atlas/world_0.png','rect':[ax,ay,*final.size],
                              'anchor':anchor,'class':category,'frames':[sprite_id]}
        row = {'path':str(path.relative_to(ROOT)),'sprite_id':sprite_id,'source':f'generated:D3_direction/{index+1:02d}',
               'native_size':list(native.size),'canvas_size':list(final.size),'anchor':anchor,
               'qa_status':'accepted' if metrics['passed'] else 'provisional','metrics':metrics,'grid':meta}
        manifest.append(row)
        reports.append(row)
    stages = evaluate_stages(normalized[4:8], [normalized[0]], normalized[8:11], normalized[11], normalized[15])
    semantic_rules = {0: 'fire_pass', 4: 'berry_pass', 5: 'berry_pass', 6: 'berry_pass',
                      7: 'berry_pass', 11: 'child_pass', 15: 'basket_pass'}
    for index, rule in semantic_rules.items():
        reports[index]['metrics']['semantic_rule'] = rule
        reports[index]['metrics']['semantic_pass'] = stages[rule]
        if not stages[rule]:
            reports[index]['qa_status'] = 'provisional'
    atlas.save(output/'atlas/world_0.png')
    (output/'atlas/world_atlas.json').write_text(json.dumps({'version':1,'sprites':entries},indent=2)+'\n')
    shadow=Image.new('RGBA',(10,3));ImageDraw.Draw(shadow).ellipse((0,0,9,2),fill=(0,0,0,255));shadow.save(output/'contact_shadow.png')
    for path,size,anchor in [('assets/art/contact_shadow.png',[10,3],[5,2]),('assets/art/atlas/world_0.png',[2048,2048],[0,0]),('assets/art/villager_ramps.png',[3,15],[0,0])]:
        manifest.append({'path':path,'source':'procedural:tools/art_pipeline/'+('build_production_palette.py' if 'ramps' in path else 'install_direction.py'),
                         'native_size':size,'anchor':anchor,'qa_status':'format_checked'})
    (output/'manifest.json').write_text(json.dumps({'version':1,'source_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'assets':manifest},indent=2)+'\n')
    report_dir=ROOT/'docs/art/production'
    (report_dir/'normalize_v3_d3.json').write_text(json.dumps(reports,indent=2)+'\n')
    lines=['# D3 v3 measurement','','Fidelity uses only common opaque pixels; keyed-out pixels reported separately. All16 installed; remaining failures are provisional.','',
           '|Sprite|Native|Mean ΔE|Keyed out|Size|Status|','|---|---|---:|---:|---|---|']
    for row in reports:
        m=row['metrics'];lines.append(f"|{row['sprite_id']}|{row['native_size']}|{m['mean_delta_e']:.3f}|{m['keyed_out']}|{m['size_pass']}|{row['qa_status']}|")
    lines += ['', '## Semantic checks', '', '```json', json.dumps(stages, indent=2), '```', '',
              'Animation checks do not apply to these single-pose direction cells.',
              'JPEG recovery tests measure colour accuracy after matching the known production palette. Raw JPEG medians do not consistently reach 90%.',
              'The specified edge-minus-midpoint score does not guarantee rejection of a strong 8px harmonic for a 4px fundamental; production-scale 5.9px harmonic fixtures pass.']
    (report_dir/'normalize_v3_d3.md').write_text('\n'.join(lines)+'\n')
    print('\n'.join(lines))


if __name__=='__main__':
    main()
