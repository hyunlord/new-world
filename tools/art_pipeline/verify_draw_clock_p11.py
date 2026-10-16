# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 tools/art_pipeline/verify_draw_clock_p11.py A APRIME B
from __future__ import annotations

import json
import math
import sys
from collections.abc import Mapping
from pathlib import Path
from typing import Final

from summarize_perf_p11 import EvidenceError, Json, mapping, number, sequence

STEP: Final = 1 / 60
TOLERANCE: Final = 1e-9
FIELDS: Final = (
    'schema_version', 'origin_frame', 'last_frame', 'manual_steps',
    'automatic_steps', 'step_seconds', 'pre_draws', 'post_draws',
    'automatic_loop_disabled', 'initial_engine_frames_drawn',
    'engine_frames_drawn', 'process_samples', 'draw_events',
)


def integer(value: Json) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or value < 0:
        raise EvidenceError('Expected a nonnegative integer')
    return value


def fixed_step(value: Json) -> float:
    result = number(value)
    if not math.isclose(result, STEP, rel_tol=0, abs_tol=TOLERANCE):
        raise EvidenceError('Observed step differs from 1/60')
    return result


def validate(stamp: Mapping[str, Json]) -> tuple[Json, ...]:
    clock = mapping(stamp['draw_clock'])
    if clock['schema_version'] != 2 or clock['valid'] is not True or clock['errors'] != [] or clock['automatic_loop_disabled'] is not True:
        raise EvidenceError('Invalid draw-clock evidence')
    if integer(clock['initial_engine_frames_drawn']) != 0 or integer(clock['origin_frame']) != 0:
        raise EvidenceError('Unobserved startup frames')
    last = integer(clock['last_frame'])
    manual = integer(clock['manual_steps'])
    automatic = integer(clock['automatic_steps'])
    pre = integer(clock['pre_draws'])
    post = integer(clock['post_draws'])
    fixed_step(clock['step_seconds'])
    if manual != last + 1 or last != integer(stamp['capture_frame']):
        raise EvidenceError('Discontinuous manual draw schedule or capture phase')
    if pre != manual + automatic or post != pre or integer(clock['engine_frames_drawn']) != automatic:
        raise EvidenceError('Unaccounted rendering callbacks or engine draws')
    samples = [mapping(row) for row in sequence(clock['process_samples'])]
    if len(samples) != last + 1:
        raise EvidenceError('Missing process delta samples')
    for index, sample in enumerate(samples):
        if integer(sample['frame']) != index or number(sample['time_scale']) != 1:
            raise EvidenceError('Process samples are discontinuous or scaled')
        fixed_step(sample['delta'])
    events = [mapping(row) for row in sequence(clock['draw_events'])]
    if len(events) != pre:
        raise EvidenceError('Incomplete draw event sequence')
    manual_frames: list[int] = []
    automatic_count = 0
    previous = -1
    for event in events:
        frame = integer(event['process_frame'])
        if frame < previous or frame > last or number(event['time_scale']) != 1:
            raise EvidenceError('Draw event frame order or time scale invalid')
        previous = frame
        step = fixed_step(event['step_seconds'])
        match event['kind']:
            case 'manual':
                manual_frames.append(frame)
            case 'automatic':
                automatic_count += 1
                if step != number(samples[frame]['delta']):
                    raise EvidenceError('Automatic draw lacks matching actual process delta')
            case _:
                raise EvidenceError('Unknown draw event kind')
    if manual_frames != list(range(last + 1)) or automatic_count != automatic:
        raise EvidenceError('Incomplete or duplicate manual/automatic draws')
    return tuple(clock[field] for field in FIELDS)


def main() -> int:
    if len(sys.argv) != 4:
        print('usage: verify_draw_clock_p11.py A APRIME B', file=sys.stderr)
        return 2
    try:
        for framing in ('P1', 'P2', 'P3', 'P4', 'P5', 'P6'):
            schedules: list[tuple[Json, ...]] = []
            for directory in sys.argv[1:]:
                path = Path(directory) / f'{framing}.json'
                stamp: Json = json.loads(path.read_text())
                try:
                    schedules.append(validate(mapping(stamp)))
                except (ValueError, KeyError, TypeError) as error:
                    raise EvidenceError(f'{path}: {error}') from error
            if schedules[0] != schedules[1] or schedules[0] != schedules[2]:
                raise EvidenceError(f'{framing}: complete render event sequences differ')
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(str(error), file=sys.stderr)
        return 1
    print('PASS: all six complete render event sequences equal; pixel comparison still required')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
