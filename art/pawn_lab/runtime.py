# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m art.pawn_lab.runtime LAB_OUTPUT [SECONDS]
"""Run standalone windowed Godot and require real renderer completion evidence."""
import json
import os
import subprocess
import sys
from pathlib import Path


def run(lab: Path, seconds: float) -> None:
    repo = Path(__file__).resolve().parents[2]
    destination = lab / 'crowd'
    destination.mkdir(exist_ok=True)
    command = ['bash', str(repo / 'tools/v8/run_bounded.sh'), str(int(seconds + 25)),
               os.environ.get('GODOT_BIN', '/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot'),
               '--path', str(repo / 'art/proto'), '--rendering-driver', 'opengl3',
               'res://scenes/pawn_lab.tscn', '--', '--manifest', str((lab / 'godot_manifest.json').resolve()),
               '--out', str(destination.resolve()), '--seconds', str(seconds)]
    (destination / 'command.json').write_text(json.dumps(command, indent=2) + '\n')
    with (destination / 'godot.log').open('w') as log:
        result = subprocess.run(command, stdout=log, stderr=subprocess.STDOUT, check=False, timeout=seconds + 30)
    log_text = (destination / 'godot.log').read_text()
    if result.returncode or 'PAWN_LAB_DONE' not in log_text or any(text in log_text for text in ('ERROR:', 'SCRIPT ERROR', 'Parse Error')):
        raise RuntimeError(f'Godot failed; see {destination / "godot.log"}')
    report = json.loads((destination / 'runtime.json').read_text())
    if report['headless'] or not report['window_visible'] or not report['all_sprite_bounds_in_viewport']:
        raise RuntimeError('Window or full sprite placement not verified')
    print(json.dumps({key: report[key] for key in ('elapsed_seconds', 'rendered_frames', 'rendered_fps')}))


if __name__ == '__main__':
    run(Path(sys.argv[1]), float(sys.argv[2]) if len(sys.argv) > 2 else 60.0)
