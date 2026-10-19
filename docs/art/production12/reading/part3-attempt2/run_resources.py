"""Run only after the leader ends all performance holds."""
import os
from pathlib import Path
import sys
import time

CHECKOUT = Path('/Users/rexxa/work/new-world-art-prod')
EVIDENCE = Path('/Users/rexxa/work/art-production12-evidence/p3/attempt2-audit')
sys.path.insert(0, str(CHECKOUT / 'tools/art_pipeline'))
from run_activity_captures_p12 import bounded_launch, QUIET

def main():
    godot = os.environ.get('GODOT_BIN', '/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot')
    environment = os.environ.copy()
    environment['P12_RESOURCE_OUT'] = str(EVIDENCE / 'resource-replay.json')
    if Path(environment['P12_RESOURCE_OUT']).exists():
        raise RuntimeError('Retain existing replay; refusing overwrite')
    command = ['bash', str(CHECKOUT / 'tools/art_pipeline/with_window_lock.sh'), '1800',
               'bash', '-c', 'while [[ -d "$1" ]]; do sleep 30; done; shift; exec "$@"',
               'p12-resource-quiet', str(QUIET), godot, '--headless', '--path', str(CHECKOUT),
               '--script', str(EVIDENCE / 'capture_resources.gd')]
    with (EVIDENCE / 'resource-replay.log').open('x') as log:
        return bounded_launch(command, CHECKOUT, environment, log, time.time() + 1800, 1800)

if __name__ == '__main__':
    raise SystemExit(main())
