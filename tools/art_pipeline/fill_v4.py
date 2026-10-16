# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/fill_v4.py --help
"""Process arbitrary sixteen-cell fill sheets without installing or accepting art."""
from __future__ import annotations
import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path

import numpy as np
from PIL import Image

from fill_catalog import load_catalog, prompt_for_fill
from pixel_normalize_v3 import recover_cell_v3, quantize_native
from pixel_normalize_v4 import fit_cell_v4
from salvage_v4 import _derive, guard_data, guarded_candidate, save_image
from v4_contracts import (STANDING, FactorDecision, FrameSpec, WorldScale, guard_chosen,
                          guard_frame, guard_group, plan_factors, target_size)


def contract_spec(sheet: dict, cell: dict, attempt: int, size: tuple[int,int]) -> FrameSpec:
    alias=cell.get('budget_alias')
    return FrameSpec(alias['sheet'] if alias else sheet['id'],alias['cell'] if alias else cell['cell'],
                     attempt,sheet['body'],cell['view'],cell['activity'],size)


def fill_factors(specs: list[FrameSpec], reference: dict | None,
                 scale: WorldScale=WorldScale()) -> tuple[FactorDecision,...]:
    decisions=plan_factors(specs,scale)
    if specs and specs[0].body and not any(s.activity in STANDING for s in specs):
        if reference is None or reference.get('body')!=specs[0].body or not 0<float(reference['factor'])<=1:
            raise ValueError('Missing same-body reference source factor')
        return tuple(FactorDecision(d.frame,float(reference['factor']),f"final_reference_factor:{reference['id']}") for d in decisions)
    return decisions


def load_final_context(root: Path, final: dict) -> list[dict]:
    ledgers={};records={}
    for slot,entry in final['slots'].items():
        if entry['disposition'] not in ('accepted','derived'):continue
        path=root/entry['ledger']
        if str(path) not in ledgers:
            if hashlib.sha256(path.read_bytes()).hexdigest()!=entry['ledger_sha256']:raise ValueError('Final selection ledger hash mismatch')
            ledgers[str(path)]={r['id']:r for r in json.loads(path.read_text())['candidates']}
        record=deepcopy(ledgers[str(path)][entry['candidate_id']])
        if record['body'] or record['sheet']=='N2' and record['cell']<=4:
            record['source_kind']='reference';records[record['id']]=record
    for identity in final['references'].values():
        if identity not in records:raise ValueError(f'Final body reference unavailable: {identity}')
    return list(records.values())


def with_guard_context(ledger: dict, chosen: list[dict]) -> list[dict]:
    lookup={r['id']:r for r in ledger['candidates']}
    replaced={r.get('target_slot',r['slot_id']) for r in chosen}
    return chosen+[lookup[identity] for identity in ledger.get('guard_context',[])
                   if lookup[identity].get('target_slot',lookup[identity]['slot_id']) not in replaced]


def cross_report(records: list[dict]):
    return guard_chosen(
        bushes=[guarded_candidate(r) for r in sorted(records,key=lambda r:r['cell']) if r['sheet']=='N2' and r['cell']<=4],
        adults=[guarded_candidate(r) for r in records if r['body'].startswith('adult')],
        children=[guarded_candidate(r) for r in records if r['body']=='child'],
        baskets=[guarded_candidate(r) for r in records if r['sheet']=='S2' and r['cell'] in (10,11)])


def process(root: Path, catalog: dict, sheet_id: str, raw: Path, attempt: int, output: Path, cache: Path) -> dict:
    if output.exists():raise ValueError('Refusing to overwrite an existing fill candidate ledger')
    sheet=next(s for s in catalog['sheets'] if s['id']==sheet_id)
    final_path=root/catalog['source_selection']
    if hashlib.sha256(final_path.read_bytes()).hexdigest()!=catalog['source_selection_sha256']:raise ValueError('Fill plan final selection changed')
    final=json.loads(final_path.read_text());scale=WorldScale(**final['scale'])
    context=load_final_context(root,final);reference=next((r for r in context if r['id']==sheet.get('reference_candidate')),None)
    if sheet['body'] and (reference is None or reference['body']!=sheet['body']):raise ValueError('Missing matching body identity reference')
    palette=[tuple(c['rgb']) for c in json.loads((root/'tools/art_pipeline/production_palette.json').read_text())['colors']]
    with Image.open(raw) as opened:source=opened.convert('RGBA')
    records=[];measurements={};specs=[]
    for cell in sheet['cells']:
        physical=cell['cell'];slot=f'{sheet_id}_{physical:02d}';identity=f'{slot}@{attempt}'
        folder=cache/identity.replace('@','_');folder.mkdir(parents=True,exist_ok=True)
        row,col=divmod(physical-1,4);box=(col*source.width//4,row*source.height//4,(col+1)*source.width//4,(row+1)*source.height//4)
        crop=source.crop(box);save_image(crop,folder/'raw.png')
        alias=cell.get('budget_alias')
        record={'id':identity,'slot_id':slot,'target_slot':cell['target_slot'],'source_sheet':sheet_id,'source_cell':physical,
                'sheet':alias['sheet'] if alias else sheet_id,'cell':alias['cell'] if alias else physical,
                'budget_alias':alias,'attempt':attempt,'body':sheet['body'],'view':cell['view'],'activity':cell['activity'],
                'frame':cell['frame'],'name':cell['name'],'source_kind':'generated','raw_path':str(raw.resolve()),'raw_box':list(box),
                'images':{'raw':str(folder/'raw.png')},'factor':None,'metrics':{},
                'guards':{'errors':['conversion_failure'],'warnings':[],'passed':False},'conversion_error':None,'review_status':'ineligible'}
        records.append(record)
        try:
            native,metadata=recover_cell_v3(crop);save_image(quantize_native(native,palette),folder/'unfitted.png')
            record['images']['unfitted']=str(folder/'unfitted.png')
            record['grid']={k:v for k,v in metadata.items() if k!='sample_rgba'}
            measurements[identity]=metadata;specs.append(contract_spec(sheet,cell,attempt,native.size))
        except (ValueError,IndexError) as error:record['conversion_error']=f'{type(error).__name__}: {error}'
    decisions=fill_factors(specs,reference,scale)
    factors={record['id']:decision for record,decision in zip(
        (record for record in records if record['id'] in measurements),decisions)}
    for record in records:
        if record['id'] not in measurements:continue
        decision=factors[record['id']];record['factor']=decision.factor;record['factor_source']=decision.source
        try:
            with Image.open(record['images']['raw']) as crop:
                fit=fit_cell_v4(crop,palette,decision.factor,measurement=measurements[record['id']],
                    icon_canvas=(record['sheet']=='U1' or record['sheet']=='W1' and record['cell'] in (12,16)),icon_size=target_size(decision.frame,scale)[0])
            folder=Path(record['images']['raw']).parent;save_image(fit.native,folder/'native.png');save_image(fit.sprite,folder/'sprite.png')
            record['images'].update(native=str(folder/'native.png'),sprite=str(folder/'sprite.png'))
            if not np.isfinite(fit.metrics['mean_delta_e']):raise ValueError('no_common_opaque_fidelity')
            record['metrics']=dict(fit.metrics);report=guard_frame(guarded_candidate(record),scale)
            record['guards']=guard_data(report)
            record['guards']['warnings']=list(dict.fromkeys((*fit.metrics['warnings'],*decision.warnings,*report.warnings)))
            record['review_status']='pending' if report.passed else 'ineligible'
        except (ValueError,IndexError) as error:
            record['conversion_error']=f'{type(error).__name__}: {error}'
            if (record['sheet']=='U1' or record['sheet']=='W1' and record['cell'] in (12,16)) and 'Icon factor' in str(error):
                with Image.open(record['images']['raw']) as crop:
                    prototype=fit_cell_v4(crop,palette,decision.factor,measurement=measurements[record['id']])
                folder=Path(record['images']['raw']).parent
                save_image(prototype.native,folder/'native.png');save_image(prototype.sprite,folder/'sprite.png')
                record['images'].update(native=str(folder/'native.png'),sprite=str(folder/'sprite.png'))
                if np.isfinite(prototype.metrics['mean_delta_e']):record['metrics']=dict(prototype.metrics,failed_icon_canvas=True)
    selected={};groups=[];actor_groups={}
    for record in records:
        if record['body']:actor_groups.setdefault((record['view'],record['activity']),[]).append(record)
        elif record['guards']['passed']:selected[record['slot_id']]=record['id']
    for (view,activity),original in actor_groups.items():
        passing=[r for r in original if r['guards']['passed']];completed=list(passing)
        if passing and sheet.get('derive_missing', True):
            for missing in original:
                if not missing['guards']['passed']:
                    derived=_derive(missing,passing,cache,palette,scale);records.append(derived)
                    if derived['guards']['passed']:completed.append(derived)
        report=guard_group([guarded_candidate(r) for r in completed],scale)
        ready=report.ready_for_review and len(completed)==len(original)
        for record in original+completed:record['group_guards']=guard_data(report)
        groups.append({'body':sheet['body'],'sheet':sheet_id,'view':view,'activity':activity,'attempt':attempt,
                       'candidate_ids':[r['id'] for r in completed],'passing_original_count':len(passing),
                       'mean_delta_e':float(np.mean([r['metrics']['mean_delta_e'] for r in passing])) if passing else None,
                       'guards':guard_data(report),'selected':ready})
        if ready:selected.update({r['slot_id']:r['id'] for r in completed})
    ledger={'schema_version':1,'scale':final['scale'],'candidates':records+context,'selected':selected,
            'references':final['references'],'groups':groups,'guard_context':[r['id'] for r in context],
            'review_catalog':{'schema_version':1,'sheets':[sheet]},'acceptance_status':'pending_independent_visual_review',
            'raw_sha256':hashlib.sha256(raw.read_bytes()).hexdigest(),'attempted_cells':16,
            'checkpoint_attempted_cells':catalog['checkpoint_attempted_cells']}
    lookup={r['id']:r for r in records};cross=cross_report(with_guard_context(ledger,[lookup[i] for i in selected.values()]))
    ledger['cross_guards']=guard_data(cross)
    if not cross.passed:
        # Keep failed candidates visible; cross constraints never silently accept a fill.
        blocked=[]
        for slot,identity in selected.items():
            r=lookup[identity]
            if (r['body']=='child' and any(e in cross.errors for e in ('child_relative_height','missing_chosen_adult_reference'))
                or r['sheet']=='N2' and r['cell']<=4 and 'berry_stage_order' in cross.errors
                or r['sheet']=='S2' and r['cell'] in (10,11) and any('basket' in e or 'bush' in e or e=='berry_stage_order' for e in cross.errors)):
                r['cross_guards']=guard_data(cross);blocked.append(slot)
        for slot in blocked:selected.pop(slot)
    output.parent.mkdir(parents=True,exist_ok=True);temporary=output.with_suffix('.tmp')
    temporary.write_text(json.dumps(ledger,indent=2,allow_nan=False)+'\n');temporary.replace(output)
    return ledger


def checkpoint(ledgers: list[dict], adjudications: list[dict], baseline: dict,
               installation_holds: set[str] | None=None) -> dict:
    """Six calls always mean96 attempted cells; duplicates and derived frames do not inflate success."""
    if len(ledgers)!=6 or len(adjudications)!=6:raise ValueError('Checkpoint requires all six call results')
    targets=set();holds=installation_holds or set()
    for ledger,report in zip(ledgers,adjudications):
        if ledger['attempted_cells']!=16:raise ValueError('Every generated call contributes sixteen attempted cells')
        lookup={r['id']:r for r in ledger['candidates']}
        for identity in report['accepted'].values():
            record=lookup[identity];target=record.get('target_slot',record['slot_id'])
            prior=baseline['slots'].get(target,{})
            if record['source_kind']=='generated' and (prior.get('disposition')!='accepted' or prior.get('candidate_id') in holds):
                targets.add(target)
    return {'attempted_cells':96,'newly_accepted_unique_targets':sorted(targets),
            'newly_accepted_count':len(targets),'acceptance_fraction':len(targets)/96,
            'stop_generation':len(targets)<48}


def main() -> None:
    root=Path(__file__).resolve().parents[2];parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command',choices=['validate','prompt','process','checkpoint'])
    parser.add_argument('--catalog',type=Path,default=root/'docs/art/production2/fill_sheets.json')
    parser.add_argument('--sheet');parser.add_argument('--raw',type=Path);parser.add_argument('--attempt',type=int,default=1)
    parser.add_argument('--output',type=Path);parser.add_argument('--cache',type=Path,default=root/'.harness/evidence/art-production2/fills')
    parser.add_argument('--ledgers',type=Path,nargs='+');parser.add_argument('--adjudications',type=Path,nargs='+')
    args=parser.parse_args();catalog=load_catalog(args.catalog)
    if args.command=='validate':print(f"Validated {len(catalog['sheets'])} sixteen-cell sheets");return
    if args.command=='checkpoint':
        if not args.ledgers or not args.adjudications:parser.error('--ledgers and --adjudications are required')
        if len(set(args.ledgers))!=6:parser.error('Provide six distinct call ledgers')
        result=checkpoint([json.loads(p.read_text()) for p in args.ledgers],
            [json.loads(p.read_text()) for p in args.adjudications],json.loads((root/catalog['source_selection']).read_text()),
            set(catalog.get('installation_holds',{})))
        output=args.output or root/'docs/art/production2/fills/checkpoint6.json'
        output.parent.mkdir(parents=True,exist_ok=True);output.write_text(json.dumps(result,indent=2)+'\n')
        print(f"Checkpoint: {result['newly_accepted_count']}/96 new targets; stop={result['stop_generation']}");return
    if not args.sheet:parser.error('--sheet is required')
    if args.command=='prompt':print(prompt_for_fill(next(s for s in catalog['sheets'] if s['id']==args.sheet)));return
    if not args.raw:parser.error('--raw is required')
    output=args.output or root/f'docs/art/production2/fills/{args.sheet}_attempt_{args.attempt}.json'
    ledger=process(root,catalog,args.sheet,args.raw,args.attempt,output,args.cache.resolve())
    print(f"Saved sixteen raw records, {len(ledger['selected'])} pending-review cells to {output}")


if __name__=='__main__':main()
