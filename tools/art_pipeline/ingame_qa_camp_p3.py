"""Measure camp clearings and ground density from captured renderer evidence."""
from __future__ import annotations
import json
from pathlib import Path
from PIL import Image
from ingame_qa_pixels import Check, result


def camp_ground(directory: Path, repo: Path) -> Check:
    """Check all drawn map placements, including those outside the viewport."""
    path = directory/'forest_evidence.json'
    if not path.is_file():
        return Check('A15', 'N/A', 'Missing captured terrain, placements and clearing snapshots')
    data = json.loads(path.read_text())
    terrain = data['terrain']
    codes = terrain['terrain']
    if isinstance(codes, str):
        codes = json.loads(codes)
    width = int(terrain['width'])
    centers: set[tuple[int, int]] = set()
    for domain in ['building','construction','resource','settlement']:
        snapshot = data[domain]
        xkey,ykey = ('formation_xs','formation_ys') if domain == 'settlement' else ('xs','ys')
        for index,(x,y) in enumerate(zip(snapshot[xkey],snapshot[ykey],strict=True)):
            if domain != 'resource' or snapshot['kinds'][index] == 0:
                centers.add((int(x),int(y)))
    violations: list[str] = []
    rocks: list[tuple[int,int]] = []
    decals: list[tuple[int,int]] = []
    rock_counts = {2:0,3:0}
    decal_counts = {0:0,1:0,4:0}
    checked = 0
    for entity in data['entities']:
        kind = entity['kind']
        if kind not in {'tree','shrub','log','rock','reed','decal'} or not entity.get('shown', True):
            continue
        checked += 1
        x,y = int(entity['tile_x']),int(entity['tile_y'])
        if not (0 <= x < width and 0 <= y < int(terrain['height'])):
            violations.append(f'invalid tile {kind}:{x},{y}')
            continue
        radius = 3 if kind == 'decal' else 4
        if any(max(abs(x-cx),abs(y-cy)) <= radius for cx,cy in centers):
            violations.append(f'clearing radius {radius}: {kind}:{x},{y}')
        code = codes[y*width+x]
        if kind == 'rock':
            rocks.append((x,y))
            rock_counts[code] = rock_counts.get(code,0)+1
            if code not in {2,3}:
                violations.append(f'rock terrain {code}:{x},{y}')
        if kind == 'decal':
            decals.append((x,y))
            decal_counts[code] = decal_counts.get(code,0)+1
    duplicates = {'rock':len(rocks)-len(set(rocks)), 'decal':len(decals)-len(set(decals))}
    densities = []
    for kind,code,minimum,maximum,count in [('rock',3,.04,.08,rock_counts[3]),('rock',2,.40,.50,rock_counts[2]),
            ('decal',0,0,.04,decal_counts[0]),('decal',1,0,.03,decal_counts[1])]:
        denominator = codes.count(code)
        share = count/denominator if denominator else None
        passed = share is not None and minimum <= share <= maximum
        densities.append({'kind':kind,'terrain_code':code,'count':count,'all_tiles':denominator,'share':share,'pass':passed})
    atlas = json.loads((repo/'assets/art/atlas/world_atlas.json').read_text())['sprites']
    manifest = json.loads((repo/'assets/art/manifest.json').read_text())['assets']
    installed = {key for key in atlas if key.startswith('N3_')}
    records = {entry['sprite_id']:entry for entry in manifest if entry.get('sprite_id') in installed}
    native = []
    for ident in sorted(installed):
        entry = records.get(ident)
        if entry is None or not entry.get('native_path'):
            violations.append('missing native provenance: '+ident)
            continue
        source = repo/entry['native_path']
        if not source.is_file():
            violations.append('missing native PNG: '+ident)
            continue
        with Image.open(source) as image:
            size = image.size
        limit = 6 if ident in {'N3_14','N3_15'} else 8
        passed = min(size)>0 and max(size)<=limit and list(size)==entry.get('native_size')
        native.append({'id':ident,'size':size,'limit':limit,'pass':passed})
    passed = checked>0 and bool(installed) and not violations and not any(duplicates.values()) and all(row['pass'] for row in densities+native)
    detail = {'tick':data['tick'],'checked_placements':checked,'clearing_centers':len(centers),
        'violations':violations,'duplicate_tiles':duplicates,'densities':densities,'native':native,
        'installed_decals':len(installed),'scope':'all shown placements, including offscreen; density denominator is every terrain tile'}
    return result('A15',passed,json.dumps(detail))
