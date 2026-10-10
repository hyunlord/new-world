# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/install_v4.py --selection docs/art/production2/final_selection.json
"""Plan reviewed v4 installation; asset writes require an explicit matching selection digest."""
from __future__ import annotations
import argparse
from dataclasses import dataclass, replace
import hashlib
import json
from pathlib import Path
from typing import Final, Literal, TypedDict
import numpy as np
from PIL import Image
from hair_registry_p4 import HairFrame, HairRegistry, ReviewedMapError
from pixel_normalize import outline
from procedural_animation import animate
from production_sheets import SHEETS
from salvage_verdicts import candidate_fingerprint
from v4_contracts import FrameCandidate, FrameSpec, WorldScale, guard_chosen, guard_frame, guard_group, target_size
from salvage_v4 import guarded_candidate
from v3_semantics import semantic_counts

ROOT: Final = Path(__file__).resolve().parents[2]
ICON_IDS: Final = {'carry':'U1_05','gather':'U1_06','build':'U1_04','eat':'U1_12',
                   'drink':'U1_11','sit':'U1_13','asleep':'U1_13','talk':'U1_01',
                   'walk':'U1_07','run':'U1_07','hop':'U1_07','hug':'U1_01',
                   'crouch':'U1_06','crossed':'U1_13'}


class InstallError(ValueError):
    """Installation evidence or lineage is inconsistent."""
    code = 'invalid_installation'


@dataclass(frozen=True, slots=True)
class InstallRequest:
    root: Path
    selection: Path
    supplements: tuple[Path, ...] = ()
    holds: Path | None = None


@dataclass(frozen=True, slots=True)
class Target:
    slot_id: str
    body: str
    view: str
    activity: str
    frame: int


@dataclass(frozen=True, slots=True)
class PlannedSprite(Target):
    qa_status: str
    source: str
    source_sprite: str
    contains_basket: bool
    fallback_kind: str
    activity_icon: str
    native: Image.Image
    sprite: Image.Image
    candidate_id: str = ''


class Suppression(TypedDict):
    drawable: bool
    qa_status: str
    fallback_kind: str
    reason: str


@dataclass(frozen=True, slots=True)
class InstallPlan:
    request: InstallRequest
    selection_sha256: str
    sprites: tuple[PlannedSprite, ...]
    suppressed: dict[str, Suppression]
    evidence: tuple[tuple[Path, str], ...]
    unresolved: tuple[str, ...]
    fallback_guard_diagnostics: tuple[str, ...] = ()


def digest(path: Path) -> str:
    """Hash exact on-disk evidence bytes."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify_file(path: Path, expected: str) -> None:
    """Reject absent or changed frozen evidence before any writes."""
    if not path.is_file() or digest(path) != expected:
        raise InstallError(f'Evidence changed or missing: {path}')


def slot_policy(slot: str) -> Literal['actor','retain','icon','suppress']:
    """Keep only explicit essential states and meaningful functional UI glyphs."""
    sheet, index = slot.rsplit('_',1)
    if sheet.startswith(('V_', 'P2V_')):
        return 'actor'
    if (sheet == 'S1' and (int(index) <= 13 or int(index) == 16)) or (sheet == 'S2' and int(index) <= 4) or (sheet == 'N2' and int(index) <= 4):
        return 'retain'
    return 'icon' if sheet == 'U1' else 'suppress'


def meaningful_icon_source(slot: str, source: str, status: str) -> bool:
    """Retain only an explicitly named matching glyph or prior accepted generated icon."""
    index=int(slot.rsplit('_',1)[1])-1
    expected=SHEETS['U1'][index].name
    expected='berries' if expected=='berry' else expected
    return source.startswith(f'procedural:ascii:{expected}:') or (source.startswith('generated:') and status=='accepted')


def fallback_for(target: Target, sources: tuple[PlannedSprite, ...], icons: frozenset[str],
                 limit: tuple[int,int] | None = None) -> PlannedSprite | None:
    """Use accepted same-body sources only; every substitution keeps explicit lineage."""
    safe = sorted((s for s in sources if s.body == target.body and s.qa_status == 'accepted'
                   and (target.activity == 'carry' or not s.contains_basket)), key=lambda s:(s.frame != target.frame, abs(s.frame-target.frame), s.slot_id))
    opposite = {'E':'W','W':'E'}.get(target.view)
    carry_sources = [s for s in safe if s.activity == 'carry' and not s.activity_icon
                     and s.fallback_kind != 'idle_icon'] if target.activity == 'carry' else []
    if carry_sources:
        source = next((s for s in carry_sources if s.view == target.view), None)
        if source is None:
            source = next((s for s in carry_sources if s.view == opposite), None)
        if source is None:
            source = next((s for s in carry_sources if s.view == 'S'), carry_sources[0])
        kind = 'mirror' if source.view == opposite else 'borrowed_carry_view'
        native = source.native.transpose(Image.Transpose.FLIP_LEFT_RIGHT) if kind == 'mirror' else source.native.copy()
    else:
        source = next((s for s in safe if s.activity == target.activity and s.view == opposite), None)
        kind = 'mirror'
        if source is not None:
            native = source.native.transpose(Image.Transpose.FLIP_LEFT_RIGHT)
        else:
            same = [s for s in safe if s.view == target.view and s.activity in (target.activity,'idle')]
            source = next((s for s in same if s.activity == target.activity), same[0] if same else None)
            kind = 'procedural'
            native = None
            if source is not None and not (target.activity == 'carry' and source.activity != 'carry'):
                try:
                    native = animate(source.native,target.activity,target.frame,target.view)
                except ValueError:
                    native = None
            if native is not None and limit is not None and (native.width > limit[0] or native.height > limit[1]+2):
                native = None
            if native is None:
                source = next((s for s in safe if s.activity == target.activity and s.view == 'S'), None)
                kind = 'south_view'
                if source is not None:
                    native = source.native.copy()
            if native is None:
                source = next((s for s in safe if s.activity == 'idle' and s.view == target.view and not s.contains_basket), None)
                kind = 'idle_icon'
                if source is None or ICON_IDS.get(target.activity,'') not in icons:
                    return None
                native = source.native.copy()
    if source is None or native is None:
        return None
    bbox=native.getchannel('A').getbbox()
    if bbox is None: return None
    native=native.crop(bbox)
    opaque = np.asarray(source.sprite)
    colors = opaque[opaque[...,3] == 255,:3]
    dark = tuple(map(int, colors[np.argmin(colors.sum(axis=1))]))
    return PlannedSprite(target.slot_id,target.body,target.view,target.activity,target.frame,
        'procedural' if kind == 'procedural' else 'fallback',
        ('procedural:' if kind == 'procedural' else 'derived:')+kind,source.slot_id,
        source.contains_basket,kind,ICON_IDS[target.activity] if kind == 'idle_icon' else '',
        native,outline(native,dark))


def build_plan(request: InstallRequest) -> InstallPlan:
    """Resolve every slot without creating files; require frozen review plus technical guards."""
    root=request.root
    selection_path=request.selection if request.selection.is_absolute() else root/request.selection
    selection=json.loads(selection_path.read_text())
    if selection.get('schema_version') != 1:
        raise InstallError('Unsupported selection schema')
    scale=WorldScale(**selection['scale']); evidence={selection_path:digest(selection_path)}
    hold_path=request.holds or root/'docs/art/production2/fill_sheets.json'
    if not hold_path.is_absolute(): hold_path=root/hold_path
    holds=json.loads(hold_path.read_text()).get('installation_holds',{}) if hold_path.is_file() else {}
    if hold_path.is_file(): evidence[hold_path]=digest(hold_path)
    supplement_paths={path.resolve() for path in request.supplements}
    manifest_path=root/'assets/art/manifest.json'; manifest=json.loads(manifest_path.read_text())
    evidence[manifest_path]=digest(manifest_path)
    existing={row['sprite_id']:row for row in manifest['assets'] if row.get('sprite_id')}
    sprites=[]; targets=[]; ledgers={}; verdicts={}; suppressed={}; unresolved=[]
    definitions=[]
    known=set()
    for sheet,specs in SHEETS.items():
        for index,spec in enumerate(specs,1):
            slot=f'{sheet}_{index:02d}'
            body={'a':'adult_a','b':'adult_b','c':'child'}.get(sheet[2:3],'') if spec.view else ''
            definitions.append((Target(slot,body,spec.view,spec.activity,spec.frame),spec.name))
            known.add(slot)
    for slot,decision in selection['slots'].items():
        if slot in known: continue
        meta=decision.get('semantic',decision)
        if not slot.startswith('P2V_') or meta.get('body') not in ('adult_a','adult_b','child') or not all(key in meta for key in ('view','activity','frame','name')):
            raise InstallError(f'Unknown or incomplete supplemental target: {slot}')
        definitions.append((Target(slot,meta['body'],meta['view'],meta['activity'],meta['frame']),meta['name']))
    chosen=[]
    physical_groups={}
    for target,name in definitions:
        slot=target.slot_id; body=target.body; targets.append(target)
        decision=selection['slots'].get(slot)
        if not decision or decision['disposition'] == 'unfilled' or decision.get('candidate_id') in holds:
            continue
        ledger_path=root/decision['ledger']; verdict_path=root/decision['verdict']
        for path,key in ((ledger_path,'ledger_sha256'),(verdict_path,'verdict_sha256')):
            verify_file(path,decision[key]); evidence[path]=decision[key]
        if ledger_path not in ledgers:
            ledger=json.loads(ledger_path.read_text())
            if ledger['scale'] != selection['scale']: raise InstallError('Mixed scale ledgers')
            ledgers[ledger_path]={row['id']:row for row in ledger['candidates']}
        if verdict_path not in verdicts: verdicts[verdict_path]=json.loads(verdict_path.read_text())
        record=ledgers[ledger_path][decision['candidate_id']]; review=verdicts[verdict_path]
        if record.get('target_slot',record['slot_id']) != slot or (record['body'],record['view'],record['activity'],record['frame']) != (body,target.view,target.activity,target.frame):
            raise InstallError(f'Candidate target mismatch: {slot}')
        ancestor=record; seen=set(); held=False
        while ancestor.get('source_candidate'):
            identity=ancestor['source_candidate']
            if identity in seen: raise InstallError(f'Cyclic source lineage: {slot}')
            seen.add(identity)
            if identity in holds: held=True; break
            ancestor=ledgers[ledger_path].get(identity)
            if ancestor is None or ancestor['body'] != body:
                raise InstallError(f'Missing or cross-body lineage: {slot}')
            if body and (ancestor.get('source_sheet',ancestor.get('sheet')),ancestor.get('attempt')) != (record.get('source_sheet',record['sheet']),record['attempt']):
                raise InstallError(f'Mixed physical derived lineage: {slot}')
            if body and target.activity != 'carry' and (ancestor.get('contains_basket') or ancestor['activity']=='carry'):
                raise InstallError(f'Basket-bearing ancestor outside carry: {slot}')
        if held: continue
        verdict=next((v for v in review['verdicts'] if v['sprite_id']==record['slot_id'] and v['candidate_id']==record['id']),None)
        if not verdict or verdict['verdict'] != 'ok' or not record['guards']['passed'] or record['guards'].get('errors') or any(record.get(key,{}).get('errors') for key in ('group_guards','cross_guards')):
            raise InstallError(f'Candidate lacks visual and technical approval: {slot}')
        if review['candidate_fingerprints'].get(record['slot_id']) != candidate_fingerprint(record):
            raise InstallError(f'Reviewed pixels changed: {slot}')
        for path in record['images'].values(): evidence[Path(path)]=digest(Path(path))
        with Image.open(record['images']['native']) as opened: native=opened.convert('RGBA')
        with Image.open(record['images']['sprite']) as opened: sprite=opened.convert('RGBA')
        basket=bool(record.get('contains_basket',target.activity=='carry' or 'basket' in name))
        if body and basket and target.activity != 'carry': raise InstallError(f'Basket outside carry: {slot}')
        candidate=guarded_candidate(record)
        berry,fire=semantic_counts(candidate.image)
        candidate=replace(candidate,berry_texels=berry,fire_texels=fire)
        report=guard_frame(candidate,scale)
        if not report.passed: raise InstallError(f'Current frame guard failed: {slot}: {report.errors}')
        if body:
            key=(body,target.view,target.activity)
            physical_groups.setdefault(key,[]).append((record.get('source_sheet',record['sheet']),record['attempt'],candidate))
        chosen.append((target,candidate))
        generated=record['source_kind']=='generated'
        status='accepted' if generated else 'derived'
        if decision['disposition'] != status: raise InstallError(f'Dishonest source status: {slot}')
        source=(f"generated:{record.get('source_sheet',record['sheet'])}/{record.get('source_cell',record['cell']):02d}@{record['attempt']}:v4:f={record['factor']:g}" if generated else 'derived:'+record['derivation_rule'])
        sprites.append(PlannedSprite(target.slot_id,target.body,target.view,target.activity,target.frame,status,source,record.get('source_candidate',''),basket,'','',native,sprite,record['id']))
    for key,members in physical_groups.items():
        validate_physical_group(tuple(members),scale)
    accepted=tuple(sprites)
    # Retain known functional glyphs first, so actor icon fallback has a real texture.
    for target in targets:
        if any(s.slot_id==target.slot_id for s in sprites) or slot_policy(target.slot_id) not in ('retain','icon'): continue
        old=existing.get(target.slot_id)
        meaningful=old and (slot_policy(target.slot_id)=='retain' or meaningful_icon_source(target.slot_id,old.get('source',''),old.get('qa_status','')))
        if not meaningful: continue
        for key in ('path','native_path'):
            if old.get(key): evidence[root/old[key]]=digest(root/old[key])
        with Image.open(root/old['path']) as opened: sprite=opened.convert('RGBA')
        with Image.open(root/old.get('native_path',old['path'])) as opened: native=opened.convert('RGBA')
        sprites.append(PlannedSprite(target.slot_id,target.body,target.view,target.activity,target.frame,
            'fallback',old['source'],target.slot_id,bool(old.get('contains_basket',False)),
            'retained_installed','',native,sprite))
    icons=frozenset(s.slot_id for s in sprites if s.slot_id.startswith('U1_'))
    for target in targets:
        if any(s.slot_id==target.slot_id for s in sprites): continue
        sheet,index=target.slot_id.rsplit('_',1)
        limit=target_size(FrameSpec(sheet,int(index),1,target.body,target.view,target.activity),scale)
        fallback=fallback_for(target,accepted,icons,limit) if target.body else None
        if fallback is not None: sprites.append(fallback); continue
        reason='no_review_approved_candidate' if slot_policy(target.slot_id)=='suppress' else 'no_safe_source'
        suppressed[target.slot_id]={'drawable':False,'qa_status':'suppressed','fallback_kind':'unaccepted_decorative' if not target.body else 'unresolved_actor','reason':reason}
        if reason=='no_safe_source': unresolved.append(target.slot_id)
    diagnostics=validate_assembled(tuple(sprites),tuple(chosen),scale)
    if supplement_paths and not supplement_paths.issubset({p.resolve() for p in ledgers}):
        raise InstallError('Supplement is not referenced by explicit merged selection')
    return InstallPlan(request,evidence[selection_path],tuple(sprites),suppressed,tuple(evidence.items()),tuple(unresolved),diagnostics)


def validate_physical_group(members: tuple[tuple[str,int,FrameCandidate],...], scale: WorldScale) -> None:
    """Reject attempt/sheet mixing even when semantic aliases happen to match."""
    if len({(sheet,attempt) for sheet,attempt,_ in members}) != 1:
        raise InstallError('Mixed physical animation group')
    report=guard_group([candidate for _,_,candidate in members],scale)
    if not report.ready_for_review: raise InstallError(f'Current selected group guard failed: {report.errors + report.warnings}')


def validate_assembled(sprites: tuple[PlannedSprite,...], chosen: tuple[tuple[Target,FrameCandidate],...], scale: WorldScale) -> tuple[str,...]:
    """Recompute final geometry, animation and world-relative constraints from native pixels."""
    groups={}
    reviewed_groups={}
    diagnostics=[]
    palette=json.loads((Path(__file__).with_name("production_palette.json")).read_text())
    allowed={tuple(row["rgb"]) for row in palette["colors"]}
    candidates={target.slot_id:candidate for target,candidate in chosen}
    final={}
    for sprite in sprites:
        sheet,index=sprite.slot_id.rsplit('_',1)
        source=candidates.get(sprite.slot_id)
        # Final fallback pixels have no conversion drift metric; source review remains separately bound.
        berry,fire=semantic_counts(sprite.native)
        pixels=np.asarray(sprite.native)
        opaque=pixels[pixels[...,3]>0,:3]
        membership=100.0*sum(tuple(map(int,pixel)) in allowed for pixel in opaque)/max(1,len(opaque))
        candidate=FrameCandidate(FrameSpec(sheet,int(index),1,sprite.body,sprite.view,sprite.activity,sprite.native.size),
            sprite.native,source.mean_delta_e if source else 0.0,membership,
            bool(np.all(np.isin(np.asarray(sprite.native)[...,3],[0,255]))),berry,fire)
        final[sprite.slot_id]=candidate
        report=guard_frame(candidate,scale)
        if not report.passed: raise InstallError(f'Assembled frame guard failed: {sprite.slot_id}: {report.errors}')
        if sprite.body:
            key=(sprite.body,sprite.view,sprite.activity)
            # Runtime order is semantic frame, independent of physical fill-sheet cell positions.
            normalized=replace(candidate,frame=replace(candidate.frame,sheet='assembled',cell=sprite.frame+1))
            groups.setdefault(key,[]).append(normalized)
            if sprite.qa_status in ("accepted","derived"):
                reviewed_groups.setdefault(key,[]).append(normalized)
    for key,members in reviewed_groups.items():
        report=guard_group(members,scale)
        if not report.ready_for_review: raise InstallError(f'Assembled reviewed group guard failed: {key}: {report.errors + report.warnings}')
    for key,members in groups.items():
        report=guard_group(members,scale)
        if not report.ready_for_review:
            diagnostics.append(f'{key}: {report.errors + report.warnings}')
    report=guard_chosen(
        bushes=[final[key] for key in sorted(final) if key in ('N2_01','N2_02','N2_03','N2_04')],
        adults=[c for c in final.values() if c.frame.body.startswith('adult')],
        children=[c for c in final.values() if c.frame.body=='child'],
        baskets=[final[key] for key in ('S2_10','S2_11') if key in final])
    if not report.passed: raise InstallError(f'Assembled cross guard failed: {report.errors}')
    return tuple(diagnostics)


def write_report(plan: InstallPlan, path: Path) -> None:
    """Write only a reviewable dry-run report, never production assets."""
    payload={'selection_sha256':plan.selection_sha256,'unresolved':list(plan.unresolved),
        'fallback_guard_diagnostics':list(plan.fallback_guard_diagnostics),
        'evidence':{str(p):sha for p,sha in plan.evidence},'suppressed':plan.suppressed,
        'sprites':[{'sprite_id':s.slot_id,'candidate_id':s.candidate_id,'source':s.source,
        'qa_status':s.qa_status,'source_sprite':s.source_sprite,'body':s.body,'view':s.view,
        'activity':s.activity,'frame':s.frame,'contains_basket':s.contains_basket,
        'fallback_kind':s.fallback_kind,'activity_icon':s.activity_icon,'drawable':True,
        'native_size':list(s.native.size),'canvas_size':list(s.sprite.size)} for s in plan.sprites]}
    path.parent.mkdir(parents=True,exist_ok=True); path.write_text(json.dumps(payload,indent=2)+'\n')


def apply_plan(plan: InstallPlan, approved_selection_sha256: str) -> None:
    """Install a reviewed plan only while every bound source remains byte-identical."""
    if approved_selection_sha256 != plan.selection_sha256: raise InstallError('Selection approval digest mismatch')
    for path,sha in plan.evidence: verify_file(path,sha)
    try:
        hair = HairRegistry.load(plan.request.root)
        for sprite in plan.sprites:
            if sprite.body:
                hair.resolve(HairFrame(sprite.body, sprite.activity, sprite.sprite))
    except ReviewedMapError as error:
        raise InstallError(str(error)) from error
    root=plan.request.root; path=root/'assets/art/manifest.json'; manifest=json.loads(path.read_text())
    by_path={row['path']:row for row in manifest['assets']}
    for sprite in plan.sprites:
        sheet,cell=sprite.slot_id.rsplit('_',1); folder=root/'assets/art/production'/sheet
        folder.mkdir(parents=True,exist_ok=True); target=folder/f'{cell}.png'; native=folder/f'{cell}_native.png'
        sprite.sprite.save(target);sprite.native.save(native)
        entry={'sprite_id':sprite.slot_id,'path':str(target.relative_to(root)),'native_path':str(native.relative_to(root)),
            'source':sprite.source,'source_sprite':sprite.source_sprite,'qa_status':sprite.qa_status,
            'candidate_id':sprite.candidate_id,
            'body':sprite.body,'view':sprite.view,'activity':sprite.activity,'frame':sprite.frame,
            'class':'child' if sprite.body=='child' else ('adult' if sprite.body else sheet),
            'contains_basket':sprite.contains_basket,'fallback_kind':sprite.fallback_kind,
            'activity_icon':sprite.activity_icon,'drawable':True,'native_size':list(sprite.native.size),
            'canvas_size':list(sprite.sprite.size),'anchor':[sprite.sprite.width//2,sprite.sprite.height-1]}
        by_path[entry['path']]=entry
        by_path[str(native.relative_to(root))]={'path':str(native.relative_to(root)),'source':sprite.source,
            'qa_status':sprite.qa_status,'native_size':list(sprite.native.size),'role':'native_reference','anchor':[0,0]}
    for row in by_path.values():
        if row.get('sprite_id') in plan.suppressed: row['drawable']=False
    manifest['assets']=list(by_path.values());manifest['suppressed']=plan.suppressed
    manifest['installation_selection_sha256']=plan.selection_sha256
    temporary=path.with_suffix('.tmp');temporary.write_text(json.dumps(manifest,indent=2)+'\n');temporary.replace(path)


def main() -> None:
    """Dry-run by default; never infer permission to install from passing checks."""
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root',type=Path,default=ROOT);parser.add_argument('--selection',type=Path,required=True)
    parser.add_argument('--supplement',type=Path,action='append',default=[])
    parser.add_argument('--holds',type=Path);parser.add_argument('--report',type=Path,default=ROOT/'docs/art/production2/install_plan.json')
    parser.add_argument('--apply-selection-sha256')
    args=parser.parse_args();plan=build_plan(InstallRequest(args.root,args.selection,tuple(args.supplement),args.holds))
    write_report(plan,args.report)
    if args.apply_selection_sha256: apply_plan(plan,args.apply_selection_sha256)
    print(f'Planned {len(plan.sprites)} drawable, {len(plan.suppressed)} suppressed; {len(plan.unresolved)} unresolved')


if __name__=='__main__': main()
