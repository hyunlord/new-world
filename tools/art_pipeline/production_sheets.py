# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/production_sheets.py --sheet V_a1 --raw FILE --attempt 1
"""Sheet contracts and append-only acceptance of generated native cells."""
from __future__ import annotations
import argparse
import json
from dataclasses import dataclass
from pathlib import Path
import numpy as np
from PIL import Image, ImageDraw
from pixel_normalize_v3 import recover_cell_v3, quantize_native, evaluate_sprite
from pixel_normalize import outline
from v3_semantics import evaluate_animation, semantic_counts
from palette_tools import rgb_to_lab

@dataclass(frozen=True, slots=True)
class CellSpec:
    name: str
    size: tuple[int, int]
    view: str = ''
    activity: str = ''
    frame: int = 0


def _poses(groups: list[tuple[str, str, int]], size: tuple[int, int]) -> list[CellSpec]:
    return [CellSpec(f'{view}_{activity}_{i + 1}', (size[1], size[0]) if activity == 'asleep' else size,
                     view, activity, i) for view, activity, count in groups for i in range(count)]


def definitions() -> dict[str, list[CellSpec]]:
    """All256 requested row-major slots; ids are sheet_id_01 through sheet_id_16."""
    sheets = {}
    locomotion = [('S', 'idle', 2), ('N', 'idle', 1), ('E', 'idle', 1), ('S', 'walk', 4), ('E', 'walk', 4), ('N', 'walk', 4)]
    work = [('S','carry',4),('E','carry',4),('E','gather',2),('S','gather',2),('E','build',2),('S','build',2)]
    rest = [('N','carry',2),('E','drink',2),('S','eat',2),('S','sit',1),('E','sit',1),('S','talk',2),('E','talk',2),('E','asleep',1),('S','crouch',2),('S','crossed',1)]
    child_rest = [('S','sit',1),('E','sit',1),('S','hop',2),('E','run',4),('S','talk',2),('E','talk',2),('E','asleep',1),('S','eat',2),('S','hug',1)]
    for body in ('a','b'):
        sheets[f'V_{body}1'] = _poses(locomotion, (18,28))
        sheets[f'V_{body}2'] = _poses(work, (18,28))
        sheets[f'V_{body}3'] = _poses(rest, (18,28))
        sheets[f'V_{body}3'][12] = CellSpec('E_asleep_1',(28,14),'E','asleep')
    sheets['V_c1'] = _poses(locomotion,(14,22))
    sheets['V_c2'] = _poses(child_rest,(14,22))
    sheets['V_c2'][12] = CellSpec('E_asleep_1',(22,12),'E','asleep')
    names = {
      'S1': ['hearth_unlit']+[f'hearth_lit_{i}' for i in range(4)]+['embers_1','embers_2','storage_empty','storage_half','storage_full','shelter_A','shelter_B','shelter_C','woodpile','sitting_log','cold_ring'],
      'S2': ['construction_stakes','construction_frame','construction_half','construction_near','drying_rack','hide_frame','knapping','bedroll','waterskin','basket_empty','basket_berries','cairn_1','cairn_2','stone_circle','sticks','stone_stack'],
      'N1': ['broadleaf_large_1','broadleaf_large_2','broadleaf_large_3','broadleaf_small_1','broadleaf_small_2','conifer_1','conifer_2','dead_tree','stump_1','stump_2','shrub_1','shrub_2','shrub_3','sapling','fallen_log','pale_bark_tree'],
      'N2': ['berry_full','berry_two','berry_one','berry_bare','boulder_small','boulder_medium','boulder_large','rock_cluster_1','rock_cluster_2','outcrop_1','outcrop_2','flint','reed_1','reed_2','reed_3','cattails'],
      'N3': ['grass_1','grass_2','grass_3','grass_4','flower_white','flower_yellow','flower_purple','flower_red','mushroom_1','mushroom_2','pebble_1','pebble_2','pebble_3','leaf_1','leaf_2','lily'],
      'F1': ['smoke_1','smoke_2','smoke_3','smoke_4','spark_1','spark_2','dust_1','dust_2','dust_3','splash_1','splash_2','splash_3','sparkle_1','sparkle_2','sleep_1','sleep_2'],
      'F2': ['bird_1','bird_2','bird_3','bird_4','butterfly_1','butterfly_2','butterfly_3','butterfly_4','fish_1','fish_2','fish_3','fish_4','dragonfly_1','dragonfly_2','leaf_1','leaf_2'],
      'U1': ['people','hearth','shelter','hammer','basket','berries','sprout','cairn','sun','moon','water','berry','sleep','pause','fast','scroll']}
    for sheet, items in names.items():
        cells = []
        for i, name in enumerate(items):
            size = (24,22)
            if sheet == 'S1': size = (24,26) if i < 7 else ((34,30) if i < 13 else (24,22))
            if sheet == 'S2' and i < 6: size = (34,30)
            if sheet == 'N1': size = (32,40) if i < 3 else ((24,36) if i in (3,4,5,6,7,13,15) else (24,22))
            if sheet == 'N2' and i in (9,10): size = (32,30)
            if sheet == 'N3': size = (12,12)
            if sheet in ('F1','U1'): size = (16,16)
            if sheet == 'F2': size = (16,12)
            cells.append(CellSpec(name,size))
        sheets[sheet] = cells
    return sheets

SHEETS = definitions()


def normalize_sheet(root: Path, sheet: str, raw: Path, attempt: int) -> dict:
    """Record every rejection and preserve previously accepted cells verbatim."""
    if sheet not in SHEETS or not 1 <= attempt <= 3:
        raise ValueError('Unknown sheet or attempt outside1..3')
    palette_json = json.loads((root/'tools/art_pipeline/production_palette.json').read_text())
    palette = [tuple(entry['rgb']) for entry in palette_json['colors']]
    dark = palette[int(np.argmin(rgb_to_lab(np.asarray(palette))[:,0]))]
    manifest_path = root/'assets/art/manifest.json'
    manifest = json.loads(manifest_path.read_text())
    existing = {entry['sprite_id']: entry for entry in manifest['assets'] if 'sprite_id' in entry}
    with Image.open(raw) as opened: source = opened.convert('RGBA')
    specs = SHEETS[sheet]
    candidates, records = {}, []
    contact = Image.new('RGB',(4*512,4*320),'#20242a')
    draw = ImageDraw.Draw(contact)
    for i, spec in enumerate(specs):
        col,row = i%4,i//4
        cell = source.crop((col*source.width//4,row*source.height//4,(col+1)*source.width//4,(row+1)*source.height//4))
        sprite_id = f'{sheet}_{i+1:02d}'
        record = {'sprite_id':sprite_id,'name':spec.name,'attempt':attempt,'accepted':False,'reasons':[]}
        try:
            native, grid = recover_cell_v3(cell)
            quantized = quantize_native(native,palette)
            sample = Image.fromarray(np.asarray(grid.pop('sample_rgba'),dtype=np.uint8))
            metrics = evaluate_sprite(sample,quantized,palette,spec.size)
            record.update({'metrics':metrics,'grid':grid})
            for key in ('size_pass','fidelity_pass','format_pass'):
                if not metrics[key]: record['reasons'].append(key)
            if spec.name.startswith('hearth_lit') and metrics['fire_texels'] < 20: record['reasons'].append('fire_ramp')
            if sheet == 'U1' and quantized.size != (16,16): record['reasons'].append('exact_16x16')
            candidates[i] = quantized
        except ValueError as error:
            record['reasons'].append(str(error))
        preview = cell.copy(); preview.thumbnail((240,260),Image.Resampling.NEAREST)
        contact.paste(preview,(col*512,row*320+36),preview)
        records.append(record)
    if sheet == 'N2' and all(i in candidates for i in range(4)):
        counts = [semantic_counts(candidates[i])[0] for i in range(4)]
        if not (counts[0]>counts[1]>counts[2]>=1 and counts[3]==0):
            for i in range(4): records[i]['reasons'].append('berry_stage_order')
    if sheet == 'N2' and any(i not in candidates for i in range(4)):
        for i in range(4): records[i]['reasons'].append('incomplete_berry_stages')
    for i,spec in enumerate(specs):
        record = records[i]; sprite_id = record['sprite_id']
        if spec.view and i in candidates:
            idle_index = {'S':0,'N':2,'E':3}[spec.view]
            idle_sheet = sheet[:3]+'1'
            idle = candidates.get(idle_index) if sheet == idle_sheet else None
            idle_entry = existing.get(f'{idle_sheet}_{idle_index+1:02d}')
            if idle is None and idle_entry is not None and idle_entry.get('qa_status') == 'accepted':
                with Image.open(root/idle_entry['native_path']) as opened: idle = opened.convert('RGBA')
            if idle is None or (sheet == idle_sheet and records[idle_index]['reasons']):
                record['reasons'].append('missing_accepted_view_idle')
            else:
                check = evaluate_animation(idle,[candidates[i]])
                if not check['color_pass']: record['reasons'].append('animation_color')
                if not check['feet_baseline_pass']: record['reasons'].append('feet_baseline')
                if spec.activity in ('idle','walk') and not check['head_top_pass']: record['reasons'].append('head_top')
                if spec.activity in ('walk','carry','run'):
                    group = [j for j,s in enumerate(specs) if s.view==spec.view and s.activity==spec.activity]
                    if len(group)==4 and all(j in candidates for j in group):
                        cycle = evaluate_animation(idle,[],[candidates[j] for j in group])
                        if not cycle['walk_pass']: record['reasons'].append('walk_leg_difference')
                    elif len(group)==4:
                        record['reasons'].append('incomplete_walk_cycle')
        if sheet == 'V_c1' and i in candidates:
            adult_heights = [entry['native_size'][1] for entry in existing.values()
                             if entry.get('qa_status') == 'accepted' and entry.get('class') == 'adult' and entry.get('activity') == 'idle']
            if not adult_heights or candidates[i].height >= min(adult_heights):
                record['reasons'].append('child_relative_height')
        if spec.name == 'basket_berries' and i in candidates:
            bush_sizes = [entry['native_size'] for entry in existing.values()
                          if 'berry_bush' in entry.get('sprite_id','') or entry.get('sprite_id','') in ('N2_01','N2_02','N2_03','N2_04')]
            basket = candidates[i]
            if not bush_sizes or any(basket.width > w or basket.height > h or basket.width*basket.height >= w*h for w,h in bush_sizes):
                record['reasons'].append('basket_relative_size')
        preserved = existing.get(sprite_id,{}).get('qa_status') == 'accepted'
        record['preserved'] = preserved
        record['accepted'] = not record['reasons'] or preserved
        if record['accepted'] and not preserved:
            native = candidates[i]
            sprite = outline(native,dark)
            directory = root/'assets/art/production'/sheet; directory.mkdir(parents=True,exist_ok=True)
            path = directory/f'{i+1:02d}.png'; native_path = directory/f'{i+1:02d}_native.png'
            sprite.save(path); native.save(native_path)
            entry = {'sprite_id':sprite_id,'path':str(path.relative_to(root)),'native_path':str(native_path.relative_to(root)),
                'source':f'generated:{sheet}/{i+1:02d}','attempt':attempt,'native_size':list(native.size),
                'canvas_size':list(sprite.size),'anchor':[sprite.width//2,sprite.height-1],'qa_status':'accepted',
                'class':'child' if sheet.startswith('V_c') else ('adult' if spec.view else sheet),
                'body':{'a':'adult_a','b':'adult_b','c':'child'}.get(sheet[2:3],'') if spec.view else '',
                'view':spec.view,'activity':spec.activity,'frame':spec.frame,'metrics':record['metrics']}
            manifest['assets'] = [e for e in manifest['assets'] if e.get('sprite_id') != sprite_id]
            manifest['assets'].append(entry)
            manifest['assets'].append({'path':str(native_path.relative_to(root)),'source':entry['source'],
                'native_size':list(native.size),'anchor':[native.width//2,native.height-1],'qa_status':'accepted','role':'native_reference'})
        if preserved:
            with Image.open(root/existing[sprite_id]['native_path']) as opened: candidates[i] = opened.convert('RGBA')
        if i in candidates:
            preview = candidates[i].resize((candidates[i].width*6,candidates[i].height*6),Image.Resampling.NEAREST)
            contact.paste(preview,(i%4*512+264,i//4*320+40),preview)
        draw.text((i%4*512+4,i//4*320+4),f'{sprite_id} {spec.name}',fill='white')
        draw.text((i%4*512+4,i//4*320+20),'ACCEPT' if record['accepted'] else ','.join(record['reasons']),fill='#99dd99' if record['accepted'] else '#ff9999')
    manifest_path.write_text(json.dumps(manifest,indent=2)+'\n')
    out = root/'docs/art/production/sheets'; out.mkdir(parents=True,exist_ok=True)
    report = {'sheet':sheet,'attempt':attempt,'raw':str(raw),'accepted':sum(r['accepted'] for r in records),'rejected':sum(not r['accepted'] for r in records),'preserved':sum(r['preserved'] for r in records),'cells':records}
    (out/f'{sheet}_attempt_{attempt}.json').write_text(json.dumps(report,indent=2)+'\n')
    contact.save(out/f'{sheet}_attempt_{attempt}.png')
    return report

def prompt_for(sheet: str, very_small: bool = False) -> str:
    """Compose the approved frame, D3 style, identity and row-major cell list."""
    specs = SHEETS[sheet]
    frame = "A 4 × 4 grid sprite sheet on a solid flat magenta (#FF00FF) background. Every item is centred in its own equal square cell with at least 22 % empty margin on every side; nothing touches a cell edge or another item; no shadows on the background; no text, numbers or labels. Same pixel-art style, palette and lighting as the reference image. Items, left to right, top to bottom: "
    descriptions = []
    for spec in specs:
        label = spec.name.replace('_',' ')
        if spec.view:
            facing = {'S':'facing the viewer','N':'facing away','E':'facing right'}[spec.view]
            pose = spec.activity
            if pose in ('walk','carry','run'):
                pose += ', '+('left foot forward','feet passing','right foot forward','feet passing')[spec.frame%4]
            if pose.startswith('carry'):
                pose += ', basket of berries held at chest'
            if spec.name == 'S_idle_2': pose += ', shoulders 1 pixel lower for breathing'
            label = f'{facing}, {pose}'
        descriptions.append(label)
    result = frame+'; '.join(descriptions)+'. '
    if sheet.startswith('V_'):
        body = {'a':'a sturdy adult with shoulder-length dark hair, a short sleeveless hide tunic with a belt, bare arms and legs, leather foot wraps',
                'b':'a slender adult with long dark hair tied back, a knee-length hide wrap dress with a fur collar, bare arms, leather foot wraps',
                'c':'a small child about two-thirds the height of an adult, short hair, a simple hide tunic, bare feet'}[sheet[2]]
        result += f'All sixteen cells show the same person: {body}. Same body proportions, clothes, colours and size in every cell; only the pose changes. '
        result += 'Each figure occupies less than half the cell height, generous empty space, approximately 100–130 image pixels tall. '
    if very_small or sheet in ('N3','F1','F2','U1'):
        result += 'Draw each item very small, filling about a third of its cell; leave exceptionally generous magenta margins. '
    result += f'Native pixel budget per cell: '+', '.join(f'{c.size[0]}×{c.size[1]}' for c in specs)+'. Large deliberate square pixel clusters, no subpixel details. '
    result += 'Flat, bold pixel art with strong silhouettes and minimal texture; large simple shapes that stay readable at small size; one dark 1-pixel outline around every figure and object; light from the upper left; front three-quarter view, standing on the ground; Stone Age people in simple hide and fur clothing with warm skin tones; crisp square pixels, no anti-aliasing, no blur, no gradients.'
    return result

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--sheet',required=True,choices=sorted(SHEETS))
    parser.add_argument('--raw',type=Path)
    parser.add_argument('--attempt',type=int,default=1)
    parser.add_argument('--print-prompt',action='store_true')
    parser.add_argument('--very-small',action='store_true')
    parser.add_argument('--root',type=Path,default=Path.cwd())
    args=parser.parse_args()
    if args.print_prompt:
        print(prompt_for(args.sheet,args.very_small))
    else:
        if args.raw is None: parser.error('--raw is required for normalization')
        report=normalize_sheet(args.root,args.sheet,args.raw,args.attempt)
        print(json.dumps({key:report[key] for key in ('sheet','attempt','accepted','rejected','preserved')}))
