"""Read-only motion evidence from adjacent captured animation snapshots."""
import hashlib
import json
from pathlib import Path
import re
from typing import TypedDict


class MotionEvidence(TypedDict, total=False):
    """Captured snapshot columns and optional hash-bound adjacent provenance."""
    before: dict[str, list[int | float]]
    start_tick: int
    end_tick: int
    after_source: str
    provenance: str
    before_source: str
    before_sha256: str
    after_source_file: str
    after_sha256: str


def adjacent_motion(sidecar: Path) -> MotionEvidence:
    """Use only the preceding frame in this exact animation, within eight ticks."""
    data = json.loads(sidecar.read_text())
    if data.get('agent_motion_evidence'):
        return data['agent_motion_evidence']
    match = re.fullmatch(r'(a_(?:strip|evening)_)(\d+)\.entities\.json', sidecar.name)
    if not match or int(match[2]) == 0:
        return {}
    previous = sidecar.with_name(f'{match[1]}{int(match[2])-1:02d}.entities.json')
    if not previous.exists():
        return {}
    before_data = json.loads(previous.read_text())
    before, after = before_data.get('agent_snapshot', {}), data.get('agent_snapshot', {})
    interval = data.get('tick', 0) - before_data.get('tick', 0)
    if not 0 < interval <= 8:
        return {}
    for key in ('agent_ids', 'ids'):
        if key not in before or key not in after or len(before[key]) != len(set(before[key])) or set(before[key]) != set(after[key]):
            return {}
    return {'before': before, 'start_tick': before_data['tick'], 'end_tick': data['tick'],
            'after_source': 'agent_snapshot', 'provenance': 'observed_adjacent_same_animation_snapshots',
            'before_source': str(previous), 'before_sha256': hashlib.sha256(previous.read_bytes()).hexdigest(),
            'after_source_file': str(sidecar), 'after_sha256': hashlib.sha256(sidecar.read_bytes()).hexdigest()}
