# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/merge_fill_selection.py --help
"""Build a separate, provenance-bound installation selection from reviewed fills."""
from __future__ import annotations

import argparse
from copy import deepcopy
from dataclasses import dataclass
import json
from pathlib import Path

from fill_v4 import cross_report
from production_sheets import SHEETS
from salvage_v4 import guarded_candidate
from salvage_verdicts import atomic_json, candidate_fingerprint, sha
from v4_contracts import WorldScale, guard_frame, guard_group


@dataclass(frozen=True, slots=True)
class FillInput:
    ledger: Path
    adjudication: Path
    verdict: Path


def semantic(record: dict) -> dict:
    """Keep installer descriptors independent of physical fill positions."""
    return {key:record[key] for key in ('body','view','activity','frame','name')}


def group_key(record: dict) -> tuple[str,str,str]:
    """Characters are grouped across sheets by their runtime activity."""
    return record['body'],record['view'],record['activity']


def merge_selection(root: Path, baseline_path: Path, fills: list[FillInput],
                    installation_holds: dict[str,str]) -> dict:
    """Validate provenance, replace whole groups, then fail closed on world guards."""
    def read(path: Path) -> dict:
        return json.loads(path.read_text())

    def resolve(path: str | Path) -> Path:
        return root/Path(path)

    baseline=read(baseline_path); result=deepcopy(baseline)
    slots=result['slots']; records={}; metadata={}; cache={}; audit=[]
    scale=WorldScale(**baseline['scale'])
    for slot,entry in slots.items():
        path=resolve(entry['ledger']); verdict=resolve(entry['verdict'])
        if sha(path)!=entry['ledger_sha256'] or sha(verdict)!=entry['verdict_sha256']:
            raise ValueError(f'Baseline provenance changed: {slot}')
        if path not in cache:cache[path]={r['id']:r for r in read(path)['candidates']}
        identity=entry.get('candidate_id')
        if identity:
            record=cache[path][identity];metadata[slot]=semantic(record)
            if identity not in installation_holds and entry['disposition'] in ('accepted','derived'):
                records[slot]=record
            else:
                entry.update(candidate_id=None,disposition='unfilled',reason='installation_hold')
    # Include missing original frames when deciding whether a baseline group is complete.
    for sheet,definition in SHEETS.items():
        for index,cell in enumerate(definition,1):
            body=next((body for prefix,body in (('V_a','adult_a'),('V_b','adult_b'),('V_c','child')) if sheet.startswith(prefix)),'')
            metadata.setdefault(f'{sheet}_{index:02d}',dict(body=body,view=cell.view,activity=cell.activity,frame=cell.frame,name=cell.name))
    original_metadata=deepcopy(metadata)
    options={}
    for bundle in fills:
        ledger_path=resolve(bundle.ledger); verdict_path=resolve(bundle.verdict); report_path=resolve(bundle.adjudication)
        ledger=read(ledger_path); review=read(verdict_path); report=read(report_path)
        if ledger['scale']!=baseline['scale']:raise ValueError('Mixed world scales')
        if review['candidates_sha256']!=sha(ledger_path) or report['authoritative_sha256']!=sha(verdict_path):
            raise ValueError('Fill review or adjudication hash mismatch')
        lookup={r['id']:r for r in ledger['candidates']}
        for record in ledger['candidates']:
            if record.get('target_slot') and record['source_kind'] in ('generated','derived'):
                target=record['target_slot'];metadata[target]=semantic(record)
                slots.setdefault(target,dict(candidate_id=None,disposition='unfilled',reason='no_approved_fill',
                    ledger=str(ledger_path),ledger_sha256=sha(ledger_path),
                    verdict=str(verdict_path),verdict_sha256=sha(verdict_path)))
        approvals={(v['sprite_id'],v['candidate_id']) for v in review['verdicts'] if v['verdict']=='ok'}
        groups={}
        for physical,identity in report['accepted'].items():
            record=lookup[identity]
            if record['slot_id']!=physical or ledger['selected'].get(physical)!=identity or (physical,identity) not in approvals:
                raise ValueError('Adjudication lacks current selected visual approval')
            if review['candidate_fingerprints'].get(physical)!=candidate_fingerprint(record):
                raise ValueError('Reviewed candidate pixels changed')
            target=record['target_slot']; metadata[target]=semantic(record)
            if identity in installation_holds:continue
            if record['guards']['errors'] or not guard_frame(guarded_candidate(record),scale).passed:continue
            key=group_key(record) if record['body'] else ('',target,'')
            source=(record.get('source_sheet',record['sheet']),record['attempt'])
            groups.setdefault((key,source),[]).append(record)
        for (key,source),members in groups.items():
            if len({r['target_slot'] for r in members})!=len(members):raise ValueError('Duplicate target in fill group')
            if key[0] and not guard_group([guarded_candidate(r) for r in members],scale).ready_for_review:continue
            entry=dict(ledger=str(ledger_path),ledger_sha256=sha(ledger_path),verdict=str(verdict_path),
                       verdict_sha256=sha(verdict_path),adjudication=str(report_path),adjudication_sha256=sha(report_path))
            options.setdefault(key,[]).append((members,entry,source))
    for key,alternatives in sorted(options.items()):
        expected={slot for slot,meta in metadata.items() if group_key(meta)==key} if key[0] else {key[1]}
        original_expected={slot for slot,meta in original_metadata.items() if group_key(meta)==key} if key[0] else {key[1]}
        if original_expected and all(slot in records and slots[slot]['disposition']=='accepted' for slot in original_expected):continue
        members,provenance,source=min(alternatives,key=lambda option:(-len(option[0]),
            sum(r['metrics']['mean_delta_e'] for r in option[0])/len(option[0]),option[2]))
        for target in expected:
            records.pop(target,None)
            slots.setdefault(target,dict(provenance))
            slots[target].update(candidate_id=None,disposition='unfilled',reason='replaced_semantic_group')
        for record in members:
            target=record['target_slot']; records[target]=record
            slots[target]=dict(provenance,candidate_id=record['id'],
                disposition='derived' if record['source_kind']=='derived' else 'accepted')
        audit.append({'group':list(key),'source':list(source),'targets':[r['target_slot'] for r in members]})
    def exclude(target: str, reasons: list[str]) -> None:
        records.pop(target,None)
        slots[target].update(candidate_id=None,disposition='unfilled',reason=';'.join(reasons))
    groups={}
    for target,record in list(records.items()):
        report=guard_frame(guarded_candidate(record),scale)
        errors=list(record['guards']['errors'])+list(report.errors)
        if errors:exclude(target,errors);continue
        if record['body']:groups.setdefault(group_key(record),[]).append(target)
    for targets in groups.values():
        report=guard_group([guarded_candidate(records[target]) for target in targets],scale)
        if not report.ready_for_review:
            for target in targets:exclude(target,list(report.errors+report.warnings))
    cross=cross_report(list(records.values()))
    for target,record in list(records.items()):
        errors=cross.errors
        blocked=(('berry_stage_order' in errors and (record['sheet']=='N2' and record['cell']<=4 or record['sheet']=='S2' and record['cell'] in (10,11)))
            or (any(e in errors for e in ('basket_relative_size','missing_chosen_bush_reference')) and record['sheet']=='S2' and record['cell'] in (10,11))
            or (any(e in errors for e in ('child_relative_height','missing_chosen_adult_reference')) and record['body']=='child'))
        if blocked:exclude(target,list(errors))
    final_cross=cross_report(list(records.values()))
    if not final_cross.passed:raise ValueError(f'Unresolved combined guards: {final_cross.errors}')
    for target,entry in slots.items():
        if target in metadata:
            entry['semantic']=metadata[target]
            entry.update(metadata[target])
    result['accepted']={target:record['id'] for target,record in records.items()}
    result['installation_holds']=dict(installation_holds)
    result['merge_provenance']={'baseline':str(baseline_path),'baseline_sha256':sha(baseline_path),
        'adopted_groups':audit,'combined_guard_exclusions':list(cross.errors)}
    return result


def main() -> None:
    """Write a new selection only when explicitly invoked by the integration owner."""
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root',type=Path,default=Path.cwd())
    parser.add_argument('--baseline',type=Path,required=True)
    parser.add_argument('--fill',type=Path,nargs=3,action='append',default=[],metavar=('LEDGER','ADJUDICATION','VERDICT'))
    parser.add_argument('--holds',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    if args.output.exists():raise ValueError('Refusing to overwrite an existing selection')
    holds=json.loads(args.holds.read_text());holds=holds.get('installation_holds',holds)
    result=merge_selection(args.root,args.baseline,[FillInput(*paths) for paths in args.fill],holds)
    atomic_json(args.output,result)


if __name__=='__main__':main()
