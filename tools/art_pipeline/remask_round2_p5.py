# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/remask_round2_p5.py
"""Apply one bounded support repair globally while preserving approved round-one pixels."""
from __future__ import annotations

from collections import Counter
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import numpy as np
from PIL import Image
from hair_masks_v5 import Reference, remask
from mask_inputs_p5 import ROOT, OUTPUT, image, load, save_image
from mask_boards_p5 import card
from remask_candidates_p5 import write
from salvage_verdicts import candidate_fingerprint


def main() -> None:
    """Freeze round one, recompute every input, and expose only rejected mask cards."""
    destination = OUTPUT / 'round2'
    if (destination / 'review_index.json').exists():
        raise FileExistsError(destination)
    rows = load(OUTPUT / 'candidates.json')['candidates']
    reviews = [row for n in (1,2,3) for row in load(OUTPUT/'round1'/f'reviews_{n}.json')['verdicts']]
    assert {row['candidate_id'] for row in reviews} == {row['id'] for row in rows}
    assert len(reviews) == len(rows)
    write(OUTPUT/'round1'/'final_reviews.json', {'verdicts':reviews,
          'review_sources_sha256':{f'reviews_{n}.json':hashlib.sha256((OUTPUT/'round1'/f'reviews_{n}.json').read_bytes()).hexdigest() for n in (1,2,3)}})
    reconsider = {row['candidate_id'] for row in reviews if row['verdict']=='reject' and row['reason']=='hair_mask_wrong'}
    references = [Reference(r['id'],r['body'],r['view'],r['activity'],image(r['sprite_path']),image(r['mask_path']))
                  for r in load(OUTPUT/'references.json')['references']]
    lookup = {row['id']:row for row in rows}
    generated = []
    assignments = []
    memo = {}
    changed = []
    for original in rows:
        row = deepcopy(original)
        row['id'] += ':r2'
        row['round1_candidate_id'] = original['id']
        canonical = lookup[original['mask_v5']['canonical_id']]
        if canonical['id'] not in memo:
            memo[canonical['id']] = remask(image(canonical['images']['sprite']),image(canonical['images']['native']),
                canonical['body'],canonical.get('canonical_view',canonical['view']),
                canonical.get('canonical_activity',canonical['activity']),references,support_down_fraction=.20)
        outcome = memo[canonical['id']]
        old = image(original['images']['sprite_hair_mask'])
        mask = outcome.mask or old
        folder = destination/'candidates'/row['id'].replace(':','_').replace('@','_')
        row['images']['sprite_hair_mask'] = save_image(mask,folder/'sprite_hair_mask.png')
        sprite = image(row['images']['sprite'])
        native = image(row['images']['native'])
        mx,my=(sprite.width-native.width)//2,(sprite.height-native.height)//2
        native_mask=np.array(mask.crop((mx,my,mx+native.width,my+native.height)))
        native_mask[...,3]=np.asarray(native)[...,3]
        row['images']['hair_mask']=save_image(Image.fromarray(native_mask),folder/'hair_mask.png')
        row['mask_v5']['support_down_fraction']=.20
        row['mask_v5']['hair_texels']=int(np.count_nonzero(np.asarray(mask)[...,0]==3))
        row['hair_mask_source']='template_v5' if outcome.mask is not None else 'p4_stored'
        generated.append(row)
        changed.append({'id':row['id'],'before':original['mask_v5']['hair_texels'],'after':row['mask_v5']['hair_texels'],
                        'changed_texels':int(np.count_nonzero(np.any(np.asarray(old)!=np.asarray(mask),axis=-1))),
                        'reviewed_again':original['id'] in reconsider})
        if original['id'] not in reconsider:
            continue
        reference = image(OUTPUT/'inputs'/('V_a1_01' if row['body']=='adult_a' else 'V_b1_01')/'sprite.png')
        board=card(row['id'],row['activity'],row['view'],sprite,mask,reference)
        path=destination/'boards'/(row['id'].replace(':','_').replace('@','_')+'.png')
        save_image(board,path)
        assignments.append({'sprite_id':row['slot_id'],'candidate_id':row['id'],'path':str(path.relative_to(ROOT)),
                            'sha256':hashlib.sha256(path.read_bytes()).hexdigest(),
                            'round1_raw_context':str((OUTPUT/'round1'/'boards'/(original['id'].replace(':','_').replace('@','_')+'.png')).relative_to(ROOT))})
    write(destination/'candidates.json',{'schema_version':1,'candidates':generated})
    write(destination/'review_index.json',{'candidate_fingerprints':{r['id']:candidate_fingerprint(r) for r in generated},'boards':assignments})
    for n,start in enumerate(range(0,len(assignments),18),1):
        write(destination/f'assignment_{n}.json',{'images':assignments[start:start+18]})
    write(destination/'change.json',{'change':'Extend T+ search support downward by round(0.20*opaque_height); no horizontal extension. All other round1 rules unchanged.',
          'reason':'Independent round1 reviewers found omitted lower long hair and beard; the three adult_b S references end at rows8-9. Preserve colour/native/component gates while admitting contiguous lower hair.',
          'round1_counts':dict(Counter(r['verdict'] for r in reviews)), 'applied':len(generated),'reviewed_again':len(assignments),'frames':changed,
          'installation_policy':'Round1 approvals retain exact round1 pixels; only fresh round2 ok pixels may replace them.'})
    print('Applied',len(generated),'re-review',len(assignments),'changed',sum(r['changed_texels']>0 for r in changed))


if __name__=='__main__':
    main()
