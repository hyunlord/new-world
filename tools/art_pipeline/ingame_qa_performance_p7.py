"""A30 consumes paired same-process frame samples and explicit audit closure."""
import json
import re
from pathlib import Path
import numpy as np
from ingame_qa_pixels import Check, result


def performance(society: Path, output: Path, expected_git_sha: str | None = None) -> Check:
    """Missing paired evidence is N/A; unrelated or incomplete samples fail."""
    paths = [society/f'frame_times_soc_{count}.json' for count in [1,3]]
    if not all(path.exists() for path in paths):
        return Check('A30','N/A','Missing paired 300-frame one/three-village timings; no performance conclusion')
    rows = [json.loads(path.read_text()) for path in paths]
    errors: list[str] = []
    manifest_path = society/'manifest.json'
    manifest = json.loads(manifest_path.read_text()) if manifest_path.exists() else {}
    if not manifest.get('git_sha') or (expected_git_sha and manifest['git_sha'] != expected_git_sha):
        errors.append('performance capture source differs from final society source or is absent')
    if manifest.get('run_id') != rows[0].get('run_id'):
        errors.append('performance manifest and sample run identities differ')
    medians = []
    for expected,row in zip([1,3],rows):
        samples = row.get('samples_ms',[])
        if len(samples)!=300 or row.get('alive_count')!=expected:
            errors.append(f'{expected}-village sample count/settlements mismatch')
        if not samples or not all(np.isfinite(sample) and sample > 0 for sample in samples):
            errors.append(f'{expected}-village frame samples must be finite and positive')
        medians.append(float(np.median(samples)) if samples else 0)
        load = row.get('load_status',{})
        if not load.get('uptime') or 'processes' not in load or load.get('uptime_exit_code') != 0 or load.get('pgrep_exit_code') not in (0,1):
            errors.append(f'{expected}-village load record absent')
    for key in ['run_id','seed','zoom','simulation_frozen']:
        if key not in rows[0] or rows[0][key] != rows[1].get(key):
            errors.append(f'pair does not share {key}')
    if not rows[0].get('run_id'):
        errors.append('no same-process run identity')
    load_1m = []
    for row in rows:
        uptime = row.get('load_status',{}).get('uptime',[])
        text = ' '.join(uptime) if isinstance(uptime,list) else str(uptime)
        match = re.search(r'load averages?:\s*([0-9.]+)',text)
        load_1m.append(float(match.group(1)) if match else None)
    ratio = medians[1]/medians[0] if medians[0]>0 else None
    if ratio is None or ratio>1.15:
        errors.append(f'median ratio {ratio} exceeds1.15')
    audit_path = output/'audit_closure.json'
    audit = json.loads(audit_path.read_text()) if audit_path.exists() else {}
    items = audit.get('items',[])
    if {row.get('id') for row in items} != {f'U{i:02}' for i in range(1,40)}:
        errors.append('39 audit items require explicit closure evidence')
    for item in items:
        if item.get('status') not in ['fixed','justified'] or not item.get('reason') or not item.get('evidence'):
            errors.append(f'audit {item.get("id")} unresolved')
    return result('A30',not errors,json.dumps({'median_ms':medians,'ratio':ratio,'threshold':1.15,'samples':[len(row.get('samples_ms',[])) for row in rows],
        'simulation_frozen':rows[0].get('simulation_frozen'),'load_1m':load_1m,
        'load_condition':'Recorded host loads differ; not a controlled same-load comparison or causal attribution.','scope':'back-to-back same-process render frame comparison only; frozen samples do not measure live simulation cost',
        'git_sha':manifest.get('git_sha'),'expected_git_sha':expected_git_sha,'load_status':[row.get('load_status') for row in rows],'audit_items':len(items),'errors':errors}))
