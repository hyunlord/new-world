# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 tools/art_pipeline/verify_draw_clock_p11.py A APRIME B
"""Fail closed before pixel comparison; run with A APRIME B directories."""
from pathlib import Path
import json
import sys

FIELDS = ('origin_frame', 'last_frame', 'manual_steps', 'step_seconds',
          'pre_draws', 'post_draws', 'automatic_loop_disabled')


def main() -> int:
    """Check every capture's complete and matching manual draw schedule."""
    if len(sys.argv) != 4:
        print('usage: verify_draw_clock_p11.py A APRIME B', file=sys.stderr)
        return 2
    try:
        for framing in ('P1', 'P2', 'P3', 'P4', 'P5', 'P6'):
            schedules = []
            for directory in sys.argv[1:]:
                stamp = json.loads((Path(directory)/f'{framing}.json').read_text())
                clock = stamp['draw_clock']
                if clock['valid'] is not True or clock['automatic_loop_disabled'] is not True:
                    raise RuntimeError(f'{directory}/{framing}: draw clock invalid')
                if clock['manual_steps'] != clock['last_frame'] - clock['origin_frame'] + 1:
                    raise RuntimeError(f'{directory}/{framing}: discontinuous render schedule')
                if clock['manual_steps'] != clock['pre_draws'] or clock['pre_draws'] != clock['post_draws']:
                    raise RuntimeError(f'{directory}/{framing}: unexpected render callbacks')
                if clock['last_frame'] != stamp['capture_frame'] or clock['step_seconds'] != 1/60:
                    raise RuntimeError(f'{directory}/{framing}: capture/render phase mismatch')
                schedules.append(tuple(clock[field] for field in FIELDS))
            if schedules[0] != schedules[1] or schedules[0] != schedules[2]:
                raise RuntimeError(f'{framing}: render schedules differ')
    except (OSError, ValueError, KeyError, TypeError, RuntimeError) as error:
        print(str(error), file=sys.stderr)
        return 1
    print('PASS: all six draw-clock schedules complete and equal; pixel comparison still required')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
