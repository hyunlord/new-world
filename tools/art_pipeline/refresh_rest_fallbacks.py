# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/refresh_rest_fallbacks.py
"""Bounded final-QA repair of rest fallbacks; accepted source pixels stay intact."""
from __future__ import annotations
import json
from pathlib import Path
import numpy as np
from PIL import Image
from install_fallbacks import _native, _ascii
from procedural_animation import animate
from pixel_normalize import outline
from palette_tools import rgb_to_lab


def refresh(root: Path) -> list[str]:
    path=root/'assets/art/manifest.json';manifest=json.loads(path.read_text())
    entries={e['sprite_id']:e for e in manifest['assets'] if e.get('sprite_id')}
    palette=[tuple(c['rgb']) for c in json.loads((root/'tools/art_pipeline/production_palette.json').read_text())['colors']]
    dark=palette[int(np.argmin(rgb_to_lab(np.asarray(palette))[:,0]))]
    changes=[]
    for sprite_id,entry in entries.items():
        if entry.get('qa_status')!='fallback':continue
        asleep=sprite_id in ('V_a3_13','V_b3_13')
        if not asleep and entry.get('activity') not in ('sit','hug'):continue
        if asleep:
            source=entries['V_b1_16'];body=_native(root,source)
            if source['qa_status']!='accepted' or body.width>14 or body.height>28:
                raise ValueError('Accepted retained adult frame no longer fits exact rotation')
            rotated=body.transpose(Image.Transpose.ROTATE_270)
            native=Image.new('RGBA',(28,14));fur=_ascii('log',palette,(28,14))
            native.alpha_composite(fur,((28-fur.width)//2,14-fur.height))
            native.alpha_composite(rotated,(0,0))
            entry['source_sprite']='V_b1_16'
            entry['source']='procedural:exact90_rotation_accepted_N_walk_with_fur'
            entry['limitations']=['Uses accepted adult B N-view passing walk frame V_b1_16, not requested S idle; body and view substitution declared.']
            if sprite_id=='V_a3_13':entry['limitations'].append('Adult A asleep uses adult B source body; identity fidelity remains limited.')
        else:
            source=entries[entry['source_sprite']]
            native=animate(_native(root,source),entry['activity'],entry.get('frame',0),entry.get('view','S'))
            entry['source']='procedural:animation:'+entry['activity']+':joined_leg_rows'
        sprite=outline(native,dark)
        native.save(root/entry['native_path']);sprite.save(root/entry['path'])
        entry.update(native_size=list(native.size),canvas_size=list(sprite.size),anchor=[sprite.width//2,sprite.height-1])
        for record in manifest['assets']:
            if record['path']==entry['native_path']:
                record.update(source=entry['source'],native_size=list(native.size),anchor=[native.width//2,native.height-1])
        changes.append(sprite_id)
    path.write_text(json.dumps(manifest,indent=2)+'\n')
    (root/'docs/art/production/rest_fallback_refresh.json').write_text(json.dumps({'changed':changes,'accepted_rotation_source':'V_b1_16','rotation':'90 degrees clockwise; native28x13 preserved on28x14furcanvas'},indent=2)+'\n')
    return changes

if __name__=='__main__':print(json.dumps(refresh(Path.cwd())))
