# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 -c "import sys;sys.path.insert(0,'tools/art_pipeline');import mask_installation_p5 as m;m.prepare()"
"""Stage current-animation review and freeze approved mask-only replacements."""
from __future__ import annotations

import hashlib
import re
from pathlib import Path
from PIL import Image
from hair_registry_p4 import HairRegistry
from harvest_boards_p5 import boards, completion_id
from harvest_p5 import Completion
from mask_inputs_p5 import ROOT, OUTPUT, image, load
from remask_candidates_p5 import write
from salvage_verdicts import candidate_fingerprint


def physical_set(row: dict, manifest: dict, seen: frozenset[str] = frozenset()) -> str:
    """Follow borrowed-view provenance to its physical generation, rejecting cycles."""
    target=row['sprite_id']
    if target in seen:
        return 'unknown:cycle'
    source=row.get('source','')
    match=re.match(r'generated:([^/]+)/[^@]+@(\d+)',source)
    if match:
        return f'{match[1]}@{match[2]}'
    parent=row.get('source_sprite')
    if parent and parent in manifest:
        return physical_set(manifest[parent],manifest,seen|{target})
    candidate_parent=next((entry for entry in manifest.values() if parent and entry.get('candidate_id')==parent),None)
    if candidate_parent is not None:
        return physical_set(candidate_parent,manifest,seen|{target})
    return 'unknown:'+source


def prepare() -> None:
    """Stage every current frame of each group touched by approved c masks."""
    records={r['id']:r for r in load(OUTPUT/'final_candidates.json')['candidates']}
    verdicts=load(OUTPUT/'final_reviews.json')['verdicts']
    approved=[records[v['candidate_id']] for v in verdicts if v['verdict']=='ok' and records[v['candidate_id']]['mask_scope']=='c']
    manifest={r['sprite_id']:r for r in load(ROOT/'assets/art/manifest.json')['assets'] if r.get('sprite_id') and r.get('drawable',True)}
    bindings={r['frame_id']:r for r in load(OUTPUT/'audit_binding.json')['frames']}
    prior={r['frame_id']:r['from'] for r in load(ROOT/'docs/art/production4/natural_audit/apply_report.json')['changes']}
    affected={(r['body'],r['view'],r['activity']) for r in approved}
    groups=[]; frames=[]
    for group in sorted(affected):
        members=sorted([r for r in manifest.values() if (r.get('body'),r.get('view'),r.get('activity'))==group],key=lambda r:(r['frame'],r['sprite_id']))
        physical=sorted({physical_set(r,manifest) for r in members})
        review_rows=[]
        for row in members:
            review_rows.append(dict(id=row['sprite_id'],body=row['body'],view=row['view'],activity=row['activity'],frame=row['frame'],
                source_kind='derived' if row['source'].startswith('derived:') else 'generated',images={'sprite':row['path']},
                source_candidate=row.get('source_sprite','')))
        completion=Completion('C0',tuple(review_rows),sum(r['source_kind']=='derived' for r in review_rows))
        reference=image(OUTPUT/'inputs'/'V_b1_01'/'sprite.png')
        paths=boards(completion,reference,OUTPUT/'animation')
        identity=completion_id(completion)
        groups.append({'id':identity,'group':list(group),'physical_sets':physical,'single_physical_set':len(physical)==1 and not physical[0].startswith('unknown:'),
                       'frames':[r['sprite_id'] for r in members],
                       'frame_rgba_sha256':{r['sprite_id']:hashlib.sha256(image(r['path']).tobytes()).hexdigest() for r in members},**paths,
                       'board_sha256':hashlib.sha256(Path(paths['board']).read_bytes()).hexdigest(),
                       'gif_sha256':hashlib.sha256(Path(paths['gif']).read_bytes()).hexdigest()})
    for record in approved:
        target=record['slot_id']; current=manifest[target]; binding=bindings[target]
        frames.append({'target':target,'review_candidate_id':record['id'],'ledger':'docs/art/production5/masks/final_candidates.json',
                       'review_index':'docs/art/production5/masks/final_review_index.json','review_path':'docs/art/production5/masks/final_reviews.json',
                       'sprite_path':record['images']['sprite'],'mask_path':record['images']['sprite_hair_mask'],
                       'audited_rgba_sha256':binding['audited_rgba_sha256'],'prior_status':prior.get(target,current['qa_status']),
                       'current_status':current['qa_status'],'group':[record['body'],record['view'],record['activity']]})
    write(OUTPUT/'installation.json',{'frames':frames,'groups':groups,'manifest_sha256':hashlib.sha256((ROOT/'assets/art/manifest.json').read_bytes()).hexdigest()})
    write(OUTPUT/'animation/assignment.json',{'images':[{key:g[key] for key in ('id','group','board','gif')} for g in groups]})
    print('c approved',len(frames),'groups',len(groups),'single physical',sum(g['single_physical_set'] for g in groups))


def freeze(review_path: Path = OUTPUT/'animation/final_reviews.json') -> None:
    """Freeze only exact current pixels with both independent frame and group approvals."""
    report=load(OUTPUT/'installation.json')
    reviews={r['id']:r for r in load(review_path)['verdicts']}
    groups={tuple(g['group']):g for g in report['groups']}
    records={r['id']:r for r in load(OUTPUT/'final_candidates.json')['candidates']}
    index=load(OUTPUT/'final_review_index.json')['candidate_fingerprints']
    verdicts={r['candidate_id']:r for r in load(OUTPUT/'final_reviews.json')['verdicts']}
    manifest={r['sprite_id']:r for r in load(ROOT/'assets/art/manifest.json')['assets'] if r.get('sprite_id')}
    registry_path=ROOT/'tools/art_pipeline/reviewed_hair_maps_p5.json'
    registry=load(registry_path) if registry_path.exists() else {'version':1,'frames':{}}
    proposals=[]; decisions=[]
    for item in report['frames']:
        group=groups[tuple(item['group'])]
        reason=''
        if not group['single_physical_set']:
            reason='mixed_or_unknown_existing_physical_sets'
        elif reviews.get(group['id'],{}).get('verdict')!='ok':
            reason='animation_not_approved'
        current_members=[r for r in manifest.values() if r.get('drawable',True) and (r.get('body'),r.get('view'),r.get('activity'))==tuple(group['group'])]
        if ({r['sprite_id'] for r in current_members}!=set(group['frames']) or
                any(hashlib.sha256(image(r['path']).tobytes()).hexdigest()!=group['frame_rgba_sha256'].get(r['sprite_id']) for r in current_members)):
            reason='reviewed_animation_changed'
        record=records[item['review_candidate_id']]
        if index[record['id']]!=candidate_fingerprint(record) or verdicts[record['id']]['verdict']!='ok':
            raise ValueError('Mask candidate binding changed: '+record['id'])
        sprite=image(record['images']['sprite']);current=image(manifest[item['target']]['path'])
        if sprite.size!=current.size or sprite.tobytes()!=current.tobytes():
            reason='current_rgba_changed'
        if reason:
            decisions.append(dict(item,installed=False,reason=reason));continue
        mask=image(item['mask_path'])
        path=ROOT/'tools/art_pipeline/hair_masks_p5'/(item['target']+'.png')
        row={'body':record['body'],'activity':record['activity'],'size':list(mask.size),'source':'template_v5','verdict':'ok',
             'mask_path':str(path.relative_to(ROOT)),'mask_sha256':hashlib.sha256((ROOT/item['mask_path']).read_bytes()).hexdigest(),
             'rgba_sha256':hashlib.sha256(sprite.tobytes()).hexdigest(),'candidate_id':record['id'],
             'candidate_fingerprint':index[record['id']],'review_path':item['review_path'],
             'review_sha256':hashlib.sha256((ROOT/item['review_path']).read_bytes()).hexdigest(),
             'animation_review_path':str(review_path.relative_to(ROOT)),'animation_id':group['id'],
             'hair_texels':mask.getchannel('R').histogram()[3]}
        proposals.append((item['target'],path,(ROOT/item['mask_path']).read_bytes(),row))
        decisions.append(dict(item,installed=True,reason=''))
    for target,path,data,row in proposals:
        path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(data)
        registry['frames'][target]=row
        for other,prior in list(registry['frames'].items()):
            if other!=target and (prior['body'],prior['activity'],prior['rgba_sha256'])==(row['body'],row['activity'],row['rgba_sha256']):
                registry['frames'][other]=dict(row,inherited_mask_from=target)
    write(registry_path,registry)
    HairRegistry.load(ROOT)
    write(OUTPUT/'freeze_report.json',{'frames':decisions,'groups':[dict(g,verdict=reviews.get(g['id'],{}).get('verdict','missing'),review_path=str(review_path.relative_to(ROOT))) for g in report['groups']],
          'frozen':len(proposals),'blocked':len(decisions)-len(proposals)})
    print('frozen c masks',len(proposals),'blocked',len(decisions)-len(proposals))
