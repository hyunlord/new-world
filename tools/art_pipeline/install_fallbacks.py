# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/install_fallbacks.py --authorize-budget-fallback
"""Explicitly authorized, provenance-preserving completion of missing256slots."""
from __future__ import annotations
import argparse
import json
from pathlib import Path
import numpy as np
from PIL import Image, ImageDraw
from palette_tools import rgb_to_lab
from pixel_normalize import outline
from pixel_masks import make_mask, MASKS
from procedural_animation import animate
from production_sheets import SHEETS, CellSpec

OBJECT_MASKS = {
 'tree': ('.....aaa.....','...aabbbaa...','..abbbbbba...','.abbbbbbbba..','aabbbbbbbbaa.','.aaabbbbaaa..','....aca......','....aca......','...aacaa.....'),
 'shelter': ('......a......','.....aba.....','....abbba....','...abbbbba...','..abbbbb b a.'.replace(' ',''),'..abbbbbbbba.','.abbb...bbba.','aabbb...bbbaa'),
 'storage': ('..aaaaaaaaa..','.aabbbbbbbaa.','aabbbbbbbbbaa','.aacccccccaa.','..aaaaaaaaa..'),
 'rack': ('a.......a','aaaaaaaaa','abbbbb b a'.replace(' ',''),'abbbbb b a'.replace(' ',''),'abbbbb b a'.replace(' ',''),'a.......a'),
 'frame': ('....a....','...a.a...','..a...a..','.aaaaaaa.','a.a...a.a','..a...a..','..a...a..'),
 'bush': ('...aaaaa...','..abbbbba..','.abbbbb b a.'.replace(' ',''),'aabbbbbbbaa','.aaabbbaaa.','....aca....'),
 'rock': ('...aaaa...','..abbbaa..','.abbbbbaa.','aabbbbccaa','.aaaaaaa..'),
 'log': ('aaaaaaaaaaa','abbbbbbbbca','aaaaaaaaaaa'),
 'fire': ('.....a.....','....aba....','...abbba...','..abbbbba..','.abbbbbbba.','..abbbbba..','...abbba...','...aaaaa...','.ccccccccc.'),
}


def _ascii(name: str, palette: list[tuple[int,int,int]], size: tuple[int,int]) -> Image.Image:
    """Author sparse native object silhouettes without geometric resampling."""
    rows = OBJECT_MASKS[name]
    order = np.argsort(rgb_to_lab(np.asarray(palette))[:,0])
    colors = {'a':palette[int(order[0])],'b':palette[int(order[len(order)*2//3])],'c':palette[int(order[len(order)//3])]}
    if name == 'fire':
        colors['b'] = min(palette,key=lambda c:sum((c[k]-[255,150,25][k])**2 for k in range(3)))
    canvas=Image.new('RGBA',(max(map(len,rows)),len(rows)))
    for y,row in enumerate(rows):
        for x,key in enumerate(row):
            if key in colors:canvas.putpixel((x,y),(*colors[key],255))
    if canvas.width>size[0] or canvas.height>size[1]:raise ValueError(f'Authored mask exceeds budget: {name}')
    return canvas


def stateful_object(sheet: str, name: str, size: tuple[int,int], palette: list) -> tuple[Image.Image,str] | None:
    """Author discrete state changes in native pixels; never scale a source."""
    def nearest(target):return min(palette,key=lambda c:sum((c[k]-target[k])**2 for k in range(3)))
    dark=nearest((12,16,17));stone=nearest((130,125,110));wood=nearest((106,71,45))
    hide=nearest((193,146,86));fire=nearest((255,150,25));leaf=nearest((70,100,45))
    phase=int(name.rsplit('_',1)[1]) if name.rsplit('_',1)[-1].isdigit() else 0
    if name.startswith(('hearth','embers')) or name=='cold_ring':
        image=Image.new('RGBA',(22,24));d=ImageDraw.Draw(image)
        for x,y in ((3,20),(7,22),(12,22),(17,20),(18,17),(2,17)):
            d.rectangle((x,y,x+2,y+1),fill=(*stone,255))
        d.line((5,20,15,17),fill=(*wood,255),width=2)
        d.line((5,17,15,20),fill=(*wood,255),width=2)
        if name.startswith('hearth_lit'):
            shift=phase%4
            d.polygon([(6,18),(5,14),(8,10),(9+shift,4+shift),(12,11),(15,8+shift),(17,15),(15,18)],fill=(*dark,255))
            d.polygon([(7,17),(7,14),(10,9+shift),(12,13),(14,11+shift),(15,16),(13,18)],fill=(*fire,255))
        if name.startswith('embers'):
            d.rectangle((8,19,10,19),fill=(*fire,255));d.point((13+phase%2,18),fill=(*fire,255))
        if name=='cold_ring':image=image.crop(image.getbbox())
        return image,'hearth_state:'+name
    if name.startswith('storage'):
        image=Image.new('RGBA',(28,20));d=ImageDraw.Draw(image)
        d.ellipse((1,8,26,19),fill=(*dark,255),outline=(*stone,255),width=2)
        d.arc((1,8,26,19),0,180,fill=(*wood,255),width=2)
        level={'storage_empty':0,'storage_half':1,'storage_full':2}[name]
        for row in range(level*2):
            for col in range(5):
                x,y=5+col*4+(row%2),15-row*2
                d.rectangle((x,y,x+2,y+1),fill=(*hide,255))
        return image,'storage_fill:'+str(level)
    if name.startswith('construction'):
        image=Image.new('RGBA',(30,27));d=ImageDraw.Draw(image)
        stage={'construction_stakes':0,'construction_frame':1,'construction_half':2,'construction_near':3}[name]
        for x in (3,14,26):d.line((x,19,x,25),fill=(*wood,255),width=2)
        d.line((3,22,26,22),fill=(*hide,255))
        if stage>=1:
            d.line((3,24,14,3,26,24),fill=(*wood,255),width=2);d.line((7,16,23,16),fill=(*wood,255),width=2)
        if stage>=2:d.polygon([(5,22),(14,5),(14,22)],fill=(*hide,255))
        if stage>=3:
            d.polygon([(14,5),(24,22),(18,22)],fill=(*hide,255));d.line((9,17,19,17),fill=(*wood,255))
        return image,'construction_stage:'+str(stage)
    if sheet=='N1' and name.startswith(('broadleaf','conifer','dead','pale','sapling')):
        w,h=min(size[0],28),min(size[1],36);image=Image.new('RGBA',(w,h));d=ImageDraw.Draw(image);cx=w//2
        d.rectangle((cx-1,h//3,cx+1,h-1),fill=(*(stone if name.startswith('pale') else wood),255))
        if name.startswith('dead'):
            d.line((cx,h//2,3,h//3),fill=(*wood,255),width=2);d.line((cx,h//2,w-4,4),fill=(*wood,255),width=2)
        elif name.startswith('conifer'):
            for y in (1,7,13):d.polygon([(cx,y),(2+phase,y+12),(w-3-phase,y+12)],fill=(*leaf,255),outline=(*dark,255))
        else:
            inset=phase%3+1
            d.ellipse((inset,1,w-2,h-9-phase%3),fill=(*leaf,255),outline=(*dark,255),width=1)
            for x,y in ((cx-4,5+phase),(cx+4,9),(cx-6,14)):
                d.rectangle((x,y,x+2,y+1),fill=(*hide,255))
        return image,'tree_species_variant:'+name
    return None


def _native(root: Path, entry: dict) -> Image.Image:
    """Read retained pre-outline source, or remove the known one-texel padding."""
    with Image.open(root/entry.get('native_path',entry['path'])) as opened:image=opened.convert('RGBA')
    if 'native_path' not in entry:
        image=image.crop((1,1,image.width-1,image.height-1))
    return image


def storage_from_base(base: Image.Image, palette: list, level: int) -> Image.Image:
    """Fill only interior opaque texels, preserving accepted size and silhouette."""
    rgba=np.array(base.convert('RGBA'))
    color=min(palette,key=lambda c:sum((c[k]-[193,146,86][k])**2 for k in range(3)))
    width,height=base.size
    for row in range(level*2):
        y=int(height*.56)-row*2
        for column in range(5):
            x=int(width*.24)+column*max(2,width//10)+(row%2)
            for dx in (0,1):
                if 0<=y<height and 0<=x+dx<width and rgba[y,x+dx,3]==255:
                    rgba[y,x+dx,:3]=color
    return Image.fromarray(rgba)


def _object(root: Path, sheet: str, spec: CellSpec, entries: dict, palette: list) -> tuple[Image.Image,str,list[str]]:
    name=spec.name
    empty=entries.get('S1_08')
    if sheet=='S1' and name in ('storage_half','storage_full') and empty is not None and empty.get('qa_status')=='accepted':
        level=1 if name=='storage_half' else 2
        return storage_from_base(_native(root,empty),palette,level),'storage_fill_from_accepted:S1_08:'+str(level),[]
    state=stateful_object(sheet,name,spec.size,palette) if sheet in ('S1','S2','N1') else None
    if state is not None:return state[0],state[1],[]
    prefix=name.split('_')[0]
    phase=int(name.rsplit('_',1)[1]) if name.rsplit('_',1)[-1].isdigit() else spec.frame
    if sheet in ('N3','F1','F2','U1'):
        key={'berry':'berries'}.get(prefix,prefix)
        image=make_mask(key,palette,spec.size,phase) if key in MASKS else make_mask('pebble',palette,spec.size)
        return image if sheet in ('U1','F1','F2') else image.crop(image.getbbox()),f'ascii:{key}:phase{phase}',[]
    mapping={
      'hearth':'01_campfire','embers':'01_campfire','shelter':'02_lean_to','storage':'03_storage_pit',
      'construction':'04_shelter_frame','berry':'05_berry_bush_full','boulder':'13_boulder',
      'broadleaf':'14_young_tree','conifer':'14_young_tree','sapling':'14_young_tree','reed':'15_reed_tuft',
      'basket':'16_berry_basket'}
    source=entries.get('direction_'+mapping.get(prefix,''))
    if source is not None:
        image=_native(root,source)
        if image.width<=spec.size[0] and image.height<=spec.size[1]:
            if prefix=='berry':
                stage={'berry_full':'05_berry_bush_full','berry_two':'06_berry_bush_two','berry_one':'07_berry_bush_one','berry_bare':'08_berry_bush_bare'}[name]
                image=_native(root,entries['direction_'+stage])
            return image,'reuse:'+source['sprite_id'],['Static direction source substituted; requested substate may not be represented.']
    family = 'rock'
    if prefix in ('broadleaf','conifer','dead','sapling','pale'):family='tree'
    if prefix in ('shrub','berry'):family='bush'
    if prefix in ('construction',):family='frame'
    if prefix in ('drying','hide'):family='rack'
    if prefix in ('shelter',):family='shelter'
    if prefix in ('storage',):family='storage'
    if prefix in ('hearth','embers'):family='fire'
    if prefix in ('woodpile','sitting','fallen','bedroll','sticks'):family='log'
    return _ascii(family,palette,spec.size),'ascii:'+family,['Authored simplified family silhouette; subtype detail remains provisional.']


def install(root: Path, reason: str) -> dict:
    """Fill missing slots only; fallback status never impersonates accepted art."""
    manifest_path=root/'assets/art/manifest.json';manifest=json.loads(manifest_path.read_text())
    entries={e['sprite_id']:e for e in manifest['assets'] if e.get('sprite_id')}
    palette=[tuple(c['rgb']) for c in json.loads((root/'tools/art_pipeline/production_palette.json').read_text())['colors']]
    dark=palette[int(np.argmin(rgb_to_lab(np.asarray(palette))[:,0]))]
    installed=[]
    for sheet,specs in SHEETS.items():
        for i,spec in enumerate(specs):
            sprite_id=f'{sheet}_{i+1:02d}'
            if entries.get(sprite_id,{}).get('qa_status') in ('accepted','fallback'):continue
            notes=[];source_id=''
            if spec.view:
                idle_index=1 if spec.activity=='asleep' else {'S':1,'N':3,'E':4}[spec.view]
                source_id=f'{sheet[:3]}1_{idle_index:02d}'
                source=entries.get(source_id)
                if source is None or source.get('qa_status')!='accepted':
                    source_id='direction_12_villager_child' if sheet.startswith('V_c') else 'direction_09_villager_gatherer'
                    source=entries[source_id]
                    notes.append('No accepted same-view idle: D3 front-view fallback; directional identity unverified.')
                    if sheet.startswith('V_b'):notes.append('Adult B distinct body unavailable; same D3 gatherer body reused.')
                idle=_native(root,source)
                basket=_native(root,entries['direction_16_berry_basket'])
                native=animate(idle,spec.activity,spec.frame,spec.view,basket=basket,bedroll=_ascii('log',palette,(28,14)))
                rule='animation:'+spec.activity
            else:
                native,rule,notes=_object(root,sheet,spec,entries,palette)
            if native.width>spec.size[0] or native.height>spec.size[1]:
                notes.append('Requested pose exceeds class budget; retained unresized budget-fitting source pose.')
                if spec.view:native=idle
                if native.width>spec.size[0] or native.height>spec.size[1]:
                    native=_ascii('log',palette,spec.size);notes.append('No budget-fitting pose; abstract resting silhouette substituted.')
            sprite=native if sheet=='U1' else outline(native,dark)
            directory=root/'assets/art/production'/sheet;directory.mkdir(parents=True,exist_ok=True)
            path=directory/f'{i+1:02d}.png';reference=directory/f'{i+1:02d}_native.png'
            sprite.save(path);native.save(reference)
            entry={'sprite_id':sprite_id,'path':str(path.relative_to(root)),'native_path':str(reference.relative_to(root)),
                   'source':'procedural:'+rule,'source_sprite':source_id,'native_size':list(native.size),
                   'canvas_size':list(sprite.size),'anchor':[sprite.width//2,sprite.height-1],
                   'qa_status':'fallback','class':'child' if sheet.startswith('V_c') else ('adult' if spec.view else sheet),
                   'body':{'a':'adult_a','b':'adult_b','c':'child'}.get(sheet[2:3],'') if spec.view else '',
                   'view':spec.view,'activity':spec.activity,'frame':spec.frame,'fallback_reason':reason,'limitations':notes}
            entries[sprite_id]=entry
            manifest['assets'].append(entry)
            manifest['assets'].append({'path':str(reference.relative_to(root)),'source':entry['source'],
                   'native_size':list(native.size),'anchor':[native.width//2,native.height-1],'qa_status':'fallback','role':'native_reference'})
            installed.append({'sprite_id':sprite_id,'rule':rule,'limitations':notes})
    manifest_path.write_text(json.dumps(manifest,indent=2)+'\n')
    report={'reason':reason,'installed_count':len(installed),'installed':installed}
    (root/'docs/art/production/fallback_install.json').write_text(json.dumps(report,indent=2)+'\n')
    return report

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root',type=Path,default=Path.cwd())
    parser.add_argument('--authorize-budget-fallback',action='store_true')
    parser.add_argument('--reason',default='Generation time or image budget exhausted; explicit parent authorization')
    args=parser.parse_args()
    if not args.authorize_budget_fallback:parser.error('Installation requires explicit --authorize-budget-fallback')
    result=install(args.root,args.reason)
    print(json.dumps({'installed':result['installed_count']}))
