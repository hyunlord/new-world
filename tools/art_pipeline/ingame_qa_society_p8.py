"""P7 society checks keep raw facts, draw evidence and blind recognition separate."""
from __future__ import annotations
from dataclasses import asdict
import json
import hashlib
from pathlib import Path
from blind_p7 import check_blind
from ingame_qa_pixels import Check, result
from ingame_qa_deaths_p7 import check_death_frame
from ingame_qa_founding_p8 import check_founding_frame
from ingame_qa_villages_p7 import check_village_frame
from ingame_qa_intervention_p8 import intervention


def society_checks(frames, society_dir: Path, output: Path, performance_dir: Path | None = None, *, society_seed: int) -> list[Check]:
    """Evaluate every frame; zero eligible frames never satisfies a gate."""
    manifest_path = society_dir/'manifest.json'
    manifest = json.loads(manifest_path.read_text()) if manifest_path.exists() else {}
    deaths = []
    village_errors = []
    founding_errors = []
    exposure_drawn = 0
    counts = {'A26':0,'A27':0,'A28':0,'overview_frames':0}
    remembered = {}
    for path,data in frames:
        scenario,seed = data.get('scenario'),data.get('seed')
        if scenario != f'SOCIETY SCENARIO seed {society_seed}' or seed != society_seed:
            continue
        counts['A26'] += 1
        deaths.extend(path.stem+': '+error for error in check_death_frame(data))
        exposure_drawn += sum(row.get('reason')==4 and row.get('in_view') for row in data.get('death_drawn',[]))
        raw = data.get('society_snapshot',{})
        if not raw:
            village_errors.append(path.stem+': missing society FFI snapshot')
            founding_errors.append(path.stem+': missing society FFI snapshot')
            continue
        key = (scenario,seed)
        village_errors.extend(path.stem+': '+error for error in check_village_frame(data,remembered.setdefault(key,{})))
        counts['A27'] += int(len(raw.get('settlement_ids',[]))>=2)
        counts['overview_frames'] += int(data.get('society_presentation',{}).get('overview_visible',False))
        founding_errors.extend(path.stem+': '+error for error in check_founding_frame(data))
        counts['A28'] += int(bool(raw.get('founder_agent_ids')))
    if not exposure_drawn:
        deaths.append('no visible actual exposure marker in any capture')
    if not counts['overview_frames']:
        village_errors.append('no actual overview discs observed; label colors alone do not prove discs')
    blind_path = output/'blind_society/results.json'
    blind_villages,blind_walk = check_blind(blind_path,'A27'),check_blind(blind_path,'A28')
    if blind_path.exists():
        current_hashes = {path.name:hashlib.sha256(path.read_bytes()).hexdigest() for path,data in frames
                          if data.get('scenario')==f'SOCIETY SCENARIO seed {society_seed}'}
        blinded = json.loads(blind_path.read_text()).get('sources',{})
        for key,source in blinded.items():
            if current_hashes.get(Path(source.get('source','')).name) != source.get('sha256'):
                if key.startswith('A27_'):
                    village_errors.append(key+': blind image does not match current captured society pixels')
                elif key.startswith('A28_'):
                    founding_errors.append(key+': blind image does not match current captured society pixels')
    checks = [result('A26',counts['A26']>0 and not deaths,json.dumps({'frames':counts['A26'],'exposure_markers_in_view':exposure_drawn,'errors':deaths})),
        result('A27',counts['A27']>0 and not village_errors and blind_villages.status=='PASS',json.dumps({'frames':counts['A27'],'overview_frames':counts['overview_frames'],'errors':village_errors,'blind':asdict(blind_villages)},ensure_ascii=False)),
        result('A28',counts['A28']>0 and not founding_errors and blind_walk.status=='PASS',json.dumps({'walking_frames':counts['A28'],'errors':founding_errors,'blind':asdict(blind_walk),'bundle_drop':False},ensure_ascii=False)),
        intervention(frames,output,society_seed=society_seed)]
    return checks
