# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/salvage_verdicts.py --help
"""Validate frozen visual verdicts, adjudicate guards, and propose coherent round 2."""
from __future__ import annotations

import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path
from typing import TypedDict

from PIL import Image

from production_sheets import SHEETS
from salvage_v4 import guarded_candidate, guard_data
from v4_contracts import WorldScale, guard_frame, guard_group, guard_chosen

REASONS=frozenset(('wrong_pose','wrong_view','broken_anatomy','different_person','wrong_object',
                  'artefact','detail_destroyed_by_fit','region_map_wrong'))


class Verdict(TypedDict):
    sprite_id: str
    candidate_id: str | None
    verdict: str
    reason: str
    observation: str
    unfitted_acceptable: bool | None


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def atomic_json(path: Path, value: dict) -> None:
    path.parent.mkdir(parents=True,exist_ok=True)
    temporary=path.with_suffix('.tmp')
    temporary.write_text(json.dumps(value,indent=2,allow_nan=False)+'\n');temporary.replace(path)


def candidate_fingerprint(record: dict) -> str:
    evidence={'record':record,'images':{name:sha(Path(path)) for name,path in record.get('images',{}).items()}}
    return hashlib.sha256(json.dumps(evidence,sort_keys=True,separators=(',',':')).encode()).hexdigest()


def rendered_fingerprints(preview: dict, directory: Path) -> dict[str,str]:
    """Bind each two-column/eight-row review card, including its region and reference."""
    result={}
    for slot in preview['previews']:
        sheet,cell=slot.rsplit('_',1);path=directory/f'{sheet}.png'
        if path.name not in preview['images_sha256']:continue
        with Image.open(path) as image:
            card_width=image.width//2;card_height=(image.height-62)//8
            index=int(cell)-1;left=(index%2)*card_width;top=62+(index//2)*card_height
            card=image.crop((left,top,left+card_width,top+card_height)).convert('RGBA')
            result[slot]=hashlib.sha256(card.tobytes()).hexdigest()
    return result


def validate_and_persist(parts: list[Path], candidates_path: Path, preview_path: Path,
                         output: Path, *, round_number: int=1, previous_path: Path | None=None,
                         expected_slots: set[str] | None=None) -> dict:
    """Persist authoritative labels before exposing any acceptance statistics."""
    if output.exists():raise ValueError('Refusing to overwrite an authoritative review round')
    if round_number not in (1,2):raise ValueError('Only two review rounds per scale are permitted')
    if round_number==2 and previous_path is None:raise ValueError('Round 2 requires frozen round 1')
    candidates=json.loads(candidates_path.read_text());preview=json.loads(preview_path.read_text())
    if preview['candidates_sha256']!=sha(candidates_path):raise ValueError('Stale preview candidate hash')
    for name,digest in preview['images_sha256'].items():
        if sha(preview_path.parent/name)!=digest:raise ValueError(f'Stale review image: {name}')
    expected=expected_slots if expected_slots is not None else {f'{sheet}_{i:02d}' for sheet in SHEETS for i in range(1,17)}
    if not expected.issubset(preview['previews']):raise ValueError('Preview missing expected slots')
    lookup={r['id']:r for r in candidates['candidates']}
    records: list[Verdict]=[];seen=set();seen_candidates=set()
    for part in parts:
        values=json.loads(part.read_text())
        if isinstance(values,dict):values=values['verdicts']
        if not isinstance(values,list):raise ValueError('Reviewer output must contain an array')
        for value in values:
            slot=value.get('sprite_id',value.get('slot_id'))
            if slot not in expected or slot in seen:raise ValueError(f'Unexpected or duplicate slot: {slot}')
            identity=value.get('candidate_id')
            if identity!=preview['previews'][slot]:raise ValueError(f'Candidate differs from frozen preview: {slot}')
            if identity is not None:
                if identity not in lookup or lookup[identity]['slot_id']!=slot:raise ValueError('Unknown candidate or wrong slot')
                if identity in seen_candidates:raise ValueError('Duplicate candidate identity')
                seen_candidates.add(identity)
            decision=value.get('verdict');reason=value.get('reason') or ''
            if identity is None:
                if decision!='unavailable':raise ValueError('Unavailable preview cannot receive visual approval')
            elif decision not in ('ok','reject'):raise ValueError('Unknown verdict')
            if decision=='ok' and reason:raise ValueError('Approval must have an empty reason')
            if decision=='reject' and reason not in REASONS:raise ValueError('Unknown rejection reason')
            unfitted=value.get('unfitted_acceptable')
            if unfitted is not None and type(unfitted) is not bool:raise ValueError('Invalid unfitted acceptance value')
            if reason=='detail_destroyed_by_fit' and unfitted is None:raise ValueError('Fitting damage requires unfitted assessment')
            observation=value.get('observation','')
            if not isinstance(observation,str):raise ValueError('Observation must be text')
            records.append(Verdict(sprite_id=slot,candidate_id=identity,verdict=decision,reason=reason,
                                   observation=observation,unfitted_acceptable=unfitted));seen.add(slot)
    if seen!=expected:raise ValueError(f'Missing verdict coverage: {sorted(expected-seen)}')
    envelope={'schema_version':1,'round':round_number,'scale':candidates.get('scale'),
              'candidates_sha256':sha(candidates_path),'preview_sha256':sha(preview_path),
              'source_parts':{str(path):sha(path) for path in parts},'references':candidates['references'],
              'previews':preview['previews'],'candidate_fingerprints':{slot:candidate_fingerprint(lookup[identity])
                 for slot,identity in preview['previews'].items() if identity is not None},
              'rendered_fingerprints':rendered_fingerprints(preview,preview_path.parent),
              'review_images_sha256':preview['images_sha256'],
              'previous_round_sha256':sha(previous_path) if previous_path else None,
              'expected_slots':sorted(expected),'verdicts':sorted(records,key=lambda r:r['sprite_id'])}
    atomic_json(output,envelope)
    return envelope


def reusable_approval(verdict: dict, previous: dict, current: dict, body: str) -> bool:
    """An old ok is valid only for identical candidate pixels and identity reference."""
    slot=verdict['sprite_id']
    return (verdict['verdict']=='ok'
            and previous['candidate_fingerprints'].get(slot)==current['candidate_fingerprints'].get(slot)
            and slot in previous['candidate_fingerprints']
            and previous.get('rendered_fingerprints',{}).get(slot)==current.get('rendered_fingerprints',{}).get(slot)
            and (not body or previous['references'].get(body)==current['references'].get(body)))


def adjudicate(candidates_path: Path, authoritative_path: Path, previous_path: Path | None=None) -> dict:
    """Re-run technical guards after visual labels have been durably saved."""
    ledger=json.loads(candidates_path.read_text());review=json.loads(authoritative_path.read_text())
    if review['candidates_sha256']!=sha(candidates_path):raise ValueError('Verdicts belong to another candidate ledger')
    lookup={r['id']:r for r in ledger['candidates']};verdicts={v['sprite_id']:v for v in review['verdicts']}
    if previous_path:
        if review['previous_round_sha256']!=sha(previous_path):raise ValueError('Previous review changed')
        previous=json.loads(previous_path.read_text())
        for verdict in previous['verdicts']:
            slot=verdict['sprite_id'];identity=ledger['selected'].get(slot)
            if slot not in verdicts and identity==verdict['candidate_id'] and reusable_approval(verdict,previous,review,lookup[identity]['body']):
                verdicts[slot]=verdict
    accepted={};rejected={};scale=WorldScale(**ledger['scale'])
    for slot,identity in ledger['selected'].items():
        record=lookup[identity];verdict=verdicts.get(slot)
        if not verdict or verdict['candidate_id']!=identity or verdict['verdict']!='ok':
            rejected[slot]=['no_current_visual_approval'];continue
        if review['candidate_fingerprints'].get(slot)!=candidate_fingerprint(record):raise ValueError(f'Candidate pixels changed after review: {slot}')
        report=guard_frame(guarded_candidate(record),scale)
        retained_errors=record['guards']['errors']
        if not report.passed or retained_errors:
            rejected[slot]=list(dict.fromkeys((*retained_errors,*report.errors)));continue
        accepted[slot]=identity
    groups={}
    for slot,identity in accepted.items():
        record=lookup[identity]
        if record['body']:
            key=(record['body'],record['sheet'],record['view'],record['activity'])
            groups.setdefault(key,[]).append(record)
    for records in groups.values():
        report=guard_group([guarded_candidate(r) for r in records],scale)
        if not report.ready_for_review:
            for record in records:
                accepted.pop(record['slot_id'],None)
                rejected[record['slot_id']]=list(report.errors+report.warnings)
    chosen=[lookup[identity] for identity in accepted.values()]
    bushes=sorted([r for r in chosen if r['sheet']=='N2' and r['cell']<=4],key=lambda r:r['cell'])
    adults=[r for r in chosen if r['body'].startswith('adult')]
    children=[r for r in chosen if r['body']=='child']
    baskets=[r for r in chosen if r['sheet']=='S2' and r['cell'] in (10,11)]
    cross=guard_chosen(*[[guarded_candidate(r) for r in group] for group in (bushes,adults,children,baskets)])
    blocked=[]
    if 'berry_stage_order' in cross.errors:blocked+=bushes+baskets
    if any(error in cross.errors for error in ('basket_relative_size','missing_chosen_bush_reference')):blocked+=baskets
    if any(error in cross.errors for error in ('child_relative_height','missing_chosen_adult_reference')):blocked+=children
    for record in blocked:
        accepted.pop(record['slot_id'],None);rejected[record['slot_id']]=list(cross.errors)
    final=[lookup[identity] for identity in accepted.values()]
    final_cross=guard_chosen(
        bushes=[guarded_candidate(r) for r in sorted(final,key=lambda r:r['cell']) if r['sheet']=='N2' and r['cell']<=4],
        adults=[guarded_candidate(r) for r in final if r['body'].startswith('adult')],
        children=[guarded_candidate(r) for r in final if r['body']=='child'],
        baskets=[guarded_candidate(r) for r in final if r['sheet']=='S2' and r['cell'] in (10,11)])
    return {'round':review['round'],'authoritative_sha256':sha(authoritative_path),'accepted':accepted,
            'not_accepted':rejected,'cross_guards_before_exclusions':guard_data(cross),
            'final_cross_guards':guard_data(final_cross),
            'statistics':{'accepted':len(accepted),'visually_ok':sum(v['verdict']=='ok' for v in verdicts.values())}}


def _group_key(record: dict) -> tuple[str,str,str,str]:
    return record['body'],record['sheet'],record['view'],record['activity']


def propose_round2(ledger: dict, review: dict, force_bodies: tuple[str,...]=()) -> dict:
    """Return a proposal only; never write candidates, images, verdicts or assets."""
    if review['round']!=1:raise ValueError('Round 2 is the last ordinary review round')
    proposed=deepcopy(ledger);lookup={r['id']:r for r in ledger['candidates']}
    rejected=[v for v in review['verdicts'] if v['verdict']=='reject' and v['candidate_id'] in lookup]
    affected=set();unavailable=[];changed_groups=[];changed_references={}
    actor_rejects={}
    reviewed_ok={v['sprite_id']:v['candidate_id'] for v in review['verdicts'] if v['verdict']=='ok'}
    for verdict in rejected:
        record=lookup[verdict['candidate_id']]
        if record['body']:actor_rejects.setdefault(_group_key(record),set()).add(record['attempt'])
        else:
            options=[r for r in ledger['candidates'] if r['slot_id']==record['slot_id'] and r['attempt']!=record['attempt'] and r['guards']['passed']]
            if options:
                choice=min(options,key=lambda r:(len(r['guards']['warnings']),r['metrics']['mean_delta_e'],r['attempt']))
                proposed['selected'][record['slot_id']]=choice['id'];affected.add(record['slot_id'])
            else:
                proposed['selected'].pop(record['slot_id'],None);unavailable.append(record['slot_id'])
    for key,attempts in actor_rejects.items():
        options=[g for g in ledger['groups'] if (g['body'],g['sheet'],g['view'],g['activity'])==key
                 and g['attempt'] not in attempts and g['guards']['passed']
                 and 'pending_cycle_completion' not in g['guards']['warnings']]
        old_slots={r['slot_id'] for r in ledger['candidates'] if _group_key(r)==key}
        for slot in old_slots:proposed['selected'].pop(slot,None)
        if not options:
            retained={slot:ledger['selected'][slot] for slot in old_slots
                      if slot in ledger['selected'] and reviewed_ok.get(slot)==ledger['selected'][slot]}
            if len({lookup[identity]['attempt'] for identity in retained.values()})>1:
                raise ValueError('Original selected group mixes attempts')
            proposed['selected'].update(retained)
            unavailable.extend(sorted(old_slots-set(retained)));continue
        choice=min(options,key=lambda g:(-g['passing_original_count'],g['mean_delta_e'],g['attempt']))
        for identity in choice['candidate_ids']:
            record=lookup[identity];proposed['selected'][record['slot_id']]=identity;affected.add(record['slot_id'])
        changed_groups.append({'body':key[0],'sheet':key[1],'view':key[2],'activity':key[3],'attempt':choice['attempt']})
        if key[2:] == ('S','walk'):
            idle=[r for r in ledger['candidates'] if r['body']==key[0] and r['sheet']==key[1]
                  and r['attempt']==choice['attempt'] and r['view']=='S' and r['activity']=='idle'
                  and r['frame']==0 and r['guards']['passed']]
            if not idle:
                idle=[r for r in ledger['candidates'] if r['body']==key[0] and r['view']=='S' and r['activity']=='idle' and r['frame']==0 and r['guards']['passed']]
            if idle:
                reference=min(idle,key=lambda r:(r['metrics']['mean_delta_e'],r['attempt']))['id']
                if reference!=proposed['references'].get(key[0]):
                    changed_references[key[0]]={'before':proposed['references'].get(key[0]),'after':reference}
                    proposed['references'][key[0]]=reference
                    affected.update(r['slot_id'] for r in ledger['candidates'] if r['body']==key[0])
    affected.update(r['slot_id'] for r in ledger['candidates'] if r['body'] in force_bodies)
    for group in proposed['groups']:
        group['selected']=any(identity in proposed['selected'].values() for identity in group['candidate_ids'])
    for record in proposed['candidates']:
        if record.get('source_kind')=='derived':
            source=lookup[record['source_candidate']]
            for field in ('grid','factor_source'):
                if field in source:record[field]=deepcopy(source[field])
                else:record.pop(field,None)
            record['guards']['warnings']=list(dict.fromkeys((*source['guards']['warnings'],
                *record['guards']['warnings'],'derived_fidelity_inherited')))
    proposed['review_round']=2
    return {'ledger':proposed,'affected_slots':sorted(affected),'no_alternate_slots':sorted(set(unavailable)),
            'changed_groups':changed_groups,'changed_references':changed_references,
            'status':'proposal_requires_frozen_preview_and_independent_round2_review'}


def main() -> None:
    parser=argparse.ArgumentParser(description=__doc__)
    sub=parser.add_subparsers(dest='command',required=True)
    collect=sub.add_parser('aggregate')
    collect.add_argument('--parts',type=Path,nargs='+',required=True)
    collect.add_argument('--candidates',type=Path,required=True);collect.add_argument('--preview',type=Path,required=True)
    collect.add_argument('--output',type=Path,required=True);collect.add_argument('--round',type=int,default=1)
    collect.add_argument('--previous',type=Path);collect.add_argument('--expected-slots',type=Path)
    for command in ('adjudicate','propose-round2'):
        child=sub.add_parser(command);child.add_argument('--candidates',type=Path,required=True)
        child.add_argument('--verdicts',type=Path,required=True);child.add_argument('--output',type=Path,required=True)
        if command=='adjudicate':child.add_argument('--previous',type=Path)
        else:
            child.add_argument('--force-body',action='append',default=[])
            child.add_argument('--candidates-output',type=Path)
    args=parser.parse_args()
    if args.command=='aggregate':
        expected=set(json.loads(args.expected_slots.read_text())) if args.expected_slots else None
        validate_and_persist(args.parts,args.candidates,args.preview,args.output,round_number=args.round,
                             previous_path=args.previous,expected_slots=expected)
    elif args.command=='adjudicate':atomic_json(args.output,adjudicate(args.candidates,args.verdicts,args.previous))
    else:
        proposal=propose_round2(json.loads(args.candidates.read_text()),json.loads(args.verdicts.read_text()),tuple(args.force_body))
        destination=args.candidates_output or args.output.with_name('candidates_round2.json')
        if destination.resolve()==args.candidates.resolve():raise ValueError('Round 1 ledger is immutable')
        atomic_json(destination,proposal.pop('ledger'))
        proposal.update(candidates_path=str(destination),candidates_sha256=sha(destination))
        atomic_json(args.output,proposal)


if __name__=='__main__':main()
