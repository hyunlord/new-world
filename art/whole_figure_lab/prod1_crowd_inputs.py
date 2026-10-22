# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
"""Prepare a real crowd workload from accepted complete looks, without lab quotas."""
from __future__ import annotations

import json
import sys
from pathlib import Path

from .build import sha
from .prod1 import FACINGS, accepted, ground, load
from .samples import STATES


def build(route: Path, output: Path) -> None:
    """Require at least one accepted ten-frame look; never use rejected substitutes."""
    people = accepted(route)
    if not people:
        raise ValueError('Crowd requires at least one accepted complete ten-frame look')
    (output / 'textures').mkdir(parents=True, exist_ok=True)
    sprites = []
    hashes = {}
    activities = ('idle', 'walk', 'work', 'carry', 'sleep')
    for number, person in enumerate(people, 1):
        for facing in FACINGS:
            files = []
            for state in STATES:
                source = route / person.look / facing / f'{state}.png'
                filename = f'p1_{number:02d}_{facing}_{state}.png'
                load(route, person.look, facing, state).save(output / 'textures' / filename)
                files.append(filename)
                hashes[str(source.resolve())] = sha(source)
                hashes[str(source.with_suffix('.json').resolve())] = sha(source.with_suffix('.json'))
            for activity in activities:
                sprites.append({'number': number, 'look': person.look, 'facing': facing,
                                'activity': activity, 'files': files})
    ground('meadow').save(output / 'meadow.png')
    record = {'dataset_status': 'ACCEPTED_SUBSET_CROWD_ONLY', 'formal_R1_eligible': False,
              'accepted_looks': [p.look for p in people], 'input_hashes': hashes,
              'sprites': sprites, 'count': 200,
              'limitation': 'Repeated accepted subset; no demographic lab quota or R1 certification'}
    (output / 'crowd_manifest.json').write_text(json.dumps(record, indent=2) + '\n')
    (output / 'DATASET_STATUS.txt').write_text('ACCEPTED_SUBSET_CROWD_ONLY\nformal_R1_eligible=false\n')


if __name__ == '__main__':
    if len(sys.argv) != 3:
        raise SystemExit('Usage: python3 -m art.whole_figure_lab.prod1_crowd_inputs ROUTE OUTPUT')
    build(Path(sys.argv[1]), Path(sys.argv[2]))
