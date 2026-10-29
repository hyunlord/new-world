"""Real-renderer PlantLayer structural evidence, separate from visual acceptance."""
from __future__ import annotations

import os
import re
from pathlib import Path

from m6a_evidence import reference_json, write_json
from m6a_gate_history import run
from m6a_gate_source import Gate, need


def validate(text: str) -> int:
    checks = re.findall(r'^M6A_PLANT_ART_PASS (.+)$', text, re.MULTILINE)
    done = re.findall(r'^M6A_PLANT_ART_DONE checks=(\d+) failures=(\d+)$', text, re.MULTILINE)
    need(len(done) == 1 and done[0] == (str(len(checks)), '0') and bool(checks),
         'plant art structural checks incomplete')
    need('M6A_PLANT_ART_FAIL' not in text and 'ERROR:' not in text,
         'plant art structural process reported an error')
    required = {'data-only plant maps admitted', 'fixture content installs',
                'duplicate selected maps rejected', 'species ripe phase wins',
                'unbound category retains placeholder'}
    for phase in range(4):
        required.update(f'{label} {phase}' for label in
                        ('one actual batch phase', 'native texture phase', 'native anchor phase', 'upright UV phase',
                         'unknown species uses category phase'))
    need(required <= set(checks), 'plant art structural contract checks absent')
    need(any(check.startswith('malformed plant art rejected:') for check in checks),
         'plant art malformed-data coverage absent')
    return len(checks)


def structural_command(executable: str) -> list[str | Path]:
    # Godot's headless dummy backend does not retain MultiMesh transforms.
    # A real renderer is required to check the unchanged native-anchor assertions.
    return ['bash', 'tools/v8/run_bounded.sh', '180', executable,
        '--rendering-driver', 'opengl3', '--path', '.',
        '--script', 'res://tools/v8/m6a_plant_art_test.gd']


def structural(gate: Gate, output: Path) -> None:
    gate.verify()
    executable = os.environ.get('GODOT_BIN', '')
    need(bool(executable), 'GODOT_BIN is required for plant art structural checks')
    command = structural_command(executable)
    log = output / 'plant-art-structural.log'
    run(gate.root, command, log)
    count = validate(log.read_text())
    gate.verify()
    write_json(output / 'plant-art-structural.json', {
        'schema_version': 1, 'kind': 'm6a_plant_art_structural_v1',
        'head': gate.head, 'command': [str(arg) for arg in command], 'exit_code': 0,
        'status': 'PASS', 'checks': count, 'visual_status': 'NOT_EVALUATED',
        'execution_mode': 'windowed_real_renderer', 'rendering_driver': 'opengl3',
        'log': reference_json(log),
        'source_files': [reference_json(gate.root / name) for name in (
            'tools/v8/m6a_plant_art_test.gd', 'tools/v8/fixtures/m6a_plant_art.json',
            'scripts/world/art/art_data.gd', 'scripts/world/colony/plant_layer.gd',
            'scripts/ui/content_context.gd', 'assets/v8art/manifest.json')],
    })
