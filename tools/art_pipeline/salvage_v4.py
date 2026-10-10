# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/salvage_v4.py
"""Read-only raw recovery and review-pending, attempt-coherent v4 selection."""
from __future__ import annotations

import argparse
from copy import deepcopy
import hashlib
import json
from itertools import product
from pathlib import Path
from statistics import mean
from typing import TypedDict, NotRequired

import numpy as np
from PIL import Image

from palette_tools import Color, rgb_to_lab
from pixel_normalize import outline
from pixel_normalize_v3 import RecoveryMetadata, recover_cell_v3, quantize_native
from pixel_normalize_v4 import fit_cell_v4
from procedural_animation import animate
from production_sheets import SHEETS
from v3_semantics import semantic_counts
from v4_contracts import (CYCLES, FrameCandidate, FrameSpec, GuardReport, WorldScale,
                          guard_chosen, guard_frame, guard_group, plan_factors, target_size)


class GuardData(TypedDict):
    errors: list[str]
    warnings: list[str]
    passed: bool


class CandidateRecord(TypedDict):
    id: str
    slot_id: str
    sheet: str
    cell: int
    attempt: int
    body: str
    view: str
    activity: str
    frame: int
    name: str
    source_kind: str
    source_candidate: NotRequired[str]
    derivation_rule: NotRequired[str]
    raw_path: str
    raw_box: list[int]
    images: dict[str,str]
    factor: float | None
    factor_source: NotRequired[str]
    grid: NotRequired[dict]
    metrics: dict
    guards: GuardData
    group_guards: NotRequired[GuardData]
    cross_guards: NotRequired[GuardData]
    review_status: str
    conversion_error: str | None


def save_image(image: Image.Image, path: Path) -> None:
    """Publish complete PNGs so concurrent review readers never see partial writes."""
    temporary=path.with_suffix('.tmp')
    image.save(temporary,format='PNG');temporary.replace(path)


def guard_data(report: GuardReport) -> GuardData:
    return {'errors':list(report.errors),'warnings':list(report.warnings),'passed':report.passed}


def frame_spec(record: CandidateRecord, size: tuple[int,int] = (1,1)) -> FrameSpec:
    return FrameSpec(record['sheet'],record['cell'],record['attempt'],record['body'],
                     record['view'],record['activity'],size)


def guarded_candidate(record: CandidateRecord) -> FrameCandidate:
    with Image.open(record['images']['native']) as source:
        image=source.convert('RGBA')
    metrics=record['metrics']
    return FrameCandidate(frame_spec(record,image.size),image,metrics['mean_delta_e'],
                          metrics['palette_match_percent'],metrics['binary_alpha'],
                          metrics['berry_texels'],metrics['fire_texels'])


def _derive(target: CandidateRecord, passing: list[CandidateRecord], cache: Path,
            palette: list[Color], scale: WorldScale) -> CandidateRecord:
    """Complete only from this attempt/activity/view; retain the raw target failure."""
    same_phase=[r for r in passing if r['frame']%2==target['frame']%2]
    source=min(same_phase or passing,key=lambda r:(abs(r['frame']-target['frame']),r['cell']))
    record=deepcopy(target)
    record['id']=target['id']+':derived'
    record['source_kind']='derived';record['source_candidate']=source['id']
    with Image.open(source['images']['native']) as opened:
        image=opened.convert('RGBA')
    if target['activity'] in CYCLES and same_phase:
        rule='same_cycle_phase_copy'
    else:
        rule='procedural_animation_from_same_group'
        image=animate(image,target['activity'],target['frame'],target['view'])
        box=image.getbbox()
        if box is not None:image=image.crop(box)
    record['derivation_rule']=rule
    folder=cache/record['id'].replace('@','_').replace(':','_');folder.mkdir(parents=True,exist_ok=True)
    dark=palette[int(np.argmin(rgb_to_lab(np.asarray(palette,dtype=np.uint8))[:,0]))]
    save_image(image,folder/'native.png');save_image(outline(image,dark),folder/'sprite.png')
    record['images']=dict(source['images'],native=str(folder/'native.png'),sprite=str(folder/'sprite.png'))
    record['raw_path']=source['raw_path'];record['raw_box']=list(source['raw_box'])
    record['factor']=source['factor'];record['metrics']=deepcopy(source['metrics'])
    for field in ('grid','factor_source'):
        if field in source:record[field]=deepcopy(source[field])
        else:record.pop(field,None)
    pixels=np.asarray(image);opaque=pixels[...,3]==255
    berry,fire=semantic_counts(image)
    record['metrics'].update(native_after=list(image.size),outlined_canvas=list(outline(image,dark).size),berry_texels=berry,fire_texels=fire,
        binary_alpha=bool(np.isin(pixels[...,3],[0,255]).all()),
        palette_match_percent=100.*sum(tuple(map(int,p)) in set(palette) for p in pixels[opaque,:3])/max(1,int(opaque.sum())),
        fidelity_source=source['id'],fidelity_inherited=True)
    record['guards']=guard_data(guard_frame(guarded_candidate(record),scale))
    record['guards']['warnings']=list(dict.fromkeys((*source['guards']['warnings'],
        *record['guards']['warnings'],'derived_fidelity_inherited')))
    record['conversion_error']=None;record['review_status']='pending'
    return record


def select_candidates(records: list[CandidateRecord], cache: Path, palette: list[Color],
                      scale: WorldScale = WorldScale()) -> dict:
    """Select candidates for review only; preserve all failures and derived lineage."""
    selected: dict[str,str]={};groups=[]
    objects: dict[str,list[CandidateRecord]]={}
    actors: dict[tuple[str,str,str,str],dict[int,list[CandidateRecord]]]={}
    for record in list(records):
        if record['source_kind']!='generated':continue
        if record['body']:
            key=(record['body'],record['sheet'],record['view'],record['activity'])
            actors.setdefault(key,{}).setdefault(record['attempt'],[]).append(record)
        elif record['guards']['passed']:
            objects.setdefault(record['slot_id'],[]).append(record)
    for slot,options in objects.items():
        chosen=min(options,key=lambda r:(len(r['guards']['warnings']),r['metrics']['mean_delta_e'],r['attempt']))
        selected[slot]=chosen['id']
    for key,attempts in actors.items():
        eligible=[]
        for attempt,original in attempts.items():
            passing=[r for r in original if r['guards']['passed']]
            completed=list(passing);derived=[]
            if passing:
                for missing in original:
                    if not missing['guards']['passed']:
                        replacement=_derive(missing,passing,cache,palette,scale)
                        derived.append(replacement)
                        if replacement['guards']['passed']:completed.append(replacement)
            report=guard_group([guarded_candidate(r) for r in completed],scale)
            if len(completed)!=len(original):
                report=GuardReport(tuple(dict.fromkeys((*report.errors,'incomplete_group'))),report.warnings)
            for record in original+derived:
                record['group_guards']=guard_data(report)
                if not record['guards']['passed'] or not report.ready_for_review:record['review_status']='ineligible'
            records.extend(derived)
            group={'body':key[0],'sheet':key[1],'view':key[2],'activity':key[3],'attempt':attempt,
                   'passing_original_count':len(passing),'mean_delta_e':mean(r['metrics']['mean_delta_e'] for r in passing) if passing else None,
                   'candidate_ids':[r['id'] for r in completed],'guards':guard_data(report),'selected':False}
            groups.append(group)
            if report.ready_for_review:eligible.append((group,completed))
        if eligible:
            group,completed=min(eligible,key=lambda pair:(-pair[0]['passing_original_count'],pair[0]['mean_delta_e'],pair[0]['attempt']))
            group['selected']=True
            selected.update({r['slot_id']:r['id'] for r in completed})
    lookup={r['id']:r for r in records}
    initial_object_selection={slot:identity for slot,identity in selected.items() if not lookup[identity]['body']}
    bush_options=[objects.get(f'N2_{cell:02d}',[]) for cell in range(1,5)]
    valid_bush_sets=[]
    for combination in product(*bush_options):
        if guard_chosen(bushes=[guarded_candidate(r) for r in combination]).passed:
            valid_bush_sets.append(combination)
    if valid_bush_sets:
        best=min(valid_bush_sets,key=lambda combination:(sum(len(r['guards']['warnings']) for r in combination),
                 sum(r['metrics']['mean_delta_e'] for r in combination),tuple(r['attempt'] for r in combination)))
        selected.update({r['slot_id']:r['id'] for r in best})
        for slot in ('S2_10','S2_11'):
            passing=[r for r in objects.get(slot,[]) if guard_chosen(
                bushes=[guarded_candidate(b) for b in best],baskets=[guarded_candidate(r)]).passed]
            if passing:
                selected[slot]=min(passing,key=lambda r:(len(r['guards']['warnings']),r['metrics']['mean_delta_e'],r['attempt']))['id']
    chosen=[lookup[identity] for identity in selected.values()]
    bushes=sorted([r for r in chosen if r['sheet']=='N2' and r['cell']<=4],key=lambda r:r['cell'])
    adults=[r for r in chosen if r['body'].startswith('adult')]
    children=[r for r in chosen if r['body']=='child']
    baskets=[r for r in chosen if r['sheet']=='S2' and r['cell'] in (10,11)]
    cross=guard_chosen(*[[guarded_candidate(r) for r in subset] for subset in (bushes,adults,children,baskets)])
    blocked=[]
    if 'berry_stage_order' in cross.errors:blocked+=bushes+baskets
    if 'basket_relative_size' in cross.errors or 'missing_chosen_bush_reference' in cross.errors:blocked+=baskets
    if 'child_relative_height' in cross.errors or 'missing_chosen_adult_reference' in cross.errors:blocked+=children
    for record in blocked:
        record['cross_guards']=guard_data(cross);record['review_status']='ineligible'
        selected.pop(record['slot_id'],None)
    initial_cross=guard_data(cross)
    final=[lookup[identity] for identity in selected.values()]
    cross=guard_chosen(
        bushes=[guarded_candidate(r) for r in sorted(final,key=lambda r:r['cell']) if r['sheet']=='N2' and r['cell']<=4],
        adults=[guarded_candidate(r) for r in final if r['body'].startswith('adult')],
        children=[guarded_candidate(r) for r in final if r['body']=='child'],
        baskets=[guarded_candidate(r) for r in final if r['sheet']=='S2' and r['cell'] in (10,11)])
    for group in groups:
        if group['selected'] and not any(identity in selected.values() for identity in group['candidate_ids']):
            group['selected']=False
    references={}
    for body in ('adult_a','adult_b','child'):
        walk=next((g for g in groups if g['selected'] and g['body']==body and g['view']=='S' and g['activity']=='walk'),None)
        idle=[r for r in records if r['body']==body and r['view']=='S' and r['activity']=='idle'
              and r['frame']==0 and r['guards']['passed'] and r.get('group_guards',{}).get('passed',False)]
        preferred=[r for r in idle if walk and r['attempt']==walk['attempt'] and r['sheet']==walk['sheet']]
        if preferred or idle:
            references[body]=min(preferred or idle,key=lambda r:(r['metrics']['mean_delta_e'],r['attempt']))['id']
    return {'selected':selected,'references':references,'groups':groups,'cross_guards':guard_data(cross),
            'initial_object_selection':initial_object_selection,'initial_cross_guards':initial_cross}


def run(root: Path, output: Path, cache: Path, scale: WorldScale) -> dict:
    inventory=json.loads((root/'docs/art/production2/raw_inventory.json').read_text())
    palette=[tuple(c['rgb']) for c in json.loads((root/'tools/art_pipeline/production_palette.json').read_text())['colors']]
    records: list[CandidateRecord]=[];measurements: dict[str,RecoveryMetadata]={};specs=[]
    for raw in inventory['raws']:
        path=Path(raw['path']);sheet=raw['sheet'];attempt=raw['attempt']
        source=None;raw_error=None
        try:
            if hashlib.sha256(path.read_bytes()).hexdigest()!=raw['sha256']:raise ValueError('raw_hash_mismatch')
            with Image.open(path) as opened:source=opened.convert('RGBA')
        except (OSError,ValueError) as error:raw_error=f'{type(error).__name__}: {error}'
        old_report=root/f'docs/art/production/sheets/{sheet}_attempt_{attempt}.json'
        old=json.loads(old_report.read_text()) if old_report.exists() else {}
        invalid_layout=old.get('manual_grid_review',{}).get('valid') is False
        for index,spec in enumerate(SHEETS[sheet]):
            cell=index+1;slot=f'{sheet}_{cell:02d}';identity=f'{slot}@{attempt}'
            folder=cache/identity.replace('@','_');folder.mkdir(parents=True,exist_ok=True)
            record=CandidateRecord(id=identity,slot_id=slot,sheet=sheet,cell=cell,attempt=attempt,
                body={'a':'adult_a','b':'adult_b','c':'child'}.get(sheet[2:3],'') if sheet.startswith('V_') else '',
                view=spec.view,activity=spec.activity,frame=spec.frame,name=spec.name,source_kind='generated',
                raw_path=str(path),raw_box=[],images={},factor=None,metrics={},
                guards=guard_data(GuardReport(('conversion_failure',))),review_status='ineligible',conversion_error=raw_error)
            records.append(record)
            if source is None:continue
            row,col=divmod(index,4)
            box=(col*source.width//4,row*source.height//4,(col+1)*source.width//4,(row+1)*source.height//4)
            record['raw_box']=list(box);cropped=source.crop(box);save_image(cropped,folder/'raw.png')
            record['images']['raw']=str(folder/'raw.png')
            try:
                native,metadata=recover_cell_v3(cropped)
                save_image(quantize_native(native,palette),folder/'unfitted.png')
                record['images']['unfitted']=str(folder/'unfitted.png')
                record['grid']={key:value for key,value in metadata.items() if key!='sample_rgba'}
                measurements[identity]=metadata;specs.append(frame_spec(record,native.size))
                record['guards']=guard_data(GuardReport(('invalid_grid_layout',) if invalid_layout else ()))
            except (ValueError,IndexError) as error:record['conversion_error']=f'{type(error).__name__}: {error}'
        print(f'measured {sheet} attempt {attempt}',flush=True)
    factors={(d.frame.sheet,d.frame.cell,d.frame.attempt):d for d in plan_factors(specs,scale)}
    for record in records:
        if record['id'] not in measurements:continue
        decision=factors[(record['sheet'],record['cell'],record['attempt'])]
        record['factor']=decision.factor;record['factor_source']=decision.source
        try:
            with Image.open(record['images']['raw']) as source:
                fit=fit_cell_v4(source,palette,decision.factor,icon_canvas=record['sheet']=='U1',measurement=measurements[record['id']],icon_size=target_size(decision.frame,scale)[0] if record['sheet']=='U1' else 16)
            folder=Path(record['images']['raw']).parent
            save_image(fit.native,folder/'native.png');save_image(fit.sprite,folder/'sprite.png')
            record['images'].update(native=str(folder/'native.png'),sprite=str(folder/'sprite.png'))
            if not np.isfinite(fit.metrics['mean_delta_e']):raise ValueError('no_common_opaque_fidelity')
            record['metrics']=dict(fit.metrics)
            guard=guard_frame(guarded_candidate(record),scale)
            errors=tuple(dict.fromkeys((*record['guards']['errors'],*guard.errors)))
            warnings=tuple(dict.fromkeys((*fit.metrics['warnings'],*decision.warnings,*guard.warnings)))
            record['guards']=guard_data(GuardReport(errors,warnings))
            record['review_status']='pending' if not errors else 'ineligible'
        except (ValueError,IndexError) as error:
            record['conversion_error']=f'{type(error).__name__}: {error}'
            record['guards']=guard_data(GuardReport(tuple(dict.fromkeys((*record['guards']['errors'],'conversion_failure')))))
            if record['sheet']=='U1' and 'Icon factor' in str(error):
                # Preserve the failed pre-canvas prototype for diagnosis, never selection.
                with Image.open(record['images']['raw']) as source:
                    prototype=fit_cell_v4(source,palette,decision.factor,measurement=measurements[record['id']])
                folder=Path(record['images']['raw']).parent
                save_image(prototype.native,folder/'native.png');save_image(prototype.sprite,folder/'sprite.png')
                record['images'].update(native=str(folder/'native.png'),sprite=str(folder/'sprite.png'))
                record['metrics']=dict(prototype.metrics,failed_icon_canvas=True)

    selection=select_candidates(records,cache,palette,scale)
    result={'schema_version':1,'scale':{'adult_height':scale.adult_height,'child_height':scale.child_height},
            'candidates':records,**selection,'acceptance_status':'pending_independent_visual_review',
            'source_inventory':str(root/'docs/art/production2/raw_inventory.json')}
    output.parent.mkdir(parents=True,exist_ok=True)
    temporary=output.with_suffix('.tmp')
    temporary.write_text(json.dumps(result,indent=2,allow_nan=False)+'\n');temporary.replace(output)
    print(f"Wrote {len(records)} candidates; {len(selection['selected'])} pending-review slots",flush=True)
    return result


def main() -> None:
    parser=argparse.ArgumentParser(description=__doc__)
    root=Path(__file__).resolve().parents[2]
    parser.add_argument('--output',type=Path,default=root/'docs/art/production2/candidates.json')
    parser.add_argument('--cache',type=Path,default=root/'.harness/evidence/art-production2/candidates')
    parser.add_argument('--escape-scale',action='store_true')
    args=parser.parse_args()
    run(root,args.output.resolve(),args.cache.resolve(),WorldScale(33,25) if args.escape_scale else WorldScale())


if __name__=='__main__':main()
